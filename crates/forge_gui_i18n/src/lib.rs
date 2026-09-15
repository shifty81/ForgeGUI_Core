//! Minimal localization catalog and RTL metadata.
#![forbid(unsafe_code)]
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum TextDirection {
    #[default]
    Ltr,
    Rtl,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocaleCatalog {
    pub locale: String,
    pub direction: TextDirection,
    pub messages: BTreeMap<String, String>,
}
#[derive(Default)]
pub struct I18n {
    catalogs: BTreeMap<String, LocaleCatalog>,
    active: String,
}
impl I18n {
    pub fn add(&mut self, c: LocaleCatalog) {
        if self.active.is_empty() {
            self.active = c.locale.clone();
        }
        self.catalogs.insert(c.locale.clone(), c);
    }
    pub fn set_locale(&mut self, locale: &str) -> bool {
        if self.catalogs.contains_key(locale) {
            self.active = locale.into();
            true
        } else {
            false
        }
    }
    pub fn tr<'a>(&'a self, key: &'a str) -> &'a str {
        self.catalogs
            .get(&self.active)
            .and_then(|c| c.messages.get(key))
            .map(String::as_str)
            .unwrap_or(key)
    }
    pub fn direction(&self) -> TextDirection {
        self.catalogs
            .get(&self.active)
            .map(|c| c.direction)
            .unwrap_or_default()
    }
}
