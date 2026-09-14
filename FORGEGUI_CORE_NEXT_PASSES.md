# ForgeGUI_Core Current Pass Baseline

Target: independent, modular authoring/runtime/UI foundation for all applications.

## Completed in the current cumulative batch

### FG-C01…FG-C20 — shared GUI/workbench foundation

- independent ForgeGUI_Core library boundary
- semantic theme/elevation system
- Forge Dark / Graphite / High Contrast presets
- Slim Icon / Labeled Tool / Dual Context / Floating Pill rails
- rail placement, density, collapse, badge and lock policy
- workbench/focus/bottom-tray state
- virtualized Asset Browser foundation
- shared Inspector/property-grid renderer
- command palette foundation
- document-map marker lane
- next-generation GUI Lab
- PCC certification expansion

### FG-C21…FG-C40 — universal authoring + runtime foundation

- shared Forge scene schema and stable entity IDs
- 2D and 3D transforms
- hierarchy / roots / layers / tags
- editor-only metadata and runtime scene cloning
- scene validation
- renderer-neutral 2D / 3D / Hybrid render contracts
- 2D camera
- perspective/orthographic 3D camera
- Hybrid authoring camera
- picking contracts
- Null/headless-safe renderer adapter
- authoring surface mode/state
- selection model
- gizmo/coordinate-space model
- grid/snap settings
- undo/redo command stack
- fixed-step runtime clock
- headless/rendered/editor-preview runtime modes
- runtime system lifecycle
- input snapshots
- Simulate / PIE / Play From Here / Detached / Standalone / Headless bridge
- independent ForgeAuthoring Lab

## Critical recovery included

This cumulative patch also repairs the accidental Phase-1 migration overwrite:

- restores the mature ForgeGUI 0.4.8 root workspace identity
- restores mature `forge_gui_core`
- restores mature `forge_gui_widgets`
- removes the duplicate Phase-1 `apps/gui_lab`
- removes obsolete Phase-1 rail/surface source files from `forge_gui_widgets`
- keeps the new work additive in dedicated shared crates

## FG-C41 onward

Next focus:

1. real `egui_dock` central document workbench integration
2. dock compass / drop previews / pin / soft-lock / detach / maximize / auto-hide
3. persisted named layouts and monitor/DPI-safe recovery
4. 2D world-space/screen-space camera math and precise picking
5. 3D matrix math / ray construction / precise picking
6. transform gizmo manipulation transactions
7. hierarchy/outliner drag-reparent
8. reusable renderer bridge for GPU-backed 2D/3D targets
9. runtime asset service + hot reload contracts
10. editor runtime overlays / debug draw
11. Play From Here spawn/camera override semantics
12. screenshot/DPI/keyboard certification for all authoring modes
