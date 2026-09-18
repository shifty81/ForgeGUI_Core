//! Reusable structural panel chrome for ForgeGUI Core.
#![forbid(unsafe_code)]

use egui::{Align, Frame, Layout, Margin, RichText, Stroke, Ui};
use forge_gui_icons::IconId;
use forge_gui_theme::{ForgeTheme, SurfaceRole};
use forge_gui_widgets::{compact_tool_button, panel_tab, WidgetTone};

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
        .stroke(Stroke::new(1.0, color(theme.chrome.separator)))
        .corner_radius(1)
        .inner_margin(Margin::same(0))
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
            Frame::new()
                .fill(color(theme.base.panel))
                .inner_margin(Margin::same(theme.effective_metrics().panel_padding))
                .show(ui, add_body)
                .inner
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
        .inner_margin(Margin::symmetric(8, 3))
        .show(ui, |ui| {
            ui.set_min_height((metrics.panel_header_height - 6.0).max(18.0));
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(forge_gui_widgets::icon_text(kind_icon(kind)))
                        .size(11.5)
                        .color(color(theme.base.accent)),
                );
                ui.label(RichText::new(title).strong().size(12.5));
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
    ui.painter().line_segment(
        [ui.min_rect().left_bottom(), ui.min_rect().right_bottom()],
        Stroke::new(1.0, color(theme.chrome.separator)),
    );
}

pub fn show_panel_tabs(
    ui: &mut Ui,
    tabs: &[(&str, &str)],
    active: &str,
    theme: &ForgeTheme,
) -> Option<String> {
    let mut selected = None;
    Frame::new()
        .fill(color(theme.base.panel_recessed))
        .inner_margin(Margin::symmetric(4, 2))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                for (id, label) in tabs {
                    if panel_tab(ui, label, *id == active, theme).clicked() {
                        selected = Some((*id).to_owned());
                    }
                }
            });
        });
    selected
}

pub fn show_section_header(
    ui: &mut Ui,
    label: &str,
    count: Option<usize>,
    open: &mut bool,
    theme: &ForgeTheme,
) -> bool {
    let mut clicked = false;
    Frame::new()
        .fill(color(theme.base.panel_recessed))
        .stroke(Stroke::new(1.0, color(theme.chrome.separator)))
        .corner_radius(2)
        .inner_margin(Margin::symmetric(6, 3))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let glyph = if *open { "▾" } else { "▸" };
                if ui
                    .selectable_label(*open, format!("{glyph}  {label}"))
                    .clicked()
                {
                    clicked = true;
                }
                if let Some(count) = count {
                    let _ = forge_gui_widgets::badge(
                        ui,
                        &count.to_string(),
                        WidgetTone::Neutral,
                        theme,
                    );
                }
            });
        });
    if clicked {
        *open = !*open;
    }
    clicked
}

fn kind_icon(kind: PanelKind) -> IconId {
    match kind {
        PanelKind::Tool => IconId::Settings,
        PanelKind::Inspector => IconId::Inspector,
        PanelKind::Assets => IconId::Asset,
        PanelKind::Outliner => IconId::Layers,
        PanelKind::Output => IconId::Console,
        PanelKind::Utility => IconId::Settings,
    }
}

fn color(value: forge_gui_core::Rgba) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(value.0, value.1, value.2, value.3)
}
