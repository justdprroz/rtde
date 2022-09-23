use std::process::Command;

use x11::{xlib::*, keysym::*};

fn main() {
    unsafe {
        let dpy: *mut Display;
        dpy = XOpenDisplay(0x0 as *const i8);

        let attr: XWindowAttributes;
        let mut start: XButtonEvent = XButtonEvent { type_: 0, serial: 0, send_event: 0, display: dpy, window: 0, root: 0, subwindow: 0, time: 0, x: 0, y: 0, x_root: 0, y_root: 0, state: 0, button: 0, same_screen: 0 };
        let mut ev: XEvent = XEvent { type_: 0 };

        XGrabKey(dpy, XKeysymToKeycode(dpy, XK_Return as u64) as i32, Mod1Mask,XDefaultRootWindow(dpy), 1, GrabModeAsync, GrabModeAsync);
        XGrabKey(dpy, XKeysymToKeycode(dpy, XK_Q as u64) as i32, Mod1Mask, XDefaultRootWindow(dpy), 1, GrabModeAsync, GrabModeAsync);
        XGrabKey(dpy, XKeysymToKeycode(dpy, XK_p as u64) as i32, Mod1Mask, XDefaultRootWindow(dpy), 1, GrabModeAsync, GrabModeAsync);

        XGrabButton(dpy, 1, Mod1Mask, XDefaultRootWindow(dpy), 1,(ButtonPressMask|ButtonReleaseMask|PointerMotionMask) as u32, GrabModeAsync, GrabModeAsync, 0, 0);
        XGrabButton(dpy, 2, Mod1Mask, XDefaultRootWindow(dpy), 1,(ButtonPressMask|ButtonReleaseMask|PointerMotionMask) as u32, GrabModeAsync, GrabModeAsync, 0, 0);
        XGrabButton(dpy, 3, Mod1Mask, XDefaultRootWindow(dpy), 1,(ButtonPressMask|ButtonReleaseMask|PointerMotionMask) as u32, GrabModeAsync, GrabModeAsync, 0, 0);

        start.subwindow = 0;

        loop {
            XNextEvent(dpy, &mut ev as *mut XEvent );
            if ev.type_ == KeyPress {
                if ev.key.subwindow != 0 {
                    if ev.key.keycode == XKeysymToKeycode(dpy, XK_Return as u64) as u32 {
                        XRaiseWindow(dpy, ev.key.subwindow);
                    }
                }
                if ev.key.keycode == XKeysymToKeycode(dpy, XK_Q as u64) as u32 {
                    break;
                }
                if ev.key.keycode == XKeysymToKeycode(dpy, XK_p as u64) as u32 {
                    Command::new("/bin/sh -c dmenu_run").spawn().expect("good");
                }
            }
        }
    }
}
