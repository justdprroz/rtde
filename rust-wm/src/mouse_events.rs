//if ev.type_ == ButtonPress {
//    let button = ev.button.unwrap();
//    let ew = button.subwindow;
//    if button.subwindow != 0 {
//        if button.button == 2 {
//            println!("   |- Selecting Window: {ew}");
//            raise_window(app.environment.window_system.display, ew);
//            set_input_focus(
//                app.environment.window_system.display,
//                ew,
//                RevertToParent,
//                CurrentTime,
//            );
//            // add window decoration
//            // XSetWindowBorderWidth(app.environment.window_system.display, ew, 2);
//            // XSetWindowBorder(app.environment.window_system.display, ew, argb_to_int(0, 98, 114, 164));
//        } else {
//            println!("   |- Started Grabbing Window: {ew}");
//            attr = get_window_attributes(
//                app.environment.window_system.display,
//                button.subwindow,
//            )
//            .unwrap();
//            start = button;
//        }
//    }
//}
//if ev.type_ == MotionNotify {
//    let motion = ev.motion.unwrap();
//    let button = ev.button.unwrap();
//    let ew: u64 = motion.window;

//    println!("   |- Window id: {ew}");

//    if button.subwindow != 0 && start.subwindow != 0 {
//        println!("   |- Resizing OR Moving Window");
//        let x_diff: i32 = button.x_root - start.x_root;
//        let y_diff: i32 = button.y_root - start.y_root;
//        move_resize_window(
//            app.environment.window_system.display,
//            start.subwindow,
//            attr.x + {
//                if start.button == 1 {
//                    x_diff
//                } else {
//                    0
//                }
//                // Get u32 keycode from keysym
//            },
//            attr.y + {
//                if start.button == 1 {
//                    y_diff
//                } else {
//                    0
//                }
//            },
//            1.max(
//                (attr.width + {
//                    if start.button == 3 {
//                        x_diff
//                    } else {
//                        0
//                    }
//                }) as u32,
//            ),
//            1.max(
//                (attr.height + {
//                    if start.button == 3 {
//                        y_diff
//                    } else {
//                        0
//                    }
//                }) as u32,
//            ),
//        );
//    } else {
//        println!("   |- Just Moving");
//        // XSetInputFocus(app.environment.window_system.display, win, RevertToNone, CurrentTime);
//    }
//}
//if ev.type_ == ButtonRelease {
//    start.subwindow = 0;
//}
