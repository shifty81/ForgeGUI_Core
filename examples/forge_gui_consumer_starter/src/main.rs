#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use forge_gui_chrome::{apply_creator_visuals, ModularSurfaceState, ShellProfile, SurfaceDock};
use forge_gui_shell::{
    show_application_shell, ForgeApplicationSpec, ForgeShellContent, ForgeShellState,
};
use forge_gui_theme::{ForgeTheme, ForgeThemePreset};

fn main() -> eframe::Result {
    eframe::run_native(
        "ForgeGUI Consumer Starter",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_title("ForgeGUI Consumer Starter")
                .with_decorations(false)
                .with_transparent(false)
                .with_inner_size([1180.0, 760.0])
                .with_min_inner_size([760.0, 520.0]),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(StarterApp::new(cc)))),
    )
}

struct StarterApp {
    theme: ForgeTheme,
    spec: ForgeApplicationSpec,
    shell: ForgeShellState,
    content: StarterContent,
}

impl StarterApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let theme = ForgeTheme::from_preset(ForgeThemePreset::MidnightMint);
        apply_creator_visuals(&cc.egui_ctx, &theme);
        let shell = ForgeShellState::new(
            ShellProfile::Standard,
            vec![
                ModularSurfaceState::new("navigation", "Navigation", SurfaceDock::Left),
                ModularSurfaceState::new("home", "Home", SurfaceDock::Center),
                ModularSurfaceState::new("details", "Details", SurfaceDock::Right),
                ModularSurfaceState::new("activity", "Activity", SurfaceDock::Bottom),
            ],
        );
        Self {
            theme,
            spec: ForgeApplicationSpec::new("ForgeGUI", "Consumer Starter"),
            shell,
            content: StarterContent::default(),
        }
    }
}

impl eframe::App for StarterApp {
    fn clear_color(&self, _: &egui::Visuals) -> [f32; 4] {
        let shell = self.theme.chrome.shell;
        [
            f32::from(shell.0) / 255.0,
            f32::from(shell.1) / 255.0,
            f32::from(shell.2) / 255.0,
            1.0,
        ]
    }

    fn ui(&mut self, root: &mut egui::Ui, _: &mut eframe::Frame) {
        let ctx = root.ctx().clone();
        let _ = show_application_shell(
            root,
            &ctx,
            &self.spec,
            &mut self.shell,
            &self.theme,
            &mut self.content,
        );
    }
}

#[derive(Default)]
struct StarterContent {
    counter: u32,
    search: String,
}

impl ForgeShellContent for StarterContent {
    fn menu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("File", |ui| {
            if ui.button("New").clicked() {
                self.counter = 0;
                ui.close();
            }
        });
        ui.menu_button("Edit", |ui| {
            ui.label("Application-owned commands go here.");
        });
        ui.menu_button("View", |ui| {
            ui.label("Docking and surfaces are ForgeGUI-owned.");
        });
        ui.menu_button("Help", |ui| {
            ui.label("This executable is an independent ForgeGUI consumer.");
        });
    }

    fn toolbar(&mut self, ui: &mut egui::Ui) {
        if ui.button("Increment").clicked() {
            self.counter += 1;
        }
        ui.separator();
        ui.label(format!("Count: {}", self.counter));
    }

    fn status(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Ready");
            ui.separator();
            ui.label("ForgeGUI public shell runtime");
        });
    }

    fn surface(&mut self, id: &str, ui: &mut egui::Ui) {
        match id {
            "navigation" => {
                ui.heading("Navigation");
                ui.text_edit_singleline(&mut self.search);
                ui.separator();
                #[allow(unused_must_use)]
                ui.selectable_label(true, "Home");
                #[allow(unused_must_use)]
                ui.selectable_label(false, "Documents");
                #[allow(unused_must_use)]
                ui.selectable_label(false, "Settings");
            }
            "home" => {
                ui.heading("Application content");
                ui.label("This center surface belongs to the consumer application.");
                ui.label("Drag panel grips, change dock zones, float panels, lock them, and resize the shell.");
                ui.add_space(12.0);
                if ui.button("Increment example state").clicked() {
                    self.counter += 1;
                }
                ui.label(format!("Counter: {}", self.counter));
            }
            "details" => {
                ui.heading("Details");
                ui.label("Applications register their own content while ForgeGUI owns hosting.");
                ui.separator();
                ui.label(format!("Search: {}", self.search));
            }
            "activity" => {
                ui.heading("Activity");
                ui.monospace("[PASS] ForgeGUI consumer starter initialized");
                ui.monospace(
                    "[INFO] Shell, docking, floating and chrome come from forge_gui_shell",
                );
            }
            _ => {
                ui.label(format!("Consumer surface: {id}"));
            }
        }
    }
}
