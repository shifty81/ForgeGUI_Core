use eframe::egui;
use forge_gui_browser::{show_browser, BrowserItem, BrowserItemKind, BrowserModel};
use forge_gui_command::{show_command_palette, CommandPalette, PaletteEntry};
use forge_gui_core::{PropertyField, PropertyObject, PropertyValue};
use forge_gui_document_map::{show_document_map, DocumentMap, MapMarker, MarkerKind};
use forge_gui_inspector::show_property_object;
use forge_gui_rails::{
    show_dual_context, show_rail, LabelPolicy, RailItem, RailModel, RailPreset, RailSection,
    RailStyle,
};
use forge_gui_theme::ForgeTheme;
use forge_gui_workbench::{apply_workbench_visuals, BottomTrayMode, WorkbenchState};

fn main() -> eframe::Result {
    eframe::run_native(
        "ForgeGUI_Core Next Lab",
        eframe::NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(NextLab::new(cc)))),
    )
}

struct NextLab {
    theme: ForgeTheme,
    primary_rail: RailModel,
    context_rail: RailModel,
    browser: BrowserModel,
    inspector_object: PropertyObject,
    palette: CommandPalette,
    document_map: DocumentMap,
    workbench: WorkbenchState,
    last_action: String,
}

impl NextLab {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let theme = ForgeTheme::forge_dark();
        apply_workbench_visuals(&cc.egui_ctx, &theme);

        Self {
            theme,
            primary_rail: primary_rail(),
            context_rail: context_rail(),
            browser: browser_model(),
            inspector_object: inspector_object(),
            palette: command_palette(),
            document_map: document_map(),
            workbench: WorkbenchState::default(),
            last_action: "Ready".into(),
        }
    }

    fn set_rail_preset(&mut self, preset: RailPreset) {
        self.primary_rail.style.preset = preset;
        self.primary_rail.style.labels = match preset {
            RailPreset::SlimIcon => LabelPolicy::TooltipOnly,
            RailPreset::LabeledTool | RailPreset::DualContext | RailPreset::FloatingPill => {
                LabelPolicy::Always
            }
        };
    }
}

impl eframe::App for NextLab {
    fn ui(&mut self, root: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = root.ctx().clone();
        if root.input(|i| i.modifiers.command && i.key_pressed(egui::Key::P)) {
            self.palette.open();
        }

        egui::Panel::top("forge.next.title")
            .exact_size(36.0)
            .show(root, |ui| {
                ui.horizontal(|ui| {
                    ui.strong("ForgeGUI_Core");
                    ui.separator();
                    ui.label("Independent Library Certification Lab");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label("FG-C01…C20");
                    });
                });
            });

        egui::Panel::top("forge.next.toolbar")
            .exact_size(44.0)
            .show(root, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.menu_button("File", |_ui| {});
                    ui.menu_button("Edit", |_ui| {});
                    ui.menu_button("Assets", |_ui| {});
                    ui.menu_button("Build", |_ui| {});
                    ui.menu_button("Run", |_ui| {});
                    ui.separator();

                    if ui.button("▶ Run").clicked() {
                        self.last_action = "Run requested".into();
                    }
                    if ui.button("■ Stop").clicked() {
                        self.last_action = "Stop requested".into();
                    }

                    ui.separator();
                    egui::ComboBox::from_id_salt("rail.preset")
                        .selected_text(self.primary_rail.style.preset.label())
                        .show_ui(ui, |ui| {
                            for preset in RailPreset::ALL {
                                if ui
                                    .selectable_label(
                                        self.primary_rail.style.preset == preset,
                                        preset.label(),
                                    )
                                    .clicked()
                                {
                                    self.set_rail_preset(preset);
                                }
                            }
                        });

                    if ui.button("Command Palette").clicked() {
                        self.palette.open();
                    }

                    if ui
                        .selectable_label(self.workbench.focus_mode, "Focus")
                        .clicked()
                    {
                        self.workbench.toggle_focus_mode();
                    }
                });
            });

        egui::Panel::bottom("forge.next.status")
            .exact_size(self.workbench.config.status_height)
            .show(root, |ui| {
                ui.horizontal(|ui| {
                    ui.colored_label(
                        egui::Color32::from_rgb(
                            self.theme.base.success.0,
                            self.theme.base.success.1,
                            self.theme.base.success.2,
                        ),
                        "●",
                    );
                    ui.label(&self.last_action);
                    ui.separator();
                    ui.label("Main.scene");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label("ForgeGUI_Core 0.4.8 + next passes");
                    });
                });
            });

        if !self.workbench.focus_mode && self.workbench.config.show_bottom {
            egui::Panel::bottom("forge.next.bottom")
                .resizable(true)
                .default_size(self.workbench.bottom_height())
                .size_range(30.0..=600.0)
                .show(root, |ui| {
                    ui.horizontal(|ui| {
                        let _ = ui.selectable_label(true, "Console / Cortex");
                        let _ = ui.selectable_label(false, "Problems");
                        let _ = ui.selectable_label(false, "Build");
                        let _ = ui.selectable_label(false, "Timeline");
                        let _ = ui.selectable_label(false, "Git");

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let label = match self.workbench.bottom_tray {
                                BottomTrayMode::Collapsed => "Expand",
                                BottomTrayMode::Normal => "Expand",
                                BottomTrayMode::Expanded => "Collapse",
                            };
                            if ui.small_button(label).clicked() {
                                self.workbench.cycle_bottom_tray();
                            }
                        });
                    });
                    ui.separator();

                    if self.workbench.bottom_tray != BottomTrayMode::Collapsed {
                        egui::ScrollArea::vertical()
                            .stick_to_bottom(true)
                            .show(ui, |ui| {
                                ui.monospace("[PASS] ForgeGUI_Core independent library boundary");
                                ui.monospace("[PASS] Four shared rail configurations");
                                ui.monospace("[PASS] Theme / surface tokens");
                                ui.monospace("[PASS] Virtualized asset browser");
                                ui.monospace("[PASS] Shared property-grid renderer");
                                ui.monospace("[PASS] Command palette foundation");
                                ui.monospace("[PASS] Document-map marker lane");
                                ui.monospace(
                                    "[NEXT] egui_dock workbench integration + persistence",
                                );
                            });
                    }
                });
        }

        if !self.workbench.focus_mode && self.workbench.config.show_left {
            egui::Panel::left("forge.next.left")
                .resizable(true)
                .default_size(350.0)
                .size_range(64.0..=520.0)
                .show(root, |ui| {
                    ui.horizontal(|ui| {
                        if self.primary_rail.style.preset == RailPreset::DualContext {
                            let (primary, secondary) = show_dual_context(
                                ui,
                                &mut self.primary_rail,
                                &mut self.context_rail,
                                &self.theme,
                            );
                            if let Some(id) = primary.activated.or(secondary.activated) {
                                self.last_action = format!("Rail: {id}");
                            }
                        } else {
                            let rail = show_rail(ui, &mut self.primary_rail, &self.theme);
                            if let Some(id) = rail.activated {
                                self.last_action = format!("Rail: {id}");
                            }

                            ui.separator();
                            ui.vertical(|ui| {
                                ui.set_min_width(245.0);
                                let browser = show_browser(ui, &mut self.browser, &self.theme);
                                if let Some(id) = browser.activated {
                                    self.last_action = format!("Open asset: {id}");
                                }
                            });
                        }
                    });
                });
        }

        if !self.workbench.focus_mode && self.workbench.config.show_right {
            egui::Panel::right("forge.next.right")
                .resizable(true)
                .default_size(self.workbench.config.right_width)
                .size_range(220.0..=640.0)
                .show(root, |ui| {
                    let inspector =
                        show_property_object(ui, &mut self.inspector_object, &self.theme);
                    if let Some(change) = inspector.changes.last() {
                        self.last_action =
                            format!("Changed {}.{}", change.object_id, change.property_id);
                    }
                });
        }

        egui::CentralPanel::default().show(root, |ui| {
            ui.horizontal(|ui| {
                let _ = ui.selectable_label(true, "Main.scene  •");
                let _ = ui.selectable_label(false, "Player.object");
                let _ = ui.selectable_label(false, "Player.sprite");
                ui.separator();
                ui.label("Scene Editor");
            });
            ui.separator();

            let available_height = ui.available_height().max(120.0);
            ui.horizontal(|ui| {
                let map_width = 28.0;
                let canvas_width = (ui.available_width() - map_width).max(100.0);

                ui.allocate_ui_with_layout(
                    egui::vec2(canvas_width, available_height),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        draw_scene_canvas(ui, &self.theme);
                    },
                );

                if let Some(position) = show_document_map(
                    ui,
                    &mut self.document_map,
                    available_height - 8.0,
                    &self.theme,
                ) {
                    self.last_action = format!("Document map jump: {:.0}%", position * 100.0);
                }
            });
        });

        if let Some(invocation) = show_command_palette(&ctx, &mut self.palette, &self.theme) {
            self.last_action = format!("Command: {}", invocation.id);
        }
    }
}

fn draw_scene_canvas(ui: &mut egui::Ui, theme: &ForgeTheme) {
    ui.horizontal(|ui| {
        let _ = ui.button("Select");
        let _ = ui.button("Move");
        let _ = ui.button("Rotate");
        let _ = ui.button("Scale");
        ui.separator();
        ui.label("Grid 32");
        ui.label("Snap On");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label("100%");
        });
    });
    ui.separator();

    let size = ui.available_size();
    let (rect, _response) = ui.allocate_exact_size(size, egui::Sense::click_and_drag());
    let bg = egui::Color32::from_rgb(
        theme.base.background.0,
        theme.base.background.1,
        theme.base.background.2,
    );
    ui.painter().rect_filled(rect, 8.0, bg);

    let grid = 32.0;
    let line = egui::Stroke::new(1.0, egui::Color32::from_gray(38));

    let mut x = rect.left();
    while x <= rect.right() {
        ui.painter().line_segment(
            [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
            line,
        );
        x += grid;
    }

    let mut y = rect.top();
    while y <= rect.bottom() {
        ui.painter().line_segment(
            [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
            line,
        );
        y += grid;
    }

    let center = rect.center();
    let object_rect = egui::Rect::from_center_size(center, egui::vec2(150.0, 110.0));
    ui.painter()
        .rect_filled(object_rect, 8.0, egui::Color32::from_rgb(40, 83, 70));
    ui.painter().rect_stroke(
        object_rect,
        8.0,
        egui::Stroke::new(
            2.0,
            egui::Color32::from_rgb(
                theme.base.accent.0,
                theme.base.accent.1,
                theme.base.accent.2,
            ),
        ),
        egui::StrokeKind::Inside,
    );
    ui.painter().text(
        center,
        egui::Align2::CENTER_CENTER,
        "Player",
        egui::FontId::proportional(16.0),
        egui::Color32::WHITE,
    );
}

fn primary_rail() -> RailModel {
    let mut model = RailModel::new(
        "forge.rail.primary",
        RailStyle {
            preset: RailPreset::SlimIcon,
            labels: LabelPolicy::TooltipOnly,
            ..RailStyle::default()
        },
    );
    model.active = Some("forge.tool.objects".into());
    model.sections = vec![
        RailSection {
            label: Some("Create".into()),
            items: vec![
                RailItem::new("forge.tool.objects", "◇", "Objects").tooltip("Objects and entities"),
                RailItem::new("forge.tool.rooms", "▦", "Rooms").tooltip("Rooms and scenes"),
                RailItem::new("forge.tool.tilesets", "▤", "Tilesets"),
                RailItem::new("forge.tool.logic", "{ }", "Logic"),
            ],
        },
        RailSection {
            label: Some("Assets".into()),
            items: vec![
                RailItem::new("forge.tool.sprites", "▧", "Sprites"),
                RailItem::new("forge.tool.audio", "♪", "Audio").badge("2"),
                RailItem::new("forge.tool.timeline", "◷", "Timeline"),
            ],
        },
        RailSection {
            label: Some("System".into()),
            items: vec![
                RailItem::new("forge.tool.project", "□", "Project"),
                RailItem::new("forge.tool.settings", "⚙", "Settings"),
            ],
        },
    ];
    model
}

fn context_rail() -> RailModel {
    let mut model = RailModel::new(
        "forge.rail.context",
        RailStyle {
            preset: RailPreset::LabeledTool,
            labels: LabelPolicy::Always,
            ..RailStyle::default()
        },
    );
    model.active = Some("forge.context.general".into());
    model.sections = vec![RailSection {
        label: Some("Object".into()),
        items: vec![
            RailItem::new("forge.context.general", "▦", "General"),
            RailItem::new("forge.context.sprite", "▧", "Sprite"),
            RailItem::new("forge.context.physics", "◇", "Physics"),
            RailItem::new("forge.context.events", "⚑", "Events"),
            RailItem::new("forge.context.variables", "{ }", "Variables"),
            RailItem::new("forge.context.notes", "≡", "Notes"),
        ],
    }];
    model
}

fn browser_model() -> BrowserModel {
    BrowserModel {
        items: vec![
            BrowserItem::new("asset.scenes", "Scenes", BrowserItemKind::Folder),
            BrowserItem::new("asset.scene.main", "Main", BrowserItemKind::Scene),
            BrowserItem::new("asset.objects", "Objects", BrowserItemKind::Folder),
            BrowserItem::new("asset.object.player", "Player", BrowserItemKind::Object),
            BrowserItem::new(
                "asset.object.workbench",
                "Workbench",
                BrowserItemKind::Object,
            ),
            BrowserItem::new("asset.sprites", "Sprites", BrowserItemKind::Folder),
            BrowserItem::new(
                "asset.sprite.player",
                "Player_Base",
                BrowserItemKind::Sprite,
            ),
            BrowserItem::new("asset.audio", "Audio", BrowserItemKind::Folder),
        ],
        ..Default::default()
    }
}

fn inspector_object() -> PropertyObject {
    PropertyObject {
        object_id: "object.player".into(),
        title: "Player".into(),
        fields: vec![
            PropertyField {
                id: "enabled".into(),
                label: "Enabled".into(),
                value: PropertyValue::Bool(true),
                read_only: false,
            },
            PropertyField {
                id: "position_x".into(),
                label: "Position X".into(),
                value: PropertyValue::Float(128.0),
                read_only: false,
            },
            PropertyField {
                id: "position_y".into(),
                label: "Position Y".into(),
                value: PropertyValue::Float(96.0),
                read_only: false,
            },
            PropertyField {
                id: "sprite".into(),
                label: "Sprite".into(),
                value: PropertyValue::Reference("asset.sprite.player".into()),
                read_only: false,
            },
            PropertyField {
                id: "runtime_id".into(),
                label: "Runtime ID".into(),
                value: PropertyValue::Text("player.main".into()),
                read_only: true,
            },
        ],
    }
}

fn command_palette() -> CommandPalette {
    CommandPalette {
        open: false,
        query: String::new(),
        selected: 0,
        entries: vec![
            PaletteEntry {
                id: "forge.command.project.open".into(),
                title: "Open Project".into(),
                category: "Project".into(),
                shortcut: Some("Ctrl+O".into()),
            },
            PaletteEntry {
                id: "forge.command.document.save".into(),
                title: "Save Active Document".into(),
                category: "File".into(),
                shortcut: Some("Ctrl+S".into()),
            },
            PaletteEntry {
                id: "forge.command.run.play".into(),
                title: "Play / Run".into(),
                category: "Run".into(),
                shortcut: Some("F6".into()),
            },
            PaletteEntry {
                id: "forge.command.layout.focus".into(),
                title: "Toggle Focus Mode".into(),
                category: "Window".into(),
                shortcut: Some("Ctrl+Shift+F".into()),
            },
        ],
    }
}

fn document_map() -> DocumentMap {
    DocumentMap {
        markers: vec![
            MapMarker {
                position: 0.12,
                kind: MarkerKind::Bookmark,
            },
            MapMarker {
                position: 0.34,
                kind: MarkerKind::Search,
            },
            MapMarker {
                position: 0.62,
                kind: MarkerKind::Warning,
            },
            MapMarker {
                position: 0.81,
                kind: MarkerKind::Error,
            },
        ],
        ..Default::default()
    }
}
