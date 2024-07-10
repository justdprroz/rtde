//! Functions ran for events

use x11::xlib::Button3;
use x11::xlib::CWHeight;
use x11::xlib::CWWidth;
use x11::xlib::CurrentTime;
use x11::xlib::PropModeReplace;
use x11::xlib::RevertToPointerRoot;
use x11::xlib::XWindowChanges;
use x11::xlib::CWX;
use x11::xlib::CWY;
use x11::xlib::XA_ATOM;

use crate::logic::*;
use crate::mouse::*;
use crate::setup::*;
use crate::structs::*;
use crate::utils::*;
use crate::wrap::xlib::*;

use x11::xlib::Button1;

pub fn key_press(app: &mut Application, ev: Event) {
    log!("|- Got keyboard event");
    // Safely retrive struct
    if let Some(ev) = ev.key {
        // Iterate over key actions matching current key input
        for action in app.config.key_actions.clone() {
            if ev.keycode == keysym_to_keycode(app.core.display, action.keysym)
                && ev.state == action.modifier
            {
                // Log action type
                log!("|- Got {:?} action", &action.result);
                // Match action result and run related function
                match &action.result {
                    ActionResult::KillClient => {
                        log!("   |- Got `KillClient` Action");
                        kill_client(app);
                    }
                    ActionResult::Spawn(cmd) => {
                        log!("   |- Got `Spawn` Action");
                        spawn(app, cmd.clone());
                    }
                    ActionResult::MoveToScreen(d) => {
                        move_to_screen(app, *d);
                    }
                    ActionResult::FocusOnScreen(d) => {
                        focus_on_screen(app, *d);
                    }
                    ActionResult::MoveToWorkspace(n) => {
                        move_to_workspace(app, *n);
                    }
                    ActionResult::FocusOnWorkspace(n) => {
                        focus_on_workspace(app, *n, true);
                    }
                    ActionResult::Quit => {
                        log!("   |- Got `Quit` Action. `Quiting`");
                        app.core.running = false;
                    }
                    ActionResult::UpdateMasterCapacity(i) => {
                        update_master_capacity(app, *i);
                    }
                    ActionResult::UpdateMasterWidth(w) => {
                        update_master_width(app, *w);
                    }
                    ActionResult::DumpInfo => {
                        log!("{:#?}", &app.runtime);
                    }
                    ActionResult::ToggleFloat => {
                        toggle_float(app);
                    }
                    ActionResult::CycleStack(_i) => {}
                }
            }
        }
    }
}

pub fn map_request(app: &mut Application, ev: Event) {
    if let Some(mp) = ev.map_request {
        let ew: u64 = mp.window;
        log!("|- Map Request From Window: {ew}");
        manage_client(app, ew);
    }
}

pub fn enter_notify(app: &mut Application, ev: Event) {
    if let Some(cr) = ev.crossing {
        let ew: u64 = cr.window;
        log!("|- Crossed Window `{}` ({})", get_client_name(app, ew), ew);
        if ew != app.core.root_win {
            log!("   |- Setting focus to window");
            // Focus on crossed window
            if let Some(cw) = get_current_client_id(app) {
                unfocus(app, cw);
            }
            focus(app, ew);
        } else {
            let ws = &mut app.runtime;

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
}

pub fn destroy_notify(app: &mut Application, ev: Event) {
    if let Some(dw) = ev.destroy_window {
        let ew: u64 = dw.window;
        log!("|- `{}` destroyed", get_client_name(app, ew));
        unmanage_window(app, ew);
    }
}

pub fn unmap_notify(app: &mut Application, ev: Event) {
    if let Some(um) = ev.unmap {
        let ew: u64 = um.window;
        log!("|- `{}` unmapped", get_client_name(app, ew));
        unmanage_window(app, ew);
    }
}

pub fn motion_notify(app: &mut Application, ev: Event) {
    if let Some(me) = ev.motion {
        log!("|- `Motion` detected");
        if app.runtime.mouse_state.button == Button1 {
            move_mouse(app, me);
        }
        if app.runtime.mouse_state.button == Button3 {
            resize_mouse(app, me);
        }
        if me.window == app.core.root_win {
            screen_mouse(app, me);
        }
    }
}

pub fn property_notify(app: &mut Application, ev: Event) {
    if let Some(p) = ev.property {
        if p.window != app.core.root_win {
            update_client_name(app, p.window);
        }
    }
}

pub fn configure_notify(app: &mut Application, ev: Event) {
    if let Some(cn) = ev.configure {
        if cn.window == app.core.root_win {
            log!("|- Got `ConfigureNotify` for `root window` -> Changing monitor layout");
            init_screens(app);
        } else if let Some((s, w, c)) = find_window_indexes(app, cn.window) {
            let client = &app.runtime.screens[s].workspaces[w].clients[c];
            log!(
                "|- Got `ConfigureNotify` of {:?} for `{}`",
                cn.event,
                client.window_name
            );
        } else {
            log!("|- Got `ConfigureNotify` from `Unmanaged window`");
        }
    }
}

pub fn client_message(app: &mut Application, ev: Event) {
    if let Some(c) = ev.client {
        log!("|- Got `Client Message`");
        if let Some(cc) = find_window_indexes(app, c.window) {
            let cc = &mut app.runtime.screens[cc.0].workspaces[cc.1].clients[cc.2];
            log!(
                "   |- Of type `{}` From: `{}`",
                get_atom_name(app.core.display, c.message_type),
                cc.window_name
            );
            if c.message_type == app.atoms.net_wm_state {
                if c.data.get_long(1) as u64 == app.atoms.net_wm_fullscreen
                    || c.data.get_long(2) as u64 == app.atoms.net_wm_fullscreen
                {
                    let sf = c.data.get_long(0) == 1 || c.data.get_long(0) == 2 && cc.fullscreen;
                    if sf && !cc.fullscreen {
                        change_property(
                            app.core.display,
                            c.window,
                            app.atoms.net_wm_state,
                            XA_ATOM,
                            32,
                            PropModeReplace,
                            &mut app.atoms.net_wm_fullscreen as *mut u64 as *mut u8,
                            1,
                        );
                        cc.fullscreen = true;
                        arrange(app);
                    } else if !sf && cc.fullscreen {
                        change_property(
                            app.core.display,
                            c.window,
                            app.atoms.net_wm_state,
                            XA_ATOM,
                            32,
                            PropModeReplace,
                            std::ptr::null_mut::<u8>(),
                            0,
                        );
                        cc.fullscreen = false;
                        arrange(app);
                    }
                } else {
                    log!("      |- Unsupported `state`");
                }
            }
        } else {
            log!(
                "   |- Of type `{}`",
                get_atom_name(app.core.display, c.message_type)
            );
            if c.message_type == app.atoms.net_current_desktop {
                focus_on_workspace(app, c.data.get_long(0) as u64, false);
            }
        }
    }
}

pub fn configure_request(app: &mut Application, ev: Event) {
    if let Some(cr) = ev.configure_request {
        log!("|- Got `ConfigureRequest` for `{}`", cr.window);
        if let Some((s, w, c)) = find_window_indexes(app, cr.window) {
            let sw = app.runtime.screens[s].width as i32;
            let sh = app.runtime.screens[s].height as i32;
            let ba = app.runtime.screens[s].bar_offsets;
            let client = &mut app.runtime.screens[s].workspaces[w].clients[c];
            let mut resized = false;

            if client.floating {
                if (cr.value_mask & CWWidth as u64) != 0 {
                    client.w = cr.width as u32;
                    resized = true;
                }
                if (cr.value_mask & CWHeight as u64) != 0 {
                    client.h = cr.height as u32;
                    resized = true;
                }
                if (cr.value_mask & CWX as u64) != 0 {
                    client.x = cr.x;
                    resized = true;
                }
                if (cr.value_mask & CWY as u64) != 0 {
                    client.y = cr.y;
                    resized = true;
                }

                if resized {
                    client.x = (sw - (client.w as i32)) / 2;
                    client.y = (sh - (ba.up as i32) - (client.h as i32)) / 2;

                    move_resize_window(
                        app.core.display,
                        client.window_id,
                        client.x,
                        client.y,
                        client.w,
                        client.h,
                    );
                }
            }
        } else {
            let mut wc = XWindowChanges {
                x: cr.x,
                y: cr.y,
                width: cr.width,
                height: cr.height,
                border_width: cr.border_width,
                sibling: cr.above,
                stack_mode: cr.detail,
            };
            configure_window(app.core.display, cr.window, cr.value_mask as u32, &mut wc);
        }
    }
}

pub fn button_press(app: &mut Application, ev: Event) {
    if let Some(bp) = ev.button {
        log!("|- Got `ButtonPress` event");
        if let Some((s, w, c)) = find_window_indexes(app, bp.window) {
            let cc = &app.runtime.screens[s].workspaces[w].clients[c];
            if cc.floating {
                app.runtime.mouse_state = MouseState {
                    win: bp.window,
                    button: bp.button,
                    pos: (bp.x_root as i64, bp.y_root as i64),
                };
                println!("{:?}", app.runtime.mouse_state.pos);
                if bp.button == Button3 {
                    warp_pointer_win(app.core.display, bp.window, cc.w as i32, cc.h as i32);
                    app.runtime.mouse_state.pos = (
                        (bp.x_root - bp.x + cc.w as i32) as i64,
                        (bp.y_root - bp.y + cc.h as i32) as i64,
                    );
                }
            }
        }
    }
}
pub fn button_release(app: &mut Application, _ev: Event) {
    log!("|- Got `ButtonRelease` event");
    app.runtime.mouse_state = MouseState {
        win: 0,
        button: 0,
        pos: (0, 0),
    };
}
