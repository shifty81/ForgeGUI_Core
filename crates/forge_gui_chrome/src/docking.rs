//! Versioned, renderer-independent layout operations for nested authoring panels.
//!
//! This is the authoritative model for the GUI Lab nested dock renderer. It
//! remains independent of egui_dock/native-window internals so consumers can
//! transact, validate, and persist layouts. Shared consumer shell adapters and
//! recursive trees inside floating native hosts remain separate migration work.

use super::{ModularSurfaceState, SurfaceDock};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

const SCHEMA: u32 = 1;
const MIN_RATIO: f32 = 0.12;
const MAX_RATIO: f32 = 0.88;
const MAX_DEPTH: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SplitAxis {
    Horizontal,
    Vertical,
}

/// Center creates a tab; the other zones create a new, independently sizable
/// leaf adjacent to the target leaf, not a global left/right/bottom region.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DockDrop {
    Tab,
    Left,
    Right,
    Top,
    Bottom,
}

/// Only a visible, unlocked panel from another host may join this OS window.
/// Rejecting self-host drops prevents spurious tab reordering on release.
pub fn can_join_native_host(source: &ModularSurfaceState, host: &str) -> bool {
    !host.is_empty()
        && source.visible
        && !source.locked
        && (source.dock != SurfaceDock::Floating || source.floating_host_id() != host)
}

/// Compute the precise leaf-local zone; never infer a destination from the
/// entire application shell. The GUI Lab renders these edge operations through
/// the recursive tree; native floating-window split hosts are separate work.
pub fn drop_zone(rect: egui::Rect, pointer: egui::Pos2) -> Option<DockDrop> {
    if !rect.min.x.is_finite()
        || !rect.min.y.is_finite()
        || !rect.max.x.is_finite()
        || !rect.max.y.is_finite()
        || rect.width() <= 0.0
        || rect.height() <= 0.0
        || !rect.contains(pointer)
    {
        return None;
    }
    let x = (pointer.x - rect.left()) / rect.width();
    let y = (pointer.y - rect.top()) / rect.height();
    let candidates = [
        (x, DockDrop::Left),
        (1.0 - x, DockDrop::Right),
        (y, DockDrop::Top),
        (1.0 - y, DockDrop::Bottom),
    ];
    let closest = candidates.iter().min_by(|a, b| a.0.total_cmp(&b.0));
    match closest {
        Some((distance, zone)) if *distance < 0.23 => Some(*zone),
        _ => Some(DockDrop::Tab),
    }
}

/// Return the preview that corresponds exactly to the requested drop.
pub fn drop_preview_rect(rect: egui::Rect, zone: DockDrop) -> egui::Rect {
    match zone {
        DockDrop::Tab => rect.shrink(5.0),
        DockDrop::Left => egui::Rect::from_min_max(
            rect.min,
            egui::pos2(rect.left() + rect.width() * 0.30, rect.bottom()),
        ),
        DockDrop::Right => egui::Rect::from_min_max(
            egui::pos2(rect.right() - rect.width() * 0.30, rect.top()),
            rect.max,
        ),
        DockDrop::Top => egui::Rect::from_min_max(
            rect.min,
            egui::pos2(rect.right(), rect.top() + rect.height() * 0.30),
        ),
        DockDrop::Bottom => egui::Rect::from_min_max(
            egui::pos2(rect.left(), rect.bottom() - rect.height() * 0.30),
            rect.max,
        ),
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DockNode {
    Tabs {
        tabs: Vec<String>,
        active: String,
    },
    Split {
        axis: SplitAxis,
        ratio: f32,
        first: Box<DockNode>,
        second: Box<DockNode>,
    },
}

impl DockNode {
    fn single(id: String) -> Self {
        Self::Tabs {
            active: id.clone(),
            tabs: vec![id],
        }
    }

    pub fn contains(&self, id: &str) -> bool {
        match self {
            Self::Tabs { tabs, .. } => tabs.iter().any(|tab| tab == id),
            Self::Split { first, second, .. } => first.contains(id) || second.contains(id),
        }
    }

    fn depth(&self) -> usize {
        match self {
            Self::Tabs { .. } => 0,
            Self::Split { first, second, .. } => 1 + first.depth().max(second.depth()),
        }
    }

    pub fn tabs(&self, into: &mut Vec<String>) {
        match self {
            Self::Tabs { tabs, .. } => into.extend(tabs.iter().cloned()),
            Self::Split { first, second, .. } => {
                first.tabs(into);
                second.tabs(into);
            }
        }
    }

    fn sanitize(
        self,
        valid: &BTreeSet<String>,
        seen: &mut BTreeSet<String>,
        depth: usize,
    ) -> Option<Self> {
        if depth > MAX_DEPTH {
            return None;
        }
        match self {
            Self::Tabs { tabs, active } => {
                let mut clean = Vec::new();
                for id in tabs {
                    if valid.contains(&id) && seen.insert(id.clone()) {
                        clean.push(id);
                    }
                }
                if clean.is_empty() {
                    return None;
                }
                let active = if clean.contains(&active) {
                    active
                } else {
                    clean[0].clone()
                };
                Some(Self::Tabs {
                    tabs: clean,
                    active,
                })
            }
            Self::Split {
                axis,
                ratio,
                first,
                second,
            } => {
                let a = (*first).sanitize(valid, seen, depth + 1);
                let b = (*second).sanitize(valid, seen, depth + 1);
                match (a, b) {
                    (Some(first), Some(second)) => Some(Self::Split {
                        axis,
                        ratio: safe_ratio(ratio),
                        first: Box::new(first),
                        second: Box::new(second),
                    }),
                    (Some(one), None) | (None, Some(one)) => Some(one),
                    (None, None) => None,
                }
            }
        }
    }

    fn without(self, id: &str) -> Option<Self> {
        match self {
            Self::Tabs { mut tabs, active } => {
                tabs.retain(|tab| tab != id);
                if tabs.is_empty() {
                    return None;
                }
                let active = if tabs.contains(&active) {
                    active
                } else {
                    tabs[0].clone()
                };
                Some(Self::Tabs { tabs, active })
            }
            Self::Split {
                axis,
                ratio,
                first,
                second,
            } => match ((*first).without(id), (*second).without(id)) {
                (Some(first), Some(second)) => Some(Self::Split {
                    axis,
                    ratio,
                    first: Box::new(first),
                    second: Box::new(second),
                }),
                (Some(one), None) | (None, Some(one)) => Some(one),
                (None, None) => None,
            },
        }
    }

    fn insert(self, target: &str, moved: String, drop: DockDrop) -> Self {
        match self {
            Self::Tabs { mut tabs, active } => {
                if !tabs.iter().any(|id| id == target) {
                    return Self::Tabs { tabs, active };
                }
                if drop == DockDrop::Tab {
                    tabs.push(moved.clone());
                    return Self::Tabs {
                        tabs,
                        active: moved,
                    };
                }
                let original = Self::Tabs { tabs, active };
                let new_leaf = Self::single(moved);
                let (axis, at_start) = match drop {
                    DockDrop::Left => (SplitAxis::Horizontal, true),
                    DockDrop::Right => (SplitAxis::Horizontal, false),
                    DockDrop::Top => (SplitAxis::Vertical, true),
                    DockDrop::Bottom => (SplitAxis::Vertical, false),
                    DockDrop::Tab => unreachable!(),
                };
                let (first, second, ratio) = if at_start {
                    (new_leaf, original, 0.30)
                } else {
                    (original, new_leaf, 0.70)
                };
                Self::Split {
                    axis,
                    ratio,
                    first: Box::new(first),
                    second: Box::new(second),
                }
            }
            Self::Split {
                axis,
                ratio,
                first,
                second,
            } => {
                if first.contains(target) {
                    Self::Split {
                        axis,
                        ratio,
                        first: Box::new((*first).insert(target, moved, drop)),
                        second,
                    }
                } else {
                    Self::Split {
                        axis,
                        ratio,
                        first,
                        second: Box::new((*second).insert(target, moved, drop)),
                    }
                }
            }
        }
    }

    fn active_tab(&mut self, id: &str) -> bool {
        match self {
            Self::Tabs { tabs, active } if tabs.iter().any(|tab| tab == id) => {
                *active = id.to_owned();
                true
            }
            Self::Split { first, second, .. } => first.active_tab(id) || second.active_tab(id),
            _ => false,
        }
    }

    fn resize(&mut self, path: &[bool], value: f32) -> bool {
        if path.is_empty() {
            if let Self::Split { ratio, .. } = self {
                if value.is_finite() {
                    *ratio = safe_ratio(value);
                    return true;
                }
            }
            return false;
        }
        match self {
            Self::Split { first, second, .. } => {
                if path[0] {
                    second.resize(&path[1..], value)
                } else {
                    first.resize(&path[1..], value)
                }
            }
            Self::Tabs { .. } => false,
        }
    }
}

fn safe_ratio(ratio: f32) -> f32 {
    if ratio.is_finite() {
        ratio.clamp(MIN_RATIO, MAX_RATIO)
    } else {
        0.5
    }
}
fn eligible(panel: &ModularSurfaceState, host: Option<&str>) -> bool {
    if !panel.visible {
        return false;
    }
    match host {
        None => panel.dock != SurfaceDock::Floating,
        Some(host) => panel.dock == SurfaceDock::Floating && panel.floating_host_id() == host,
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModularDockTree {
    pub version: u32,
    pub root: Option<DockNode>,
}

impl Default for ModularDockTree {
    fn default() -> Self {
        Self {
            version: SCHEMA,
            root: None,
        }
    }
}

impl ModularDockTree {
    /// Reconciles saved tree with currently registered visible in-shell panels.
    /// Unknown/duplicated/hidden/native-floating IDs never survive migration.
    /// New registrations are inserted without replacing user splits or ratios.
    pub fn restored(saved: Option<Self>, registered: &[ModularSurfaceState]) -> Self {
        Self::restored_for_location(saved, registered, None)
    }

    /// Restore one native floating host without borrowing or contaminating the
    /// main-window tree. A panel has exactly one valid location in the catalog.
    /// Legacy hosts bootstrap into one tab leaf; saved splits and ratios survive.
    pub fn restored_for_host(
        saved: Option<Self>,
        registered: &[ModularSurfaceState],
        host: &str,
    ) -> Self {
        if host.is_empty() {
            return Self::default();
        }
        Self::restored_for_location(saved, registered, Some(host))
    }

    fn restored_for_location(
        saved: Option<Self>,
        registered: &[ModularSurfaceState],
        host: Option<&str>,
    ) -> Self {
        let valid: BTreeSet<String> = registered
            .iter()
            .filter(|panel| eligible(panel, host) && !panel.id.is_empty())
            .map(|panel| panel.id.clone())
            .collect();
        let mut tree = saved
            .filter(|saved| saved.version == SCHEMA)
            .unwrap_or_default();
        let mut seen = BTreeSet::new();
        tree.root = tree
            .root
            .and_then(|node| node.sanitize(&valid, &mut seen, 0));
        for panel in registered.iter().filter(|panel| valid.contains(&panel.id)) {
            if !seen.insert(panel.id.clone()) {
                continue;
            }
            tree.open(panel, registered);
        }
        tree
    }

    pub fn ordered_tabs(&self) -> Vec<String> {
        let mut tabs = Vec::new();
        if let Some(root) = &self.root {
            root.tabs(&mut tabs);
        }
        tabs
    }

    pub fn has(&self, id: &str) -> bool {
        self.root.as_ref().is_some_and(|root| root.contains(id))
    }

    pub fn activate(&mut self, id: &str) -> bool {
        self.root.as_mut().is_some_and(|root| root.active_tab(id))
    }

    pub fn resize_split(&mut self, path: &[bool], ratio: f32) -> bool {
        self.root
            .as_mut()
            .is_some_and(|root| root.resize(path, ratio))
    }

    /// Only the in-shell tree changes here; the caller handles any OS-native
    /// viewport lifecycle separately. A locked source cannot be moved.
    pub fn move_relative(
        &mut self,
        source: &str,
        target: &str,
        drop: DockDrop,
        catalog: &[ModularSurfaceState],
    ) -> bool {
        self.move_relative_in_location(source, target, drop, catalog, None)
    }

    pub fn move_relative_in_host(
        &mut self,
        source: &str,
        target: &str,
        drop: DockDrop,
        catalog: &[ModularSurfaceState],
        host: &str,
    ) -> bool {
        if host.is_empty() {
            return false;
        }
        self.move_relative_in_location(source, target, drop, catalog, Some(host))
    }

    fn move_relative_in_location(
        &mut self,
        source: &str,
        target: &str,
        drop: DockDrop,
        catalog: &[ModularSurfaceState],
        host: Option<&str>,
    ) -> bool {
        if source == target || !self.has(source) || !self.has(target) {
            return false;
        }
        let Some(moving) = catalog
            .iter()
            .find(|p| p.id == source && !p.locked && eligible(p, host))
        else {
            return false;
        };
        if !catalog.iter().any(|p| p.id == target && eligible(p, host)) {
            return false;
        }
        let moved = moving.id.clone();
        let mut proposal = self.clone();
        let remaining = proposal.root.take().and_then(|node| node.without(source));
        proposal.root = remaining.map(|node| node.insert(target, moved, drop));
        if proposal
            .root
            .as_ref()
            .is_none_or(|node| node.depth() > MAX_DEPTH)
        {
            return false;
        }
        *self = proposal;
        true
    }

    /// Called only after a native host has handed off the panel. Its registered
    /// state should already be changed to a visible, non-floating destination.
    pub fn attach(
        &mut self,
        source: &str,
        target: &str,
        drop: DockDrop,
        catalog: &[ModularSurfaceState],
    ) -> bool {
        self.attach_in_location(source, target, drop, catalog, None)
    }

    pub fn attach_in_host(
        &mut self,
        source: &str,
        target: &str,
        drop: DockDrop,
        catalog: &[ModularSurfaceState],
        host: &str,
    ) -> bool {
        if host.is_empty() {
            return false;
        }
        self.attach_in_location(source, target, drop, catalog, Some(host))
    }

    fn attach_in_location(
        &mut self,
        source: &str,
        target: &str,
        drop: DockDrop,
        catalog: &[ModularSurfaceState],
        host: Option<&str>,
    ) -> bool {
        if source == target || self.has(source) || !self.has(target) {
            return false;
        }
        if !catalog
            .iter()
            .any(|p| p.id == source && !p.locked && eligible(p, host))
        {
            return false;
        }
        if !catalog.iter().any(|p| p.id == target && eligible(p, host)) {
            return false;
        }
        let mut proposal = self.clone();
        if let Some(root) = proposal.root.take() {
            proposal.root = Some(root.insert(target, source.to_owned(), drop));
            if proposal
                .root
                .as_ref()
                .is_some_and(|node| node.depth() > MAX_DEPTH)
            {
                return false;
            }
            *self = proposal;
            true
        } else {
            false
        }
    }

    pub fn detach(&mut self, id: &str) -> bool {
        if !self.has(id) {
            return false;
        }
        self.root = self.root.take().and_then(|node| node.without(id));
        true
    }

    fn open(&mut self, panel: &ModularSurfaceState, registered: &[ModularSurfaceState]) {
        if self.has(&panel.id) {
            return;
        }
        let Some(root) = self.root.take() else {
            self.root = Some(DockNode::single(panel.id.clone()));
            return;
        };
        // New panels first join the corresponding leaf, preserving custom
        // split geometry. The four legacy defaults are used only for new groups.
        let peer = registered.iter().find(|existing| {
            existing.id != panel.id && existing.dock == panel.dock && root.contains(&existing.id)
        });
        if let Some(peer) = peer {
            self.root = Some(root.insert(&peer.id, panel.id.clone(), DockDrop::Tab));
            return;
        }
        let new_leaf = DockNode::single(panel.id.clone());
        self.root = Some(match panel.dock {
            SurfaceDock::Left => DockNode::Split {
                axis: SplitAxis::Horizontal,
                ratio: 0.22,
                first: Box::new(new_leaf),
                second: Box::new(root),
            },
            SurfaceDock::Right => DockNode::Split {
                axis: SplitAxis::Horizontal,
                ratio: 0.78,
                first: Box::new(root),
                second: Box::new(new_leaf),
            },
            SurfaceDock::Bottom => DockNode::Split {
                axis: SplitAxis::Vertical,
                ratio: 0.76,
                first: Box::new(root),
                second: Box::new(new_leaf),
            },
            SurfaceDock::Center | SurfaceDock::Floating => {
                // Center registrations without an existing center leaf get a
                // distinct center leaf, never silently overwrite a tool panel.
                DockNode::Split {
                    axis: SplitAxis::Horizontal,
                    ratio: 0.30,
                    first: Box::new(root),
                    second: Box::new(new_leaf),
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn panel(id: &str, dock: SurfaceDock) -> ModularSurfaceState {
        ModularSurfaceState::new(id, id, dock)
    }
    fn catalog() -> Vec<ModularSurfaceState> {
        vec![
            panel("canvas", SurfaceDock::Center),
            panel("content", SurfaceDock::Left),
            panel("inspector", SurfaceDock::Right),
            panel("activity", SurfaceDock::Bottom),
            panel("assets", SurfaceDock::Right),
        ]
    }

    #[test]
    fn drop_target_uses_leaf_geometry_and_refuses_shell_chrome() {
        let rect = egui::Rect::from_min_size(egui::pos2(10.0, 20.0), egui::vec2(400.0, 300.0));
        assert_eq!(
            drop_zone(rect, egui::pos2(210.0, 170.0)),
            Some(DockDrop::Tab)
        );
        assert_eq!(
            drop_zone(rect, egui::pos2(12.0, 170.0)),
            Some(DockDrop::Left)
        );
        assert_eq!(
            drop_zone(rect, egui::pos2(408.0, 170.0)),
            Some(DockDrop::Right)
        );
        assert_eq!(
            drop_zone(rect, egui::pos2(210.0, 22.0)),
            Some(DockDrop::Top)
        );
        assert_eq!(
            drop_zone(rect, egui::pos2(210.0, 318.0)),
            Some(DockDrop::Bottom)
        );
        assert_eq!(drop_zone(rect, egui::pos2(900.0, 900.0)), None);
        let preview = drop_preview_rect(rect, DockDrop::Right);
        assert_eq!(preview.right(), rect.right());
        assert!(preview.width() < rect.width());
    }

    #[test]
    fn native_window_never_accepts_its_own_tabs_as_a_transfer() {
        let mut source = panel("source", SurfaceDock::Floating);
        source.floating_host = Some("host".into());
        assert!(!can_join_native_host(&source, "host"));
        assert!(can_join_native_host(&source, "other"));
        source.locked = true;
        assert!(!can_join_native_host(&source, "other"));
        source.locked = false;
        source.visible = false;
        assert!(!can_join_native_host(&source, "other"));
    }

    #[test]
    fn legacy_regions_bootstrap_one_tree_without_losing_tabs() {
        let tree = ModularDockTree::restored(None, &catalog());
        assert_eq!(tree.ordered_tabs().len(), 5);
        assert_eq!(
            tree.ordered_tabs()
                .iter()
                .filter(|id| id.as_str() == "assets")
                .count(),
            1
        );
        assert!(tree.root.is_some());
    }

    #[test]
    fn edge_drops_create_nested_splits_and_tabs_are_a_leaf_operation() {
        let panels = catalog();
        let mut tree = ModularDockTree::restored(None, &panels);
        assert!(tree.move_relative("activity", "inspector", DockDrop::Left, &panels));
        assert!(tree.move_relative("content", "activity", DockDrop::Top, &panels));
        assert!(tree.move_relative("assets", "content", DockDrop::Tab, &panels));
        assert_eq!(tree.ordered_tabs().len(), 5);
        assert!(tree.activate("assets"));
        assert!(matches!(tree.root, Some(DockNode::Split { .. })));
    }

    #[test]
    fn moving_last_tab_collapses_empty_leaf_instead_of_leaving_a_ghost() {
        let panels = catalog();
        let mut tree = ModularDockTree::restored(None, &panels);
        assert!(tree.move_relative("content", "canvas", DockDrop::Tab, &panels));
        assert_eq!(tree.ordered_tabs().len(), panels.len());
        assert_eq!(
            tree.ordered_tabs()
                .iter()
                .filter(|tab| tab.as_str() == "content")
                .count(),
            1
        );
    }

    #[test]
    fn stale_duplicate_and_non_finite_saved_data_are_repaired() {
        let panels = catalog();
        let saved = ModularDockTree {
            version: SCHEMA,
            root: Some(DockNode::Split {
                axis: SplitAxis::Vertical,
                ratio: f32::NAN,
                first: Box::new(DockNode::Tabs {
                    tabs: vec!["canvas".into(), "ghost".into(), "canvas".into()],
                    active: "ghost".into(),
                }),
                second: Box::new(DockNode::Tabs {
                    tabs: vec!["canvas".into(), "content".into()],
                    active: "canvas".into(),
                }),
            }),
        };
        let tree = ModularDockTree::restored(Some(saved), &panels);
        let tabs = tree.ordered_tabs();
        assert_eq!(tabs.len(), panels.len());
        assert_eq!(tabs.iter().filter(|id| id.as_str() == "canvas").count(), 1);
        if let Some(DockNode::Split { ratio, .. }) = tree.root {
            assert!(ratio.is_finite());
        }
    }

    #[test]
    fn stale_schema_is_reset_without_discarding_registered_panels() {
        let panels = catalog();
        let tree = ModularDockTree::restored(
            Some(ModularDockTree {
                version: 900,
                root: None,
            }),
            &panels,
        );
        assert_eq!(tree.ordered_tabs().len(), panels.len());
    }

    #[test]
    fn floating_and_hidden_panels_do_not_reappear_in_docked_tree() {
        let mut panels = catalog();
        panels[1].dock = SurfaceDock::Floating;
        panels[2].visible = false;
        let tree = ModularDockTree::restored(None, &panels);
        assert!(!tree.has("content"));
        assert!(!tree.has("inspector"));
        assert_eq!(tree.ordered_tabs().len(), 3);
    }

    #[test]
    fn locked_and_self_moves_are_rejected_without_changing_the_layout() {
        let mut panels = catalog();
        let mut tree = ModularDockTree::restored(None, &panels);
        let original = tree.clone();
        assert!(!tree.move_relative("content", "content", DockDrop::Tab, &panels));
        panels[1].locked = true;
        assert!(!tree.move_relative("content", "canvas", DockDrop::Right, &panels));
        assert_eq!(tree, original);
    }

    #[test]
    fn native_attach_and_detach_preserve_identity_without_duplication() {
        let mut panels = catalog();
        let mut tree = ModularDockTree::restored(None, &panels);
        assert!(tree.detach("content"));
        assert!(!tree.has("content"));
        panels[1].dock = SurfaceDock::Floating;
        assert!(!tree.attach("content", "canvas", DockDrop::Tab, &panels));
        panels[1].dock = SurfaceDock::Center;
        assert!(tree.attach("content", "canvas", DockDrop::Tab, &panels));
        assert!(!tree.attach("content", "canvas", DockDrop::Tab, &panels));
        assert_eq!(tree.ordered_tabs().len(), panels.len());
    }

    #[test]
    fn user_ratios_are_clamped_and_nonfinite_updates_are_rejected() {
        let panels = catalog();
        let mut tree = ModularDockTree::restored(None, &panels);
        assert!(!tree.resize_split(&[], f32::NAN));
        assert!(tree.resize_split(&[], 42.0));
        if let Some(DockNode::Split { ratio, .. }) = tree.root {
            assert_eq!(ratio, MAX_RATIO);
        }
    }
    #[test]
    fn per_host_restore_preserves_splits_and_never_imports_other_windows() {
        let mut catalog = catalog();
        catalog[1].dock = SurfaceDock::Floating;
        catalog[1].floating_host = Some("host.a".into());
        catalog[2].dock = SurfaceDock::Floating;
        catalog[2].floating_host = Some("host.a".into());
        catalog[3].dock = SurfaceDock::Floating;
        catalog[3].floating_host = Some("host.b".into());
        let mut a = ModularDockTree::restored_for_host(None, &catalog, "host.a");
        assert_eq!(a.ordered_tabs().len(), 2);
        assert!(!a.has("activity"));
        assert!(a.move_relative_in_host(
            "content",
            "inspector",
            DockDrop::Right,
            &catalog,
            "host.a"
        ));
        let saved = a.clone();
        let restored = ModularDockTree::restored_for_host(Some(saved.clone()), &catalog, "host.a");
        assert_eq!(restored, saved);
        assert!(matches!(restored.root, Some(DockNode::Split { .. })));
        assert!(!a.attach_in_host("activity", "content", DockDrop::Tab, &catalog, "host.a"));
        assert_eq!(
            ModularDockTree::restored_for_host(None, &catalog, "host.b").ordered_tabs(),
            vec!["activity".to_owned()]
        );
        assert!(ModularDockTree::restored_for_host(None, &catalog, "")
            .root
            .is_none());
    }

    #[test]
    fn floating_handoff_is_atomic_and_old_host_reconciles_away_stale_ids() {
        let mut catalog = catalog();
        for panel in &mut catalog[1..3] {
            panel.dock = SurfaceDock::Floating;
            panel.floating_host = Some("one".into());
        }
        catalog[3].dock = SurfaceDock::Floating;
        catalog[3].floating_host = Some("two".into());
        let old = ModularDockTree::restored_for_host(None, &catalog, "one");
        let mut dest = ModularDockTree::restored_for_host(None, &catalog, "two");
        let mut candidate = catalog.clone();
        candidate[1].floating_host = Some("two".into());
        assert!(dest.attach_in_host("content", "activity", DockDrop::Bottom, &candidate, "two"));
        assert!(!dest.attach_in_host("content", "activity", DockDrop::Tab, &candidate, "two"));
        let source = ModularDockTree::restored_for_host(Some(old), &candidate, "one");
        assert!(!source.has("content"));
        assert!(source.has("inspector"));
        assert_eq!(dest.ordered_tabs().len(), 2);
    }
}
