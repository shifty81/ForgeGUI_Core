# ForgeGUI_Core Current Pass Baseline

Target: a project-owned, canvas-first GUI core that can seed polished desktop tools,
editors and game-development applications without forcing every consumer into the same
panel layout.

## Certified architecture through FG-C75

- independent reusable ForgeGUI boundary
- semantic theme/elevation/token system
- rails, panels, browser, Inspector, command palette, console/notification foundations
- shared scene, renderer-neutral 2D/3D/Hybrid and universal authoring contracts
- selection/grid/snap/gizmo/undo foundations
- GUI-independent runtime and PIE/simulation bridge
- renderer-surface host and structural layout contracts
- project-owned borderless shell with draggable title region
- project-owned Minimize / Maximize-Restore / Close with draggable and double-click-title behavior
- manual project-owned resize hit regions
- one canonical runnable `forge_gui_lab`

## FG-C76–C82 — Canvas-first shell and reusable chrome

Current implementation block:

1. Permanent main-app chrome is reduced to title, File/Edit/View/Help and status.
2. The center infinite canvas receives layout priority.
3. The left universal rail groups Transform, Workspace and Runtime actions.
4. Left rail and right context host can be pinned or revealed from the workspace edge.
5. The right side is one contextual Inspector/Assets/Widgets surface instead of several
   always-visible panels.
6. Document tabs and bottom tool tray are opt-in View chrome.
7. Canvas-owned chrome includes inset frame, top/left rulers, coordinate ticks,
   slim context-aware rulers, a compact vertical layer stack and lightweight camera/selection HUD.
8. Forge Dark, Midnight Mint, Graphite, Warm Ember and High Contrast are built-in presets.
9. Native scroll areas inherit ForgeGUI scroll tokens.
10. Shared widget core now includes project-owned progress bars, radial gauges,
    custom scrollbars and segmented-choice controls.
11. The canonical Lab acts as the visual certification host; reusable behavior lives in
    shared crates.

## FG-C83–C90 — Universal control completion

Build out the remaining general-purpose chrome behind ForgeGUI adapters:

- switches, radio/checkbox families, toggle and segmented controls
- text/search/password/multiline input families
- numeric steppers, scrubbers, ranges, vectors, angles and unit-aware editors
- combo/searchable pickers and path/file/folder/asset/object references
- color, palette, gradient and curve controls
- virtualized tree, list, table, tree-table and thumbnail-grid controls
- splitters, resizable panes, dock compass/drop guides and focus/fill affordances
- context menus, rich tooltips, popovers, drawers, dialogs, sheets, toasts and banners
- status pills, operation/progress states, skeleton/loading/empty/error states
- minimap/navigator/viewport overview widgets
- normalized hover/pressed/selected/disabled/focus behavior
- keyboard navigation, focus rings, shortcut discovery and accessibility hooks

## FG-C91–C100 — Authoring chrome + consumer SDK certification

- graph/node canvas, ports/wires, graph minimap and subgraph navigation
- timeline/dope-sheet/curve-editor chrome and transport controls
- transform/selection/gizmo overlays
- asset-browser/Inspector maturity and multi-selection workflows
- reusable console/problems/build/operation-queue/source-control surfaces
- density/typography presets and consumer theme derivation
- high-DPI fixtures across all built-in themes
- keyboard-only, high-contrast and reduced-motion certification
- layout persistence/import/export and monitor-safe recovery
- consumer starter-shell templates and integration examples
- public facade/feature-flag normalization and semver compatibility reporting

## Parallel authoring/runtime integration tracks

The visual-core work does not replace these existing engineering tracks:

- GPU-backed render-surface presentation for consumer renderers
- real structural docking where a product requires it
- universal SelectionService across canvas/outliner/Inspector/browser
- hierarchy/outliner drag-reparent and canvas drag/drop placement
- precise 2D world/screen math and 3D ray/picking
- transform-gizmo manipulation transactions
- multi-surface/detached workspace support
- actual PIE/runtime renderer switching through the shared surface host

## Standing rule

Do not solve a workflow by permanently adding another bar or panel. Prefer contextual
chrome, grouped controls, menus, overlays, edge reveal, collapsible hosts and workspace-
specific opt-ins. Under layout pressure, preserve the center authoring/canvas surface first.
