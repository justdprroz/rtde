//#![allow(non_snake_case)]
//#![allow(non_upper_case_globals)]
//#![allow(dead_code)]

mod get_default;
mod grab;
use std::process::Command;

use grab::grab_button;
use grab::grab_key;

mod wrap;
mod structs;

// What the fuck is going on here
fn _argb_to_int(a: u32, r: u8, g: u8, b: u8) -> u64 {
    (a as u64) << 24 | (r as u64) << 16 | (g as u64) << 8 | (b as u64)
}

fn manage_client(
    dpy: &mut Display,
    ew: u64,
    cw: &mut u64,
    ci: &mut Option<usize>,
    clients: &mut Vec<u64>,
) {
    let mut wa: XSetWindowAttributes = get_default::xset_window_attributes();
    wa.event_mask =
        LeaveWindowMask | EnterWindowMask | SubstructureNotifyMask | StructureNotifyMask;
    change_window_attributes(dpy, ew, CWEventMask | CWCursor, &mut wa);

    // get name
    // let mut c: *mut i8 = null_mut();
    // if XFetchName(dpy, ew, get_mut_ptr(&mut c)) == True {
    //     println!("      |- Got window name: {:?}", CStr::from_ptr(c).to_str());
    //     libc::free(c as *mut libc::c_void);
    // } else {
    //     println!("      |- Failed to get window name");
    // }
    // // get class
    // let ch: *mut XClassHint = XAllocClassHint();
    // if XGetClassHint(dpy, ew, ch) == True {
    //     println!("      |- Got window class");
    //     println!(
    //         "         |- name: {:?}",
    //         CStr::from_ptr((*ch).res_name).to_str()
    //     );
    //     println!(
    //         "         |- class: {:?}",
    //         CStr::from_ptr((*ch).res_class).to_str()
    //     );
    //     XFree((*ch).res_name as *mut libc::c_void);
    //     XFree((*ch).res_class as *mut libc::c_void);
    // } else {
    //     println!("      |- Failed To Get Window Class");
    // }

    *cw = ew;
    *ci = Some(clients.len());
    clients.push(ew);

    raise_window(dpy, ew);
    move_resize_window(dpy, ew, 0, 0, 1920, 1080);
    map_window(dpy, ew);
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

use x11::keysym::*;
use x11::xlib::ButtonPress;
use x11::xlib::ButtonRelease;
use x11::xlib::CWCursor;
use x11::xlib::CWEventMask;
use x11::xlib::CurrentTime;
use x11::xlib::DestroyNotify;
use x11::xlib::Display;
use x11::xlib::EnterNotify;
use x11::xlib::EnterWindowMask;
use x11::xlib::IsViewable;
use x11::xlib::KeyPress;
use x11::xlib::LeaveNotify;
use x11::xlib::LeaveWindowMask;
use x11::xlib::MapRequest;
use x11::xlib::Mod1Mask as ModKey;
use x11::xlib::MotionNotify;
use x11::xlib::RevertToNone;
use x11::xlib::RevertToParent;
use x11::xlib::ShiftMask;
use x11::xlib::StructureNotifyMask;
use x11::xlib::SubstructureNotifyMask;
use x11::xlib::SubstructureRedirectMask;
use x11::xlib::XButtonEvent;
use x11::xlib::XSetWindowAttributes;
use x11::xlib::XWindowAttributes;

use crate::wrap::xinerama::xinerama_query_screens;
use crate::wrap::xlib::change_window_attributes;
use crate::wrap::xlib::default_root_window;
use crate::wrap::xlib::get_transient_for_hint;
use crate::wrap::xlib::get_window_attributes;
use crate::wrap::xlib::keysym_to_keycode;
use crate::wrap::xlib::kill_client;
use crate::wrap::xlib::map_window;
use crate::wrap::xlib::move_resize_window;
use crate::wrap::xlib::next_event;
use crate::wrap::xlib::open_display;
use crate::wrap::xlib::query_tree;
use crate::wrap::xlib::raise_window;
use crate::wrap::xlib::select_input;
use crate::wrap::xlib::set_input_focus;
use crate::wrap::xlib::set_window_border_width;

const MOD_KEY_SHIFT: u32 = ModKey | x11::xlib::ShiftMask;

fn setup() {

}

fn scan() {

}

fn run() {

}

fn main() {
    // Create variables
    let mut events: Vec<&str>;
    let mut dpy: &mut Display;
    let mut root_win: u64;
    let mut client: Vec<u64>;
    let mut monitors: Vec<(u64, i64, i64, u64, u64)>;

    // println!("Started Window Manager");
    //    unsafe {
    let events: Vec<&str> = get_event_names_list();
    println!("|- Created Event Look-Up Array");

    let dpy: &mut Display = open_display(None).expect("Error opening display!");
    println!("|- Opened X Display");

    let root_win: u64 = default_root_window(dpy);
    println!("|- Root window is {}", root_win);

    println!("|- Getting per monitor sizes");
    let screens = xinerama_query_screens(dpy).expect("Running without xinerama is not supported");
    println!("|- There are {} screen connected", screens.len());
    for screen in screens {
        println!(
            "|- Screen {} has size of {}x{} pixels and originates from {},{}",
            screen.screen_number, screen.width, screen.height, screen.x_org, screen.y_org
        );
    }

    let mut attr: XWindowAttributes = get_default::xwindow_attributes();
    let mut start: XButtonEvent = get_default::xbutton_event();
    start.subwindow = 0;

    let mut clients: Vec<u64> = Vec::new();
    let mut client_index: Option<usize> = None;
    let mut current_win: u64 = 0;

    println!("|- Created Useful Variables");

    let mut wa: XSetWindowAttributes = get_default::xset_window_attributes();

    // wa.event_mask = LeaveWindowMask | EnterWindowMask | SubstructureNotifyMask | StructureNotifyMask;
    wa.event_mask = SubstructureRedirectMask
        | LeaveWindowMask
        | EnterWindowMask
        | SubstructureNotifyMask
        | StructureNotifyMask;

    change_window_attributes(dpy, root_win, CWEventMask | CWCursor, &mut wa);

    select_input(dpy, root_win, wa.event_mask);

    println!("|- Applied Event Mask");

    let (mut rw, _, wins) = query_tree(dpy, root_win);

    println!("|- {} windows are alredy present", clients.len());

    for win in wins {
        println!("|-- Checking window {win}");
        let res = get_window_attributes(dpy, win);
        if let Some(wa) = res {
            if wa.override_redirect != 0 || get_transient_for_hint(dpy, win, &mut rw) != 0 {
                println!("|---- Window is transient. Skipping");
                continue;
            }
            if wa.map_state == IsViewable {
                println!("|---- Window is viewable. Managing");
                manage_client(dpy, win, &mut current_win, &mut client_index, &mut clients);
                continue;
            }
        }
        println!("|---- Can't manage window")
    }

    grab_key(dpy, XK_Return, ModKey | ShiftMask); // Move to top
    grab_key(dpy, XK_Return, ModKey); // Spawn alacritty
    grab_key(dpy, XK_Q, ModKey | ShiftMask); // Exit rust-wm
    grab_key(dpy, XK_p, ModKey); // Run dmenu
    grab_key(dpy, XK_Page_Up, ModKey); // maximize window
    grab_key(dpy, XK_C, ModKey | ShiftMask); // close window
    grab_key(dpy, XK_Tab, ModKey); // Cycle Through Windows
    grab_key(dpy, XK_l, ModKey); // Query current window information

    grab_button(dpy, 1, ModKey); // Move window
    grab_button(dpy, 2, ModKey); // Focus window
    grab_button(dpy, 3, ModKey); // Resize window

    println!("|- Grabbed Shortcuts");
    println!("|- Starting Main Loop");
    loop {
        let ev = next_event(dpy);
        println!("   |- Got Event Of Type \"{}\"", events[ev.type_ as usize]);
        if ev.type_ == KeyPress {
            let key = ev.key.unwrap();
            let _ew: u64 = key.window;

            if key.state == ModKey {
                if key.keycode == keysym_to_keycode(dpy, XK_Return) {
                    println!("   |- Spawning Terminal");
                    let mut handle = Command::new("kitty").spawn().expect("can't run kitty");
                    std::thread::spawn(move || {
                        handle.wait().expect("can't run process");
                    });
                }
                if key.keycode == keysym_to_keycode(dpy, XK_p) {
                    println!("   |- Spawning Dmenu");
                    Command::new("dmenu_run").spawn().unwrap().wait().unwrap();
                }
                if key.keycode == keysym_to_keycode(dpy, XK_Page_Up) {
                    println!("   |- Maximazing Window: {current_win}");
                    move_resize_window(dpy, current_win, 0, 0, 1920, 1080);
                    set_window_border_width(dpy, current_win, 0);
                }
                if key.keycode == keysym_to_keycode(dpy, XK_Tab) {
                    if clients.len() > 1 {
                        println!("   |- Cycling to previous windows...(Hopefully)");
                        println!("   |- Current clients are {:?}", clients);
                        let index = client_index.unwrap();
                        // XMoveWindow(dpy, clients[index], -1920, -1080);
                        client_index = Some((index + 1) % clients.len());
                        let index = client_index.unwrap();
                        raise_window(dpy, clients[index]);
                        // XMoveWindow(dpy, clients[index], 0, 0);
                    } else {
                        println!("   |- No windows. Skipping")
                    }
                }
                if key.keycode == keysym_to_keycode(dpy, XK_l) {
                    println!("   |- Current window is {current_win}");
                    println!("   |- Current Clients are {clients:?}")
                }
            }
            if key.state == MOD_KEY_SHIFT {
                if key.keycode == keysym_to_keycode(dpy, XK_C) {
                    println!("   |- Killing Window: {current_win}");
                    clients.retain(|&client| client != current_win);
                    kill_client(dpy, current_win);
                };
                if key.keycode == keysym_to_keycode(dpy, XK_Q) {
                    println!("   |- Exiting Window Manager");
                    break;
                }
            }
        }
        if ev.type_ == ButtonPress {
            let button = ev.button.unwrap();
            let ew = button.subwindow;
            if button.subwindow != 0 {
                if button.button == 2 {
                    println!("   |- Selecting Window: {ew}");
                    raise_window(dpy, ew);
                    set_input_focus(dpy, ew, RevertToParent, CurrentTime);
                    // add window decoration
                    // XSetWindowBorderWidth(dpy, ew, 2);
                    // XSetWindowBorder(dpy, ew, argb_to_int(0, 98, 114, 164));
                } else {
                    println!("   |- Started Grabbing Window: {ew}");
                    attr = get_window_attributes(dpy, button.subwindow).unwrap();
                    start = button;
                }
            }
        }
        if ev.type_ == MotionNotify {
            let motion = ev.motion.unwrap();
            let button = ev.button.unwrap();
            let ew: u64 = motion.window;

            println!("   |- Window id: {ew}");

            if button.subwindow != 0 && start.subwindow != 0 {
                println!("   |- Resizing OR Moving Window");
                let x_diff: i32 = button.x_root - start.x_root;
                let y_diff: i32 = button.y_root - start.y_root;
                move_resize_window(
                    dpy,
                    start.subwindow,
                    attr.x + {
                        if start.button == 1 {
                            x_diff
                        } else {
                            0
                        }
                        // Get u32 keycode from keysym
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
            let ew: u64 = ev.map_request.unwrap().window;
            manage_client(dpy, ew, &mut current_win, &mut client_index, &mut clients);
            println!("   |- Request From Window: {ew}");
        }

        if ev.type_ == EnterNotify {
            let ew: u64 = ev.crossing.unwrap().window;

            println!("      |- Window Id: {}", ew);

            // let mut c: *mut i8 = null_mut();
            // if XFetchName(dpy, ew, get_mut_ptr(&mut c)) == True {
            //     println!(
            //         "         |- Got Window Name: {:?}",
            //         CStr::from_ptr(c).to_str()
            //     );
            //     libc::free(c as *mut libc::c_void);
            // } else {
            //     println!("         |- Failed to get window name");
            // }

            // println!("         |- Raising window");
            // XRaiseWindow(dpy, ew);

            println!("         |- Setting focus to window");
            set_input_focus(dpy, ew, RevertToNone, CurrentTime);

            current_win = ew;
        }
        if ev.type_ == LeaveNotify {
            let ew: u64 = ev.crossing.unwrap().window;

            println!("      |- Window id: {}", ew);
        }
        if ev.type_ == DestroyNotify {
            let ew: u64 = ev.destroy_window.unwrap().window;

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
