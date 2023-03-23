use std::ptr::null_mut;
use x11::xlib::{XButtonEvent, XEvent, XSetWindowAttributes, XWindowAttributes};

pub fn XWindowAttributes() -> XWindowAttributes {
    XWindowAttributes {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
        border_width: 0,
        depth: 0,
        visual: null_mut(),
        root: 0,
        class: 0,
        bit_gravity: 0,
        win_gravity: 0,
        backing_store: 0,
        backing_planes: 0,
        backing_pixel: 0,
        save_under: 0,
        colormap: 0,
        map_installed: 0,
        map_state: 0,
        all_event_masks: 0,
        your_event_mask: 0,
        do_not_propagate_mask: 0,
        override_redirect: 0,
        screen: null_mut(),
    }
}

pub fn XButtonEvent() -> XButtonEvent {
    XButtonEvent {
        type_: 0,
        serial: 0,
        send_event: 0,
        display: null_mut(),
        window: 0,
        root: 0,
        subwindow: 0,
        time: 0,
        x: 0,
        y: 0,
        x_root: 0,
        y_root: 0,
        state: 0,
        button: 0,
        same_screen: 0,
    }
}

pub fn XEvent() -> XEvent {
    XEvent { type_: 0 }
}

pub fn XSetWindowAttributes() -> XSetWindowAttributes {
    XSetWindowAttributes {
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
    }
}
