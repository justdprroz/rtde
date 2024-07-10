use std::mem::size_of;
use std::ptr::null_mut;

use x11::xlib::{AnyButton, AnyModifier, Atom, Button1, IsViewable};
use x11::xlib::{PropModeReplace, XA_CARDINAL};

use crate::structs::*;
use crate::utils::*;
use crate::wrap::xlib::*;

use x11::xlib::Button3;
use x11::xlib::CWBorderWidth;
use x11::xlib::ClientMessage;
use x11::xlib::CurrentTime;
use x11::xlib::DestroyAll;
use x11::xlib::EnterWindowMask;
use x11::xlib::FocusChangeMask;
use x11::xlib::Mod4Mask as ModKey;
use x11::xlib::NoEventMask;
use x11::xlib::PMaxSize;
use x11::xlib::PMinSize;
use x11::xlib::PropModeAppend;
use x11::xlib::PropertyChangeMask;
use x11::xlib::RevertToPointerRoot;
use x11::xlib::StructureNotifyMask;
use x11::xlib::SubstructureNotifyMask;
use x11::xlib::Success;
use x11::xlib::XGetWindowProperty;
use x11::xlib::XMotionEvent;
use x11::xlib::XWindowAttributes;
use x11::xlib::XA_ATOM;
use x11::xlib::XA_WINDOW;

pub fn update_client_desktop(app: &mut Application, win: u64, desk: u64) {
    change_property(
        app.core.display,
        win,
        app.atoms.net_wm_desktop,
        XA_CARDINAL,
        32,
        PropModeReplace,
        &desk as *const u64 as *mut u8,
        1,
    );
}

/// Fetches clients that are already present
pub fn scan(app: &mut Application) {
    // let runtime = &mut app.runtime;
    log!("|===== scan =====");
    let (mut rw, _, wins) = query_tree(app.core.display, app.core.root_win);

    log!("|- Found {} window(s) that are already present", wins.len());

    for win in wins {
        log!("   |- Checking window {win}");
        let res = get_window_attributes(app.core.display, win);
        if let Some(wa) = res {
            if wa.override_redirect != 0
                || get_transient_for_hint(app.core.display, win, &mut rw) != 0
            {
                log!("      |- Window is transient. Skipping");
                continue;
            }
            if wa.map_state == IsViewable {
                log!("      |- Window is viewable. Managing");
                manage_client(app, win);
                continue;
            }
        }
        log!("      |- Can't manage window");
    }
}

/// Gets name from x server for specified window and undates it in struct
pub fn update_client_name(app: &mut Application, win: u64) {
    // Get name property and dispatch Option<>
    let name = match get_text_property(app.core.display, win, app.atoms.net_wm_name) {
        Some(name) => name,
        None => "_".to_string(),
    };

    // Get trackers for specified window and change name
    if let Some((s, w, c)) = find_window_indexes(app, win) {
        app.runtime.screens[s].workspaces[w].clients[c].window_name = name;
    }
}
/// Returns name of specified client
pub fn get_client_name(app: &mut Application, win: u64) -> String {
    if let Some((s, w, c)) = find_window_indexes(app, win) {
        app.runtime.screens[s].workspaces[w].clients[c]
            .window_name
            .clone()
    } else {
        "Unmanaged Window".to_string()
    }
}

/// Adds client to runtime and configures it if needed
pub fn manage_client(app: &mut Application, win: u64) {
    let wa;

    // If thes is no proper window attributes - exit
    if let Some(a) = get_window_attributes(app.core.display, win) {
        if a.override_redirect == 0 {
            wa = a;
        } else {
            return;
        };
    } else {
        return;
    }

    // Is window is manager - exit
    if find_window_indexes(app, win).is_some() {
        return;
    }

    if get_atom_prop(app, win, app.atoms.net_wm_window_type) == app.atoms.net_wm_window_type_dock {
        attach_dock(app, &wa, win);
        map_window(app.core.display, win);
        select_input(
            app.core.display,
            win,
            StructureNotifyMask | SubstructureNotifyMask,
        );
        return;
    }

    // Create client
    let mut c: Client = Client::default();
    let mut trans = 0;

    // Set essential client fields
    c.window_id = win;
    c.w = wa.width as u32;
    c.h = wa.height as u32;
    c.x = wa.x
        + app.runtime.screens[app.runtime.current_screen]
            .bar_offsets
            .left as i32;
    c.y = wa.y
        + app.runtime.screens[app.runtime.current_screen]
            .bar_offsets
            .up as i32;
    c.visible = true;

    log!("Client: {:?}", c);

    let _reserved = get_transient_for_hint(app.core.display, win, &mut trans);

    let state = get_atom_prop(app, win, app.atoms.net_wm_state);
    let wtype = get_atom_prop(app, win, app.atoms.net_wm_window_type);

    update_normal_hints(app, &mut c);

    log!("Client: {:?}", c);

    if state == app.atoms.net_wm_fullscreen {
        c.floating = true;
        c.fullscreen = true;
    }
    if wtype == app.atoms.net_wm_window_type_dialog {
        c.floating = true;
    }

    if !c.floating {
        c.floating = c.fixed || trans != 0;
    }

    log!(
        "Window: {} floating: {}, fixed: {}",
        c.window_id,
        c.floating,
        c.fixed
    );

    // Set input mask
    select_input(
        app.core.display,
        win,
        EnterWindowMask | FocusChangeMask | PropertyChangeMask | StructureNotifyMask,
    );
    // grab_button(app.core.display, win, AnyButton as u32, AnyModifier);

    // Set previous client border to normal
    if let Some(cw) = get_current_client_id(app) {
        set_window_border(
            app.core.display,
            cw,
            argb_to_int(app.config.normal_border_color),
        );
    }

    let client_desktop: Option<u64>;
    unsafe {
        let mut actual_type: Atom = 0;
        let mut actual_format: i32 = 0;
        let mut nitems: u64 = 0;
        let mut bytes_after: u64 = 0;
        let mut prop: *mut u8 = std::ptr::null_mut();
        XGetWindowProperty(
            app.core.display,
            win,
            app.atoms.net_wm_desktop,
            0,
            size_of::<Atom>() as i64,
            0,
            XA_CARDINAL,
            &mut actual_type as *mut Atom,
            &mut actual_format as *mut i32,
            &mut nitems as *mut u64,
            &mut bytes_after as *mut u64,
            &mut prop as *mut *mut u8,
        );
        client_desktop = if actual_type != 0 {
            log!("   |- Window property:");
            log!(
                "      |- Type: {}",
                get_atom_name(app.core.display, actual_type)
            );
            log!("      |- Items: {}", nitems);
            log!("      |- Format: {}", actual_format);
            log!("      |- Data: {}", *prop as u64);
            Some(*prop as u64)
        } else {
            None
        }
    }

    // Get current workspace
    let (client_screen, client_workspace) = match client_desktop {
        Some(d) => {
            let s = d as usize / 10;
            let w = d as usize % 10;
            if s < app.runtime.screens.len() && w < app.runtime.screens[s].workspaces.len() {
                (s, w)
            } else {
                (app.runtime.current_screen, app.runtime.current_workspace)
            }
        }
        None => (app.runtime.current_screen, app.runtime.current_workspace),
    };

    let w = &mut app.runtime.screens[client_screen].workspaces[client_workspace];

    // Update client tracker
    w.current_client = Some(w.clients.len());
    app.runtime.current_client = w.current_client;
    // Push to stack
    w.clients.push(c);

    // Add window to wm _NET_CLIENT_LIST
    change_property(
        app.core.display,
        app.core.root_win,
        app.atoms.net_client_list,
        XA_WINDOW,
        32,
        PropModeAppend,
        &win as *const u64 as *mut u8,
        1,
    );

    let cur_workspace: usize = client_workspace + client_screen * 10;

    update_client_desktop(app, win, cur_workspace as u64);

    let mut wc = x11::xlib::XWindowChanges {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
        border_width: app.config.border_size as i32,
        sibling: 0,
        stack_mode: 0,
    };
    configure_window(app.core.display, win, CWBorderWidth as u32, &mut wc);
    set_window_border(
        app.core.display,
        win,
        argb_to_int(app.config.active_border_color),
    );
    update_client_name(app, win);
    raise_window(app.core.display, win);
    set_input_focus(app.core.display, win, RevertToPointerRoot, CurrentTime);

    let data: [i64; 2] = [1, 0];

    change_property(
        app.core.display,
        win,
        app.atoms.wm_state,
        app.atoms.wm_state,
        32,
        PropModeReplace,
        &data as *const [i64; 2] as *mut u8,
        2,
    );

    // Arrange current workspace
    arrange(app);
    // Finish mapping
    map_window(app.core.display, win);
    log!("   |- Mapped window");
}

pub fn attach_dock(app: &mut Application, wa: &XWindowAttributes, win: u64) {
    let dx = wa.x as i64;
    let dy = wa.y as i64;
    let dw = wa.width as usize;
    let dh = wa.height as usize;
    for screen in &mut app.runtime.screens {
        if dx >= screen.x && dx < (screen.x + screen.width) {
            if dy >= screen.y && dy < (screen.y + screen.height) {
                let mut ba = screen.bar_offsets;
                // Found corresponding screen
                if dw > dh {
                    // dock is horizontal
                    if dy == screen.y {
                        // dock is on the top
                        ba.up = dh;
                    } else {
                        // dock is on the bottom
                        ba.down = dh;
                    }
                } else {
                    // dock is vertical
                    if dx == screen.x {
                        // dock is on the left
                        ba.left = dw;
                    } else {
                        // dock is on the right
                        ba.right = dw;
                    }
                }
                screen.bar_offsets = ba;
                app.runtime.bars.push(Bar {
                    window_id: win,
                    x: dx,
                    y: dy,
                    w: dw,
                    h: dh,
                });
                arrange(app);
                break;
            }
        }
    }
}

pub fn detach_dock(app: &mut Application, win: u64) {
    log!("   |- Detaching dock");
    let b = match app.runtime.bars.iter().find(|b| b.window_id == win) {
        Some(b) => b.clone(),
        None => return,
    };
    app.runtime.bars.retain(|b| b.window_id != win);
    let dx = b.x as i64;
    let dy = b.y as i64;
    let dw = b.w as usize;
    let dh = b.h as usize;
    log!("{} {} {} {}", dx, dy, dw, dh);
    for screen in &mut app.runtime.screens {
        if dx >= screen.x && dx < (screen.x + screen.width) {
            if dy >= screen.y && dy < (screen.y + screen.height) {
                let mut ba = screen.bar_offsets;
                // Found corresponding screen
                if dw > dh {
                    // dock is horizontal
                    if dy == screen.y {
                        // dock is on the top
                        ba.up = 0;
                    } else {
                        // dock is on the bottom
                        ba.down = 0;
                    }
                } else {
                    // dock is vertical
                    if dx == screen.x {
                        // dock is on the left
                        ba.left = 0;
                    } else {
                        // dock is on the right
                        ba.right = 0;
                    }
                }
                screen.bar_offsets = ba;
                arrange(app);
                break;
            }
        }
    }
}

pub fn update_normal_hints(app: &mut Application, c: &mut Client) {
    if let Some((sh, _)) = get_wm_normal_hints(app.core.display, c.window_id) {
        if (sh.flags & PMaxSize) != 0 {
            c.maxw = sh.max_width;
            c.maxh = sh.max_height;
        }
        if (sh.flags & PMinSize) != 0 {
            c.minw = sh.min_width;
            c.minh = sh.min_height;
        }
    }

    if c.minw != 0 && c.w < c.minw as u32 {
        c.w = c.minw as u32;
    }
    if c.minh != 0 && c.h < c.minh as u32 {
        c.h = c.minh as u32;
    }

    if c.maxw != 0 && c.maxh != 0 && c.maxw == c.minw && c.maxh == c.minh {
        c.fixed = true;
    }
}

/// Arranges windows of current workspace in specified layout
pub fn arrange(app: &mut Application) {
    log!("   |- Arranging...");
    let ws = &mut app.runtime;
    // Go thru all screens
    for screen in &mut ws.screens {
        // Usable screen
        let ba = screen.bar_offsets;
        let screen_height = screen.height - (ba.up + ba.down) as i64;
        // Gap width
        let gw = app.config.gap_width as i32;
        let bs = app.config.border_size as u32;
        // Get amount of visible not floating clients to arrange
        let stack_size = screen.workspaces[screen.current_workspace]
            .clients
            .iter()
            .filter(|&c| !c.floating)
            .count();
        // Get widths of master and stack areas
        let mut master_width = ((screen.width as i32 - gw * 3) as f64
            * screen.workspaces[screen.current_workspace].master_width)
            as u32;
        let stack_width = (screen.width as i32 - gw * 3) - master_width as i32;
        let mut master_capacity = screen.workspaces[screen.current_workspace].master_capacity;
        // if master_capacity out of stack_size bounds use whole screen for one column
        if master_capacity <= 0 || master_capacity >= stack_size as i64 {
            master_capacity = stack_size as i64;
            master_width = screen.width as u32 - gw as u32 * 2;
        }
        log!("   |- Arranging {} window", stack_size);
        // Iterate over all tileable clients structs
        for (index, client) in screen.workspaces[screen.current_workspace]
            .clients
            .iter_mut()
            .rev()
            .filter(|c| !c.floating)
            .enumerate()
        {
            if stack_size == 1 {
                // if only one window selected, just show it
                client.x = 0;
                client.y = ba.up as i32;
                client.w = screen.width as u32;
                client.h = screen_height as u32;
            } else if (index as i64) < master_capacity {
                // if master_capacity is not full put it here
                log!("      |- Goes to master");
                // some math...
                let win_height =
                    (screen_height - gw as i64 - master_capacity * gw as i64) / master_capacity;
                // Add gap offset to the left
                client.x = gw;
                // Top gap + clients with their gaps offset
                client.y = ba.up as i32 + gw + (win_height as i32 + gw) * index as i32;
                client.w = master_width - 2 * bs;
                client.h = win_height as u32 - 2 * bs
            } else {
                // otherwise put it in secondary stack
                log!("      |- Goes to stack");
                // a bit more of math...
                let win_height =
                    (screen_height - gw as i64 - (stack_size as i64 - master_capacity) * gw as i64)
                        / (stack_size as i64 - master_capacity);
                client.x = master_width as i32 + (gw * 2);
                client.y = ba.up as i32
                    + gw
                    + (win_height as i32 + gw) * (index as i64 - master_capacity) as i32;
                client.w = stack_width as u32 - 2 * bs;
                client.h = win_height as u32 - 2 * bs;
            }
        }

        // update corresponding windows
        for client in &screen.workspaces[screen.current_workspace].clients {
            if client.fullscreen {
                move_resize_window(
                    app.core.display,
                    client.window_id,
                    screen.x as i32,
                    screen.y as i32,
                    screen.width as u32,
                    screen.height as u32,
                );
                set_window_border_width(app.core.display, client.window_id, 0);
                raise_window(app.core.display, client.window_id);
            } else {
                set_window_border_width(
                    app.core.display,
                    client.window_id,
                    if stack_size > 1 || client.floating {
                        app.config.border_size as u32
                    } else {
                        0
                    },
                );
                move_resize_window(
                    app.core.display,
                    client.window_id,
                    client.x + screen.x as i32,
                    client.y + screen.y as i32,
                    client.w,
                    client.h,
                );
                if client.floating {
                    raise_window(app.core.display, client.window_id);
                }
            };
        }
    }
}

/// Returns window, workspace and client indexies for client with specified id
pub fn find_window_indexes(app: &mut Application, win: u64) -> Option<(usize, usize, usize)> {
    let ws = &mut app.runtime;
    for s in 0..ws.screens.len() {
        for w in 0..ws.screens[s].workspaces.len() {
            for c in 0..ws.screens[s].workspaces[w].clients.len() {
                if ws.screens[s].workspaces[w].clients[c].window_id == win {
                    return Some((s, w, c));
                }
            }
        }
    }
    None
}

/// Shows/Hides all windows on current workspace
pub fn show_hide_workspace(app: &mut Application) {
    let ws = &mut app.runtime;
    let window_decoration_offset = app.config.gap_width + app.config.border_size;
    // Iterate over all clients
    for client in &mut ws.screens[ws.current_screen].workspaces[ws.current_workspace].clients {
        move_resize_window(
            app.core.display,
            client.window_id,
            -(client.w as i32 + window_decoration_offset as i32),
            -(client.h as i32 + window_decoration_offset as i32),
            client.w,
            client.h,
        );
        // flip visibility state
        client.visible = !client.visible;
    }
}

/// Shifts current client tracker after destroying clients
pub fn shift_current_client(
    app: &mut Application,
    screen: Option<usize>,
    workspace: Option<usize>,
) {
    let screen = match screen {
        Some(index) => index,
        None => app.runtime.current_screen,
    };

    let workspace = match workspace {
        Some(index) => index,
        None => app.runtime.current_workspace,
    };

    let ws = &mut app.runtime;
    // Find next client
    ws.screens[screen].workspaces[workspace].current_client = {
        // Get reference to windows stack
        let clients = &ws.screens[screen].workspaces[workspace].clients;
        if clients.is_empty() {
            // None if no windows
            None
        } else {
            // Get old client index
            if let Some(cc) = ws.screens[screen].workspaces[workspace].current_client {
                // If selected client was not last do nothing
                if cc < clients.len() {
                    Some(cc)
                } else {
                    // Else set it to being last
                    Some(clients.len() - 1)
                }
            } else {
                None
            }
        }
    };
    // Only do global changes if current_workspace is equal to workspace we shifting!
    if workspace == ws.current_workspace {
        // update secondary tracker
        ws.current_client = ws.screens[screen].workspaces[workspace].current_client;
        if let Some(index) = ws.current_client {
            let win = ws.screens[screen].workspaces[workspace].clients[index].window_id;
            set_input_focus(app.core.display, win, RevertToPointerRoot, CurrentTime);
        }
        update_active_window(app);
    }
}

/// Safely sends atom to X server
pub fn send_atom(app: &mut Application, win: u64, e: x11::xlib::Atom) -> bool {
    if let Some(ps) = get_wm_protocols(app.core.display, win) {
        // If protocol not supported
        if ps.into_iter().filter(|p| *p == e).collect::<Vec<_>>().len() == 0 {
            return false;
        }
    } else {
        // If failed obtaining protocols
        return false;
    }

    // proceed to send event to window
    let mut ev = Event {
        type_: ClientMessage,
        button: None,
        crossing: None,
        key: None,
        map_request: None,
        destroy_window: None,
        motion: None,
        unmap: None,
        property: None,
        configure: None,
        configure_request: None,
        client: Some(x11::xlib::XClientMessageEvent {
            type_: ClientMessage,
            serial: 0,
            send_event: 0,
            display: null_mut(),
            window: win,
            message_type: app.atoms.wm_protocols,
            format: 32,
            data: {
                let mut d = x11::xlib::ClientMessageData::new();
                d.set_long(0, e as i64);
                d.set_long(1, CurrentTime as i64);
                d
            },
        }),
    };
    return send_event(app.core.display, win, false, NoEventMask, &mut ev);
}

// TODO: What is going on here
pub fn get_atom_prop(app: &mut Application, win: u64, prop: Atom) -> Atom {
    let mut dummy_atom: u64 = 0;
    let mut dummy_int: i32 = 0;
    let mut dummy_long: u64 = 0;
    let mut property_return: *mut u8 = std::ptr::null_mut::<u8>();
    let mut atom: u64 = 0;
    unsafe {
        if x11::xlib::XGetWindowProperty(
            app.core.display,
            win,
            prop,
            0,
            size_of::<Atom>() as i64,
            0,
            XA_ATOM,
            &mut dummy_atom as *mut u64,
            &mut dummy_int as *mut i32,
            &mut dummy_long as *mut u64,
            &mut dummy_long as *mut u64,
            &mut property_return as *mut *mut u8,
        ) == Success as i32
            && property_return as usize != 0
        {
            atom = *(property_return as *mut Atom);
            x11::xlib::XFree(property_return as *mut libc::c_void);
        }
    }
    atom
}

/// Removes window from runtime
pub fn unmanage_window(app: &mut Application, win: u64) {
    // Find trackers for window
    if let Some((s, w, c)) = find_window_indexes(app, win) {
        log!("   |- Found window {} at indexes {}, {}, {}", win, s, w, c);
        delete_property(app.core.display, win, app.atoms.net_wm_desktop);
        app.runtime.screens[s].workspaces[w].clients.remove(c);
        shift_current_client(app, Some(s), Some(w));
        arrange(app);
        update_client_list(app);
    } else {
        if app
            .runtime
            .bars
            .iter()
            .find(|b| b.window_id == win)
            .is_some()
        {
            detach_dock(app, win);
        }
    }
}

pub fn update_active_window(app: &mut Application) {
    let ws = &mut app.runtime;
    if let Some(index) = ws.current_client {
        let win =
            ws.screens[ws.current_screen].workspaces[ws.current_workspace].clients[index].window_id;
        change_property(
            app.core.display,
            app.core.root_win,
            app.atoms.net_active_window,
            XA_WINDOW,
            32,
            PropModeReplace,
            &win as *const u64 as *mut u8,
            1,
        );
    } else {
        if ws.screens[ws.current_screen].workspaces[ws.current_workspace]
            .clients
            .is_empty()
        {
            set_input_focus(
                app.core.display,
                app.core.root_win,
                RevertToPointerRoot,
                CurrentTime,
            );
            delete_property(
                app.core.display,
                app.core.root_win,
                app.atoms.net_active_window,
            );
        }
    }
}

pub fn get_current_client_id(app: &mut Application) -> Option<u64> {
    let client_index = match app.runtime.current_client {
        Some(index) => index,
        None => return None,
    };

    let screen = match app.runtime.screens.get(app.runtime.current_screen) {
        Some(s) => s,
        None => return None,
    };

    let workspace = match screen.workspaces.get(app.runtime.current_workspace) {
        Some(w) => w,
        None => return None,
    };

    let client = match workspace.clients.get(client_index) {
        Some(c) => c,
        None => return None,
    };

    return Some(client.window_id);
}

pub fn update_trackers(app: &mut Application, win: u64) {
    if let Some((s, w, c)) = find_window_indexes(app, win) {
        let ws = &mut app.runtime;
        ws.current_screen = s;
        ws.current_workspace = w;
        ws.screens[s].current_workspace = w;
        ws.current_client = Some(c);
        ws.screens[s].workspaces[w].current_client = Some(c);
    };
}

pub fn spawn(app: &mut Application, cmd: String) {
    unsafe {
        match nix::unistd::fork() {
            Ok(nix::unistd::ForkResult::Parent { child: _ }) => {
                log!("     |- Spawned");
            }
            Ok(nix::unistd::ForkResult::Child) => {
                log!("     |- Hello from child)");
                if app.core.display as *mut x11::xlib::Display as usize != 0 {
                    match nix::unistd::close(x11::xlib::XConnectionNumber(app.core.display)) {
                        Ok(_) => {}
                        Err(_) => {}
                    };
                }
                let args = [
                    &std::ffi::CString::from_vec_unchecked("/usr/bin/sh".as_bytes().to_vec()),
                    &std::ffi::CString::from_vec_unchecked("-c".as_bytes().to_vec()),
                    &std::ffi::CString::from_vec_unchecked(cmd.as_bytes().to_vec()),
                ];
                let _ = nix::unistd::execvp(args[0], &args);
            }
            Err(_) => {
                log!("Fork Failed");
            }
        }
    }
}

pub fn update_client_list(app: &mut Application) {
    delete_property(
        app.core.display,
        app.core.root_win,
        app.atoms.net_client_list,
    );

    for screen in &app.runtime.screens {
        for workspace in &screen.workspaces {
            for client in &workspace.clients {
                change_property(
                    app.core.display,
                    app.core.root_win,
                    app.atoms.net_client_list,
                    XA_WINDOW,
                    32,
                    PropModeAppend,
                    &client.window_id as *const u64 as *mut u8,
                    1,
                );
            }
        }
    }
}

pub fn kill_client(app: &mut Application) {
    // Check if there any windows selected
    if let Some(index) = app.runtime.current_client {
        let id = app.runtime.screens[app.runtime.current_screen].workspaces
            [app.runtime.current_workspace]
            .clients[index]
            .window_id;
        log!("      |- Killing window {}", id);
        if !send_atom(app, id, app.atoms.wm_delete) {
            grab_server(app.core.display);
            set_close_down_mode(app.core.display, DestroyAll);
            x_kill_client(app.core.display, id);
            ungrab_server(app.core.display);
        };
    } else {
        log!("      |- No window selected");
    };
}

pub fn move_to_screen(app: &mut Application, d: ScreenSwitching) {
    // Check if window is selected
    if let Some(index) = app.runtime.current_client {
        // Get current screen index
        let mut cs = app.runtime.current_screen;
        // Update index depending on supplied direction
        cs = match d {
            ScreenSwitching::Next => (cs + 1) % app.runtime.screens.len(),
            ScreenSwitching::Previous => {
                (cs + app.runtime.screens.len() - 1) % app.runtime.screens.len()
            }
        };
        // Pop client
        let cc = app.runtime.screens[app.runtime.current_screen].workspaces
            [app.runtime.current_workspace]
            .clients
            .remove(index);
        set_window_border(
            app.core.display,
            cc.window_id,
            argb_to_int(app.config.normal_border_color),
        );

        let cur_workspace: usize = app.runtime.screens[cs].current_workspace + cs * 10;

        update_client_desktop(app, cc.window_id, cur_workspace as u64);

        // Update client tracker on current screen
        shift_current_client(app, None, None);
        // Get workspace tracker(borrow checker is really mad at me)
        let nw = app.runtime.screens[cs].current_workspace;
        // Add window to stack of another display
        app.runtime.screens[cs].workspaces[nw].clients.push(cc);
        // Arrange all monitors
        arrange(app);
    }
}

pub fn focus_on_screen_index(app: &mut Application, n: usize) {
    log!("Focusing on screen");
    if let Some(cw) = get_current_client_id(app) {
        log!("unfocusing {}", cw);
        set_window_border(
            app.core.display,
            cw,
            argb_to_int(app.config.normal_border_color),
        );
        unfocus(app, cw);
    }
    // Change trackers
    app.runtime.current_screen = n;
    app.runtime.current_workspace =
        app.runtime.screens[app.runtime.current_screen].current_workspace;
    app.runtime.current_client = app.runtime.screens[app.runtime.current_screen].workspaces
        [app.runtime.current_workspace]
        .current_client;
    if let Some(index) = app.runtime.current_client {
        let win = app.runtime.screens[app.runtime.current_screen].workspaces
            [app.runtime.current_workspace]
            .clients[index]
            .window_id;
        set_input_focus(app.core.display, win, RevertToPointerRoot, CurrentTime);
    }
    update_active_window(app);
    if let Some(cw) = get_current_client_id(app) {
        set_window_border(
            app.core.display,
            cw,
            argb_to_int(app.config.active_border_color),
        );
    }
    let w: u64 = n as u64 * 10 + app.runtime.screens[n].current_workspace as u64;
    change_property(
        app.core.display,
        app.core.root_win,
        app.atoms.net_current_desktop,
        XA_CARDINAL,
        32,
        PropModeReplace,
        &w as *const u64 as *mut u64 as *mut u8,
        1,
    );
}

pub fn focus_on_screen(app: &mut Application, d: ScreenSwitching) {
    // Get current screen
    let mut cs = app.runtime.current_screen;
    // Update it
    cs = match d {
        ScreenSwitching::Next => (cs + 1) % app.runtime.screens.len(),
        ScreenSwitching::Previous => {
            (cs + app.runtime.screens.len() - 1) % app.runtime.screens.len()
        }
    };
    focus_on_screen_index(app, cs);
}

pub fn move_to_workspace(app: &mut Application, n: u64) {
    log!("   |- Got `MoveToWorkspace` Action ");
    // Check if moving to another workspace
    if n as usize != app.runtime.current_workspace {
        // Check if any client is selected
        if let Some(index) = app.runtime.current_client {
            // Pop current client
            let mut cc = app.runtime.screens[app.runtime.current_screen].workspaces
                [app.runtime.current_workspace]
                .clients
                .remove(index);
            set_window_border(
                app.core.display,
                cc.window_id,
                argb_to_int(app.config.normal_border_color),
            );
            let cur_workspace: usize = n as usize + app.runtime.current_screen * 10;

            update_client_desktop(app, cc.window_id, cur_workspace as u64);

            // Update current workspace layout
            arrange(app);
            // Move window out of view
            move_resize_window(
                app.core.display,
                cc.window_id,
                -(cc.w as i32),
                -(cc.h as i32),
                cc.w,
                cc.h,
            );
            cc.visible = !cc.visible;
            // Update tracker
            shift_current_client(app, None, None);
            // Add client to choosen workspace (will be arranged later)
            app.runtime.screens[app.runtime.current_screen].workspaces[n as usize]
                .clients
                .push(cc);
        }
    }
}

pub fn focus_on_workspace(app: &mut Application, n: u64, r: bool) {
    let n = if !r {
        focus_on_screen_index(app, n as usize / 10);
        n % 10
    } else {
        n
    };
    log!("   |- Got `FocusOnWorkspace` Action");
    // Check is focusing on another workspace
    if n as usize != app.runtime.current_workspace {
        // Hide current workspace
        show_hide_workspace(app);
        // unfocus current win
        if let Some(cw) = get_current_client_id(app) {
            set_window_border(
                app.core.display,
                cw,
                argb_to_int(app.config.normal_border_color),
            );
            unfocus(app, cw);
        }
        // Update workspace index
        app.runtime.current_workspace = n as usize;
        app.runtime.screens[app.runtime.current_screen].current_workspace = n as usize;

        let w = n + app.runtime.current_screen as u64 * 10;

        change_property(
            app.core.display,
            app.core.root_win,
            app.atoms.net_current_desktop,
            XA_CARDINAL,
            32,
            PropModeReplace,
            &w as *const u64 as *mut u64 as *mut u8,
            1,
        );
        // Update current client
        app.runtime.current_client = app.runtime.screens[app.runtime.current_screen].workspaces
            [app.runtime.current_workspace]
            .current_client;
        if let Some(cw) = get_current_client_id(app) {
            set_window_border(
                app.core.display,
                cw,
                argb_to_int(app.config.active_border_color),
            );
        }
        // Show current client
        show_hide_workspace(app);
        // Arrange update workspace
        arrange(app);
        if let Some(index) = app.runtime.current_client {
            let win = app.runtime.screens[app.runtime.current_screen].workspaces
                [app.runtime.current_workspace]
                .clients[index]
                .window_id;
            set_input_focus(app.core.display, win, RevertToPointerRoot, CurrentTime);
        }
        update_active_window(app);
    }
}

pub fn update_master_width(app: &mut Application, w: f64) {
    // Update master width
    app.runtime.screens[app.runtime.current_screen].workspaces[app.runtime.current_workspace]
        .master_width += w;
    // Rearrange windows
    arrange(app);
}

pub fn update_master_capacity(app: &mut Application, i: i64) {
    // Change master size
    app.runtime.screens[app.runtime.current_screen].workspaces[app.runtime.current_workspace]
        .master_capacity += i;
    // Rearrange windows
    arrange(app);
}

pub fn toggle_float(app: &mut Application) {
    if let Some(c) = app.runtime.current_client {
        let state = app.runtime.screens[app.runtime.current_screen].workspaces
            [app.runtime.current_workspace]
            .clients[c]
            .floating;
        app.runtime.screens[app.runtime.current_screen].workspaces[app.runtime.current_workspace]
            .clients[c]
            .floating = !state;
        arrange(app);
    }
}

pub fn focus(app: &mut Application, win: u64) {
    set_window_border(
        app.core.display,
        win,
        argb_to_int(app.config.active_border_color),
    );
    update_trackers(app, win);
    update_active_window(app);
    set_input_focus(app.core.display, win, RevertToPointerRoot, CurrentTime);
    grab_button(app.core.display, win, Button1, ModKey);
    grab_button(app.core.display, win, Button3, ModKey);

    let w = app.runtime.current_workspace + app.runtime.current_screen * 10;

    change_property(
        app.core.display,
        app.core.root_win,
        app.atoms.net_current_desktop,
        XA_CARDINAL,
        32,
        PropModeReplace,
        &w as *const usize as *mut usize as *mut u8,
        1,
    );
}

pub fn unfocus(app: &mut Application, win: u64) {
    set_window_border(
        app.core.display,
        win,
        argb_to_int(app.config.normal_border_color),
    );
    ungrab_button(app.core.display, AnyButton as u32, AnyModifier, win);
}

pub fn move_mouse(app: &mut Application, me: XMotionEvent) {
    log!("    |- Moving");

    let mw: u64 = app.runtime.mouse_state.win;

    if let Some((s, w, c)) = find_window_indexes(app, mw) {
        let sx = app.runtime.screens[s].x as i32;
        let sy = app.runtime.screens[s].y as i32;
        let sw = app.runtime.screens[s].width as i32;
        let sh = app.runtime.screens[s].height as i32;
        let (mx, my) = (me.x_root as i64, me.y_root as i64);
        let (px, py) = app.runtime.mouse_state.pos;
        let (dx, dy) = (mx - px, my - py);

        let (sbl, sbu, sbr, sbd) = {
            let s = &app.runtime.screens[s];
            let sbl = s.x + s.bar_offsets.left as i64;
            let sbu = s.y + s.bar_offsets.up as i64;
            let sbr = s.x + s.width - s.bar_offsets.right as i64;
            let sbd = s.y + s.height - s.bar_offsets.down as i64;
            (sbl, sbu, sbr, sbd)
        };

        let cc = &mut app.runtime.screens[s].workspaces[w].clients[c];
        let mut nx = cc.x + dx as i32;
        let mut ny = cc.y + dy as i32;

        // Stick to screen border
        let stick = 50;
        let unstick = 30 as i64;

        if (nx < sbl as i32 + stick)
            && (dx < 0 && nx > (sbl - unstick) as i32
                || nx > sbl as i32 && nx < (sbl + unstick) as i32)
        {
            nx = sbl as i32;
        }
        if (nx + cc.w as i32) > sbr as i32 - stick
            && (dx > 0 && (nx + cc.w as i32) < (sbr + unstick) as i32
                || (nx + cc.w as i32) < sbr as i32 && (nx + cc.w as i32) > (sbr - unstick) as i32)
        {
            nx = sbr as i32 - cc.w as i32 - 2 * app.config.border_size as i32;
        }

        if ny < sbu as i32 + stick
            && (dy < 0 && ny > (sbu - unstick) as i32
                || ny > sbu as i32 && ny < (sbu + unstick) as i32)
        {
            ny = sbu as i32;
        }
        if (ny + cc.h as i32) > sbd as i32 - stick
            && (dy > 0 && (ny + cc.h as i32) < (sbd + unstick) as i32
                || (ny + cc.h as i32) < sbd as i32 && (ny + cc.h as i32) > (sbd - unstick) as i32)
        {
            ny = sbd as i32 - cc.h as i32 - 2 * app.config.border_size as i32;
        }

        // Unstick from border

        if cc.x != nx {
            app.runtime.mouse_state.pos.0 = mx;
        }

        if cc.y != ny {
            app.runtime.mouse_state.pos.1 = my;
        }

        cc.x = nx;
        cc.y = ny;

        if cc.x < 0 {
            cc.x = 0;
        }
        if cc.y < 0 {
            cc.y = 0;
        }
        if (cc.x + cc.w as i32) > sw {
            cc.x = sw - cc.w as i32;
        }
        if (cc.y + cc.h as i32) > sh {
            cc.y = sh - cc.h as i32;
        }

        move_resize_window(app.core.display, mw, cc.x + sx, cc.y + sy, cc.w, cc.h);
    }
}

pub fn resize_mouse(app: &mut Application, me: XMotionEvent) {
    log!("    |- Resizing");

    let (mx, my) = (me.x_root as i64, me.y_root as i64);

    let (px, py) = app.runtime.mouse_state.pos;
    let (dx, dy) = (mx - px, my - py);
    app.runtime.mouse_state.pos = (mx, my);
    let mw: u64 = app.runtime.mouse_state.win;

    if let Some((s, w, c)) = find_window_indexes(app, mw) {
        let sox = (app.runtime.screens[s].x) as i32;
        let soy = (app.runtime.screens[s].y) as i32;
        let cc = &mut app.runtime.screens[s].workspaces[w].clients[c];
        let mut nw = cc.w as i32;
        let mut nh = cc.h as i32;
        if (nw + dx as i32) > cc.minw {
            if cc.maxw == 0 || cc.maxw > 0 && (nw + dx as i32) < cc.maxw {
                nw += dx as i32;
            }
        };
        if (nh + dy as i32) > cc.minh {
            if cc.maxh == 0 || cc.maxh > 0 && (nh + dy as i32) < cc.maxh {
                nh += dy as i32;
            }
        }
        cc.w = nw as u32;
        cc.h = nh as u32;
        move_resize_window(app.core.display, mw, cc.x + sox, cc.y + soy, cc.w, cc.h);
    }
}

pub fn screen_mouse(app: &mut Application, me: XMotionEvent) {
    log!("    |- Moving on root");

    let (mx, my) = (me.x_root as i64, me.y_root as i64);

    for screen in &app.runtime.screens {
        if screen.x <= mx
            && mx < screen.x + screen.width
            && screen.y <= my
            && my < screen.y + screen.height
        {
            // Update trackers
            app.runtime.current_screen = screen.number as usize;
            app.runtime.current_workspace =
                app.runtime.screens[app.runtime.current_screen].current_workspace;
            app.runtime.current_client = app.runtime.screens[app.runtime.current_screen].workspaces
                [app.runtime.current_workspace]
                .current_client;
            let w = app.runtime.current_workspace + app.runtime.current_screen * 10;

            change_property(
                app.core.display,
                app.core.root_win,
                app.atoms.net_current_desktop,
                XA_CARDINAL,
                32,
                PropModeReplace,
                &w as *const usize as *mut usize as *mut u8,
                1,
            );
        }
    }
}
