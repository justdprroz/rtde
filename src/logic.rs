//! Main windows manager logic processed as response to events

use crate::structs::*;
use crate::utils::*;
use crate::wrapper::xlib::*;

use x11::xlib::AnyButton;
use x11::xlib::AnyModifier;
use x11::xlib::Button1;
use x11::xlib::Button3;
use x11::xlib::CurrentTime;
use x11::xlib::DestroyAll;
use x11::xlib::Mod4Mask as ModKey;
use x11::xlib::PropModeReplace;
use x11::xlib::RevertToPointerRoot;
use x11::xlib::XA_CARDINAL;

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
        arrange_current(app);
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
            arrange_current(app);
            show_workspace(
                app,
                app.runtime.current_screen,
                app.runtime.current_workspace,
            );
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
            arrange_workspace(app, app.runtime.current_screen, n as usize);
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
        hide_workspace(
            app,
            app.runtime.current_screen,
            app.runtime.current_workspace,
        );
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
        show_workspace(
            app,
            app.runtime.current_screen,
            app.runtime.current_workspace,
        );
        // Arrange update workspace
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
    arrange_current(app);
}

pub fn update_master_capacity(app: &mut Application, i: i64) {
    // Change master size
    app.runtime.screens[app.runtime.current_screen].workspaces[app.runtime.current_workspace]
        .master_capacity += i;
    // Rearrange windows
    arrange_current(app);
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
        arrange_current(app);
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

/// Arrange windows of current workspace in specified layout
/// 1. Iterate over all screens
/// 2. Arrange current workspace
pub fn arrange_current(app: &mut Application) {
    log!("   |- Arranging...");
    // 1. Iterate over all screens
    let screens_amount = app.runtime.screens.len();
    for index in 0..screens_amount {
        let current_workspace = app.runtime.screens[index].current_workspace;
        arrange_workspace(app, index, current_workspace);
    }
}

/// Arrange all clients
/// 1. Iterate over all screens
/// 2. Iterate over all workspaces
/// 3. Arrange it
pub fn arrange_all(app: &mut Application) {
    let screens_amount = app.runtime.screens.len();
    for screen in 0..screens_amount {
        let workspaces_amount = app.runtime.screens[screen].workspaces.len();
        for workspace in 0..workspaces_amount {
            arrange_workspace(app, screen, workspace);
        }
    }
}
