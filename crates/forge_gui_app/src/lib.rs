//! Renderer-neutral application-shell contracts for ForgeGUI.
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum AppProfile {
    Minimal,
    #[default]
    Standard,
    Dashboard,
    Utility,
    Workspace,
    Kiosk,
    Custom,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Breakpoint {
    Compact,
    Medium,
    Large,
    Wide,
}

impl Breakpoint {
    pub fn from_width(width: f32) -> Self {
        if width < 680.0 {
            Self::Compact
        } else if width < 980.0 {
            Self::Medium
        } else if width < 1440.0 {
            Self::Large
        } else {
            Self::Wide
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AppPolicy {
    pub profile: AppProfile,
    pub show_menu: bool,
    pub show_navigation: bool,
    pub show_status: bool,
    pub content_first: bool,
    pub allow_multi_window: bool,
}
impl AppPolicy {
    pub fn for_profile(profile: AppProfile) -> Self {
        match profile {
            AppProfile::Minimal => Self {
                profile,
                show_menu: false,
                show_navigation: false,
                show_status: false,
                content_first: true,
                allow_multi_window: false,
            },
            AppProfile::Utility => Self {
                profile,
                show_menu: true,
                show_navigation: false,
                show_status: true,
                content_first: true,
                allow_multi_window: false,
            },
            AppProfile::Dashboard => Self {
                profile,
                show_menu: true,
                show_navigation: true,
                show_status: true,
                content_first: true,
                allow_multi_window: true,
            },
            AppProfile::Kiosk => Self {
                profile,
                show_menu: false,
                show_navigation: false,
                show_status: false,
                content_first: true,
                allow_multi_window: false,
            },
            AppProfile::Workspace => Self {
                profile,
                show_menu: true,
                show_navigation: true,
                show_status: true,
                content_first: true,
                allow_multi_window: true,
            },
            AppProfile::Standard | AppProfile::Custom => Self {
                profile,
                show_menu: true,
                show_navigation: true,
                show_status: true,
                content_first: true,
                allow_multi_window: true,
            },
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum MenuItemKind {
    Command,
    Toggle,
    Radio { group: String },
    Separator,
    Submenu,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MenuItem {
    pub id: String,
    pub label: String,
    pub kind: MenuItemKind,
    pub command_id: Option<String>,
    pub enabled: bool,
    pub checked: bool,
    pub children: Vec<MenuItem>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Menu {
    pub id: String,
    pub label: String,
    pub items: Vec<MenuItem>,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum StatusSeverity {
    #[default]
    Neutral,
    Activity,
    Success,
    Warning,
    Error,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct StatusItem {
    pub id: String,
    pub label: String,
    pub value: String,
    pub severity: StatusSeverity,
    pub command_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WindowDescriptor {
    pub id: String,
    pub title: String,
    pub size: [f32; 2],
    pub min_size: [f32; 2],
    pub resizable: bool,
    pub persistent: bool,
}

#[derive(Default)]
pub struct WindowRegistry {
    windows: BTreeMap<String, WindowDescriptor>,
}
impl WindowRegistry {
    pub fn register(&mut self, window: WindowDescriptor) -> bool {
        self.windows.insert(window.id.clone(), window).is_none()
    }
    pub fn get(&self, id: &str) -> Option<&WindowDescriptor> {
        self.windows.get(id)
    }
    pub fn iter(&self) -> impl Iterator<Item = &WindowDescriptor> {
        self.windows.values()
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct NavigationState {
    pub current: Option<String>,
    pub back: Vec<String>,
    pub forward: Vec<String>,
    pub favorites: Vec<String>,
    pub recent: Vec<String>,
}
impl NavigationState {
    pub fn navigate(&mut self, target: impl Into<String>) {
        let target = target.into();
        if let Some(current) = self.current.take() {
            if current != target {
                self.back.push(current);
                self.forward.clear();
            }
        }
        self.current = Some(target.clone());
        self.recent.retain(|v| v != &target);
        self.recent.insert(0, target);
        self.recent.truncate(20);
    }
    pub fn go_back(&mut self) -> Option<&str> {
        let target = self.back.pop()?;
        if let Some(current) = self.current.replace(target) {
            self.forward.push(current);
        }
        self.current.as_deref()
    }
    pub fn go_forward(&mut self) -> Option<&str> {
        let target = self.forward.pop()?;
        if let Some(current) = self.current.replace(target) {
            self.back.push(current);
        }
        self.current.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn utility_is_content_first() {
        assert!(AppPolicy::for_profile(AppProfile::Utility).content_first);
    }
    #[test]
    fn nav_history_roundtrips() {
        let mut n = NavigationState::default();
        n.navigate("a");
        n.navigate("b");
        assert_eq!(n.go_back(), Some("a"));
        assert_eq!(n.go_forward(), Some("b"));
    }
}
