
# Rust Tiling Desktop Environment

## Data Stucture Reference

> **Container** is an object(struct with defined methods) used to store and manage related to it data

* `ApplicationContainer`

    > Main WM container that holds all data and mothods
    >
    > **Fields:**
    >
    > `environment` - holds `EnvironmentContainer`
    >
    > `api` - holds `ApiContrainer`

    * `EnvironmentContainer`

        > Container that holds everything related to user side of WM
        >
        > **Fields:**
        >
        > `config` - holds `ConfigurationContainer`
        >
        > `window_system` - holds `WindowSystemContainer`

        * `ConfigurationContainer`

            > Stores **immutable** data like UI preferences, shortcuts, arrangement rules, etc.
            >
            > **Fields:**
            >
            > `visual_preferences` - holds `VisualPreferencesContainer`
            >
            > `actions` - holds `Vec` of `Action`s
            >
            > `layout_rules` - holds `LayoutRulesContainer`
            >
            > `status_bar_builder` - holds `StatusBarBuilderContainer`

            * `VisualPreferencesContainer`

                > Holds UI settings like: colors, workspace names etc.
                >
                > <span style="background:yellow;color:black;padding:5px"><b>Not yet implemented</b></span>

            * `Action`

                > Container for storing bindings of **trigger -> actions**
                >
                > **Fields:**
                >
                > `trigger` - `ActionTrigger`
                >
                > `result` - `ActionResult`

                * `ActionTrigger`

                    > Enum which defines events which should occur to run `Action`
                    >
                    > **Fields:**
                    >
                    > `KeyPress` - `KeyPressTrigger`
                    >
                    > `ButtonClick` - `ButtonClickTrigger`
                    >

                    * `KeyPressTrigger`

                        > Struct which defines KeyPress event
                        >
                        > **Fields:**
                        >
                        > `modifier` - key modifier of `XEvent.key` event
                        >
                        > `keycode` - keycode

                    * `ButtonClickTrigger`

                        > Struct which defines several types of button click(which because of complexity will be calculated in other place)
                        >
                        > <span style="background:yellow;color:black;padding:5px"><b>Not yet implemented</b></span>

                * `ActionResult`

                    > Enum which defines function that will be executed after succesful trigger
                    >
                    > **Fields:**
                    >
                    > `KillClient` - kills current client
                    >
                    > `Spawn` - executes provided `Command`
                    >
                    > `MoveToScreen` - moves current client to `Screen` specified by `ScreenSwitching`
                    >
                    > `MoveToWorkspace` - moves current client to specified `Workspace`
                    >
                    > `FocusScreen` - focuses on `Screen` specified by `ScreenSwitching`
                    >
                    > `FocusWorkspace` - focuses on specified `Workspace`
                    >
                    > `Quit` - exits WM gracefuly

                    * `ScreenSwitching`

                        > Enum which specifies requested `Screen`
                        >
                        > **Fields:**
                        >
                        > `Next` - moves to next screen
                        >
                        > `Previous` - moves to previous screen

            * `LayoutRulesContainer`

                > Stores information about preferred layouts, window arrangement rules, etc
                >
                > <span style="background:yellow;color:black;padding:5px"><b>Not yet implemented</b></span>

            * `StatusBarBuilderContainer`

                > Stores user-defined set of rules for builfing status bar
                >
                > <span style="background:yellow;color:black;padding:5px"><b>Not yet implemented</b></span>

        * `WindowSystemContainer`

            > Main container that stores **mutable** data manipulated during each iteration of main loop.
            > Including: apps, `Screen`s, workspaces and everything related to X windows
            >
            > **Fields:**
            >
            > `status_bar` - holds `StatusBarContainer`
            >
            > `variables` - holds `VariablesContainer`
            >
            > `screens` - holds collection of `Screen`s
            >
            > `current_screen` - holds index of current_screen

            * `StatusBarContainer`

                > Stores constucred status bar ready to be rendered
                >
                > <span style="background:yellow;color:black;padding:5px"><b>Not yet implemented</b></span>

            * `VariablesContainer`

                > Stores some session-only data like scales, selected layout type, active window info, etc.
                >
                > <span style="background:yellow;color:black;padding:5px"><b>Not yet implemented</b></span>

            * `Screen`

                > An abstract type above screen
                >
                > **Fields:**
                >
                > `number` - holds number of given screen
                >
                > `x` - holds x position in Xinerama layout
                >
                > `y` - holds y position in Xinerama layout
                >
                > `width` - holds the width of screen
                >
                > `height` - holds the height if screen
                >
                > `workspaces` - holds `Vec` of `Workspace`s
                >
                > `current_workspace` - stores current workspace index

                * `Workspace`

                    > An abstract type for workspace
                    >
                    > **Fields**
                    >
                    > `number` - holds the number of workspace in `WorkspaceContainer`
                    >
                    > `clients` - holds `Vec`(`Vec` in this case used as stack implementation) of `ClientContainer`
                    >
                    > `current_client` - stores index of current client

                    * `Client`

                        > abstract type over window
                        >
                        > **Fields**
                        >
                        > `window_id` - holds `u64` representing X server id of corresponding window
                        >
                        > `window_name` - holds name of given window
                        >
                        > `x`, `y`, `w`, `h` - basic geometry of window
                        >
                        > `visible` - states if window is currently observed by user
                        >
                        > `px`, `py` - stores previous position in case of hiding window(moving away from screen)

    * ApiContrainer

        > <span style="background:#e8765f;color:black;padding:5px"><b>TODO: Design API container reference</b></span>

## Call Stack of WM

> `formal functions` are functions that will exit in code as they are shown here
> 
> *informal actions* are simpified explanations of what code does at given stage

* `main`

    > Main entry function of WM, every varible storing in scope of this function

    * `setup`

        > Declares all variables in `VariableContainer`, creates LUTs, prepares WM to function normally

    * `scan`

        > Gets windows that are already present and adds them to `ClientsArrangementContainer` with rules from `LayoutRulesContainer` applied

    * `run`

        > Main loop for processing events, triggering actions, updating status bar, managing opened windows

        > <span style="background:#e8765f;color:black;padding:5px"><b>TODO: Design main loop call stack</b></span>

        * *Fetch events*
        * *Update status bar*

    * `cleanup`

        > Close all connections to X server, delete all temporary data, flush data to cache, finish logging and finally gracefuly quit WM

