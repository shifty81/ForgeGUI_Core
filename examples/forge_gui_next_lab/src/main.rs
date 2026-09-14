use eframe::egui;
use forge_gui_browser::{show_browser, BrowserItem, BrowserItemKind, BrowserModel};
use forge_gui_chrome::{
    apply_creator_visuals, show_action_group, show_chrome_bar, show_product_identity,
    show_status_items, show_tool_tray_tabs, show_workspace_tabs, ChromeBarKind, StatusItem,
    ToolTrayTab, WorkspaceTab,
};
use forge_gui_command::{show_command_palette, CommandPalette, PaletteEntry};
use forge_gui_core::{PropertyField, PropertyObject, PropertyValue};
use forge_gui_icons::IconId;
use forge_gui_inspector::show_property_object;
use forge_gui_layout::{ForgeLayoutState, LayoutPreset};
use forge_gui_panels::{show_panel, PanelHeaderAction, PanelKind};
use forge_gui_theme::ForgeTheme;
use forge_gui_widgets::{compact_tool_button, WidgetTone};
use forge_gui_workspace::{show_renderer_workspace, WorkspaceViewState};
use forge_render_core::{AuthoringCamera, NullRenderBackend, RenderRequest};
use forge_render_surface::{RenderFamily, RenderSurfaceDescriptor, RenderSurfaceHost};
use forge_scene_core::{ForgeScene, SceneEntity};

fn main() -> eframe::Result {
    eframe::run_native(
        "ForgeGUI Core Creator Studio Lab",
        eframe::NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(CreatorStudioLab::new(cc)))),
    )
}

struct CreatorStudioLab {
    theme: ForgeTheme,
    layout: ForgeLayoutState,
    workspace_tabs: Vec<WorkspaceTab>,
    tool_tabs: Vec<ToolTrayTab>,
    active_tool_tab: String,
    browser: BrowserModel,
    inspector_object: PropertyObject,
    palette: CommandPalette,
    workspace_view: WorkspaceViewState,
    render_surface: RenderSurfaceHost,
    scene: ForgeScene,
    last_action: String,
    runtime_playing: bool,
}

impl CreatorStudioLab {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let theme = ForgeTheme::forge_dark();
        apply_creator_visuals(&cc.egui_ctx, &theme);

        let mut scene = ForgeScene::new("creator.lab.scene", "Creator Studio Demo");
        scene
            .insert_root(SceneEntity::new("entity.world", "World Root"))
            .expect("demo world root is valid");
        scene
            .insert_root(SceneEntity::new("entity.player", "Player"))
            .expect("demo player is valid");

        Self {
            theme,
            layout: ForgeLayoutState::default(),
            workspace_tabs: workspace_tabs(),
            tool_tabs: tool_tabs(),
            active_tool_tab: "tool.console".into(),
            browser: browser_model(),
            inspector_object: inspector_object(),
            palette: command_palette(),
            workspace_view: WorkspaceViewState {
                selection_label: "Player".into(),
                ..Default::default()
            },
            render_surface: RenderSurfaceHost::new(
                RenderSurfaceDescriptor::new(
                    "surface.creator.main",
                    "Main Game View",
                    RenderFamily::Hybrid,
                ),
                NullRenderBackend::default(),
            ),
            scene,
            last_action: "Creator Studio ready".into(),
            runtime_playing: false,
        }
    }

    fn run_renderer(&mut self) {
        let request = RenderRequest {
            dimension: self.render_surface.descriptor().family.base_dimension(),
            extent: self.render_surface.extent(),
            camera: AuthoringCamera::default(),
            selected: Vec::new(),
            hovered: None,
            show_grid: self.workspace_view.show_grid,
            show_gizmos: true,
            show_editor_overlays: self.workspace_view.show_overlays,
        };

        if let Err(error) = self.render_surface.render(&self.scene, &request) {
            self.last_action = format!("Renderer: {error}");
        }
    }
}

impl eframe::App for CreatorStudioLab {
    fn ui(&mut self, root: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = root.ctx().clone();
        let metrics = self.theme.effective_metrics();

        if root.input(|i| i.modifiers.command && i.key_pressed(egui::Key::P)) {
            self.palette.open();
        }

        egui::Panel::top("forge.creator.product")
            .exact_size(metrics.product_bar_height)
            .show(root, |ui| {
                show_product_identity(
                    ui,
                    "ForgeGUI Core",
                    "Creator Studio / Universal Game Workspace",
                    Some("FG-C41…C55"),
                    &self.theme,
                );
            });

        egui::Panel::top("forge.creator.menu")
            .exact_size(metrics.menu_bar_height)
            .show(root, |ui| {
                show_chrome_bar(ui, ChromeBarKind::Menu, &self.theme, |ui| {
                    ui.horizontal(|ui| {
                        ui.menu_button("File", |ui| {
                            if ui.button("New Workspace").clicked() {
                                self.last_action = "New workspace requested".into();
                                ui.close();
                            }
                            if ui.button("Save Layout").clicked() {
                                self.last_action = "Layout save requested".into();
                                ui.close();
                            }
                        });
                        ui.menu_button("Edit", |ui| {
                            let _ = ui.button("Undo");
                            let _ = ui.button("Redo");
                        });
                        ui.menu_button("Assets", |ui| {
                            let _ = ui.button("Create Asset");
                            let _ = ui.button("Import");
                        });
                        ui.menu_button("Workspace", |ui| {
                            if ui.button("Creator Layout").clicked() {
                                self.layout = ForgeLayoutState::from_preset(LayoutPreset::Creator);
                                ui.close();
                            }
                            if ui.button("World Authoring Layout").clicked() {
                                self.layout =
                                    ForgeLayoutState::from_preset(LayoutPreset::WorldAuthoring);
                                ui.close();
                            }
                        });
                        ui.menu_button("Run", |ui| {
                            let _ = ui.button("Play");
                            let _ = ui.button("Simulate");
                            let _ = ui.button("Stop");
                        });
                        ui.menu_button("Help", |ui| {
                            ui.label("ForgeGUI Core Creator Studio Lab");
                        });
                    });
                });
            });

        egui::Panel::top("forge.creator.actions")
            .exact_size(metrics.action_bar_height)
            .show(root, |ui| {
                show_chrome_bar(ui, ChromeBarKind::Action, &self.theme, |ui| {
                    ui.horizontal(|ui| {
                        let file_actions = [
                            (
                                "file.save".into(),
                                IconId::Save,
                                "Save".into(),
                                false,
                                WidgetTone::Neutral,
                            ),
                            (
                                "edit.undo".into(),
                                IconId::Undo,
                                "Undo".into(),
                                false,
                                WidgetTone::Neutral,
                            ),
                            (
                                "edit.redo".into(),
                                IconId::Redo,
                                "Redo".into(),
                                false,
                                WidgetTone::Neutral,
                            ),
                        ];
                        if let Some(action) = show_action_group(ui, &file_actions, &self.theme) {
                            self.last_action = action;
                        }

                        ui.separator();

                        let play_label = if self.runtime_playing {
                            "Playing"
                        } else {
                            "Play"
                        };
                        let run_actions = [
                            (
                                "run.play".into(),
                                IconId::Play,
                                play_label.into(),
                                self.runtime_playing,
                                WidgetTone::Success,
                            ),
                            (
                                "run.pause".into(),
                                IconId::Pause,
                                "Pause".into(),
                                false,
                                WidgetTone::Warning,
                            ),
                            (
                                "run.stop".into(),
                                IconId::Stop,
                                "Stop".into(),
                                false,
                                WidgetTone::Danger,
                            ),
                        ];
                        if let Some(action) = show_action_group(ui, &run_actions, &self.theme) {
                            if action == "run.play" {
                                self.runtime_playing = true;
                            } else if action == "run.stop" {
                                self.runtime_playing = false;
                            }
                            self.last_action = action;
                        }

                        ui.separator();
                        if compact_tool_button(
                            ui,
                            IconId::Search,
                            "Command palette",
                            false,
                            &self.theme,
                        )
                        .clicked()
                        {
                            self.palette.open();
                        }
                        let _ = compact_tool_button(
                            ui,
                            IconId::Settings,
                            "Workspace settings",
                            false,
                            &self.theme,
                        );
                    });
                });
            });

        egui::Panel::top("forge.creator.workspace_tabs")
            .exact_size(metrics.workspace_tab_height)
            .show(root, |ui| {
                if let Some(selected) = show_workspace_tabs(
                    ui,
                    &self.workspace_tabs,
                    &self.layout.active_workspace,
                    &self.theme,
                ) {
                    self.layout.active_workspace = selected.clone();
                    self.last_action = format!("Workspace: {selected}");
                }
            });

        if self.layout.structural.status_visible {
            egui::Panel::bottom("forge.creator.status")
                .exact_size(metrics.status_bar_height)
                .show(root, |ui| {
                    let items = vec![
                        StatusItem {
                            label: "Renderer".into(),
                            value: self.render_surface.backend_id().into(),
                            tone: WidgetTone::Accent,
                        },
                        StatusItem {
                            label: "Mode".into(),
                            value: self.render_surface.descriptor().family.label().into(),
                            tone: WidgetTone::Neutral,
                        },
                        StatusItem {
                            label: "Frame".into(),
                            value: self.render_surface.frame_index().to_string(),
                            tone: WidgetTone::Neutral,
                        },
                        StatusItem {
                            label: "State".into(),
                            value: self.last_action.clone(),
                            tone: WidgetTone::Success,
                        },
                    ];
                    show_status_items(ui, &items, &self.theme);
                });
        }

        if self.layout.structural.bottom_visible {
            egui::Panel::bottom("forge.creator.bottom_tray")
                .resizable(true)
                .default_size(self.layout.structural.bottom_height)
                .size_range(96.0..=420.0)
                .show(root, |ui| {
                    if let Some(selected) =
                        show_tool_tray_tabs(ui, &self.tool_tabs, &self.active_tool_tab, &self.theme)
                    {
                        self.active_tool_tab = selected;
                    }
                    ui.separator();
                    show_bottom_tool(ui, &self.active_tool_tab, &self.last_action, &self.theme);
                });
        }

        if self.layout.structural.left_visible {
            egui::Panel::left("forge.creator.inspector")
                .resizable(true)
                .default_size(self.layout.structural.left_width)
                .size_range(240.0..=520.0)
                .show(root, |ui| {
                    let actions = vec![PanelHeaderAction {
                        id: "inspector.lock".into(),
                        icon: Some(IconId::Inspector),
                        label: "Lock".into(),
                        tooltip: "Pin inspector selection".into(),
                        active: false,
                    }];
                    let (inspector, header) = show_panel(
                        ui,
                        "Inspector",
                        Some("Selection"),
                        PanelKind::Inspector,
                        &actions,
                        &self.theme,
                        |ui| show_property_object(ui, &mut self.inspector_object, &self.theme),
                    );
                    if let Some(action) = header.invoked {
                        self.last_action = action;
                    }
                    if let Some(change) = inspector.changes.last() {
                        self.last_action =
                            format!("Changed {}.{}", change.object_id, change.property_id);
                    }
                });
        }

        if self.layout.structural.right_visible {
            egui::Panel::right("forge.creator.assets")
                .resizable(true)
                .default_size(self.layout.structural.right_width)
                .size_range(250.0..=560.0)
                .show(root, |ui| {
                    let actions = vec![
                        PanelHeaderAction {
                            id: "assets.add".into(),
                            icon: Some(IconId::Add),
                            label: "Add".into(),
                            tooltip: "Create or import asset".into(),
                            active: false,
                        },
                        PanelHeaderAction {
                            id: "assets.settings".into(),
                            icon: Some(IconId::Settings),
                            label: "Settings".into(),
                            tooltip: "Asset browser options".into(),
                            active: false,
                        },
                    ];
                    let (browser_response, header) = show_panel(
                        ui,
                        "Content",
                        Some("Project Assets"),
                        PanelKind::Assets,
                        &actions,
                        &self.theme,
                        |ui| show_browser(ui, &mut self.browser, &self.theme),
                    );
                    if let Some(action) = header.invoked {
                        self.last_action = action;
                    }
                    if let Some(asset) = browser_response.activated {
                        self.last_action = format!("Open asset: {asset}");
                    } else if browser_response.selection_changed {
                        self.last_action = "Asset selection changed".into();
                    }
                });
        }

        egui::CentralPanel::default().show(root, |ui| {
            let workspace_response = show_renderer_workspace(
                ui,
                &mut self.render_surface,
                &mut self.workspace_view,
                &self.theme,
                |ui, rect, surface| draw_renderer_preview(ui, rect, surface, &self.theme),
            );

            if let Some(family) = workspace_response.family_changed {
                self.workspace_view.camera_label = match family {
                    RenderFamily::TwoD => "Orthographic 2D".into(),
                    RenderFamily::TwoPointFiveD => "Layered Orthographic".into(),
                    RenderFamily::ThreeD => "Perspective 3D".into(),
                    RenderFamily::Voxel => "Voxel Perspective".into(),
                    RenderFamily::Hybrid => "Hybrid Camera".into(),
                };
                self.last_action = format!("Renderer family: {}", family.label());
            }
            if workspace_response.requested_focus {
                self.last_action = "Workspace focused".into();
            }
        });

        self.run_renderer();

        if let Some(invocation) = show_command_palette(&ctx, &mut self.palette, &self.theme) {
            self.last_action = format!("Command: {}", invocation.id);
        }
    }
}

fn draw_renderer_preview(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    surface: &RenderSurfaceHost,
    theme: &ForgeTheme,
) {
    let painter = ui.painter();
    let grid_color = egui::Color32::from_gray(35);
    let major_color = egui::Color32::from_gray(48);
    let step = 32.0;

    let mut x = rect.left();
    let mut column = 0usize;
    while x <= rect.right() {
        let stroke = if column.is_multiple_of(4) {
            egui::Stroke::new(1.0, major_color)
        } else {
            egui::Stroke::new(1.0, grid_color)
        };
        painter.line_segment(
            [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
            stroke,
        );
        x += step;
        column += 1;
    }

    let mut y = rect.top();
    let mut row = 0usize;
    while y <= rect.bottom() {
        let stroke = if row.is_multiple_of(4) {
            egui::Stroke::new(1.0, major_color)
        } else {
            egui::Stroke::new(1.0, grid_color)
        };
        painter.line_segment(
            [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
            stroke,
        );
        y += step;
        row += 1;
    }

    let center = rect.center();
    let player = egui::Rect::from_center_size(center, egui::vec2(120.0, 86.0));
    painter.rect_filled(player, 8.0, egui::Color32::from_rgb(38, 77, 67));
    painter.rect_stroke(
        player,
        8.0,
        egui::Stroke::new(2.0, color(theme.base.accent)),
        egui::StrokeKind::Inside,
    );
    painter.text(
        player.center(),
        egui::Align2::CENTER_CENTER,
        "Player",
        egui::FontId::proportional(16.0),
        egui::Color32::WHITE,
    );

    let camera = egui::Rect::from_min_size(
        egui::pos2(rect.left() + 90.0, rect.top() + 120.0),
        egui::vec2(190.0, 120.0),
    );
    painter.rect_stroke(
        camera,
        4.0,
        egui::Stroke::new(1.0, color(theme.base.warning)),
        egui::StrokeKind::Inside,
    );
    painter.text(
        camera.left_top() + egui::vec2(6.0, 6.0),
        egui::Align2::LEFT_TOP,
        "Camera Preview",
        egui::FontId::proportional(12.0),
        color(theme.base.warning),
    );

    if let Some(target) = surface.last_target() {
        painter.text(
            rect.right_bottom() + egui::vec2(-12.0, -10.0),
            egui::Align2::RIGHT_BOTTOM,
            format!("target: {}", target.id),
            egui::FontId::monospace(11.0),
            color(theme.base.text_muted),
        );
    }
}

fn show_bottom_tool(ui: &mut egui::Ui, active: &str, last_action: &str, theme: &ForgeTheme) {
    match active {
        "tool.console" => {
            ui.monospace("[PASS] Creator shell initialized");
            ui.monospace("[PASS] Renderer-backed workspace host attached");
            ui.monospace("[INFO] Current Lab backend: forge.render.null");
            ui.monospace(format!("[ACTION] {last_action}"));
        }
        "tool.problems" => {
            ui.label("Problems");
            ui.label(
                egui::RichText::new("No runtime problems in the Lab model")
                    .color(color(theme.base.success)),
            );
        }
        "tool.build" => {
            ui.label("Build / Certification");
            ui.monospace("PCC Full Gate remains the build authority.");
        }
        "tool.runtime" => {
            ui.label("Runtime / PIE");
            ui.monospace("Edit → Simulate → PIE → Detached → Standalone");
        }
        "tool.cortex" => {
            ui.label("Cortex");
            ui.monospace("Consumer-provided AI/tooling panel slot.");
        }
        _ => {
            ui.label("Tool tray");
        }
    }
}

fn workspace_tabs() -> Vec<WorkspaceTab> {
    vec![
        WorkspaceTab {
            id: "workspace.main".into(),
            label: "Main World".into(),
            icon: Some(IconId::World),
            dirty: true,
            closable: false,
        },
        WorkspaceTab {
            id: "workspace.ui".into(),
            label: "Interface".into(),
            icon: Some(IconId::Asset),
            dirty: false,
            closable: true,
        },
        WorkspaceTab {
            id: "workspace.logic".into(),
            label: "Logic".into(),
            icon: Some(IconId::Graph),
            dirty: false,
            closable: true,
        },
        WorkspaceTab {
            id: "workspace.animation".into(),
            label: "Animation".into(),
            icon: Some(IconId::Timeline),
            dirty: false,
            closable: true,
        },
    ]
}

fn tool_tabs() -> Vec<ToolTrayTab> {
    vec![
        ToolTrayTab::new("tool.console", "Console"),
        ToolTrayTab::new("tool.problems", "Problems"),
        ToolTrayTab::new("tool.build", "Build"),
        ToolTrayTab::new("tool.runtime", "Runtime / PIE"),
        ToolTrayTab::new("tool.cortex", "Cortex"),
    ]
}

fn browser_model() -> BrowserModel {
    BrowserModel {
        items: vec![
            BrowserItem::new("asset.scenes", "Scenes", BrowserItemKind::Folder),
            BrowserItem::new("asset.scene.main", "Main World", BrowserItemKind::Scene),
            BrowserItem::new("asset.objects", "Objects", BrowserItemKind::Folder),
            BrowserItem::new("asset.object.player", "Player", BrowserItemKind::Object),
            BrowserItem::new(
                "asset.object.ship",
                "Ship Prototype",
                BrowserItemKind::Object,
            ),
            BrowserItem::new("asset.sprites", "Sprites", BrowserItemKind::Folder),
            BrowserItem::new(
                "asset.sprite.player",
                "Player_Base",
                BrowserItemKind::Sprite,
            ),
            BrowserItem::new("asset.tilesets", "Tilesets", BrowserItemKind::Folder),
            BrowserItem::new("asset.audio", "Audio", BrowserItemKind::Folder),
            BrowserItem::new("asset.logic", "Logic", BrowserItemKind::Folder),
            BrowserItem::new(
                "asset.script.player",
                "Player Logic",
                BrowserItemKind::Script,
            ),
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
                id: "forge.command.workspace.new".into(),
                title: "New Workspace".into(),
                category: "Workspace".into(),
                shortcut: Some("Ctrl+N".into()),
            },
            PaletteEntry {
                id: "forge.command.document.save".into(),
                title: "Save Active Document".into(),
                category: "File".into(),
                shortcut: Some("Ctrl+S".into()),
            },
            PaletteEntry {
                id: "forge.command.run.play".into(),
                title: "Play In Editor".into(),
                category: "Run".into(),
                shortcut: Some("F6".into()),
            },
            PaletteEntry {
                id: "forge.command.layout.creator".into(),
                title: "Restore Creator Layout".into(),
                category: "Window".into(),
                shortcut: None,
            },
        ],
    }
}

fn color(value: forge_gui_core::Rgba) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(value.0, value.1, value.2, value.3)
}
