//! Main windows manager logic processed as response to events

use std::mem::size_of;

use x11::xlib::{
    AnyButton, AnyModifier, Atom, Button1, Button3, Mod4Mask as ModKey, PropModeReplace,
    XA_CARDINAL,
};

use crate::structs::*;
use crate::utils::*;
use crate::wrap::xlib::*;

use x11::xlib::CWBorderWidth;
use x11::xlib::CurrentTime;
use x11::xlib::DestroyAll;
use x11::xlib::EnterWindowMask;
use x11::xlib::FocusChangeMask;
use x11::xlib::PropModeAppend;
use x11::xlib::PropertyChangeMask;
use x11::xlib::RevertToPointerRoot;
use x11::xlib::StructureNotifyMask;
use x11::xlib::SubstructureNotifyMask;
use x11::xlib::XGetWindowProperty;
use x11::xlib::XWindowAttributes;
use x11::xlib::XA_WINDOW;

/// Add client to runtime
///
/// 1. Get window attributes
///     * Exit if no proper attributes or if `override_redirect` is set
/// 2. Check if already managed
///     * Return
/// 3. Check if window is dock and setup
///     * Call [`attach_dock`]
///     * Map window
///     * Return
/// 4. Create client and setup essential fields
/// 5. Get properties
/// 6. Update hints by running [`update_normal_hints`]
/// 7. Set flags
/// 8. Set input mask for events
/// 9. set previously active client border to normal
/// 10. Get desktop info left from previous wm session
/// 11. Find where to place window
/// 12. Add to stack
/// 13. Update client list & desktops
/// 14. Configure window
/// 15. Arrange clients
/// 16. Map window
pub fn manage_client(app: &mut Application, win: u64) {
    // 1. Get attributes
    let wa;
    if let Some(a) = get_window_attributes(app.core.display, win) {
        if a.override_redirect == 0 {
            wa = a;
        } else {
            return;
        };
    } else {
        return;
    }

    // 2. Check managed
    if find_window_indexes(app, win).is_some() {
        return;
    }

    // 3. Check if dock
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

    // 4. Create client
    let mut c: Client = Client::default();
    let mut trans = 0;
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

    // 5. Properties
    let _reserved = get_transient_for_hint(app.core.display, win, &mut trans);
    let state = get_atom_prop(app, win, app.atoms.net_wm_state);
    let wtype = get_atom_prop(app, win, app.atoms.net_wm_window_type);

    // 6. Update hints
    update_normal_hints(app, &mut c);

    // 7. Set flags
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

    // 8. Set input mask for events
    select_input(
        app.core.display,
        win,
        EnterWindowMask | FocusChangeMask | PropertyChangeMask | StructureNotifyMask,
    );

    // 9. set previously active client border to normal
    if let Some(cw) = get_current_client_id(app) {
        set_window_border(
            app.core.display,
            cw,
            argb_to_int(app.config.normal_border_color),
        );
    }

    // 10. Get desktop
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

    // 11. Calculate window place
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

    // 12. Add window to stack
    w.current_client = Some(w.clients.len());
    app.runtime.current_client = w.current_client;
    w.clients.push(c);

    // 13. Update client list & window desktop
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

    // 14. Configure window
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

    // 15. Arrange current workspace
    arrange(app);
    // 16. Tag window as mapped
    map_window(app.core.display, win);
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

/// Arrange windows of current workspace in specified layout
/// TODO DOCUMENTATION
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

/// Spawn new program by forking
///
/// 1. Fork
/// 2. For child close connections from Parent
/// 3. Spawn program using sh
pub fn spawn(app: &mut Application, cmd: String) {
    unsafe {
        match nix::unistd::fork() {
            Ok(nix::unistd::ForkResult::Parent { child: _ }) => {}
            Ok(nix::unistd::ForkResult::Child) => {
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
            Err(_) => {}
        }
    }
}

/// Kill active window
/// 1. Check if there is focused client
/// 2. Ask client to close
/// 3. Forcefully close client
pub fn kill_client(app: &mut Application) {
    // 1. Check
    if let Some(index) = app.runtime.current_client {
        let id = app.runtime.screens[app.runtime.current_screen].workspaces
            [app.runtime.current_workspace]
            .clients[index]
            .window_id;
        // 2. Ask
        if !send_atom(app, id, app.atoms.wm_delete) {
            // 3. Close
            grab_server(app.core.display);
            set_close_down_mode(app.core.display, DestroyAll);
            x_kill_client(app.core.display, id);
            ungrab_server(app.core.display);
        };
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

/// Get name from x server for specified window and undate it in struct
/// 1. Get name property
/// 2. Set window name if window is managed
pub fn update_client_name(app: &mut Application, win: u64) {
    // 1. Get
    let name = match get_text_property(app.core.display, win, app.atoms.net_wm_name) {
        Some(name) => name,
        None => "_".to_string(),
    };

    // 2. Set
    if let Some((s, w, c)) = find_window_indexes(app, win) {
        app.runtime.screens[s].workspaces[w].clients[c].window_name = name;
    }
}

/// Returns name of specified client
///
/// 1. If client is managed return its name
/// 2. If not managed return "Unmanaged Window"
pub fn get_client_name(app: &mut Application, win: u64) -> String {
    if let Some((s, w, c)) = find_window_indexes(app, win) {
        app.runtime.screens[s].workspaces[w].clients[c]
            .window_name
            .clone()
    } else {
        "Unmanaged Window".to_string()
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
