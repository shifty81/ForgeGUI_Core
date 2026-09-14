mod panels;
use forge_gui::{egui, GuiEvent, IconId, PanelId};
use panels::*;
use std::sync::{Arc, Mutex};
struct LabApp {
    gui: forge_gui::ForgeGuiRuntime,
    shared: Arc<Mutex<DemoState>>,
    status: String,
    command_palette: bool,
}
impl LabApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut gui = forge_gui::ForgeGuiRuntime::default();
        let mut fonts = egui::FontDefinitions::default();
        forge_gui::widgets::add_default_icon_font(&mut fonts);
        cc.egui_ctx.set_fonts(fonts);
        gui.install_theme(&cc.egui_ctx);
        let shared = Arc::new(Mutex::new(DemoState::default()));
        register_demo_panels(&mut gui, shared.clone()).expect("panel registration");
        for id in [
            CANVAS, PROJECT, ASSETS, INSPECTOR, LAYERS, CONSOLE, PROBLEMS,
        ] {
            gui.open_panel(PanelId::from(id));
        }
        Self {
            gui,
            shared,
            status: "Ready".into(),
            command_palette: false,
        }
    }
    fn open(&mut self, id: &str) {
        self.gui.open_panel(PanelId::from(id));
    }
    fn command(&mut self, id: &str) {
        match id {
            "forgegui.play" => {
                if let Ok(mut s) = self.shared.lock() {
                    s.pie.play(forge_gui::PlayRequest {
                        mode: forge_gui::PlayMode::Normal,
                        document_id: Some("demo.scene".into()),
                    });
                }
                self.status = "PIE started".into();
            }
            "forgegui.play_here" => {
                if let Ok(mut s) = self.shared.lock() {
                    s.runtime_player = [320.0, 240.0];
                    s.pie.play(forge_gui::PlayRequest {
                        mode: forge_gui::PlayMode::FromHere,
                        document_id: Some("demo.scene".into()),
                    });
                }
                self.status = "PIE started from cursor/demo anchor".into();
            }
            "forgegui.pause" => {
                if let Ok(mut s) = self.shared.lock() {
                    if s.pie.status.state == forge_gui::PieState::Playing {
                        s.pie.pause();
                        self.status = "PIE paused".into();
                    } else if s.pie.status.state == forge_gui::PieState::Paused {
                        s.pie.resume();
                        self.status = "PIE resumed".into();
                    }
                }
            }
            "forgegui.step" => {
                if let Ok(mut s) = self.shared.lock() {
                    s.pie.step();
                }
                self.status = "PIE stepped".into();
            }
            "forgegui.restart" => {
                if let Ok(mut s) = self.shared.lock() {
                    let request = s
                        .pie
                        .last_request
                        .clone()
                        .unwrap_or(forge_gui::PlayRequest {
                            mode: forge_gui::PlayMode::Normal,
                            document_id: Some("demo.scene".into()),
                        });
                    s.pie.stop();
                    s.pie.play(request);
                }
                self.status = "PIE restarted".into();
            }
            "forgegui.stop" => {
                if let Ok(mut s) = self.shared.lock() {
                    s.pie.stop();
                }
                self.status = "PIE stopped".into();
            }
            _ => self.status = format!("Command {id}"),
        }
    }
    fn top_menu(&mut self, root: &mut egui::Ui) {
        egui::Panel::top("forgegui-menu")
            .exact_size(28.0)
            .show(root, |ui| {
                ui.horizontal_centered(|ui| {
                    for item in ["File", "Edit", "View", "Build", "Settings", "Help"] {
                        if ui.button(item).clicked() {
                            self.status = format!("{item} menu");
                        }
                    }
                    ui.separator();
                    if ui.button("Command Palette").clicked() {
                        self.command_palette = true;
                    }
                });
            });
    }
    fn toolbar(&mut self, root: &mut egui::Ui) {
        egui::Panel::top("forgegui-toolbar")
            .exact_size(self.gui.theme.toolbar_height)
            .show(root, |ui| {
                ui.horizontal_centered(|ui| {
                    if forge_gui::widgets::icon_button(ui, IconId::Play, "Play").clicked() {
                        self.command("forgegui.play");
                    }
                    if ui.button("From Here").clicked() {
                        self.command("forgegui.play_here");
                    }
                    if forge_gui::widgets::icon_button(ui, IconId::Pause, "Pause / Resume")
                        .clicked()
                    {
                        self.command("forgegui.pause");
                    }
                    if forge_gui::widgets::icon_button(ui, IconId::Step, "Step").clicked() {
                        self.command("forgegui.step");
                    }
                    if forge_gui::widgets::icon_button(ui, IconId::Restart, "Restart").clicked() {
                        self.command("forgegui.restart");
                    }
                    if forge_gui::widgets::icon_button(ui, IconId::Stop, "Stop").clicked() {
                        self.command("forgegui.stop");
                    }
                    ui.separator();
                    for (id, icon, label) in [
                        (PROJECT, IconId::Project, "Project"),
                        (ASSETS, IconId::Asset, "Assets"),
                        (INSPECTOR, IconId::Inspector, "Inspector"),
                        (GRAPH, IconId::Graph, "Graph"),
                        (SETTINGS, IconId::Settings, "Settings"),
                    ] {
                        if forge_gui::widgets::labeled_icon_button(ui, icon, label).clicked() {
                            self.open(id);
                        }
                    }
                });
            });
    }
    fn status(&mut self, root: &mut egui::Ui) {
        egui::Panel::bottom("forgegui-status")
            .exact_size(self.gui.theme.status_height)
            .show(root, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.strong("ForgeGUI_Core Lab");
                    ui.separator();
                    ui.label(&self.status);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label("v0.4.8");
                        ui.separator();
                        let unread = self
                            .shared
                            .lock()
                            .map(|s| s.notifications.unread_count())
                            .unwrap_or(0);
                        ui.label(format!("{} notices", unread));
                    });
                });
            });
    }
    fn palette(&mut self, root: &mut egui::Ui) {
        if !self.command_palette {
            return;
        }
        let mut open = true;
        egui::Window::new("Command Palette")
            .open(&mut open)
            .show(root.ctx(), |ui| {
                for (id, label) in [
                    ("forgegui.play", "Play"),
                    ("forgegui.play_here", "Play From Here"),
                    ("forgegui.pause", "Pause / Resume"),
                    ("forgegui.step", "Step"),
                    ("forgegui.restart", "Restart"),
                    ("forgegui.stop", "Stop"),
                    ("demo.graph", "Open Visual Graph"),
                    ("demo.timeline", "Open Timeline"),
                    ("demo.settings", "Open Settings"),
                ] {
                    if ui.button(label).clicked() {
                        if id.starts_with("demo.") {
                            self.open(id);
                        } else {
                            self.command(id);
                        }
                        self.command_palette = false;
                    }
                }
            });
        if !open {
            self.command_palette = false;
        }
    }
}
impl eframe::App for LabApp {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        if ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::P)) {
            self.command_palette = true;
        }
        if ui.input(|i| i.key_pressed(egui::Key::F5)) {
            self.command("forgegui.play");
        }
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.command("forgegui.stop");
        }
        self.top_menu(ui);
        self.toolbar(ui);
        self.status(ui);
        self.gui.show(ui);
        let events: Vec<_> = self.gui.drain_events().collect();
        for event in events {
            match event {
                GuiEvent::Status(s) => self.status = s,
                GuiEvent::Command(id) => self.command(id.as_str()),
            }
        }
        self.palette(ui);
    }
}
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "ForgeGUI_Core Lab",
        options,
        Box::new(|cc| Ok(Box::new(LabApp::new(cc)))),
    )
}
