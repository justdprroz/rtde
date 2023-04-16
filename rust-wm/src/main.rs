//#![allow(non_snake_case)]
//#![allow(non_upper_case_globals)]
//#![allow(dead_code)]

mod get_default;
mod grab;
use std::io::WriterPanicked;
use std::process::Command;
use std::ptr::null_mut;
use std::vec;

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

use libc::LC_CTYPE;
use wrap::xlib::get_text_property;
use wrap::xlib::get_wm_protocols;
use wrap::xlib::intern_atom;
use wrap::xlib::send_event;
use wrap::xlib::Event;
use wrap::xlib::set_locale;
use x11::keysym::*;
use x11::xlib::ButtonPressMask;
use x11::xlib::CWCursor;
use x11::xlib::CWEventMask;
use x11::xlib::ClientMessage;
use x11::xlib::CurrentTime;
use x11::xlib::EnterWindowMask;
use x11::xlib::FocusChangeMask;
use x11::xlib::IsViewable;
use x11::xlib::LeaveWindowMask;
use x11::xlib::Mod1Mask as ModKey;
use x11::xlib::NoEventMask;
use x11::xlib::PointerMotionMask;
use x11::xlib::PropertyChangeMask;
use x11::xlib::RevertToNone;
use x11::xlib::ShiftMask;
use x11::xlib::StructureNotifyMask;
use x11::xlib::SubstructureNotifyMask;
use x11::xlib::SubstructureRedirectMask;
use x11::xlib::XSetWindowAttributes;
use x11::xlib::XSupportsLocale;

use crate::wrap::xinerama::xinerama_query_screens;
use crate::wrap::xlib::change_property;
use crate::wrap::xlib::change_window_attributes;
use crate::wrap::xlib::default_root_window;
use crate::wrap::xlib::get_transient_for_hint;
use crate::wrap::xlib::get_window_attributes;
use crate::wrap::xlib::keysym_to_keycode;
use crate::wrap::xlib::map_window;
use crate::wrap::xlib::move_resize_window;
use crate::wrap::xlib::next_event;
use crate::wrap::xlib::open_display;
use crate::wrap::xlib::query_tree;
use crate::wrap::xlib::raise_window;
use crate::wrap::xlib::select_input;
use crate::wrap::xlib::set_input_focus;

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
                modifier: 0,
                keysym: XF86XK_AudioRaiseVolume,
                result: ActionResult::Spawn("/usr/local/bin/volumeup".to_string()),
            },
            KeyAction {
                modifier: 0,
                keysym: XF86XK_AudioLowerVolume,
                result: ActionResult::Spawn("/usr/local/bin/volumedown".to_string()),
            },
            KeyAction {
                modifier: 0,
                keysym: XF86XK_AudioMute,
                result: ActionResult::Spawn("/usr/local/bin/volumemute".to_string()),
            },
            KeyAction {
                modifier: 0,
                keysym: XF86XK_AudioPlay,
                result: ActionResult::Spawn("playerctl play-pause".to_string()),
            },
            KeyAction {
                modifier: 0,
                keysym: XF86XK_AudioNext,
                result: ActionResult::Spawn("playerctl next".to_string()),
            },
            KeyAction {
                modifier: 0,
                keysym: XF86XK_AudioPrev,
                result: ActionResult::Spawn("playerctl previous".to_string()),
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
            },
            KeyAction {
                modifier: ModKey,
                keysym: XK_comma,
                result: ActionResult::FocusOnScreen(ScreenSwitching::Previous),
            },
            KeyAction {
                modifier: ModKey,
                keysym: XK_period,
                result: ActionResult::FocusOnScreen(ScreenSwitching::Next),
            },
            KeyAction {
                modifier: ModKey | ShiftMask,
                keysym: XK_comma,
                result: ActionResult::MoveToScreen(ScreenSwitching::Previous),
            },
            KeyAction {
                modifier: ModKey | ShiftMask,
                keysym: XK_period,
                result: ActionResult::MoveToScreen(ScreenSwitching::Next),
            },
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
    log!("{:#?}", app.environment.window_system.screens);
    log!("|- Initialized xinerama `Screens` and nested `Workspaces`");
    // TODO: Init Api

    // Setup WM with X server info

    let mut netatoms: Vec<x11::xlib::Atom> = vec![];
    for name in [
        "_NET_ACTIVE_WINDOW",
        "_NET_SUPPORTED",
        "_NET_WM_NAME",
        "_NET_SUPPORTING_WM_CHECK",
        "_NET_WM_STATE_FULLSCREEN",
        "_NET_WM_WINDOW_TYPE",
        "_NET_WM_WINDOW_TYPE_DIALOG",
        "_NET_CLIENT_LIST",
        "_NET_WM_STATE",
    ] {
        netatoms.push(intern_atom(
            app.environment.window_system.display,
            name.to_string(),
            false,
        ));
    }

    let net_supported = intern_atom(app.environment.window_system.display, "_NET_SUPPORTED".to_string(), false);
    change_property(
        app.environment.window_system.display,
        app.environment.window_system.root_win,
        net_supported,
        x11::xlib::XA_ATOM,
        32,
        x11::xlib::PropModeReplace,
        &mut netatoms,
    );

    let mut wa: XSetWindowAttributes = get_default::xset_window_attributes();
    wa.event_mask = SubstructureRedirectMask
        | LeaveWindowMask
        | EnterWindowMask
        | SubstructureNotifyMask
        | StructureNotifyMask
        | PointerMotionMask
        | ButtonPressMask
        | PropertyChangeMask;

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

fn scan(_config: &ConfigurationContainer, window_system: &mut WindowSystemContainer) {
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

fn update_client_name(window_system: &mut WindowSystemContainer, win: u64) {
    let name_atom = intern_atom(window_system.display, "_NET_WM_NAME".to_string(), false);
    let name = match get_text_property(window_system.display, win, name_atom) {
        Some(name) => name,
        None => "WHAT THE FUCK".to_string(),
    };

    if let Some((s, w, c)) = find_window_indexes(window_system, win) {
        window_system.screens[s].workspaces[w].clients[c].window_name = name;
    }
}

fn get_client_name(window_system: &mut WindowSystemContainer, win: u64) -> Option<String> {
    if let Some((s, w, c)) = find_window_indexes(window_system, win) {
        Some(window_system.screens[s].workspaces[w].clients[c].window_name.clone())
    } else {
        None
    }
}

fn manage_client(window_system: &mut WindowSystemContainer, win: u64) {
    select_input(
        window_system.display,
        win,
        EnterWindowMask | FocusChangeMask | PropertyChangeMask | StructureNotifyMask,
    );
    let s = &mut window_system.screens[window_system.current_screen];
    let w = &mut s.workspaces[s.current_workspace];
    w.current_client = Some(w.clients.len());
    window_system.current_client = w.current_client;
    w.clients.push(Client {
        window_id: win,
        window_name: "Unmanaged window".to_string(),
        x: 0,
        y: 0,
        w: 1920,
        h: 1080,
        visible: true,
        px: 0,
        py: 0,
    });
    update_client_name(window_system, win);
    raise_window(window_system.display, win);
    arrange(window_system);
    map_window(window_system.display, win);
}

fn arrange(ws: &mut WindowSystemContainer) {
    log!("   |- Arranging...");
    for screen in &mut ws.screens {
        let master_width =
            (screen.width as f64 * screen.workspaces[screen.current_workspace].master_width) as u32;
        let master_size = screen.workspaces[screen.current_workspace].master_size;
        let stack_size = screen.workspaces[screen.current_workspace].clients.len();
        log!("   |- Arranging {} window", stack_size);
        for (index, client) in screen.workspaces[screen.current_workspace]
            .clients
            .iter_mut()
            .rev()
            .enumerate()
        {
            if stack_size == 1 {
                client.x = 0;
                client.y = 0;
                client.w = screen.width as u32;
                client.h = screen.height as u32;
            } else {
                if (index as i64) < master_size {
                    log!("      |- Goes to master");
                    client.x = 0;
                    client.y =
                        ((index as f64) * (screen.height as f64) / (master_size as f64)) as i32;
                    client.w = master_width;
                    client.h = ((screen.height as f64) / (master_size as f64)) as u32;
                } else {
                    log!("      |- Goes to stack");
                    client.x = master_width as i32;
                    client.y = ((index as i64 - master_size) as f64 * (screen.height as f64)
                        / (stack_size as i64 - master_size) as f64)
                        as i32;
                    client.w = screen.width as u32 - master_width;
                    client.h =
                        ((screen.height as f64) / (stack_size as i64 - master_size) as f64) as u32;
                }
            }
        }
        for client in &screen.workspaces[screen.current_workspace].clients {
            move_resize_window(
                ws.display,
                client.window_id,
                client.x + screen.x as i32,
                client.y + screen.y as i32,
                client.w,
                client.h,
            );
        }
    }
}

fn update_current_client(window_system: &mut WindowSystemContainer, win: u64) {
    let s = &mut window_system.screens[window_system.current_screen];
    let w = &mut s.workspaces[s.current_workspace];
    match w.clients.iter().position(|r| r.window_id == win) {
        Some(index) => {
            w.current_client = Some(index);
            window_system.current_client = Some(index);
        }
        None => {}
    }
}

fn get_current_client_id(window_system: &mut WindowSystemContainer) -> Option<u64> {
    match window_system.current_client {
        Some(index) => Some(
            window_system.screens[window_system.current_screen].workspaces
                [window_system.current_workspace]
                .clients[index]
                .window_id,
        ),
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
            move_resize_window(
                ws.display,
                client.window_id,
                client.w as i32 * -1,
                client.h as i32 * -1,
                client.w,
                client.h,
            );
        } else {
            move_resize_window(
                ws.display,
                client.window_id,
                client.x,
                client.y,
                client.w,
                client.h,
            );
        }
        client.visible = !client.visible;
    }
}

fn shift_current_client(ws: &mut WindowSystemContainer) {
    ws.screens[ws.current_screen].workspaces[ws.current_workspace].current_client = {
        let clients = &ws.screens[ws.current_screen].workspaces[ws.current_workspace].clients;
        if clients.len() == 1 {
            Some(0)
        } else if clients.len() == 0 {
            None
        } else {
            let cc = ws.screens[ws.current_screen].workspaces[ws.current_workspace]
                .current_client
                .expect("WHAT THE FUCK");
            if cc < clients.len() {
                Some(cc)
            } else {
                Some(cc - 1)
            }
        }
    };
    ws.current_client =
        ws.screens[ws.current_screen].workspaces[ws.current_workspace].current_client;
}

fn send_atom(ws: &mut WindowSystemContainer, win: u64, e: x11::xlib::Atom) -> bool {
    if let Some(ps) = get_wm_protocols(ws.display, win) {
        for p in ps {
            if p == e {
                let mut ev = Event {
                    type_: ClientMessage,
                    button: None,
                    crossing: None,
                    key: None,
                    map_request: None,
                    destroy_window: None,
                    motion: None,
                    unmap: None,
                    property: None,
                    client: Some(x11::xlib::XClientMessageEvent {
                        type_: ClientMessage,
                        serial: 0,
                        send_event: 0,
                        display: null_mut(),
                        window: win,
                        message_type: intern_atom(ws.display, "WM_PROTOCOLS".to_string(), false),
                        format: 32,
                        data: {
                            let mut d = x11::xlib::ClientMessageData::new();
                            d.set_long(0, e as i64);
                            d.set_long(1, CurrentTime as i64);
                            d
                        },
                    }),
                };
                send_event(ws.display, win, false, NoEventMask, &mut ev);
                return true;
            }
        }
        return false;
    } else {
        return false;
    }
}

fn unmanage_window(ws: &mut WindowSystemContainer, win: u64) {
    if let Some((s, w, c)) = find_window_indexes(ws, win) {
        log!("   |- Found window {} at indexes {}, {}, {}", win, s, w, c);
        let clients = &mut ws.screens[s].workspaces[w].clients;
        clients.remove(c);
        shift_current_client(ws);
        arrange(ws);
    } else {
        log!("   |- Window is not managed");
    }
}

fn run(config: &ConfigurationContainer, window_system: &mut WindowSystemContainer) {
    log!("|===== run =====");
    while window_system.running {
        // Process Events
        let ev = next_event(window_system.display);
        // log!("|- Got event {}", get_event_names_list()[ev.type_ as usize]);

        match ev.type_ {
            x11::xlib::KeyPress => {
                let ev = ev.key.unwrap();
                for action in &config.key_actions {
                    if ev.keycode == keysym_to_keycode(window_system.display, action.keysym)
                        && ev.state == action.modifier
                    {
                        log!("   |- Got {:?} action", action.result);
                        match &action.result {
                            ActionResult::KillClient => {
                                log!("   |- Got `KillClient` Action");
                                match get_current_client_id(window_system) {
                                    Some(id) => {
                                        log!("      |- Killing window {}", id);
                                        let a = intern_atom(
                                            window_system.display,
                                            "WM_DELETE_WINDOW".to_string(),
                                            false,
                                        );
                                        send_atom(window_system, id, a);
                                        // kill_client(window_system.display, id);
                                    }
                                    None => {
                                        log!("      |- No window selected");
                                    }
                                };
                            }
                            ActionResult::Spawn(cmd) => {
                                println!("   |- Got `Spawn` Action");
                                let mut handle = Command::new("/usr/bin/sh")
                                    .arg("-c")
                                    .arg(cmd)
                                    .spawn()
                                    .expect(format!("can't execute {cmd}").as_str());
                                std::thread::spawn(move || {
                                    handle.wait().expect("can't run process");
                                });
                            }
                            ActionResult::MoveToScreen(d) => {
                                if let Some(index) = window_system.current_client {
                                    let mut cs = window_system.current_screen;
                                    cs = match d {
                                        ScreenSwitching::Next => {
                                            (cs + 1) % window_system.screens.len()
                                        }
                                        ScreenSwitching::Previous => {
                                            (cs + window_system.screens.len() - 1)
                                                % window_system.screens.len()
                                        }
                                    };
                                    let cc = window_system.screens[window_system.current_screen]
                                        .workspaces[window_system.current_workspace]
                                        .clients
                                        .remove(index);
                                    shift_current_client(window_system);
                                    let nw = window_system.screens[cs].current_workspace;
                                    window_system.screens[cs].workspaces[nw].clients.push(cc);
                                    arrange(window_system);
                                }
                            }
                            ActionResult::FocusOnScreen(d) => {
                                let mut cs = window_system.current_screen;
                                cs = match d {
                                    ScreenSwitching::Next => (cs + 1) % window_system.screens.len(),
                                    ScreenSwitching::Previous => {
                                        (cs + window_system.screens.len() - 1)
                                            % window_system.screens.len()
                                    }
                                };
                                window_system.current_screen = cs;
                                window_system.current_workspace = window_system.screens
                                    [window_system.current_screen]
                                    .current_workspace;
                            }
                            ActionResult::MoveToWorkspace(n) => {
                                log!("   |- Got `MoveToWorkspace` Action ");
                                if *n as usize != window_system.current_workspace {
                                    if let Some(index) = window_system.current_client {
                                        let mut cc = window_system.screens
                                            [window_system.current_screen]
                                            .workspaces[window_system.current_workspace]
                                            .clients
                                            .remove(index);
                                        arrange(window_system);
                                        move_resize_window(
                                            window_system.display,
                                            cc.window_id,
                                            cc.w as i32 * -1,
                                            cc.h as i32 * -1,
                                            cc.w,
                                            cc.h,
                                        );
                                        cc.visible = !cc.visible;
                                        shift_current_client(window_system);
                                        window_system.screens[window_system.current_screen]
                                            .workspaces[*n as usize]
                                            .clients
                                            .push(cc);
                                    }
                                }
                            }
                            ActionResult::FocusOnWorkspace(n) => {
                                log!("   |- Got `FocusOnWorkspace` Action");
                                if *n as usize != window_system.current_workspace {
                                    show_hide_workspace(window_system);
                                    window_system.current_workspace = *n as usize;
                                    window_system.screens[window_system.current_screen]
                                        .current_workspace = *n as usize;
                                    window_system.current_client = window_system.screens
                                        [window_system.current_screen]
                                        .workspaces[window_system.current_workspace]
                                        .current_client;
                                    show_hide_workspace(window_system);
                                    arrange(window_system);
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
                                log!("{:#?}", &window_system);
                            }
                        }
                    }
                }
            }

            x11::xlib::MapRequest => {
                let ew: u64 = ev.map_request.unwrap().window;
                log!("|- Map Request From Window: {ew}");
                if let Some(wa) = get_window_attributes(window_system.display, ew) {
                    if wa.override_redirect == 0 {
                        log!("   |- Window can be mapped");
                        manage_client(window_system, ew);
                    }
                }
            }

            x11::xlib::EnterNotify => {
                let ew: u64 = ev.crossing.unwrap().window;
                log!("|- Crossed Window `{}`", get_client_name(window_system, ew).unwrap_or("Unmanaged window".to_string()));
                log!("   |- Setting focus to window");
                set_input_focus(window_system.display, ew, RevertToNone, CurrentTime);
                update_current_client(window_system, ew);
                if let Some((s, w, c)) = find_window_indexes(window_system, ew) {
                    window_system.current_screen = s;
                    window_system.current_workspace = w;
                    window_system.current_client = Some(c);
                };
            }

            x11::xlib::DestroyNotify => {
                let ew: u64 = ev.destroy_window.unwrap().window;
                log!("|- `{}` destroyed", get_client_name(window_system, ew).unwrap_or("Unmanaged window".to_string()));
                unmanage_window(window_system, ew);
            }

            x11::xlib::UnmapNotify => {
                let ew: u64 = ev.unmap.unwrap().window;
                log!("|- `{}` unmapped", get_client_name(window_system, ew).unwrap_or("Unmanaged window".to_string()));
                unmanage_window(window_system, ew);
            }
            x11::xlib::MotionNotify => {
                log!("|- `Motion` detected");
                let p = ev.motion.unwrap();
                let (x, y) = (p.x as i64, p.y as i64);
                for screen in &window_system.screens {
                    if screen.x <= x
                        && x < screen.x + screen.width
                        && screen.y <= y
                        && y < screen.y + screen.height
                    {
                        window_system.current_screen = screen.number as usize;
                    }
                }
            }
            x11::xlib::PropertyNotify => {
                let p = ev.property.unwrap();
                if p.window != window_system.root_win {
                    log!("|- `Property` changed for window {} `{}`", p.window, get_client_name(window_system, p.window).unwrap_or("Unmanaged window".to_string()));
                    update_client_name(window_system, p.window);
                }
            }
            _ => {}
        };
    }
}

fn cleanup(_app: &mut ApplicationContainer) {}

fn main() {
    // Set locale for proper work with unicde symnbols
    set_locale(LC_CTYPE, ""); 

    // Init `app` container
    let mut app: ApplicationContainer = setup();

    // Scan for existing windows
    scan(&app.environment.config, &mut app.environment.window_system);

    // start main loop
    run(&app.environment.config, &mut app.environment.window_system);

    // close all connections, dump data, exit
    cleanup(&mut app);
}
