#![allow(non_snake_case)]
use std::process::Command;
use x11::{keysym::*, xlib::*};

mod utils;
use utils::get_default;

fn grab_key(dpy: *mut Display, keysym: u32, mask: u32) {
    unsafe {
        XGrabKey(
            dpy,
            XKeysymToKeycode(dpy, keysym as u64) as i32,
            mask,
            XDefaultRootWindow(dpy),
            1,
            GrabModeAsync,
            GrabModeAsync,
        );
    }
}

fn grab_button(dpy: *mut Display, button: u32, mask: u32) {
    unsafe {
        XGrabButton(
            dpy,
            button,
            mask,
            XDefaultRootWindow(dpy),
            1,
            (ButtonPressMask | ButtonReleaseMask | PointerMotionMask) as u32,
            GrabModeAsync,
            GrabModeAsync,
            0,
            0,
        );
    }
}

fn main() {
    unsafe {
        use Mod1Mask as ModKey;
        let dpy: *mut Display;
        dpy = XOpenDisplay(0x0 as *const i8);
        let mut attr: XWindowAttributes = get_default::XWindowAttributes();
        let mut start: XButtonEvent = get_default::XButtonEvent();
        let mut ev: XEvent = get_default::XEvent();

        grab_key(dpy, XK_Return, ModKey | ShiftMask); // Move to top
        grab_key(dpy, XK_Return, ModKey); // Spawn alacritty
        grab_key(dpy, XK_Q, ModKey); // Exit rust-wm
        grab_key(dpy, XK_p, ModKey); // Run dmenu

        grab_button(dpy, 1, Mod1Mask); // Move window
        grab_button(dpy, 2, Mod1Mask); // Focus window
        grab_button(dpy, 3, Mod1Mask); // Resize window

        start.subwindow = 0;

        loop {
            XNextEvent(dpy, &mut ev as *mut XEvent);
            if ev.type_ == KeyPress {
                if ev.key.subwindow != 0 {
                    if ev.key.keycode == XKeysymToKeycode(dpy, XK_Return as u64) as u32 {
                        XRaiseWindow(dpy, ev.key.subwindow);
                    }
                }
                if ev.key.keycode == XKeysymToKeycode(dpy, XK_Q as u64) as u32
                {
                    break;
                }
                if ev.key.keycode == XKeysymToKeycode(dpy, XK_p as u64) as u32 {
                    Command::new("dmenu_run").spawn().unwrap();
                }
            } else if ev.type_ == ButtonPress && ev.button.subwindow != 0 {
                if ev.button.button == 2 {
                    XRaiseWindow(dpy, ev.button.subwindow);
                } else {
                    XGetWindowAttributes(
                        dpy,
                        ev.button.subwindow,
                        &mut attr as *mut XWindowAttributes,
                    );
                    start = ev.button;
                }
            } else if ev.type_ == MotionNotify && start.subwindow != 0 {
                let x_diff = ev.button.x_root - start.x_root;
                let y_diff = ev.button.y_root - start.y_root;
                XMoveResizeWindow(
                    dpy,
                    start.subwindow,
                    attr.x + {
                        if start.button == 1 {
                            x_diff
                        } else {
                            0
                        }
                    },
                    attr.y + {
                        if start.button == 1 {
                            y_diff
                        } else {
                            0
                        }
                    },
                    1.max(
                        (attr.width + {
                            if start.button == 3 {
                                x_diff
                            } else {
                                0
                            }
                        }) as u32,
                    ),
                    1.max(
                        (attr.height + {
                            if start.button == 3 {
                                y_diff
                            } else {
                                0
                            }
                        }) as u32,
                    ),
                );
            } else if ev.type_ == ButtonRelease {
                start.subwindow = 0;
            }
        }
    }
}
