//! Functions for mouse support

use x11::xlib::{PropModeReplace, XMotionEvent, XA_CARDINAL};

use crate::helper::find_window_indexes;
use crate::structs::Application;
use crate::wrapper::xlib::change_property;
use crate::wrapper::xlib::move_resize_window;

pub fn move_mouse(app: &mut Application, motion_event: XMotionEvent) {
    let moving_window: u64 = app.runtime.mouse_state.win;

    if let Some((s, w, c)) = find_window_indexes(app, moving_window) {
        let screen_x = app.runtime.screens[s].x as i32;
        let screen_y = app.runtime.screens[s].y as i32;
        let screen_w = app.runtime.screens[s].width as i32;
        let screen_h = app.runtime.screens[s].height as i32;
        let (mouse_x, mouse_y) = (motion_event.x_root as i64, motion_event.y_root as i64);
        let (pos_x, pos_y) = app.runtime.mouse_state.pos;
        let (dx, dy) = (mouse_x - pos_x, mouse_y - pos_y);

        // Screen Bar Left/Up/Right/Down
        let (sbl, sbu, sbr, sbd) = {
            let screen = &app.runtime.screens[s];
            let sbl = screen.x + screen.bar_offsets.left as i64;
            let sbu = screen.y + screen.bar_offsets.up as i64;
            let sbr = screen.x + screen.width - screen.bar_offsets.right as i64;
            let sbd = screen.y + screen.height - screen.bar_offsets.down as i64;
            (sbl, sbu, sbr, sbd)
        };

        let client = &mut app.runtime.screens[s].workspaces[w].clients[c];
        let mut new_x = client.x + dx as i32;
        let mut new_y = client.y + dy as i32;

        // Stick to screen border
        let stick = 50;
        let unstick = 30 as i64;

        if (new_x < sbl as i32 + stick)
            && (dx < 0 && new_x > (sbl - unstick) as i32
                || new_x > sbl as i32 && new_x < (sbl + unstick) as i32)
        {
            new_x = sbl as i32;
        }
        if (new_x + client.w as i32) > sbr as i32 - stick
            && (dx > 0 && (new_x + client.w as i32) < (sbr + unstick) as i32
                || (new_x + client.w as i32) < sbr as i32 && (new_x + client.w as i32) > (sbr - unstick) as i32)
        {
            new_x = sbr as i32 - client.w as i32 - 2 * app.config.border_size as i32;
        }

        if new_y < sbu as i32 + stick
            && (dy < 0 && new_y > (sbu - unstick) as i32
                || new_y > sbu as i32 && new_y < (sbu + unstick) as i32)
        {
            new_y = sbu as i32;
        }
        if (new_y + client.h as i32) > sbd as i32 - stick
            && (dy > 0 && (new_y + client.h as i32) < (sbd + unstick) as i32
                || (new_y + client.h as i32) < sbd as i32 && (new_y + client.h as i32) > (sbd - unstick) as i32)
        {
            new_y = sbd as i32 - client.h as i32 - 2 * app.config.border_size as i32;
        }

        // Unstick from border

        if client.x != new_x {
            app.runtime.mouse_state.pos.0 = mouse_x;
        }

        if client.y != new_y {
            app.runtime.mouse_state.pos.1 = mouse_y;
        }

        client.x = new_x;
        client.y = new_y;

        if client.x < screen_x {
            client.x = screen_x;
        }
        if client.y < screen_y {
            client.y = screen_y;
        }
        if (client.x + client.w as i32) > screen_x + screen_w {
            client.x = screen_x + screen_w - client.w as i32;
        }
        if (client.y + client.h as i32) > screen_y + screen_h {
            client.y = screen_y + screen_h - client.h as i32;
        }

        move_resize_window(
            app.core.display,
            moving_window,
            client.x,
            client.y,
            client.w,
            client.h,
        );
    }
}

pub fn resize_mouse(app: &mut Application, motion_event: XMotionEvent) {
    let (mouse_x, mouse_y) = (motion_event.x_root as i64, motion_event.y_root as i64);

    let (pos_x, pos_y) = app.runtime.mouse_state.pos;
    let (dx, dy) = (mouse_x - pos_x, mouse_y - pos_y);
    app.runtime.mouse_state.pos = (mouse_x, mouse_y);
    let mw: u64 = app.runtime.mouse_state.win;

    if let Some((s, w, c)) = find_window_indexes(app, mw) {
        let client = &mut app.runtime.screens[s].workspaces[w].clients[c];
        let mut nw = client.w as i32;
        let mut nh = client.h as i32;
        if (nw + dx as i32) > client.minw {
            if client.maxw == 0 || client.maxw > 0 && (nw + dx as i32) < client.maxw {
                nw += dx as i32;
            }
        };
        if (nh + dy as i32) > client.minh {
            if client.maxh == 0 || client.maxh > 0 && (nh + dy as i32) < client.maxh {
                nh += dy as i32;
            }
        }
        client.w = nw as u32;
        client.h = nh as u32;
        move_resize_window(app.core.display, mw, client.x, client.y, client.w, client.h);
    }
}

pub fn screen_mouse(app: &mut Application, me: XMotionEvent) {
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
