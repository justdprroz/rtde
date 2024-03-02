//! A window manager written in Rust with nearly same functionality of [dwm](https://dwm.suckless.org/)
//!
//! List of features supported by rwm:
//! - Multi monitor setup
//! - Workspaces aka tags
//! - Stack layout
//! - Shortcuts

mod structs;
mod wrap;

use crate::structs::*;
use crate::wrap::xinerama::*;
use crate::wrap::xlib::*;
use libc::LC_CTYPE;
use std::ffi::CString;
use std::mem::size_of;
use std::ptr::null_mut;
use std::vec;
use x11::keysym::*;
use x11::xlib::Atom;
use x11::xlib::ButtonPressMask;
use x11::xlib::CWBorderWidth;
use x11::xlib::CWCursor;
use x11::xlib::CWEventMask;
use x11::xlib::CWHeight;
use x11::xlib::CWWidth;
use x11::xlib::ClientMessage;
use x11::xlib::CurrentTime;
use x11::xlib::DestroyAll;
use x11::xlib::EnterWindowMask;
use x11::xlib::FocusChangeMask;
use x11::xlib::IsViewable;
use x11::xlib::LeaveWindowMask;
use x11::xlib::Mod4Mask as ModKey;
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
use x11::xlib::XWindowAttributes;
use x11::xlib::XWindowChanges;
use x11::xlib::CWX;
use x11::xlib::CWY;
use x11::xlib::XA_ATOM;
use x11::xlib::XA_CARDINAL;
use x11::xlib::XA_WINDOW;

fn argb_to_int(c: Color) -> u64 {
    (c.alpha as u64) << 24 | (c.red as u64) << 16 | (c.green as u64) << 8 | (c.blue as u64)
}

fn vec_string_to_bytes(strings: Vec<String>) -> Vec<u8> {
    let mut bytes: Vec<u8> = vec![];
    for string in strings {
        let mut b = CString::new(string).unwrap().into_bytes_with_nul();
        bytes.append(&mut b);
    }
    bytes
}

macro_rules! log {
    ($($e:expr),+) => {
        #[cfg(debug_assertions)]
        println!($($e),+);
    };
}

const EVENT_LOOKUP: [&str; 37] = [
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
    "LASTEvent",
];

fn setup() -> ApplicationContainer {
    log!("|===== setup =====");
    let display = open_display(None).expect("Failed to open display");

    let mut app = ApplicationContainer {
        config: ConfigurationContainer {
            gap_width: 4,
            border_size: 2,
            normal_border_color: Color {
                alpha: 255,
                red: 64,
                green: 64,
                blue: 128,
            },
            active_border_color: Color {
                alpha: 255,
                red: 126,
                green: 36,
                blue: 135,
            },
            key_actions: Vec::new(),
        },
        runtime: RuntimeContainer {
            display,
            root_win: 0,
            wm_check_win: 0,
            running: true,
            screens: Vec::new(),
            current_screen: 0,
            current_workspace: 0,
            current_client: None,
            bars: vec![],
        },
        atoms: Atoms {
            utf8string: 0,
            wm_protocols: 0,
            wm_delete: 0,
            wm_state: 0,
            wm_name: 0,
            net_wm_check: 0,
            wm_take_focus: 0,
            net_active_window: 0,
            net_supported: 0,
            net_wm_name: 0,
            net_wm_state: 0,
            net_wm_fullscreen: 0,
            net_wm_window_type: 0,
            net_wm_window_type_dock: 0,
            net_wm_window_type_dialog: 0,
            net_client_list: 0,
            net_number_of_desktops: 0,
            net_current_desktop: 0,
            net_desktop_viewport: 0,
            net_desktop_names: 0,
            net_wm_desktop: 0,
        },
    };

    app.runtime.root_win = default_root_window(app.runtime.display);

    init_actions(&mut app);
    init_supported_atoms(&mut app);
    init_wm_check(&mut app);
    init_screens(&mut app);
    set_error_handler();

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

    wa.event_mask = SubstructureRedirectMask
        | LeaveWindowMask
        | EnterWindowMask
        | SubstructureNotifyMask
        | StructureNotifyMask
        | PointerMotionMask
        | ButtonPressMask
        | PropertyChangeMask;

    change_window_attributes(
        app.runtime.display,
        app.runtime.root_win,
        CWEventMask | CWCursor,
        &mut wa,
    );

    select_input(app.runtime.display, app.runtime.root_win, wa.event_mask);

    app
}

fn init_wm_check(app: &mut ApplicationContainer) {
    app.runtime.wm_check_win = create_simple_window(
        app.runtime.display,
        app.runtime.root_win,
        0,
        0,
        1,
        1,
        0,
        0,
        0,
    );
    let mut wmchckwin = app.runtime.wm_check_win;

    change_property(
        app.runtime.display,
        wmchckwin,
        app.atoms.net_wm_check,
        XA_WINDOW,
        32,
        PropModeReplace,
        &mut wmchckwin as *mut u64 as *mut u8,
        1,
    );

    let wm_name = std::ffi::CString::new("rtwm".to_string()).unwrap();
    change_property(
        app.runtime.display,
        wmchckwin,
        app.atoms.net_wm_name,
        app.atoms.utf8string,
        8,
        PropModeReplace,
        wm_name.as_ptr() as *mut u8,
        7,
    );

    change_property(
        app.runtime.display,
        app.runtime.root_win,
        app.atoms.net_wm_check,
        XA_WINDOW,
        32,
        PropModeReplace,
        &mut wmchckwin as *mut u64 as *mut u8,
        1,
    );
}

fn init_actions(app: &mut ApplicationContainer) {
    let actions: Vec<KeyAction> = {
        use ActionResult::*;

        let terminal: String = "kitty".to_string();
        let file_manager: String = "thunar".to_string();
        let app_launcher: String = "dmenu_run -p \"Open app:\" -sb \"#944b9c\" -nb \"#111222\" -sf \"#ffffff\" -nf \"#9b989c\" -fn \"monospace:size=10\" -b".to_string();
        let screenshot: String = "screenshot".to_string();

        let mut a = vec![
            KeyAction {
                modifier: ModKey,
                keysym: XK_Return,
                result: Spawn(terminal),
            },
            KeyAction {
                modifier: ModKey,
                keysym: XK_e,
                result: Spawn(file_manager),
            },
            KeyAction {
                modifier: ModKey,
                keysym: XK_p,
                result: Spawn(app_launcher),
            },
            KeyAction {
                modifier: 0,
                keysym: XF86XK_AudioRaiseVolume,
                result: Spawn("volumeup".to_string()),
            },
            KeyAction {
                modifier: 0,
                keysym: XF86XK_AudioLowerVolume,
                result: Spawn("volumedown".to_string()),
            },
            KeyAction {
                modifier: 0,
                keysym: XF86XK_AudioMute,
                result: Spawn("volumemute".to_string()),
            },
            KeyAction {
                modifier: 0,
                keysym: XF86XK_AudioPlay,
                result: Spawn("playerctl play-pause".to_string()),
            },
            KeyAction {
                modifier: 0,
                keysym: XF86XK_AudioNext,
                result: Spawn("playerctl next".to_string()),
            },
            KeyAction {
                modifier: 0,
                keysym: XF86XK_AudioPrev,
                result: Spawn("playerctl previous".to_string()),
            },
            KeyAction {
                modifier: ModKey | ShiftMask,
                keysym: XK_s,
                result: Spawn(screenshot),
            },
            KeyAction {
                modifier: ModKey | ShiftMask,
                keysym: XK_q,
                result: Quit,
            },
            KeyAction {
                modifier: ModKey | ShiftMask,
                keysym: XK_c,
                result: KillClient,
            },
            KeyAction {
                modifier: ModKey,
                keysym: XK_w,
                result: DumpInfo,
            },
            KeyAction {
                modifier: ModKey,
                keysym: XK_comma,
                result: FocusOnScreen(ScreenSwitching::Previous),
            },
            KeyAction {
                modifier: ModKey,
                keysym: XK_period,
                result: FocusOnScreen(ScreenSwitching::Next),
            },
            KeyAction {
                modifier: ModKey | ShiftMask,
                keysym: XK_comma,
                result: MoveToScreen(ScreenSwitching::Previous),
            },
            KeyAction {
                modifier: ModKey | ShiftMask,
                keysym: XK_period,
                result: MoveToScreen(ScreenSwitching::Next),
            },
            KeyAction {
                modifier: ModKey,
                keysym: XK_i,
                result: UpdateMasterCapacity(1),
            },
            KeyAction {
                modifier: ModKey,
                keysym: XK_d,
                result: UpdateMasterCapacity(-1),
            },
            KeyAction {
                modifier: ModKey,
                keysym: XK_l,
                result: UpdateMasterWidth(0.05),
            },
            KeyAction {
                modifier: ModKey,
                keysym: XK_h,
                result: UpdateMasterWidth(-0.05),
            },
            KeyAction {
                modifier: ModKey | ShiftMask,
                keysym: XK_space,
                result: ToggleFloat,
            },
            KeyAction {
                modifier: ModKey,
                keysym: XK_j,
                result: CycleStack(-1),
            },
            KeyAction {
                modifier: ModKey,
                keysym: XK_k,
                result: CycleStack(1),
            },
        ];

        for (index, key) in [XK_1, XK_2, XK_3, XK_4, XK_5, XK_6, XK_7, XK_8, XK_9, XK_0]
            .iter()
            .enumerate()
        {
            a.push(KeyAction {
                modifier: ModKey,
                keysym: *key,
                result: FocusOnWorkspace(index as u64),
            });
            a.push(KeyAction {
                modifier: ModKey | ShiftMask,
                keysym: *key,
                result: MoveToWorkspace(index as u64),
            });
        }
        a
    };

    app.config.key_actions = actions;
    for action in app.config.key_actions.iter() {
        grab_key(app.runtime.display, action.keysym, action.modifier);
    }
}

fn init_supported_atoms(app: &mut ApplicationContainer) {
    // let dpy = &mut app.runtime.display;
    macro_rules! intern_atom {
        ($e:expr) => {
            intern_atom(app.runtime.display, $e.to_string(), false)
        };
    }
    app.atoms = Atoms {
        utf8string: intern_atom!("UTF8_STRING"),
        wm_protocols: intern_atom!("WM_PROTOCOLS"),
        wm_delete: intern_atom!("WM_DELETE_WINDOW"),
        wm_state: intern_atom!("WM_STATE"),
        wm_name: intern_atom!("WM_NAME"),
        wm_take_focus: intern_atom!("WM_TAKE_FOCUS"),
        net_active_window: intern_atom!("_NET_ACTIVE_WINDOW"),
        net_supported: intern_atom!("_NET_SUPPORTED"),
        net_wm_name: intern_atom!("_NET_WM_NAME"),
        net_wm_state: intern_atom!("_NET_WM_STATE"),
        net_wm_check: intern_atom!("_NET_SUPPORTING_WM_CHECK"),
        net_wm_fullscreen: intern_atom!("_NET_WM_STATE_FULLSCREEN"),
        net_wm_window_type: intern_atom!("_NET_WM_WINDOW_TYPE"),
        net_wm_window_type_dialog: intern_atom!("_NET_WM_WINDOW_TYPE_DIALOG"),
        net_wm_window_type_dock: intern_atom!("_NET_WM_WINDOW_TYPE_DOCK"),
        net_client_list: intern_atom!("_NET_CLIENT_LIST"),
        net_number_of_desktops: intern_atom!("_NET_NUMBER_OF_DESKTOPS"),
        net_current_desktop: intern_atom!("_NET_CURRENT_DESKTOP"),
        net_desktop_names: intern_atom!("_NET_DESKTOP_NAMES"),
        net_desktop_viewport: intern_atom!("_NET_DESKTOP_VIEWPORT"),
        net_wm_desktop: intern_atom!("_NET_WM_DESKTOP"),
    };
    let mut netatoms = vec![
        app.atoms.net_active_window,
        app.atoms.net_supported,
        app.atoms.net_wm_name,
        app.atoms.net_wm_check,
        app.atoms.net_wm_fullscreen,
        app.atoms.net_wm_window_type,
        app.atoms.net_wm_window_type_dialog,
        app.atoms.net_client_list,
        app.atoms.net_wm_state,
        app.atoms.net_number_of_desktops,
        app.atoms.net_current_desktop,
        app.atoms.net_desktop_viewport,
        app.atoms.net_desktop_names,
    ];

    change_property(
        app.runtime.display,
        app.runtime.root_win,
        app.atoms.net_supported,
        x11::xlib::XA_ATOM,
        32,
        x11::xlib::PropModeReplace,
        netatoms.as_mut_ptr() as *mut u8,
        netatoms.len() as i32,
    );
}

fn init_screens(app: &mut ApplicationContainer) {
    let mut desktop_names = vec![];
    let mut viewports: Vec<i64> = vec![];
    let mut workspaces: u64 = 0;

    let n = app.runtime.screens.len();
    let screens = xinerama_query_screens(app.runtime.display)
        .expect("Running without xinerama is not supported");
    let screens_amount = screens.len();

    log!("   |- Add new empty screens");
    for i in n..screens_amount {
        log!("      |- Adding screen `{}`", i);
        app.runtime.screens.push(Screen {
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
                    });
                }
                wv
            },
            current_workspace: 0,
            bar_offsets: (0, 0, 0, 0),
        })
    }

    log!(
        "   |- Update all present screens geometries. Amount of screens: {}/{}",
        screens.len(),
        app.runtime.screens.len()
    );
    for (index, screen) in screens.iter().enumerate() {
        log!(
            "      |- Updating geometry for screen `{}` index:({})",
            screen.screen_number,
            index
        );
        app.runtime.screens[index].number = screen.screen_number as i64;
        app.runtime.screens[index].x = screen.x_org as i64;
        app.runtime.screens[index].y = screen.y_org as i64;
        app.runtime.screens[index].width = screen.width as i64;
        app.runtime.screens[index].height = screen.height as i64;
        log!("         |- Basic info updated");
        for i in 0..app.runtime.screens[index].workspaces.len() {
            desktop_names.push(format!("{}", i + 1));
            viewports.push(screen.x_org as i64);
            viewports.push(screen.y_org as i64);
            workspaces += 1;
        }
        log!("         |- Workspaces properties updated");
    }

    log!("   |- Remove exceeded screens");
    for i in screens_amount..n {
        log!("      |- Removing screen `{}`", i);
        let lsw = app.runtime.screens.pop().unwrap().workspaces;
        for (index, workspace) in lsw.into_iter().enumerate() {
            for client in workspace.clients {
                update_client_desktop(app, client.window_id, index as u64);
                app.runtime.screens[0].workspaces[index]
                    .clients
                    .push(client);
            }
        }
    }
    // Set amount of workspaces
    change_property(
        app.runtime.display,
        app.runtime.root_win,
        app.atoms.net_number_of_desktops,
        XA_CARDINAL,
        32,
        PropModeReplace,
        &mut workspaces as *mut u64 as *mut u8,
        1,
    );

    // Set workspaces names
    let mut bytes = vec_string_to_bytes(desktop_names);
    change_property(
        app.runtime.display,
        app.runtime.root_win,
        app.atoms.net_desktop_names,
        app.atoms.utf8string,
        8,
        PropModeReplace,
        bytes.as_mut_ptr(),
        bytes.len() as i32,
    );

    // Set workspaces viewports
    change_property(
        app.runtime.display,
        app.runtime.root_win,
        app.atoms.net_desktop_viewport,
        XA_CARDINAL,
        32,
        PropModeReplace,
        viewports.as_mut_ptr() as *mut u8,
        viewports.len() as i32,
    );
    arrange(app);
}

fn update_client_desktop(app: &mut ApplicationContainer, win: u64, desk: u64) {
    change_property(
        app.runtime.display,
        win,
        app.atoms.net_wm_desktop,
        XA_CARDINAL,
        32,
        PropModeReplace,
        &desk as *const u64 as *mut u8,
        1,
    );
}

/// Fetches clients that are already present
fn scan(app: &mut ApplicationContainer) {
    // let runtime = &mut app.runtime;
    log!("|===== scan =====");
    let (mut rw, _, wins) = query_tree(app.runtime.display, app.runtime.root_win);

    log!("|- Found {} window(s) that are already present", wins.len());

    for win in wins {
        log!("   |- Checking window {win}");
        let res = get_window_attributes(app.runtime.display, win);
        if let Some(wa) = res {
            if wa.override_redirect != 0
                || get_transient_for_hint(app.runtime.display, win, &mut rw) != 0
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
    let name = match get_text_property(app.runtime.display, win, app.atoms.net_wm_name) {
        Some(name) => name,
        None => "_".to_string(),
    };

    // Get trackers for specified window and change name
    if let Some((s, w, c)) = find_window_indexes(app, win) {
        app.runtime.screens[s].workspaces[w].clients[c].window_name = name;
    }
}
/// Returns name of specified client
fn get_client_name(app: &mut ApplicationContainer, win: u64) -> String {
    if let Some((s, w, c)) = find_window_indexes(app, win) {
        app.runtime.screens[s].workspaces[w].clients[c]
            .window_name
            .clone()
    } else {
        "Unmanaged Window".to_string()
    }
}

/// Adds client to runtime and configures it if needed
fn manage_client(app: &mut ApplicationContainer, win: u64) {
    let wa;

    // If thes is no proper window attributes - exit
    if let Some(a) = get_window_attributes(app.runtime.display, win) {
        if a.override_redirect == 0 {
            wa = a;
        } else {
            return;
        };
    } else {
        return;
    }

    // Is window is manager - exit
    if find_window_indexes(app, win).is_some() {
        return;
    }

    if get_atom_prop(app, win, app.atoms.net_wm_window_type) == app.atoms.net_wm_window_type_dock {
        attach_dock(app, &wa, win);
        map_window(app.runtime.display, win);
        select_input(
            app.runtime.display,
            win,
            StructureNotifyMask | SubstructureNotifyMask,
        );
        return;
    }

    // Create client
    let mut c: Client = Client::default();
    let mut trans = 0;

    // Set essential client fields
    c.window_id = win;
    c.w = wa.width as u32;
    c.h = wa.height as u32;
    c.x = wa.x
        + app.runtime.screens[app.runtime.current_screen]
            .bar_offsets
            .3 as i32;
    c.y = wa.y
        + app.runtime.screens[app.runtime.current_screen]
            .bar_offsets
            .0 as i32;
    c.visible = true;

    let _reserved = get_transient_for_hint(app.runtime.display, win, &mut trans);

    let state = get_atom_prop(app, win, app.atoms.net_wm_state);
    let wtype = get_atom_prop(app, win, app.atoms.net_wm_window_type);

    if state == app.atoms.net_wm_fullscreen {
        c.floating = true;
        c.fullscreen = true;
    }
    if wtype == app.atoms.net_wm_window_type_dialog {
        c.floating = true;
    }

    if !c.floating {
        c.floating = c.fixed || trans != 0;
    }

    update_normal_hints(app, &mut c);

    // Set input mask
    select_input(
        app.runtime.display,
        win,
        EnterWindowMask | FocusChangeMask | PropertyChangeMask | StructureNotifyMask,
    );
    // Set previous client border to normal
    if let Some(cw) = get_current_client_id(app) {
        set_window_border(
            app.runtime.display,
            cw,
            argb_to_int(app.config.normal_border_color),
        );
    }
    // Get current workspace
    let w = &mut app.runtime.screens[app.runtime.current_screen].workspaces
        [app.runtime.current_workspace];
    // Update client tracker
    w.current_client = Some(w.clients.len());
    app.runtime.current_client = w.current_client;
    // Push to stack
    w.clients.push(c);
    // Add window to wm _NET_CLIENT_LIST
    change_property(
        app.runtime.display,
        app.runtime.root_win,
        app.atoms.net_client_list,
        XA_WINDOW,
        32,
        PropModeAppend,
        &win as *const u64 as *mut u8,
        1,
    );

    let cur_workspace: usize = app.runtime.current_workspace + app.runtime.current_screen * 10;

    update_client_desktop(app, win, cur_workspace as u64);

    let mut wc = x11::xlib::XWindowChanges {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
        border_width: app.config.border_size as i32,
        sibling: 0,
        stack_mode: 0,
    };
    configure_window(app.runtime.display, win, CWBorderWidth as u32, &mut wc);
    set_window_border(
        app.runtime.display,
        win,
        argb_to_int(app.config.active_border_color),
    );
    update_client_name(app, win);
    raise_window(app.runtime.display, win);
    set_input_focus(app.runtime.display, win, RevertToPointerRoot, CurrentTime);

    let data: [i64; 2] = [1, 0];

    change_property(
        app.runtime.display,
        win,
        app.atoms.wm_state,
        app.atoms.wm_state,
        32,
        PropModeReplace,
        &data as *const [i64; 2] as *mut u8,
        2,
    );

    // Arrange current workspace
    arrange(app);
    // Finish mapping
    map_window(app.runtime.display, win);
    log!("   |- Mapped window");
}

fn attach_dock(app: &mut ApplicationContainer, wa: &XWindowAttributes, win: u64) {
    let dx = wa.x as i64;
    let dy = wa.y as i64;
    let dw = wa.width as usize;
    let dh = wa.height as usize;
    for screen in &mut app.runtime.screens {
        if dx >= screen.x && dx < (screen.x + screen.width) {
            if dy >= screen.y && dy < (screen.y + screen.height) {
                let mut ba = screen.bar_offsets;
                // Found corresponding screen
                if dw > dh {
                    // dock is horizontal
                    if dy == screen.y {
                        // dock is on the top
                        ba.0 = dh;
                    } else {
                        // dock is on the bottom
                        ba.2 = dh;
                    }
                } else {
                    // dock is vertical
                    if dx == screen.x {
                        // dock is on the left
                        ba.3 = dw;
                    } else {
                        // dock is on the right
                        ba.1 = dw;
                    }
                }
                screen.bar_offsets = ba;
                app.runtime.bars.push(Bar {
                    window_id: win,
                    x: dx,
                    y: dy,
                    w: dw,
                    h: dh,
                });
                arrange(app);
                break;
            }
        }
    }
}

fn detach_dock(app: &mut ApplicationContainer, win: u64) {
    log!("   |- Detaching dock");
    let b = match app.runtime.bars.iter().find(|b| b.window_id == win) {
        Some(b) => b.clone(),
        None => return,
    };
    app.runtime.bars.retain(|b| b.window_id != win);
    let dx = b.x as i64;
    let dy = b.y as i64;
    let dw = b.w as usize;
    let dh = b.h as usize;
    log!("{} {} {} {}", dx, dy, dw, dh);
    for screen in &mut app.runtime.screens {
        if dx >= screen.x && dx < (screen.x + screen.width) {
            if dy >= screen.y && dy < (screen.y + screen.height) {
                let mut ba = screen.bar_offsets;
                // Found corresponding screen
                if dw > dh {
                    // dock is horizontal
                    if dy == screen.y {
                        // dock is on the top
                        ba.0 = 0;
                    } else {
                        // dock is on the bottom
                        ba.2 = 0;
                    }
                } else {
                    // dock is vertical
                    if dx == screen.x {
                        // dock is on the left
                        ba.3 = 0;
                    } else {
                        // dock is on the right
                        ba.1 = 0;
                    }
                }
                screen.bar_offsets = ba;
                arrange(app);
                break;
            }
        }
    }
}

fn update_normal_hints(app: &mut ApplicationContainer, c: &mut Client) {
    if let Some((sh, _)) = get_wm_normal_hints(app.runtime.display, c.window_id) {
        if (sh.flags & PMaxSize) != 0 {
            c.maxw = sh.max_width;
            c.maxh = sh.max_height;
        }
        if (sh.flags & PMinSize) != 0 {
            c.minw = sh.min_width;
            c.minh = sh.min_height;
        }
    }

    if c.minw != 0 && c.w < c.minw as u32 {
        c.w = c.minw as u32;
    }
    if c.minh != 0 && c.h < c.minh as u32 {
        c.h = c.minh as u32;
    }

    if c.maxw != 0 && c.maxh != 0 && c.maxw == c.minw && c.maxh == c.minh {
        c.fixed = true;
    }
}

/// Arranges windows of current workspace in specified layout
fn arrange(app: &mut ApplicationContainer) {
    log!("   |- Arranging...");
    let ws = &mut app.runtime;
    // Go thru all screens
    for screen in &mut ws.screens {
        // Usable screen
        let ba = screen.bar_offsets;
        let screen_height = screen.height - (ba.0 + ba.2) as i64;
        // Gap width
        let gw = app.config.gap_width as i32;
        let bs = app.config.border_size as u32;
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
                client.y = ba.0 as i32;
                client.w = screen.width as u32;
                client.h = screen_height as u32;
            } else if (index as i64) < master_capacity {
                // if master_capacity is not full put it here
                log!("      |- Goes to master");
                // some math...
                let win_height =
                    (screen_height - gw as i64 - master_capacity * gw as i64) / master_capacity;
                // Add gap offset to the left
                client.x = gw;
                // Top gap + clients with their gaps offset
                client.y = ba.0 as i32 + gw + (win_height as i32 + gw) * index as i32;
                client.w = master_width - 2 * bs;
                client.h = win_height as u32 - 2 * bs
            } else {
                // otherwise put it in secondary stack
                log!("      |- Goes to stack");
                // a bit more of math...
                let win_height =
                    (screen_height - gw as i64 - (stack_size as i64 - master_capacity) * gw as i64)
                        / (stack_size as i64 - master_capacity);
                client.x = master_width as i32 + (gw * 2);
                client.y = ba.0 as i32
                    + gw
                    + (win_height as i32 + gw) * (index as i64 - master_capacity) as i32;
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
                    if stack_size > 1 || client.floating {
                        app.config.border_size as u32
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
                if client.floating {
                    raise_window(ws.display, client.window_id);
                }
            };
        }
    }
}

/// Returns window, workspace and client indexies for client with specified id
fn find_window_indexes(app: &mut ApplicationContainer, win: u64) -> Option<(usize, usize, usize)> {
    let ws = &mut app.runtime;
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
    let ws = &mut app.runtime;
    // Iterate over all clients
    for client in &mut ws.screens[ws.current_screen].workspaces[ws.current_workspace].clients {
        move_resize_window(
            ws.display,
            client.window_id,
            -(client.w as i32),
            -(client.h as i32),
            client.w,
            client.h,
        );
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
        None => app.runtime.current_screen,
    };

    let workspace = match workspace {
        Some(index) => index,
        None => app.runtime.current_workspace,
    };

    let ws = &mut app.runtime;
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
                .expect("error getting client");
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
    if let Some(ps) = get_wm_protocols(app.runtime.display, win) {
        // If protocol not supported
        if ps.into_iter().filter(|p| *p == e).collect::<Vec<_>>().len() == 0 {
            return false;
        }
    } else {
        // If failed obtaining protocols
        return false;
    }

    // proceed to send event to window
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
        configure_request: None,
        client: Some(x11::xlib::XClientMessageEvent {
            type_: ClientMessage,
            serial: 0,
            send_event: 0,
            display: null_mut(),
            window: win,
            message_type: app.atoms.wm_protocols,
            format: 32,
            data: {
                let mut d = x11::xlib::ClientMessageData::new();
                d.set_long(0, e as i64);
                d.set_long(1, CurrentTime as i64);
                d
            },
        }),
    };
    return send_event(app.runtime.display, win, false, NoEventMask, &mut ev);
}

// TODO: What is going on here
fn get_atom_prop(app: &mut ApplicationContainer, win: u64, prop: Atom) -> Atom {
    let mut dummy_atom: u64 = 0;
    let mut dummy_int: i32 = 0;
    let mut dummy_long: u64 = 0;
    let mut property_return: *mut u8 = std::ptr::null_mut::<u8>();
    let mut atom: u64 = 0;
    unsafe {
        if x11::xlib::XGetWindowProperty(
            app.runtime.display,
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

/// Removes window from runtime
fn unmanage_window(app: &mut ApplicationContainer, win: u64) {
    // Find trackers for window
    if let Some((s, w, c)) = find_window_indexes(app, win) {
        log!("   |- Found window {} at indexes {}, {}, {}", win, s, w, c);
        delete_property(app.runtime.display, win, app.atoms.net_wm_desktop);
        app.runtime.screens[s].workspaces[w].clients.remove(c);
        shift_current_client(app, Some(s), Some(w));
        arrange(app);
        update_client_list(app);
    } else {
        if app
            .runtime
            .bars
            .iter()
            .find(|b| b.window_id == win)
            .is_some()
        {
            detach_dock(app, win);
        }
    }
}

fn update_active_window(app: &mut ApplicationContainer) {
    let ws = &mut app.runtime;
    if let Some(index) = ws.current_client {
        let win =
            ws.screens[ws.current_screen].workspaces[ws.current_workspace].clients[index].window_id;
        change_property(
            ws.display,
            ws.root_win,
            app.atoms.net_active_window,
            XA_WINDOW,
            32,
            PropModeReplace,
            &win as *const u64 as *mut u8,
            1,
        );
    }
}

fn get_current_client_id(app: &mut ApplicationContainer) -> Option<u64> {
    let ws = &app.runtime;
    ws.current_client.map(|index| {
        ws.screens[ws.current_screen].workspaces[ws.current_workspace].clients[index].window_id
    })
}

fn update_trackers(app: &mut ApplicationContainer, win: u64) {
    if let Some((s, w, c)) = find_window_indexes(app, win) {
        let ws = &mut app.runtime;
        ws.current_screen = s;
        ws.current_workspace = w;
        ws.screens[s].current_workspace = w;
        ws.current_client = Some(c);
        ws.screens[s].workspaces[w].current_client = Some(c);
    };
}

fn spawn(app: &mut ApplicationContainer, cmd: String) {
    unsafe {
        match nix::unistd::fork() {
            Ok(nix::unistd::ForkResult::Parent { child: _ }) => {
                log!("     |- Spawned");
            }
            Ok(nix::unistd::ForkResult::Child) => {
                log!("     |- Hello from child)");
                if app.runtime.display as *mut x11::xlib::Display as usize != 0 {
                    nix::unistd::close(x11::xlib::XConnectionNumber(app.runtime.display)).unwrap();
                }
                let args = [
                    &std::ffi::CString::new("/usr/bin/sh").unwrap(),
                    &std::ffi::CString::new("-c").unwrap(),
                    &std::ffi::CString::new(cmd).unwrap(),
                ];
                let _ = nix::unistd::execvp(args[0], &args);
            }
            Err(_) => println!("Fork Failed"),
        }
    }
}

fn update_client_list(app: &mut ApplicationContainer) {
    let ws = &mut app.runtime;

    delete_property(ws.display, ws.root_win, app.atoms.net_client_list);

    for screen in &app.runtime.screens {
        for workspace in &screen.workspaces {
            for client in &workspace.clients {
                change_property(
                    app.runtime.display,
                    app.runtime.root_win,
                    app.atoms.net_client_list,
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

fn kill_client(app: &mut ApplicationContainer) {
    // Check if there any windows selected
    if let Some(index) = app.runtime.current_client {
        let id = app.runtime.screens[app.runtime.current_screen].workspaces
            [app.runtime.current_workspace]
            .clients[index]
            .window_id;
        log!("      |- Killing window {}", id);
        if !send_atom(app, id, app.atoms.wm_delete) {
            grab_server(app.runtime.display);
            set_close_down_mode(app.runtime.display, DestroyAll);
            x_kill_client(app.runtime.display, id);
            ungrab_server(app.runtime.display);
        };
    } else {
        log!("      |- No window selected");
    };
}

fn move_to_screen(app: &mut ApplicationContainer, d: ScreenSwitching) {
    // Check if window is selected
    if let Some(index) = app.runtime.current_client {
        // Get current screen index
        let mut cs = app.runtime.current_screen;
        // Update index depending on supplied direction
        cs = match d {
            ScreenSwitching::Next => (cs + 1) % app.runtime.screens.len(),
            ScreenSwitching::Previous => {
                (cs + app.runtime.screens.len() - 1) % app.runtime.screens.len()
            }
        };
        // Pop client
        let cc = app.runtime.screens[app.runtime.current_screen].workspaces
            [app.runtime.current_workspace]
            .clients
            .remove(index);
        set_window_border(
            app.runtime.display,
            cc.window_id,
            argb_to_int(app.config.normal_border_color),
        );

        let cur_workspace: usize = app.runtime.screens[cs].current_workspace + cs * 10;

        update_client_desktop(app, cc.window_id, cur_workspace as u64);

        // Update client tracker on current screen
        shift_current_client(app, None, None);
        // Get workspace tracker(borrow checker is really mad at me)
        let nw = app.runtime.screens[cs].current_workspace;
        // Add window to stack of another display
        app.runtime.screens[cs].workspaces[nw].clients.push(cc);
        // Arrange all monitors
        arrange(app);
    }
}

fn focus_on_screen_index(app: &mut ApplicationContainer, n: usize) {
    if let Some(cw) = get_current_client_id(app) {
        set_window_border(
            app.runtime.display,
            cw,
            argb_to_int(app.config.normal_border_color),
        );
    }
    // Change trackers
    app.runtime.current_screen = n;
    app.runtime.current_workspace =
        app.runtime.screens[app.runtime.current_screen].current_workspace;
    app.runtime.current_client = app.runtime.screens[app.runtime.current_screen].workspaces
        [app.runtime.current_workspace]
        .current_client;
    if let Some(index) = app.runtime.current_client {
        let win = app.runtime.screens[app.runtime.current_screen].workspaces
            [app.runtime.current_workspace]
            .clients[index]
            .window_id;
        set_input_focus(app.runtime.display, win, RevertToPointerRoot, CurrentTime);
        update_active_window(app);
    }
    if let Some(cw) = get_current_client_id(app) {
        set_window_border(
            app.runtime.display,
            cw,
            argb_to_int(app.config.active_border_color),
        );
    }
    let w: u64 = n as u64 * 10 + app.runtime.screens[n].current_workspace as u64;
    change_property(
        app.runtime.display,
        app.runtime.root_win,
        app.atoms.net_current_desktop,
        XA_CARDINAL,
        32,
        PropModeReplace,
        &w as *const u64 as *mut u64 as *mut u8,
        1,
    );
}

fn focus_on_screen(app: &mut ApplicationContainer, d: ScreenSwitching) {
    // Get current screen
    let mut cs = app.runtime.current_screen;
    // Update it
    cs = match d {
        ScreenSwitching::Next => (cs + 1) % app.runtime.screens.len(),
        ScreenSwitching::Previous => {
            (cs + app.runtime.screens.len() - 1) % app.runtime.screens.len()
        }
    };
    focus_on_screen_index(app, cs);
}

fn move_to_workspace(app: &mut ApplicationContainer, n: u64) {
    log!("   |- Got `MoveToWorkspace` Action ");
    // Check if moving to another workspace
    if n as usize != app.runtime.current_workspace {
        // Check if any client is selected
        if let Some(index) = app.runtime.current_client {
            // Pop current client
            let mut cc = app.runtime.screens[app.runtime.current_screen].workspaces
                [app.runtime.current_workspace]
                .clients
                .remove(index);
            set_window_border(
                app.runtime.display,
                cc.window_id,
                argb_to_int(app.config.normal_border_color),
            );
            let cur_workspace: usize = n as usize + app.runtime.current_screen * 10;

            update_client_desktop(app, cc.window_id, cur_workspace as u64);

            // Update current workspace layout
            arrange(app);
            // Move window out of view
            move_resize_window(
                app.runtime.display,
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
            app.runtime.screens[app.runtime.current_screen].workspaces[n as usize]
                .clients
                .push(cc);
        }
    }
}

fn focus_on_workspace(app: &mut ApplicationContainer, n: u64, r: bool) {
    let n = if !r {
        focus_on_screen_index(app, n as usize / 10);
        n % 10
    } else {
        n
    };
    log!("   |- Got `FocusOnWorkspace` Action");
    // Check is focusing on another workspace
    if n as usize != app.runtime.current_workspace {
        // Hide current workspace
        show_hide_workspace(app);
        // unfocus current win
        if let Some(cw) = get_current_client_id(app) {
            set_window_border(
                app.runtime.display,
                cw,
                argb_to_int(app.config.normal_border_color),
            );
        }
        // Update workspace index
        app.runtime.current_workspace = n as usize;
        app.runtime.screens[app.runtime.current_screen].current_workspace = n as usize;

        let w = n + app.runtime.current_screen as u64 * 10;

        change_property(
            app.runtime.display,
            app.runtime.root_win,
            app.atoms.net_current_desktop,
            XA_CARDINAL,
            32,
            PropModeReplace,
            &w as *const u64 as *mut u64 as *mut u8,
            1,
        );
        // Update current client
        app.runtime.current_client = app.runtime.screens[app.runtime.current_screen].workspaces
            [app.runtime.current_workspace]
            .current_client;
        if let Some(cw) = get_current_client_id(app) {
            set_window_border(
                app.runtime.display,
                cw,
                argb_to_int(app.config.active_border_color),
            );
        }
        // Show current client
        show_hide_workspace(app);
        // Arrange update workspace
        arrange(app);
        if let Some(index) = app.runtime.current_client {
            let win = app.runtime.screens[app.runtime.current_screen].workspaces
                [app.runtime.current_workspace]
                .clients[index]
                .window_id;
            set_input_focus(app.runtime.display, win, RevertToPointerRoot, CurrentTime);
            update_active_window(app);
        }
    }
}

fn update_master_width(app: &mut ApplicationContainer, w: f64) {
    // Update master width
    app.runtime.screens[app.runtime.current_screen].workspaces[app.runtime.current_workspace]
        .master_width += w;
    // Rearrange windows
    arrange(app);
}

fn update_master_capacity(app: &mut ApplicationContainer, i: i64) {
    // Change master size
    app.runtime.screens[app.runtime.current_screen].workspaces[app.runtime.current_workspace]
        .master_capacity += i;
    // Rearrange windows
    arrange(app);
}

fn toggle_float(app: &mut ApplicationContainer) {
    if let Some(c) = app.runtime.current_client {
        let state = app.runtime.screens[app.runtime.current_screen].workspaces
            [app.runtime.current_workspace]
            .clients[c]
            .floating;
        app.runtime.screens[app.runtime.current_screen].workspaces[app.runtime.current_workspace]
            .clients[c]
            .floating = !state;
        arrange(app);
    }
}

fn key_press(app: &mut ApplicationContainer, ev: Event) {
    log!("|- Got keyboard event");
    // Safely retrive struct
    let ev = ev.key.unwrap();
    // Iterate over key actions matching current key input
    for action in app.config.key_actions.clone() {
        if ev.keycode == keysym_to_keycode(app.runtime.display, action.keysym)
            && ev.state == action.modifier
        {
            // Log action type
            log!("|- Got {:?} action", &action.result);
            // Match action result and run related function
            match &action.result {
                ActionResult::KillClient => {
                    log!("   |- Got `KillClient` Action");
                    kill_client(app);
                }
                ActionResult::Spawn(cmd) => {
                    log!("   |- Got `Spawn` Action");
                    spawn(app, cmd.clone());
                }
                ActionResult::MoveToScreen(d) => {
                    move_to_screen(app, *d);
                }
                ActionResult::FocusOnScreen(d) => {
                    focus_on_screen(app, *d);
                }
                ActionResult::MoveToWorkspace(n) => {
                    move_to_workspace(app, *n);
                }
                ActionResult::FocusOnWorkspace(n) => {
                    focus_on_workspace(app, *n, true);
                }
                ActionResult::Quit => {
                    log!("   |- Got `Quit` Action. `Quiting`");
                    app.runtime.running = false;
                }
                ActionResult::UpdateMasterCapacity(i) => {
                    update_master_capacity(app, *i);
                }
                ActionResult::UpdateMasterWidth(w) => {
                    update_master_width(app, *w);
                }
                ActionResult::DumpInfo => {
                    log!("{:#?}", &app.runtime);
                }
                ActionResult::ToggleFloat => {
                    toggle_float(app);
                }
                ActionResult::CycleStack(_i) => {}
            }
        }
    }
}

fn map_request(app: &mut ApplicationContainer, ev: Event) {
    let ew: u64 = ev.map_request.unwrap().window;
    log!("|- Map Request From Window: {ew}");
    manage_client(app, ew);
}

fn enter_notify(app: &mut ApplicationContainer, ev: Event) {
    let ew: u64 = ev.crossing.unwrap().window;
    log!("|- Crossed Window `{}` ({})", get_client_name(app, ew), ew);
    if ew != app.runtime.root_win {
        log!("   |- Setting focus to window");
        // Focus on crossed window
        if let Some(cw) = get_current_client_id(app) {
            set_window_border(
                app.runtime.display,
                cw,
                argb_to_int(app.config.normal_border_color),
            );
        }
        set_window_border(
            app.runtime.display,
            ew,
            argb_to_int(app.config.active_border_color),
        );
        update_trackers(app, ew);
        update_active_window(app);
        set_input_focus(app.runtime.display, ew, RevertToPointerRoot, CurrentTime);

        let w = app.runtime.current_workspace + app.runtime.current_screen * 10;

        change_property(
            app.runtime.display,
            app.runtime.root_win,
            app.atoms.net_current_desktop,
            XA_CARDINAL,
            32,
            PropModeReplace,
            &w as *const usize as *mut usize as *mut u8,
            1,
        );
    } else {
        let ws = &mut app.runtime;

        if ws.screens[ws.current_screen].workspaces[ws.current_workspace]
            .clients
            .is_empty()
        {
            set_input_focus(ws.display, ws.root_win, RevertToPointerRoot, CurrentTime);
            delete_property(ws.display, ws.root_win, app.atoms.net_active_window);
        }
    }
}

fn destroy_notify(app: &mut ApplicationContainer, ev: Event) {
    let ew: u64 = ev.destroy_window.unwrap().window;
    log!("|- `{}` destroyed", get_client_name(app, ew));
    unmanage_window(app, ew);
}

fn unmap_notify(app: &mut ApplicationContainer, ev: Event) {
    let ew: u64 = ev.unmap.unwrap().window;
    log!("|- `{}` unmapped", get_client_name(app, ew));
    unmanage_window(app, ew);
}

fn motion_notify(app: &mut ApplicationContainer, ev: Event) {
    log!("|- `Motion` detected");
    let p = ev.motion.unwrap();
    let (x, y) = (p.x as i64, p.y as i64);
    for screen in &app.runtime.screens {
        if screen.x <= x
            && x < screen.x + screen.width
            && screen.y <= y
            && y < screen.y + screen.height
        {
            // Update trackers
            app.runtime.current_screen = screen.number as usize;
            app.runtime.current_workspace =
                app.runtime.screens[app.runtime.current_screen].current_workspace;
            app.runtime.current_client = app.runtime.screens[app.runtime.current_screen].workspaces
                [app.runtime.current_workspace]
                .current_client;
            let w = app.runtime.current_workspace + app.runtime.current_screen * 10;

            change_property(
                app.runtime.display,
                app.runtime.root_win,
                app.atoms.net_current_desktop,
                XA_CARDINAL,
                32,
                PropModeReplace,
                &w as *const usize as *mut usize as *mut u8,
                1,
            );
        }
    }
}

fn property_notify(app: &mut ApplicationContainer, ev: Event) {
    // Safely retrive event struct
    let p = ev.property.unwrap();
    // log!("|- Got `PropertyNotify`");
    let prop_name = get_atom_name(app.runtime.display, p.atom);
    // If current window is not root proceed to updating name
    if p.window != app.runtime.root_win {
        //log!(
        //    "   |- Property: `{}` changed for window `{}`({})",
        //    prop_name,
        //    get_client_name(app, p.window),
        //    p.window
        //);
        update_client_name(app, p.window);
    } else {
        //log!("   |- Property `{}` change for root window", prop_name);
    }
}

fn configure_notify(app: &mut ApplicationContainer, ev: Event) {
    let cn = ev.configure.unwrap();
    if cn.window == app.runtime.root_win {
        log!("|- Got `ConfigureNotify` for `root window` -> Changing monitor layout");
        init_screens(app);
    } else if let Some((s, w, c)) = find_window_indexes(app, cn.window) {
        let client = &app.runtime.screens[s].workspaces[w].clients[c];
        log!(
            "|- Got `ConfigureNotify` of {:?} for `{}`",
            cn.event,
            client.window_name
        );
    } else {
        log!("|- Got `ConfigureNotify` from `Unmanaged window`");
    }
}

fn client_message(app: &mut ApplicationContainer, ev: Event) {
    let c = ev.client.unwrap();
    log!("|- Got `Client Message`");
    if let Some(cc) = find_window_indexes(app, c.window) {
        let cc = &mut app.runtime.screens[cc.0].workspaces[cc.1].clients[cc.2];
        log!(
            "   |- Of type `{}` From: `{}`",
            get_atom_name(app.runtime.display, c.message_type),
            cc.window_name
        );
        if c.message_type == app.atoms.net_wm_state {
            if c.data.get_long(1) as u64 == app.atoms.net_wm_fullscreen
                || c.data.get_long(2) as u64 == app.atoms.net_wm_fullscreen
            {
                let sf = c.data.get_long(0) == 1 || c.data.get_long(0) == 2 && cc.fullscreen;
                if sf && !cc.fullscreen {
                    change_property(
                        app.runtime.display,
                        c.window,
                        app.atoms.net_wm_state,
                        XA_ATOM,
                        32,
                        PropModeReplace,
                        &mut app.atoms.net_wm_fullscreen as *mut u64 as *mut u8,
                        1,
                    );
                    cc.fullscreen = true;
                    arrange(app);
                } else if !sf && cc.fullscreen {
                    change_property(
                        app.runtime.display,
                        c.window,
                        app.atoms.net_wm_state,
                        XA_ATOM,
                        32,
                        PropModeReplace,
                        std::ptr::null_mut::<u8>(),
                        0,
                    );
                    cc.fullscreen = false;
                    arrange(app);
                }
            } else {
                log!("      |- Unsupported `state`");
            }
        }
    } else {
        log!(
            "   |- Of type `{}`",
            get_atom_name(app.runtime.display, c.message_type)
        );
        if c.message_type == app.atoms.net_current_desktop {
            focus_on_workspace(app, c.data.get_long(0) as u64, false);
        }
    }
}

fn configure_request(app: &mut ApplicationContainer, ev: Event) {
    let cr = ev.configure_request.unwrap();

    log!("|- Got `ConfigureRequest` for `{}`", cr.window);
    if let Some((s, w, c)) = find_window_indexes(app, cr.window) {
        let sw = app.runtime.screens[s].width as i32;
        let sh = app.runtime.screens[s].height as i32;
        let ba = app.runtime.screens[s].bar_offsets;
        let client = &mut app.runtime.screens[s].workspaces[w].clients[c];
        let mut resized = false;

        if client.floating {
            if (cr.value_mask & CWWidth as u64) != 0 {
                client.w = cr.width as u32;
                resized = true;
            }
            if (cr.value_mask & CWHeight as u64) != 0 {
                client.h = cr.height as u32;
                resized = true;
            }
            if (cr.value_mask & CWX as u64) != 0 {
                client.x = cr.x;
                resized = true;
            }
            if (cr.value_mask & CWY as u64) != 0 {
                client.y = cr.y;
                resized = true;
            }

            if resized {
                client.x = (sw - (client.w as i32)) / 2;
                client.y = (sh - (ba.0 as i32) - (client.h as i32)) / 2;

                move_resize_window(
                    app.runtime.display,
                    client.window_id,
                    client.x,
                    client.y,
                    client.w,
                    client.h,
                );
            }
        }
    } else {
        let mut wc = XWindowChanges {
            x: cr.x,
            y: cr.y,
            width: cr.width,
            height: cr.height,
            border_width: cr.border_width,
            sibling: cr.above,
            stack_mode: cr.detail,
        };
        configure_window(
            app.runtime.display,
            cr.window,
            cr.value_mask as u32,
            &mut wc,
        );
    }
}

fn run(app: &mut ApplicationContainer) {
    log!("|===== run =====");
    while app.runtime.running {
        let ev = next_event(app.runtime.display);
        match ev.type_ {
            x11::xlib::KeyPress => key_press(app, ev),
            x11::xlib::MapRequest => map_request(app, ev),
            x11::xlib::EnterNotify => enter_notify(app, ev),
            x11::xlib::DestroyNotify => destroy_notify(app, ev),
            x11::xlib::UnmapNotify => unmap_notify(app, ev),
            x11::xlib::MotionNotify => motion_notify(app, ev),
            x11::xlib::PropertyNotify => property_notify(app, ev),
            x11::xlib::ConfigureNotify => configure_notify(app, ev),
            x11::xlib::ClientMessage => client_message(app, ev),
            x11::xlib::ConfigureRequest => configure_request(app, ev),
            _ => {
                log!(
                    "|- Event `{}` is not currently managed",
                    EVENT_LOOKUP[ev.type_ as usize]
                );
            }
        };
    }
}

fn cleanup(_app: &mut ApplicationContainer) {}

fn no_zombies() {
    use nix::sys::signal::*;
    unsafe {
        let sa = SigAction::new(
            SigHandler::SigIgn,
            SaFlags::SA_NOCLDSTOP | SaFlags::SA_NOCLDWAIT | SaFlags::SA_RESTART,
            SigSet::empty(),
        );
        let _ = sigaction(SIGCHLD, &sa);
    }
}

fn main() {
    set_locale(LC_CTYPE, "");
    no_zombies();
    let mut app: ApplicationContainer = setup();
    spawn(
        &mut app,
        format!("{}/.rtde/autostart.sh", std::env!("HOME")),
    );
    scan(&mut app);
    run(&mut app);
    cleanup(&mut app);
}
