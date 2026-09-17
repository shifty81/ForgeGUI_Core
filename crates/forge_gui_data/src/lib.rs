//! Generic provider-backed list/tree/grid/table contracts.
#![forbid(unsafe_code)]
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum CollectionView {
    Tree,
    List,
    #[default]
    Grid,
    Table,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Query {
    pub text: String,
    pub filters: Vec<(String, String)>,
    pub sort: Option<(String, SortDirection)>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Item {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub parent: Option<String>,
    pub detail: String,
    pub tags: Vec<String>,
}
pub trait CollectionProvider {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn item(&self, index: usize) -> Option<Item>;
    fn query(&self, q: &Query) -> Vec<Item> {
        let text = q.text.to_lowercase();
        let mut items: Vec<_> = (0..self.len())
            .filter_map(|i| self.item(i))
            .filter(|i| {
                text.is_empty()
                    || i.label.to_lowercase().contains(&text)
                    || i.detail.to_lowercase().contains(&text)
                    || i.tags.iter().any(|t| t.to_lowercase().contains(&text))
            })
            .collect();
        if let Some((key, dir)) = &q.sort {
            items.sort_by(|a, b| {
                let o = match key.as_str() {
                    "kind" => a.kind.cmp(&b.kind),
                    "detail" => a.detail.cmp(&b.detail),
                    _ => a.label.cmp(&b.label),
                };
                if matches!(dir, SortDirection::Descending) {
                    o.reverse()
                } else {
                    o
                }
            });
        }
        items
    }
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CollectionSelection {
    pub ids: Vec<String>,
    pub primary: Option<String>,
}
impl CollectionSelection {
    pub fn select_single(&mut self, id: impl Into<String>) {
        let id = id.into();
        self.ids = vec![id.clone()];
        self.primary = Some(id);
    }
    pub fn clear(&mut self) {
        self.ids.clear();
        self.primary = None;
    }
    pub fn contains(&self, id: &str) -> bool {
        self.ids.iter().any(|x| x == id)
    }
    pub fn len(&self) -> usize {
        self.ids.len()
    }
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Cell {
    Text(String),
    Number(String),
    Boolean(bool),
    Badge(String),
    Empty,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Column {
    pub id: String,
    pub label: String,
    pub visible: bool,
    pub resizable: bool,
    pub min_width: u16,
}
pub trait TabularProvider {
    fn row_count(&self) -> usize;
    fn columns(&self) -> &[Column];
    fn cell(&self, row: usize, column: &str) -> Cell;
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct VecTable {
    pub columns: Vec<Column>,
    pub rows: Vec<Vec<Cell>>,
}
impl TabularProvider for VecTable {
    fn row_count(&self) -> usize {
        self.rows.len()
    }
    fn columns(&self) -> &[Column] {
        &self.columns
    }
    fn cell(&self, row: usize, column: &str) -> Cell {
        let Some(ci) = self.columns.iter().position(|c| c.id == column) else {
            return Cell::Empty;
        };
        self.rows
            .get(row)
            .and_then(|r| r.get(ci))
            .cloned()
            .unwrap_or(Cell::Empty)
    }
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct VirtualRange {
    pub start: usize,
    pub end: usize,
}
impl VirtualRange {
    pub fn for_scroll(
        total: usize,
        row_height: f32,
        scroll: f32,
        viewport: f32,
        overscan: usize,
    ) -> Self {
        if total == 0 || row_height <= 0.0 {
            return Self { start: 0, end: 0 };
        }
        let first = (scroll.max(0.0) / row_height).floor() as usize;
        let visible = (viewport.max(0.0) / row_height).ceil() as usize + 1;
        Self {
            start: first.saturating_sub(overscan),
            end: (first + visible + overscan).min(total),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn virtual_range_clamps() {
        assert_eq!(VirtualRange::for_scroll(10, 20.0, 999.0, 100.0, 2).end, 10);
    }
}
