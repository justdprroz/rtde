pub mod xlib {
    use crate::utils::get_default::{XEvent, XWindowAttributes};

    pub fn OpenDisplay(display_name: Option<&str>) -> Option<&mut x11::xlib::Display> {
        unsafe {
            let result = match display_name {
                Some(dn) => {
                    let name_ptr = dn.as_ptr() as *const i8;
                    x11::xlib::XOpenDisplay(name_ptr)
                }
                None => x11::xlib::XOpenDisplay(0x0 as *const i8),
            };
            return result.as_mut();
        }
    }

    pub fn DefaultRootWindow(display: &mut x11::xlib::Display) -> u64 {
        unsafe { x11::xlib::XDefaultRootWindow(display as *mut x11::xlib::Display) }
    }

    pub fn ChangeWindowAttributes(
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

    pub fn SelectInput(display: &mut x11::xlib::Display, w: u64, event_mask: i64) -> i32 {
        unsafe { x11::xlib::XSelectInput(display as *mut x11::xlib::Display, w, event_mask) }
    }

    pub fn QueryTree(display: &mut x11::xlib::Display, w: u64) -> (u64, u64, Vec<u64>) {
        unsafe {
            let mut root_return: u64 = 0;
            let mut parent_return: u64 = 0;
            let mut nchildren_return: u32 = 0;
            let mut children_return: *mut u64 = 0 as *mut u64;

            println!("==== Created vars");

            x11::xlib::XQueryTree(
                display as *mut x11::xlib::Display,
                w,
                &mut root_return as *mut u64,
                &mut parent_return as *mut u64,
                &mut children_return as *mut *mut u64,
                &mut nchildren_return as *mut u32,
            );

            println!("==== Got tree");

            (
                0,
                0,
                std::slice::from_raw_parts_mut(children_return, nchildren_return as usize)
                    .iter()
                    .map(|win| *win)
                    .collect(),
            )
        }
    }

    pub fn GetWindowAttributes(
        display: &mut x11::xlib::Display,
        w: u64,
    ) -> Option<x11::xlib::XWindowAttributes> {
        unsafe {
            let mut wa: x11::xlib::XWindowAttributes = XWindowAttributes();
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

    pub fn GetTransientForHint(
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

    pub fn NextEvent(display: &mut x11::xlib::Display) -> Event {
        unsafe {
            let mut ev: x11::xlib::XEvent = XEvent();
            x11::xlib::XNextEvent(
                display as *mut x11::xlib::Display,
                &mut ev as *mut x11::xlib::XEvent,
            );
            let mut event = Event::default();
            event.type_ = ev.type_;
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
                _ => {}
            };
            event
        }
    }

    pub fn MoveResizeWindow(
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

    pub fn SetWindowBorderWidth(display: &mut x11::xlib::Display, w: u64, width: u32) {
        unsafe {
            x11::xlib::XSetWindowBorderWidth(display as *mut x11::xlib::Display, w, width);
        }
    }

    pub fn RaiseWindow(display: &mut x11::xlib::Display, w: u64) {
        unsafe {
            x11::xlib::XRaiseWindow(display as *mut x11::xlib::Display, w);
        }
    }

    pub fn KillClient(display: &mut x11::xlib::Display, w: u64) {
        unsafe {
            x11::xlib::XKillClient(display as *mut x11::xlib::Display, w);
        }
    }

    pub fn SetInputFocus(display: &mut x11::xlib::Display, focus: u64, revert_to: i32, time: u64) {
        unsafe {
            x11::xlib::XSetInputFocus(display as *mut x11::xlib::Display, focus, revert_to, time);
        }
    }

    pub fn MapWindow(display: &mut x11::xlib::Display, w: u64) {
        unsafe {
            x11::xlib::XMapWindow(display as *mut x11::xlib::Display, w);
        }
    }

    pub fn KeysymToKeycode(display: &mut x11::xlib::Display, keysym: u32) -> u32 {
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
    }
}

pub mod xinerama {
    pub fn XineramaQueryScreens(
        display: &mut x11::xlib::Display,
    ) -> Option<Vec<x11::xinerama::XineramaScreenInfo>> {
        unsafe {
            let mut screens_amount: i32 = 0;
            println!("====created var");
            match x11::xinerama::XineramaQueryScreens(
                display as *mut x11::xlib::Display,
                &mut screens_amount as *mut i32,
            )
            .as_mut()
            {
                Some(xqs) => {
                    println!("==== got something");
                    Some(std::slice::from_raw_parts_mut(xqs, screens_amount as usize).to_vec())
                }
                None => None,
            }
        }
    }
}
