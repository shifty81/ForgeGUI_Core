//! Reusable structural panel chrome for ForgeGUI Core.
#![forbid(unsafe_code)]

use egui::{Align, Frame, Layout, Margin, RichText, Stroke, Ui};
use forge_gui_icons::IconId;
use forge_gui_theme::{ForgeTheme, SurfaceRole};
use forge_gui_widgets::{compact_tool_button, WidgetTone};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PanelKind {
    #[default]
    Tool,
    Inspector,
    Assets,
    Outliner,
    Output,
    Utility,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PanelHeaderAction {
    pub id: String,
    pub icon: Option<IconId>,
    pub label: String,
    pub tooltip: String,
    pub active: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PanelHeaderResponse {
    pub invoked: Option<String>,
}

pub fn show_panel<R>(
    ui: &mut Ui,
    title: &str,
    subtitle: Option<&str>,
    kind: PanelKind,
    actions: &[PanelHeaderAction],
    theme: &ForgeTheme,
    add_body: impl FnOnce(&mut Ui) -> R,
) -> (R, PanelHeaderResponse) {
    let surface = theme.surface(SurfaceRole::Raised1);
    let mut header_response = PanelHeaderResponse::default();
    let inner = Frame::new()
        .fill(color(surface.fill))
        .stroke(Stroke::new(1.0, color(surface.border)))
        .corner_radius(surface.corner_radius as u8)
        .inner_margin(Margin::same(theme.effective_metrics().panel_padding))
        .show(ui, |ui| {
            show_panel_header(
                ui,
                title,
                subtitle,
                kind,
                actions,
                theme,
                &mut header_response,
            );
            ui.add_space(4.0);
            add_body(ui)
        })
        .inner;

    (inner, header_response)
}

#[allow(clippy::too_many_arguments)]
fn show_panel_header(
    ui: &mut Ui,
    title: &str,
    subtitle: Option<&str>,
    kind: PanelKind,
    actions: &[PanelHeaderAction],
    theme: &ForgeTheme,
    response: &mut PanelHeaderResponse,
) {
    let metrics = theme.effective_metrics();
    Frame::new()
        .fill(color(theme.chrome.panel_header))
        .corner_radius(5)
        .inner_margin(Margin::symmetric(8, 4))
        .show(ui, |ui| {
            ui.set_min_height((metrics.panel_header_height - 8.0).max(18.0));
            ui.horizontal(|ui| {
                ui.label(RichText::new(kind_glyph(kind)).color(color(theme.base.text_muted)));
                ui.strong(title);
                if let Some(subtitle) = subtitle {
                    ui.label(
                        RichText::new(subtitle)
                            .small()
                            .color(color(theme.base.text_muted)),
                    );
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    for action in actions.iter().rev() {
                        let clicked = if let Some(icon) = action.icon {
                            compact_tool_button(ui, icon, &action.tooltip, action.active, theme)
                                .clicked()
                        } else {
                            ui.small_button(&action.label)
                                .on_hover_text(&action.tooltip)
                                .clicked()
                        };
                        if clicked {
                            response.invoked = Some(action.id.clone());
                        }
                    }
                });
            });
        });
}

pub fn show_section_header(
    ui: &mut Ui,
    label: &str,
    count: Option<usize>,
    open: &mut bool,
    theme: &ForgeTheme,
) -> bool {
    let response = ui.horizontal(|ui| {
        let glyph = if *open { "▾" } else { "▸" };
        let clicked = ui
            .selectable_label(*open, format!("{glyph}  {label}"))
            .clicked();
        if let Some(count) = count {
            let _ = forge_gui_widgets::badge(ui, &count.to_string(), WidgetTone::Neutral, theme);
        }
        clicked
    });
    if response.inner {
        *open = !*open;
    }
    response.inner
}

fn kind_glyph(kind: PanelKind) -> &'static str {
    match kind {
        PanelKind::Tool => "◆",
        PanelKind::Inspector => "≡",
        PanelKind::Assets => "▦",
        PanelKind::Outliner => "☷",
        PanelKind::Output => "▤",
        PanelKind::Utility => "◇",
    }
}

fn color(value: forge_gui_core::Rgba) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(value.0, value.1, value.2, value.3)
}
