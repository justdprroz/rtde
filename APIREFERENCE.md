
# Rust Tiling Desktop Environment

## Data Stucture Reference

> **Container** is an object(struct with defined methods) used to store and manage related to it data

* `ApplicationContrainer`

    > Main WM container that holds all data and mothods

    * `EnvironmentContainer`

        > Container that holds everything related to user side of WM

        * `ConfigurationContainer`

            > Stores **immutable** data like UI preferences, shortcuts, arrangement rules, etc.

            * `VisualPreferencesContainer`

                > Holds UI settings like: colors, workspace names etc.

            * `ActionsContainer`

                > Container for storing bindings of **trigger -> actions**

            * `LayoutRulesContainer`

                > Stores information about preferred layouts, window arrangement rules, etc

            * `StatusBarBuilderContainer`

                > Stores user-defined set of rules for builfing status bar

        * `ClientsArrangementContainer`

            > Main container that stores **mutable** data manipulated during each iteration of main loop.
            > Including: apps, monitos, workspaces and everything related to X windows

            * `StatusContainer`

                > Stores constucred status bar ready to be rendered

            * `VariableContainer`

                > Stores some session-only data like scales, selected layout type, active window info, etc.

            * `MonitorContainer`

                > Container used to arrange monitors

                * `WorkspaceContainer`

                    > Stores array of workspaces (eg tags for dwm users)

                    * `ClientContainer`

                        > Stores and manages stack **or** tree of windows(clients) used on current workspace

    * ApiContrainer

        > <span style="background:#e8765f;color:black;padding:5px">
        >     TODO: Design API container reference
        > </span>

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

        > <span style="background:#e8765f;color:black;padding:5px">
        >    TODO: Design main loop call stack
        > </span>

        * *Fetch events*
        * *Update status bar*

    * `cleanup`

        > Close all connections to X server, delete all temporary data, flush data to cache, finish logging and finally gracefuly quit WM

