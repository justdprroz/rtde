#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
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

fn get_mut_ptr<T>(value: &mut T) -> *mut T {
    value as *mut T
}

fn get_keycode(dpy: *mut Display, keysym: u32) -> u32 {
    unsafe { XKeysymToKeycode(dpy, keysym as u64) as u32 }
}

fn argb_to_int(a: u32, r: u8, g: u8, b: u8) -> u64 {
    // (((r as u64) << 16 | (g as u64) << 8 | (b as u64)) & 0x00000000_00ffffff) | (a << 24) as u64;
    eprintln!(
        "{}",
        (((r as u64) << 16 | (g as u64) << 8 | (b as u64)) & 0x00000000_00ffffff)
            | ((a as u64) << 24) as u64
    );
    (a as u64) << 24 | (r as u64) << 16 | (g as u64) << 8 | (b as u64)
}

use Mod1Mask as ModKey;
const ModKeyShift: u32 = ModKey | ShiftMask;

fn main() {
    unsafe {
        let dpy: *mut Display = XOpenDisplay(0x0 as *const i8);
        let mut attr: XWindowAttributes = get_default::XWindowAttributes();
        let mut start: XButtonEvent = get_default::XButtonEvent();
        let mut ev: XEvent = get_default::XEvent();

        grab_key(dpy, XK_Return, ModKey | ShiftMask); // Move to top
        grab_key(dpy, XK_Return, ModKey); // Spawn alacritty
        grab_key(dpy, XK_Q, ModKey | ShiftMask); // Exit rust-wm
        grab_key(dpy, XK_p, ModKey); // Run dmenu

        grab_button(dpy, 1, Mod1Mask); // Move window
        grab_button(dpy, 2, Mod1Mask); // Focus window
        grab_button(dpy, 3, Mod1Mask); // Resize window

        start.subwindow = 0;

        loop {
            XNextEvent(dpy, get_mut_ptr(&mut ev));
            if ev.type_ == KeyPress {
                if ev.key.state == ModKey {
                    if ev.key.keycode == get_keycode(dpy, XK_Return) {
                        Command::new("alacritty").spawn().unwrap();
                    }
                    if ev.key.keycode == get_keycode(dpy, XK_p) {
                        Command::new("dmenu_run").spawn().unwrap();
                    }
                }
                if ev.key.state == ModKeyShift {
                    if ev.key.keycode == get_keycode(dpy, XK_Return) {
                        if ev.key.subwindow != 0 {
                            XRaiseWindow(dpy, ev.key.subwindow);
                        }
                    }
                    if ev.key.keycode == get_keycode(dpy, XK_q) {
                        break;
                    }
                }
            }
            if ev.type_ == ButtonPress {
                if ev.button.subwindow != 0 {
                    if ev.button.button == 2 {
                        XRaiseWindow(dpy, ev.button.subwindow);
                        XSetWindowBorderWidth(dpy, ev.button.subwindow, 5);
                        XSetWindowBorder(dpy, ev.button.subwindow, {
                            argb_to_int(0, 98, 114, 164)
                        });
                    } else {
                        XGetWindowAttributes(
                            dpy,
                            ev.button.subwindow,
                            &mut attr as *mut XWindowAttributes,
                        );
                        start = ev.button;
                    }
                }
            }
            if ev.type_ == MotionNotify {
                if ev.button.subwindow != 0 && start.subwindow != 0 {
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
                }
            }
            if ev.type_ == ButtonRelease {
                start.subwindow = 0;
            }
        }
    }
}
