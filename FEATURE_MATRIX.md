# Feature Matrix

| Area | 0.4.8 status | Notes |
|---|---|---|
| Renderer-neutral panel/core contracts | Ready | Namespaced IDs, roles, scope, instance policies, capabilities |
| Structural left/right/bottom rails | Ready | Runtime-owned rail tab groups |
| Center docking workspace | Ready | `egui_dock` adapter remains replaceable |
| PreferredDock authority | Ready | Floating currently falls back to center |
| Semantic icons | Ready | `IconId` -> Phosphor adapter |
| Virtual data table | Ready | `egui_extras` virtualized rows |
| 2D canvas contracts | Ready | finite/infinite, pan/zoom/grid/snap/minimap proof |
| Console / Cortex data model | Ready | output/cortex/terminal/activity channel kinds; no terminal process host yet |
| Terminal process host | Planned | capability is intentionally false in `forge.gui.toml` |
| Notifications | Ready | durable notification center model |
| PIE contracts | Ready | play/from-here/pause/step/restart/stop reference proof |
| Project/Asset/Inspector/Layers panels | Reference proof | generic templates continue after recovery GREEN |
| Timeline / Curve / Graph surfaces | Reference proof | shared contracts/templates still to deepen |
| InterfacePreset / layout schema | Foundation | validation exists; runtime preset application is not yet authoritative |
| OSS tree/layout candidates | Evaluation | compile/smoke certified, not public durable contracts |
| Interaction testing | Foundation | `egui_kittest` smoke lane established |
| 3D RenderSurfaceHost | Planned | required for Subspace/3D editors |
| Schema-driven data editors | Planned | intended for Windstead/Ember/Subspace data tooling |
| Accessibility/DPI/localization hardening | Planned | later 1.0 hardening lane |
| PCC native runner | Ready | PowerShell 5.1 stderr-safe, exit-code authoritative |
| Automatic handoff ZIP | Ready | failure/manual + PASS checkpoint bundles |
| Transactional patch intake | Ready | one per session, backup/receipt/rollback |
| GREEN commit/push authority | Ready | exact certified source fingerprint required |
