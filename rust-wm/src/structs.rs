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
    DumpInfo(),
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

impl std::fmt::Debug for WindowSystemContainer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WindowSystemContainer")
            .field("status_bar", &self.status_bar)
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
pub struct StatusBarContainer {}

#[derive(Debug)]
pub struct Screen {
    pub number: i64,
    pub x: i64,
    pub y: i64,
    pub width: i64,
    pub height: i64,
    pub workspaces: Vec<Workspace>,
    pub current_workspace: usize,
}

#[derive(Debug)]
pub struct Workspace {
    pub number: u64,
    pub master_size: i64,
    pub master_width: f64,
    pub clients: Vec<Client>,
    pub current_client: Option<usize>,
}

#[derive(Debug)]
pub struct Client {
    pub window_id: u64,
    pub window_name: String,
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
    pub visible: bool,
    pub px: i32,
    pub py: i32,
}

pub struct Api {}
