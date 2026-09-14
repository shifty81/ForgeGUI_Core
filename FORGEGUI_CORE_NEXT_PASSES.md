# ForgeGUI_Core Current Pass Baseline

Target: independent, modular authoring/runtime/UI foundation for all applications and games.

## Completed cumulative foundation

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

## FG-C41…FG-C55 — Creator Studio / universal renderer workspace

- C41 creator density and metric tokens
- C42 dedicated product/menu/action/workspace/panel/tray/status chrome tokens
- C43 reusable panel frame/header primitive
- C44 reusable section/header and panel action model
- C45 creator action/compact tool widgets and status badges
- C46 Creator Studio product/menu/action chrome
- C47 workspace tab strip and creator tab model
- C48 status bar and bottom tool-tray chrome
- C49 durable `forge.workspace_layout.v1` layout schema and presets
- C50 renderer surface family contract for 2D / 2.5D / 3D / Voxel / Hybrid workflows
- C51 frame scheduling and focused/visible/obscured/hidden render-surface lifecycle
- C52 renderer-backed `RenderSurfaceHost` wrapping the universal `RenderBackend` contract
- C53 reusable renderer workspace widget with local tools and authoring overlays
- C54 rebuilt Creator Studio Lab with Inspector-left / Assets-right / renderer-center / tools-bottom workflow
- C55 cumulative strict-Clippy cleanup for the remaining authoring test plus first-pass Creator Studio integration

## FG-C56 onward

Next focus:

1. GPU-backed surface presenter/texture bridge for wgpu/OpenGL/custom game renderers
2. real `egui_dock` structural docking for Inspector/Assets/Outliner/tool-tray regions
3. Asset Browser Quick Access / Favorites / Recent / tags / saved searches / grid thumbnails
4. Inspector sections, history, pinned instances, mixed-value multi-selection
5. universal SelectionService linking renderer, Outliner, Inspector and Asset Browser
6. Outliner/Layers hierarchy and drag-reparent
7. precise 2D world/screen math and 3D ray construction/picking
8. transform gizmo manipulation transactions
9. workspace layout persistence/import/export and monitor/DPI-safe restore
10. multi-render-surface and detached native workspaces
11. actual PIE/runtime renderer switching through the same surface host
12. interaction/screenshot/DPI/accessibility certification for Creator Studio
