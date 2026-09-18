//! Default egui/egui_dock adapter for ForgeGUI.
#![forbid(unsafe_code)]

use egui::{Color32, RichText, Stroke, Ui};
use egui_dock::{DockArea, DockState, NodeIndex, TabViewer};
use forge_gui_core::{
    CommandId, ForgeGuiError, GuiResult, InstancePolicy, PanelDefinition, PanelId, PanelInstanceId,
    PreferredDock, Rgba, ThemeTokens,
};
use std::collections::BTreeMap;

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

/// One dock tree owns all panel surfaces. Floating surfaces are egui windows;
/// the OS-native multi-viewport bridge is a separate host-level responsibility.
/// There are deliberately no parallel structural rail tab containers.
pub struct ForgeGuiRuntime {
    pub theme: ThemeTokens,
    registry: EguiPanelRegistry,
    instances: PanelInstances,
    dock: DockState<PanelInstanceId>,
    events: Vec<GuiEvent>,
    next_instance: u64,
}

impl Default for ForgeGuiRuntime {
    fn default() -> Self {
        Self {
            theme: ThemeTokens::default(),
            registry: EguiPanelRegistry::default(),
            instances: PanelInstances::default(),
            dock: DockState::new(Vec::new()),
            events: Vec::new(),
            next_instance: 1,
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
        let preferred_dock = definition.preferred_dock;
        let is_singleton = matches!(&policy, InstancePolicy::Singleton);

        let instance = match &policy {
            InstancePolicy::Singleton => PanelInstanceId::singleton(panel.clone()),
            InstancePolicy::Multiple { max_instances } => {
                if let Some(max) = *max_instances {
                    let open_count = self.open_instance_count(&panel);
                    if open_count >= max as usize {
                        return;
                    }
                }
                let key = format!("instance-{}", self.next_instance);
                self.next_instance += 1;
                PanelInstanceId::keyed(panel.clone(), key)
            }
        };

        if is_singleton && self.contains_instance(&instance) {
            self.activate_instance(&instance);
            return;
        }

        self.insert_by_preferred_dock(instance, preferred_dock);
    }

    fn open_instance_count(&self, panel: &PanelId) -> usize {
        self.dock
            .iter_all_tabs()
            .filter(|(_, tab)| &tab.panel == panel)
            .count()
    }

    fn contains_instance(&self, instance: &PanelInstanceId) -> bool {
        self.dock.find_tab(instance).is_some()
    }

    fn activate_instance(&mut self, instance: &PanelInstanceId) {
        // A singleton can be relocated by the user. Its current location always
        // wins over its definition's original preferred dock.
        if let Some(path) = self.dock.find_tab(instance) {
            let _ = self.dock.set_active_tab(path);
            self.dock.set_focused_node_and_surface(path.node_path());
        }
    }

    fn insert_by_preferred_dock(&mut self, instance: PanelInstanceId, dock: PreferredDock) {
        if dock == PreferredDock::Floating {
            self.dock.add_window(vec![instance]);
            return;
        }

        // Group new tabs with an existing panel of the same default region, if
        // it is still docked in the main surface. User rearrangements stay intact.
        let existing = self.dock.main_surface().find_tab_from(|tab| {
            self.registry
                .definition(&tab.panel)
                .is_some_and(|definition| definition.preferred_dock == dock)
        });
        if let Some((node, _)) = existing {
            self.dock.main_surface_mut()[node].append_tab(instance);
            return;
        }

        if self.dock.main_surface().num_tabs() == 0 {
            // Do not reconstruct DockState here: floating windows may already
            // exist and must never disappear when a new docked tab is opened.
            self.dock.main_surface_mut().push_to_first_leaf(instance);
            return;
        }

        let root = NodeIndex::root();
        match dock {
            PreferredDock::Left => {
                self.dock
                    .main_surface_mut()
                    .split_left(root, 0.78, vec![instance]);
            }
            PreferredDock::Right => {
                self.dock
                    .main_surface_mut()
                    .split_right(root, 0.78, vec![instance]);
            }
            PreferredDock::Bottom => {
                self.dock
                    .main_surface_mut()
                    .split_below(root, 0.75, vec![instance]);
            }
            PreferredDock::Center => {
                // The normal startup opens the document/canvas first. For
                // consumers that open a navigation rail first, place the
                // center opposite the existing rail where possible.
                let existing_side = [
                    PreferredDock::Right,
                    PreferredDock::Bottom,
                    PreferredDock::Left,
                ]
                .into_iter()
                .find(|side| {
                    self.dock
                        .main_surface()
                        .find_tab_from(|tab| {
                            self.registry
                                .definition(&tab.panel)
                                .is_some_and(|definition| definition.preferred_dock == *side)
                        })
                        .is_some()
                });
                match existing_side {
                    Some(PreferredDock::Right) => {
                        self.dock
                            .main_surface_mut()
                            .split_left(root, 0.25, vec![instance]);
                    }
                    Some(PreferredDock::Bottom) => {
                        self.dock
                            .main_surface_mut()
                            .split_above(root, 0.25, vec![instance]);
                    }
                    _ => {
                        self.dock
                            .main_surface_mut()
                            .split_right(root, 0.25, vec![instance]);
                    }
                }
            }
            PreferredDock::Floating => unreachable!("floating was handled above"),
        }
    }

    pub fn has_panel(&self, panel: &PanelId) -> bool {
        self.dock
            .iter_all_tabs()
            .any(|(_, tab)| &tab.panel == panel)
    }

    /// Public dock surface access for project-owned save/restore and explicit
    /// layout commands. The runtime remains the sole owner of this state.
    pub fn dock_state(&self) -> &DockState<PanelInstanceId> {
        &self.dock
    }

    pub fn dock_state_mut(&mut self) -> &mut DockState<PanelInstanceId> {
        &mut self.dock
    }

    /// Move an already-open panel to an in-app floating window. This is not yet
    /// an independent OS-native viewport; that requires the host window bridge.
    pub fn float_instance(&mut self, instance: &PanelInstanceId, rect: egui::Rect) -> bool {
        let Some(path) = self.dock.find_tab(instance) else {
            return false;
        };
        if self
            .registry
            .definition(&instance.panel)
            .is_some_and(|def| {
                matches!(
                    def.host_mode,
                    forge_gui_core::HostMode::Structural | forge_gui_core::HostMode::Docked
                )
            })
        {
            return false;
        }
        self.dock.detach_tab(path, rect);
        true
    }

    pub fn install_theme(&self, ctx: &egui::Context) {
        install_theme(ctx, &self.theme);
    }

    pub fn show(&mut self, ui: &mut Ui) {
        let Self {
            theme,
            registry,
            instances,
            dock,
            events,
            ..
        } = self;

        let mut viewer = Viewer {
            registry,
            instances,
            events,
            theme,
        };
        DockArea::new(dock).show_inside(ui, &mut viewer);

        // When a user closes a dock tab or its floating surface, destroy the
        // corresponding live panel instance, including its input/selection state.
        let open: std::collections::BTreeSet<_> =
            dock.iter_all_tabs().map(|(_, tab)| tab.clone()).collect();
        instances
            .values
            .retain(|instance, _| open.contains(instance));
    }

    pub fn drain_events(&mut self) -> impl Iterator<Item = GuiEvent> + '_ {
        self.events.drain(..)
    }
}

struct Viewer<'a> {
    registry: &'a EguiPanelRegistry,
    instances: &'a mut PanelInstances,
    events: &'a mut Vec<GuiEvent>,
    theme: &'a ThemeTokens,
}

impl TabViewer for Viewer<'_> {
    type Tab = PanelInstanceId;

    fn id(&mut self, tab: &mut Self::Tab) -> egui::Id {
        egui::Id::new(tab)
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        self.registry
            .definition(&tab.panel)
            .map(|definition| RichText::new(&definition.title).into())
            .unwrap_or_else(|| RichText::new(tab.panel.as_str()).into())
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        self.instances
            .render(self.registry, tab, ui, self.events, self.theme);
    }

    fn allowed_in_windows(&self, tab: &mut Self::Tab) -> bool {
        self.registry
            .definition(&tab.panel)
            .is_some_and(|definition| {
                !matches!(
                    definition.host_mode,
                    forge_gui_core::HostMode::Structural | forge_gui_core::HostMode::Docked
                )
            })
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

    #[test]
    fn every_preferred_dock_uses_one_tree_and_floating_is_a_real_surface() {
        let mut runtime = ForgeGuiRuntime::default();
        for (id, preferred) in [
            ("test.center", PreferredDock::Center),
            ("test.left", PreferredDock::Left),
            ("test.right", PreferredDock::Right),
            ("test.bottom", PreferredDock::Bottom),
            ("test.float", PreferredDock::Floating),
        ] {
            runtime
                .register_panel(definition(id, preferred), |_| ProbePanel)
                .unwrap();
            runtime.open_panel(PanelId::from(id));
        }
        assert_eq!(runtime.dock.iter_all_tabs().count(), 5);
        assert_eq!(runtime.dock.main_surface().num_tabs(), 4);
        let floating = runtime
            .dock
            .find_tab(&PanelInstanceId::singleton("test.float"))
            .unwrap();
        assert!(!floating.surface.is_main());
    }

    #[test]
    fn singleton_reopen_never_moves_or_duplicates_a_floated_panel() {
        let mut runtime = ForgeGuiRuntime::default();
        runtime
            .register_panel(definition("test.single", PreferredDock::Left), |_| {
                ProbePanel
            })
            .unwrap();
        runtime.open_panel(PanelId::from("test.single"));
        let instance = PanelInstanceId::singleton("test.single");
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(350.0, 300.0));
        assert!(runtime.float_instance(&instance, rect));
        runtime.open_panel(PanelId::from("test.single"));
        assert_eq!(runtime.dock.iter_all_tabs().count(), 1);
        assert!(!runtime.dock.find_tab(&instance).unwrap().surface.is_main());
    }

    #[test]
    fn a_new_docked_panel_cannot_erase_an_existing_floating_surface() {
        let mut runtime = ForgeGuiRuntime::default();
        runtime
            .register_panel(definition("test.float", PreferredDock::Floating), |_| {
                ProbePanel
            })
            .unwrap();
        runtime
            .register_panel(definition("test.center", PreferredDock::Center), |_| {
                ProbePanel
            })
            .unwrap();
        runtime.open_panel(PanelId::from("test.float"));
        runtime.open_panel(PanelId::from("test.center"));
        assert_eq!(runtime.dock.iter_all_tabs().count(), 2);
        assert_eq!(runtime.dock.main_surface().num_tabs(), 1);
    }

    #[test]
    fn multi_instance_limit_includes_floating_instances() {
        let mut runtime = ForgeGuiRuntime::default();
        let mut def = definition("test.multi", PreferredDock::Floating);
        def.instance_policy = InstancePolicy::Multiple {
            max_instances: Some(2),
        };
        runtime.register_panel(def, |_| ProbePanel).unwrap();
        runtime.open_panel(PanelId::from("test.multi"));
        runtime.open_panel(PanelId::from("test.multi"));
        runtime.open_panel(PanelId::from("test.multi"));
        assert_eq!(runtime.dock.iter_all_tabs().count(), 2);
    }
}
