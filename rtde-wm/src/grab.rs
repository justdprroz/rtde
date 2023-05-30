use x11::xlib::*;

pub fn grab_key(dpy: *mut Display, keysym: u32, mask: u32) {
    unsafe {
        XGrabKey(
            dpy,
            XKeysymToKeycode(dpy, keysym as u64) as i32,
            mask,
            XDefaultRootWindow(dpy),
            1,
            GrabModeAsync,
            GrabModeAsync,
        );
    }
}

pub fn _grab_button(dpy: *mut Display, button: u32, mask: u32) {
    unsafe {
        XGrabButton(
            dpy,
            button,
            mask,
            XDefaultRootWindow(dpy),
            1,
            (ButtonPressMask | ButtonReleaseMask | PointerMotionMask) as u32,
            GrabModeAsync,
            GrabModeAsync,
            0,
            0,
        );
    }
}
