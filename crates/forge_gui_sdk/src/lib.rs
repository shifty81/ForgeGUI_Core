//! Compatibility facade. New consumers should depend on `forge_gui` directly.
#![forbid(unsafe_code)]
pub use forge_gui::*;
pub mod prelude {
    pub use forge_gui::{
        CommandId, EguiPanel, ForgeGuiRuntime, GuiEvent, IconId, PanelDefinition, PanelId,
        PanelInstanceId, PanelRole, PanelScope, PanelUiContext, PreferredDock, ThemeTokens,
    };
}
