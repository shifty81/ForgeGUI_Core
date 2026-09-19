//! egui renderer adapter for the shared ForgeGUI nested docking model.
#![forbid(unsafe_code)]

use egui::{Color32, RichText, Stroke, Ui};
use forge_gui_chrome::docking::{
    drop_preview_rect, drop_zone, DockDrop, DockNode, ModularDockTree, SplitAxis,
};
use forge_gui_chrome::{ModularSurfaceState, SurfaceDock};
use forge_gui_core::{
    CommandId, ForgeGuiError, GuiResult, InstancePolicy, PanelDefinition, PanelId, PanelInstanceId,
    PreferredDock, Rgba, ThemeTokens,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug)]
pub enum GuiEvent {
    Command(CommandId),
    Status(String),
}

pub struct PanelUiContext<'a> {
    pub events: &'a mut Vec<GuiEvent>,
    pub theme: &'a ThemeTokens,
}

impl PanelUiContext<'_> {
    pub fn emit(&mut self, event: GuiEvent) {
        self.events.push(event);
    }
}

pub trait EguiPanel {
    fn ui(&mut self, ui: &mut Ui, ctx: &mut PanelUiContext<'_>);
}

pub trait EguiPanelFactory: Send + Sync {
    fn create(&self, instance: &PanelInstanceId) -> Box<dyn EguiPanel>;
}

struct ConcretePanelFactory<P, F> {
    factory: F,
    panel: std::marker::PhantomData<fn() -> P>,
}

impl<P, F> ConcretePanelFactory<P, F> {
    fn new(factory: F) -> Self {
        Self {
            factory,
            panel: std::marker::PhantomData,
        }
    }
}

impl<P, F> EguiPanelFactory for ConcretePanelFactory<P, F>
where
    P: EguiPanel + 'static,
    F: Fn(PanelInstanceId) -> P + Send + Sync,
{
    fn create(&self, instance: &PanelInstanceId) -> Box<dyn EguiPanel> {
        Box::new((self.factory)(instance.clone()))
    }
}

struct RegisteredPanel {
    definition: PanelDefinition,
    factory: Box<dyn EguiPanelFactory>,
}

#[derive(Default)]
pub struct EguiPanelRegistry {
    entries: BTreeMap<PanelId, RegisteredPanel>,
}

impl EguiPanelRegistry {
    pub fn register<P, F>(&mut self, definition: PanelDefinition, factory: F) -> GuiResult<()>
    where
        P: EguiPanel + 'static,
        F: Fn(PanelInstanceId) -> P + Send + Sync + 'static,
    {
        self.register_factory(definition, ConcretePanelFactory::<P, F>::new(factory))
    }

    pub fn register_factory<F>(&mut self, definition: PanelDefinition, factory: F) -> GuiResult<()>
    where
        F: EguiPanelFactory + 'static,
    {
        definition.validate()?;
        if self.entries.contains_key(&definition.id) {
            return Err(ForgeGuiError::DuplicatePanel(definition.id.to_string()));
        }
        self.entries.insert(
            definition.id.clone(),
            RegisteredPanel {
                definition,
                factory: Box::new(factory),
            },
        );
        Ok(())
    }

    pub fn definition(&self, id: &PanelId) -> Option<&PanelDefinition> {
        self.entries.get(id).map(|entry| &entry.definition)
    }

    pub fn definitions(&self) -> impl Iterator<Item = &PanelDefinition> {
        self.entries.values().map(|entry| &entry.definition)
    }
}

#[derive(Default)]
struct PanelInstances {
    values: BTreeMap<PanelInstanceId, Box<dyn EguiPanel>>,
}

impl PanelInstances {
    fn render(
        &mut self,
        registry: &EguiPanelRegistry,
        tab: &PanelInstanceId,
        ui: &mut Ui,
        events: &mut Vec<GuiEvent>,
        theme: &ThemeTokens,
    ) {
        if !self.values.contains_key(tab) {
            if let Some(entry) = registry.entries.get(&tab.panel) {
                self.values.insert(tab.clone(), entry.factory.create(tab));
            }
        }

        if let Some(panel) = self.values.get_mut(tab) {
            let mut ctx = PanelUiContext { events, theme };
            panel.ui(ui, &mut ctx);
        } else {
            ui.colored_label(
                Color32::LIGHT_RED,
                format!("Panel '{}' is not registered", tab.panel),
            );
        }
    }
}

/// Stable, collision-free within a process, persisted identifier for one panel instance.
/// The length prefix unambiguously separates the validated panel ID from instance keys,
/// which may themselves contain delimiters such as `:`.
fn instance_key(instance: &PanelInstanceId) -> String {
    format!(
        "{}:{}:{}",
        instance.panel.as_str().len(),
        instance.panel.as_str(),
        instance.instance
    )
}

fn surface_dock(dock: PreferredDock) -> SurfaceDock {
    match dock {
        PreferredDock::Left => SurfaceDock::Left,
        PreferredDock::Right => SurfaceDock::Right,
        PreferredDock::Center => SurfaceDock::Center,
        PreferredDock::Bottom => SurfaceDock::Bottom,
        PreferredDock::Floating => SurfaceDock::Floating,
    }
}

fn can_float(definition: &PanelDefinition) -> bool {
    matches!(
        definition.host_mode,
        forge_gui_core::HostMode::Window | forge_gui_core::HostMode::DockedOrWindow
    )
}

/// The same canonical `ModularDockTree` drives the Lab and independent consumers.
/// `egui_dock::DockState` is no longer a parallel source of layout truth.
/// Floating hosts in this adapter are *in-app egui windows*; a product requiring
/// native OS windows must supply the viewport bridge and call the host APIs.
pub struct ForgeGuiRuntime {
    pub theme: ThemeTokens,
    registry: EguiPanelRegistry,
    instances: PanelInstances,
    ids: BTreeMap<String, PanelInstanceId>,
    surfaces: Vec<ModularSurfaceState>,
    dock: ModularDockTree,
    floating: BTreeMap<String, ModularDockTree>,
    floating_rects: BTreeMap<String, egui::Rect>,
    events: Vec<GuiEvent>,
    next_instance: u64,
    dragging: Option<String>,
    leaf_rects: Vec<(Option<String>, String, egui::Rect)>,
}

impl Default for ForgeGuiRuntime {
    fn default() -> Self {
        Self {
            theme: ThemeTokens::default(),
            registry: EguiPanelRegistry::default(),
            instances: PanelInstances::default(),
            ids: BTreeMap::new(),
            surfaces: Vec::new(),
            dock: ModularDockTree::default(),
            floating: BTreeMap::new(),
            floating_rects: BTreeMap::new(),
            events: Vec::new(),
            next_instance: 1,
            dragging: None,
            leaf_rects: Vec::new(),
        }
    }
}

impl ForgeGuiRuntime {
    pub fn register_panel<P, F>(&mut self, definition: PanelDefinition, factory: F) -> GuiResult<()>
    where
        P: EguiPanel + 'static,
        F: Fn(PanelInstanceId) -> P + Send + Sync + 'static,
    {
        self.registry.register(definition, factory)
    }

    pub fn register_panel_factory<F>(
        &mut self,
        definition: PanelDefinition,
        factory: F,
    ) -> GuiResult<()>
    where
        F: EguiPanelFactory + 'static,
    {
        self.registry.register_factory(definition, factory)
    }

    pub fn definition(&self, panel: &PanelId) -> Option<&PanelDefinition> {
        self.registry.definition(panel)
    }

    pub fn open_panel(&mut self, panel: PanelId) {
        let Some(definition) = self.registry.definition(&panel) else {
            return;
        };
        let policy = definition.instance_policy.clone();
        let preferred = definition.preferred_dock;
        let allow_float = can_float(definition);
        let instance = match policy {
            InstancePolicy::Singleton => PanelInstanceId::singleton(panel),
            InstancePolicy::Multiple { max_instances } => {
                if max_instances.is_some_and(|max| {
                    self.ids.values().filter(|id| id.panel == panel).count() >= max as usize
                }) {
                    return;
                }
                let key = format!("instance-{}", self.next_instance);
                self.next_instance += 1;
                PanelInstanceId::keyed(panel, key)
            }
        };
        let id = instance_key(&instance);
        if let Some(surface) = self.surfaces.iter_mut().find(|s| s.id == id) {
            if !surface.visible {
                surface.visible = true;
                if surface.dock == SurfaceDock::Floating {
                    // A closed native/in-app host returns to its last known dock.
                    surface.redock();
                }
                self.reconcile();
            }
            self.activate(&id);
            return;
        }
        let dock = if preferred == PreferredDock::Floating && !allow_float {
            SurfaceDock::Center
        } else {
            surface_dock(preferred)
        };
        let title = self
            .registry
            .definition(&instance.panel)
            .map(|entry| entry.title.clone())
            .unwrap_or_default();
        let mut surface = ModularSurfaceState::new(id.clone(), title, dock);
        if dock == SurfaceDock::Floating {
            surface.floating_host = Some(id.clone());
        }
        self.ids.insert(id.clone(), instance);
        self.surfaces.push(surface);
        self.reconcile();
        self.activate(&id);
    }

    pub fn has_panel(&self, panel: &PanelId) -> bool {
        self.ids.values().any(|instance| &instance.panel == panel)
    }

    pub fn contains_instance(&self, instance: &PanelInstanceId) -> bool {
        self.ids.contains_key(&instance_key(instance))
    }

    pub fn activate_instance(&mut self, instance: &PanelInstanceId) -> bool {
        self.activate(&instance_key(instance))
    }

    fn activate(&mut self, id: &str) -> bool {
        let Some(surface) = self
            .surfaces
            .iter()
            .find(|surface| surface.id == id && surface.visible)
        else {
            return false;
        };
        if surface.dock == SurfaceDock::Floating {
            self.floating
                .get_mut(surface.floating_host_id())
                .is_some_and(|tree| tree.activate(id))
        } else {
            self.dock.activate(id)
        }
    }

    pub fn dock_tree(&self) -> &ModularDockTree {
        &self.dock
    }
    pub fn dock_tree_mut(&mut self) -> &mut ModularDockTree {
        &mut self.dock
    }
    pub fn floating_trees(&self) -> &BTreeMap<String, ModularDockTree> {
        &self.floating
    }
    pub fn surfaces(&self) -> &[ModularSurfaceState] {
        &self.surfaces
    }

    /// Restore a previously persisted canonical layout. Unknown, duplicate,
    /// hidden, wrong-host and invalid-ratio nodes are repaired by the model.
    pub fn restore_layout(
        &mut self,
        dock: ModularDockTree,
        floating: BTreeMap<String, ModularDockTree>,
    ) {
        self.dock = dock;
        self.floating = floating;
        self.reconcile();
    }

    /// Persist `surfaces()` together with `dock_tree()` and `floating_trees()`.
    /// Rehydration only accepts locations for currently registered instances;
    /// an old save cannot inject a panel, change a title or bypass host policy.
    pub fn restore_layout_with_surfaces(
        &mut self,
        saved_surfaces: &[ModularSurfaceState],
        dock: ModularDockTree,
        floating: BTreeMap<String, ModularDockTree>,
    ) {
        for current in &mut self.surfaces {
            let Some(saved) = saved_surfaces.iter().find(|saved| saved.id == current.id) else {
                continue;
            };
            let allowed = self
                .ids
                .get(&current.id)
                .and_then(|id| self.registry.definition(&id.panel))
                .is_some_and(can_float);
            if saved.dock == SurfaceDock::Floating && !allowed {
                continue;
            }
            current.visible = saved.visible;
            current.dock = saved.dock;
            current.last_dock = if saved.last_dock == SurfaceDock::Floating {
                SurfaceDock::Center
            } else {
                saved.last_dock
            };
            current.floating_host = if saved.dock == SurfaceDock::Floating {
                Some(saved.floating_host_id().to_owned())
            } else {
                None
            };
        }
        self.restore_layout(dock, floating);
    }

    fn reconcile(&mut self) {
        self.dock = ModularDockTree::restored(Some(self.dock.clone()), &self.surfaces);
        let hosts: BTreeSet<String> = self
            .surfaces
            .iter()
            .filter(|s| s.visible && s.dock == SurfaceDock::Floating)
            .map(|s| s.floating_host_id().to_owned())
            .collect();
        self.floating.retain(|host, _| hosts.contains(host));
        self.floating_rects.retain(|host, _| hosts.contains(host));
        for host in hosts {
            let saved = self.floating.remove(&host);
            let tree = ModularDockTree::restored_for_host(saved, &self.surfaces, &host);
            self.floating.insert(host, tree);
        }
    }

    pub fn float_instance(&mut self, instance: &PanelInstanceId, rect: egui::Rect) -> bool {
        let id = instance_key(instance);
        let Some(def) = self.registry.definition(&instance.panel) else {
            return false;
        };
        if !can_float(def)
            || ![rect.min.x, rect.min.y, rect.max.x, rect.max.y]
                .iter()
                .all(|value| value.is_finite())
            || rect.width() <= 0.0
            || rect.height() <= 0.0
        {
            return false;
        }
        let Some(surface) = self.surfaces.iter_mut().find(|s| s.id == id && s.visible) else {
            return false;
        };
        if !surface.move_to(SurfaceDock::Floating) {
            return false;
        }
        surface.floating_host = Some(id.clone());
        self.floating_rects.insert(id, rect);
        self.reconcile();
        true
    }

    pub fn hide_instance(&mut self, instance: &PanelInstanceId) -> bool {
        let id = instance_key(instance);
        let Some(surface) = self.surfaces.iter_mut().find(|s| s.id == id && s.visible) else {
            return false;
        };
        surface.visible = false;
        self.reconcile();
        true
    }

    /// Consumer-facing atomic split/tab command using the very same operation as
    /// the Lab, not a second `DockState`-specific implementation.
    pub fn dock_relative(
        &mut self,
        source: &PanelInstanceId,
        target: &PanelInstanceId,
        zone: DockDrop,
    ) -> bool {
        self.move_to(&instance_key(source), &instance_key(target), zone)
    }

    fn move_to(&mut self, source: &str, target: &str, zone: DockDrop) -> bool {
        if source == target {
            return false;
        }
        let Some(origin) = self
            .surfaces
            .iter()
            .find(|s| s.id == source && s.visible && !s.locked)
            .cloned()
        else {
            return false;
        };
        let Some(destination) = self
            .surfaces
            .iter()
            .find(|s| s.id == target && s.visible)
            .cloned()
        else {
            return false;
        };
        let new_host = if destination.dock == SurfaceDock::Floating {
            Some(destination.floating_host_id().to_owned())
        } else {
            None
        };
        if new_host.is_some()
            && !self
                .ids
                .get(source)
                .and_then(|id| self.registry.definition(&id.panel))
                .is_some_and(can_float)
        {
            return false;
        }
        let old_host = if origin.dock == SurfaceDock::Floating {
            Some(origin.floating_host_id().to_owned())
        } else {
            None
        };
        if old_host == new_host {
            let result = match new_host {
                Some(ref host) => self.floating.get_mut(host).is_some_and(|tree| {
                    tree.move_relative_in_host(source, target, zone, &self.surfaces, host)
                }),
                None => self
                    .dock
                    .move_relative(source, target, zone, &self.surfaces),
            };
            if result {
                self.activate(source);
            }
            return result;
        }
        // Validate the target candidate *before* releasing the old owner.
        let mut proposal = self.surfaces.clone();
        let Some(moving) = proposal.iter_mut().find(|s| s.id == source) else {
            return false;
        };
        if let Some(ref host) = new_host {
            if !moving.join_floating_host(host) {
                return false;
            }
        } else if !moving.move_to(destination.dock) {
            return false;
        }
        let accepted = if let Some(ref host) = new_host {
            let mut proposed_tree = self.floating.get(host).cloned().unwrap_or_default();
            if !proposed_tree.attach_in_host(source, target, zone, &proposal, host) {
                return false;
            }
            self.floating.insert(host.clone(), proposed_tree);
            true
        } else {
            let mut proposed_tree = self.dock.clone();
            if !proposed_tree.attach(source, target, zone, &proposal) {
                return false;
            }
            self.dock = proposed_tree;
            true
        };
        if accepted {
            if let Some(host) = old_host {
                if let Some(tree) = self.floating.get_mut(&host) {
                    tree.detach(source);
                }
            } else {
                self.dock.detach(source);
            }
            self.surfaces = proposal;
            self.reconcile();
            self.activate(source);
        }
        accepted
    }

    fn redock_host(&mut self, host: &str) {
        for surface in self.surfaces.iter_mut().filter(|s| {
            s.visible && s.dock == SurfaceDock::Floating && s.floating_host_id() == host
        }) {
            surface.redock();
        }
        self.reconcile();
    }

    pub fn install_theme(&self, ctx: &egui::Context) {
        install_theme(ctx, &self.theme);
    }

    pub fn show(&mut self, ui: &mut Ui) {
        self.reconcile();
        self.leaf_rects.clear();
        if let Some(root) = self.dock.root.clone() {
            let rect = ui.available_rect_before_wrap();
            self.show_node(ui, &root, rect, &[], None);
        } else {
            ui.label("No open panels");
        }
        // Floating windows share the same nested renderer and model. Native OS
        // viewport ownership is an explicit host responsibility, not emulated.
        let hosts: Vec<String> = self.floating.keys().cloned().collect();
        let ctx = ui.ctx().clone();
        for host in hosts {
            let Some(root) = self.floating.get(&host).and_then(|tree| tree.root.clone()) else {
                continue;
            };
            let title = self
                .surfaces
                .iter()
                .find(|s| s.floating_host_id() == host)
                .map(|s| s.title.clone())
                .unwrap_or_else(|| "Floating panels".into());
            let previous = self.floating_rects.get(&host).copied();
            let mut open = true;
            let mut window = egui::Window::new(title)
                .id(egui::Id::new(("forgegui.consumer.floating", &host)))
                .title_bar(false)
                .resizable(true)
                .collapsible(false)
                .frame(
                    egui::Frame::new()
                        .fill(color(self.theme.panel))
                        .corner_radius(8),
                );
            if let Some(rect) = previous {
                window = window.default_rect(rect);
            }
            let response = window.open(&mut open).show(&ctx, |ui| {
                let rect = ui.available_rect_before_wrap();
                self.show_node(ui, &root, rect, &[], Some(&host));
            });
            if let Some(inner) = response {
                // Window geometry is OS/egui-owned after opening: never set an
                // exact size every frame, which caused earlier resize jitter.
                self.floating_rects
                    .insert(host.clone(), inner.response.rect);
            }
            if !open {
                self.redock_host(&host);
            }
        }
        self.finish_drag(&ctx);
    }

    fn show_node(
        &mut self,
        ui: &mut Ui,
        node: &DockNode,
        rect: egui::Rect,
        path: &[bool],
        host: Option<&str>,
    ) {
        if rect.width() < 35.0 || rect.height() < 35.0 {
            return;
        }
        match node {
            DockNode::Tabs { tabs, active } => {
                let leaf_salt = (
                    "forgegui.consumer.leaf",
                    host.unwrap_or("main"),
                    path.to_vec(),
                );
                ui.scope_builder(
                    egui::UiBuilder::new().id_salt(leaf_salt).max_rect(rect),
                    |ui| self.show_leaf(ui, tabs, active, rect, host),
                );
            }
            DockNode::Split {
                axis,
                ratio,
                first,
                second,
            } => {
                let gap = 6.0;
                let ratio = ratio.clamp(0.12, 0.88);
                let (a, splitter, b) = match axis {
                    SplitAxis::Horizontal => {
                        let mid = rect.left() + rect.width() * ratio;
                        (
                            egui::Rect::from_min_max(
                                rect.min,
                                egui::pos2(mid - gap * 0.5, rect.bottom()),
                            ),
                            egui::Rect::from_min_max(
                                egui::pos2(mid - gap * 0.5, rect.top()),
                                egui::pos2(mid + gap * 0.5, rect.bottom()),
                            ),
                            egui::Rect::from_min_max(
                                egui::pos2(mid + gap * 0.5, rect.top()),
                                rect.max,
                            ),
                        )
                    }
                    SplitAxis::Vertical => {
                        let mid = rect.top() + rect.height() * ratio;
                        (
                            egui::Rect::from_min_max(
                                rect.min,
                                egui::pos2(rect.right(), mid - gap * 0.5),
                            ),
                            egui::Rect::from_min_max(
                                egui::pos2(rect.left(), mid - gap * 0.5),
                                egui::pos2(rect.right(), mid + gap * 0.5),
                            ),
                            egui::Rect::from_min_max(
                                egui::pos2(rect.left(), mid + gap * 0.5),
                                rect.max,
                            ),
                        )
                    }
                };
                let mut left_path = path.to_vec();
                left_path.push(false);
                self.show_node(ui, first, a, &left_path, host);
                let mut right_path = path.to_vec();
                right_path.push(true);
                self.show_node(ui, second, b, &right_path, host);
                let handle = ui.interact(
                    splitter.expand(3.0),
                    ui.id()
                        .with(("forgegui.consumer.split", host, path.to_vec())),
                    egui::Sense::click_and_drag(),
                );
                if handle.dragged_by(egui::PointerButton::Primary) {
                    if let Some(pointer) = ui.ctx().pointer_hover_pos() {
                        let next = match axis {
                            SplitAxis::Horizontal => (pointer.x - rect.left()) / rect.width(),
                            SplitAxis::Vertical => (pointer.y - rect.top()) / rect.height(),
                        };
                        if let Some(host) = host {
                            if let Some(tree) = self.floating.get_mut(host) {
                                tree.resize_split(path, next);
                            }
                        } else {
                            self.dock.resize_split(path, next);
                        }
                    }
                }
                let cursor = match axis {
                    SplitAxis::Horizontal => egui::CursorIcon::ResizeHorizontal,
                    SplitAxis::Vertical => egui::CursorIcon::ResizeVertical,
                };
                handle.on_hover_cursor(cursor);
                ui.painter()
                    .rect_filled(splitter, 2.0, color(self.theme.border));
            }
        }
    }

    fn show_leaf(
        &mut self,
        ui: &mut Ui,
        tabs: &[String],
        selected: &str,
        rect: egui::Rect,
        host: Option<&str>,
    ) {
        let visible: Vec<String> = tabs
            .iter()
            .filter(|id| {
                self.surfaces
                    .iter()
                    .any(|s| s.id.as_str() == id.as_str() && s.visible)
            })
            .cloned()
            .collect();
        if visible.is_empty() {
            return;
        }
        let active = if visible.iter().any(|id| id == selected) {
            selected.to_owned()
        } else {
            visible[0].clone()
        };
        let mut requested_active = None;
        let mut hide = false;
        let mut drag = None;
        let theme = self.theme.clone();
        egui::Frame::new()
            .fill(color(theme.panel))
            .corner_radius(8)
            .inner_margin(egui::Margin::same(4))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    for tab in &visible {
                        let title = self
                            .surfaces
                            .iter()
                            .find(|surface| surface.id == *tab)
                            .map(|surface| surface.title.as_str())
                            .unwrap_or("Panel");
                        let locked = self
                            .surfaces
                            .iter()
                            .find(|surface| surface.id == *tab)
                            .is_some_and(|s| s.locked);
                        let response = ui.add(
                            egui::Button::new(RichText::new(title).strong())
                                .selected(tab == &active)
                                .sense(egui::Sense::click_and_drag()),
                        );
                        if response.clicked() {
                            requested_active = Some(tab.clone());
                        }
                        if !locked && response.drag_started_by(egui::PointerButton::Primary) {
                            drag = Some(tab.clone());
                        }
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("×").on_hover_text("Hide panel").clicked() {
                            hide = true;
                        }
                    });
                });
                ui.separator();
                if let Some(instance) = self.ids.get(&active).cloned() {
                    self.instances.render(
                        &self.registry,
                        &instance,
                        ui,
                        &mut self.events,
                        &self.theme,
                    );
                }
            });
        self.leaf_rects
            .push((host.map(str::to_owned), active.clone(), rect));
        if let Some(active) = requested_active {
            if let Some(host) = host {
                if let Some(tree) = self.floating.get_mut(host) {
                    tree.activate(&active);
                }
            } else {
                self.dock.activate(&active);
            }
        }
        if let Some(tab) = drag {
            self.dragging = Some(tab);
        }
        if hide {
            if let Some(instance) = self.ids.get(&active).cloned() {
                self.hide_instance(&instance);
            }
        }
        if let Some(dragging) = &self.dragging {
            if let Some(pointer) = ui.ctx().pointer_hover_pos() {
                if rect.contains(pointer) && dragging != &active {
                    if let Some(zone) = drop_zone(rect, pointer) {
                        ui.painter().rect_stroke(
                            drop_preview_rect(rect, zone),
                            4.0,
                            Stroke::new(2.0, Color32::LIGHT_BLUE),
                            egui::StrokeKind::Inside,
                        );
                    }
                }
            }
        }
    }

    fn finish_drag(&mut self, ctx: &egui::Context) {
        if self.dragging.is_none() {
            return;
        }
        let released = ctx.input(|input| input.pointer.any_released());
        if !released {
            return;
        }
        let source = self.dragging.take().unwrap_or_default();
        let Some(pointer) = ctx.pointer_hover_pos() else {
            return;
        };
        let destination = self.leaf_rects.iter().rev().find_map(|(_, target, rect)| {
            if !rect.contains(pointer) || target == &source {
                return None;
            }
            drop_zone(*rect, pointer).map(|zone| (target.clone(), zone))
        });
        if let Some((target, zone)) = destination {
            self.move_to(&source, &target, zone);
        } else if !self
            .leaf_rects
            .iter()
            .any(|(_, _, rect)| rect.contains(pointer))
        {
            if let Some(instance) = self.ids.get(&source).cloned() {
                let rect = egui::Rect::from_min_size(pointer, egui::vec2(420.0, 320.0));
                self.float_instance(&instance, rect);
            }
        }
    }

    pub fn drain_events(&mut self) -> impl Iterator<Item = GuiEvent> + '_ {
        self.events.drain(..)
    }
}

pub fn install_theme(ctx: &egui::Context, tokens: &ThemeTokens) {
    let active = ctx.theme();
    let mut style = (*ctx.style_of(active)).clone();
    style.visuals.panel_fill = color(tokens.panel);
    style.visuals.window_fill = color(tokens.panel);
    style.visuals.extreme_bg_color = color(tokens.background);
    style.visuals.widgets.inactive.bg_fill = color(tokens.panel_raised);
    style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, color(tokens.border));
    style.visuals.override_text_color = Some(color(tokens.text));
    ctx.set_style_of(active, style);
}

pub fn rgba(value: Rgba) -> Color32 {
    color(value)
}

fn color(value: Rgba) -> Color32 {
    Color32::from_rgba_unmultiplied(value.0, value.1, value.2, value.3)
}

#[cfg(test)]
mod tests {
    use super::*;
    use forge_gui_core::PanelRole;
    struct ProbePanel;
    impl EguiPanel for ProbePanel {
        fn ui(&mut self, _ui: &mut Ui, _ctx: &mut PanelUiContext<'_>) {}
    }
    fn definition(id: &str, dock: PreferredDock) -> PanelDefinition {
        PanelDefinition::new(id, id, PanelRole::Tool, dock)
    }
    fn populated() -> ForgeGuiRuntime {
        let mut runtime = ForgeGuiRuntime::default();
        for (id, dock) in [
            ("test.center", PreferredDock::Center),
            ("test.left", PreferredDock::Left),
            ("test.right", PreferredDock::Right),
            ("test.bottom", PreferredDock::Bottom),
            ("test.float", PreferredDock::Floating),
        ] {
            runtime
                .register_panel(definition(id, dock), |_| ProbePanel)
                .unwrap();
            runtime.open_panel(PanelId::from(id));
        }
        runtime
    }
    #[test]
    fn consumer_uses_shared_tree_for_main_and_floating() {
        let runtime = populated();
        assert_eq!(runtime.dock.ordered_tabs().len(), 4);
        assert_eq!(
            runtime
                .floating
                .values()
                .flat_map(ModularDockTree::ordered_tabs)
                .count(),
            1
        );
        assert_eq!(runtime.ids.len(), 5);
    }
    #[test]
    fn identity_keys_are_collision_free_with_delimiters() {
        // Panel IDs must be namespaced and cannot contain `:`. Instance keys can.
        let left = PanelInstanceId::keyed("test.a", "b:c");
        let right = PanelInstanceId::keyed("test.a.b", "c");
        let same_panel_other_instance = PanelInstanceId::keyed("test.a", "b");
        assert_ne!(instance_key(&left), instance_key(&right));
        assert_ne!(
            instance_key(&left),
            instance_key(&same_panel_other_instance)
        );
        assert_ne!(
            instance_key(&right),
            instance_key(&same_panel_other_instance)
        );
        assert_eq!(instance_key(&left), "6:test.a:b:c");
    }
    #[test]
    fn split_and_tab_commands_move_existing_identity() {
        let mut runtime = populated();
        let left = PanelInstanceId::singleton("test.left");
        let right = PanelInstanceId::singleton("test.right");
        assert!(runtime.dock_relative(&left, &right, DockDrop::Tab));
        assert!(runtime.dock_relative(&left, &right, DockDrop::Left));
        assert_eq!(runtime.dock.ordered_tabs().len(), 4);
        assert_eq!(runtime.ids.len(), 5);
    }
    #[test]
    fn host_transfer_is_atomic_and_does_not_duplicate() {
        let mut runtime = populated();
        let left = PanelInstanceId::singleton("test.left");
        let floating = PanelInstanceId::singleton("test.float");
        assert!(runtime.dock_relative(&left, &floating, DockDrop::Right));
        assert_eq!(runtime.dock.ordered_tabs().len(), 3);
        assert_eq!(
            runtime
                .floating
                .values()
                .flat_map(ModularDockTree::ordered_tabs)
                .count(),
            2
        );
        assert!(runtime.dock_relative(
            &left,
            &PanelInstanceId::singleton("test.center"),
            DockDrop::Tab
        ));
        assert_eq!(runtime.dock.ordered_tabs().len(), 4);
        assert_eq!(
            runtime
                .floating
                .values()
                .flat_map(ModularDockTree::ordered_tabs)
                .count(),
            1
        );
    }
    #[test]
    fn singleton_hide_and_reopen_preserve_id() {
        let mut runtime = populated();
        let left = PanelInstanceId::singleton("test.left");
        assert!(runtime.hide_instance(&left));
        assert_eq!(runtime.dock.ordered_tabs().len(), 3);
        runtime.open_panel(PanelId::from("test.left"));
        assert_eq!(runtime.dock.ordered_tabs().len(), 4);
        assert_eq!(runtime.ids.len(), 5);
    }
    #[test]
    fn closing_a_floating_host_returns_panels_to_main() {
        let mut runtime = populated();
        let left = PanelInstanceId::singleton("test.left");
        let floating = PanelInstanceId::singleton("test.float");
        assert!(runtime.dock_relative(&left, &floating, DockDrop::Tab));
        let host = instance_key(&floating);
        runtime.redock_host(&host);
        assert_eq!(runtime.floating.len(), 0);
        assert_eq!(runtime.dock.ordered_tabs().len(), 5);
    }
    #[test]
    fn saved_layout_rejects_unknown_panels_and_unrelated_host_claims() {
        let mut runtime = populated();
        let mut saved = runtime.surfaces().to_vec();
        saved.push(ModularSurfaceState::new(
            "injected",
            "Injected",
            SurfaceDock::Floating,
        ));
        let center = instance_key(&PanelInstanceId::singleton("test.center"));
        saved
            .iter_mut()
            .find(|s| s.id == center)
            .unwrap()
            .floating_host = Some("injected".into());
        let main = runtime.dock.clone();
        let float = runtime.floating.clone();
        runtime.restore_layout_with_surfaces(&saved, main, float);
        assert_eq!(runtime.ids.len(), 5);
        assert_eq!(runtime.dock.ordered_tabs().len(), 4);
        assert_eq!(
            runtime
                .floating
                .values()
                .flat_map(ModularDockTree::ordered_tabs)
                .count(),
            1
        );
    }
    #[test]
    fn docked_only_panel_cannot_be_floated_by_drop() {
        let mut runtime = ForgeGuiRuntime::default();
        let mut def = definition("test.fixed", PreferredDock::Center);
        def.host_mode = forge_gui_core::HostMode::Docked;
        runtime.register_panel(def, |_| ProbePanel).unwrap();
        runtime.open_panel(PanelId::from("test.fixed"));
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400.0, 300.0));
        assert!(!runtime.float_instance(&PanelInstanceId::singleton("test.fixed"), rect));
    }
}
