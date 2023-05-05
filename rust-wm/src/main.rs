//! A window manager written in Rust with nearly same functionality of [dwm](https://dwm.suckless.org/)
//!
//! List of features supported by rwm:
//! - Multi monitor setup
//! - Workspaces aka tags
//! - Stack layout
//! - Shortcuts

mod get_default;
mod grab;
use std::process::Command;
use std::ptr::null_mut;
use std::vec;

use grab::grab_key;

mod structs;
mod wrap;

/// Converts ARGS value to single int
fn _argb_to_int(a: u32, r: u8, g: u8, b: u8) -> u64 {
    (a as u64) << 24 | (r as u64) << 16 | (g as u64) << 8 | (b as u64)
}

use libc::LC_CTYPE;
use wrap::xlib::get_text_property;
use wrap::xlib::get_wm_normal_hints;
use wrap::xlib::get_wm_protocols;
use wrap::xlib::intern_atom;
use wrap::xlib::send_event;
use wrap::xlib::set_locale;
use wrap::xlib::Event;
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
use x11::xlib::PMaxSize;
use x11::xlib::PMinSize;
use x11::xlib::PointerMotionMask;
use x11::xlib::PropertyChangeMask;
use x11::xlib::RevertToNone;
use x11::xlib::RevertToPointerRoot;
use x11::xlib::ShiftMask;
use x11::xlib::StructureNotifyMask;
use x11::xlib::SubstructureNotifyMask;
use x11::xlib::SubstructureRedirectMask;
use x11::xlib::XSetWindowAttributes;

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
use crate::wrap::xlib::set_error_handler;
use crate::wrap::xlib::set_input_focus;

use crate::structs::*;

/// Does println! in debug, does nothing in release
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

/// Initially sets ApplicationContainer variables
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
                keysym: XK_q,
                result: ActionResult::Quit,
            },
            KeyAction {
                modifier: ModKey | ShiftMask,
                keysym: XK_c,
                result: ActionResult::KillClient,
            },
            KeyAction {
                modifier: ModKey,
                keysym: XK_w,
                result: ActionResult::DumpInfo,
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
            KeyAction {
                modifier: ModKey,
                keysym: XK_i,
                result: ActionResult::UpdateMasterCapacity(1),
            },
            KeyAction {
                modifier: ModKey,
                keysym: XK_d,
                result: ActionResult::UpdateMasterCapacity(-1),
            },
            KeyAction {
                modifier: ModKey,
                keysym: XK_l,
                result: ActionResult::UpdateMasterWidth(0.05),
            },
            KeyAction {
                modifier: ModKey,
                keysym: XK_h,
                result: ActionResult::UpdateMasterWidth(-0.05),
            },
            KeyAction {
                modifier: ModKey | ShiftMask,
                keysym: XK_space,
                result: ActionResult::ToggleFloat,
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
                        master_capacity: 1,
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

    set_error_handler();

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

    let net_supported = intern_atom(
        app.environment.window_system.display,
        "_NET_SUPPORTED".to_string(),
        false,
    );
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

/// Fetches clients that are already present
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

/// Gets name from x server for specified window and undates it in struct
fn update_client_name(window_system: &mut WindowSystemContainer, win: u64) {
    // Create atom for property request
    let name_atom = intern_atom(window_system.display, "_NET_WM_NAME".to_string(), false);
    // Get name property and dispatch Option<>
    let name = match get_text_property(window_system.display, win, name_atom) {
        Some(name) => name,
        None => "WHAT THE FUCK".to_string(),
    };

    // Get trackers for specified window and change name
    if let Some((s, w, c)) = find_window_indexes(window_system, win) {
        window_system.screens[s].workspaces[w].clients[c].window_name = name;
    }
}
/// Returns name of specified client
fn get_client_name(window_system: &mut WindowSystemContainer, win: u64) -> Option<String> {
    if let Some((s, w, c)) = find_window_indexes(window_system, win) {
        Some(
            window_system.screens[s].workspaces[w].clients[c]
                .window_name
                .clone(),
        )
    } else {
        None
    }
}

/// Adds client to window_system and configures it if needed
fn manage_client(ws: &mut WindowSystemContainer, win: u64) {
    // Check if window can be managed
    let wa = match get_window_attributes(ws.display, win) {
        Some(a) => {
            if a.override_redirect != 0 {
                return;
            }
            a
        },
        None => {
            return;
        }
    };

    // Create client
    let mut c: Client = Client::default();
    let mut trans = 0;

    // Set essential client fields
    c.window_id = win;
    c.w = wa.width as u32;
    c.h = wa.height as u32;
    c.x = wa.x;
    c.y = wa.y;
    c.visible = true;

    // Check if window is transient
    if get_transient_for_hint(ws.display, win, &mut trans) != 1 {
        log!("   |- Transient");
    } else {
        log!("   |- Not transient");
    }

    // Check if dialog or fullscreen 

    // Get window default size hints
    log!("   |- Getting default sizes");
    if let Some((sh, _)) = get_wm_normal_hints(ws.display, win) {
        if (sh.flags & PMaxSize) != 0 {
            c.maxw = sh.max_width;
            c.maxh = sh.max_height;
            log!("      |- Setting max sizes to `{}, {}` `(width, height)`", sh.max_width, sh.max_height);
        } else {
            c.maxw = 0;
            c.maxh = 0;
            log!("      |- Error getting max sizes. Falling to default `(0, 0)`");
        }
        if (sh.flags & PMinSize) != 0 {
            c.minw = sh.min_width;
            c.minh = sh.min_height;
            log!("      |- Setting min sizes to `{}, {}` `(width, height)`", sh.min_width, sh.min_height);
        } else {
            c.minw = 0;
            c.minh = 0;
            log!("      |- Error getting min sizes. Falling to default `(0, 0)`");
        }
    } else {
        log!("   |- Cant fetch normal hints, setting everything to 0");
        c.maxw = 0;
        c.maxh = 0;
        c.minw = 0;
        c.minh = 0;
    }

    // Set fixed if max = min and if not zero(no size restrictions)
    if c.maxw != 0 && c.maxh != 0 && c.maxw == c.minw && c.maxh == c.minh {
        c.fixed = true;
    }

    // If fixed or transient => always float;
    if !c.floating {
        c.floating = c.fixed || trans != 0;
    }

    // Set input mask
    select_input(
        ws.display,
        win,
        EnterWindowMask | FocusChangeMask | PropertyChangeMask | StructureNotifyMask,
    );

    // Push new client
    // (Client {
    //    window_name: "Unmanaged window".to_string(),
    //    visible: true,
    //    floating: trans != 0 || false,
    //    fixed: false,
    // });
    // Add client to stack
    // Get current workspace
    let w = &mut ws.screens[ws.current_screen].workspaces[ws.current_workspace];
    // Update client tracker
    w.current_client = Some(w.clients.len());
    ws.current_client = w.current_client;
    // Push to stack
    w.clients.push(c);
    // Fetch and set client name
    update_client_name(ws, win);
    // Raise window above other`
    raise_window(ws.display, win);
    // Arrange current workspace
    arrange(ws);
    // Finish mapping
    map_window(ws.display, win);
}

/// Arranges windows of current workspace in specified layout
fn arrange(ws: &mut WindowSystemContainer) {
    log!("   |- Arranging...");
    // Go thru all screens
    for screen in &mut ws.screens {
        // Get amount of visible not floating clients to arrange
        let stack_size = screen.workspaces[screen.current_workspace].clients.iter().filter(|&c| !c.floating).count();
        // Get capacity and width of master area
        let mut master_width =
            (screen.width as f64 * screen.workspaces[screen.current_workspace].master_width) as u32;
        let mut master_capacity = screen.workspaces[screen.current_workspace].master_capacity;
        // if master_capacity out of stack_size bounds use whole screen for one column
        if master_capacity <= 0 || master_capacity >= stack_size as i64 {
            master_capacity = stack_size as i64;
            master_width = screen.width as u32;
        }
        log!("   |- Arranging {} window", stack_size);
        // Iterate over all clients structs
        for (index, client) in screen.workspaces[screen.current_workspace]
            .clients
            .iter_mut()
            .rev()
            .filter(|c| !c.floating)
            .enumerate()
        {
            if stack_size == 1 {
                // if only one window selected, just show it
                client.x = 0;
                client.y = 0;
                client.w = screen.width as u32;
                client.h = screen.height as u32;
            } else {
                if (index as i64) < master_capacity {
                    // if master_capacity is not full put it here
                    log!("      |- Goes to master");
                    // some math...
                    client.x = 0;
                    client.y =
                        ((index as f64) * (screen.height as f64) / (master_capacity as f64)) as i32;
                    client.w = master_width;
                    client.h = ((screen.height as f64) / (master_capacity as f64)) as u32;
                } else {
                    // otherwise put it in secondary stack
                    log!("      |- Goes to stack");
                    // a bit more of math...
                    client.x = master_width as i32;
                    client.y = ((index as i64 - master_capacity) as f64 * (screen.height as f64)
                        / (stack_size as i64 - master_capacity) as f64)
                        as i32;
                    client.w = screen.width as u32 - master_width;
                    client.h = ((screen.height as f64)
                        / (stack_size as i64 - master_capacity) as f64)
                        as u32;
                }
            }
        }
        // update corresponding windows
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

/// Returns window, workspace and client indexies for client with specified id
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

/// Shows/Hides all windows on current workspace
fn show_hide_workspace(ws: &mut WindowSystemContainer) {
    // Iterate over all clients
    for client in &mut ws.screens[ws.current_screen].workspaces[ws.current_workspace].clients {
        if client.visible {
            // move window to left top
            move_resize_window(
                ws.display,
                client.window_id,
                client.w as i32 * -1,
                client.h as i32 * -1,
                client.w,
                client.h,
            );
        } else {
            // move to normal position
            move_resize_window(
                ws.display,
                client.window_id,
                client.w as i32 * -1,
                client.h as i32 * -1,
                client.w,
                client.h,
            );
        }
        // flip visibility state
        client.visible = !client.visible;
    }
}

/// Shifts current client tracker after destroying clients
fn shift_current_client(ws: &mut WindowSystemContainer) {
    // Find next client
    ws.screens[ws.current_screen].workspaces[ws.current_workspace].current_client = {
        // Get reference to windows stack
        let clients = &ws.screens[ws.current_screen].workspaces[ws.current_workspace].clients;
        if clients.len() == 0 {
            // None if no windows
            None
        } else {
            // Get old client index
            let cc = ws.screens[ws.current_screen].workspaces[ws.current_workspace]
                .current_client
                .expect("WHAT THE FUCK");
            // If selected client was not last do nothing
            if cc < clients.len() {
                Some(cc)
            } else {
                // Else set it to being last
                Some(clients.len() - 1)
            }
        }
    };
    // update secondary tracker
    ws.current_client =
        ws.screens[ws.current_screen].workspaces[ws.current_workspace].current_client;
}

/// Safely sends atom to X server
fn send_atom(ws: &mut WindowSystemContainer, win: u64, e: x11::xlib::Atom) -> bool {
    if let Some(ps) = get_wm_protocols(ws.display, win) {
        // Get supported protocols
        for p in ps {
            if p == e {
                // if requested protocol found, then proceed
                // create event with requested message
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
                // send it to requested window
                return send_event(ws.display, win, false, NoEventMask, &mut ev);
            }
        }
        return false;
    } else {
        return false;
    }
}

/// Removes window from window_system
fn unmanage_window(ws: &mut WindowSystemContainer, win: u64) {
    // Find trackers for window
    if let Some((s, w, c)) = find_window_indexes(ws, win) {
        log!("   |- Found window {} at indexes {}, {}, {}", win, s, w, c);
        // Removed corresponding client from stack
        let clients = &mut ws.screens[s].workspaces[w].clients;
        clients.remove(c);
        // Update client tracker
        shift_current_client(ws);
        // Rearrange
        arrange(ws);
    } else {
        log!("   |- Window is not managed");
    }
}

/// Starts main WM loop
fn run(config: &ConfigurationContainer, ws: &mut WindowSystemContainer) {
    log!("|===== run =====");
    while ws.running {
        // Process Events
        let ev = next_event(ws.display);

        // Do pattern matching for known events
        match ev.type_ {
            x11::xlib::KeyPress => {
                // Safely retrive struct
                let ev = ev.key.unwrap();
                // Iterate over key actions matching current key input
                for action in &config.key_actions {
                    if ev.keycode == keysym_to_keycode(ws.display, action.keysym)
                        && ev.state == action.modifier
                    {
                        // Log action type
                        log!("   |- Got {:?} action", action.result);
                        // Match action result and run related function
                        match &action.result {
                            ActionResult::KillClient => {
                                log!("   |- Got `KillClient` Action");
                                // Check if there any windows selected
                                match ws.current_client {
                                    Some(index) => {
                                        let id = ws.screens[ws.current_screen].workspaces
                                            [ws.current_workspace]
                                            .clients[index]
                                            .window_id;
                                        log!("      |- Killing window {}", id);
                                        // Politely ask window to fuck off... I MEAN CLOSE ITSELF
                                        let a = intern_atom(
                                            ws.display,
                                            "WM_DELETE_WINDOW".to_string(),
                                            false,
                                        );
                                        if !send_atom(ws, id, a) {
                                        };
                                    }
                                    None => {
                                        log!("      |- No window selected");
                                    }
                                };
                            }
                            ActionResult::Spawn(cmd) => {
                                println!("   |- Got `Spawn` Action");
                                // Run sh with specified command
                                let mut handle = Command::new("/usr/bin/sh")
                                    .arg("-c")
                                    .arg(cmd)
                                    .spawn()
                                    .expect(format!("can't execute {cmd}").as_str());
                                // Run it in sepated thread
                                // TODO: WHERE ARE HANDLERS STORED
                                std::thread::spawn(move || {
                                    handle.wait().expect("can't run process");
                                });
                            }
                            ActionResult::MoveToScreen(d) => {
                                // Check if window is selected
                                if let Some(index) = ws.current_client {
                                    // Get current screen index
                                    let mut cs = ws.current_screen;
                                    // Update index depending on supplied direction
                                    cs = match d {
                                        ScreenSwitching::Next => (cs + 1) % ws.screens.len(),
                                        ScreenSwitching::Previous => {
                                            (cs + ws.screens.len() - 1) % ws.screens.len()
                                        }
                                    };
                                    // Pop client
                                    let cc = ws.screens[ws.current_screen].workspaces
                                        [ws.current_workspace]
                                        .clients
                                        .remove(index);
                                    // Update client tracker on current screen
                                    shift_current_client(ws);
                                    // Get workspace tracker(borrow checker is really mad at me)
                                    let nw = ws.screens[cs].current_workspace;
                                    // Add window to stack of another display
                                    ws.screens[cs].workspaces[nw].clients.push(cc);
                                    // Arrange all monitors
                                    arrange(ws);
                                }
                            }
                            ActionResult::FocusOnScreen(d) => {
                                // Get current screen
                                let mut cs = ws.current_screen;
                                // Update it
                                cs = match d {
                                    ScreenSwitching::Next => (cs + 1) % ws.screens.len(),
                                    ScreenSwitching::Previous => {
                                        (cs + ws.screens.len() - 1) % ws.screens.len()
                                    }
                                };
                                // Change trackers
                                ws.current_screen = cs;
                                ws.current_workspace =
                                    ws.screens[ws.current_screen].current_workspace;
                                ws.current_client = ws.screens[ws.current_screen].workspaces
                                    [ws.current_workspace]
                                    .current_client;
                            }
                            ActionResult::MoveToWorkspace(n) => {
                                log!("   |- Got `MoveToWorkspace` Action ");
                                // Check if moving to another workspace
                                if *n as usize != ws.current_workspace {
                                    // Check if any client is selected
                                    if let Some(index) = ws.current_client {
                                        // Pop current client
                                        let mut cc = ws.screens[ws.current_screen].workspaces
                                            [ws.current_workspace]
                                            .clients
                                            .remove(index);
                                        // Update current workspace layout
                                        arrange(ws);
                                        // Move window out of view
                                        move_resize_window(
                                            ws.display,
                                            cc.window_id,
                                            cc.w as i32 * -1,
                                            cc.h as i32 * -1,
                                            cc.w,
                                            cc.h,
                                        );
                                        cc.visible = !cc.visible;
                                        // Update tracker
                                        shift_current_client(ws);
                                        // Add client to choosen workspace (will be arranged later)
                                        ws.screens[ws.current_screen].workspaces[*n as usize]
                                            .clients
                                            .push(cc);
                                    }
                                }
                            }
                            ActionResult::FocusOnWorkspace(n) => {
                                log!("   |- Got `FocusOnWorkspace` Action");
                                // Check is focusing on another workspace
                                if *n as usize != ws.current_workspace {
                                    // Hide current workspace
                                    show_hide_workspace(ws);
                                    // Update workspace index
                                    ws.current_workspace = *n as usize;
                                    ws.screens[ws.current_screen].current_workspace = *n as usize;
                                    // Update current client
                                    ws.current_client = ws.screens[ws.current_screen].workspaces
                                        [ws.current_workspace]
                                        .current_client;
                                    // Show current client
                                    show_hide_workspace(ws);
                                    // Arrange update workspace
                                    arrange(ws);
                                }
                            }
                            ActionResult::MaximazeWindow => {
                                log!("   |- Action `MaximazeWindow` is not currently supported");
                            }
                            ActionResult::Quit => {
                                log!("   |- Got `Quit` Action. `Quiting`");
                                ws.running = false;
                            }
                            ActionResult::UpdateMasterCapacity(i) => {
                                // Change master size
                                ws.screens[ws.current_screen].workspaces[ws.current_workspace]
                                    .master_capacity += *i;
                                // Rearrange windows
                                arrange(ws);
                            }
                            ActionResult::UpdateMasterWidth(w) => {
                                // Update master width
                                ws.screens[ws.current_screen].workspaces[ws.current_workspace]
                                    .master_width += *w;
                                // Rearrange windows
                                arrange(ws);
                            }
                            ActionResult::DumpInfo => {
                                // Dump all info to log
                                log!("{:#?}", &ws);
                            }
                            ActionResult::ToggleFloat => {
                                if let Some(c) = ws.current_client {
                                    let state = ws.screens[ws.current_screen].workspaces[ws.current_workspace].clients[c].floating;
                                    ws.screens[ws.current_screen].workspaces[ws.current_workspace].clients[c].floating = !state;
                                    arrange(ws);
                                }
                            }
                        }
                    }
                }
            }

            // Used for adding windows to WM
            x11::xlib::MapRequest => {
                let ew: u64 = ev.map_request.unwrap().window;
                log!("|- Map Request From Window: {ew}");
                manage_client(ws, ew);
            }

            // Checks if window got crossed, used for focus and monitor tracking
            x11::xlib::EnterNotify => {
                let ew: u64 = ev.crossing.unwrap().window;
                log!(
                    "|- Crossed Window `{}`",
                    get_client_name(ws, ew).unwrap_or("Unmanaged window".to_string())
                );
                log!("   |- Setting focus to window");
                // Focus on crossed window
                set_input_focus(ws.display, ew, RevertToPointerRoot, CurrentTime);
                // if window managed get trackers
                if let Some((s, w, c)) = find_window_indexes(ws, ew) {
                    ws.current_screen = s;
                    ws.current_workspace = w;
                    ws.screens[s].current_workspace = w;
                    ws.current_client = Some(c);
                    ws.screens[s].workspaces[w].current_client = Some(c);
                };
            }

            // Used to unmanage_window
            x11::xlib::DestroyNotify => {
                let ew: u64 = ev.destroy_window.unwrap().window;
                log!(
                    "|- `{}` destroyed",
                    get_client_name(ws, ew).unwrap_or("Unmanaged window".to_string())
                );
                unmanage_window(ws, ew);
            }

            // Used to unmanage_window
            x11::xlib::UnmapNotify => {
                let ew: u64 = ev.unmap.unwrap().window;
                log!(
                    "|- `{}` unmapped",
                    get_client_name(ws, ew).unwrap_or("Unmanaged window".to_string())
                );
                unmanage_window(ws, ew);
            }

            // Used for tracking selected monitor if no windows present
            x11::xlib::MotionNotify => {
                // Log some info
                log!("|- `Motion` detected");
                // Safely retrive event struct
                let p = ev.motion.unwrap();
                // Get mouse positions
                let (x, y) = (p.x as i64, p.y as i64);
                // Iterate over all screens
                for screen in &ws.screens {
                    // Check if mouse position "inside" screens
                    if screen.x <= x
                        && x < screen.x + screen.width
                        && screen.y <= y
                        && y < screen.y + screen.height
                    {
                        // Update trackers
                        ws.current_screen = screen.number as usize;
                        ws.current_workspace = ws.screens[ws.current_screen].current_workspace;
                        ws.current_client = ws.screens[ws.current_screen].workspaces
                            [ws.current_workspace]
                            .current_client;
                    }
                }
            }

            // Used for updating names of applications
            x11::xlib::PropertyNotify => {
                // Safely retrive event struct
                let p = ev.property.unwrap();

                // If current window is not root proceed to updating name
                if p.window != ws.root_win {
                    log!(
                        "|- `Property` changed for window {} `{}`",
                        p.window,
                        get_client_name(ws, p.window).unwrap_or("Unmanaged window".to_string())
                    );
                    update_client_name(ws, p.window);
                }
            }
            _ => {}
        };
    }
}

/// Closes all connections and saves info
fn cleanup(_app: &mut ApplicationContainer) {}

/// Rust default entry function
fn main() {
    // Set locale for proper work with unicode symbols
    // For example: getting names of windows
    set_locale(LC_CTYPE, "");

    // Init `app` container
    // App container consists of all data needed for WM to function
    let mut app: ApplicationContainer = setup();

    // Scan for existing windows
    scan(&app.environment.config, &mut app.environment.window_system);

    // Start main loop
    // Event processing, and all stuff occur here
    run(&app.environment.config, &mut app.environment.window_system);

    // Gracefully exit
    // Close all connections, dump data, exit
    cleanup(&mut app);
}
