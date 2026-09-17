#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use eframe::egui;
use forge_gui_chrome::apply_creator_visuals;
use forge_gui_theme::{ForgeTheme, ForgeThemePreset};
use forge_gui_universal_egui::{show_universal_suite, UniversalSuiteState, UniversalTab};
fn main() -> eframe::Result {
    eframe::run_native(
        "ForgeGUI Micro Lab",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_title("ForgeGUI Micro Lab")
                .with_inner_size([720.0, 520.0])
                .with_min_inner_size([520.0, 360.0]),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(MicroApp::new(cc)))),
    )
}
struct MicroApp {
    theme: ForgeTheme,
    state: UniversalSuiteState,
}
impl MicroApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let theme = ForgeTheme::from_preset(ForgeThemePreset::ForgeDark);
        apply_creator_visuals(&cc.egui_ctx, &theme);
        let mut state = UniversalSuiteState::default();
        state.tab = UniversalTab::Controls;
        Self { theme, state }
    }
}
impl eframe::App for MicroApp {
    fn ui(&mut self, root: &mut egui::Ui, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(root, |ui| {
            ui.heading("ForgeGUI Micro Lab");
            ui.label(
                "Small ordinary utility host — no canvas, scene, asset or editor assumptions.",
            );
            ui.separator();
            let _ = show_universal_suite(ui, &mut self.state, &self.theme);
        });
    }
}
