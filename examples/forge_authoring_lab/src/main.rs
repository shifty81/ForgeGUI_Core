use eframe::egui;
use forge_authoring_core::{AuthoringMode, AuthoringSession};
use forge_authoring_egui::show_authoring_surface;
use forge_gui_theme::ForgeTheme;
use forge_runtime_bridge::{PlayMode, PlayRequest, RuntimeBridge};
use forge_runtime_core::{InputSnapshot, RuntimeState};
use forge_scene_core::{EntityId, ForgeScene, SceneEntity, Transform, Transform2D, Transform3D};

fn main() -> eframe::Result {
    eframe::run_native(
        "Forge Universal Authoring Lab",
        eframe::NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(AuthoringLab::new(cc)))),
    )
}

struct AuthoringLab {
    theme: ForgeTheme,
    authoring: AuthoringSession,
    runtime: RuntimeBridge,
    status: String,
}

impl AuthoringLab {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let theme = ForgeTheme::forge_dark();

        cc.egui_ctx.set_theme(egui::Theme::Dark);
        let mut style = (*cc.egui_ctx.style_of(egui::Theme::Dark)).clone();
        style.visuals = egui::Visuals::dark();
        style.visuals.panel_fill = rgb(theme.base.panel);
        style.visuals.window_fill = rgb(theme.base.panel_raised);
        style.visuals.extreme_bg_color = rgb(theme.base.panel_recessed);
        style.visuals.selection.bg_fill = rgb(theme.base.panel_raised);
        style.visuals.selection.stroke = egui::Stroke::new(1.0, rgb(theme.base.accent));
        cc.egui_ctx.set_style_of(egui::Theme::Dark, style);

        let mut scene = ForgeScene::new("lab.scene", "Universal Authoring Demo");

        let mut player = SceneEntity::new(EntityId::from("player"), "Player");
        player.transform = Transform::ThreeD(Transform3D {
            translation: [0.0, 1.0, 0.0],
            ..Transform3D::default()
        });
        player.tags.insert("player".into());
        scene.insert_root(player).unwrap();

        let mut prop = SceneEntity::new(EntityId::from("workbench"), "Workbench");
        prop.transform = Transform::ThreeD(Transform3D {
            translation: [3.0, 0.5, 2.0],
            ..Transform3D::default()
        });
        scene.insert_root(prop).unwrap();

        let mut sprite = SceneEntity::new(EntityId::from("marker_2d"), "2D Marker");
        sprite.transform = Transform::TwoD(Transform2D {
            translation: [-120.0, 70.0],
            z_order: 4.0,
            ..Transform2D::default()
        });
        scene.insert_root(sprite).unwrap();

        Self {
            theme,
            authoring: AuthoringSession::new("lab.document", scene),
            runtime: RuntimeBridge::default(),
            status: "Ready".into(),
        }
    }

    fn play(&mut self, mode: PlayMode) {
        let request = PlayRequest {
            mode,
            start_position: if mode == PlayMode::PlayFromHere {
                Some([0.0, 1.0, 0.0])
            } else {
                None
            },
            preserve_runtime_changes: false,
        };

        match self.runtime.play(&self.authoring, request) {
            Ok(()) => self.status = format!("Runtime started: {mode:?}"),
            Err(error) => self.status = format!("Runtime error: {error}"),
        }
    }

    fn stop(&mut self) {
        match self.runtime.stop(Some(&mut self.authoring)) {
            Ok(()) => self.status = "Runtime stopped; authoring scene restored".into(),
            Err(error) => self.status = format!("Stop error: {error}"),
        }
    }
}

impl eframe::App for AuthoringLab {
    fn ui(&mut self, root: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if self.runtime.runtime.state == RuntimeState::Running {
            if let Err(error) = self.runtime.update(1.0 / 60.0, &InputSnapshot::default()) {
                self.status = format!("Runtime update error: {error}");
            }
        }

        egui::Panel::top("authoring_lab.toolbar")
            .exact_size(42.0)
            .show(root, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.strong("Universal Authoring Surface");
                    ui.separator();

                    if ui.button("Simulate").clicked() {
                        self.play(PlayMode::Simulate);
                    }
                    if ui.button("PIE").clicked() {
                        self.play(PlayMode::PlayInEditor);
                    }
                    if ui.button("Play From Here").clicked() {
                        self.play(PlayMode::PlayFromHere);
                    }
                    if ui.button("Headless").clicked() {
                        self.play(PlayMode::Headless);
                    }
                    if ui.button("Stop").clicked() {
                        self.stop();
                    }

                    ui.separator();

                    if ui.button("2D").clicked() {
                        self.authoring.surface.set_mode(AuthoringMode::TwoD);
                    }
                    if ui.button("3D").clicked() {
                        self.authoring.surface.set_mode(AuthoringMode::ThreeD);
                    }
                    if ui.button("Hybrid").clicked() {
                        self.authoring.surface.set_mode(AuthoringMode::Hybrid);
                    }
                });
            });

        egui::Panel::bottom("authoring_lab.status")
            .exact_size(28.0)
            .show(root, |ui| {
                ui.horizontal(|ui| {
                    ui.label(&self.status);
                    ui.separator();
                    ui.label(format!("Mode: {:?}", self.authoring.surface.mode));
                    ui.separator();
                    ui.label(format!("Runtime: {:?}", self.runtime.runtime.state));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label("ForgeGUI_Core / ForgeAuthoring / ForgeRuntime");
                    });
                });
            });

        egui::Panel::left("authoring_lab.outliner")
            .resizable(true)
            .default_size(220.0)
            .show(root, |ui| {
                ui.heading("Outliner");
                ui.separator();

                for entity in self.authoring.scene.entities.values() {
                    let selected = self
                        .authoring
                        .surface
                        .selection
                        .entities
                        .contains(&entity.id);

                    if ui.selectable_label(selected, &entity.name).clicked() {
                        self.authoring
                            .surface
                            .selection
                            .select_only(entity.id.clone());
                    }
                }
            });

        egui::Panel::right("authoring_lab.inspector")
            .resizable(true)
            .default_size(260.0)
            .show(root, |ui| {
                ui.heading("Authoring State");
                ui.separator();
                ui.label(format!(
                    "Selected: {}",
                    self.authoring
                        .surface
                        .selection
                        .primary
                        .as_ref()
                        .map(|id| id.as_str())
                        .unwrap_or("none")
                ));
                ui.label(format!("Gizmo: {:?}", self.authoring.surface.gizmo));
                ui.label(format!(
                    "Coordinate space: {:?}",
                    self.authoring.surface.coordinate_space
                ));
                ui.checkbox(&mut self.authoring.surface.snap.enabled, "Snapping");
                ui.checkbox(&mut self.authoring.surface.grid.visible, "Grid");
                ui.checkbox(&mut self.authoring.surface.show_guides, "Guides");
                ui.checkbox(
                    &mut self.authoring.surface.show_runtime_overlay,
                    "Runtime overlay",
                );
            });

        egui::CentralPanel::default().show(root, |ui| {
            let response = show_authoring_surface(ui, &mut self.authoring, &self.theme);
            if let Some(selected) = response.selected {
                self.status = format!("Selected {}", selected.as_str());
            } else if response.camera_changed {
                self.status = "Authoring camera changed".into();
            }
        });
    }
}

fn rgb(value: forge_gui_core::Rgba) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(value.0, value.1, value.2, value.3)
}
