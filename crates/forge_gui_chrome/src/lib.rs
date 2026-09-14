//! Product chrome, creator bars, workspace tabs, tool trays, and status chrome.
#![forbid(unsafe_code)]

use egui::{Align, Frame, Layout, Margin, RichText, Stroke, Ui};
use forge_gui_icons::IconId;
use forge_gui_theme::ForgeTheme;
use forge_gui_widgets::{compact_tool_button, tool_button, WidgetTone};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChromeBarKind {
    Product,
    Menu,
    Action,
    WorkspaceTabs,
    BottomTray,
    Status,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceTab {
    pub id: String,
    pub label: String,
    pub icon: Option<IconId>,
    pub dirty: bool,
    pub closable: bool,
}

impl WorkspaceTab {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            dirty: false,
            closable: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolTrayTab {
    pub id: String,
    pub label: String,
    pub badge: Option<String>,
}

impl ToolTrayTab {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            badge: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusItem {
    pub label: String,
    pub value: String,
    pub tone: WidgetTone,
}

pub fn apply_creator_visuals(ctx: &egui::Context, theme: &ForgeTheme) {
    ctx.set_theme(egui::Theme::Dark);
    let mut style = (*ctx.style_of(egui::Theme::Dark)).clone();
    style.visuals = egui::Visuals::dark();
    style.visuals.panel_fill = color(theme.chrome.shell);
    style.visuals.window_fill = color(theme.base.panel_raised);
    style.visuals.extreme_bg_color = color(theme.chrome.canvas);
    style.visuals.selection.bg_fill = color(theme.base.panel_raised);
    style.visuals.selection.stroke = Stroke::new(1.0, color(theme.base.accent));
    style.visuals.widgets.noninteractive.bg_fill = color(theme.base.panel);
    style.visuals.widgets.inactive.bg_fill = color(theme.base.panel);
    style.visuals.widgets.hovered.bg_fill = color(theme.base.panel_raised);
    style.visuals.widgets.active.bg_fill = color(theme.base.panel_raised);
    style.spacing.item_spacing = egui::vec2(6.0, 6.0);
    style.spacing.button_padding = egui::vec2(9.0, 6.0);
    ctx.set_style_of(egui::Theme::Dark, style);
}

pub fn show_chrome_bar<R>(
    ui: &mut Ui,
    kind: ChromeBarKind,
    theme: &ForgeTheme,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> R {
    let fill = match kind {
        ChromeBarKind::Product => theme.chrome.product_bar,
        ChromeBarKind::Menu => theme.chrome.menu_bar,
        ChromeBarKind::Action => theme.chrome.action_bar,
        ChromeBarKind::WorkspaceTabs => theme.chrome.workspace_tabs,
        ChromeBarKind::BottomTray => theme.chrome.bottom_tray,
        ChromeBarKind::Status => theme.chrome.status_bar,
    };

    Frame::new()
        .fill(color(fill))
        .inner_margin(Margin::symmetric(8, 4))
        .show(ui, add_contents)
        .inner
}

pub fn show_product_identity(
    ui: &mut Ui,
    product: &str,
    project: &str,
    branch: Option<&str>,
    theme: &ForgeTheme,
) {
    show_chrome_bar(ui, ChromeBarKind::Product, theme, |ui| {
        ui.horizontal(|ui| {
            ui.strong(product);
            ui.separator();
            ui.label(RichText::new(project).color(color(theme.base.text)));
            if let Some(branch) = branch {
                ui.label(
                    RichText::new(format!("branch: {branch}"))
                        .small()
                        .color(color(theme.base.text_muted)),
                );
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(
                    RichText::new("Creator Studio")
                        .small()
                        .color(color(theme.base.accent)),
                );
            });
        });
    });
}

pub fn show_action_group(
    ui: &mut Ui,
    actions: &[(String, IconId, String, bool, WidgetTone)],
    theme: &ForgeTheme,
) -> Option<String> {
    let mut invoked = None;
    for (id, icon, label, active, tone) in actions {
        if tool_button(ui, *icon, label, *active, *tone, theme).clicked() {
            invoked = Some(id.clone());
        }
    }
    invoked
}

pub fn show_workspace_tabs(
    ui: &mut Ui,
    tabs: &[WorkspaceTab],
    active: &str,
    theme: &ForgeTheme,
) -> Option<String> {
    let mut selected = None;
    show_chrome_bar(ui, ChromeBarKind::WorkspaceTabs, theme, |ui| {
        ui.horizontal(|ui| {
            for tab in tabs {
                let is_active = tab.id == active;
                let mut label = tab.label.clone();
                if tab.dirty {
                    label.push_str("  •");
                }
                if let Some(icon) = tab.icon {
                    label = format!("{}  {label}", forge_gui_widgets::icon_text(icon));
                }
                if ui.selectable_label(is_active, label).clicked() {
                    selected = Some(tab.id.clone());
                }
            }
            ui.separator();
            let _ = compact_tool_button(ui, IconId::Add, "New workspace", false, theme);
        });
    });
    selected
}

pub fn show_tool_tray_tabs(
    ui: &mut Ui,
    tabs: &[ToolTrayTab],
    active: &str,
    theme: &ForgeTheme,
) -> Option<String> {
    let mut selected = None;
    show_chrome_bar(ui, ChromeBarKind::BottomTray, theme, |ui| {
        ui.horizontal(|ui| {
            for tab in tabs {
                let mut label = tab.label.clone();
                if let Some(badge) = tab.badge.as_deref() {
                    label.push_str(&format!("  {badge}"));
                }
                if ui.selectable_label(tab.id == active, label).clicked() {
                    selected = Some(tab.id.clone());
                }
            }
        });
    });
    selected
}

pub fn show_status_items(ui: &mut Ui, items: &[StatusItem], theme: &ForgeTheme) {
    show_chrome_bar(ui, ChromeBarKind::Status, theme, |ui| {
        ui.horizontal(|ui| {
            for item in items {
                let tone = match item.tone {
                    WidgetTone::Neutral => theme.base.text_muted,
                    WidgetTone::Accent => theme.base.accent,
                    WidgetTone::Success => theme.base.success,
                    WidgetTone::Warning => theme.base.warning,
                    WidgetTone::Danger => theme.base.danger,
                };
                ui.label(
                    RichText::new(format!("{}: {}", item.label, item.value))
                        .small()
                        .color(color(tone)),
                );
                ui.separator();
            }
        });
    });
}

fn color(value: forge_gui_core::Rgba) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(value.0, value.1, value.2, value.3)
}
