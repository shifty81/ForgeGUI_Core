# Third-party notices

ForgeGUI keeps third-party implementation crates behind ForgeGUI-owned adapters. Durable project formats must not serialize their private types. Exact resolved transitive dependencies are recorded by `Cargo.lock` after the project-owned PCC generates it.

| Component | Pinned line | Role | License |
|---|---:|---|---|
| egui | 0.36.2 | immediate-mode GUI | MIT OR Apache-2.0 |
| eframe | 0.36.2 | native reference host | MIT OR Apache-2.0 |
| egui_extras | 0.36.2 | virtual table/widget adapter | MIT OR Apache-2.0 |
| egui_kittest | 0.36.1 | UI interaction test lane | MIT OR Apache-2.0 |
| egui_dock | 0.21.1 | current center-workspace docking adapter | MIT |
| egui-phosphor | 0.14.0 | semantic icon renderer adapter | MIT OR Apache-2.0; Phosphor Icons MIT |
| egui_ltreeview | 0.9.0 | tree-view evaluation lane | MIT |
| egui_tiles | 0.17.1 | Studio layout-engine bakeoff | MIT OR Apache-2.0 |
| serde | 1.x | serialization | MIT OR Apache-2.0 |

Upstream copyright and license texts remain authoritative. `DEPENDENCY_PROVENANCE.json` records ForgeGUI's direct dependency roles and policy.
