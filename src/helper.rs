//! Set of functions used by [`crate::logic`]

use std::ffi::CStr;
use std::mem::size_of;
use std::ptr::null_mut;

use crate::structs::*;
use crate::utils::*;
use crate::wrapper::xlib::*;

use x11::xlib::Atom;
use x11::xlib::ClientMessage;
use x11::xlib::CurrentTime;
use x11::xlib::NoEventMask;
use x11::xlib::PMaxSize;
use x11::xlib::PMinSize;
use x11::xlib::PropModeAppend;
use x11::xlib::RevertToPointerRoot;
use x11::xlib::StructureNotifyMask;
use x11::xlib::Success;
use x11::xlib::XGetWindowProperty;
use x11::xlib::XA_ATOM;
use x11::xlib::XA_WINDOW;
use x11::xlib::{PropModeReplace, XA_CARDINAL};

/// Set desktop for specified window
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

/// Updates client list property of WM
/// 1. Delete present list
/// 2. For every client on every workspace on every screen add client to list
pub fn update_client_list(app: &mut Application) {
    // 1. Delete
    delete_property(
        app.core.display,
        app.core.root_win,
        app.atoms.net_client_list,
    );

    // 2. Update
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
    let ev = EEvent::ClientMessage {
        client_message_event: x11::xlib::XClientMessageEvent {
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
        },
    };
    return send_event(app.core.display, win, false, NoEventMask, ev);
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

/// Shows all windows on current workspace
pub fn show_workspace(app: &mut Application, screen: usize, workspace: usize) {
    let screen = &mut app.runtime.screens[screen];
    let workspace = &mut screen.workspaces.get_mut(workspace).unwrap();
    // Iterate over all clients
    for client in &mut workspace.clients {
        // 10. Fullscreen window if needed
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
            // 11. Update borders
            set_window_border_width(app.core.display, client.window_id, client.border);
            // 12. Position windows
            move_resize_window(
                app.core.display,
                client.window_id,
                client.x + screen.x as i32,
                client.y + screen.y as i32,
                // client.x,
                // client.y,
                client.w,
                client.h,
            );
            if client.floating {
                raise_window(app.core.display, client.window_id);
            }
        };
        client.visible = true;
    }
}

/// Hides all windows on current workspace
pub fn hide_workspace(app: &mut Application, screen: usize, workspace: usize) {
    let window_decoration_offset = app.config.gap_width + app.config.border_size;
    let screen = &mut app.runtime.screens[screen];
    let workspace = &mut screen.workspaces.get_mut(workspace).unwrap();
    // Iterate over all clients
    for client in &mut workspace.clients {
        move_resize_window(
            app.core.display,
            client.window_id,
            -(client.w as i32 + window_decoration_offset as i32),
            -(client.h as i32 + window_decoration_offset as i32),
            client.w,
            client.h,
        );
        // flip visibility state
        client.visible = false;
    }
}
/// Arrange windows of specified workspace in specified layout
/// 1. Get structs by index
/// 2. Calculate usable screen sizes, gaps, borders etc
/// 3. Get amount of clients to be tiled
/// 4. Check if all client go to master
/// 5. Iterate all clients in current workspace and calculate geometry
/// 6. Show maximized clients
/// 7. Show master clients
/// 8. Show stack clients
/// 9. Update calculated geometry
/// 10. Fullscreen window if needed
/// 11. Update borders
/// 12. Position windows
pub fn arrange_workspace(app: &mut Application, screen: usize, workspace: usize) {
    log!("======ARRANGING S: {}, W: {}", screen, workspace);
    // 1. Get actual structures
    let screen = &mut app.runtime.screens[screen];
    let workspace = &mut screen.workspaces[workspace];
    // 2. Calculate usable screen sizes, gaps, borders etc
    let bar_offsets = screen.bar_offsets;
    let screen_height = screen.height - (bar_offsets.up + bar_offsets.down) as i64;
    let gap = app.config.gap_width as i32;
    let border = app.config.border_size as u32;
    let mut master_width = ((screen.width as i32 - gap * 3) as f64 * workspace.master_width) as u32;
    let stack_width = (screen.width as i32 - gap * 3) - master_width as i32;
    let mut master_capacity = workspace.master_capacity;

    // 3. Get amount of clients to be tiled
    let stack_size = workspace.clients.iter().filter(|&c| !c.floating).count();
    // 4. Check if all client go to master
    if master_capacity <= 0 || master_capacity >= stack_size as i64 {
        master_capacity = stack_size as i64;
        master_width = screen.width as u32 - gap as u32 * 2;
    }
    log!("   |- Arranging {} tilable window", stack_size);
    // 5. Iterate all clients in current workspace and calculate geometry
    for (index, client) in workspace
        .clients
        .iter_mut()
        .rev()
        .filter(|c| !c.floating && !c.fullscreen)
        .enumerate()
    {
        // 6. Show maximized clients
        if stack_size == 1 {
            client.x = 0;
            client.y = bar_offsets.up as i32;
            client.w = screen.width as u32;
            client.h = screen_height as u32;
            client.border = 0;
        } else {
            if (index as i64) < master_capacity {
                // 7. Show master clients
                let win_height =
                    (screen_height - gap as i64 - master_capacity * gap as i64) / master_capacity;
                client.x = gap;
                client.y = bar_offsets.up as i32 + gap + (win_height as i32 + gap) * index as i32;
                client.w = master_width - 2 * border;
                client.h = win_height as u32 - 2 * border
            } else {
                // 8. Show stack clients
                let win_height = (screen_height
                    - gap as i64
                    - (stack_size as i64 - master_capacity) * gap as i64)
                    / (stack_size as i64 - master_capacity);
                client.x = master_width as i32 + (gap * 2);
                client.y = bar_offsets.up as i32
                    + gap
                    + (win_height as i32 + gap) * (index as i64 - master_capacity) as i32;
                client.w = stack_width as u32 - 2 * border;
                client.h = win_height as u32 - 2 * border;
            }
            client.border = app.config.border_size as u32;
        }

        // client.x += screen.x as i32;
        // client.y += screen.y as i32;
    }
    return;
}

/// Spawn new program by forking
///
/// 1. Fork get child PID for rules
/// 2. For child close connections from Parent
/// 3. Spawn program using sh
pub fn spawn<S: AsRef<CStr>>(app: &mut Application, args: &[S], rule: Option<(usize, usize)>) {
    unsafe {
        match nix::unistd::fork() {
            Ok(nix::unistd::ForkResult::Parent { child }) => {
                // 1. Add child to rules if specified
                if let Some((s, w)) = rule {
                    app.runtime.autostart_rules.push(AutostartRulePID {
                        pid: child.into(),
                        screen: s,
                        workspace: w,
                    })
                }
            }
            Ok(nix::unistd::ForkResult::Child) => {
                // 2. Close
                if app.core.display as *mut x11::xlib::Display as usize != 0 {
                    match nix::unistd::close(x11::xlib::XConnectionNumber(app.core.display)) {
                        Ok(_) => {}
                        Err(_) => {}
                    };
                }
                // 3. Run
                let _ = nix::unistd::execvp(args[0].as_ref(), &args);
            }
            Err(_) => {}
        }
    }
}

pub fn get_client_pid(app: &mut Application, win: u64) -> Option<i32> {
    unsafe {
        let mut actual_type: Atom = 0;
        let mut actual_format: i32 = 0;
        let mut nitems: u64 = 0;
        let mut bytes_after: u64 = 0;
        let mut prop: *mut u8 = std::ptr::null_mut();
        XGetWindowProperty(
            app.core.display,
            win,
            app.atoms.net_wm_pid,
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
        if actual_type != 0 {
            Some(*prop as i32 + *prop.wrapping_add(1) as i32 * 256)
        } else {
            None
        }
    }
}

pub fn get_client_workspace(app: &mut Application, win: u64) -> Option<(usize, usize)> {
    let client_desktop = unsafe {
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
        if actual_type != 0 {
            Some(*prop as u64)
        } else {
            None
        }
    };

    match client_desktop {
        Some(d) => {
            let s = d as usize / 10;
            let w = d as usize % 10;
            if s < app.runtime.screens.len() && w < app.runtime.screens[s].workspaces.len() {
                Some((s, w))
            } else {
                None
            }
        }
        None => None,
    }
}

/// Update EWMH desktop properties
pub fn update_desktop_ewmh_info(
    app: &mut Application,
    names: Vec<String>,
    mut viewports: Vec<i64>,
) {
    // Set amount of workspaces
    change_property(
        app.core.display,
        app.core.root_win,
        app.atoms.net_number_of_desktops,
        XA_CARDINAL,
        32,
        PropModeReplace,
        &mut names.len() as *mut usize as *mut u8,
        1,
    );

    // Set workspaces names
    let mut bytes = vec_string_to_bytes(names);
    change_property(
        app.core.display,
        app.core.root_win,
        app.atoms.net_desktop_names,
        app.atoms.utf8string,
        8,
        PropModeReplace,
        bytes.as_mut_ptr(),
        bytes.len() as i32,
    );

    // Set workspaces viewports
    change_property(
        app.core.display,
        app.core.root_win,
        app.atoms.net_desktop_viewport,
        XA_CARDINAL,
        32,
        PropModeReplace,
        viewports.as_mut_ptr() as *mut u8,
        viewports.len() as i32,
    );
}

pub fn configure(dpy: &mut x11::xlib::Display, client: &mut Client) {
    let ce = x11::xlib::XConfigureEvent {
        type_: x11::xlib::ConfigureNotify,
        display: dpy,
        event: client.window_id,
        window: client.window_id,
        x: client.x,
        y: client.y,
        width: client.w as i32,
        height: client.h as i32,
        border_width: client.border as i32,
        above: 0,
        override_redirect: 0,
        serial: 0,
        send_event: 0,
    };
    send_event(
        dpy,
        client.window_id,
        false,
        StructureNotifyMask,
        EEvent::ConfigureNotify { configure: ce },
    );
}
