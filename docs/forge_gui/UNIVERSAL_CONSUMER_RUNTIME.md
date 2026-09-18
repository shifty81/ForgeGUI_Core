# ForgeGUI universal consumer runtime

ForgeGUI_Core is a GUI framework. `forge_gui_lab` is only a showcase executable.

## Consumer rule

A normal application registers surfaces and supplies application-specific content through
`ForgeShellContent`. It does **not** copy window chrome, dock handling, tab-stack state, floating
window logic, toolbar placement or layout persistence from the Lab.

## Public reference path

- `forge_gui_shell::ForgeShellState` — persistent shell/surface state
- `forge_gui_shell::ForgeApplicationSpec` — application identity
- `forge_gui_shell::ForgeShellContent` — consumer-owned content callbacks
- `forge_gui_shell::show_application_shell` — complete reusable egui host
- `forge_gui_consumer_starter` — independent integration/certification executable

## Acceptance rule

A ForgeGUI capability is not considered universally consumable merely because the Lab can display
it. The independent consumer starter must be able to exercise it through the public framework API
without copying Lab implementation code.
