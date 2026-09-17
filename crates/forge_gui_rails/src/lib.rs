//! Configurable activity/tab rails for ForgeGUI_Core.
#![forbid(unsafe_code)]

use egui::{Align, Button, Color32, Layout, Margin, RichText, Stroke, Ui, Vec2};
use forge_gui_core::{DockLockMode, Rgba};
use forge_gui_theme::{ForgeTheme, SurfaceRole};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum RailPreset {
    SlimIcon,
    LabeledTool,
    DualContext,
    FloatingPill,
}

impl RailPreset {
    pub const ALL: [Self; 4] = [
        Self::SlimIcon,
        Self::LabeledTool,
        Self::DualContext,
        Self::FloatingPill,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::SlimIcon => "Slim Icon",
            Self::LabeledTool => "Labeled Tool",
            Self::DualContext => "Dual Context",
            Self::FloatingPill => "Floating Pill",
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum RailEdge {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum RailPlacement {
    Flush,
    InsideEdge,
    Inset,
    Floating,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum RailDensity {
    Compact,
    Normal,
    Comfortable,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum LabelPolicy {
    Hidden,
    TooltipOnly,
    SelectedOnly,
    Always,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum RailItemState {
    Neutral,
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RailItem {
    pub id: String,
    pub label: String,
    pub icon: String,
    pub tooltip: Option<String>,
    pub badge: Option<String>,
    pub dirty: bool,
    pub disabled: bool,
    pub state: RailItemState,
}

impl RailItem {
    pub fn new(id: impl Into<String>, icon: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: icon.into(),
            tooltip: None,
            badge: None,
            dirty: false,
            disabled: false,
            state: RailItemState::Neutral,
        }
    }

    pub fn tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    pub fn badge(mut self, badge: impl Into<String>) -> Self {
        self.badge = Some(badge.into());
        self
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RailSection {
    pub label: Option<String>,
    pub items: Vec<RailItem>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RailStyle {
    pub preset: RailPreset,
    pub edge: RailEdge,
    pub placement: RailPlacement,
    pub density: RailDensity,
    pub labels: LabelPolicy,
    pub lock_mode: DockLockMode,
    pub collapsible: bool,
    pub auto_hide: bool,
    pub allow_reorder: bool,
    pub allow_overflow: bool,
    pub show_badges: bool,
}

impl Default for RailStyle {
    fn default() -> Self {
        Self {
            preset: RailPreset::SlimIcon,
            edge: RailEdge::Left,
            placement: RailPlacement::InsideEdge,
            density: RailDensity::Normal,
            labels: LabelPolicy::TooltipOnly,
            lock_mode: DockLockMode::Flexible,
            collapsible: true,
            auto_hide: false,
            allow_reorder: true,
            allow_overflow: true,
            show_badges: true,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RailModel {
    pub id: String,
    pub active: Option<String>,
    pub collapsed: bool,
    pub style: RailStyle,
    pub sections: Vec<RailSection>,
}

impl RailModel {
    pub fn new(id: impl Into<String>, style: RailStyle) -> Self {
        Self {
            id: id.into(),
            active: None,
            collapsed: false,
            style,
            sections: Vec::new(),
        }
    }

    pub fn set_active(&mut self, id: impl Into<String>) {
        self.active = Some(id.into());
    }

    pub fn is_active(&self, id: &str) -> bool {
        self.active.as_deref() == Some(id)
    }

    pub fn toggle_collapsed(&mut self) {
        if self.style.collapsible {
            self.collapsed = !self.collapsed;
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct RailResponse {
    pub activated: Option<String>,
    pub collapse_toggled: bool,
}

pub fn show_rail(ui: &mut Ui, model: &mut RailModel, theme: &ForgeTheme) -> RailResponse {
    let vertical = matches!(model.style.edge, RailEdge::Left | RailEdge::Right);
    let surface = theme.surface(match model.style.preset {
        RailPreset::FloatingPill => SurfaceRole::Base,
        _ => SurfaceRole::Raised1,
    });
    let mut response = RailResponse::default();

    egui::Frame::new()
        .fill(color(surface.fill))
        .stroke(Stroke::new(1.0, color(surface.border)))
        .corner_radius(surface.corner_radius as u8)
        .inner_margin(Margin::same(4))
        .show(ui, |ui| {
            if model.style.collapsible {
                let glyph = if model.collapsed {
                    if vertical {
                        "»"
                    } else {
                        "∨"
                    }
                } else if vertical {
                    "«"
                } else {
                    "∧"
                };
                if ui.small_button(glyph).clicked() {
                    model.toggle_collapsed();
                    response.collapse_toggled = true;
                }
                ui.add_space(3.0);
            }

            if model.collapsed {
                return;
            }

            if vertical {
                ui.vertical(|ui| render_sections(ui, model, theme, true, &mut response));
            } else {
                ui.horizontal_wrapped(|ui| render_sections(ui, model, theme, false, &mut response));
            }
        });

    response
}

fn render_sections(
    ui: &mut Ui,
    model: &mut RailModel,
    theme: &ForgeTheme,
    vertical: bool,
    response: &mut RailResponse,
) {
    let sections = model.sections.clone();

    for (section_index, section) in sections.into_iter().enumerate() {
        if section_index > 0 {
            if vertical {
                ui.separator();
            } else {
                ui.add_space(2.0);
                ui.separator();
                ui.add_space(2.0);
            }
        }

        if vertical
            && matches!(
                model.style.preset,
                RailPreset::LabeledTool | RailPreset::DualContext
            )
        {
            if let Some(label) = section.label.as_deref() {
                ui.add_space(4.0);
                ui.label(
                    RichText::new(label.to_uppercase())
                        .small()
                        .color(color(theme.base.text_muted)),
                );
                ui.add_space(2.0);
            }
        }

        for item in section.items {
            let active = model.is_active(&item.id);
            let item_response = show_item(ui, model, &item, active, theme, vertical);

            if item_response.clicked() && !item.disabled {
                model.set_active(item.id.clone());
                response.activated = Some(item.id.clone());
            }
        }
    }
}

fn show_item(
    ui: &mut Ui,
    model: &RailModel,
    item: &RailItem,
    active: bool,
    theme: &ForgeTheme,
    vertical: bool,
) -> egui::Response {
    let show_label = match model.style.labels {
        LabelPolicy::Hidden | LabelPolicy::TooltipOnly => false,
        LabelPolicy::SelectedOnly => active,
        LabelPolicy::Always => true,
    };

    let mut label = if show_label {
        format!("{}  {}", item.icon, item.label)
    } else {
        item.icon.clone()
    };

    if model.style.show_badges {
        if let Some(badge) = item.badge.as_deref() {
            label.push_str("  ");
            label.push_str(badge);
        }
    }
    if item.dirty {
        label.push_str("  •");
    }

    let height = match model.style.density {
        RailDensity::Compact => 32.0,
        RailDensity::Normal => 40.0,
        RailDensity::Comfortable => 48.0,
    };

    let width = if vertical {
        match model.style.preset {
            RailPreset::SlimIcon => 46.0,
            RailPreset::LabeledTool => 174.0,
            RailPreset::DualContext => 180.0,
            RailPreset::FloatingPill => 122.0,
        }
    } else if show_label {
        126.0
    } else {
        42.0
    };

    let selected_fill = match model.style.preset {
        RailPreset::FloatingPill => theme.surface(SurfaceRole::Focus).fill,
        _ => theme.base.panel_raised,
    };

    let state_color = match item.state {
        RailItemState::Neutral => theme.base.text_muted,
        RailItemState::Info => theme.base.accent,
        RailItemState::Success => theme.base.success,
        RailItemState::Warning => theme.base.warning,
        RailItemState::Error => theme.base.danger,
    };

    let text_color = if active { theme.base.text } else { state_color };

    let button = Button::selectable(active, RichText::new(label).color(color(text_color)))
        .min_size(Vec2::new(width, height))
        .corner_radius(match model.style.preset {
            RailPreset::FloatingPill => 12,
            _ => 7,
        })
        .fill(if active {
            color(selected_fill)
        } else {
            color(theme.base.panel)
        });

    let response = ui.add_enabled(!item.disabled, button);
    let response = if let Some(tooltip) = item.tooltip.as_deref() {
        response.on_hover_text(tooltip)
    } else if !show_label {
        response.on_hover_text(&item.label)
    } else {
        response
    };

    if active && matches!(model.style.preset, RailPreset::SlimIcon) {
        let rect = response.rect;
        let stroke = Stroke::new(3.0, color(theme.base.accent));

        if vertical {
            let x = if matches!(model.style.edge, RailEdge::Right) {
                rect.right() + 1.0
            } else {
                rect.left() - 1.0
            };
            ui.painter().line_segment(
                [
                    egui::pos2(x, rect.top() + 5.0),
                    egui::pos2(x, rect.bottom() - 5.0),
                ],
                stroke,
            );
        } else {
            let y = if matches!(model.style.edge, RailEdge::Bottom) {
                rect.bottom() + 1.0
            } else {
                rect.top() - 1.0
            };
            ui.painter().line_segment(
                [
                    egui::pos2(rect.left() + 5.0, y),
                    egui::pos2(rect.right() - 5.0, y),
                ],
                stroke,
            );
        }
    }

    response
}

pub fn show_dual_context(
    ui: &mut Ui,
    primary: &mut RailModel,
    secondary: &mut RailModel,
    theme: &ForgeTheme,
) -> (RailResponse, RailResponse) {
    ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
        let primary_response = show_rail(ui, primary, theme);
        ui.add_space(4.0);
        let secondary_response = show_rail(ui, secondary, theme);
        (primary_response, secondary_response)
    })
    .inner
}

fn color(value: Rgba) -> Color32 {
    Color32::from_rgba_unmultiplied(value.0, value.1, value.2, value.3)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_item_round_trip() {
        let mut model = RailModel::new("forge.rail.test", RailStyle::default());
        model.set_active("forge.item.rooms");
        assert!(model.is_active("forge.item.rooms"));
        assert!(!model.is_active("forge.item.objects"));
    }

    #[test]
    fn collapse_obeys_policy() {
        let mut model = RailModel::new("forge.rail.test", RailStyle::default());
        model.toggle_collapsed();
        assert!(model.collapsed);

        model.style.collapsible = false;
        model.toggle_collapsed();
        assert!(model.collapsed);
    }
}
