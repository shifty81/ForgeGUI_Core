//! Shared egui widget adapters owned by ForgeGUI.
//!
//! Third-party widget crates stay behind this layer so project code does not
//! serialize or persist their private types.

#![forbid(unsafe_code)]

use egui::{Button, Frame, Id, Margin, Response, RichText, Stroke, Ui, Vec2};
use egui_extras::{Column, TableBuilder};
use forge_gui_icons::IconId;
use forge_gui_theme::ForgeTheme;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WidgetTone {
    #[default]
    Neutral,
    Accent,
    Success,
    Warning,
    Danger,
}

pub fn add_default_icon_font(fonts: &mut egui::FontDefinitions) {
    egui_phosphor::add_to_fonts(fonts, egui_phosphor::Variant::Regular);
}

pub fn icon_text(icon: IconId) -> &'static str {
    use egui_phosphor::regular as p;
    match icon {
        IconId::Add => p::PLUS,
        IconId::Remove => p::MINUS,
        IconId::Close => p::X,
        IconId::Save => p::FLOPPY_DISK,
        IconId::Undo => p::ARROW_COUNTER_CLOCKWISE,
        IconId::Redo | IconId::Restart => p::ARROW_CLOCKWISE,
        IconId::Search => p::MAGNIFYING_GLASS,
        IconId::Settings | IconId::Build | IconId::Crafting => p::GEAR,
        IconId::Folder | IconId::Project => p::FOLDER,
        IconId::FolderOpen => p::FOLDER_OPEN,
        IconId::File => p::FILE,
        IconId::Asset | IconId::Package | IconId::Inventory => p::PACKAGE,
        IconId::Inspector => p::SLIDERS_HORIZONTAL,
        IconId::Layers => p::STACK,
        IconId::Console | IconId::Terminal | IconId::Cortex => p::TERMINAL,
        IconId::Play | IconId::Step => p::PLAY,
        IconId::Pause => p::PAUSE,
        IconId::Stop => p::STOP,
        IconId::Patch => p::GIT_DIFF,
        IconId::GitBranch => p::GIT_BRANCH,
        IconId::Success | IconId::Test => p::CHECK,
        IconId::Warning | IconId::Problems | IconId::Notifications | IconId::Info => p::INFO,
        IconId::Error => p::X,
        IconId::World => p::GLOBE,
        IconId::Scene | IconId::Entity | IconId::Ship | IconId::Module | IconId::Socket => p::CUBE,
        IconId::Graph => p::GRAPH,
        IconId::Timeline | IconId::History => p::STACK,
        _ => icon.fallback(),
    }
}

pub fn icon_button(ui: &mut Ui, icon: IconId, tooltip: &str) -> Response {
    ui.button(RichText::new(icon_text(icon)))
        .on_hover_text(tooltip)
}

pub fn labeled_icon_button(ui: &mut Ui, icon: IconId, label: &str) -> Response {
    ui.button(format!("{} {label}", icon_text(icon)))
}

pub fn tool_button(
    ui: &mut Ui,
    icon: IconId,
    label: &str,
    active: bool,
    tone: WidgetTone,
    theme: &ForgeTheme,
) -> Response {
    let metrics = theme.effective_metrics();
    let fill = if active {
        color(theme.base.panel_raised)
    } else {
        color(theme.chrome.action_bar)
    };
    let text = tone_color(tone, theme);
    let button = Button::new(
        RichText::new(format!("{}  {label}", icon_text(icon)))
            .color(color(text))
            .size(12.5),
    )
    .selected(active)
    .fill(fill)
    .min_size(Vec2::new(0.0, metrics.action_bar_height - 10.0))
    .corner_radius(3);
    ui.add(button)
}

pub fn compact_tool_button(
    ui: &mut Ui,
    icon: IconId,
    tooltip: &str,
    active: bool,
    theme: &ForgeTheme,
) -> Response {
    let metrics = theme.effective_metrics();
    let fill = if active {
        color(theme.base.panel_raised)
    } else {
        color(theme.chrome.action_bar)
    };
    ui.add(
        Button::new(RichText::new(icon_text(icon)).size(metrics.icon_size))
            .selected(active)
            .fill(fill)
            .min_size(Vec2::splat(metrics.action_bar_height - 10.0))
            .corner_radius(3),
    )
    .on_hover_text(tooltip)
}

pub fn chrome_tab(
    ui: &mut Ui,
    label: &str,
    active: bool,
    dirty: bool,
    theme: &ForgeTheme,
) -> Response {
    let text = if dirty {
        format!("{label}  •")
    } else {
        label.to_owned()
    };
    let fill = if active {
        color(theme.base.panel_raised)
    } else {
        color(theme.chrome.workspace_tabs)
    };
    ui.add(
        Button::new(RichText::new(text).size(12.5))
            .selected(active)
            .fill(fill)
            .min_size(Vec2::new(
                78.0,
                theme.effective_metrics().workspace_tab_height - 4.0,
            ))
            .corner_radius(2),
    )
}

pub fn panel_tab(ui: &mut Ui, label: &str, active: bool, theme: &ForgeTheme) -> Response {
    let fill = if active {
        color(theme.base.panel_raised)
    } else {
        color(theme.base.panel)
    };
    ui.add(
        Button::new(RichText::new(label).size(12.0))
            .selected(active)
            .fill(fill)
            .min_size(Vec2::new(
                62.0,
                theme.effective_metrics().bottom_tab_height - 4.0,
            ))
            .corner_radius(2),
    )
}

pub fn badge(ui: &mut Ui, text: &str, tone: WidgetTone, theme: &ForgeTheme) -> Response {
    let background = tone_color(tone, theme);
    let foreground = if matches!(tone, WidgetTone::Neutral) {
        theme.base.text
    } else {
        theme.base.background
    };
    ui.add(
        Button::new(RichText::new(text).small().color(color(foreground)))
            .fill(color(background))
            .corner_radius(8)
            .frame(true),
    )
}

pub fn search_box(ui: &mut Ui, value: &mut String, hint: &str, theme: &ForgeTheme) -> Response {
    Frame::new()
        .fill(color(theme.base.panel_recessed))
        .stroke(Stroke::new(1.0, color(theme.base.border)))
        .corner_radius(3)
        .inner_margin(Margin::symmetric(6, 3))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(icon_text(IconId::Search))
                        .color(color(theme.base.text_muted))
                        .size(12.0),
                );
                ui.add(
                    egui::TextEdit::singleline(value)
                        .hint_text(hint)
                        .desired_width(f32::INFINITY)
                        .frame(Frame::NONE),
                )
            })
            .inner
        })
        .inner
}

pub fn property_label(ui: &mut Ui, text: &str, theme: &ForgeTheme) {
    ui.label(
        RichText::new(text)
            .size(12.0)
            .color(color(theme.base.text_muted)),
    );
}

pub fn virtual_table(
    ui: &mut Ui,
    id: Id,
    headers: &[&str],
    row_count: usize,
    row_height: f32,
    mut render_cell: impl FnMut(&mut Ui, usize, usize),
) {
    if headers.is_empty() {
        return;
    }

    let mut table = TableBuilder::new(ui)
        .id_salt(id)
        .striped(true)
        .resizable(true)
        .vscroll(true);

    for _ in headers {
        table = table.column(Column::remainder().at_least(56.0));
    }

    table
        .header(22.0, |mut header| {
            for title in headers {
                header.col(|ui| {
                    ui.strong(*title);
                });
            }
        })
        .body(|body| {
            body.rows(row_height.max(1.0), row_count, |mut row| {
                let row_index = row.index();
                for column_index in 0..headers.len() {
                    row.col(|ui| render_cell(ui, row_index, column_index));
                }
            });
        });
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollbarAxis {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollbarState {
    /// Normalized scroll position in the range 0..=1.
    pub position: f32,
    /// Visible fraction of the content in the range 0..=1.
    pub viewport_fraction: f32,
}

impl Default for ScrollbarState {
    fn default() -> Self {
        Self {
            position: 0.0,
            viewport_fraction: 0.25,
        }
    }
}

impl ScrollbarState {
    pub fn clamp(&mut self) {
        self.position = self.position.clamp(0.0, 1.0);
        self.viewport_fraction = self.viewport_fraction.clamp(0.04, 1.0);
    }
}

/// Forge-owned progress bar so applications do not need to style toolkit-native
/// progress widgets individually.
pub fn progress_bar(
    ui: &mut Ui,
    fraction: f32,
    label: Option<&str>,
    tone: WidgetTone,
    theme: &ForgeTheme,
) -> Response {
    let fraction = fraction.clamp(0.0, 1.0);
    let desired = Vec2::new(ui.available_width().max(80.0), 18.0);
    let (rect, response) = ui.allocate_exact_size(desired, egui::Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, 4.0, color(theme.base.panel_recessed));
    painter.rect_stroke(
        rect,
        4.0,
        Stroke::new(1.0, color(theme.base.border)),
        egui::StrokeKind::Inside,
    );

    let fill_rect = egui::Rect::from_min_max(
        rect.min,
        egui::pos2(rect.left() + rect.width() * fraction, rect.bottom()),
    );
    painter.rect_filled(fill_rect, 4.0, color(tone_color(tone, theme)));

    let text = label
        .map(str::to_owned)
        .unwrap_or_else(|| format!("{}%", (fraction * 100.0).round() as i32));
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        egui::FontId::proportional(10.5),
        color(theme.base.text),
    );
    response
}

/// Compact circular gauge for health, load, temperature, compile progress, and
/// similar dashboard values. It is renderer-owned and theme-aware.
pub fn gauge(
    ui: &mut Ui,
    value: f32,
    min: f32,
    max: f32,
    label: &str,
    tone: WidgetTone,
    theme: &ForgeTheme,
) -> Response {
    let desired = Vec2::new(92.0, 82.0);
    let (rect, response) = ui.allocate_exact_size(desired, egui::Sense::hover());
    let painter = ui.painter();
    let center = egui::pos2(rect.center().x, rect.top() + 40.0);
    let radius = 29.0;
    let denominator = (max - min).abs().max(f32::EPSILON);
    let t = ((value - min) / denominator).clamp(0.0, 1.0);
    let start = std::f32::consts::PI * 0.75;
    let sweep = std::f32::consts::PI * 1.5;

    let background = gauge_arc_points(center, radius, start, start + sweep, 36);
    painter.add(egui::Shape::line(
        background,
        Stroke::new(5.0, color(theme.base.border)),
    ));
    let active = gauge_arc_points(center, radius, start, start + sweep * t, 36);
    if active.len() > 1 {
        painter.add(egui::Shape::line(
            active,
            Stroke::new(5.0, color(tone_color(tone, theme))),
        ));
    }

    painter.text(
        center,
        egui::Align2::CENTER_CENTER,
        format!("{value:.0}"),
        egui::FontId::proportional(16.0),
        color(theme.base.text),
    );
    painter.text(
        egui::pos2(rect.center().x, rect.bottom() - 3.0),
        egui::Align2::CENTER_BOTTOM,
        label,
        egui::FontId::proportional(10.5),
        color(theme.base.text_muted),
    );
    response
}

/// Minimal project-owned scrollbar primitive. It can be used independently or
/// embedded into custom virtualized views and infinite-canvas subregions.
pub fn scrollbar(
    ui: &mut Ui,
    axis: ScrollbarAxis,
    state: &mut ScrollbarState,
    theme: &ForgeTheme,
) -> Response {
    state.clamp();
    let width = theme.base.scrollbar_width.max(7.0);
    let desired = match axis {
        ScrollbarAxis::Horizontal => Vec2::new(ui.available_width().max(80.0), width),
        ScrollbarAxis::Vertical => Vec2::new(width, ui.available_height().max(80.0)),
    };
    let (rect, response) = ui.allocate_exact_size(desired, egui::Sense::click_and_drag());
    let painter = ui.painter();
    painter.rect_filled(rect, width * 0.5, color(theme.base.panel_recessed));

    let track_len = match axis {
        ScrollbarAxis::Horizontal => rect.width(),
        ScrollbarAxis::Vertical => rect.height(),
    };
    let thumb_len = (track_len * state.viewport_fraction).clamp(width * 1.6, track_len);
    let travel = (track_len - thumb_len).max(0.0);
    let thumb_start = travel * state.position;
    let thumb = match axis {
        ScrollbarAxis::Horizontal => egui::Rect::from_min_size(
            egui::pos2(rect.left() + thumb_start, rect.top()),
            Vec2::new(thumb_len, rect.height()),
        ),
        ScrollbarAxis::Vertical => egui::Rect::from_min_size(
            egui::pos2(rect.left(), rect.top() + thumb_start),
            Vec2::new(rect.width(), thumb_len),
        ),
    };
    let thumb_fill = if response.hovered() || response.dragged() {
        color(theme.base.text_muted)
    } else {
        color(theme.base.border)
    };
    painter.rect_filled(thumb.shrink(1.0), width * 0.5, thumb_fill);

    if response.dragged() && travel > 0.0 {
        let delta = ui.input(|i| i.pointer.delta());
        let d = match axis {
            ScrollbarAxis::Horizontal => delta.x,
            ScrollbarAxis::Vertical => delta.y,
        };
        state.position = (state.position + d / travel).clamp(0.0, 1.0);
    } else if response.clicked() && travel > 0.0 {
        if let Some(pointer) = response.interact_pointer_pos() {
            let p = match axis {
                ScrollbarAxis::Horizontal => pointer.x - rect.left(),
                ScrollbarAxis::Vertical => pointer.y - rect.top(),
            };
            state.position = ((p - thumb_len * 0.5) / travel).clamp(0.0, 1.0);
        }
    }

    response
}

/// A reusable text-first segmented choice for compact mode/tool selectors.
pub fn segmented_choice<T: Copy + PartialEq>(
    ui: &mut Ui,
    current: &mut T,
    options: &[(T, &str)],
    theme: &ForgeTheme,
) -> Option<T> {
    let mut changed = None;
    Frame::new()
        .fill(color(theme.base.panel_recessed))
        .stroke(Stroke::new(1.0, color(theme.base.border)))
        .corner_radius(4)
        .inner_margin(Margin::same(2))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                for (value, label) in options.iter().copied() {
                    if ui.selectable_label(*current == value, label).clicked() {
                        *current = value;
                        changed = Some(value);
                    }
                }
            });
        });
    changed
}

fn gauge_arc_points(
    center: egui::Pos2,
    radius: f32,
    start: f32,
    end: f32,
    segments: usize,
) -> Vec<egui::Pos2> {
    let segments = segments.max(2);
    (0..=segments)
        .map(|index| {
            let t = index as f32 / segments as f32;
            let angle = start + (end - start) * t;
            egui::pos2(
                center.x + angle.cos() * radius,
                center.y + angle.sin() * radius,
            )
        })
        .collect()
}

fn tone_color(tone: WidgetTone, theme: &ForgeTheme) -> forge_gui_theme::Rgba {
    match tone {
        WidgetTone::Neutral => theme.base.text_muted,
        WidgetTone::Accent => theme.base.accent,
        WidgetTone::Success => theme.base.success,
        WidgetTone::Warning => theme.base.warning,
        WidgetTone::Danger => theme.base.danger,
    }
}

fn color(value: forge_gui_theme::Rgba) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(value.0, value.1, value.2, value.3)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_icons_resolve_to_nonempty_adapter_text() {
        for id in [IconId::Play, IconId::Project, IconId::Cortex, IconId::Ship] {
            assert!(!icon_text(id).is_empty());
        }
    }

    #[test]
    fn semantic_icon_adapter_keeps_a_nonempty_forward_compatible_fallback() {
        assert!(!IconId::Info.fallback().is_empty());
    }

    #[test]
    fn scrollbar_state_clamps_to_valid_range() {
        let mut state = ScrollbarState {
            position: 2.0,
            viewport_fraction: 0.0,
        };
        state.clamp();
        assert_eq!(state.position, 1.0);
        assert!(state.viewport_fraction >= 0.04);
    }

    #[test]
    fn gauge_arc_builder_is_stable() {
        let points = gauge_arc_points(egui::pos2(0.0, 0.0), 10.0, 0.0, 1.0, 8);
        assert_eq!(points.len(), 9);
    }
}
