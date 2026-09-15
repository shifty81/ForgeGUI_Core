# Feature Matrix

| Area | 0.4.8 status | Notes |
|---|---|---|
| Renderer-neutral panel/core contracts | Ready | Namespaced IDs, roles, scope, instance policies, capabilities |
| Universal grouped tool rail | Ready | Slim left rail with grouped transform/workspace/runtime tools plus pin/edge-reveal shell behavior |
| Project-owned application chrome | Ready | Borderless window, owned draggable title surface, Minimize/Maximize-Restore/Close, double-click maximize/restore, distinct File/Edit/View/Help menu strip, manual resize handles |
| Structural panel widgets | Foundation ready | Shared panel frame/header/action primitives added |
| Workspace layout schema | Foundation ready | `forge.workspace_layout.v1` plus Creator/World/Animation/Scripting presets |
| Canvas-first renderer workspace | Ready foundation | Center-priority surface with thin internal frame, slim 2D/2.5D/Hybrid rulers, compact vertical layer stack and lightweight HUD |
| Universal renderer families | Foundation ready | 2D / 2.5D / 3D / Voxel / Hybrid authoring profiles |
| GPU surface presentation | Next | Renderer output still needs backend-specific texture/swapchain presentation bridge |
| Center structural docking | Next | `egui_dock` is reserved for structural tool regions, not the renderer itself |
| Semantic icons | Ready | `IconId` -> Phosphor adapter |
| Core chrome widgets | Ready foundation | Compact/labeled controls, badges, progress, radial gauge, themed native scroll areas, owned scrollbar, segmented choice |
| Virtual data table | Ready | `egui_extras` virtualized rows |
| 2D canvas contracts | Ready | finite/infinite, pan/zoom/grid/snap/minimap proof |
| Console / Cortex data model | Ready | output/cortex/terminal/activity channel kinds; no terminal process host yet |
| Terminal process host | Planned | capability is intentionally false in `forge.gui.toml` |
| Notifications | Ready | durable notification center model |
| PIE contracts | Ready | play/from-here/pause/step/restart/stop reference proof |
| Context Inspector/Assets/Widgets host | Ready foundation | One collapsible/pinnable context surface replaces permanently competing side panels |
| Timeline / Curve / Graph surfaces | Reference proof | shared contracts/templates still to deepen |
| OSS tree/layout candidates | Evaluation | compile/smoke certified, not public durable contracts |
| Interaction testing | Foundation | `egui_kittest` smoke lane established |
| Schema-driven data editors | Planned | intended for game/project data tooling |
| Accessibility/DPI/localization hardening | Planned | Creator Studio certification lane |
| PCC native runner | Ready | PowerShell 5.1 stderr-safe, exit-code authoritative |
| Automatic handoff ZIP | Ready | failure/manual + PASS checkpoint bundles |
| Transactional patch intake | Ready | one per session, backup/receipt/rollback |
| GREEN commit/push authority | Ready | exact certified source fingerprint required |


## FG-C56..FG-C70 — Canonical Creator Studio / chrome normalization

- C56 canonical `forge_gui_lab` promoted to the Creator Studio shell.
- C57 ForgeGUI facade now exposes browser / inspector / command creator services.
- C58 compact Creator density and chrome metrics.
- C59 flush structural panel frame and header treatment.
- C60 reusable panel tab strips.
- C61 Asset Browser search / favorites / tree-row visual pass.
- C62 Inspector object header and compact property-grid pass.
- C63 bottom tool tray tab normalization and reduced default tray height.
- C64 menu / product / action chrome normalization.
- C65 compact Creator widget family and semantic tabs.
- C66 workspace document header and local toolbar normalization.
- C67 renderer-family selector moved into workspace-local chrome.
- C68 renderer surface remains the authoritative center view; null presenter remains Lab-only.
- C69 Creator Studio reference preview and selection chrome tightened.
- C70 canonical Lab parity: PCC `run.lab` now reaches the Creator Studio because `forge_gui_lab` itself is the promoted target.


## FG-C71..FG-C75 — Project-owned shell normalization

- C71 native OS title-bar decorations removed from the canonical GUI Lab.
- C72 ForgeGUI-owned draggable title surface established; current shell normalizes it to Minimize / Maximize-Restore / Close with double-click maximize/restore.
- C73 project-owned edge/corner resize handles preserve manual desktop sizing without native chrome.
- C74 duplicate `forge_gui_next_lab` and `forge_authoring_lab` executables retired; `forge_gui_lab` is the one visible Lab.
- C75 reference concept images remain the visual-direction authority; dock/workbench focus-fill remains available as a workspace behavior while the desktop shell now also supports native-style maximize/restore.

## FG-C76..FG-C82 — Canvas-first GUI core normalization

- C76 canonical Lab reduced to project-owned title + File/Edit/View/Help + status as permanent app bars.
- C77 universal left tool rail grouped by Transform / Workspace / Runtime.
- C78 left rail and right context surface support pinned or hover-edge reveal behavior.
- C79 renderer workspace owns the internal frame, rulers and lightweight HUD; current normalization reduces the frame/ruler footprint and hides rulers where 3D/Voxel profiles do not need them.
- C80 layer visibility remains built into the workspace rather than a permanent panel; current normalization presents it as a compact vertical layer stack.
- C81 built-in Forge Dark / Midnight Mint / Graphite / Warm Ember / High Contrast presets plus themed ScrollArea chrome.
- C82 reusable progress bar, radial gauge, owned scrollbar and segmented-choice primitives plus canonical widget gallery.
