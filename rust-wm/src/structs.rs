pub struct ApplicationContainer {
    pub environment: EnvironmentContainer,
    pub api: Api,
}

pub struct EnvironmentContainer {
    pub config: ConfigurationContainer,
    pub window_system: WindowSystemContainer,
}

pub struct ConfigurationContainer {
    pub visual_preferences: VisualPreferencesContainer,
    pub actions: Vec<Action>,
    pub layout_rules: LayoutRulesContainer,
    pub status_bar_builder: StatusBarBuilderContainer,
}

pub struct VisualPreferencesContainer {

}

pub struct Action {
    pub trigger: ActionTrigger,
    pub result: ActionResult,
}

pub enum ActionTrigger {
    KeyPress(KeyPressTrigger),
    ButtonClick(ButtonClickTrigger),
}

pub struct KeyPressTrigger {
    pub modifier: i32,
    pub ketcode: i32, 
}

pub struct ButtonClickTrigger {

}

pub enum ActionResult {
    KillClient,
    Spawn(String),
    MoveToScreen(ScreenSwitching),
    FocusOnScreen(ScreenSwitching),
    MoveToWorkspace(u64),
    FocusOnWorkspac(u64),
    Quit,
}

pub enum ScreenSwitching {
    Next,
    Previous,
}

pub struct LayoutRulesContainer {

}

pub struct StatusBarBuilderContainer {

}

pub struct WindowSystemContainer {
    pub status_bar: StatusBarContainer,
    pub variables: VariablesContainer,
    pub screens: Vec<Screen>,
    pub current_screen: usize
}

pub struct StatusBarContainer {

}

pub struct VariablesContainer {

}

pub struct Screen {
    pub number: u64,
    pub x: i64,
    pub y: i64,
    pub width: u64,
    pub height: u64,
    pub workspaces: Vec<Workspace>,
    pub current_workspace: usize
}

pub struct Workspace {
    pub number: u64,
    pub clients: Vec<Client>,
    pub current_client: usize,
}

pub struct Client {
    pub window_id: u64,
    pub window_name: String,
    pub x: i64,
    pub y: i64,
    pub w: u64,
    pub h: u64,
    pub visible: bool,
    pub px: i64,
    pub py: i64
}

pub struct Api {

}
