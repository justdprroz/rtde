# RTDE - acronym for Rusty Tiling Desktop Environment

## Requirement
- ```dmenu``` as application laucnher

## Features
- Support for workspaces and screens
- Stack settings for each workspace
- Tiling window layout

## Installation
1. Install Rust https://rustup.rs/
2. Install Xlib using your package manager
3. Run ```start.sh```. Window manager executable will be installed in ```~/.cargo/bin```
4. Add ```exec rust-wm``` to your ```~/.xinitrc```
5. Further configuration is up to you!
6. Use ```src/config.rs``` for configuring WM 

## Shortcuts
```ModKey = Mod1Key = Alt```
- ```Modkey + 1..0``` - Switch to workspace (0 is 10th workspace)
- ```Modkey + Shift + 1..0``` - Move current window to workspace (0 is 10th workspace)
- ```Modkey + ,``` - Switch to previous screen
- ```Modkey + .``` - Switch to previous screen
- ```Modkey + Shift + ,``` - Move current window to previous screen
- ```Modkey + Shift + .``` - Move current window to previous screen
- ```Modkey + w``` - Dump all info into stdin
- ```Modkey + i``` - Increment amount of windows in main stack
- ```Modkey + d``` - Decrement amount of windows in main stack
- ```Modkey + h``` - Decrease main stack width
- ```Modkey + l``` - Incease main stack width
- ```Modkey + Space``` - Toggle float state
- ```ModKey + Enter``` - Spawn terminal ```kitty```
- ```ModKey + Shift + Q``` - Exit window manager
- ```ModKey + p``` - Spawn application launcher ```dmenu```
- ```ModKey + Shift + C``` - Kill current window
