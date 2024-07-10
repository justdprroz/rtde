use crate::config::*;
use crate::logic::*;
use crate::structs::*;
use crate::utils::*;
use crate::wrap::xinerama::*;
use crate::wrap::xlib::*;
use std::process::exit;
use std::vec;
use x11::xlib::ButtonPressMask;
use x11::xlib::CWCursor;
use x11::xlib::CWEventMask;
use x11::xlib::EnterWindowMask;
use x11::xlib::LeaveWindowMask;
use x11::xlib::PointerMotionMask;
use x11::xlib::PropModeReplace;
use x11::xlib::PropertyChangeMask;
use x11::xlib::StructureNotifyMask;
use x11::xlib::SubstructureNotifyMask;
use x11::xlib::SubstructureRedirectMask;
use x11::xlib::XSetWindowAttributes;
use x11::xlib::XA_CARDINAL;
use x11::xlib::XA_WINDOW;

pub fn setup() -> Application {
    log!("|===== setup =====");
    let display = match open_display(None) {
        Some(d) => d,
        None => {
            eprintln!("Failed to open display");
            exit(1);
        }
    };

    let mut app = Application {
        config: config(),
        core: WmCore {
            display,
            root_win: 0,
            wm_check_win: 0,
            running: true,
        },
        runtime: Runtime {
            mouse_state: MouseState {
                win: 0,
                button: 0,
                pos: (0, 0),
            },
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

    app.core.root_win = default_root_window(app.core.display);

    init_supported_atoms(&mut app);
    init_wm_check(&mut app);
    init_screens(&mut app);
    init_desktops(&mut app);
    init_actions(&mut app);
    arrange(&mut app);
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
        app.core.display,
        app.core.root_win,
        CWEventMask | CWCursor,
        &mut wa,
    );

    select_input(app.core.display, app.core.root_win, wa.event_mask);

    app
}

pub fn init_wm_check(app: &mut Application) {
    app.core.wm_check_win =
        create_simple_window(app.core.display, app.core.root_win, 0, 0, 1, 1, 0, 0, 0);
    let mut wmchckwin = app.core.wm_check_win;

    change_property(
        app.core.display,
        wmchckwin,
        app.atoms.net_wm_check,
        XA_WINDOW,
        32,
        PropModeReplace,
        &mut wmchckwin as *mut u64 as *mut u8,
        1,
    );

    let wm_name = match std::ffi::CString::new("rtwm".to_string()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error creating name: {}. Exiting", e);
            exit(1);
        }
    };
    change_property(
        app.core.display,
        wmchckwin,
        app.atoms.net_wm_name,
        app.atoms.utf8string,
        8,
        PropModeReplace,
        wm_name.as_ptr() as *mut u8,
        7,
    );

    change_property(
        app.core.display,
        app.core.root_win,
        app.atoms.net_wm_check,
        XA_WINDOW,
        32,
        PropModeReplace,
        &mut wmchckwin as *mut u64 as *mut u8,
        1,
    );
}

pub fn init_actions(app: &mut Application) {
    for action in app.config.key_actions.iter() {
        grab_key(app.core.display, action.keysym, action.modifier);
    }
}

pub fn init_supported_atoms(app: &mut Application) {
    // let dpy = &mut app.core.display;
    macro_rules! intern_atom {
        ($e:expr) => {
            intern_atom(app.core.display, $e.to_string(), false)
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
        app.core.display,
        app.core.root_win,
        app.atoms.net_supported,
        x11::xlib::XA_ATOM,
        32,
        x11::xlib::PropModeReplace,
        netatoms.as_mut_ptr() as *mut u8,
        netatoms.len() as i32,
    );
}

// Improve this code
pub fn init_screens(app: &mut Application) {
    let n = app.runtime.screens.len();
    let screens = match xinerama_query_screens(app.core.display) {
        Some(s) => s,
        None => {
            eprintln!("Running without xinerama is not supported");
            exit(1);
        }
    };
    let screens_amount = screens.len();

    for _ in n..screens_amount {
        app.runtime.screens.push(Screen {
            number: 0,
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            workspaces: vec![],
            current_workspace: 0,
            bar_offsets: BarOffsets::default(),
        })
    }

    for (index, screen) in screens.iter().enumerate() {
        app.runtime.screens[index].number = screen.screen_number as i64;
        app.runtime.screens[index].x = screen.x_org as i64;
        app.runtime.screens[index].y = screen.y_org as i64;
        app.runtime.screens[index].width = screen.width as i64;
        app.runtime.screens[index].height = screen.height as i64;
    }

    for _ in screens_amount..n {
        if let Some(removed_screen) = app.runtime.screens.pop() {
            let removed_workspaces = removed_screen.workspaces;
            for (index, workspace) in removed_workspaces.into_iter().enumerate() {
                for client in workspace.clients {
                    update_client_desktop(app, client.window_id, index as u64);
                    app.runtime.screens[0].workspaces[index]
                        .clients
                        .push(client);
                }
            }
        }
    }
}

pub fn init_desktops(app: &mut Application) {
    // let mut desktop_names_type = 0; // 0 - number, 1 - short name, 2 - number + short name

    let mut desktop_names_ewmh: Vec<String> = vec![];
    let mut viewports: Vec<i64> = vec![];

    // Iterate over all screens
    for (index, screen) in app.runtime.screens.iter_mut().enumerate() {
        if screen.workspaces.is_empty() {
            for i in 0..NUMBER_OF_DESKTOPS {
                screen.workspaces.push(Workspace {
                    number: i as u64,
                    clients: Vec::new(),
                    current_client: None,
                    master_capacity: 1,
                    master_width: 0.5,
                });
            }
        }
        for i in 0..screen.workspaces.len() {
            if index < app.config.desktops.names.len() {
                desktop_names_ewmh.push(format!("{}", app.config.desktops.names[index][i]));
            } else {
                desktop_names_ewmh.push(format!("{}", i + 1));
            }
            viewports.push(screen.x as i64);
            viewports.push(screen.y as i64);
        }
    }
    init_desktop_ewmh_info(app, desktop_names_ewmh, viewports);
}

pub fn init_desktop_ewmh_info(app: &mut Application, names: Vec<String>, mut viewports: Vec<i64>) {
    // Update EWMH desktop properties

    // Set amount of workspaces
    change_property(
        app.core.display,
        app.core.root_win,
        app.atoms.net_number_of_desktops,
        XA_CARDINAL,
        32,
        PropModeReplace,
        &mut names.len() as *mut usize as *mut u8,
        1,
    );

    // Set workspaces names
    let mut bytes = vec_string_to_bytes(names);
    change_property(
        app.core.display,
        app.core.root_win,
        app.atoms.net_desktop_names,
        app.atoms.utf8string,
        8,
        PropModeReplace,
        bytes.as_mut_ptr(),
        bytes.len() as i32,
    );

    // Set workspaces viewports
    change_property(
        app.core.display,
        app.core.root_win,
        app.atoms.net_desktop_viewport,
        XA_CARDINAL,
        32,
        PropModeReplace,
        viewports.as_mut_ptr() as *mut u8,
        viewports.len() as i32,
    );
}
