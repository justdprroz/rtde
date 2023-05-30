//! Definitions of custom abstract structures for window manager

/// ApplicationContainer structure used to store Api and EnvironmentContainer variables
pub struct ApplicationContainer {
    pub environment: EnvironmentContainer,
    pub api: Api,
}

/// Stores all data needed by window manager to operate eg configuration and
/// states
pub struct EnvironmentContainer {
    pub config: ConfigurationContainer,
    pub window_system: WindowSystemContainer,
}

/// Contains user-definable configurations like visual preferences, shortcuts(actions), rules for
/// layouts and finally status bar builing rules
pub struct ConfigurationContainer {
    pub visual_preferences: VisualPreferences,
    pub key_actions: Vec<KeyAction>,
    pub layout_rules: Vec<LayoutRule>,
    pub status_bar_builder: Vec<StatusBarBuilder>,
}

/// Stores settings used un UI
pub struct VisualPreferences {
    pub gap_width: usize,
    pub border_size: usize,
    pub normal_border_color: u64,
    pub active_border_color: u64,
}

/// Struct used for defining keboard shortcuts. Has keysym (Key Symbol), modifier (eg Ctrl, Shift
/// etc) and result (action that will be run on successful trigger)
#[derive(Clone)]
pub struct KeyAction {
    pub keysym: u32,
    pub modifier: u32,
    pub result: ActionResult,
}

pub struct ButtonClickTrigger {}

/// Enum that defines results of shortcuts
#[derive(Debug, Clone)]
pub enum ActionResult {
    /// Kills current client
    KillClient,
    /// Spawns specified command (String)
    Spawn(String),
    /// Moves current client to chosen screen
    MoveToScreen(ScreenSwitching),
    /// Focuses on chosen screen
    FocusOnScreen(ScreenSwitching),
    /// Changes width of master client
    UpdateMasterCapacity(i64),
    /// Changes amount of windows in master
    UpdateMasterWidth(f64),
    /// Moves current client to specified workspace on current screen
    MoveToWorkspace(u64),
    /// Focuses on specified workspace on current screen
    FocusOnWorkspace(u64),
    /// Temporaly maximazes window (NOT YET IMPLEMENTED)
    MaximazeWindow,
    ToggleFloat,
    /// Dumps all states to logs
    DumpInfo,
    /// Exits Wm
    Quit,
}

/// Enum for choosing screen. Currently supports Next and Previous screen
#[derive(Debug, Clone)]
pub enum ScreenSwitching {
    Next,
    Previous,
}

pub struct LayoutRule {}

pub struct StatusBarBuilder {}

#[derive(Debug)]
pub struct Atoms {
    pub wm_protocols: u64,
    pub wm_delete: u64,
    pub wm_state: u64,
    pub wm_take_focus: u64,
    pub net_active_window: u64,
    pub net_supported: u64,
    pub net_wm_name: u64,
    pub net_wm_state: u64,
    pub net_wm_check: u64,
    pub net_wm_fullscreen: u64,
    pub net_wm_window_type: u64,
    pub net_wm_window_type_dialog: u64,
    pub net_client_list: u64,
}

/// Stores all states required by WM to operate
pub struct WindowSystemContainer {
    pub status_bar: StatusBarContainer,
    pub screens: Vec<Screen>,
    pub current_screen: usize,
    pub current_workspace: usize,
    pub current_client: Option<usize>,
    pub display: &'static mut x11::xlib::Display,
    pub root_win: u64,
    pub wm_check_win: u64,
    pub running: bool,
    pub atoms: Atoms,
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

/// Stores info about screen
#[derive(Debug)]
pub struct Screen {
    /// Index of display
    pub number: i64,
    pub x: i64,
    pub y: i64,
    pub width: i64,
    pub height: i64,
    /// Associated workspaces
    pub workspaces: Vec<Workspace>,
    /// Currently selected workspace
    pub current_workspace: usize,
}

#[derive(Debug)]
pub struct Workspace {
    pub number: u64,
    /// Amount of clients in master
    pub master_capacity: i64,
    /// Width of master window (0 - 0 pixels, 1 - full width of current screen)
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

pub struct Api {}
