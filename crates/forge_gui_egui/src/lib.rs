//! Default egui/egui_dock adapter for ForgeGUI.
#![forbid(unsafe_code)]

use egui::{Color32, RichText, Stroke, Ui};
use egui_dock::{DockArea, DockState, TabViewer};
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

#[derive(Default)]
struct StructuralRail {
    tabs: Vec<PanelInstanceId>,
    active: usize,
}

impl StructuralRail {
    fn open(&mut self, instance: PanelInstanceId) {
        if let Some(index) = self.tabs.iter().position(|tab| tab == &instance) {
            self.active = index;
            return;
        }
        self.tabs.push(instance);
        self.active = self.tabs.len().saturating_sub(1);
    }

    fn contains(&self, instance: &PanelInstanceId) -> bool {
        self.tabs.iter().any(|tab| tab == instance)
    }

    fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }
}

pub struct ForgeGuiRuntime {
    pub theme: ThemeTokens,
    registry: EguiPanelRegistry,
    instances: PanelInstances,
    center: DockState<PanelInstanceId>,
    left: StructuralRail,
    right: StructuralRail,
    bottom: StructuralRail,
    events: Vec<GuiEvent>,
    next_instance: u64,
}

impl Default for ForgeGuiRuntime {
    fn default() -> Self {
        Self {
            theme: ThemeTokens::default(),
            registry: EguiPanelRegistry::default(),
            instances: PanelInstances::default(),
            center: DockState::new(Vec::new()),
            left: StructuralRail::default(),
            right: StructuralRail::default(),
            bottom: StructuralRail::default(),
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
            self.activate_instance(&instance, preferred_dock);
            return;
        }

        self.insert_by_preferred_dock(instance, preferred_dock);
    }

    fn open_instance_count(&self, panel: &PanelId) -> usize {
        let mut count = self
            .left
            .tabs
            .iter()
            .chain(self.right.tabs.iter())
            .chain(self.bottom.tabs.iter())
            .filter(|tab| &tab.panel == panel)
            .count();
        count += self
            .center
            .iter_all_tabs()
            .filter(|(_, tab)| &tab.panel == panel)
            .count();
        count
    }

    fn contains_instance(&self, instance: &PanelInstanceId) -> bool {
        self.left.contains(instance)
            || self.right.contains(instance)
            || self.bottom.contains(instance)
            || self.center.find_tab(instance).is_some()
    }

    fn activate_instance(&mut self, instance: &PanelInstanceId, dock: PreferredDock) {
        match dock {
            PreferredDock::Left => self.left.open(instance.clone()),
            PreferredDock::Right => self.right.open(instance.clone()),
            PreferredDock::Bottom => self.bottom.open(instance.clone()),
            PreferredDock::Center | PreferredDock::Floating => {
                if let Some(path) = self.center.find_tab(instance) {
                    let _ = self.center.set_active_tab(path);
                }
            }
        }
    }

    fn insert_by_preferred_dock(&mut self, instance: PanelInstanceId, dock: PreferredDock) {
        match dock {
            PreferredDock::Left => self.left.open(instance),
            PreferredDock::Right => self.right.open(instance),
            PreferredDock::Bottom => self.bottom.open(instance),
            PreferredDock::Center | PreferredDock::Floating => {
                if self.center.main_surface().num_tabs() == 0 {
                    self.center = DockState::new(vec![instance]);
                } else {
                    self.center
                        .main_surface_mut()
                        .push_to_focused_leaf(instance);
                }
            }
        }
    }

    pub fn has_panel(&self, panel: &PanelId) -> bool {
        self.left.tabs.iter().any(|tab| &tab.panel == panel)
            || self.right.tabs.iter().any(|tab| &tab.panel == panel)
            || self.bottom.tabs.iter().any(|tab| &tab.panel == panel)
            || self
                .center
                .iter_all_tabs()
                .any(|(_, tab)| &tab.panel == panel)
    }

    pub fn install_theme(&self, ctx: &egui::Context) {
        install_theme(ctx, &self.theme);
    }

    pub fn show(&mut self, ui: &mut Ui) {
        let Self {
            theme,
            registry,
            instances,
            center,
            left,
            right,
            bottom,
            events,
            ..
        } = self;

        if !left.is_empty() {
            egui::Panel::left("forgegui-left-rail")
                .default_size(240.0)
                .min_size(160.0)
                .max_size(520.0)
                .resizable(true)
                .show(ui, |ui| {
                    render_structural_rail(left, registry, instances, events, theme, ui)
                });
        }

        if !right.is_empty() {
            egui::Panel::right("forgegui-right-rail")
                .default_size(300.0)
                .min_size(180.0)
                .max_size(620.0)
                .resizable(true)
                .show(ui, |ui| {
                    render_structural_rail(right, registry, instances, events, theme, ui)
                });
        }

        if !bottom.is_empty() {
            egui::Panel::bottom("forgegui-bottom-rail")
                .default_size(230.0)
                .min_size(120.0)
                .max_size(520.0)
                .resizable(true)
                .show(ui, |ui| {
                    render_structural_rail(bottom, registry, instances, events, theme, ui)
                });
        }

        egui::CentralPanel::default().show(ui, |ui| {
            let mut viewer = Viewer {
                registry,
                instances,
                events,
                theme,
            };
            DockArea::new(center).show_inside(ui, &mut viewer);
        });

        let mut open = std::collections::BTreeSet::new();
        for tab in left
            .tabs
            .iter()
            .chain(right.tabs.iter())
            .chain(bottom.tabs.iter())
        {
            open.insert(tab.clone());
        }
        for (_, tab) in center.iter_all_tabs() {
            open.insert(tab.clone());
        }
        instances
            .values
            .retain(|instance, _| open.contains(instance));
    }

    pub fn drain_events(&mut self) -> impl Iterator<Item = GuiEvent> + '_ {
        self.events.drain(..)
    }
}

fn render_structural_rail(
    rail: &mut StructuralRail,
    registry: &EguiPanelRegistry,
    instances: &mut PanelInstances,
    events: &mut Vec<GuiEvent>,
    theme: &ThemeTokens,
    ui: &mut Ui,
) {
    let mut requested = None;
    ui.horizontal_wrapped(|ui| {
        for (index, tab) in rail.tabs.iter().enumerate() {
            let title = registry
                .definition(&tab.panel)
                .map(|definition| definition.title.as_str())
                .unwrap_or_else(|| tab.panel.as_str());
            if ui.selectable_label(index == rail.active, title).clicked() {
                requested = Some(index);
            }
        }
    });
    if let Some(index) = requested {
        rail.active = index;
    }
    ui.separator();
    if let Some(tab) = rail.tabs.get(rail.active).cloned() {
        instances.render(registry, &tab, ui, events, theme);
    }
    let remaining = ui.available_size();
    if remaining.x > 0.0 && remaining.y > 0.0 {
        ui.allocate_space(remaining);
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
    fn preferred_dock_routes_to_structural_rails_and_center() {
        let mut runtime = ForgeGuiRuntime::default();
        runtime
            .register_panel(definition("test.left", PreferredDock::Left), |_| ProbePanel)
            .unwrap();
        runtime
            .register_panel(definition("test.right", PreferredDock::Right), |_| {
                ProbePanel
            })
            .unwrap();
        runtime
            .register_panel(definition("test.bottom", PreferredDock::Bottom), |_| {
                ProbePanel
            })
            .unwrap();
        runtime
            .register_panel(definition("test.center", PreferredDock::Center), |_| {
                ProbePanel
            })
            .unwrap();

        runtime.open_panel(PanelId::from("test.left"));
        runtime.open_panel(PanelId::from("test.right"));
        runtime.open_panel(PanelId::from("test.bottom"));
        runtime.open_panel(PanelId::from("test.center"));

        assert_eq!(runtime.left.tabs.len(), 1);
        assert_eq!(runtime.right.tabs.len(), 1);
        assert_eq!(runtime.bottom.tabs.len(), 1);
        assert_eq!(runtime.center.iter_all_tabs().count(), 1);
    }

    #[test]
    fn multi_instance_limit_is_enforced() {
        let mut runtime = ForgeGuiRuntime::default();
        let mut def = definition("test.multi", PreferredDock::Center);
        def.instance_policy = InstancePolicy::Multiple {
            max_instances: Some(2),
        };
        runtime.register_panel(def, |_| ProbePanel).unwrap();

        runtime.open_panel(PanelId::from("test.multi"));
        runtime.open_panel(PanelId::from("test.multi"));
        runtime.open_panel(PanelId::from("test.multi"));

        assert_eq!(runtime.center.iter_all_tabs().count(), 2);
    }
}
