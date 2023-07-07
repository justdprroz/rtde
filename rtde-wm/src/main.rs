//! A window manager written in Rust with nearly same functionality of [dwm](https://dwm.suckless.org/)
//!
//! List of features supported by rwm:
//! - Multi monitor setup
//! - Workspaces aka tags
//! - Stack layout
//! - Shortcuts

mod get_default;
mod grab;
use std::mem::size_of;
use std::ptr::null_mut;
use std::vec;

use grab::grab_key;

mod structs;
mod wrap;

/// Converts ARGS value to single int
fn argb_to_int(a: u32, r: u8, g: u8, b: u8) -> u64 {
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
use x11::xlib::Atom;
use x11::xlib::ButtonPressMask;
use x11::xlib::CWBorderWidth;
use x11::xlib::CWCursor;
use x11::xlib::CWEventMask;
use x11::xlib::ClientMessage;
use x11::xlib::CurrentTime;
use x11::xlib::DestroyAll;
use x11::xlib::EnterWindowMask;
use x11::xlib::FocusChangeMask;
use x11::xlib::IsViewable;
use x11::xlib::LeaveWindowMask;
use x11::xlib::Mod1Mask as ModKey;
use x11::xlib::NoEventMask;
use x11::xlib::PMaxSize;
use x11::xlib::PMinSize;
use x11::xlib::PointerMotionMask;
use x11::xlib::PropModeAppend;
use x11::xlib::PropModeReplace;
use x11::xlib::PropertyChangeMask;
use x11::xlib::RevertToPointerRoot;
use x11::xlib::ShiftMask;
use x11::xlib::StructureNotifyMask;
use x11::xlib::SubstructureNotifyMask;
use x11::xlib::SubstructureRedirectMask;
use x11::xlib::Success;
use x11::xlib::XSetWindowAttributes;
use x11::xlib::XA_ATOM;
use x11::xlib::XA_WINDOW;

use crate::wrap::xinerama::xinerama_query_screens;
use crate::wrap::xlib::change_property;
use crate::wrap::xlib::change_window_attributes;
use crate::wrap::xlib::configure_window;
use crate::wrap::xlib::create_simple_window;
use crate::wrap::xlib::default_root_window;
use crate::wrap::xlib::delete_property;
use crate::wrap::xlib::get_transient_for_hint;
use crate::wrap::xlib::get_window_attributes;
use crate::wrap::xlib::grab_server;
use crate::wrap::xlib::keysym_to_keycode;
use crate::wrap::xlib::kill_client;
use crate::wrap::xlib::map_window;
use crate::wrap::xlib::move_resize_window;
use crate::wrap::xlib::next_event;
use crate::wrap::xlib::open_display;
use crate::wrap::xlib::query_tree;
use crate::wrap::xlib::raise_window;
use crate::wrap::xlib::select_input;
use crate::wrap::xlib::set_close_down_mode;
use crate::wrap::xlib::set_error_handler;
use crate::wrap::xlib::set_input_focus;

use crate::structs::*;
use crate::wrap::xlib::set_window_border;
use crate::wrap::xlib::set_window_border_width;
use crate::wrap::xlib::ungrab_server;

/// Does println! in debug, does nothing in release
macro_rules! log {
    ($($e:expr),+) => {
        #[cfg(debug_assertions)]
        println!($($e),+);
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
                visual_preferences: VisualPreferences {
                    gap_width: 4,
                    border_size: 2,
                    normal_border_color: argb_to_int(255, 64, 64, 64),
                    active_border_color: argb_to_int(255, 64, 192, 64),
                },
                key_actions: Vec::new(),
                layout_rules: Vec::new(),
                status_bar_builder: Vec::new(),
            },
            window_system: WindowSystemContainer {
                status_bar: StatusBarContainer {},
                display,
                root_win: 0,
                wm_check_win: 0,
                running: true,
                screens: Vec::new(),
                current_screen: 0,
                current_workspace: 0,
                current_client: None,

                atoms: Atoms {
                    wm_protocols: 0,
                    wm_delete: 0,
                    wm_state: 0,
                    net_wm_check: 0,
                    wm_take_focus: 0,
                    net_active_window: 0,
                    net_supported: 0,
                    net_wm_name: 0,
                    net_wm_state: 0,
                    net_wm_fullscreen: 0,
                    net_wm_window_type: 0,
                    net_wm_window_type_dialog: 0,
                    net_client_list: 0,
                },
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
                keysym: XK_e,
                result: ActionResult::Spawn("thunar".to_string()),
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
            },
            KeyAction {
                modifier: ModKey,
                keysym: XK_j,
                result: ActionResult::CycleStack(-1),
            },
            KeyAction {
                modifier: ModKey,
                keysym: XK_k,
                result: ActionResult::CycleStack(1),
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
        default_root_window(app.environment.window_system.display);

    let dpy = &mut app.environment.window_system.display;

    app.environment.window_system.atoms = Atoms {
        wm_protocols: intern_atom(dpy, "WM_PROTOCOLS".to_string(), false),
        wm_delete: intern_atom(dpy, "WM_DELETE_WINDOW".to_string(), false),
        wm_state: intern_atom(dpy, "WM_STATE".to_string(), false),
        wm_take_focus: intern_atom(dpy, "WM_TAKE_FOCUS".to_string(), false),

        net_active_window: intern_atom(dpy, "_NET_ACTIVE_WINDOW".to_string(), false),
        net_supported: intern_atom(dpy, "_NET_SUPPORTED".to_string(), false),
        net_wm_name: intern_atom(dpy, "_NET_WM_NAME".to_string(), false),
        net_wm_state: intern_atom(dpy, "_NET_WM_STATE".to_string(), false),
        net_wm_check: intern_atom(dpy, "_NET_SUPPORTING_WM_CHECK".to_string(), false),
        net_wm_fullscreen: intern_atom(dpy, "_NET_WM_STATE_FULLSCREEN".to_string(), false),
        net_wm_window_type: intern_atom(dpy, "_NET_WM_WINDOW_TYPE".to_string(), false),
        net_wm_window_type_dialog: intern_atom(
            dpy,
            "_NET_WM_WINDOW_TYPE_DIALOG".to_string(),
            false,
        ),
        net_client_list: intern_atom(dpy, "_NET_CLIENT_LIST".to_string(), false),
    };

    log!("|- Initialized `Variables`");

    // Init supporting window
    app.environment.window_system.wm_check_win = create_simple_window(
        dpy,
        app.environment.window_system.root_win,
        0,
        0,
        1,
        1,
        0,
        0,
        0,
    );
    let mut wmchckwin = app.environment.window_system.wm_check_win;

    let utf8string = intern_atom(dpy, "UTF8_STRING".to_string(), false);

    change_property(
        dpy,
        wmchckwin,
        app.environment.window_system.atoms.net_wm_check,
        XA_WINDOW,
        32,
        PropModeReplace,
        &mut wmchckwin as *mut u64 as *mut u8,
        1,
    );
    let wm_name = std::ffi::CString::new("rtwm".to_string()).unwrap();
    change_property(
        dpy,
        wmchckwin,
        app.environment.window_system.atoms.net_wm_name,
        utf8string,
        8,
        PropModeReplace,
        wm_name.as_ptr() as *mut u8,
        7,
    );
    change_property(
        dpy,
        app.environment.window_system.root_win,
        app.environment.window_system.atoms.net_wm_check,
        XA_WINDOW,
        32,
        PropModeReplace,
        &mut wmchckwin as *mut u64 as *mut u8,
        1,
    );

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
    log!("{:#?}", &app.environment.window_system.screens);
    log!("|- Initialized xinerama `Screens` and nested `Workspaces`");
    // TODO: Init Api

    // Setup WM with X server info

    set_error_handler();

    let ws = &mut app.environment.window_system;
    let mut netatoms = vec![
        ws.atoms.net_active_window,
        ws.atoms.net_supported,
        ws.atoms.net_wm_name,
        ws.atoms.net_wm_check,
        ws.atoms.net_wm_fullscreen,
        ws.atoms.net_wm_window_type,
        ws.atoms.net_wm_window_type_dialog,
        ws.atoms.net_client_list,
        ws.atoms.net_wm_state,
    ];

    change_property(
        app.environment.window_system.display,
        app.environment.window_system.root_win,
        app.environment.window_system.atoms.net_supported,
        x11::xlib::XA_ATOM,
        32,
        x11::xlib::PropModeReplace,
        netatoms.as_mut_ptr() as *mut u8,
        netatoms.len() as i32,
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
fn scan(app: &mut ApplicationContainer) {
    // let window_system = &mut app.environment.window_system;
    log!("|===== scan =====");
    let (mut rw, _, wins) = query_tree(
        app.environment.window_system.display,
        app.environment.window_system.root_win,
    );

    log!("|- Found {} window(s) that are already present", wins.len());

    for win in wins {
        log!("   |- Checking window {win}");
        let res = get_window_attributes(app.environment.window_system.display, win);
        if let Some(wa) = res {
            if wa.override_redirect != 0
                || get_transient_for_hint(app.environment.window_system.display, win, &mut rw) != 0
            {
                log!("      |- Window is transient. Skipping");
                continue;
            }
            if wa.map_state == IsViewable {
                log!("      |- Window is viewable. Managing");
                manage_client(app, win);
                continue;
            }
        }
        log!("      |- Can't manage window");
    }
}

/// Gets name from x server for specified window and undates it in struct
fn update_client_name(app: &mut ApplicationContainer, win: u64) {
    // Get name property and dispatch Option<>
    let name = match get_text_property(
        app.environment.window_system.display,
        win,
        app.environment.window_system.atoms.net_wm_name,
    ) {
        Some(name) => name,
        None => "WHAT THE FUCK".to_string(),
    };

    // Get trackers for specified window and change name
    if let Some((s, w, c)) = find_window_indexes(app, win) {
        app.environment.window_system.screens[s].workspaces[w].clients[c].window_name = name;
    }
}
/// Returns name of specified client
fn get_client_name(app: &mut ApplicationContainer, win: u64) -> Option<String> {
    if let Some((s, w, c)) = find_window_indexes(app, win) {
        Some(
            app.environment.window_system.screens[s].workspaces[w].clients[c]
                .window_name
                .clone(),
        )
    } else {
        None
    }
}

/// Adds client to window_system and configures it if needed
fn manage_client(app: &mut ApplicationContainer, win: u64) {
    // Check if window can be managed
    let wa = match get_window_attributes(app.environment.window_system.display, win) {
        Some(a) => {
            if a.override_redirect != 0 {
                return;
            }
            a
        }
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
    if get_transient_for_hint(app.environment.window_system.display, win, &mut trans) != 1 {
        log!("   |- Transient");
    } else {
        log!("   |- Not transient");
    }

    // Check if dialog or fullscreen

    let state = get_atom_prop(app, win, app.environment.window_system.atoms.net_wm_state);
    let wtype = get_atom_prop(
        app,
        win,
        app.environment.window_system.atoms.net_wm_window_type,
    );

    if state == app.environment.window_system.atoms.net_wm_fullscreen {
        c.floating = true;
        c.fullscreen = true;
    }
    if wtype
        == app
            .environment
            .window_system
            .atoms
            .net_wm_window_type_dialog
    {
        c.floating = true;
    }

    // Get window default size hints
    log!("   |- Getting default sizes");
    if let Some((sh, _)) = get_wm_normal_hints(app.environment.window_system.display, win) {
        if (sh.flags & PMaxSize) != 0 {
            c.maxw = sh.max_width;
            c.maxh = sh.max_height;
            log!(
                "      |- Setting max sizes to `{}, {}` `(width, height)`",
                sh.max_width,
                sh.max_height
            );
        } else {
            c.maxw = 0;
            c.maxh = 0;
            log!("      |- Error getting max sizes. Falling to default `(0, 0)`");
        }
        if (sh.flags & PMinSize) != 0 {
            c.minw = sh.min_width;
            c.minh = sh.min_height;
            log!(
                "      |- Setting min sizes to `{}, {}` `(width, height)`",
                sh.min_width,
                sh.min_height
            );
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

    if c.minw != 0 && c.w < c.minw as u32 {
        c.w = c.minw as u32;
    }
    if c.minh != 0 && c.h < c.minh as u32 {
        c.h = c.minh as u32;
    }

    // Set fixed if max = min and if not zero(no size restrictions)
    if c.maxw != 0 && c.maxh != 0 && c.maxw == c.minw && c.maxh == c.minh {
        c.fixed = true;
    }

    // If fixed or transient => always float;
    if !c.floating {
        c.floating = c.fixed || trans != 0;
    } else {
        log!("   |- Floating");
    }

    // floating windows are moved to centre of screen?

    // Set input mask
    select_input(
        app.environment.window_system.display,
        win,
        EnterWindowMask | FocusChangeMask | PropertyChangeMask | StructureNotifyMask,
    );
    // Set previous client border to normal
    if let Some(cw) = get_current_client_id(app) {
        set_window_border(
            app.environment.window_system.display,
            cw,
            app.environment
                .config
                .visual_preferences
                .normal_border_color,
        );
    }
    // Get current workspace
    let w = &mut app.environment.window_system.screens
        [app.environment.window_system.current_screen]
        .workspaces[app.environment.window_system.current_workspace];
    // Update client tracker
    w.current_client = Some(w.clients.len());
    app.environment.window_system.current_client = w.current_client;
    // Push to stack
    w.clients.push(c);
    // Add window to wm _NET_CLIENT_LIST
    change_property(
        app.environment.window_system.display,
        app.environment.window_system.root_win,
        app.environment.window_system.atoms.net_client_list,
        XA_WINDOW,
        32,
        PropModeAppend,
        &win as *const u64 as *mut u8,
        1,
    );
    // set border size
    let mut wc = x11::xlib::XWindowChanges {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
        border_width: app.environment.config.visual_preferences.border_size as i32,
        sibling: 0,
        stack_mode: 0,
    };
    configure_window(
        app.environment.window_system.display,
        win,
        CWBorderWidth as u32,
        &mut wc,
    );
    set_window_border(
        app.environment.window_system.display,
        win,
        app.environment
            .config
            .visual_preferences
            .active_border_color,
    );
    // Fetch and set client name
    update_client_name(app, win);
    // Raise window above other`
    raise_window(app.environment.window_system.display, win);
    // Focus on created window
    set_input_focus(
        app.environment.window_system.display,
        win,
        RevertToPointerRoot,
        CurrentTime,
    );
    // Arrange current workspace
    arrange(app);
    // Finish mapping
    map_window(app.environment.window_system.display, win);
    log!("   |- Mapped window");
}

/// Arranges windows of current workspace in specified layout
fn arrange(app: &mut ApplicationContainer) {
    log!("   |- Arranging...");
    let ws = &mut app.environment.window_system;
    // Go thru all screens
    for screen in &mut ws.screens {
        // Gap width
        let gw = app.environment.config.visual_preferences.gap_width as i32;
        let bs = app.environment.config.visual_preferences.border_size as u32;
        // Get amount of visible not floating clients to arrange
        let stack_size = screen.workspaces[screen.current_workspace]
            .clients
            .iter()
            .filter(|&c| !c.floating)
            .count();
        // Get widths of master and stack areas
        let mut master_width = ((screen.width as i32 - gw * 3) as f64
            * screen.workspaces[screen.current_workspace].master_width)
            as u32;
        let stack_width = (screen.width as i32 - gw * 3) - master_width as i32;
        let mut master_capacity = screen.workspaces[screen.current_workspace].master_capacity;
        // if master_capacity out of stack_size bounds use whole screen for one column
        if master_capacity <= 0 || master_capacity >= stack_size as i64 {
            master_capacity = stack_size as i64;
            master_width = screen.width as u32 - gw as u32 * 2;
        }
        log!("   |- Arranging {} window", stack_size);
        // Iterate over all tileable clients structs
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
            } else if (index as i64) < master_capacity {
                // if master_capacity is not full put it here
                log!("      |- Goes to master");
                // some math...
                let win_height =
                    (screen.height - gw as i64 - master_capacity * gw as i64) / master_capacity;
                // Add gap offset to the left
                client.x = gw;
                // Top gap + clients with their gaps offset
                client.y = gw + (win_height as i32 + gw) * index as i32;
                client.w = master_width - 2 * bs;
                client.h = win_height as u32 - 2 * bs
            } else {
                // otherwise put it in secondary stack
                log!("      |- Goes to stack");
                // a bit more of math...
                let win_height =
                    (screen.height - gw as i64 - (stack_size as i64 - master_capacity) * gw as i64)
                        / (stack_size as i64 - master_capacity);
                client.x = master_width as i32 + (gw * 2);
                client.y = gw + (win_height as i32 + gw) * (index as i64 - master_capacity) as i32;
                client.w = stack_width as u32 - 2 * bs;
                client.h = win_height as u32 - 2 * bs;
            }
        }

        // update corresponding windows
        for client in &screen.workspaces[screen.current_workspace].clients {
            if client.fullscreen {
                move_resize_window(
                    ws.display,
                    client.window_id,
                    screen.x as i32,
                    screen.y as i32,
                    screen.width as u32,
                    screen.height as u32,
                );
                set_window_border_width(ws.display, client.window_id, 0);
                raise_window(ws.display, client.window_id);
            } else {
                set_window_border_width(
                    ws.display,
                    client.window_id,
                    if stack_size > 1 {
                        app.environment.config.visual_preferences.border_size as u32
                    } else {
                        0
                    },
                );
                move_resize_window(
                    ws.display,
                    client.window_id,
                    client.x + screen.x as i32,
                    client.y + screen.y as i32,
                    client.w,
                    client.h,
                );
            };
        }
    }
}

/// Returns window, workspace and client indexies for client with specified id
fn find_window_indexes(app: &mut ApplicationContainer, win: u64) -> Option<(usize, usize, usize)> {
    let ws = &mut app.environment.window_system;
    for s in 0..ws.screens.len() {
        for w in 0..ws.screens[s].workspaces.len() {
            for c in 0..ws.screens[s].workspaces[w].clients.len() {
                if ws.screens[s].workspaces[w].clients[c].window_id == win {
                    return Some((s, w, c));
                }
            }
        }
    }
    None
}

/// Shows/Hides all windows on current workspace
fn show_hide_workspace(app: &mut ApplicationContainer) {
    let ws = &mut app.environment.window_system;
    // Iterate over all clients
    for client in &mut ws.screens[ws.current_screen].workspaces[ws.current_workspace].clients {
        if client.visible {
            // move window to left top
            move_resize_window(
                ws.display,
                client.window_id,
                -(client.w as i32),
                -(client.h as i32),
                client.w,
                client.h,
            );
        }
        // flip visibility state
        client.visible = !client.visible;
    }
}

/// Shifts current client tracker after destroying clients
fn shift_current_client(
    app: &mut ApplicationContainer,
    screen: Option<usize>,
    workspace: Option<usize>,
) {
    let screen = match screen {
        Some(index) => index,
        None => app.environment.window_system.current_screen,
    };

    let workspace = match workspace {
        Some(index) => index,
        None => app.environment.window_system.current_workspace,
    };

    let ws = &mut app.environment.window_system;
    // Find next client
    ws.screens[screen].workspaces[workspace].current_client = {
        // Get reference to windows stack
        let clients = &ws.screens[screen].workspaces[workspace].clients;
        if clients.is_empty() {
            // None if no windows
            None
        } else {
            // Get old client index
            let cc = ws.screens[screen].workspaces[workspace]
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
    ws.current_client = ws.screens[screen].workspaces[workspace].current_client;
    if let Some(index) = ws.current_client {
        let win = ws.screens[screen].workspaces[workspace].clients[index].window_id;
        set_input_focus(ws.display, win, RevertToPointerRoot, CurrentTime);
    }
    update_active_window(app);
}

/// Safely sends atom to X server
fn send_atom(app: &mut ApplicationContainer, win: u64, e: x11::xlib::Atom) -> bool {
    if let Some(ps) = get_wm_protocols(app.environment.window_system.display, win) {
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
                    configure: None,
                    client: Some(x11::xlib::XClientMessageEvent {
                        type_: ClientMessage,
                        serial: 0,
                        send_event: 0,
                        display: null_mut(),
                        window: win,
                        message_type: app.environment.window_system.atoms.wm_protocols,
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
                return send_event(
                    app.environment.window_system.display,
                    win,
                    false,
                    NoEventMask,
                    &mut ev,
                );
            }
        }
        false
    } else {
        false
    }
}

fn get_atom_prop(app: &mut ApplicationContainer, win: u64, prop: Atom) -> Atom {
    // return 0;
    let mut dummy_atom: u64 = 0;
    let mut dummy_int = 0;
    let mut dummy_long: u64 = 0;
    let mut property_return: *mut u8 = std::ptr::null_mut::<u8>();
    let mut atom: u64 = 0;
    unsafe {
        if x11::xlib::XGetWindowProperty(
            app.environment.window_system.display,
            win,
            prop,
            0,
            size_of::<Atom>() as i64,
            0,
            XA_ATOM,
            &mut dummy_atom as *mut u64,
            &mut dummy_int as *mut i32,
            &mut dummy_long as *mut u64,
            &mut dummy_long as *mut u64,
            &mut property_return as *mut *mut u8,
        ) == Success as i32
            && property_return as usize != 0
        {
            atom = *(property_return as *mut Atom);
            x11::xlib::XFree(property_return as *mut libc::c_void);
        }
    }
    atom
}

/// Removes window from window_system
fn unmanage_window(app: &mut ApplicationContainer, win: u64) {
    // Find trackers for window
    if let Some((s, w, c)) = find_window_indexes(app, win) {
        log!("   |- Found window {} at indexes {}, {}, {}", win, s, w, c);
        // Removed corresponding client from stack
        let clients = &mut app.environment.window_system.screens[s].workspaces[w].clients;
        clients.remove(c);
        // Update client tracker
        shift_current_client(app, Some(s), Some(w));
        // Rearrange
        arrange(app);
        // update client List
        update_client_list(app);
    } else {
        log!("   |- Window is not managed");
    }
}

fn update_active_window(app: &mut ApplicationContainer) {
    let ws = &mut app.environment.window_system;
    if let Some(index) = ws.current_client {
        let win =
            ws.screens[ws.current_screen].workspaces[ws.current_workspace].clients[index].window_id;
        change_property(
            ws.display,
            ws.root_win,
            ws.atoms.net_active_window,
            XA_WINDOW,
            32,
            PropModeReplace,
            &win as *const u64 as *mut u8,
            1,
        );
    }
}

fn get_current_client_id(app: &mut ApplicationContainer) -> Option<u64> {
    let ws = &app.environment.window_system;
    ws.current_client.map(|index| ws.screens[ws.current_screen].workspaces[ws.current_workspace].clients[index].window_id)
}

fn update_trackers(app: &mut ApplicationContainer, win: u64) {
    if let Some((s, w, c)) = find_window_indexes(app, win) {
        let ws = &mut app.environment.window_system;
        ws.current_screen = s;
        ws.current_workspace = w;
        ws.screens[s].current_workspace = w;
        ws.current_client = Some(c);
        ws.screens[s].workspaces[w].current_client = Some(c);
    };
}

fn spawn(app: &mut ApplicationContainer, cmd: &str) {
    unsafe {
        match nix::unistd::fork() {
            Ok(nix::unistd::ForkResult::Parent { child: _ }) => {
                log!("     |- Spawned");
            }
            Ok(nix::unistd::ForkResult::Child) => {
                log!("CHILD SPAWNED");
                if app.environment.window_system.display as *mut x11::xlib::Display as usize != 0 {
                    nix::unistd::close(x11::xlib::XConnectionNumber(
                        app.environment.window_system.display,
                    ))
                    .unwrap();
                }
                let args = [
                    &std::ffi::CString::new("/usr/bin/sh").unwrap(),
                    &std::ffi::CString::new("-c").unwrap(),
                    &std::ffi::CString::new(cmd.to_owned()).unwrap(),
                ];
                let _ = nix::unistd::execvp(args[0], &args);
            }
            Err(_) => println!("Fork Failed"),
        }
    }
}

fn update_client_list(app: &mut ApplicationContainer) {
    let ws = &mut app.environment.window_system;

    delete_property(ws.display, ws.root_win, ws.atoms.net_client_list);

    for screen in &app.environment.window_system.screens {
        for workspace in &screen.workspaces {
            for client in &workspace.clients {
                change_property(
                    app.environment.window_system.display,
                    app.environment.window_system.root_win,
                    app.environment.window_system.atoms.net_client_list,
                    XA_WINDOW,
                    32,
                    PropModeAppend,
                    &client.window_id as *const u64 as *mut u8,
                    1,
                );
            }
        }
    }
}

/// Starts main WM loop
// fn run(config: &ConfigurationContainer, ws: &mut WindowSystemContainer) {
fn run(app: &mut ApplicationContainer) {
    log!("|===== run =====");
    loop {
        // Process Events
        let ev = next_event(app.environment.window_system.display);
        // Do pattern matching for known events
        match ev.type_ {
            x11::xlib::KeyPress => {
                log!("|- Got keyboard event");
                // Safely retrive struct
                let ev = ev.key.unwrap();
                // Iterate over key actions matching current key input
                for action in app.environment.config.key_actions.clone() {
                    if ev.keycode
                        == keysym_to_keycode(app.environment.window_system.display, action.keysym)
                        && ev.state == action.modifier
                    {
                        // Log action type
                        log!("|- Got {:?} action", &action.result);
                        // Match action result and run related function
                        match &action.result {
                            ActionResult::KillClient => {
                                log!("   |- Got `KillClient` Action");
                                // Check if there any windows selected
                                match app.environment.window_system.current_client {
                                    Some(index) => {
                                        let id = app.environment.window_system.screens
                                            [app.environment.window_system.current_screen]
                                            .workspaces
                                            [app.environment.window_system.current_workspace]
                                            .clients[index]
                                            .window_id;
                                        log!("      |- Killing window {}", id);
                                        if !send_atom(
                                            app,
                                            id,
                                            app.environment.window_system.atoms.wm_delete,
                                        ) {
                                            grab_server(app.environment.window_system.display);
                                            set_close_down_mode(
                                                app.environment.window_system.display,
                                                DestroyAll,
                                            );
                                            kill_client(app.environment.window_system.display, id);
                                            ungrab_server(app.environment.window_system.display);
                                        };
                                    }
                                    None => {
                                        log!("      |- No window selected");
                                    }
                                };
                            }
                            ActionResult::Spawn(cmd) => {
                                log!("   |- Got `Spawn` Action");
                                spawn(app, cmd);
                                // // Run sh with specified command
                                // let mut handle = Command::new("/usr/bin/sh")
                                //     .arg("-c")
                                //     .arg(cmd)
                                //     .spawn()
                                //     .expect(format!("can't execute {cmd}").as_str());
                                // // Run it in sepated thread
                                // // TODO: WHERE ARE HANDLERS STORED
                                // let _lost_handle = std::thread::Builder::new().name(cmd.to_owned()).spawn(move || {
                                //     handle.wait().expect("can't run process");
                                // });
                            }
                            ActionResult::MoveToScreen(d) => {
                                // Check if window is selected
                                if let Some(index) = app.environment.window_system.current_client {
                                    // Get current screen index
                                    let mut cs = app.environment.window_system.current_screen;
                                    // Update index depending on supplied direction
                                    cs = match d {
                                        ScreenSwitching::Next => {
                                            (cs + 1) % app.environment.window_system.screens.len()
                                        }
                                        ScreenSwitching::Previous => {
                                            (cs + app.environment.window_system.screens.len() - 1)
                                                % app.environment.window_system.screens.len()
                                        }
                                    };
                                    // Pop client
                                    let cc = app.environment.window_system.screens
                                        [app.environment.window_system.current_screen]
                                        .workspaces
                                        [app.environment.window_system.current_workspace]
                                        .clients
                                        .remove(index);
                                    set_window_border(
                                        app.environment.window_system.display,
                                        cc.window_id,
                                        app.environment
                                            .config
                                            .visual_preferences
                                            .normal_border_color,
                                    );
                                    // Update client tracker on current screen
                                    shift_current_client(app, None, None);
                                    // Get workspace tracker(borrow checker is really mad at me)
                                    let nw =
                                        app.environment.window_system.screens[cs].current_workspace;
                                    // Add window to stack of another display
                                    app.environment.window_system.screens[cs].workspaces[nw]
                                        .clients
                                        .push(cc);
                                    // Arrange all monitors
                                    arrange(app);
                                }
                            }
                            ActionResult::FocusOnScreen(d) => {
                                // Get current screen
                                let mut cs = app.environment.window_system.current_screen;
                                // Update it
                                cs = match d {
                                    ScreenSwitching::Next => {
                                        (cs + 1) % app.environment.window_system.screens.len()
                                    }
                                    ScreenSwitching::Previous => {
                                        (cs + app.environment.window_system.screens.len() - 1)
                                            % app.environment.window_system.screens.len()
                                    }
                                };
                                if let Some(cw) = get_current_client_id(app) {
                                    set_window_border(
                                        app.environment.window_system.display,
                                        cw,
                                        app.environment
                                            .config
                                            .visual_preferences
                                            .normal_border_color,
                                    );
                                }
                                // Change trackers
                                app.environment.window_system.current_screen = cs;
                                app.environment.window_system.current_workspace =
                                    app.environment.window_system.screens
                                        [app.environment.window_system.current_screen]
                                        .current_workspace;
                                app.environment.window_system.current_client = app
                                    .environment
                                    .window_system
                                    .screens[app.environment.window_system.current_screen]
                                    .workspaces[app.environment.window_system.current_workspace]
                                    .current_client;
                                if let Some(index) = app.environment.window_system.current_client {
                                    let win = app.environment.window_system.screens
                                        [app.environment.window_system.current_screen]
                                        .workspaces
                                        [app.environment.window_system.current_workspace]
                                        .clients[index]
                                        .window_id;
                                    set_input_focus(
                                        app.environment.window_system.display,
                                        win,
                                        RevertToPointerRoot,
                                        CurrentTime,
                                    );
                                    update_active_window(app);
                                }
                                if let Some(cw) = get_current_client_id(app) {
                                    set_window_border(
                                        app.environment.window_system.display,
                                        cw,
                                        app.environment
                                            .config
                                            .visual_preferences
                                            .active_border_color,
                                    );
                                }
                            }
                            ActionResult::MoveToWorkspace(n) => {
                                log!("   |- Got `MoveToWorkspace` Action ");
                                // Check if moving to another workspace
                                if *n as usize != app.environment.window_system.current_workspace {
                                    // Check if any client is selected
                                    if let Some(index) =
                                        app.environment.window_system.current_client
                                    {
                                        // Pop current client
                                        let mut cc = app.environment.window_system.screens
                                            [app.environment.window_system.current_screen]
                                            .workspaces
                                            [app.environment.window_system.current_workspace]
                                            .clients
                                            .remove(index);
                                        set_window_border(
                                            app.environment.window_system.display,
                                            cc.window_id,
                                            app.environment
                                                .config
                                                .visual_preferences
                                                .normal_border_color,
                                        );
                                        // Update current workspace layout
                                        arrange(app);
                                        // Move window out of view
                                        move_resize_window(
                                            app.environment.window_system.display,
                                            cc.window_id,
                                            -(cc.w as i32),
                                            -(cc.h as i32),
                                            cc.w,
                                            cc.h,
                                        );
                                        cc.visible = !cc.visible;
                                        // Update tracker
                                        shift_current_client(app, None, None);
                                        // Add client to choosen workspace (will be arranged later)
                                        app.environment.window_system.screens
                                            [app.environment.window_system.current_screen]
                                            .workspaces[*n as usize]
                                            .clients
                                            .push(cc);
                                    }
                                }
                            }
                            ActionResult::FocusOnWorkspace(n) => {
                                log!("   |- Got `FocusOnWorkspace` Action");
                                // Check is focusing on another workspace
                                if *n as usize != app.environment.window_system.current_workspace {
                                    // Hide current workspace
                                    show_hide_workspace(app);
                                    // unfocus current win
                                    if let Some(cw) = get_current_client_id(app) {
                                        set_window_border(
                                            app.environment.window_system.display,
                                            cw,
                                            app.environment
                                                .config
                                                .visual_preferences
                                                .normal_border_color,
                                        );
                                    }
                                    // Update workspace index
                                    app.environment.window_system.current_workspace = *n as usize;
                                    app.environment.window_system.screens
                                        [app.environment.window_system.current_screen]
                                        .current_workspace = *n as usize;
                                    // Update current client
                                    app.environment.window_system.current_client =
                                        app.environment.window_system.screens
                                            [app.environment.window_system.current_screen]
                                            .workspaces
                                            [app.environment.window_system.current_workspace]
                                            .current_client;
                                    if let Some(cw) = get_current_client_id(app) {
                                        set_window_border(
                                            app.environment.window_system.display,
                                            cw,
                                            app.environment
                                                .config
                                                .visual_preferences
                                                .active_border_color,
                                        );
                                    }
                                    // Show current client
                                    show_hide_workspace(app);
                                    // Arrange update workspace
                                    arrange(app);
                                    if let Some(index) =
                                        app.environment.window_system.current_client
                                    {
                                        let win = app.environment.window_system.screens
                                            [app.environment.window_system.current_screen]
                                            .workspaces
                                            [app.environment.window_system.current_workspace]
                                            .clients[index]
                                            .window_id;
                                        set_input_focus(
                                            app.environment.window_system.display,
                                            win,
                                            RevertToPointerRoot,
                                            CurrentTime,
                                        );
                                        update_active_window(app);
                                    }
                                }
                            }
                            ActionResult::Quit => {
                                log!("   |- Got `Quit` Action. `Quiting`");
                                app.environment.window_system.running = false;
                            }
                            ActionResult::UpdateMasterCapacity(i) => {
                                // Change master size
                                app.environment.window_system.screens
                                    [app.environment.window_system.current_screen]
                                    .workspaces[app.environment.window_system.current_workspace]
                                    .master_capacity += *i;
                                // Rearrange windows
                                arrange(app);
                            }
                            ActionResult::UpdateMasterWidth(w) => {
                                // Update master width
                                app.environment.window_system.screens
                                    [app.environment.window_system.current_screen]
                                    .workspaces[app.environment.window_system.current_workspace]
                                    .master_width += *w;
                                // Rearrange windows
                                arrange(app);
                            }
                            ActionResult::DumpInfo => {
                                // Dump all info to log
                                log!("{:#?}", &app.environment.window_system);
                            }
                            ActionResult::ToggleFloat => {
                                if let Some(c) = app.environment.window_system.current_client {
                                    let state = app.environment.window_system.screens
                                        [app.environment.window_system.current_screen]
                                        .workspaces
                                        [app.environment.window_system.current_workspace]
                                        .clients[c]
                                        .floating;
                                    app.environment.window_system.screens
                                        [app.environment.window_system.current_screen]
                                        .workspaces
                                        [app.environment.window_system.current_workspace]
                                        .clients[c]
                                        .floating = !state;
                                    arrange(app);
                                }
                            }
                            ActionResult::CycleStack(_i) => {}
                        }
                    }
                }
            }

            // Used for adding windows to WM
            x11::xlib::MapRequest => {
                let ew: u64 = ev.map_request.unwrap().window;
                log!("|- Map Request From Window: {ew}");
                manage_client(app, ew);
            }

            // Checks if window got crossed, used for focus and monitor tracking
            x11::xlib::EnterNotify => {
                let ew: u64 = ev.crossing.unwrap().window;
                log!(
                    "|- Crossed Window `{}` ({})",
                    get_client_name(app, ew).unwrap_or("Unmanaged window".to_string()),
                    ew
                );
                if ew != app.environment.window_system.root_win {
                    log!("   |- Setting focus to window");
                    // Focus on crossed window
                    if let Some(cw) = get_current_client_id(app) {
                        set_window_border(
                            app.environment.window_system.display,
                            cw,
                            app.environment
                                .config
                                .visual_preferences
                                .normal_border_color,
                        );
                    }
                    set_window_border(
                        app.environment.window_system.display,
                        ew,
                        app.environment
                            .config
                            .visual_preferences
                            .active_border_color,
                    );
                    update_trackers(app, ew);
                    update_active_window(app);
                    set_input_focus(
                        app.environment.window_system.display,
                        ew,
                        RevertToPointerRoot,
                        CurrentTime,
                    );
                } else {
                    let ws = &mut app.environment.window_system;

                    if ws.screens[ws.current_screen].workspaces[ws.current_workspace]
                        .clients
                        .is_empty()
                    {
                        set_input_focus(ws.display, ws.root_win, RevertToPointerRoot, CurrentTime);
                        delete_property(ws.display, ws.root_win, ws.atoms.net_active_window);
                    }
                }
            }

            // Used to unmanage_window
            x11::xlib::DestroyNotify => {
                let ew: u64 = ev.destroy_window.unwrap().window;
                log!(
                    "|- `{}` destroyed",
                    get_client_name(app, ew).unwrap_or("Unmanaged window".to_string())
                );
                unmanage_window(app, ew);
            }

            // Used to unmanage_window
            x11::xlib::UnmapNotify => {
                let ew: u64 = ev.unmap.unwrap().window;
                log!(
                    "|- `{}` unmapped",
                    get_client_name(app, ew).unwrap_or("Unmanaged window".to_string())
                );
                unmanage_window(app, ew);
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
                for screen in &app.environment.window_system.screens {
                    // Check if mouse position "inside" screens
                    if screen.x <= x
                        && x < screen.x + screen.width
                        && screen.y <= y
                        && y < screen.y + screen.height
                    {
                        // Update trackers
                        app.environment.window_system.current_screen = screen.number as usize;
                        app.environment.window_system.current_workspace =
                            app.environment.window_system.screens
                                [app.environment.window_system.current_screen]
                                .current_workspace;
                        app.environment.window_system.current_client =
                            app.environment.window_system.screens
                                [app.environment.window_system.current_screen]
                                .workspaces[app.environment.window_system.current_workspace]
                                .current_client;
                    }
                }
            }

            // Used for updating names of applications
            x11::xlib::PropertyNotify => {
                // Safely retrive event struct
                let p = ev.property.unwrap();
                log!("|- Got property notify");
                // If current window is not root proceed to updating name
                if p.window != app.environment.window_system.root_win {
                    log!(
                        "|- `Property` changed for window {} `{}`",
                        p.window,
                        get_client_name(app, p.window).unwrap_or("Unmanaged window".to_string())
                    );
                    update_client_name(app, p.window);
                }
            }
            x11::xlib::ConfigureNotify => {
                let c = ev.configure.unwrap();
                log!("|- Got ConfigureNotify");
                if c.window == app.environment.window_system.root_win {
                    let n = app.environment.window_system.screens.len();
                    let screens = xinerama_query_screens(app.environment.window_system.display)
                        .expect("Running without xinerama is not supported");
                    let screens_amount = screens.len();
                    for _ in n..screens_amount {
                        app.environment.window_system.screens.push(Screen {
                            number: 0,
                            x: 0,
                            y: 0,
                            width: 0,
                            height: 0,
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
                    for (index, screen) in screens.iter().enumerate() {
                        app.environment.window_system.screens[index].number =
                            screen.screen_number as i64;
                        app.environment.window_system.screens[index].x = screen.x_org as i64;
                        app.environment.window_system.screens[index].y = screen.y_org as i64;
                        app.environment.window_system.screens[index].width = screen.width as i64;
                        app.environment.window_system.screens[index].height = screen.height as i64;
                    }
                    for _ in screens_amount..n {
                        let lsw = app
                            .environment
                            .window_system
                            .screens
                            .pop()
                            .unwrap()
                            .workspaces;
                        for (index, workspace) in lsw.into_iter().enumerate() {
                            app.environment.window_system.screens[0].workspaces[index]
                                .clients
                                .extend(workspace.clients);
                        }
                    }
                    arrange(app);
                }
            }
            x11::xlib::ClientMessage => {
                let c = ev.client.unwrap();
                if let Some(cc) = find_window_indexes(app, c.window) {
                    let cc = &mut app.environment.window_system.screens[cc.0].workspaces[cc.1]
                        .clients[cc.2];
                    log!("|- Got `message`");
                    log!("   |- From: `{}`", &cc.window_name);
                    if c.message_type == app.environment.window_system.atoms.net_wm_state {
                        log!("   |- Type: `window state`");
                        if c.data.get_long(1) as u64
                            == app.environment.window_system.atoms.net_wm_fullscreen
                            || c.data.get_long(2) as u64
                                == app.environment.window_system.atoms.net_wm_fullscreen
                        {
                            let sf =
                                c.data.get_long(0) == 1 || c.data.get_long(0) == 2 && cc.fullscreen;
                            if sf && !cc.fullscreen {
                                change_property(
                                    app.environment.window_system.display,
                                    c.window,
                                    app.environment.window_system.atoms.net_wm_state,
                                    XA_ATOM,
                                    32,
                                    PropModeReplace,
                                    &mut app.environment.window_system.atoms.net_wm_fullscreen
                                        as *mut u64 as *mut u8,
                                    1,
                                );
                                cc.fullscreen = true;
                                arrange(app);
                            } else if !sf && cc.fullscreen {
                                change_property(
                                    app.environment.window_system.display,
                                    c.window,
                                    app.environment.window_system.atoms.net_wm_state,
                                    XA_ATOM,
                                    32,
                                    PropModeReplace,
                                    std::ptr::null_mut::<u8>(),
                                    0,
                                );
                                cc.fullscreen = false;
                                arrange(app);
                            }
                        }
                    }
                }
            }
            _ => {}
        };
        if !app.environment.window_system.running {
            break;
        }
    }
}

/// Closes all connections and saves info
fn cleanup(_app: &mut ApplicationContainer) {}

/// Rust default entry function
fn main() {
    // Set locale for proper work with unicode symbols
    // For example: getting names of windows
    set_locale(LC_CTYPE, "");

    // Disable zombie processes
    unsafe {
        let sa = nix::sys::signal::SigAction::new(
            nix::sys::signal::SigHandler::SigIgn,
            nix::sys::signal::SaFlags::SA_NOCLDSTOP
                | nix::sys::signal::SaFlags::SA_NOCLDWAIT
                | nix::sys::signal::SaFlags::SA_RESTART,
            nix::sys::signal::SigSet::empty(),
        );
        let _ = nix::sys::signal::sigaction(nix::sys::signal::Signal::SIGCHLD, &sa);
    }

    // Run autostart
    // Command::new("/usr/bin/sh")
    //     .arg(format!("{}/{}", std::env!("HOME"), ".rtde/autostart.sh"))
    //     .status()
    //     .unwrap();

    // Init `app` container
    // App container consists of all data needed for WM to function
    let mut app: ApplicationContainer = setup();

    spawn(
        &mut app,
        &format!(
            "{} {}/{}",
            "/usr/bin/sh",
            std::env!("HOME"),
            ".rtde/autostart.sh"
        ),
    );

    // Scan for existing windows
    scan(&mut app);

    // Start main loop
    // Event processing, and all stuff occur here
    run(&mut app);

    // Gracefully exit
    // Close all connections, dump data, exit
    cleanup(&mut app);
}
