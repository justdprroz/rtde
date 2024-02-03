use std::fs;

use x11::keysym::*;
use x11::xlib::Mod1Mask as ModKey;
use x11::xlib::ShiftMask;

use crate::structs::ActionResult;
use crate::structs::ScreenSwitching;
use crate::structs::{Color, ConfigurationContainer, KeyAction};

pub fn read_config(path: String) -> ConfigurationContainer {
    match fs::read_to_string(path) {
        Ok(_) | Err(_) => ConfigurationContainer {
            visuals: crate::structs::Visuals {
                gap_width: 0,
                border_size: 1,
                normal_border_color: Color {
                    alpha: 0,
                    red: 0,
                    green: 0,
                    blue: 0,
                },
                active_border_color: Color {
                    alpha: 0,
                    red: 0,
                    green: 0,
                    blue: 0,
                },
            },
            key_actions: {
                let mut a = vec![
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
            },
            bar: None,
        },
    }
}
