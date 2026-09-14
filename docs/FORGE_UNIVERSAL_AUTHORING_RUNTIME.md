# Universal Authoring + Runtime Core

ForgeGUI_Core is independently maintained, but the repository now deliberately
hosts three separable library families:

```text
ForgeGUI_Core repository
├─ ForgeGUI       reusable GUI/workbench framework
├─ ForgeAuthoring universal 2D/3D/hybrid authoring framework
└─ ForgeRuntime   headless/rendered runtime framework
```

Applications can depend on only the families they need.

## Non-negotiable dependency rule

The runtime is GUI-independent.

`forge_runtime_core` and `forge_scene_core` must never depend on `egui`,
ForgeGUI widgets, Ember, Cortex, Havenwild, Subspace, or another application.

The editor may host the runtime. The runtime may not require the editor.

## Shared scene contract

`forge_scene_core::ForgeScene` is the common scene representation used by:

- authoring sessions
- validation
- PIE/simulate
- headless tests
- rendered runtime
- package/cook pipelines

Editor-only metadata is carried separately on scene entities and is stripped to
neutral defaults by `runtime_clone()`.

## Universal authoring surface

`forge_authoring_core` owns authoring state:

- 2D / 3D / Hybrid modes
- cameras
- selection
- gizmo mode
- coordinate space
- grid
- snapping
- guides
- command/undo/redo stack

`forge_authoring_egui` is a host implementation for editor interaction and a
fallback visualization. Production applications may provide a native GPU
renderer through `forge_render_core::RenderBackend` while preserving the same
authoring state and tool contracts.

## Runtime

`forge_runtime_core` starts with:

- headless / rendered / editor-preview modes
- fixed-step simulation clock
- input snapshots
- runtime system registration
- scene lifecycle
- pause/resume/stop
- deterministic frame/fixed-step separation

Future runtime modules may add audio, physics adapters, networking, asset
streaming, ECS specialization, scripting, and renderer implementations without
coupling the shared runtime to a specific game.

## PIE bridge

`forge_runtime_bridge` supports:

- Simulate
- Play In Editor
- Play From Here
- Detached
- Standalone
- Headless

The bridge runtime-clones the authoring scene and restores the editor snapshot
on stop unless a caller explicitly requests runtime changes to be retained.

## Consumer model

A small utility may use only:

```text
forge_gui_core
forge_gui_theme
forge_gui_widgets
```

An editor may use:

```text
forge_gui_*
forge_scene_core
forge_authoring_core
forge_authoring_egui
forge_render_core
forge_runtime_bridge
```

A standalone game/runtime can use:

```text
forge_scene_core
forge_render_core
forge_runtime_core
```

No Ember dependency is required by the core libraries.
