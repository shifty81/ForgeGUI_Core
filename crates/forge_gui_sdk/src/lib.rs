//! Compatibility facade. New consumers should depend on `forge_gui` directly.
#![forbid(unsafe_code)]
pub use forge_gui::*;
pub mod prelude {
    pub use forge_gui::{
        show_application_shell, CommandId, EguiPanel, ForgeApplicationSpec, ForgeGuiRuntime,
        ForgeShellContent, ForgeShellState, GuiEvent, IconId, PanelDefinition, PanelId,
        PanelInstanceId, PanelRole, PanelScope, PanelUiContext, PreferredDock, ThemeTokens,
    };
}
