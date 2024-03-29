pub mod xlib {
    use libc::c_void;
    use x11::xlib::{Atom, Success, XGetWindowProperty};

    use crate::get_default::{xevent, xwindow_attributes};

    unsafe extern "C" fn handler_func(
        _d: *mut x11::xlib::Display,
        _e: *mut x11::xlib::XErrorEvent,
    ) -> i32 {
        0
    }

    pub fn set_error_handler() {
        unsafe {
            x11::xlib::XSetErrorHandler(Some(handler_func));
        }
    }

    pub fn set_locale(c: i32, l: &str) {
        unsafe {
            let locale = std::ffi::CString::new(l).unwrap();
            libc::setlocale(c, locale.as_ptr());
        }
    }

    pub fn open_display(display_name: Option<&str>) -> Option<&mut x11::xlib::Display> {
        unsafe {
            let result = match display_name {
                Some(dn) => {
                    let name_ptr = dn.as_ptr() as *const i8;
                    x11::xlib::XOpenDisplay(name_ptr)
                }
                None => x11::xlib::XOpenDisplay(std::ptr::null::<i8>()),
            };
            return result.as_mut();
        }
    }

    pub fn default_root_window(display: &mut x11::xlib::Display) -> u64 {
        unsafe { x11::xlib::XDefaultRootWindow(display as *mut x11::xlib::Display) }
    }

    pub fn set_window_border(display: &mut x11::xlib::Display, w: u64, border_pixel: u64) {
        unsafe {
            x11::xlib::XSetWindowBorder(display as *mut x11::xlib::Display, w, border_pixel);
        }
    }

    pub fn delete_property(display: &mut x11::xlib::Display, w: u64, property: x11::xlib::Atom) {
        unsafe {
            x11::xlib::XDeleteProperty(display as *mut x11::xlib::Display, w, property);
        }
    }

    pub fn default_screen(display: &mut x11::xlib::Display) -> i32 {
        unsafe { x11::xlib::XDefaultScreen(display as *mut x11::xlib::Display) }
    }

    pub fn change_window_attributes(
        display: &mut x11::xlib::Display,
        w: u64,
        valuemask: u64,
        attributes: &mut x11::xlib::XSetWindowAttributes,
    ) -> i32 {
        unsafe {
            x11::xlib::XChangeWindowAttributes(
                display as *mut x11::xlib::Display,
                w,
                valuemask,
                attributes as *mut x11::xlib::XSetWindowAttributes,
            )
        }
    }

    pub fn select_input(display: &mut x11::xlib::Display, w: u64, event_mask: i64) -> i32 {
        unsafe { x11::xlib::XSelectInput(display as *mut x11::xlib::Display, w, event_mask) }
    }

    pub fn configure_window(
        display: &mut x11::xlib::Display,
        w: u64,
        valuemask: u32,
        values: &mut x11::xlib::XWindowChanges,
    ) {
        unsafe {
            x11::xlib::XConfigureWindow(
                display as *mut x11::xlib::Display,
                w,
                valuemask,
                values as *mut x11::xlib::XWindowChanges,
            );
        }
    }

    pub fn create_simple_window(
        display: &mut x11::xlib::Display,
        parent: u64,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        border_width: u32,
        border: u64,
        background: u64,
    ) -> u64 {
        unsafe {
            x11::xlib::XCreateSimpleWindow(
                display as *mut x11::xlib::Display,
                parent,
                x,
                y,
                width,
                height,
                border_width,
                border,
                background,
            )
        }
    }

    pub fn set_class_hints(
        display: &mut x11::xlib::Display,
        w: u64,
        class_hints: &mut x11::xlib::XClassHint,
    ) {
        unsafe {
            x11::xlib::XSetClassHint(
                display as *mut x11::xlib::Display,
                w,
                class_hints as *mut x11::xlib::XClassHint,
            );
        }
    }

    pub fn create_window(
        display: &mut x11::xlib::Display,
        parent: u64,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        border_width: u32,
        depth: i32,
        class: u32,
        visual: &mut x11::xlib::Visual,
        valuemask: u64,
        attributes: &mut x11::xlib::XSetWindowAttributes,
    ) -> u64 {
        unsafe {
            x11::xlib::XCreateWindow(
                display as *mut x11::xlib::Display,
                parent,
                x,
                y,
                width,
                height,
                border_width,
                depth,
                class,
                visual as *mut x11::xlib::Visual,
                valuemask,
                attributes as *mut x11::xlib::XSetWindowAttributes,
            )
        }
    }

    pub fn default_depth(display: &mut x11::xlib::Display, number: i32) -> i32 {
        unsafe { x11::xlib::XDefaultDepth(display as *mut x11::xlib::Display, number) }
    }

    pub fn default_visual(display: &mut x11::xlib::Display, number: i32) -> x11::xlib::Visual {
        unsafe { *x11::xlib::XDefaultVisual(display as *mut x11::xlib::Display, number) }
    }

    pub fn query_tree(display: &mut x11::xlib::Display, w: u64) -> (u64, u64, Vec<u64>) {
        unsafe {
            let mut root_return: u64 = 0;
            let mut parent_return: u64 = 0;
            let mut nchildren_return: u32 = 0;
            let mut children_return: *mut u64 = std::ptr::null_mut::<u64>();

            x11::xlib::XQueryTree(
                display as *mut x11::xlib::Display,
                w,
                &mut root_return as *mut u64,
                &mut parent_return as *mut u64,
                &mut children_return as *mut *mut u64,
                &mut nchildren_return as *mut u32,
            );

            (
                0,
                0,
                std::slice::from_raw_parts_mut(children_return, nchildren_return as usize).to_vec(),
            )
        }
    }

    pub fn grab_server(display: &mut x11::xlib::Display) {
        unsafe {
            x11::xlib::XGrabServer(display as *mut x11::xlib::Display);
        }
    }

    pub fn ungrab_server(display: &mut x11::xlib::Display) {
        unsafe {
            x11::xlib::XUngrabServer(display as *mut x11::xlib::Display);
        }
    }

    pub fn set_close_down_mode(display: &mut x11::xlib::Display, mode: i32) {
        unsafe {
            x11::xlib::XSetCloseDownMode(display as *mut x11::xlib::Display, mode);
        }
    }

    pub fn get_window_attributes(
        display: &mut x11::xlib::Display,
        w: u64,
    ) -> Option<x11::xlib::XWindowAttributes> {
        unsafe {
            let mut wa: x11::xlib::XWindowAttributes = xwindow_attributes();
            if x11::xlib::XGetWindowAttributes(
                display as *mut x11::xlib::Display,
                w,
                &mut wa as *mut x11::xlib::XWindowAttributes,
            ) != 0
            {
                Some(wa)
            } else {
                None
            }
        }
    }

    pub fn get_wm_protocols(
        display: &mut x11::xlib::Display,
        w: u64,
    ) -> Option<Vec<x11::xlib::Atom>> {
        unsafe {
            let mut protocols_return: *mut x11::xlib::Atom = std::ptr::null_mut::<u64>();
            let mut count_return: i32 = 0;
            if x11::xlib::XGetWMProtocols(
                display as *mut x11::xlib::Display,
                w,
                &mut protocols_return as *mut *mut x11::xlib::Atom,
                &mut count_return as *mut i32,
            ) != 0
            {
                Some(std::slice::from_raw_parts(protocols_return, count_return as usize).to_vec())
            } else {
                None
            }
        }
    }

    pub fn intern_atom(
        display: &mut x11::xlib::Display,
        atom_name: String,
        oie: bool,
    ) -> x11::xlib::Atom {
        unsafe {
            let name_ptr = std::ffi::CString::new(atom_name).unwrap();
            x11::xlib::XInternAtom(
                display as *mut x11::xlib::Display,
                name_ptr.as_ptr() as *const i8,
                oie as i32,
            )
        }
    }

    pub fn send_event(
        display: &mut x11::xlib::Display,
        w: u64,
        p: bool,
        event_mask: i64,
        event: &mut Event,
    ) -> bool {
        unsafe {
            let mut xe = xevent();
            xe.type_ = event.type_;
            if let Some(e) = event.client {
                xe.client_message = e;
            }
            if let Some(e) = event.key {
                xe.key = e;
            }
            if let Some(e) = event.unmap {
                xe.unmap = e;
            }
            if let Some(e) = event.button {
                xe.button = e;
            }
            if let Some(e) = event.motion {
                xe.motion = e;
            }
            if let Some(e) = event.crossing {
                xe.crossing = e;
            }
            if let Some(e) = event.map_request {
                xe.map_request = e;
            }
            if let Some(e) = event.destroy_window {
                xe.destroy_window = e;
            }
            x11::xlib::XSendEvent(
                display as *mut x11::xlib::Display,
                w,
                p as i32,
                event_mask,
                &mut xe as *mut x11::xlib::XEvent,
            ) != 0
        }
    }

    pub fn get_transient_for_hint(
        display: &mut x11::xlib::Display,
        w: u64,
        prop_window_return: &mut u64,
    ) -> i32 {
        unsafe {
            x11::xlib::XGetTransientForHint(
                display as *mut x11::xlib::Display,
                w,
                prop_window_return as *mut u64,
            )
        }
    }

    pub fn next_event(display: &mut x11::xlib::Display) -> Event {
        unsafe {
            let mut ev: x11::xlib::XEvent = xevent();
            x11::xlib::XNextEvent(
                display as *mut x11::xlib::Display,
                &mut ev as *mut x11::xlib::XEvent,
            );
            let mut event: Event = Event {
                type_: ev.type_,
                ..Default::default()
            };
            match ev.type_ {
                x11::xlib::KeyPress | x11::xlib::KeyRelease => {
                    event.key = Some(ev.key);
                }
                x11::xlib::ButtonPress | x11::xlib::ButtonRelease | x11::xlib::MotionNotify => {
                    event.button = Some(ev.button);
                    event.motion = Some(ev.motion);
                }
                x11::xlib::MapRequest => {
                    event.map_request = Some(ev.map_request);
                }
                x11::xlib::EnterNotify | x11::xlib::LeaveNotify => {
                    event.crossing = Some(ev.crossing);
                }
                x11::xlib::DestroyNotify => {
                    event.destroy_window = Some(ev.destroy_window);
                }
                x11::xlib::UnmapNotify => {
                    event.unmap = Some(ev.unmap);
                }
                x11::xlib::PropertyNotify => {
                    event.property = Some(ev.property);
                }
                x11::xlib::ConfigureNotify => {
                    event.configure = Some(ev.configure);
                }
                x11::xlib::ClientMessage => {
                    event.client = Some(ev.client_message);
                }
                _ => {}
            };
            event
        }
    }

    pub fn get_wm_normal_hints(
        display: &mut x11::xlib::Display,
        w: u64,
    ) -> Option<(x11::xlib::XSizeHints, i64)> {
        unsafe {
            let mut supplied_return: i64 = 0;
            let mut hints_return: x11::xlib::XSizeHints =
                std::mem::MaybeUninit::zeroed().assume_init();
            if x11::xlib::XGetWMNormalHints(
                display as *mut x11::xlib::Display,
                w,
                &mut hints_return as *mut x11::xlib::XSizeHints,
                &mut supplied_return as *mut i64,
            ) != 0
            {
                Some((hints_return, supplied_return))
            } else {
                None
            }
        }
    }

    pub fn move_resize_window(
        display: &mut x11::xlib::Display,
        w: u64,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) {
        unsafe {
            x11::xlib::XMoveResizeWindow(
                display as *mut x11::xlib::Display,
                w,
                x,
                y,
                width,
                height,
            );
        }
    }

    #[allow(dead_code)]
    pub fn set_window_border_width(display: &mut x11::xlib::Display, w: u64, width: u32) {
        unsafe {
            x11::xlib::XSetWindowBorderWidth(display as *mut x11::xlib::Display, w, width);
        }
    }

    pub fn raise_window(display: &mut x11::xlib::Display, w: u64) {
        unsafe {
            x11::xlib::XRaiseWindow(display as *mut x11::xlib::Display, w);
        }
    }

    #[allow(dead_code)]
    pub fn x_kill_client(display: &mut x11::xlib::Display, w: u64) {
        unsafe {
            x11::xlib::XKillClient(display as *mut x11::xlib::Display, w);
        }
    }

    pub fn set_input_focus(
        display: &mut x11::xlib::Display,
        focus: u64,
        revert_to: i32,
        time: u64,
    ) {
        unsafe {
            x11::xlib::XSetInputFocus(display as *mut x11::xlib::Display, focus, revert_to, time);
        }
    }

    pub fn map_window(display: &mut x11::xlib::Display, w: u64) {
        unsafe {
            x11::xlib::XMapWindow(display as *mut x11::xlib::Display, w);
        }
    }

    pub fn keysym_to_keycode(display: &mut x11::xlib::Display, keysym: u32) -> u32 {
        unsafe {
            x11::xlib::XKeysymToKeycode(display as *mut x11::xlib::Display, keysym as u64) as u32
        }
    }

    #[derive(Default)]
    pub struct Event {
        pub type_: i32,
        pub button: Option<x11::xlib::XButtonEvent>,
        pub crossing: Option<x11::xlib::XCrossingEvent>,
        pub key: Option<x11::xlib::XKeyEvent>,
        pub map_request: Option<x11::xlib::XMapRequestEvent>,
        pub destroy_window: Option<x11::xlib::XDestroyWindowEvent>,
        pub motion: Option<x11::xlib::XMotionEvent>,
        pub unmap: Option<x11::xlib::XUnmapEvent>,
        pub client: Option<x11::xlib::XClientMessageEvent>,
        pub property: Option<x11::xlib::XPropertyEvent>,
        pub configure: Option<x11::xlib::XConfigureEvent>,
    }

    pub fn change_property(
        display: &mut x11::xlib::Display,
        w: u64,
        property: x11::xlib::Atom,
        type_: x11::xlib::Atom,
        format: i32,
        mode: i32,
        data: *mut u8,
        nelements: i32,
    ) {
        unsafe {
            x11::xlib::XChangeProperty(
                display as *mut x11::xlib::Display,
                w,
                property,
                type_,
                format,
                mode,
                data,
                nelements,
            );
        }
    }

    pub fn get_text_property(
        display: &mut x11::xlib::Display,
        w: u64,
        a: x11::xlib::Atom,
    ) -> Option<String> {
        unsafe {
            let mut tr: x11::xlib::XTextProperty = x11::xlib::XTextProperty {
                value: std::ptr::null_mut::<u8>(),
                encoding: 0,
                format: 0,
                nitems: 0,
            };
            let mut strings_return = std::ptr::null_mut::<*mut i8>();
            let mut amount = 0;

            if x11::xlib::XGetTextProperty(
                display as *mut x11::xlib::Display,
                w,
                &mut tr as *mut x11::xlib::XTextProperty,
                a,
            ) == 0
                || tr.nitems == 0
            {
                return None;
            }

            let mut name: Option<String> = None;

            if tr.encoding == x11::xlib::XA_STRING {
                name = Some(
                    match std::ffi::CStr::from_ptr(tr.value as *const i8).to_string_lossy() {
                        std::borrow::Cow::Borrowed(s) => s.to_string(),
                        std::borrow::Cow::Owned(s) => s,
                    },
                );
            } else if x11::xlib::XmbTextPropertyToTextList(
                display as *mut x11::xlib::Display,
                &mut tr as *mut x11::xlib::XTextProperty,
                &mut strings_return as *mut *mut *mut i8,
                &mut amount as *mut i32,
            ) >= x11::xlib::Success as i32
                && amount > 0
                && !(*strings_return).is_null()
            {
                name = Some(
                    match std::ffi::CStr::from_ptr(*strings_return).to_string_lossy() {
                        std::borrow::Cow::Borrowed(s) => s.to_string(),
                        std::borrow::Cow::Owned(s) => s,
                    },
                );

                x11::xlib::XFreeStringList(strings_return);
            }

            x11::xlib::XFree(tr.value as *mut libc::c_void);
            name
        }
    }
}

pub mod xinerama {
    pub fn xinerama_query_screens(
        display: &mut x11::xlib::Display,
    ) -> Option<Vec<x11::xinerama::XineramaScreenInfo>> {
        unsafe {
            let mut screens_amount: i32 = 0;
            x11::xinerama::XineramaQueryScreens(
                display as *mut x11::xlib::Display,
                &mut screens_amount as *mut i32,
            )
            .as_mut()
            .map(|xqs| std::slice::from_raw_parts_mut(xqs, screens_amount as usize).to_vec())
        }
    }
}
