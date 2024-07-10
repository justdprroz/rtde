//! A window manager written in Rust with nearly same functionality of [dwm](https://dwm.suckless.org/)
//!
//! List of features supported by rwm:
//! - Multi monitor setup
//! - Workspaces aka tags
//! - Stack layout
//! - Shortcuts

pub mod config;
pub mod events;
pub mod logic;
pub mod mouse;
pub mod setup;
pub mod structs;
pub mod utils;
pub mod wrap;

use crate::events::*;
use crate::logic::*;
use crate::setup::*;
use crate::structs::*;
use crate::utils::*;
use crate::wrap::xlib::*;

use libc::LC_CTYPE;

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

fn run(app: &mut Application) {
    log!("|===== run =====");
    while app.core.running {
        let ev = next_event(app.core.display);
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
            x11::xlib::ButtonPress => button_press(app, ev),
            x11::xlib::ButtonRelease => button_release(app, ev),
            _ => {
                log!(
                    "|- Event `{}` is not currently managed",
                    EVENT_LOOKUP[ev.type_ as usize]
                );
            }
        };
    }
}

fn main() {
    set_locale(LC_CTYPE, "");
    no_zombies();
    let mut app: Application = setup();
    spawn(
        &mut app,
        format!("{}/.rtde/autostart.sh", std::env!("HOME")),
    );
    scan(&mut app);
    run(&mut app);
}
