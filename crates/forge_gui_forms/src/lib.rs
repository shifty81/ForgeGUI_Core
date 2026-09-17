//! Backend-neutral form schema, values and validation.
#![forbid(unsafe_code)]
#![allow(clippy::collapsible_match)]
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum FormValue {
    Empty,
    Bool(bool),
    Integer(i64),
    Number(f64),
    Text(String),
    Choice(String),
    List(Vec<String>),
}
impl FormValue {
    pub fn is_blank(&self) -> bool {
        matches!(self, Self::Empty)
            | matches!(self,Self::Text(s) if s.trim().is_empty())
            | matches!(self,Self::List(v) if v.is_empty())
    }
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum FieldKind {
    Text,
    Password,
    Multiline,
    Boolean,
    Choice,
    SearchChoice,
    Integer,
    Number,
    Slider,
    Date,
    Time,
    DateRange,
    Color,
    Path,
    ReadOnly,
    Custom,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FieldSpec {
    pub id: String,
    pub label: String,
    pub description: String,
    pub kind: FieldKind,
    pub required: bool,
    pub read_only: bool,
    pub placeholder: Option<String>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub allowed: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ValidationSeverity {
    Warning,
    Error,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValidationMessage {
    pub field_id: String,
    pub severity: ValidationSeverity,
    pub message: String,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct FormModel {
    pub fields: Vec<FieldSpec>,
    pub values: BTreeMap<String, FormValue>,
    pub baseline: BTreeMap<String, FormValue>,
}
impl FormModel {
    pub fn set(&mut self, id: &str, v: FormValue) {
        self.values.insert(id.into(), v);
    }
    pub fn value(&self, id: &str) -> Option<&FormValue> {
        self.values.get(id)
    }
    pub fn is_dirty(&self) -> bool {
        self.values != self.baseline
    }
    pub fn commit(&mut self) {
        self.baseline = self.values.clone();
    }
    pub fn reset(&mut self) {
        self.values = self.baseline.clone();
    }
    pub fn validate(&self) -> Vec<ValidationMessage> {
        let mut out = vec![];
        for f in &self.fields {
            let v = self.values.get(&f.id).unwrap_or(&FormValue::Empty);
            if f.required && v.is_blank() {
                out.push(ValidationMessage {
                    field_id: f.id.clone(),
                    severity: ValidationSeverity::Error,
                    message: "Required".into(),
                });
                continue;
            }
            match v {
                FormValue::Integer(n) => {
                    let x = *n as f64;
                    if f.min.is_some_and(|m| x < m) || f.max.is_some_and(|m| x > m) {
                        out.push(ValidationMessage {
                            field_id: f.id.clone(),
                            severity: ValidationSeverity::Error,
                            message: "Out of range".into(),
                        });
                    }
                }
                FormValue::Number(x) => {
                    if f.min.is_some_and(|m| *x < m) || f.max.is_some_and(|m| *x > m) {
                        out.push(ValidationMessage {
                            field_id: f.id.clone(),
                            severity: ValidationSeverity::Error,
                            message: "Out of range".into(),
                        });
                    }
                }
                FormValue::Choice(c) => {
                    if !f.allowed.is_empty() && !f.allowed.iter().any(|x| x == c) {
                        out.push(ValidationMessage {
                            field_id: f.id.clone(),
                            severity: ValidationSeverity::Error,
                            message: "Unsupported value".into(),
                        });
                    }
                }
                _ => {}
            }
            if let (Some(min), Some(max)) = (f.min, f.max) {
                if min > max {
                    out.push(ValidationMessage {
                        field_id: f.id.clone(),
                        severity: ValidationSeverity::Error,
                        message: "Invalid field range".into(),
                    });
                }
            }
        }
        out
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn whitespace_required_is_invalid() {
        let f = FieldSpec {
            id: "x".into(),
            label: "X".into(),
            description: "".into(),
            kind: FieldKind::Text,
            required: true,
            read_only: false,
            placeholder: None,
            min: None,
            max: None,
            allowed: vec![],
        };
        let mut m = FormModel {
            fields: vec![f],
            ..Default::default()
        };
        m.set("x", FormValue::Text("  ".into()));
        assert_eq!(m.validate().len(), 1);
    }
}
