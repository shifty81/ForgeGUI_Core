//! One-stop renderer-neutral facade for ordinary ForgeGUI applications.
#![forbid(unsafe_code)]
pub use forge_gui_accessibility as accessibility;
pub use forge_gui_app as app;
pub use forge_gui_core as core;
pub use forge_gui_data as data;
pub use forge_gui_dialog as dialog;
pub use forge_gui_extension as extension;
pub use forge_gui_forms as forms;
pub use forge_gui_i18n as i18n;
pub use forge_gui_log as log;
pub use forge_gui_overlay as overlay;
pub use forge_gui_persistence as persistence;
pub use forge_gui_platform as platform;
pub use forge_gui_search as search;
pub use forge_gui_services as services;
pub use forge_gui_settings as settings;
pub use forge_gui_state as state;
pub mod prelude {
    pub use crate::app::{AppPolicy, AppProfile, Breakpoint, NavigationState};
    pub use crate::data::{
        CollectionProvider, CollectionSelection, CollectionView, Query, TabularProvider,
    };
    pub use crate::forms::{FieldKind, FieldSpec, FormModel, FormValue};
    pub use crate::services::{
        CommandRegistry, CommandSpec, CommandState, SelectionService, TaskRegistry,
    };
}
