# Feature Matrix

| Area | 0.4.8 status | Notes |
|---|---|---|
| Renderer-neutral panel/core contracts | Ready | Namespaced IDs, roles, scope, instance policies, capabilities |
| Structural left/right/bottom rails | Ready | Runtime-owned rail tab groups |
| Creator Studio chrome | Foundation ready | Product/menu/action/workspace/status/tray chrome now has shared tokens and widgets |
| Structural panel widgets | Foundation ready | Shared panel frame/header/action primitives added |
| Workspace layout schema | Foundation ready | `forge.workspace_layout.v1` plus Creator/World/Animation/Scripting presets |
| Renderer workspace host | Foundation ready | `RenderSurfaceHost` owns renderer backend lifecycle, extent and scheduling |
| Universal renderer families | Foundation ready | 2D / 2.5D / 3D / Voxel / Hybrid authoring profiles |
| GPU surface presentation | Next | Renderer output still needs backend-specific texture/swapchain presentation bridge |
| Center structural docking | Next | `egui_dock` is reserved for structural tool regions, not the renderer itself |
| Semantic icons | Ready | `IconId` -> Phosphor adapter |
| Creator action widgets | Ready | Compact/labeled tool controls and semantic tone badges |
| Virtual data table | Ready | `egui_extras` virtualized rows |
| 2D canvas contracts | Ready | finite/infinite, pan/zoom/grid/snap/minimap proof |
| Console / Cortex data model | Ready | output/cortex/terminal/activity channel kinds; no terminal process host yet |
| Terminal process host | Planned | capability is intentionally false in `forge.gui.toml` |
| Notifications | Ready | durable notification center model |
| PIE contracts | Ready | play/from-here/pause/step/restart/stop reference proof |
| Project/Asset/Inspector/Layers panels | Reference proof | now presented in Creator Studio shell; provider depth continues next |
| Timeline / Curve / Graph surfaces | Reference proof | shared contracts/templates still to deepen |
| OSS tree/layout candidates | Evaluation | compile/smoke certified, not public durable contracts |
| Interaction testing | Foundation | `egui_kittest` smoke lane established |
| Schema-driven data editors | Planned | intended for game/project data tooling |
| Accessibility/DPI/localization hardening | Planned | Creator Studio certification lane |
| PCC native runner | Ready | PowerShell 5.1 stderr-safe, exit-code authoritative |
| Automatic handoff ZIP | Ready | failure/manual + PASS checkpoint bundles |
| Transactional patch intake | Ready | one per session, backup/receipt/rollback |
| GREEN commit/push authority | Ready | exact certified source fingerprint required |
