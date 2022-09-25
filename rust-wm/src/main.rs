#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
use core::ffi::CStr;
use std::{process::Command, ptr::null_mut};
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
    (a as u64) << 24 | (r as u64) << 16 | (g as u64) << 8 | (b as u64)
}

use Mod1Mask as ModKey;

const ModKeyShift: u32 = ModKey | ShiftMask;

fn main() {
    unsafe {
        let events = vec![
            "",
            "",
            "KeyPress",
            "KeyRelease",
            "ButtonPress",
            "ButtonRelease",
            "MotionNotify",
            "EnterNotify",
            "LeaveNotify",
            "FocusIn",
            "FocusOut",
            "KeymapNotify",
            "Expose",
            "GraphicsExpose",
            "NoExpose",
            "VisibilityNotify",
            "CreateNotify",
            "DestroyNotify",
            "UnmapNotify",
            "MapNotify",
            "MapRequest",
            "ReparentNotify",
            "ConfigureNotify",
            "ConfigureRequest",
            "GravityNotify",
            "ResizeRequest",
            "CirculateNotify",
            "CirculateRequest",
            "PropertyNotify",
            "SelectionClear",
            "SelectionRequest",
            "SelectionNotify",
            "ColormapNotify",
            "ClientMessage",
            "MappingNotify",
            "GenericEvent",
            "LASTEvent",
        ];
        let dpy: *mut Display = XOpenDisplay(0x0 as *const i8);
        let mut attr: XWindowAttributes = get_default::XWindowAttributes();
        let mut start: XButtonEvent = get_default::XButtonEvent();
        let mut ev: XEvent = get_default::XEvent();

        let mut wa: XSetWindowAttributes = XSetWindowAttributes {
            background_pixmap: 0,
            background_pixel: 0,
            border_pixmap: 0,
            border_pixel: 0,
            bit_gravity: 0,
            win_gravity: 0,
            backing_store: 0,
            backing_planes: 0,
            backing_pixel: 0,
            save_under: 0,
            event_mask: 0,
            do_not_propagate_mask: 0,
            override_redirect: 0,
            colormap: 0,
            cursor: 0,
        };
        wa.event_mask = SubstructureNotifyMask | StructureNotifyMask;
        XChangeWindowAttributes(
            dpy,
            XDefaultRootWindow(dpy),
            CWEventMask | CWCursor,
            get_mut_ptr(&mut wa),
        );

        grab_key(dpy, XK_Return, ModKey | ShiftMask); // Move to top
        grab_key(dpy, XK_Return, ModKey); // Spawn alacritty
        grab_key(dpy, XK_Q, ModKey | ShiftMask); // Exit rust-wm
        grab_key(dpy, XK_p, ModKey); // Run dmenu

        grab_button(dpy, 1, Mod1Mask); // Move window
        grab_button(dpy, 2, Mod1Mask); // Focus window
        grab_button(dpy, 3, Mod1Mask); // Resize window

        start.subwindow = 0;

        loop {
            println!("|--Getting event");
            XNextEvent(dpy, get_mut_ptr(&mut ev));
            println!("  |--got event of type {}", events[ev.type_ as usize]);
            if ev.type_ == KeyPress {
                if ev.key.state == ModKey {
                    if ev.key.keycode == get_keycode(dpy, XK_Return) {
                        Command::new("xterm").spawn().unwrap();
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
                        let win = ev.button.subwindow;
                        XRaiseWindow(dpy, win);
                        XSetInputFocus(dpy, win, RevertToParent, CurrentTime);
                        // add window decoration
                        XSetWindowBorderWidth(dpy, win, 2);
                        XSetWindowBorder(dpy, win, argb_to_int(0, 98, 114, 164));
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
            if ev.type_ == CreateNotify {
                let win = ev.create_window.window;

                // get name
                let mut c: *mut i8 = null_mut();
                if XFetchName(dpy, win, get_mut_ptr(&mut c)) == True {
                    println!("|-- Got window name");
                    println!("  |-- Name is {:?}", CStr::from_ptr(c).to_str());
                    libc::free(c as *mut libc::c_void);
                } else {
                    println!("|-- Failed to get window name");
                }

                // get class
                let ch = XAllocClassHint();
                if XGetClassHint(dpy, win, ch) == True {
                    println!("|-- Got window class");
                    println!("  |-- name: {:?}", CStr::from_ptr((*ch).res_name).to_str());
                    println!(
                        "  |-- class: {:?}",
                        CStr::from_ptr((*ch).res_class).to_str()
                    );
                    XFree((*ch).res_name as *mut libc::c_void);
                    XFree((*ch).res_class as *mut libc::c_void);
                } else {
                    println!("|-- Failed to get window class");
                }
            }
            if ev.type_ == MapNotify {
                let win = ev.map.window;

                // place window on top of others
                XRaiseWindow(dpy, win);

                // focus on window
                let mut attr = get_default::XWindowAttributes();
                XGetWindowAttributes(dpy, win, get_mut_ptr(&mut attr));
                if attr.map_state == IsViewable {
                    println!("Window is viewable");
                    XSetInputFocus(dpy, win, RevertToParent, CurrentTime);
                } else {
                    println!("Window is NOT viewable");
                }

                // add window decoration
                XSetWindowBorderWidth(dpy, win, 2);
                XSetWindowBorder(dpy, win, argb_to_int(0, 98, 114, 164));
            }
        }
    }
}
