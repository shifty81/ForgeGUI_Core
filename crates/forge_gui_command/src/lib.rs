//! Command palette and quick-action surfaces for ForgeGUI_Core.
#![forbid(unsafe_code)]

use egui::{Key, RichText};
use forge_gui_core::CommandCatalog;
use forge_gui_theme::ForgeTheme;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaletteEntry {
    pub id: String,
    pub title: String,
    pub category: String,
    pub shortcut: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandInvocation {
    pub id: String,
}

#[derive(Clone, Debug, Default)]
pub struct CommandPalette {
    pub open: bool,
    pub query: String,
    pub entries: Vec<PaletteEntry>,
    pub selected: usize,
}

impl CommandPalette {
    pub fn from_catalog(catalog: &CommandCatalog) -> Self {
        let entries = catalog
            .iter()
            .map(|command| PaletteEntry {
                id: command.id.to_string(),
                title: command.title.clone(),
                category: command.category.clone(),
                shortcut: command.default_shortcut.clone(),
            })
            .collect();

        Self {
            open: false,
            query: String::new(),
            entries,
            selected: 0,
        }
    }

    pub fn open(&mut self) {
        self.open = true;
        self.query.clear();
        self.selected = 0;
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    pub fn filtered(&self) -> Vec<&PaletteEntry> {
        let query = self.query.trim().to_lowercase();

        self.entries
            .iter()
            .filter(|entry| {
                query.is_empty()
                    || entry.title.to_lowercase().contains(&query)
                    || entry.category.to_lowercase().contains(&query)
                    || entry.id.to_lowercase().contains(&query)
            })
            .collect()
    }
}

pub fn show_command_palette(
    ctx: &egui::Context,
    palette: &mut CommandPalette,
    theme: &ForgeTheme,
) -> Option<CommandInvocation> {
    if !palette.open {
        return None;
    }

    if ctx.input(|i| i.key_pressed(Key::Escape)) {
        palette.close();
        return None;
    }

    let mut invocation = None;

    egui::Window::new("Command Palette")
        .id(egui::Id::new("forge.command.palette"))
        .collapsible(false)
        .resizable(true)
        .default_width(620.0)
        .default_height(420.0)
        .anchor(egui::Align2::CENTER_TOP, [0.0, 90.0])
        .show(ctx, |ui| {
            ui.add(
                egui::TextEdit::singleline(&mut palette.query)
                    .hint_text("Type a command, panel, asset, or action…")
                    .desired_width(f32::INFINITY),
            );
            ui.separator();

            let filtered: Vec<PaletteEntry> = palette.filtered().into_iter().cloned().collect();

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for entry in filtered {
                        let clicked = ui
                            .horizontal(|ui| {
                                let clicked = ui
                                    .selectable_label(false, RichText::new(&entry.title).strong())
                                    .clicked();

                                ui.label(
                                    RichText::new(&entry.category)
                                        .small()
                                        .color(color(theme.base.text_muted)),
                                );

                                if let Some(shortcut) = entry.shortcut.as_deref() {
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            ui.label(
                                                RichText::new(shortcut)
                                                    .small()
                                                    .color(color(theme.base.text_muted)),
                                            );
                                        },
                                    );
                                }

                                clicked
                            })
                            .inner;

                        if clicked {
                            invocation = Some(CommandInvocation {
                                id: entry.id.clone(),
                            });
                        }
                    }
                });
        });

    if invocation.is_some() {
        palette.close();
    }

    invocation
}

fn color(value: forge_gui_core::Rgba) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(value.0, value.1, value.2, value.3)
}
