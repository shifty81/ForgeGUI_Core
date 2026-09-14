use forge_gui::{
    canvas::{CanvasCamera, CanvasExtent, CanvasSettings},
    console::{ConsoleHub, ConsoleKind, ConsoleLevel, ConsoleViewState},
    egui,
    notify::{NotificationCenter, NotificationRecord, NotificationSeverity},
    pie::{InputOwner, PieController, PieState, PlayMode, PlayRequest},
    EguiPanel, ForgeGuiRuntime, GuiEvent, GuiResult, IconId, PanelDefinition, PanelInstanceId,
    PanelRole, PanelScope, PanelUiContext, PreferredDock,
};
use std::sync::{Arc, Mutex};
pub const PROJECT: &str = "demo.project";
pub const ASSETS: &str = "demo.assets";
pub const CANVAS: &str = "demo.canvas";
pub const INSPECTOR: &str = "demo.inspector";
pub const LAYERS: &str = "demo.layers";
pub const CONSOLE: &str = "demo.console";
pub const NOTIFICATIONS: &str = "demo.notifications";
pub const PROBLEMS: &str = "demo.problems";
pub const TIMELINE: &str = "demo.timeline";
pub const GRAPH: &str = "demo.graph";
pub const HISTORY: &str = "demo.history";
pub const CURVE: &str = "demo.curve";
pub const SETTINGS: &str = "demo.settings";
#[derive(Clone)]
pub struct DemoEntity {
    pub name: String,
    pub position: [f32; 2],
    pub visible: bool,
}
#[derive(Clone)]
pub struct DemoLayer {
    pub name: String,
    pub visible: bool,
    pub locked: bool,
}
pub struct DemoState {
    pub selected: String,
    pub entities: Vec<DemoEntity>,
    pub layers: Vec<DemoLayer>,
    pub history: Vec<String>,
    pub console: ConsoleHub,
    pub notifications: NotificationCenter,
    pub pie: PieController,
    pub canvas: CanvasSettings,
    pub extent: CanvasExtent,
    pub runtime_player: [f32; 2],
}
impl Default for DemoState {
    fn default() -> Self {
        let mut console = ConsoleHub::default();
        console
            .ensure_typed_channel("output", "Output", ConsoleKind::Output)
            .push(
                ConsoleLevel::Success,
                "ForgeGUI",
                "Reference Studio initialized",
            );
        console
            .ensure_typed_channel("cortex", "Cortex", ConsoleKind::Cortex)
            .push(
                ConsoleLevel::Info,
                "Cortex",
                "Embedded assistant channel ready",
            );
        let mut notifications = NotificationCenter::default();
        notifications.push(NotificationRecord::simple(
            "welcome",
            "ForgeGUI recovered",
            "The richer reference studio surface is active.",
            NotificationSeverity::Success,
        ));
        Self {
            selected: "Player".into(),
            entities: vec![
                DemoEntity {
                    name: "Player".into(),
                    position: [160.0, 130.0],
                    visible: true,
                },
                DemoEntity {
                    name: "Camera".into(),
                    position: [320.0, 160.0],
                    visible: true,
                },
                DemoEntity {
                    name: "Workbench".into(),
                    position: [460.0, 260.0],
                    visible: true,
                },
            ],
            layers: vec![
                DemoLayer {
                    name: "Gameplay".into(),
                    visible: true,
                    locked: false,
                },
                DemoLayer {
                    name: "Props".into(),
                    visible: true,
                    locked: false,
                },
                DemoLayer {
                    name: "Collision".into(),
                    visible: true,
                    locked: true,
                },
            ],
            history: vec!["Open project".into(), "Select Player".into()],
            console,
            notifications,
            pie: PieController::default(),
            canvas: CanvasSettings::default(),
            extent: CanvasExtent::from_size(1024.0, 768.0),
            runtime_player: [320.0, 240.0],
        }
    }
}
fn def(
    id: &str,
    title: &str,
    category: &str,
    summary: &str,
    role: PanelRole,
    dock: PreferredDock,
) -> PanelDefinition {
    let mut d = PanelDefinition::new(id, title, role, dock);
    d.category = category.into();
    d.summary = summary.into();
    d
}
pub fn register_demo_panels(
    gui: &mut ForgeGuiRuntime,
    shared: Arc<Mutex<DemoState>>,
) -> GuiResult<()> {
    let s = shared.clone();
    gui.register_panel(
        def(
            PROJECT,
            "Project Explorer",
            "Navigation",
            "Project files, scenes, modules and resources.",
            PanelRole::Navigation,
            PreferredDock::Left,
        ),
        move |_: PanelInstanceId| ProjectPanel {
            shared: s.clone(),
            search: String::new(),
        },
    )?;
    let s = shared.clone();
    gui.register_panel(
        def(
            ASSETS,
            "Asset Browser",
            "Assets",
            "Grid/list asset browsing surface.",
            PanelRole::Explorer,
            PreferredDock::Left,
        ),
        move |_: PanelInstanceId| AssetPanel {
            shared: s.clone(),
            search: String::new(),
        },
    )?;
    let s = shared.clone();
    gui.register_panel(
        def(
            CANVAS,
            "Scene / Room",
            "Editor",
            "Finite/infinite scene host with grid, snap, selection and PIE.",
            PanelRole::Canvas,
            PreferredDock::Center,
        ),
        move |_: PanelInstanceId| CanvasPanel {
            shared: s.clone(),
            camera: CanvasCamera::default().with_limits(0.08, 12.0),
        },
    )?;
    let s = shared.clone();
    gui.register_panel(
        def(
            INSPECTOR,
            "Inspector",
            "Properties",
            "Selection-driven property editor.",
            PanelRole::Inspector,
            PreferredDock::Right,
        ),
        move |_: PanelInstanceId| InspectorPanel { shared: s.clone() },
    )?;
    let s = shared.clone();
    gui.register_panel(
        def(
            LAYERS,
            "Layers",
            "Scene",
            "Semantic layer stack.",
            PanelRole::Layers,
            PreferredDock::Right,
        ),
        move |_: PanelInstanceId| LayersPanel { shared: s.clone() },
    )?;
    let s = shared.clone();
    gui.register_panel(def(CONSOLE,"Console / Cortex","Activity","Embedded output and Cortex lane; terminal channel kind is reserved for a future process host.",PanelRole::Console,PreferredDock::Bottom),move|_:PanelInstanceId|ConsolePanel{shared:s.clone(),view:ConsoleViewState::default()})?;
    let s = shared.clone();
    gui.register_panel(
        def(
            NOTIFICATIONS,
            "Notification Center",
            "Activity",
            "Durable notifications and actions.",
            PanelRole::Notification,
            PreferredDock::Right,
        ),
        move |_: PanelInstanceId| NotificationPanel { shared: s.clone() },
    )?;
    gui.register_panel(
        def(
            PROBLEMS,
            "Problems",
            "Diagnostics",
            "Compiler, validation and runtime diagnostics.",
            PanelRole::Diagnostics,
            PreferredDock::Bottom,
        ),
        |_: PanelInstanceId| ProblemsPanel,
    )?;
    let s = shared.clone();
    gui.register_panel(
        def(
            TIMELINE,
            "Timeline",
            "Animation",
            "Shared sequencer proof surface.",
            PanelRole::Timeline,
            PreferredDock::Bottom,
        ),
        move |_: PanelInstanceId| TimelinePanel {
            shared: s.clone(),
            frame: 11,
        },
    )?;
    gui.register_panel(
        def(
            GRAPH,
            "Visual Graph",
            "Logic",
            "Node/behavior graph proof surface.",
            PanelRole::Graph,
            PreferredDock::Center,
        ),
        |_: PanelInstanceId| GraphPanel,
    )?;
    let s = shared.clone();
    gui.register_panel(
        def(
            HISTORY,
            "History",
            "Editing",
            "Transaction/history surface.",
            PanelRole::History,
            PreferredDock::Right,
        ),
        move |_: PanelInstanceId| HistoryPanel { shared: s.clone() },
    )?;
    gui.register_panel(
        def(
            CURVE,
            "Curve Editor",
            "Animation",
            "Reusable curve editor proof surface.",
            PanelRole::Timeline,
            PreferredDock::Bottom,
        ),
        |_: PanelInstanceId| CurvePanel,
    )?;
    let mut settings = def(
        SETTINGS,
        "ForgeGUI Settings",
        "System",
        "Theme and shared canvas settings.",
        PanelRole::Settings,
        PreferredDock::Right,
    );
    settings.scope = PanelScope::Global;
    let s = shared.clone();
    gui.register_panel(settings, move |_: PanelInstanceId| SettingsPanel {
        shared: s.clone(),
    })?;
    Ok(())
}
struct ProjectPanel {
    shared: Arc<Mutex<DemoState>>,
    search: String,
}
impl EguiPanel for ProjectPanel {
    fn ui(&mut self, ui: &mut egui::Ui, ctx: &mut PanelUiContext<'_>) {
        ui.heading("Project Explorer");
        ui.text_edit_singleline(&mut self.search);
        for name in ["Scenes", "Assets", "Logic", "Data", "Settings"] {
            if (self.search.is_empty() || name.to_lowercase().contains(&self.search.to_lowercase()))
                && ui.selectable_label(false, name).clicked()
            {
                if let Ok(mut s) = self.shared.lock() {
                    s.selected = name.into();
                    s.history.push(format!("Select {name}"));
                }
                ctx.emit(GuiEvent::Status(format!("Selected {name}")));
            }
        }
    }
}
struct AssetPanel {
    shared: Arc<Mutex<DemoState>>,
    search: String,
}
impl EguiPanel for AssetPanel {
    fn ui(&mut self, ui: &mut egui::Ui, ctx: &mut PanelUiContext<'_>) {
        ui.heading("Asset Browser");
        ui.horizontal(|ui| {
            ui.label(forge_gui::widgets::icon_text(IconId::Search));
            ui.text_edit_singleline(&mut self.search);
        });
        for asset in [
            "player.sprite",
            "terrain.atlas",
            "ship.module",
            "ui.theme",
            "world.prefab",
        ] {
            if (self.search.is_empty() || asset.contains(&self.search))
                && ui.button(asset).clicked()
            {
                if let Ok(mut s) = self.shared.lock() {
                    s.selected = asset.into();
                }
                ctx.emit(GuiEvent::Status(format!("Asset {asset}")));
            }
        }
    }
}
struct CanvasPanel {
    shared: Arc<Mutex<DemoState>>,
    camera: CanvasCamera,
}
impl EguiPanel for CanvasPanel {
    fn ui(&mut self, ui: &mut egui::Ui, ctx: &mut PanelUiContext<'_>) {
        ui.horizontal(|ui| {
            if let Ok(mut s) = self.shared.lock() {
                ui.checkbox(&mut s.canvas.infinite_enabled, "Infinite");
                ui.checkbox(&mut s.canvas.grid, "Grid");
                ui.checkbox(&mut s.canvas.snap, "Snap");
                ui.checkbox(&mut s.canvas.guides, "Guides");
                ui.checkbox(&mut s.canvas.minimap, "Map");
                ui.label(format!("Zoom {:.0}%", self.camera.zoom * 100.0));
            }
        });
        let size = ui.available_size();
        let (response, painter) = ui.allocate_painter(size, egui::Sense::click_and_drag());
        let rect = response.rect;
        let alt_pan =
            ui.input(|i| i.modifiers.alt) && response.dragged_by(egui::PointerButton::Primary);
        if response.dragged_by(egui::PointerButton::Middle) || alt_pan {
            let d = ui.input(|i| i.pointer.delta());
            self.camera.pan_by([d.x, d.y]);
        }
        let scroll: f32 = ui.input(|i| i.smooth_scroll_delta().y);
        if let (true, Some(pos)) = (
            response.hovered() && scroll.abs() > 0.0_f32,
            ui.input(|i| i.pointer.hover_pos()),
        ) {
            let factor = (1.0_f32 + scroll * 0.0015_f32).clamp(0.85_f32, 1.15_f32);
            self.camera
                .zoom_at_screen([pos.x, pos.y], [rect.left(), rect.top()], factor);
        }
        painter.rect_filled(rect, 0.0, forge_gui::rgba(ctx.theme.panel_recessed));
        let mut selected = None;
        if let Ok(mut s) = self.shared.lock() {
            if s.canvas.grid {
                let step =
                    self.camera.adaptive_grid_step(s.canvas.snap_step, 24.0) * self.camera.zoom;
                let color = forge_gui::rgba(ctx.theme.border);
                let mut x = rect.left() + self.camera.pan[0].rem_euclid(step);
                while x < rect.right() {
                    painter.line_segment(
                        [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                        egui::Stroke::new(1.0, color),
                    );
                    x += step;
                }
                let mut y = rect.top() + self.camera.pan[1].rem_euclid(step);
                while y < rect.bottom() {
                    painter.line_segment(
                        [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                        egui::Stroke::new(1.0, color),
                    );
                    y += step;
                }
            }
            for entity in &s.entities {
                if !entity.visible {
                    continue;
                }
                let p = self
                    .camera
                    .world_to_screen(entity.position, [rect.left(), rect.top()]);
                let pos = egui::pos2(p[0], p[1]);
                let active = s.selected == entity.name;
                painter.circle_filled(
                    pos,
                    if active { 8.0 } else { 6.0 },
                    forge_gui::rgba(if active {
                        ctx.theme.accent
                    } else {
                        ctx.theme.text_muted
                    }),
                );
                painter.text(
                    pos + egui::vec2(10.0, -6.0),
                    egui::Align2::LEFT_TOP,
                    &entity.name,
                    egui::FontId::default(),
                    forge_gui::rgba(ctx.theme.text),
                );
            }
            if let (true, Some(pos)) = (response.clicked(), response.interact_pointer_pos()) {
                let world = self
                    .camera
                    .screen_to_world([pos.x, pos.y], [rect.left(), rect.top()]);
                let mut point = s.canvas.snap_point(world);
                if !s.canvas.is_infinite() {
                    point = s.extent.clamp(point);
                }
                selected = Some(format!("Canvas {:.0},{:.0}", point[0], point[1]));
            }
            let dt = ui.input(|i| i.stable_dt);
            if s.pie.status.state == PieState::Playing
                && s.pie.status.input_owner == InputOwner::Runtime
            {
                let movement = ui.input(|i| {
                    let mut v = [0.0f32, 0.0f32];
                    if i.key_down(egui::Key::A) {
                        v[0] -= 1.0;
                    }
                    if i.key_down(egui::Key::D) {
                        v[0] += 1.0;
                    }
                    if i.key_down(egui::Key::W) {
                        v[1] -= 1.0;
                    }
                    if i.key_down(egui::Key::S) {
                        v[1] += 1.0;
                    }
                    v
                });
                s.runtime_player[0] += movement[0] * 180.0 * dt;
                s.runtime_player[1] += movement[1] * 180.0 * dt;
                let rp = self
                    .camera
                    .world_to_screen(s.runtime_player, [rect.left(), rect.top()]);
                painter.circle_filled(
                    egui::pos2(rp[0], rp[1]),
                    9.0,
                    forge_gui::rgba(ctx.theme.success),
                );
            }
            s.pie.tick(dt as f64);
            if s.pie.is_running() {
                painter.text(
                    rect.left_top() + egui::vec2(12.0, 12.0),
                    egui::Align2::LEFT_TOP,
                    format!(
                        "PIE {:?} / frame {}",
                        s.pie.status.state, s.pie.status.frame
                    ),
                    egui::FontId::default(),
                    forge_gui::rgba(ctx.theme.success),
                );
            }
            if s.canvas.minimap {
                let mini = egui::Rect::from_min_size(
                    rect.right_bottom() - egui::vec2(170.0, 110.0),
                    egui::vec2(158.0, 98.0),
                );
                painter.rect_stroke(
                    mini,
                    4.0,
                    egui::Stroke::new(1.0, forge_gui::rgba(ctx.theme.border_focus)),
                    egui::StrokeKind::Inside,
                );
                painter.text(
                    mini.left_top() + egui::vec2(6.0, 6.0),
                    egui::Align2::LEFT_TOP,
                    "Navigator",
                    egui::FontId::default(),
                    forge_gui::rgba(ctx.theme.text_muted),
                );
            }
        }
        if let Some(sel) = selected {
            if let Ok(mut s) = self.shared.lock() {
                s.selected = sel.clone();
                s.history.push(format!("Select {sel}"));
            }
            ctx.emit(GuiEvent::Status(sel));
        }
    }
}
struct InspectorPanel {
    shared: Arc<Mutex<DemoState>>,
}
impl EguiPanel for InspectorPanel {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut PanelUiContext<'_>) {
        ui.heading("Inspector");
        if let Ok(mut s) = self.shared.lock() {
            ui.label(format!("Selection: {}", s.selected));
            ui.separator();
            ui.label("Transform");
            let mut x = 160.0;
            let mut y = 130.0;
            ui.horizontal(|ui| {
                ui.label("X");
                ui.add(egui::DragValue::new(&mut x));
                ui.label("Y");
                ui.add(egui::DragValue::new(&mut y));
            });
            ui.checkbox(&mut s.canvas.snap, "Snap placement");
        }
    }
}
struct LayersPanel {
    shared: Arc<Mutex<DemoState>>,
}
impl EguiPanel for LayersPanel {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut PanelUiContext<'_>) {
        ui.heading("Layers");
        if let Ok(mut s) = self.shared.lock() {
            for layer in &mut s.layers {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut layer.visible, "");
                    ui.label(&layer.name);
                    ui.checkbox(&mut layer.locked, "Lock");
                });
            }
        }
    }
}
struct ConsolePanel {
    shared: Arc<Mutex<DemoState>>,
    view: ConsoleViewState,
}
impl EguiPanel for ConsolePanel {
    fn ui(&mut self, ui: &mut egui::Ui, ctx: &mut PanelUiContext<'_>) {
        ui.horizontal(|ui| {
            for id in ["output", "cortex"] {
                if ui
                    .selectable_label(self.view.active_channel == id, id)
                    .clicked()
                {
                    self.view.active_channel = id.into();
                }
            }
            ui.label("Filter");
            ui.text_edit_singleline(&mut self.view.filter);
        });
        if ui.button("Emit sample operation").clicked() {
            if let Ok(mut s) = self.shared.lock() {
                s.console.push(
                    &self.view.active_channel,
                    ConsoleLevel::Success,
                    "Lab",
                    "Sample operation completed",
                );
            }
            ctx.emit(GuiEvent::Status("Console operation emitted".into()));
        }
        egui::ScrollArea::vertical()
            .stick_to_bottom(self.view.follow_tail)
            .show(ui, |ui| {
                if let Ok(s) = self.shared.lock() {
                    if let Some(ch) = s.console.channel(&self.view.active_channel) {
                        for e in ch.matching(&self.view.filter) {
                            ui.horizontal(|ui| {
                                ui.label(format!("#{:04}", e.sequence));
                                ui.strong(&e.source);
                                ui.label(&e.text);
                            });
                        }
                    }
                }
            });
    }
}
struct NotificationPanel {
    shared: Arc<Mutex<DemoState>>,
}
impl EguiPanel for NotificationPanel {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut PanelUiContext<'_>) {
        ui.heading("Notification Center");
        if let Ok(mut s) = self.shared.lock() {
            ui.horizontal(|ui| {
                ui.label(format!("{} unread", s.notifications.unread_count()));
                if ui.button("Mark all read").clicked() {
                    s.notifications.mark_all_read();
                }
            });
            for r in s.notifications.records() {
                ui.group(|ui| {
                    ui.strong(&r.title);
                    ui.label(&r.message);
                    ui.small(format!("{:?}", r.severity));
                });
            }
        }
    }
}
struct ProblemsPanel;
impl EguiPanel for ProblemsPanel {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut PanelUiContext<'_>) {
        ui.heading("Problems");
        forge_gui::widgets::virtual_table(
            ui,
            egui::Id::new("problems-table"),
            &["Severity", "Source", "Message"],
            3,
            22.0,
            |ui, row, col| {
                let rows = [
                    ["Info", "PCC", "No pending patches"],
                    ["Warning", "Assets", "One placeholder asset"],
                    ["Info", "Runtime", "PIE stopped"],
                ];
                ui.label(rows[row][col]);
            },
        );
    }
}
struct TimelinePanel {
    shared: Arc<Mutex<DemoState>>,
    frame: u32,
}
impl EguiPanel for TimelinePanel {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut PanelUiContext<'_>) {
        ui.heading("Timeline");
        ui.add(egui::Slider::new(&mut self.frame, 0..=120).text("Frame"));
        if let Ok(s) = self.shared.lock() {
            ui.label(format!("PIE frame {}", s.pie.status.frame));
        }
        let rect = ui.available_rect_before_wrap();
        let y = rect.top() + 28.0;
        ui.painter().line_segment(
            [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
            egui::Stroke::new(2.0, egui::Color32::GRAY),
        );
    }
}
struct GraphPanel;
impl EguiPanel for GraphPanel {
    fn ui(&mut self, ui: &mut egui::Ui, ctx: &mut PanelUiContext<'_>) {
        ui.heading("Visual Graph");
        let rect = ui.available_rect_before_wrap();
        let a = egui::Rect::from_min_size(
            rect.left_top() + egui::vec2(30.0, 45.0),
            egui::vec2(140.0, 70.0),
        );
        let b = egui::Rect::from_min_size(
            rect.left_top() + egui::vec2(250.0, 145.0),
            egui::vec2(140.0, 70.0),
        );
        for (r, t) in [(a, "Input Event"), (b, "Action")] {
            ui.painter()
                .rect_filled(r, 6.0, forge_gui::rgba(ctx.theme.panel_raised));
            ui.painter().rect_stroke(
                r,
                6.0,
                egui::Stroke::new(1.0, forge_gui::rgba(ctx.theme.border_focus)),
                egui::StrokeKind::Inside,
            );
            ui.painter().text(
                r.center(),
                egui::Align2::CENTER_CENTER,
                t,
                egui::FontId::default(),
                forge_gui::rgba(ctx.theme.text),
            );
        }
        ui.painter().line_segment(
            [a.right_center(), b.left_center()],
            egui::Stroke::new(2.0, forge_gui::rgba(ctx.theme.accent)),
        );
    }
}
struct HistoryPanel {
    shared: Arc<Mutex<DemoState>>,
}
impl EguiPanel for HistoryPanel {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut PanelUiContext<'_>) {
        ui.heading("History");
        if let Ok(s) = self.shared.lock() {
            for (i, item) in s.history.iter().rev().enumerate() {
                ui.label(format!("{}  {}", i + 1, item));
            }
        }
    }
}
struct CurvePanel;
impl EguiPanel for CurvePanel {
    fn ui(&mut self, ui: &mut egui::Ui, ctx: &mut PanelUiContext<'_>) {
        ui.heading("Curve Editor");
        let rect = ui.available_rect_before_wrap();
        let points = [
            egui::pos2(rect.left() + 20.0, rect.bottom() - 30.0),
            egui::pos2(rect.left() + 120.0, rect.top() + 80.0),
            egui::pos2(rect.left() + 240.0, rect.center().y),
            egui::pos2(rect.right() - 30.0, rect.top() + 35.0),
        ];
        for pair in points.windows(2) {
            ui.painter().line_segment(
                [pair[0], pair[1]],
                egui::Stroke::new(2.0, forge_gui::rgba(ctx.theme.accent)),
            );
        }
        for p in points {
            ui.painter()
                .circle_filled(p, 4.0, forge_gui::rgba(ctx.theme.text));
        }
    }
}
struct SettingsPanel {
    shared: Arc<Mutex<DemoState>>,
}
impl EguiPanel for SettingsPanel {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut PanelUiContext<'_>) {
        ui.heading("ForgeGUI Settings");
        ui.label("Shared project defaults");
        if let Ok(mut s) = self.shared.lock() {
            egui::CollapsingHeader::new("Canvas defaults")
                .default_open(true)
                .show(ui, |ui| {
                    ui.checkbox(&mut s.canvas.infinite_enabled, "Infinite canvas enabled");
                    ui.checkbox(&mut s.canvas.grid, "Grid");
                    ui.checkbox(&mut s.canvas.snap, "Snap");
                    ui.checkbox(&mut s.canvas.guides, "Guides");
                    ui.checkbox(&mut s.canvas.minimap, "Minimap");
                    ui.add(
                        egui::Slider::new(&mut s.canvas.snap_step, 1.0..=128.0).text("Snap step"),
                    );
                });
            egui::CollapsingHeader::new("PIE")
                .default_open(true)
                .show(ui, |ui| {
                    ui.label(format!("State: {:?}", s.pie.status.state));
                    if ui.button("Play").clicked() {
                        s.pie.play(PlayRequest {
                            mode: PlayMode::Normal,
                            document_id: Some("demo.scene".into()),
                        });
                    }
                    if ui.button("Stop").clicked() {
                        s.pie.stop();
                    }
                });
        }
    }
}
