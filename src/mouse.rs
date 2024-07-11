//! Functions for mouse support

use x11::xlib::{PropModeReplace, XMotionEvent, XA_CARDINAL};

use crate::{
    structs::Application,
    utils::find_window_indexes,
    wrapper::xlib::{change_property, move_resize_window},
};

pub fn move_mouse(app: &mut Application, motion_event: XMotionEvent) {
    let moving_window: u64 = app.runtime.mouse_state.win;

    if let Some((s, w, c)) = find_window_indexes(app, moving_window) {
        let sx = app.runtime.screens[s].x as i32;
        let sy = app.runtime.screens[s].y as i32;
        let sw = app.runtime.screens[s].width as i32;
        let sh = app.runtime.screens[s].height as i32;
        let (mx, my) = (motion_event.x_root as i64, motion_event.y_root as i64);
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

        move_resize_window(
            app.core.display,
            moving_window,
            cc.x + sx,
            cc.y + sy,
            cc.w,
            cc.h,
        );
    }
}

pub fn resize_mouse(app: &mut Application, me: XMotionEvent) {
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
