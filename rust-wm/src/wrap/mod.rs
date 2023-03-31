pub mod xlib {
    pub fn XOpenDisplay(display_name: Option<&str>) -> Option<&mut x11::xlib::Display> {
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

    pub fn XDefaultRootWindow(display: &mut x11::xlib::Display) -> u64 {
        unsafe { x11::xlib::XDefaultRootWindow(display as *mut x11::xlib::Display) }
    }

    pub fn XChangeWindowAttributes(
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

    pub fn XSelectInput(display: &mut x11::xlib::Display, w: u64, event_mask: i64) -> i32 {
        unsafe { x11::xlib::XSelectInput(display as *mut x11::xlib::Display, w, event_mask) }
    }

    pub fn XQueryTree(display: &mut x11::xlib::Display, w: u64) -> (u64, u64, Vec<u64>) {
        unsafe {
            let mut root_return: *mut u64 = 0 as *mut u64;
            let mut parent_return: *mut u64 = 0 as *mut u64;
            let mut nchildren_return: *mut u32 = 0 as *mut u32;
            let mut children_return: *mut *mut u64 = 0 as *mut *mut u64;

            x11::xlib::XQueryTree(
                display as *mut x11::xlib::Display,
                w,
                root_return,
                parent_return,
                children_return,
                nchildren_return,
            );

            (
                0,
                0,
                std::slice::from_raw_parts_mut(*children_return, *nchildren_return as usize)
                    .iter()
                    .map(|win| *win)
                    .collect(),
            )
        }
    }

    pub fn XGetWindowAttributes(
        display: &mut x11::xlib::Display,
        w: u64,
    ) -> Option<x11::xlib::XWindowAttributes> {
        unsafe {
            let wa = 0 as *mut x11::xlib::XWindowAttributes;
            if x11::xlib::XGetWindowAttributes(display as *mut x11::xlib::Display, w, wa) != 0 {
                Some(*wa)
            } else {
                None
            }
        }
    }

    pub fn XGetTransientForHint(
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

    pub fn XNextEvent(display: &mut x11::xlib::Display) -> x11::xlib::XEvent {
        unsafe {
            let ev = 0 as *mut x11::xlib::XEvent;
            x11::xlib::XNextEvent(display as *mut x11::xlib::Display, ev);
            *ev
        }
    }

    pub fn XMoveResizeWindow(
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

    pub fn XSetWindowBorderWidth(display: &mut x11::xlib::Display, w: u64, width: u32) {
        unsafe {
            x11::xlib::XSetWindowBorderWidth(display as *mut x11::xlib::Display, w, width);
        }
    }

    pub fn XRaiseWindow(display: &mut x11::xlib::Display, w: u64) {
        unsafe {
            x11::xlib::XRaiseWindow(display as *mut x11::xlib::Display, w);
        }
    }

    pub fn XKillClient(display: &mut x11::xlib::Display, w: u64) {
        unsafe {
            x11::xlib::XKillClient(display as *mut x11::xlib::Display, w);
        }
    }

    pub fn XSetInputFocus(display: &mut x11::xlib::Display, focus: u64, revert_to: i32, time: u64) {
        unsafe {
            x11::xlib::XSetInputFocus(display as *mut x11::xlib::Display, focus, revert_to, time);
        }
    }

    pub fn XMapWindow(display: &mut x11::xlib::Display, w: u64) {
        unsafe {
            x11::xlib::XMapWindow(display as *mut x11::xlib::Display, w);
        }
    }

    pub fn XKeysymToKeycode(display: &mut x11::xlib::Display, keysym: u32) -> u32 {
        unsafe {
            x11::xlib::XKeysymToKeycode(display as *mut x11::xlib::Display, keysym as u64) as u32
        }
    }
}

pub mod xinerama {
    pub fn XineramaQueryScreens(
        display: &mut x11::xlib::Display,
    ) -> Option<Vec<x11::xinerama::XineramaScreenInfo>> {
        unsafe {
            let mut screens_amount: i32 = 0;
            match x11::xinerama::XineramaQueryScreens(
                display as *mut x11::xlib::Display,
                screens_amount as *mut i32,
            )
            .as_mut()
            {
                Some(xqs) => {
                    Some(std::slice::from_raw_parts_mut(xqs, screens_amount as usize).to_vec())
                }
                None => None,
            }
        }
    }
}
