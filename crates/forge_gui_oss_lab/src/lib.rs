//! Compile-certified evaluation lane for candidate ForgeGUI OSS adapters.
#![forbid(unsafe_code)]

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Candidate {
    pub crate_name: &'static str,
    pub pinned_version: &'static str,
    pub disposition: &'static str,
}

pub const CANDIDATES: &[Candidate] = &[
    Candidate {
        crate_name: "egui_extras",
        pinned_version: "0.36.2",
        disposition: "adopt-behind-forgegui-widgets",
    },
    Candidate {
        crate_name: "egui_ltreeview",
        pinned_version: "0.9.0",
        disposition: "bakeoff-tree-adapter",
    },
    Candidate {
        crate_name: "egui_tiles",
        pinned_version: "0.17.1",
        disposition: "bakeoff-layout-engine",
    },
    Candidate {
        crate_name: "egui_kittest",
        pinned_version: "0.36.1",
        disposition: "adopt-testing-only",
    },
    Candidate {
        crate_name: "egui-phosphor",
        pinned_version: "0.14.0",
        disposition: "adopt-behind-semantic-icons",
    },
];

pub fn candidate(name: &str) -> Option<&'static Candidate> {
    CANDIDATES
        .iter()
        .find(|candidate| candidate.crate_name == name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Id;
    use egui_kittest::Harness;
    use egui_ltreeview::TreeView;
    use egui_tiles::{Tiles, Tree};

    #[test]
    fn candidate_versions_are_pinned_and_unique() {
        let mut names = std::collections::BTreeSet::new();
        for candidate in CANDIDATES {
            assert!(names.insert(candidate.crate_name));
            assert!(!candidate.pinned_version.is_empty());
            assert!(!candidate.disposition.is_empty());
        }
    }

    #[test]
    fn tree_view_and_kittest_smoke() {
        let mut harness = Harness::new_ui(|ui| {
            TreeView::new(Id::new("forgegui-oss-tree-smoke")).show(ui, |builder| {
                builder.dir(0, "Root");
                builder.leaf(1, "Child");
                builder.close_dir();
            });
        });
        harness.run();
    }

    #[test]
    fn virtual_table_and_kittest_smoke() {
        let mut harness = Harness::new_ui(|ui| {
            forge_gui_widgets::virtual_table(
                ui,
                Id::new("forgegui-oss-table-smoke"),
                &["Name", "State"],
                10_000,
                20.0,
                |ui, row, column| {
                    if column == 0 {
                        ui.label(format!("Row {row}"));
                    } else {
                        ui.label("Ready");
                    }
                },
            );
        });
        harness.run();
    }

    #[test]
    fn tiles_layout_smoke() {
        let mut tiles = Tiles::default();
        let first = tiles.insert_pane("Canvas");
        let second = tiles.insert_pane("Graph");
        let root = tiles.insert_tab_tile(vec![first, second]);
        let tree = Tree::new("forgegui-oss-tiles-smoke", root, tiles);
        assert_eq!(tree.tiles.len(), 3);
        assert_eq!(tree.root, Some(root));
    }
}
