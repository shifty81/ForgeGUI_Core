//! Registered settings pages with staged apply/cancel semantics.
#![forbid(unsafe_code)]
use forge_gui_forms::{FormModel, FormValue};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum SettingScope {
    User,
    Machine,
    Workspace,
    Session,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SettingsPage {
    pub id: String,
    pub label: String,
    pub category: String,
    pub form: FormModel,
}
#[derive(Default)]
pub struct SettingsRegistry {
    pages: BTreeMap<String, SettingsPage>,
    staged: BTreeMap<(SettingScope, String, String), FormValue>,
    committed: BTreeMap<(SettingScope, String, String), FormValue>,
}
impl SettingsRegistry {
    pub fn register(&mut self, p: SettingsPage) -> bool {
        self.pages.insert(p.id.clone(), p).is_none()
    }
    pub fn pages(&self) -> impl Iterator<Item = &SettingsPage> {
        self.pages.values()
    }
    pub fn stage(&mut self, scope: SettingScope, page: &str, key: &str, value: FormValue) {
        self.staged.insert((scope, page.into(), key.into()), value);
    }
    pub fn apply(&mut self) {
        for (k, v) in std::mem::take(&mut self.staged) {
            self.committed.insert(k, v);
        }
    }
    pub fn cancel(&mut self) {
        self.staged.clear();
    }
    pub fn get(&self, scope: SettingScope, page: &str, key: &str) -> Option<&FormValue> {
        self.staged
            .get(&(scope, page.into(), key.into()))
            .or_else(|| self.committed.get(&(scope, page.into(), key.into())))
    }
}
