# ForgeGUI_Core Implementation Roadmap

## Completed foundation through FG-C40

- independent reusable-library boundary
- mature ForgeGUI 0.4.8 recovery
- semantic theme/elevation system
- four rail presets
- workbench/browser/Inspector/command/document-map foundations
- shared ForgeScene model
- renderer-neutral 2D/3D/Hybrid contracts
- universal authoring-surface state
- selection/grid/snap/gizmo/undo foundations
- GUI-independent runtime clock/system host
- Simulate/PIE/Play From Here/Detached/Standalone/Headless runtime bridge
- ForgeGUI Lab, ForgeGUI Next Lab, and ForgeAuthoring Lab certification lanes

## FG-C41–C50 — Workbench + persistence

Integrate `egui_dock` as the flexible document/tool workspace while keeping permanent
structural regions outside the dock tree. Add dock compass/drop previews, panel
pin/soft-lock/detach/maximize/auto-hide behavior, serialized workspace layouts,
monitor/DPI-safe recovery, and user-saveable named presets.

## FG-C51–C60 — Authoring interaction

Implement precise 2D and 3D camera transforms, ray/screen/world conversions, hierarchy
reparenting, robust picking, selection transactions, transform-gizmo transactions,
pivots, coordinate spaces, rulers/guides, snapping, measurement overlays, and viewport
bookmarks.

## FG-C61–C70 — Render-surface bridge

Add a concrete GPU render-surface hosting contract while retaining the null/headless
backend. Support consumer-provided 2D/3D renderers, render-target resize/DPI lifecycle,
editor overlay composition, picking buffers/hooks, and debug visualization.

## FG-C71–C80 — Asset/Inspector maturity

Expand the Asset Browser to tree/list/grid views, thumbnails, saved searches,
favorites/recent, drag/drop, tags, source-control overlays, and async loading. Expand
Inspector metadata, mixed values, vectors/colors/gradients, asset references,
reset/revert, validation, provenance, and undo integration.

## FG-C81–C90 — Timeline / graph / runtime tooling

Normalize timeline/dope-sheet/curve-editor controls, node graph interaction,
runtime debugging, console/problem/build surfaces, operation queue/status,
and reusable Cortex/automation hosting surfaces.

## FG-C91–C100 — SDK + certification

Normalize feature flags and public facades, add consumer integration examples,
headless-runtime certification, visual/DPI snapshot fixtures, keyboard-only testing,
high-contrast/reduced-motion certification, layout persistence/recovery tests,
and semver/API compatibility reporting.
