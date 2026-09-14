//! Shared Inspector / property-grid rendering for ForgeGUI_Core.
#![forbid(unsafe_code)]

use egui::{Grid, RichText, Ui};
use forge_gui_core::{PropertyObject, PropertyValue};
use forge_gui_theme::ForgeTheme;

#[derive(Clone, Debug, PartialEq)]
pub struct InspectorChange {
    pub object_id: String,
    pub property_id: String,
    pub value: PropertyValue,
}

#[derive(Debug, Default)]
pub struct InspectorResponse {
    pub changes: Vec<InspectorChange>,
}

pub fn show_property_object(
    ui: &mut Ui,
    object: &mut PropertyObject,
    theme: &ForgeTheme,
) -> InspectorResponse {
    let mut response = InspectorResponse::default();

    ui.heading(&object.title);
    ui.label(
        RichText::new(&object.object_id)
            .small()
            .color(color(theme.base.text_muted)),
    );
    ui.separator();

    Grid::new(("forge.inspector.grid", &object.object_id))
        .num_columns(2)
        .spacing([12.0, 8.0])
        .striped(false)
        .show(ui, |ui| {
            for field in &mut object.fields {
                ui.label(&field.label);

                let before = field.value.clone();
                ui.add_enabled_ui(!field.read_only, |ui| match &mut field.value {
                    PropertyValue::Bool(value) => {
                        ui.checkbox(value, "");
                    }
                    PropertyValue::Integer(value) => {
                        ui.add(egui::DragValue::new(value).speed(1.0));
                    }
                    PropertyValue::Float(value) => {
                        ui.add(egui::DragValue::new(value).speed(0.05));
                    }
                    PropertyValue::Text(value) => {
                        ui.add(egui::TextEdit::singleline(value).desired_width(f32::INFINITY));
                    }
                    PropertyValue::Reference(value) => {
                        ui.horizontal(|ui| {
                            ui.add(egui::TextEdit::singleline(value).desired_width(180.0));
                            let _ = ui.small_button("…").on_hover_text("Browse references");
                        });
                    }
                    PropertyValue::None => {
                        ui.label(RichText::new("—").color(color(theme.base.text_muted)));
                    }
                });

                if field.value != before {
                    response.changes.push(InspectorChange {
                        object_id: object.object_id.clone(),
                        property_id: field.id.clone(),
                        value: field.value.clone(),
                    });
                }

                ui.end_row();
            }
        });

    response
}

fn color(value: forge_gui_core::Rgba) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(value.0, value.1, value.2, value.3)
}
