pub struct ApplicationContainer {
    pub environment: EnvironmentContainer,
    pub api: Api,
}

pub struct EnvironmentContainer {
    pub config: ConfigurationContainer,
    pub window_system: WindowSystemContainer,
}

pub struct ConfigurationContainer {
    pub visual_preferences: Vec<VisualPreference>,
    pub key_actions: Vec<KeyAction>,
    pub layout_rules: Vec<LayoutRule>,
    pub status_bar_builder: Vec<StatusBarBuilder>,
}

pub struct VisualPreference {}

pub struct KeyAction {
    pub keysym: u32,
    pub modifier: u32,
    pub result: ActionResult,
}

pub struct ButtonClickTrigger {}

#[derive(Debug)]
pub enum ActionResult {
    KillClient,
    Spawn(String),
    MoveToScreen(ScreenSwitching),
    FocusOnScreen(ScreenSwitching),
    UpdateMasterSize(i64),
    UpdateMasterWidth(f64),
    MoveToWorkspace(u64),
    FocusOnWorkspace(u64),
    MaximazeWindow(),
    Quit,
}

#[derive(Debug)]
pub enum ScreenSwitching {
    Next,
    Previous,
}

pub struct LayoutRule {}

pub struct StatusBarBuilder {}

pub struct WindowSystemContainer {
    pub status_bar: StatusBarContainer,
    pub screens: Vec<Screen>,
    pub current_screen: usize,
    pub current_workspace: usize,
    pub current_client: Option<usize>,
    pub display: &'static mut x11::xlib::Display,
    pub root_win: u64,
    pub running: bool,
}

pub struct StatusBarContainer {}

pub struct Screen {
    pub number: i64,
    pub x: i64,
    pub y: i64,
    pub width: i64,
    pub height: i64,
    pub workspaces: Vec<Workspace>,
    pub current_workspace: usize,
}

pub struct Workspace {
    pub number: u64,
    pub master_size: i64,
    pub master_width: f64,
    pub clients: Vec<Client>,
    pub current_client: Option<usize>,
}

pub struct Client {
    pub window_id: u64,
    pub window_name: String,
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
    pub visible: bool,
    pub px: i64,
    pub py: i64,
}

pub struct Api {}
