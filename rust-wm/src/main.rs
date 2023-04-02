//#![allow(non_snake_case)]
//#![allow(non_upper_case_globals)]
//#![allow(dead_code)]

mod get_default;
mod grab;
use std::process::Command;

use grab::grab_key;

mod structs;
mod wrap;

// What the fuck is going on here
fn _argb_to_int(a: u32, r: u8, g: u8, b: u8) -> u64 {
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

use wrap::xlib::Event;
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

use crate::structs::*;

const MOD_KEY_SHIFT: u32 = ModKey | x11::xlib::ShiftMask;

macro_rules! log {
    ($($e:expr),+) => {
        {
            #[cfg(debug_assertions)]
            {
                println!($($e),+)
            }
            #[cfg(not(debug_assertions))]
            {
                ($($e),+)
            }
        }
    };
}

fn setup() -> ApplicationContainer {
    log!("|===== setup =====");

    // Create some static variables
    // cause app.env.ws.vars.display holds static reference we should supply it with it during init
    let display = open_display(None).expect("Failed to open display");
    log!("|- Opened display");

    // Create Main container
    let mut app = ApplicationContainer {
        environment: EnvironmentContainer {
            config: ConfigurationContainer {
                visual_preferences: Vec::new(),
                key_actions: Vec::new(),
                layout_rules: Vec::new(),
                status_bar_builder: Vec::new(),
            },
            window_system: WindowSystemContainer {
                status_bar: StatusBarContainer {},
                display,
                root_win: 0,
                running: true,
                screens: Vec::new(),
                current_screen: 0,
                current_workspace: 0,
                current_client: None,
            },
        },
        api: Api {},
    };
    log!("|- Initialized `ApplicationContainer`");

    // TODO: Load visual_preferences

    // TODO: Load actions
    let actions: Vec<KeyAction> = {
        let mut a = vec![
            KeyAction {
                modifier: ModKey,
                keysym: XK_Return,
                result: ActionResult::Spawn("kitty".to_string()),
            },
            KeyAction {
                modifier: ModKey,
                keysym: XK_p,
                result: ActionResult::Spawn("dmenu_run".to_string()),
            },
            KeyAction {
                modifier: ModKey | ShiftMask,
                keysym: XK_Q,
                result: ActionResult::Quit,
            },
            KeyAction {
                modifier: ModKey | ShiftMask,
                keysym: XK_C,
                result: ActionResult::KillClient,
            },
            KeyAction {
                modifier: ModKey,
                keysym: XK_l,
                result: ActionResult::DumpInfo(),
            }
        ];

        for (index, key) in vec![XK_1, XK_2, XK_3, XK_4, XK_5, XK_6, XK_7, XK_8, XK_9, XK_0]
            .iter()
            .enumerate()
        {
            a.push(KeyAction {
                modifier: ModKey,
                keysym: *key,
                result: ActionResult::FocusOnWorkspace(index as u64),
            });
            a.push(KeyAction {
                modifier: ModKey | ShiftMask,
                keysym: *key,
                result: ActionResult::MoveToWorkspace(index as u64),
            });
        }
        a
    };

    app.environment.config.key_actions = actions;

    for (index, action) in app.environment.config.key_actions.iter().enumerate() {
        log!(
            "|- Grabbed {} action of type `{:?}`",
            index + 1,
            &action.result
        );
        grab_key(
            app.environment.window_system.display,
            action.keysym,
            action.modifier,
        );
    }

    log!(
        "|- Loaded {} `Actions`",
        app.environment.config.key_actions.len()
    );

    //grab_key(
    //    display,
    //    XK_Return,
    //    ModKey | ShiftMask,
    //); // Move to top
    //grab_key(
    //    display,
    //    XK_Return,
    //    ModKey,
    //); // Spawn alacritty
    //grab_key(
    //    display,
    //    XK_Q,
    //    ModKey | ShiftMask,
    //); // Exit rust-wm
    //grab_key(
    //    display,
    //    XK_p,
    //    ModKey,
    //); // Run dmenu
    //grab_key(
    //    display,
    //    XK_Page_Up,
    //    ModKey,
    //); // maximize window
    //grab_key(
    //    display,
    //    XK_C,
    //    ModKey | ShiftMask,
    //); // close window
    //grab_key(
    //    display,
    //    XK_Tab,
    //    ModKey,
    //); // Cycle Through Windows
    //grab_key(
    //    display,
    //    XK_l,
    //    ModKey,
    //); // Query current window information

    //grab_button(app.environment.window_system.display, 1, ModKey); // Move window
    //grab_button(app.environment.window_system.display, 2, ModKey); // Focus window
    //grab_button(app.environment.window_system.display, 3, ModKey); // Resize window

    // TODO: Load layout_rules

    // TODO: Load status_bar_builder

    // TODO: Init status_bar

    // Init variables
    // app.env.win.var.dis was already initialized (Proof me wrong)
    app.environment.window_system.root_win =
        default_root_window(&mut app.environment.window_system.display);

    log!("|- Initialized `Variables`");

    // Init screens
    for screen in xinerama_query_screens(app.environment.window_system.display)
        .expect("Running without xinerama is not supported")
    {
        app.environment.window_system.screens.push(Screen {
            number: screen.screen_number as i64,
            x: screen.x_org as i64,
            y: screen.y_org as i64,
            width: screen.width as i64,
            height: screen.height as i64,
            workspaces: {
                let mut wv = Vec::new();
                for i in 0..10 {
                    wv.push(Workspace {
                        number: i,
                        clients: Vec::new(),
                        current_client: None,
                        master_size: 1,
                        master_width: 0.5,
                    })
                }
                wv
            },
            current_workspace: 0,
        })
    }
    log!("|- Initialized xinerama `Screens` and nested `Workspaces`");
    // TODO: Init Api

    // Setup WM with X server info
    let mut wa: XSetWindowAttributes = get_default::xset_window_attributes();
    wa.event_mask = SubstructureRedirectMask
        | LeaveWindowMask
        | EnterWindowMask
        | SubstructureNotifyMask
        | StructureNotifyMask;

    change_window_attributes(
        app.environment.window_system.display,
        app.environment.window_system.root_win,
        CWEventMask | CWCursor,
        &mut wa,
    );

    select_input(
        app.environment.window_system.display,
        app.environment.window_system.root_win,
        wa.event_mask,
    );
    log!("|- Applied `Event mask`");

    // Return initialized container
    app
}

fn scan(config: &ConfigurationContainer, window_system: &mut WindowSystemContainer) {
    log!("|===== scan =====");
    let (mut rw, _, wins) = query_tree(window_system.display, window_system.root_win);

    log!("|- Found {} window(s) that are already present", wins.len());

    for win in wins {
        log!("   |- Checking window {win}");
        let res = get_window_attributes(window_system.display, win);
        if let Some(wa) = res {
            if wa.override_redirect != 0
                || get_transient_for_hint(window_system.display, win, &mut rw) != 0
            {
                log!("      |- Window is transient. Skipping");
                continue;
            }
            if wa.map_state == IsViewable {
                log!("      |- Window is viewable. Managing");
                manage_client(window_system, win);
                continue;
            }
        }
        log!("      |- Can't manage window");
    }
}

fn manage_client(window_system: &mut WindowSystemContainer, win: u64) {
    let mut wa: XSetWindowAttributes = get_default::xset_window_attributes();
    wa.event_mask =
        LeaveWindowMask | EnterWindowMask | SubstructureNotifyMask | StructureNotifyMask;
    change_window_attributes(window_system.display, win, CWEventMask | CWCursor, &mut wa);

    // get name
    // let mut c: *mut i8 = null_mut();
    // if XFetchName(app.environment.window_system.display, ew, get_mut_ptr(&mut c)) == True {
    //     println!("      |- Got window name: {:?}", CStr::from_ptr(c).to_str());
    //     libc::free(c as *mut libc::c_void);
    // } else {
    //     println!("      |- Failed to get window name");
    // }
    // // get class
    // let ch: *mut XClassHint = XAllocClassHint();
    // if XGetClassHint(app.environment.window_system.display, ew, ch) == True {
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

    // *cw = ew;
    // *ci = Some(clients.len());
    // clients.push(ew);

    let s = &mut window_system.screens[window_system.current_screen];
    let w = &mut s.workspaces[s.current_workspace];
    w.current_client = Some(w.clients.len());
    window_system.current_client = w.current_client;
    w.clients.push(Client {
        window_id: win,
        window_name: "Window".to_string(),
        x: 0,
        y: 0,
        w: 1920,
        h: 1080,
        visible: true,
        px: 0,
        py: 0,
    });

    raise_window(window_system.display, win);
    arrange(window_system);
    map_window(window_system.display, win);
}

fn arrange(ws: &mut WindowSystemContainer) {
    let master_width = (ws.screens[ws.current_screen].width as f64
        * ws.screens[ws.current_screen].workspaces[ws.current_workspace].master_width)
        as u32;
    let master_size = ws.screens[ws.current_screen].workspaces[ws.current_workspace].master_size;
    let screen_size = (ws.screens[ws.current_screen].width, ws.screens[ws.current_screen].height);
    let stack_size = ws.screens[ws.current_screen].workspaces[ws.current_workspace].clients.len();
    log!("   |- Arranging {} window", stack_size);
    for (index, client) in ws.screens[ws.current_screen].workspaces[ws.current_workspace].clients.iter_mut().rev().enumerate() {
        if stack_size == 1 {
            client.x = 0;
            client.y = 0;
            client.w = screen_size.0 as u32;
            client.h = screen_size.1 as u32;
        } else {
            if (index as i64) < master_size {
                log!("      |- Goes to master");
                client.x = 0;
                client.y = ((index as f64) * (screen_size.1 as f64) / (master_size as f64)) as i32;
                client.w = master_width;
                client.h = ((screen_size.1 as f64) / (master_size as f64)) as u32;
            } else {
                log!("      |- Goes to stack");
                dbg!(master_width, master_size, screen_size, stack_size, index);
                log!("hello");
                client.x = master_width as i32;
                client.y = ((index as i64 - master_size) as f64 * (screen_size.1 as f64) / (stack_size as i64 - master_size) as f64) as i32;
                client.w = screen_size.0 as u32 - master_width;
                client.h = ((screen_size.1 as f64) / (stack_size as i64 - master_size) as f64) as u32;
            }
        }
    }
    for client in &ws.screens[ws.current_screen].workspaces[ws.current_workspace].clients {
        move_resize_window(ws.display, client.window_id, client.x, client.y, client.w, client.h);
    }
}

fn update_current_client(window_system: &mut WindowSystemContainer, win: u64) {
    let s = &mut window_system.screens[window_system.current_screen];
    let w = &mut s.workspaces[s.current_workspace];
    match w.clients.iter().position(|r| r.window_id == win) {
        Some(index) => {
            w.current_client = Some(index);
            window_system.current_client = Some(index);
        },
        None => {},
    }
}

fn get_current_client_id(window_system: &mut WindowSystemContainer) -> Option<u64> {
    match window_system.current_client {
        Some(index) => Some(window_system.screens[window_system.current_screen].workspaces[window_system.current_workspace].clients[index].window_id),
        None => None,
    }
}

fn find_window_indexes(
    window_system: &mut WindowSystemContainer,
    win: u64,
) -> Option<(usize, usize, usize)> {
    for s in 0..window_system.screens.len() {
        for w in 0..window_system.screens[s].workspaces.len() {
            for c in 0..window_system.screens[s].workspaces[w].clients.len() {
                if window_system.screens[s].workspaces[w].clients[c].window_id == win {
                    return Some((s, w, c));
                }
            }
        }
    }
    return None;
}

fn show_hide_workspace(ws: &mut WindowSystemContainer) {
    for client in &mut ws.screens[ws.current_screen].workspaces[ws.current_workspace].clients {
        if client.visible {
            move_resize_window(ws.display, client.window_id, client.w as i32 * -1, client.h as i32 * -1, client.w, client.h);
        } else {
            move_resize_window(ws.display, client.window_id, client.x, client.y, client.w, client.h);
        }
        client.visible = !client.visible;
    }
}

fn run(config: &ConfigurationContainer, window_system: &mut WindowSystemContainer) {
    log!("|===== run =====");
    while window_system.running {
        // Process Events
        let ev = next_event(window_system.display);
        log!("|- Got event {}", get_event_names_list()[ev.type_ as usize]);

        match ev.type_ {
            x11::xlib::KeyPress => {
                let ev = ev.key.unwrap();
                for action in &config.key_actions {
                    if ev.keycode == keysym_to_keycode(window_system.display, action.keysym)
                        && ev.state == action.modifier
                    {
                        match &action.result {
                            ActionResult::KillClient => {
                                log!("   |- Got `KillClient` Action");
                                match get_current_client_id(window_system) {
                                    Some(id) => {
                                        log!("      |- Killing window {}", id);
                                        kill_client(window_system.display, id);
                                    }
                                    None => {
                                        log!("      |- No window selected");
                                    }
                                };
                            }
                            ActionResult::Spawn(cmd) => {
                                println!("   |- Got `Spawn` Action");
                                let mut handle = Command::new(cmd)
                                    .spawn()
                                    .expect(format!("can't execute {cmd}").as_str());
                                std::thread::spawn(move || {
                                    handle.wait().expect("can't run process");
                                });
                            }
                            ActionResult::MoveToScreen(_) => {
                                log!("   |- Action `MoveToScreen` is not currently supported");
                            }
                            ActionResult::FocusOnScreen(_) => {
                                log!("   |- Action `FocusOnScreen` is not currently supported");
                            }
                            ActionResult::MoveToWorkspace(_) => {
                                log!("   |- Action `MoveToWorkspace` is not currently supported");
                            }
                            ActionResult::FocusOnWorkspace(n) => {
                                // log!("   |- Action `FocusOnWorkspace` is not currently supported");
                                if *n as usize != window_system.current_workspace {
                                    show_hide_workspace(window_system);
                                    window_system.current_workspace = *n as usize;
                                    window_system.screens[window_system.current_screen].current_workspace = *n as usize;
                                    show_hide_workspace(window_system);
                                }
                            }
                            ActionResult::MaximazeWindow() => {
                                log!("   |- Action `MaximazeWindow` is not currently supported");
                            }
                            ActionResult::Quit => {
                                log!("   |- Got `Quit` Action. `Quiting`");
                                window_system.running = false;
                            }
                            ActionResult::UpdateMasterSize(i) => {
                                window_system.screens[window_system.current_screen].workspaces
                                    [window_system.current_workspace]
                                    .master_size += *i;
                            }
                            ActionResult::UpdateMasterWidth(w) => {
                                window_system.screens[window_system.current_screen].workspaces
                                    [window_system.current_workspace]
                                    .master_width += *w;
                            }
                            ActionResult::DumpInfo() => {
                                log!("{:#?}", window_system.screens);
                            },
                        }
                    }
                }
            }

            x11::xlib::MapRequest => {
                let ew: u64 = ev.map_request.unwrap().window;
                log!("|- Map Request From Window: {ew}");
                manage_client(window_system, ew);
            }

            x11::xlib::EnterNotify => {
                let ew: u64 = ev.crossing.unwrap().window;
                log!("|- Crossed Window {}", ew);
                log!("   |- Setting focus to window");
                set_input_focus(window_system.display, ew, RevertToNone, CurrentTime);
                update_current_client(window_system, ew);
            }

            x11::xlib::DestroyNotify => {
                let ew: u64 = ev.destroy_window.unwrap().window;
                log!("|- Window {} destroyed", ew);
                if let Some((s, w, c)) = find_window_indexes(window_system, ew) {
                    log!("   |- Found window {} at indexes {}, {}, {}", ew, s, w, c);
                    let clients = &mut window_system.screens[s].workspaces[w].clients;
                    clients.remove(c);
                    window_system.screens[s].workspaces[w].current_client = {
                        let clients = &window_system.screens[s].workspaces[w].clients;
                        if clients.len() == 1 {
                            Some(0)
                        } else if clients.len() == 0 {
                            None
                        } else {
                            let cc = window_system.screens[s].workspaces[w]
                                .current_client
                                .expect("WHAT THE FUCK");
                            if cc < clients.len() {
                                Some(cc)
                            } else {
                                Some(cc - 1)
                            }
                        }
                    };
                    window_system.current_client = window_system.screens[s].workspaces[w].current_client;
                    arrange(window_system);
                } else {
                    log!("   |- Window is not managed");
                }
            }
            _ => {}
        };

        // if ev.type_ == MapRequest {}

        // if ev.type_ == EnterNotify {
        //     let ew: u64 = ev.crossing.unwrap().window;

        //     println!("      |- Window Id: {}", ew);

        //     // let mut c: *mut i8 = null_mut();
        //     // if XFetchName(app.environment.window_system.display, ew, get_mut_ptr(&mut c)) == True {
        //     //     println!(
        //     //         "         |- Got Window Name: {:?}",
        //     //         CStr::from_ptr(c).to_str()
        //     //     );
        //     //     libc::free(c as *mut libc::c_void);
        //     // } else {
        //     //     println!("         |- Failed to get window name");
        //     // }

        //     // println!("         |- Raising window");
        //     // XRaiseWindow(app.environment.window_system.display, ew);
        // }
        // if ev.type_ == LeaveNotify {
        //     let ew: u64 = ev.crossing.unwrap().window;

        //     println!("      |- Window id: {}", ew);
        // }
        // if ev.type_ == DestroyNotify {}
        // println!("   |- Got Event Of Type \"{}\"", events[ev.type_ as usize]);
        //if ev.type_ == KeyPress {
        //    let key = ev.key.unwrap();
        //    let _ew: u64 = key.window;

        //    if key.state == ModKey {
        //        if key.keycode
        //            == keysym_to_keycode(app.environment.window_system.display, XK_Return)
        //        {
        //            println!("   |- Spawning Terminal");
        //            let mut handle = Command::new("kitty").spawn().expect("can't run kitty");
        //            std::thread::spawn(move || {
        //                handle.wait().expect("can't run process");
        //            });
        //        }
        //        if key.keycode
        //            == keysym_to_keycode(app.environment.window_system.display, XK_p)
        //        {
        //            println!("   |- Spawning Dmenu");
        //            Command::new("dmenu_run").spawn().unwrap().wait().unwrap();
        //        }
        //        if key.keycode
        //            == keysym_to_keycode(
        //                app.environment.window_system.display,
        //                XK_Page_Up,
        //            )
        //        {
        //            println!("   |- Maximazing Window: {current_win}");
        //            move_resize_window(
        //                app.environment.window_system.display,
        //                current_win,
        //                0,
        //                0,
        //                1920,
        //                1080,
        //            );
        //            set_window_border_width(
        //                app.environment.window_system.display,
        //                current_win,
        //                0,
        //            );
        //        }
        //        if key.keycode
        //            == keysym_to_keycode(app.environment.window_system.display, XK_Tab)
        //        {
        //            if clients.len() > 1 {
        //                println!("   |- Cycling to previous windows...(Hopefully)");
        //                println!("   |- Current clients are {:?}", clients);
        //                let index = client_index.unwrap();
        //                // XMoveWindow(app.environment.window_system.display, clients[index], -1920, -1080);
        //                client_index = Some((index + 1) % clients.len());
        //                let index = client_index.unwrap();
        //                raise_window(
        //                    app.environment.window_system.display,
        //                    clients[index],
        //                );
        //                // XMoveWindow(app.environment.window_system.display, clients[index], 0, 0);
        //            } else {
        //                println!("   |- No windows. Skipping")
        //            }
        //        }
        //        if key.keycode
        //            == keysym_to_keycode(app.environment.window_system.display, XK_l)
        //        {
        //            println!("   |- Current window is {current_win}");
        //            println!("   |- Current Clients are {clients:?}")
        //        }
        //    }
        //    if key.state == MOD_KEY_SHIFT {
        //        if key.keycode
        //            == keysym_to_keycode(app.environment.window_system.display, XK_C)
        //        {
        //            println!("   |- Killing Window: {current_win}");
        //            clients.retain(|&client| client != current_win);
        //            kill_client(app.environment.window_system.display, current_win);
        //        };
        //        if key.keycode
        //            == keysym_to_keycode(app.environment.window_system.display, XK_Q)
        //        {
        //            println!("   |- Exiting Window Manager");
        //            break;
        //        }
        //    }
        //}
        //if ev.type_ == ButtonPress {
        //    let button = ev.button.unwrap();
        //    let ew = button.subwindow;
        //    if button.subwindow != 0 {
        //        if button.button == 2 {
        //            println!("   |- Selecting Window: {ew}");
        //            raise_window(app.environment.window_system.display, ew);
        //            set_input_focus(
        //                app.environment.window_system.display,
        //                ew,
        //                RevertToParent,
        //                CurrentTime,
        //            );
        //            // add window decoration
        //            // XSetWindowBorderWidth(app.environment.window_system.display, ew, 2);
        //            // XSetWindowBorder(app.environment.window_system.display, ew, argb_to_int(0, 98, 114, 164));
        //        } else {
        //            println!("   |- Started Grabbing Window: {ew}");
        //            attr = get_window_attributes(
        //                app.environment.window_system.display,
        //                button.subwindow,
        //            )
        //            .unwrap();
        //            start = button;
        //        }
        //    }
        //}
        //if ev.type_ == MotionNotify {
        //    let motion = ev.motion.unwrap();
        //    let button = ev.button.unwrap();
        //    let ew: u64 = motion.window;

        //    println!("   |- Window id: {ew}");

        //    if button.subwindow != 0 && start.subwindow != 0 {
        //        println!("   |- Resizing OR Moving Window");
        //        let x_diff: i32 = button.x_root - start.x_root;
        //        let y_diff: i32 = button.y_root - start.y_root;
        //        move_resize_window(
        //            app.environment.window_system.display,
        //            start.subwindow,
        //            attr.x + {
        //                if start.button == 1 {
        //                    x_diff
        //                } else {
        //                    0
        //                }
        //                // Get u32 keycode from keysym
        //            },
        //            attr.y + {
        //                if start.button == 1 {
        //                    y_diff
        //                } else {
        //                    0
        //                }
        //            },
        //            1.max(
        //                (attr.width + {
        //                    if start.button == 3 {
        //                        x_diff
        //                    } else {
        //                        0
        //                    }
        //                }) as u32,
        //            ),
        //            1.max(
        //                (attr.height + {
        //                    if start.button == 3 {
        //                        y_diff
        //                    } else {
        //                        0
        //                    }
        //                }) as u32,
        //            ),
        //        );
        //    } else {
        //        println!("   |- Just Moving");
        //        // XSetInputFocus(app.environment.window_system.display, win, RevertToNone, CurrentTime);
        //    }
        //}
        //if ev.type_ == ButtonRelease {
        //    start.subwindow = 0;
        //}
    }
}

fn cleanup(app: &mut ApplicationContainer) {}

fn main() {
    // Create variables
    // let mut events: Vec<&str>;
    // let mut app.environment.window_system.display: &mut Display;
    // let mut root_win: u64;
    // let mut client: Vec<u64>;
    // let mut monitors: Vec<(u64, i64, i64, u64, u64)>;

    // println!("Started Window Manager");
    //    unsafe {
    // let events: Vec<&str> = get_event_names_list();
    // println!("|- Created Event Look-Up Array");

    // let app.environment.window_system.display: &mut Display = open_display(None).expect("Error opening display!");
    // println!("|- Opened X Display");

    // let root_win: u64 = default_root_window(app.environment.window_system.display);
    // println!(
    // "|- Root window is {}",
    // app.environment.window_system.root_win
    // );

    // let mut attr: XWindowAttributes = get_default::xwindow_attributes();
    // let mut start: XButtonEvent = get_default::xbutton_event();
    // start.subwindow = 0;

    // let mut clients: Vec<u64> = Vec::new();
    // let mut client_index: Option<usize> = None;
    // let mut current_win: u64 = 0;

    // println!("|- Created Useful Variables");

    // println!("|- Applied Event Mask");

    // println!("|- Grabbed Shortcuts");
    // println!("|- Starting Main Loop");

    // Init `app` container
    let mut app: ApplicationContainer = setup();

    // Scan for existing windows
    scan(&app.environment.config, &mut app.environment.window_system);

    // start main loop
    run(&app.environment.config, &mut app.environment.window_system);

    // close all connections, dump data, exit
    cleanup(&mut app);
}
