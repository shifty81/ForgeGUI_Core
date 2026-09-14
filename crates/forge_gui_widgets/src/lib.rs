//! Shared egui widget adapters owned by ForgeGUI.
//!
//! Third-party widget crates stay behind this layer so project code does not
//! serialize or persist their private types.

#![forbid(unsafe_code)]

use egui::{Id, Response, RichText, Ui};
use egui_extras::{Column, TableBuilder};
use forge_gui_icons::IconId;

/// Extend a host-owned font definition with the default ForgeGUI icon family.
///
/// The host keeps ownership of its font configuration; ForgeGUI does not reset
/// fonts behind the application's back.
pub fn add_default_icon_font(fonts: &mut egui::FontDefinitions) {
    egui_phosphor::add_to_fonts(fonts, egui_phosphor::Variant::Regular);
}

/// Resolve a semantic ForgeGUI icon to the current default Phosphor adapter.
///
/// This mapping is intentionally private to the widget/backend layer. Durable
/// project data stores `IconId`, not these codepoints.
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

/// Virtualized table adapter for large project/file/asset/log/data sets.
///
/// Only visible body rows are constructed by `egui_extras::TableBody::rows`.
/// Projects supply cell rendering without taking a dependency on
/// `egui_extras` table state or serialized layout internals.
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
        // IconId is #[non_exhaustive] by design. The adapter therefore keeps a
        // wildcard fallback so adding a future semantic icon cannot break every
        // downstream backend with an exhaustive-match compile error.
        assert!(!IconId::Info.fallback().is_empty());
    }
}
