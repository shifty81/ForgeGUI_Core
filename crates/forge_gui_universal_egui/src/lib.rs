//! egui presentation for the universal ForgeGUI certification suite.
#![forbid(unsafe_code)]
#![allow(clippy::field_reassign_with_default)]

use egui::{RichText, Ui};
use forge_gui_app::{AppPolicy, AppProfile, Breakpoint};
use forge_gui_data::{CollectionSelection, CollectionView};
use forge_gui_dialog::{DialogAction, DialogKind, DialogManager, DialogSpec};
use forge_gui_forms::{FieldKind, FieldSpec, FormModel, FormValue};
use forge_gui_notify::{NotificationCenter, NotificationRecord, NotificationSeverity};
use forge_gui_services::{
    CommandRegistry, CommandSpec, CommandState, TaskRecord, TaskRegistry, TaskState,
};
use forge_gui_theme::ForgeTheme;
use forge_gui_widgets::{badge, progress_bar, search_box, WidgetTone};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UniversalTab {
    #[default]
    Overview,
    Controls,
    Forms,
    Data,
    Commands,
    Dialogs,
    Tasks,
    Notifications,
    Settings,
    States,
}
impl UniversalTab {
    pub const ALL: [Self; 10] = [
        Self::Overview,
        Self::Controls,
        Self::Forms,
        Self::Data,
        Self::Commands,
        Self::Dialogs,
        Self::Tasks,
        Self::Notifications,
        Self::Settings,
        Self::States,
    ];
    pub const fn label(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Controls => "Controls",
            Self::Forms => "Forms",
            Self::Data => "Data",
            Self::Commands => "Commands",
            Self::Dialogs => "Dialogs",
            Self::Tasks => "Tasks",
            Self::Notifications => "Notifications",
            Self::Settings => "Settings",
            Self::States => "States",
        }
    }
}

#[derive(Clone, Debug)]
struct DemoRecord {
    id: String,
    name: String,
    kind: String,
    status: String,
}

pub struct UniversalSuiteState {
    pub tab: UniversalTab,
    pub profile: AppProfile,
    pub search: String,
    pub command_search: String,
    pub selection: CollectionSelection,
    pub data_view: CollectionView,
    pub form: FormModel,
    pub commands: CommandRegistry,
    pub dialogs: DialogManager,
    pub tasks: TaskRegistry,
    pub notifications: NotificationCenter,
    pub compact_navigation: bool,
    pub diagnostics: bool,
    pub auto_save: bool,
    pub scale: f32,
    records: Vec<DemoRecord>,
}

impl Default for UniversalSuiteState {
    fn default() -> Self {
        let mut form = FormModel::default();
        form.fields = vec![
            FieldSpec {
                id: "name".into(),
                label: "Display name".into(),
                description: "Required generic text field".into(),
                kind: FieldKind::Text,
                required: true,
                read_only: false,
                placeholder: Some("Enter a name".into()),
                min: None,
                max: None,
                allowed: vec![],
            },
            FieldSpec {
                id: "mode".into(),
                label: "Mode".into(),
                description: "Generic choice field".into(),
                kind: FieldKind::Choice,
                required: true,
                read_only: false,
                placeholder: None,
                min: None,
                max: None,
                allowed: vec!["Standard".into(), "Compact".into(), "Comfortable".into()],
            },
            FieldSpec {
                id: "limit".into(),
                label: "Limit".into(),
                description: "Validated numeric field".into(),
                kind: FieldKind::Integer,
                required: false,
                read_only: false,
                placeholder: None,
                min: Some(1.0),
                max: Some(100.0),
                allowed: vec![],
            },
        ];
        form.values
            .insert("name".into(), FormValue::Text("ForgeGUI Consumer".into()));
        form.values
            .insert("mode".into(), FormValue::Choice("Standard".into()));
        form.values.insert("limit".into(), FormValue::Integer(24));
        form.commit();

        let mut commands = CommandRegistry::default();
        for (id, label, category, shortcut) in [
            ("app.new", "New", "File", Some("Ctrl+N")),
            ("app.save", "Save", "File", Some("Ctrl+S")),
            ("app.search", "Search", "Navigate", Some("Ctrl+K")),
            ("app.settings", "Settings", "Application", Some("Ctrl+,")),
        ] {
            commands
                .register(CommandSpec {
                    id: id.into(),
                    label: label.into(),
                    category: category.into(),
                    shortcut: shortcut.map(str::to_owned),
                    aliases: Vec::new(),
                    state: CommandState::enabled(),
                })
                .expect("unique demo command");
        }

        let mut tasks = TaskRegistry::default();
        tasks.upsert(TaskRecord {
            id: "task.index".into(),
            label: "Indexing records".into(),
            state: TaskState::Running,
            progress: Some(0.62),
            detail: "Provider-backed background task".into(),
            cancellable: true,
        });
        tasks.upsert(TaskRecord {
            id: "task.sync".into(),
            label: "Synchronize settings".into(),
            state: TaskState::Queued,
            progress: None,
            detail: "Waiting".into(),
            cancellable: true,
        });

        let mut notifications = NotificationCenter::default();
        notifications.push(NotificationRecord::simple(
            "notice.ready",
            "Application ready",
            "Universal application services initialized",
            NotificationSeverity::Success,
        ));
        notifications.push(NotificationRecord::simple(
            "notice.info",
            "Background activity",
            "One demonstration task is running",
            NotificationSeverity::Info,
        ));

        Self {
            tab: UniversalTab::Overview,
            profile: AppProfile::Standard,
            search: String::new(),
            command_search: String::new(),
            selection: CollectionSelection::default(),
            data_view: CollectionView::Table,
            form,
            commands,
            dialogs: DialogManager::default(),
            tasks,
            notifications,
            compact_navigation: false,
            diagnostics: true,
            auto_save: true,
            scale: 1.0,
            records: demo_records(),
        }
    }
}

pub struct UniversalSuiteResponse {
    pub last_action: Option<String>,
}

pub fn show_universal_suite(
    ui: &mut Ui,
    state: &mut UniversalSuiteState,
    theme: &ForgeTheme,
) -> UniversalSuiteResponse {
    let mut last_action = None;
    ui.horizontal_wrapped(|ui| {
        for tab in UniversalTab::ALL {
            if ui.selectable_label(state.tab == tab, tab.label()).clicked() {
                state.tab = tab;
            }
        }
    });
    ui.separator();
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| match state.tab {
            UniversalTab::Overview => show_overview(ui, state, theme),
            UniversalTab::Controls => show_controls(ui, state, theme),
            UniversalTab::Forms => show_forms(ui, state, theme),
            UniversalTab::Data => show_data(ui, state, theme),
            UniversalTab::Commands => show_commands(ui, state, theme, &mut last_action),
            UniversalTab::Dialogs => show_dialogs(ui, state, theme, &mut last_action),
            UniversalTab::Tasks => show_tasks(ui, state, theme),
            UniversalTab::Notifications => show_notifications(ui, state, theme),
            UniversalTab::Settings => show_settings(ui, state, theme),
            UniversalTab::States => show_states(ui, state, theme),
        });
    UniversalSuiteResponse { last_action }
}

fn section(ui: &mut Ui, title: &str, subtitle: &str) {
    ui.add_space(4.0);
    ui.heading(title);
    ui.label(RichText::new(subtitle).small().weak());
    ui.add_space(6.0);
}
fn show_overview(ui: &mut Ui, state: &mut UniversalSuiteState, theme: &ForgeTheme) {
    section(ui,"Universal application certification","The same contracts are intended for utilities, dashboards, settings applications, data tools, chat/control applications and advanced workspaces.");
    let policy = AppPolicy::for_profile(state.profile);
    ui.horizontal_wrapped(|ui| {
        for profile in [
            AppProfile::Minimal,
            AppProfile::Standard,
            AppProfile::Dashboard,
            AppProfile::Utility,
            AppProfile::Workspace,
            AppProfile::Kiosk,
        ] {
            if ui
                .selectable_label(state.profile == profile, format!("{profile:?}"))
                .clicked()
            {
                state.profile = profile;
            }
        }
    });
    ui.add_space(8.0);
    egui::Grid::new("forge.universal.profile.grid")
        .num_columns(2)
        .striped(true)
        .show(ui, |ui| {
            ui.label("Menu");
            ui.label(if policy.show_menu {
                "Enabled"
            } else {
                "Hidden"
            });
            ui.end_row();
            ui.label("Navigation");
            ui.label(if policy.show_navigation {
                "Enabled"
            } else {
                "Hidden"
            });
            ui.end_row();
            ui.label("Status");
            ui.label(if policy.show_status {
                "Enabled"
            } else {
                "Hidden"
            });
            ui.end_row();
            ui.label("Multi-window");
            ui.label(if policy.allow_multi_window {
                "Allowed"
            } else {
                "Disabled"
            });
            ui.end_row();
            ui.label("Current breakpoint");
            ui.label(format!(
                "{:?}",
                Breakpoint::from_width(ui.available_width())
            ));
            ui.end_row();
        });
    ui.add_space(10.0);
    ui.horizontal_wrapped(|ui| {
        let _ = badge(ui, "Generic", WidgetTone::Accent, theme);
        let _ = badge(ui, "Provider-backed", WidgetTone::Success, theme);
        let _ = badge(ui, "Renderer-neutral contracts", WidgetTone::Neutral, theme);
    });
}
fn show_controls(ui: &mut Ui, state: &mut UniversalSuiteState, theme: &ForgeTheme) {
    section(
        ui,
        "Controls",
        "Normal desktop application controls and visual states.",
    );
    ui.horizontal_wrapped(|ui| {
        let _ = ui.button("Primary action");
        let _ = ui.button("Secondary");
        ui.add_enabled(false, egui::Button::new("Disabled"));
        let _ = badge(ui, "Info", WidgetTone::Accent, theme);
        let _ = badge(ui, "Warning", WidgetTone::Warning, theme);
    });
    ui.checkbox(&mut state.auto_save, "Toggle / checkbox");
    ui.add(egui::Slider::new(&mut state.scale, 0.5..=2.0).text("Scale"));
    let _ = progress_bar(
        ui,
        0.72,
        Some("Determinate progress · 72%"),
        WidgetTone::Accent,
        theme,
    );
    ui.add(egui::Spinner::new());
}
fn show_forms(ui: &mut Ui, state: &mut UniversalSuiteState, theme: &ForgeTheme) {
    section(
        ui,
        "Forms and validation",
        "Schema-driven fields remain host-neutral.",
    );
    for field in state.form.fields.clone() {
        ui.label(RichText::new(&field.label).strong());
        match field.kind {
            FieldKind::Text => {
                let mut v = match state.form.value(&field.id) {
                    Some(FormValue::Text(s)) => s.clone(),
                    _ => String::new(),
                };
                if ui.text_edit_singleline(&mut v).changed() {
                    state.form.set(&field.id, FormValue::Text(v));
                }
            }
            FieldKind::Choice => {
                let mut v = match state.form.value(&field.id) {
                    Some(FormValue::Choice(s)) => s.clone(),
                    _ => String::new(),
                };
                egui::ComboBox::from_id_salt(format!("choice.{}", field.id))
                    .selected_text(&v)
                    .show_ui(ui, |ui| {
                        for choice in &field.allowed {
                            ui.selectable_value(&mut v, choice.clone(), choice);
                        }
                    });
                state.form.set(&field.id, FormValue::Choice(v));
            }
            FieldKind::Integer => {
                let mut v = match state.form.value(&field.id) {
                    Some(FormValue::Integer(n)) => *n,
                    _ => 0,
                };
                if ui.add(egui::DragValue::new(&mut v)).changed() {
                    state.form.set(&field.id, FormValue::Integer(v));
                }
            }
            _ => {
                ui.label("Field presentation available through host adapter");
            }
        }
        if !field.description.is_empty() {
            ui.label(RichText::new(&field.description).small().weak());
        }
        ui.add_space(5.0);
    }
    for message in state.form.validate() {
        ui.colored_label(
            color(theme.base.danger),
            format!("{}: {}", message.field_id, message.message),
        );
    }
    ui.horizontal(|ui| {
        if ui
            .add_enabled(state.form.is_dirty(), egui::Button::new("Apply"))
            .clicked()
        {
            state.form.commit();
        }
        if ui
            .add_enabled(state.form.is_dirty(), egui::Button::new("Reset"))
            .clicked()
        {
            state.form.reset();
        }
        ui.label(if state.form.is_dirty() {
            "Modified"
        } else {
            "Saved"
        });
    });
}
fn show_data(ui: &mut Ui, state: &mut UniversalSuiteState, theme: &ForgeTheme) {
    section(
        ui,
        "Provider-backed data",
        "Selection survives presentation changes between Tree, List, Grid and Table concepts.",
    );
    let _ = search_box(ui, &mut state.search, "Filter records…", theme);
    ui.horizontal(|ui| {
        for v in [
            CollectionView::Tree,
            CollectionView::List,
            CollectionView::Grid,
            CollectionView::Table,
        ] {
            if ui
                .selectable_label(state.data_view == v, format!("{v:?}"))
                .clicked()
            {
                state.data_view = v;
            }
        }
    });
    ui.separator();
    let filtered: Vec<_> = state
        .records
        .iter()
        .filter(|r| {
            state.search.trim().is_empty()
                || r.name.to_lowercase().contains(&state.search.to_lowercase())
                || r.kind.to_lowercase().contains(&state.search.to_lowercase())
        })
        .cloned()
        .collect();
    egui::Grid::new("forge.universal.data.grid")
        .num_columns(4)
        .striped(true)
        .show(ui, |ui| {
            ui.strong("Name");
            ui.strong("Type");
            ui.strong("Status");
            ui.strong("Select");
            ui.end_row();
            for r in filtered {
                ui.label(&r.name);
                ui.label(&r.kind);
                ui.label(&r.status);
                let selected = state.selection.contains(&r.id);
                if ui
                    .selectable_label(selected, if selected { "Selected" } else { "Select" })
                    .clicked()
                {
                    state.selection.select_single(r.id.clone());
                }
                ui.end_row();
            }
        });
}
fn show_commands(
    ui: &mut Ui,
    state: &mut UniversalSuiteState,
    theme: &ForgeTheme,
    last: &mut Option<String>,
) {
    section(ui,"Command Registry","Menus, shortcuts, palettes, context actions and automation should dispatch the same command IDs.");
    ui.text_edit_singleline(&mut state.command_search);
    for c in state.commands.search(&state.command_search) {
        ui.horizontal(|ui| {
            if ui
                .add_enabled(c.state.enabled, egui::Button::new(&c.label))
                .clicked()
            {
                *last = Some(format!("Command: {}", c.id));
            }
            ui.label(RichText::new(&c.category).small().weak());
            if let Some(s) = &c.shortcut {
                ui.label(RichText::new(s).monospace().small());
            }
        });
    }
    let conflicts = state.commands.shortcut_conflicts();
    if !conflicts.is_empty() {
        ui.colored_label(
            color(theme.base.danger),
            format!("{} shortcut conflict(s)", conflicts.len()),
        );
    }
}
fn show_dialogs(
    ui: &mut Ui,
    state: &mut UniversalSuiteState,
    _theme: &ForgeTheme,
    last: &mut Option<String>,
) {
    section(
        ui,
        "Dialogs and flows",
        "Reusable modal/modeless state without host-domain assumptions.",
    );
    if ui.button("Open confirmation").clicked() {
        state.dialogs.push(DialogSpec {
            id: "demo.confirm".into(),
            title: "Confirm action".into(),
            message: "This is a generic confirmation dialog.".into(),
            kind: DialogKind::Confirm,
            modal: true,
            actions: vec![
                DialogAction {
                    id: "cancel".into(),
                    label: "Cancel".into(),
                    primary: false,
                    destructive: false,
                },
                DialogAction {
                    id: "confirm".into(),
                    label: "Confirm".into(),
                    primary: true,
                    destructive: false,
                },
            ],
        });
    }
    if let Some(d) = state.dialogs.active().cloned() {
        let ctx = ui.ctx().clone();
        let mut resolved = None;
        egui::Window::new(&d.title)
            .id(egui::Id::new(&d.id))
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(&ctx, |ui| {
                ui.label(&d.message);
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    for a in &d.actions {
                        if ui.button(&a.label).clicked() {
                            resolved = Some(a.id.clone());
                        }
                    }
                });
            });
        if let Some(action) = resolved {
            *last = Some(format!("Dialog action: {action}"));
            state.dialogs.resolve();
        }
    }
}
fn show_tasks(ui: &mut Ui, state: &mut UniversalSuiteState, theme: &ForgeTheme) {
    section(
        ui,
        "Background tasks",
        "Queued/running/completed work shares one generic representation.",
    );
    ui.label(format!("{} active", state.tasks.active_count()));
    for t in state.tasks.iter() {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.strong(&t.label);
                ui.label(format!("{:?}", t.state));
            });
            ui.label(RichText::new(&t.detail).small().weak());
            if let Some(p) = t.progress {
                let _ = progress_bar(
                    ui,
                    p,
                    Some(&format!("{:.0}%", p * 100.0)),
                    WidgetTone::Accent,
                    theme,
                );
            }
        });
    }
}
fn show_notifications(ui: &mut Ui, state: &mut UniversalSuiteState, _theme: &ForgeTheme) {
    section(
        ui,
        "Notification center",
        "Transient and persistent application feedback.",
    );
    ui.horizontal(|ui| {
        ui.label(format!("{} unread", state.notifications.unread_count()));
        if ui.button("Mark all read").clicked() {
            state.notifications.mark_all_read();
        }
    });
    for n in state.notifications.records() {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.strong(&n.title);
                ui.label(format!("{:?}", n.severity));
            });
            ui.label(&n.message);
        });
    }
}
fn show_settings(ui: &mut Ui, state: &mut UniversalSuiteState, _theme: &ForgeTheme) {
    section(
        ui,
        "Settings",
        "A normal application-settings surface, not an editor inspector.",
    );
    ui.checkbox(&mut state.auto_save, "Autosave changes");
    ui.checkbox(&mut state.compact_navigation, "Compact navigation");
    ui.checkbox(&mut state.diagnostics, "Enable diagnostics");
    ui.add(egui::Slider::new(&mut state.scale, 0.75..=1.5).text("UI scale"));
}
fn show_states(ui: &mut Ui, _state: &mut UniversalSuiteState, theme: &ForgeTheme) {
    section(
        ui,
        "Interaction states",
        "Certification matrix for visual consistency.",
    );
    for (label, enabled, tone) in [
        ("Normal", true, WidgetTone::Neutral),
        ("Accent", true, WidgetTone::Accent),
        ("Success", true, WidgetTone::Success),
        ("Warning", true, WidgetTone::Warning),
        ("Danger", true, WidgetTone::Danger),
        ("Disabled", false, WidgetTone::Neutral),
    ] {
        ui.horizontal(|ui| {
            ui.add_enabled(enabled, egui::Button::new(label));
            let _ = badge(ui, label, tone, theme);
        });
    }
}
fn demo_records() -> Vec<DemoRecord> {
    (1..=18)
        .map(|i| DemoRecord {
            id: format!("record.{i:03}"),
            name: format!("Record {i:02}"),
            kind: if i % 3 == 0 {
                "Group".into()
            } else {
                "Item".into()
            },
            status: if i % 4 == 0 {
                "Warning".into()
            } else {
                "Ready".into()
            },
        })
        .collect()
}

fn color(value: forge_gui_core::Rgba) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(value.0, value.1, value.2, value.3)
}
