//! Backend-neutral accessibility semantics.
#![forbid(unsafe_code)]
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Role {
    Application,
    Window,
    Button,
    Toggle,
    TextField,
    Label,
    List,
    ListItem,
    Tree,
    TreeItem,
    Table,
    Row,
    Cell,
    Menu,
    MenuItem,
    Dialog,
    Progress,
    Status,
    Custom,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct A11yState {
    pub disabled: bool,
    pub selected: bool,
    pub checked: Option<bool>,
    pub expanded: Option<bool>,
    pub busy: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct A11yNode {
    pub id: String,
    pub role: Role,
    pub label: String,
    pub description: String,
    pub state: A11yState,
    pub children: Vec<String>,
}
