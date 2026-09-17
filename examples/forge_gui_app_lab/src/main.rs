#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use eframe::egui;
use forge_gui_chrome::apply_creator_visuals;
use forge_gui_theme::{ForgeTheme, ForgeThemePreset};
use forge_gui_universal_egui::{show_universal_suite, UniversalSuiteState};
fn main() -> eframe::Result {
    eframe::run_native(
        "ForgeGUI Universal App Lab",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_title("ForgeGUI Universal App Lab")
                .with_inner_size([1180.0, 760.0])
                .with_min_inner_size([720.0, 480.0]),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(AppLab::new(cc)))),
    )
}
struct AppLab {
    theme: ForgeTheme,
    state: UniversalSuiteState,
    nav: usize,
}
impl AppLab {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let theme = ForgeTheme::from_preset(ForgeThemePreset::MidnightMint);
        apply_creator_visuals(&cc.egui_ctx, &theme);
        Self {
            theme,
            state: UniversalSuiteState::default(),
            nav: 0,
        }
    }
}
impl eframe::App for AppLab {
    fn ui(&mut self, root: &mut egui::Ui, _: &mut eframe::Frame) {
        egui::Panel::top("app.header").show(root, |ui| {
            ui.horizontal(|ui| {
                ui.strong("ForgeGUI Universal App Lab");
                ui.separator();
                for (i, label) in ["Home", "Data", "Settings"].into_iter().enumerate() {
                    if ui.selectable_label(self.nav == i, label).clicked() {
                        self.nav = i;
                    }
                }
            });
        });
        egui::Panel::left("app.nav")
            .resizable(false)
            .default_size(180.0)
            .show(root, |ui| {
                ui.heading("Application");
                ui.label("Normal desktop navigation");
                ui.separator();
                ui.label("No editor-only dependencies in the UI model.");
            });
        egui::CentralPanel::default().show(root, |ui| {
            let response = show_universal_suite(ui, &mut self.state, &self.theme);
            if let Some(action) = response.last_action {
                ui.label(action);
            }
        });
        egui::Panel::bottom("app.status").show(root, |ui| {
            ui.horizontal(|ui| {
                ui.label("Ready");
                ui.separator();
                ui.label(format!("Theme: {}", self.theme.label));
            });
        });
    }
}
