//! Some utility functions without much logic in them

use std::ffi::CString;

use crate::logic::*;
use crate::structs::*;
use crate::wrap::xlib::*;

use x11::xlib::AnyButton;
use x11::xlib::AnyModifier;
use x11::xlib::Button1;
use x11::xlib::Button3;
use x11::xlib::CurrentTime;
use x11::xlib::Mod4Mask as ModKey;
use x11::xlib::RevertToPointerRoot;
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
