//! Structural workbench state for ForgeGUI_Core.
#![forbid(unsafe_code)]

use forge_gui_theme::ForgeTheme;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum BottomTrayMode {
    Collapsed,
    Normal,
    Expanded,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WorkbenchConfig {
    pub top_height: f32,
    pub left_width: f32,
    pub right_width: f32,
    pub bottom_height: f32,
    pub bottom_expanded_height: f32,
    pub status_height: f32,
    pub show_left: bool,
    pub show_right: bool,
    pub show_bottom: bool,
}

impl Default for WorkbenchConfig {
    fn default() -> Self {
        Self {
            top_height: 72.0,
            left_width: 260.0,
            right_width: 320.0,
            bottom_height: 220.0,
            bottom_expanded_height: 360.0,
            status_height: 26.0,
            show_left: true,
            show_right: true,
            show_bottom: true,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WorkbenchState {
    pub focus_mode: bool,
    pub bottom_tray: BottomTrayMode,
    pub config: WorkbenchConfig,
    pub active_document: Option<String>,
    pub active_left_tool: Option<String>,
    pub active_right_tool: Option<String>,
    pub active_bottom_tool: Option<String>,
}

impl Default for WorkbenchState {
    fn default() -> Self {
        Self {
            focus_mode: false,
            bottom_tray: BottomTrayMode::Normal,
            config: WorkbenchConfig::default(),
            active_document: None,
            active_left_tool: None,
            active_right_tool: None,
            active_bottom_tool: Some("forge.bottom.console".into()),
        }
    }
}

impl WorkbenchState {
    pub fn toggle_focus_mode(&mut self) {
        self.focus_mode = !self.focus_mode;
    }

    pub fn cycle_bottom_tray(&mut self) {
        self.bottom_tray = match self.bottom_tray {
            BottomTrayMode::Collapsed => BottomTrayMode::Normal,
            BottomTrayMode::Normal => BottomTrayMode::Expanded,
            BottomTrayMode::Expanded => BottomTrayMode::Collapsed,
        };
    }

    pub fn bottom_height(&self) -> f32 {
        match self.bottom_tray {
            BottomTrayMode::Collapsed => 30.0,
            BottomTrayMode::Normal => self.config.bottom_height,
            BottomTrayMode::Expanded => self.config.bottom_expanded_height,
        }
    }
}

pub fn apply_workbench_visuals(ctx: &egui::Context, theme: &ForgeTheme) {
    ctx.set_theme(egui::Theme::Dark);
    let mut style = (*ctx.style_of(egui::Theme::Dark)).clone();
    style.visuals = egui::Visuals::dark();
    style.visuals.panel_fill = color(theme.base.panel);
    style.visuals.window_fill = color(theme.base.panel_raised);
    style.visuals.extreme_bg_color = color(theme.base.panel_recessed);
    style.visuals.faint_bg_color = color(theme.base.panel_raised);
    style.visuals.selection.bg_fill = color(theme.base.panel_raised);
    style.visuals.selection.stroke = egui::Stroke::new(1.0, color(theme.base.accent));
    style.visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, color(theme.base.text));
    style.visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, color(theme.base.text_muted));
    style.visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, color(theme.base.text));
    style.visuals.widgets.hovered.bg_fill = color(theme.base.panel_raised);
    style.visuals.widgets.active.bg_fill = color(theme.base.panel_raised);
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(10.0, 7.0);
    ctx.set_style_of(egui::Theme::Dark, style);
}

fn color(value: forge_gui_core::Rgba) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(value.0, value.1, value.2, value.3)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bottom_tray_cycles() {
        let mut state = WorkbenchState::default();
        assert_eq!(state.bottom_tray, BottomTrayMode::Normal);
        state.cycle_bottom_tray();
        assert_eq!(state.bottom_tray, BottomTrayMode::Expanded);
        state.cycle_bottom_tray();
        assert_eq!(state.bottom_tray, BottomTrayMode::Collapsed);
    }

    #[test]
    fn focus_mode_toggles() {
        let mut state = WorkbenchState::default();
        state.toggle_focus_mode();
        assert!(state.focus_mode);
    }
}
