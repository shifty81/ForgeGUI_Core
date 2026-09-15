# ForgeGUI_Core Implementation Roadmap

This roadmap tracks the canonical ForgeGUI Core line. The GUI Lab is the visual and
interaction certification host; reusable behavior belongs in shared crates rather than
being implemented only in the Lab.

## Completed foundation — FG-C01–C40

- independent reusable-library boundary
- mature ForgeGUI recovery baseline
- semantic theme/elevation system
- rail/workbench/browser/Inspector/command/document-map foundations
- shared ForgeScene model
- renderer-neutral 2D/3D/Hybrid contracts
- universal authoring-surface state
- selection/grid/snap/gizmo/undo foundations
- GUI-independent runtime clock/system host
- Simulate/PIE/Play From Here/Detached/Standalone/Headless runtime bridge

## FG-C41–C50 — Workbench + persistence

- flexible dock/document workspace
- permanent structural regions kept outside document docking
- dock compass/drop previews
- panel pin, soft-lock, detach, focus-fill and auto-hide behavior
- serialized workspace layouts and named presets
- monitor/DPI-safe layout recovery

## FG-C51–C60 — Authoring interaction

- precise 2D/3D camera transforms
- screen/world/ray conversion contracts
- hierarchy reparenting and robust picking
- selection and transform transactions
- pivot and coordinate-space behavior
- snap/measurement overlays and viewport bookmarks

## FG-C61–C70 — Render-surface bridge

- concrete consumer render-surface hosting contract
- retained null/headless backend
- consumer-provided 2D/3D renderers
- render-target resize/DPI lifecycle
- editor overlay composition
- picking/debug visualization hooks

## FG-C71–C75 — Project-owned shell + one canonical Lab

- remove native Windows title chrome from the visible application surface
- ForgeGUI-owned draggable title region
- project-owned Minimize + Close only; no Maximize button
- project-owned resize hit regions for manual sizing
- one canonical runnable `forge_gui_lab`
- duplicate visual labs retired from the active workspace
- reference-art direction locked as the visual target

## FG-C76–C82 — Canvas-first shell + widget core (current)

- File / Edit / View / Help as the permanent application menu vocabulary
- central infinite canvas as the primary workspace surface
- internal canvas frame and ruler chrome
- text-only canvas layer toggles rather than a permanent layer panel
- compact canvas HUD/telemetry overlays
- grouped universal left tool rail
- pinned, collapsed and hover-edge reveal behavior
- contextual/collapsible right-side surface
- optional document tabs and bottom tray instead of permanent canvas consumption
- project-owned progress, gauge and scrollbar primitives
- themed native scrolling behavior
- built-in Forge Dark, Midnight Mint, Graphite, Warm Ember and High Contrast presets
- canonical Lab as the visual certification host for all of the above

## FG-C83–C90 — Universal control completion

Complete the reusable control vocabulary without adding permanent shell clutter:

- toggle switches, checkbox/radio families and segmented selectors
- numeric, vector, range, angle and unit-aware editors
- color, gradient, curve and palette controls
- file/asset/object reference pickers
- searchable combo boxes, command pickers and breadcrumb controls
- virtualized tree, table, list and grid primitives
- splitters, resizable panes and dock-guide overlays
- context menus, popovers, tooltips, drawers and modal/non-modal dialogs
- toast/notification/status-message surfaces
- minimap, navigator and viewport overview widgets
- complete hover/pressed/selected/disabled/focus state normalization
- keyboard navigation and focus-ring behavior

## FG-C91–C100 — Authoring chrome + SDK certification

Generalize specialist authoring chrome while keeping it opt-in per hosted tool:

- graph/node-editor chrome and interaction helpers
- timeline, dope-sheet and curve-editor chrome
- transform/selection overlays and authoring gizmo chrome
- reusable console/problems/build/operation-queue surfaces
- asset-browser and Inspector maturity passes
- density and typography presets
- high-DPI visual fixtures across all built-in themes
- keyboard-only, high-contrast and reduced-motion certification
- layout persistence/recovery tests
- consumer starter-shell templates and integration examples
- public API/feature-flag normalization and semver compatibility reporting

## Standing architecture rule

A hosted project or authoring panel may opt into whichever ForgeGUI systems it needs, but
must not inherit the entire application shell by default. The application owns global
navigation and chrome; the active authoring surface owns only its contextual tools. The
center work/canvas area remains the priority allocation whenever layout pressure occurs.
