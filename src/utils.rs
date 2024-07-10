//! Some utility functions without much logic in them

use std::ffi::CString;
use std::mem::size_of;
use std::ptr::null_mut;

use crate::structs::*;
use crate::wrap::xlib::*;

use x11::xlib::Atom;
use x11::xlib::ClientMessage;
use x11::xlib::CurrentTime;
use x11::xlib::NoEventMask;
use x11::xlib::PMaxSize;
use x11::xlib::PMinSize;
use x11::xlib::PropModeAppend;
use x11::xlib::RevertToPointerRoot;
use x11::xlib::Success;
use x11::xlib::XA_ATOM;
use x11::xlib::XA_WINDOW;
use x11::xlib::{PropModeReplace, XA_CARDINAL};

/// Convert color to 64 bit int for x11
pub fn argb_to_int(c: Color) -> u64 {
    (c.alpha as u64) << 24 | (c.red as u64) << 16 | (c.green as u64) << 8 | (c.blue as u64)
}

/// Convert Rust Vector of Strings to C array of bytes
pub fn vec_string_to_bytes(strings: Vec<String>) -> Vec<u8> {
    let mut bytes: Vec<u8> = vec![];
    for string in strings {
        match CString::new(string) {
            Ok(c) => bytes.append(&mut c.into_bytes_with_nul()),
            Err(_) => todo!(),
        }
    }
    bytes
}

/// Log if in debug
#[macro_export]
macro_rules! log {
    ($($e:expr),+) => {
        #[cfg(debug_assertions)]
        println!($($e),+);
    };
}
pub use log;

/// Remove zombie processes after spawning with shortcuts
pub fn no_zombies() {
    use nix::sys::signal::*;
    unsafe {
        let sa = SigAction::new(
            SigHandler::SigIgn,
            SaFlags::SA_NOCLDSTOP | SaFlags::SA_NOCLDWAIT | SaFlags::SA_RESTART,
            SigSet::empty(),
        );
        let _ = sigaction(SIGCHLD, &sa);
    }
}

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
