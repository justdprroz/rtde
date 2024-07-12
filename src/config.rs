//! Configuration file for editing user defined settings

use std::ffi::CString;

use crate::structs::{ActionResult::*, AutostartRuleCMD, DesktopsConfig, ScreenSwitching};
use crate::structs::{Color, Configuration, KeyAction};
use x11::keysym::*;
use x11::xlib::Mod4Mask as ModKey;
use x11::xlib::ShiftMask;

pub const NUMBER_OF_DESKTOPS: usize = 10;

pub fn config() -> Configuration {
    // Macro for creating array of strings used by nix's execvp function
    #[macro_export]
    macro_rules! CMD {
        ( $( $e:expr ),* ) => {
            {
                let mut temp_vec = Vec::new();
                $(
                    temp_vec.push(CString::new($e).unwrap());
                )*
                temp_vec
            }
        };
    }

    let mut c = Configuration {
        gap_width: 4,
        border_size: 2,
        normal_border_color: Color {
            //#404080
            alpha: 255,
            red: 64,
            green: 64,
            blue: 128,
        },
        active_border_color: Color {
            //#7e2487
            alpha: 255,
            red: 126,
            green: 36,
            blue: 135,
        },
        key_actions: vec![],
        autostart: vec![],
        desktops: DesktopsConfig::new(),
    };

    // Setup shortcuts "Key Actions"
    let terminal = CMD!("alacritty");
    let file_manager = CMD!("thunar");
    let app_launcher = CMD!(
        "dmenu_run",
        "-p",
        "Open app:",
        "-sb",
        "#944b9c",
        "-nb",
        "#111222",
        "-sf",
        "#ffffff",
        "-nf",
        "#9b989c",
        "-fn",
        "monospace:size=10",
        "-b"
    );
    let screenshot = CMD!("screenshot");

    c.key_actions.extend(vec![
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
            result: Spawn(CMD!("volumeup")),
        },
        KeyAction {
            modifier: 0,
            keysym: XF86XK_AudioLowerVolume,
            result: Spawn(CMD!("volumedown")),
        },
        KeyAction {
            modifier: 0,
            keysym: XF86XK_AudioMute,
            result: Spawn(CMD!("volumemute")),
        },
        KeyAction {
            modifier: 0,
            keysym: XF86XK_AudioPlay,
            result: Spawn(CMD!("playerctl play-pause")),
        },
        KeyAction {
            modifier: 0,
            keysym: XF86XK_AudioNext,
            result: Spawn(CMD!("playerctl next")),
        },
        KeyAction {
            modifier: 0,
            keysym: XF86XK_AudioPrev,
            result: Spawn(CMD!("playerctl previous")),
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
    ]);

    // Setup desktop names
    c.desktops.keysyms = [XK_1, XK_2, XK_3, XK_4, XK_5, XK_6, XK_7, XK_8, XK_9, XK_0];

    c.desktops.names =
        vec![["1", "2", "3", "4", "5", "6", "7", "8", "9", "10"].map(|s| s.to_string())];

    for (i, k) in c.desktops.keysyms.iter().enumerate() {
        c.key_actions.push(KeyAction {
            modifier: ModKey,
            keysym: *k,
            result: FocusOnWorkspace(i as u64),
        });
        c.key_actions.push(KeyAction {
            modifier: ModKey | ShiftMask,
            keysym: *k,
            result: MoveToWorkspace(i as u64),
        });
    }

    // Macro for creating autostart rules
    #[macro_export]
    macro_rules! AUTOSTART {
        ($cmd:expr) => {
            AutostartRuleCMD {
                cmd: $cmd,
                rule: None,
            }
        };
        ($cmd:expr, $s:expr, $w:expr) => {
            AutostartRuleCMD {
                cmd: $cmd,
                rule: Some(($s, $w)),
            }
        };
    }

    c.autostart = vec![
        // Positioned
        AUTOSTART!(CMD!("alacritty"), 0, 0),
        AUTOSTART!(CMD!("firefox"), 0, 1),
        AUTOSTART!(CMD!("telegram-desktop"), 0, 3),
        // Cli
        AUTOSTART!(CMD!("picom")),
        AUTOSTART!(CMD!("polybar")),
        AUTOSTART!(CMD!(
            "setxkbmap",
            "us,ru",
            "-option",
            "grp:win_space_toggle"
        )),
        AUTOSTART!(CMD!(std::env!("HOME").to_owned() + "/.fehbg")),
        AUTOSTART!(CMD!("touch", "/tmp/rtwmrunning")),
    ];

    // return local temporary config
    return c;
}
