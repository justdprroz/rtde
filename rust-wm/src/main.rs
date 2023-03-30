#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::CStr;
use std::iter::Enumerate;
use std::{process::Command, ptr::null_mut};
use libc::c_ulong;
use x11::xinerama::XineramaQueryScreens;
use x11::{keysym::*, xlib::*};

mod utils;
use utils::get_default;
use utils::grab::grab_button;
use utils::grab::grab_key;

// Get *Raw Pointer*
fn get_mut_ptr<T>(value: &mut T) -> *mut T {
    value as *mut T
}

// Get u32 keycode from keysym
fn get_keycode(dpy: *mut Display, keysym: u32) -> u32 {
    unsafe { XKeysymToKeycode(dpy, keysym as u64) as u32 }
}

// What the fuck is going on here
fn argb_to_int(a: u32, r: u8, g: u8, b: u8) -> u64 {
    (a as u64) << 24 | (r as u64) << 16 | (g as u64) << 8 | (b as u64)
}

fn get_event_names_list() -> Vec<&'static str> {
    vec![
        "_",
        "_",
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
        "_",
    ]
}

struct WindowInfo {}

fn get_window_info(_win: u64) -> WindowInfo {
    WindowInfo {  }
}

use Mod1Mask as ModKey;

const ModKeyShift: u32 = ModKey | ShiftMask;

fn main() {
    println!("Started Window Manager");
    unsafe {
        let events: Vec<&str> = get_event_names_list();

        println!("|- Created Event Look-Up Array");

        let dpy: *mut Display = XOpenDisplay(0x0 as *const i8);

        println!("|- Opened X Display");

        println!("|- Getting per monitor sizes");

        let mut nn: i32 = 0;
        let info = XineramaQueryScreens(dpy, get_mut_ptr(&mut nn));
        println!("|- There are {} screen connected", nn);
        let screens = std::slice::from_raw_parts_mut(info, nn as usize);
        for screen in screens {
            println!("|- Screen {} has size of {}x{} pixels and originates from {},{}", screen.screen_number, screen.width, screen.height, screen.x_org, screen.y_org);
        };

        let mut attr: XWindowAttributes = get_default::XWindowAttributes();
        let mut start: XButtonEvent = get_default::XButtonEvent();
        start.subwindow = 0;

        let mut ev: XEvent = get_default::XEvent();
        let mut clients: Vec<u64> = Vec::new();
        let mut client_index: Option<usize> = None;
        let mut current_win: u64 = 0;

        println!("|- Created Useful Variables");

        let mut wa: XSetWindowAttributes = get_default::XSetWindowAttributes();

        // wa.event_mask = LeaveWindowMask | EnterWindowMask | SubstructureNotifyMask | StructureNotifyMask;
        wa.event_mask = SubstructureRedirectMask
            | LeaveWindowMask
            | EnterWindowMask
            | SubstructureNotifyMask
            | StructureNotifyMask;

        XChangeWindowAttributes(
            dpy,
            XDefaultRootWindow(dpy),
            CWEventMask | CWCursor,
            get_mut_ptr(&mut wa),
        );
        XSelectInput(dpy, XDefaultRootWindow(dpy), wa.event_mask);

        println!("|- Applied Event Mask");

        let mut nums: u32 = 0;
        let mut d1: c_ulong = 0;
        let mut d2: c_ulong = 0;
        let mut wins: *mut c_ulong = null_mut();

        XQueryTree(dpy, XDefaultRootWindow(dpy), get_mut_ptr(&mut d1), get_mut_ptr(&mut d2), get_mut_ptr(&mut wins), get_mut_ptr(&mut nums));

        println!("|- {nums} windows are alredy present");

        let mut wins = std::slice::from_raw_parts_mut(wins, nums as usize);
        for win in wins {
            println!("|-- Checking window {win}");
            let mut wa = get_default::XWindowAttributes();
            if XGetWindowAttributes(dpy, *win, get_mut_ptr(&mut wa)) == 0 || wa.override_redirect != 0 || XGetTransientForHint(dpy, *win, get_mut_ptr(&mut d1)) != 0{
                println!("|---- Window is transient. Skipping");
                continue;
            }
            if wa.map_state == IsViewable {
                println!("|---- Window is viewable. Managing");
                current_win = *win;
                client_index = Some(clients.len());
                clients.push(*win);
                XRaiseWindow(dpy, *win);
            }
        }

        grab_key(dpy, XK_Return, ModKey | ShiftMask); // Move to top
        grab_key(dpy, XK_Return, ModKey); // Spawn alacritty
        grab_key(dpy, XK_Q, ModKey | ShiftMask); // Exit rust-wm
        grab_key(dpy, XK_p, ModKey); // Run dmenu
        grab_key(dpy, XK_Page_Up, ModKey); // maximize window
        grab_key(dpy, XK_C, ModKey | ShiftMask); // close window
        grab_key(dpy, XK_Tab, ModKey); // Cycle Through Windows
        grab_key(dpy, XK_l, ModKey); // Query current window information

        grab_button(dpy, 1, Mod1Mask); // Move window
        grab_button(dpy, 2, Mod1Mask); // Focus window
        grab_button(dpy, 3, Mod1Mask); // Resize window

        println!("|- Grabbed Shortcuts");
        println!("|- Default Root Window Id Is: {}", XDefaultRootWindow(dpy));
        println!("|- Starting Main Loop");
        loop {
            XNextEvent(dpy, get_mut_ptr(&mut ev));
            println!("   |- Got Event Of Type \"{}\"", events[ev.type_ as usize]);
            if ev.type_ == KeyPress {
                let _ew: u64 = ev.key.window;

                if ev.key.state == ModKey {
                    if ev.key.keycode == get_keycode(dpy, XK_Return) {
                        println!("   |- Spawning Terminal");
                        let mut handle = Command::new("kitty").spawn().expect("can't run kitty");
                        std::thread::spawn(move || {
                            handle.wait().expect("can't run process");
                        });
                    }
                    if ev.key.keycode == get_keycode(dpy, XK_p) {
                        println!("   |- Spawning Dmenu");
                        Command::new("dmenu_run").spawn().unwrap().wait().unwrap();
                    }
                    if ev.key.keycode == get_keycode(dpy, XK_Page_Up) {
                        println!("   |- Maximazing Window: {current_win}");
                        XMoveResizeWindow(dpy, current_win, 0, 0, 1920, 1080);
                        XSetWindowBorderWidth(dpy, current_win, 0);
                    }
                    if ev.key.keycode == get_keycode(dpy, XK_Tab) {
                        if clients.len() > 1 {
                            println!("   |- Cycling to previous windows...(Hopefully)");
                            println!("   |- Current clients are {:?}", clients);
                            let index = client_index.unwrap();
                            // XMoveWindow(dpy, clients[index], -1920, -1080);
                            client_index  = Some((index + 1) % clients.len());
                            let index = client_index.unwrap();
                            XRaiseWindow(dpy, clients[index]);
                            // XMoveWindow(dpy, clients[index], 0, 0);
                        } else {
                            println!("   |- No windows. Skipping")
                        }
                    }
                    if ev.key.keycode == get_keycode(dpy, XK_l) {
                        println!("   |- Current window is {current_win}");
                        println!("   |- Current Clients are {clients:?}")
                    }
                }
                if ev.key.state == ModKeyShift {
                    if ev.key.keycode == get_keycode(dpy, XK_C) {
                        println!("   |- Killing Window: {current_win}");
                        clients.retain(|&client| client != current_win);
                        XKillClient(dpy, current_win);
                    };
                    if ev.key.keycode == get_keycode(dpy, XK_Q) {
                        println!("   |- Exiting Window Manager");
                        break;
                    }
                }
            }
            if ev.type_ == ButtonPress {
                let ew = ev.button.subwindow;
                if ev.button.subwindow != 0 {
                    if ev.button.button == 2 {
                        println!("   |- Selecting Window: {ew}");
                        XRaiseWindow(dpy, ew);
                        XSetInputFocus(dpy, ew, RevertToParent, CurrentTime);
                        // add window decoration
                        // XSetWindowBorderWidth(dpy, ew, 2);
                        // XSetWindowBorder(dpy, ew, argb_to_int(0, 98, 114, 164));
                    } else {
                        println!("   |- Started Grabbing Window: {ew}");
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
                let ew: u64 = ev.motion.window;

                println!("   |- Window id: {ew}");

                if ev.button.subwindow != 0 && start.subwindow != 0 {
                    println!("   |- Resizing OR Moving Window");
                    let x_diff: i32 = ev.button.x_root - start.x_root;
                    let y_diff: i32 = ev.button.y_root - start.y_root;
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
                } else {
                    println!("   |- Just Moving");
                    // XSetInputFocus(dpy, win, RevertToNone, CurrentTime);
                }
            }
            if ev.type_ == ButtonRelease {
                start.subwindow = 0;
            }
            if ev.type_ == MapRequest {
                let ew: u64 = ev.map_request.window;
                println!("   |- Request From Window: {ew}");

                let mut wa: XSetWindowAttributes = get_default::XSetWindowAttributes();
                wa.event_mask = LeaveWindowMask
                    | EnterWindowMask
                    | SubstructureNotifyMask
                    | StructureNotifyMask;
                XChangeWindowAttributes(dpy, ew, CWEventMask | CWCursor, get_mut_ptr(&mut wa));

                // get name
                let mut c: *mut i8 = null_mut();
                if XFetchName(dpy, ew, get_mut_ptr(&mut c)) == True {
                    println!("      |- Got window name: {:?}", CStr::from_ptr(c).to_str());
                    libc::free(c as *mut libc::c_void);
                } else {
                    println!("      |- Failed to get window name");
                }
                // get class
                let ch: *mut XClassHint = XAllocClassHint();
                if XGetClassHint(dpy, ew, ch) == True {
                    println!("      |- Got window class");
                    println!(
                        "         |- name: {:?}",
                        CStr::from_ptr((*ch).res_name).to_str()
                    );
                    println!(
                        "         |- class: {:?}",
                        CStr::from_ptr((*ch).res_class).to_str()
                    );
                    XFree((*ch).res_name as *mut libc::c_void);
                    XFree((*ch).res_class as *mut libc::c_void);
                } else {
                    println!("      |- Failed To Get Window Class");
                }

                current_win = ew;
                client_index = Some(clients.len());
                clients.push(ew);

                XRaiseWindow(dpy, ew);
                XMoveResizeWindow(dpy, ew, 0, 0, 1920, 1080);
                XMapWindow(dpy, ew);
            }

            if ev.type_ == EnterNotify {
                let ew: u64 = ev.crossing.window;

                println!("      |- Window Id: {}", ew);

                let mut c: *mut i8 = null_mut();
                if XFetchName(dpy, ew, get_mut_ptr(&mut c)) == True {
                    println!(
                        "         |- Got Window Name: {:?}",
                        CStr::from_ptr(c).to_str()
                    );
                    libc::free(c as *mut libc::c_void);
                } else {
                    println!("         |- Failed to get window name");
                }

                // println!("         |- Raising window");
                // XRaiseWindow(dpy, ew);

                println!("         |- Setting focus to window");
                XSetInputFocus(dpy, ew, RevertToNone, CurrentTime);

                current_win = ew;
            }
            if ev.type_ == LeaveNotify {
                let ew: u64 = ev.crossing.window;

                println!("      |- Window id: {}", ew);
            }
            if ev.type_ == DestroyNotify {
                let ew: u64 = ev.destroy_window.window;

                println!("      |- Window [{}] destroyed", ew);
                clients.retain(|&c| c != ew);

                if clients.len() > 0 {
                    client_index = Some(client_index.unwrap() % clients.len());
                } else {
                    client_index = None;
                }
            }
        }
    }
}
