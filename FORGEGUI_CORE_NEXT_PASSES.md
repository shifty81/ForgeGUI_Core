# ForgeGUI_Core Current Pass Baseline

Target: a universal modular desktop GUI core that can seed utilities, dashboards, launchers,
tooling applications and advanced 2D/3D authoring environments without forcing consumers
into a canvas/editor-shaped shell.

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

## M1–M6 modular-shell baseline

The generic application shell is now the primary visual baseline: independent surfaces may
dock left/center/right/bottom or float; toolbars may dock/floating/hide/lock; shell profiles
control global chrome; Settings owns theme/density/interaction policy; Canvas / Viewport is
an optional authoring workspace rather than the framework identity.

## Standing rule

Do not solve a workflow by permanently adding another bar or panel. Prefer configurable
surfaces, tab stacks, contextual chrome, movable toolbars, menus, overlays, floating hosts
and shell-profile opt-ins. Preserve the host application's chosen primary content first; a
canvas receives priority only when the active consumer profile explicitly requests it.

## M1-M6+ continuity

- PCC main-menu return after every completed action.
- Source-authoritative Lab launch and visible generation marker.
- Drag grips and dock previews for modular surfaces/toolbars.
- Next: persistence of user-created surface/tab layouts, multi-window detached workspace persistence, and keyboard docking commands.

## M8 — Public modular application runtime

- `forge_gui_shell` is the high-level drop-in application host for Rust/egui consumers.
- Applications register independent surfaces and provide only application-owned content.
- ForgeGUI owns shared chrome, dock groups, tab stacks, floating hosts, locks, toolbar placement,
  surface drag/drop and persisted shell layouts.
- `forge_gui_consumer_starter` is the independent acceptance executable. It must stay free of
  copied Lab shell logic.
- ForgeGUI Lab remains a visual catalog/showcase only. New framework capability is not considered
  universal until the independent consumer starter can use it through public APIs.

Next: migrate the Lab's remaining private shell rendering helpers onto `forge_gui_shell`, then add
starter packaging/onboarding so a repository can copy or depend on the framework with one bounded
setup step.


### FG-C71R rebased correction / gate pending
Preserve the newer native-window Lab; port the shared `ForgeGuiRuntime` to one egui_dock tree, improve semantic icon coverage and persist current Lab modular-surface positions. Follow-on: bridge the Lab modular surface authority to the shared dock tree (rather than replacing its existing native viewport support), then certify full Windows drag/snap/DPI and compile/test results.

FG-C71R does not modify Cargo/PCC: the current project already builds the single canonical `forge_gui_lab` through its PCC.

## FG-C73/C74 cumulative continuation — source candidate, Windows gate pending

- C73 native frameless floating hosts/tab groups are retained without replaying C71/C72.
- C74 switches the Lab's dock-drop hit testing to measured panel rectangles, adds live native-host hover indication, and saves selected floating tabs and stable tab order under additive storage keys.
- **Still required:** one canonical nested dock-tree owner for the Lab and shared application shell, native-viewport drag bridge with event ownership, split/tab serialization and migration, multi-monitor validation, actual renderer tests and Windows Full Gate. Region tab groups do not count as arbitrary nested splits.
- Never enqueue a Cortex-consumer patch targeting missing `apps/forge_control_center` files inside ForgeGUI; archive it safely outside root intake until the proper consumer repository is available.

## FG-C75 cumulative progress / C76 integration next

- Source implements a renderer-neutral, versioned recursive tab/split dock tree with atomic transactions, split ratio limits, legacy surface reconciliation and unit coverage. It is exported from `forge_gui_chrome::docking` but is not yet the Lab's live view/model authority.
- Lab native floating transfers now reject a source rejoining its own host, refuse locked/invisible transfer candidates, and clear drag state after a real source-viewport release rather than waiting for pointer-hover to vanish.
- **C76 required:** remove Lab's separate fixed left/right/bottom/center grouping and draw all docked leaves from one tree, then store split ratios and tab activity. Keep native-host window lifecycle separate from logical dock layout but commit transitions atomically. Do not call split docking complete until visual and Windows interaction tests pass.
- **C77 onward:** cross-native-window drop capture, monitor reattachment, DPI and no jitter/regression, actual GPU surface and keyboard-accessible dock actions. PCC Full Gate is still required for C75.

## After FG-C76 (cumulative candidate)

- C76: GUI Lab has an actual recursive tree renderer, nested split resizing,
  leaf-local edge/center drops, and persisted tree schema. **Windows gate pending.**
- C77: Cross-native viewport drag bridge; versioned floating-host subtrees,
  native window group merge/split, monitor-safe pointer coordinate handling.
- C78: Windows Full Gate, drag/resize/restart tests at mixed-DPI and with
  multiple monitors; fix any compile or interaction failures from C76/C77.
- C79: Consolidate shared runtime and consumer shell docking adapters on the
  canonical model without breaking independent project-owned services.
- C80: Polish, profiling, accessibility, screenshot regression and release
  qualification. No application/library-wide completion claim before all gates.

## After FG-C77 (cumulative candidate)

- C77: native floating hosts have their own persisted dock trees, live recursive tab/split rendering and per-leaf drop proposals, with source tests for migration/ownership. **Not a certified Windows native drag bridge**: crossing independent OS windows, monitor scaling, and edge-resize must be checked on a real machine.
- C78: migrate `forge_gui_egui` consumer shell from its independent `egui_dock` state to the canonical `ModularDockTree`, remove dual layout authority, expose stable host registration contracts.
- C79: Windows Full Gate and GUI interaction certification, real multi-viewport pointer telemetry, stale drag prevention and mixed-DPI bounds.
- Later: GPU canvas presentation, widget behavior audit, accessibility, performance and distributable SDK.

## After FG-C78 — cumulative candidate, local Full Gate not yet run

- C78 code migrates the independent `forge_gui_egui` model to the canonical `ModularDockTree`; it still needs an actual Rust/Windows build and an independent consumer acceptance build. The public adapter API intentionally changed from `dock_state` to `dock_tree`; audit all external users before release. The consumer adapter's floating windows are in-app egui hosts, while GUI Lab retains native OS viewports.
- C79: run Windows PCC Full Gate against the exact cumulative source, repair compile/API failures, test frame-by-frame drag ownership, redock, mixed DPI and resize stability. Prune the unused `egui_dock` dependency with Cargo-managed lock regeneration once source compiles.
- C80: refine shared chrome and consumer integration (including native viewport host contract), deterministic interaction/screenshot tests, accessibility and performance. GPU canvas and full widget SDK certification remain later milestones.
- All passes remain cumulative until the user confirms they are home. Do not imply local builds passed from patch-application tests alone.
