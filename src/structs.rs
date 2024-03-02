pub struct ApplicationContainer {
    pub config: ConfigurationContainer,
    pub runtime: RuntimeContainer,
    pub atoms: Atoms,
}

pub struct ConfigurationContainer {
    pub key_actions: Vec<KeyAction>,
    pub gap_width: usize,
    pub border_size: usize,
    pub normal_border_color: Color,
    pub active_border_color: Color,
}

#[derive(Clone, Copy)]
pub struct Color {
    pub alpha: u8,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[derive(Clone)]
pub struct KeyAction {
    pub keysym: u32,
    pub modifier: u32,
    pub result: ActionResult,
}

#[derive(Debug, Clone)]
pub enum ActionResult {
    KillClient,
    Spawn(String),
    MoveToScreen(ScreenSwitching),
    FocusOnScreen(ScreenSwitching),
    UpdateMasterCapacity(i64),
    UpdateMasterWidth(f64),
    MoveToWorkspace(u64),
    FocusOnWorkspace(u64),
    CycleStack(i64),
    ToggleFloat,
    DumpInfo,
    Quit,
}

#[derive(Debug, Clone, Copy)]
pub enum ScreenSwitching {
    Next,
    Previous,
}

#[derive(Debug)]
pub struct Atoms {
    pub utf8string: u64,
    pub wm_protocols: u64,
    pub wm_delete: u64,
    pub wm_state: u64,
    pub wm_take_focus: u64,
    pub wm_name: u64,
    pub net_active_window: u64,
    pub net_supported: u64,
    pub net_wm_name: u64,
    pub net_wm_state: u64,
    pub net_wm_check: u64,
    pub net_wm_fullscreen: u64,
    pub net_wm_window_type: u64,
    pub net_wm_window_type_dialog: u64,
    pub net_wm_window_type_dock: u64,
    pub net_client_list: u64,
    pub net_number_of_desktops: u64,
    pub net_current_desktop: u64,
    pub net_wm_desktop: u64,
    pub net_desktop_names: u64,
    pub net_desktop_viewport: u64,
}

/// Stores all states required by WM to operate
pub struct RuntimeContainer {
    pub screens: Vec<Screen>,
    pub current_screen: usize,
    pub current_workspace: usize,
    pub current_client: Option<usize>,
    pub display: &'static mut x11::xlib::Display,
    pub root_win: u64,
    pub mouse_state: MouseState, // win, button, pos
    pub wm_check_win: u64,
    pub running: bool,
    pub bars: Vec<Bar>,
}

pub struct MouseState {
    pub win: u64,
    pub button: u32,
    pub pos: (i64, i64),
}

impl std::fmt::Debug for RuntimeContainer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WindowSystemContainer")
            .field("screens", &self.screens)
            .field("current_screen", &self.current_screen)
            .field("current_workspace", &self.current_workspace)
            .field("current_client", &self.current_client)
            .field("display", &"no_display")
            .field("root_win", &self.root_win)
            .field("running", &self.running)
            .finish()
    }
}

#[derive(Debug)]
pub struct Screen {
    pub number: i64,
    pub x: i64,
    pub y: i64,
    pub width: i64,
    pub height: i64,
    pub workspaces: Vec<Workspace>,
    pub current_workspace: usize,
    pub bar_offsets: (usize, usize, usize, usize), // up, right, bottom, left
}

#[derive(Debug, Clone)]
pub struct Bar {
    pub window_id: u64,
    pub x: i64,
    pub y: i64,
    pub w: usize,
    pub h: usize,
}

#[derive(Debug)]
pub struct Workspace {
    pub number: u64,
    pub master_capacity: i64,
    pub master_width: f64,
    pub clients: Vec<Client>,
    pub current_client: Option<usize>,
}

#[derive(Debug, Default)]
pub struct Client {
    pub window_id: u64,
    pub window_name: String,
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
    pub visible: bool,
    pub floating: bool,
    pub fullscreen: bool,
    pub fixed: bool,
    pub minw: i32,
    pub minh: i32,
    pub maxw: i32,
    pub maxh: i32,
}
