//! Reusable project/asset browser primitives for ForgeGUI Core.
#![forbid(unsafe_code)]

use egui::{Button, Frame, Margin, RichText, ScrollArea, Stroke, Ui};
use forge_gui_theme::ForgeTheme;
use forge_gui_widgets::{icon_text, IconId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum BrowserViewMode {
    Tree,
    List,
    Grid,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum BrowserItemKind {
    Folder,
    Scene,
    Object,
    Sprite,
    Tileset,
    Audio,
    Script,
    Shader,
    Data,
    Other,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BrowserItem {
    pub id: String,
    pub label: String,
    pub kind: BrowserItemKind,
    pub secondary: Option<String>,
    pub parent: Option<String>,
    pub favorite: bool,
    pub dirty: bool,
    pub warning: bool,
}

impl BrowserItem {
    pub fn new(id: impl Into<String>, label: impl Into<String>, kind: BrowserItemKind) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            kind,
            secondary: None,
            parent: None,
            favorite: false,
            dirty: false,
            warning: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BrowserModel {
    pub query: String,
    pub view_mode: BrowserViewMode,
    pub items: Vec<BrowserItem>,
    pub selected: BTreeSet<String>,
    pub favorites_only: bool,
}

impl Default for BrowserModel {
    fn default() -> Self {
        Self {
            query: String::new(),
            view_mode: BrowserViewMode::Tree,
            items: Vec::new(),
            selected: BTreeSet::new(),
            favorites_only: false,
        }
    }
}

impl BrowserModel {
    pub fn filtered_indices(&self) -> Vec<usize> {
        let needle = self.query.trim().to_lowercase();

        self.items
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                (!self.favorites_only || item.favorite)
                    && (needle.is_empty()
                        || item.label.to_lowercase().contains(&needle)
                        || item
                            .secondary
                            .as_deref()
                            .unwrap_or_default()
                            .to_lowercase()
                            .contains(&needle))
            })
            .map(|(index, _)| index)
            .collect()
    }

    pub fn set_single_selection(&mut self, id: impl Into<String>) {
        self.selected.clear();
        self.selected.insert(id.into());
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct BrowserResponse {
    pub activated: Option<String>,
    pub selection_changed: bool,
}

pub fn show_browser(ui: &mut Ui, model: &mut BrowserModel, theme: &ForgeTheme) -> BrowserResponse {
    let mut response = BrowserResponse::default();
    let metrics = theme.effective_metrics();

    Frame::new()
        .fill(color(theme.base.panel_recessed))
        .stroke(Stroke::new(1.0, color(theme.chrome.separator)))
        .corner_radius(2)
        .inner_margin(Margin::symmetric(5, 3))
        .show(ui, |ui| {
            let _ =
                forge_gui_widgets::search_box(ui, &mut model.query, "Search project assets", theme);
        });

    ui.add_space(3.0);
    ui.horizontal(|ui| {
        let favorite_label = if model.favorites_only {
            "★ Favorites"
        } else {
            "☆ Favorites"
        };
        if ui
            .add(
                Button::new(favorite_label)
                    .selected(model.favorites_only)
                    .corner_radius(2),
            )
            .clicked()
        {
            model.favorites_only = !model.favorites_only;
        }
        ui.separator();
        ui.selectable_value(&mut model.view_mode, BrowserViewMode::Tree, "Tree");
        ui.selectable_value(&mut model.view_mode, BrowserViewMode::List, "List");
        ui.selectable_value(&mut model.view_mode, BrowserViewMode::Grid, "Grid");
    });

    ui.add_space(3.0);
    Frame::new()
        .fill(color(theme.base.panel_recessed))
        .inner_margin(Margin::symmetric(6, 3))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("PROJECT").small().strong());
                ui.label(
                    RichText::new(format!("{} items", model.filtered_indices().len()))
                        .small()
                        .color(color(theme.base.text_muted)),
                );
            });
        });

    let filtered = model.filtered_indices();
    let row_height = metrics.asset_row_height.max(22.0);

    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show_rows(ui, row_height, filtered.len(), |ui, rows| {
            for visible_row in rows {
                let item_index = filtered[visible_row];
                let item = model.items[item_index].clone();
                let selected = model.selected.contains(&item.id);
                let depth = if item.parent.is_some() { 1 } else { 0 };

                let fill = if selected {
                    color(theme.base.panel_raised)
                } else if visible_row.is_multiple_of(2) {
                    color(theme.base.panel)
                } else {
                    color(theme.base.background)
                };

                Frame::new()
                    .fill(fill)
                    .corner_radius(1)
                    .inner_margin(Margin::symmetric(5, 2))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.add_space(depth as f32 * 12.0);
                            let icon = icon_text(kind_icon(item.kind));
                            ui.label(RichText::new(icon).size(12.0).color(color(
                                if matches!(item.kind, BrowserItemKind::Folder) {
                                    theme.base.warning
                                } else {
                                    theme.base.text_muted
                                },
                            )));

                            let mut text = item.label.clone();
                            if item.dirty {
                                text.push_str("  •");
                            }
                            let label = if item.warning {
                                RichText::new(text).color(color(theme.base.warning))
                            } else {
                                RichText::new(text)
                            };

                            let item_response = ui.selectable_label(selected, label);
                            let item_response = if let Some(secondary) = item.secondary.as_deref() {
                                item_response.on_hover_text(secondary)
                            } else {
                                item_response
                            };

                            if item.favorite {
                                ui.label(
                                    RichText::new("★").small().color(color(theme.base.warning)),
                                );
                            }

                            if item_response.clicked() {
                                let modifiers = ui.input(|i| i.modifiers);
                                if modifiers.command || modifiers.ctrl {
                                    if selected {
                                        model.selected.remove(&item.id);
                                    } else {
                                        model.selected.insert(item.id.clone());
                                    }
                                } else {
                                    model.set_single_selection(item.id.clone());
                                }
                                response.selection_changed = true;
                            }

                            if item_response.double_clicked() {
                                response.activated = Some(item.id.clone());
                            }
                        });
                    });
            }
        });

    response
}

fn kind_icon(kind: BrowserItemKind) -> IconId {
    match kind {
        BrowserItemKind::Folder => IconId::Folder,
        BrowserItemKind::Scene => IconId::Scene,
        BrowserItemKind::Object => IconId::Entity,
        BrowserItemKind::Sprite => IconId::Asset,
        BrowserItemKind::Tileset => IconId::Layers,
        BrowserItemKind::Audio => IconId::Play,
        BrowserItemKind::Script => IconId::File,
        BrowserItemKind::Shader => IconId::Graph,
        BrowserItemKind::Data => IconId::Inventory,
        BrowserItemKind::Other => IconId::File,
    }
}

fn color(value: forge_gui_core::Rgba) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(value.0, value.1, value.2, value.3)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filtering_is_case_insensitive() {
        let mut model = BrowserModel::default();
        model.items.push(BrowserItem::new(
            "asset.player",
            "PlayerSprite",
            BrowserItemKind::Sprite,
        ));
        model.query = "players".into();
        assert_eq!(model.filtered_indices(), vec![0]);
    }
}
