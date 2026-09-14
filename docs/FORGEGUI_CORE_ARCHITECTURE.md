# ForgeGUI_Core Architecture

ForgeGUI_Core is an independently maintained, versioned modular development
foundation. The repository contains three separable library families:

```text
ForgeGUI_Core
├─ ForgeGUI       GUI/workbench framework
├─ ForgeAuthoring universal authoring framework
└─ ForgeRuntime   runtime/scene/render foundation
```

This repository is not an Ember subsystem. Ember, Forge, Cortex, game-specific
editors, standalone tools, and future applications are consumers.

## Dependency direction

```text
Applications / Project Editors / Tools
              │
      ┌───────┴────────┐
      ▼                ▼
   ForgeGUI       ForgeAuthoring
      │                │
      └───────┬────────┘
              ▼
         ForgeScene
         ForgeRender
              │
         ForgeRuntime
              │
           Platform
```

The runtime and scene layers are GUI-independent.

## Mature GUI modules retained

- `forge_gui_core`
- `forge_gui_icons`
- `forge_gui_widgets`
- `forge_gui_canvas`
- `forge_gui_console`
- `forge_gui_notify`
- `forge_gui_pie`
- `forge_gui_egui`
- `forge_gui`
- `forge_gui_sdk`
- `forge_gui_testkit`
- `forge_gui_oss_lab` as reference/testing material

## Shared GUI modules added

- `forge_gui_theme`
- `forge_gui_rails`
- `forge_gui_workbench`
- `forge_gui_browser`
- `forge_gui_inspector`
- `forge_gui_command`
- `forge_gui_document_map`

## Universal authoring/runtime modules added

- `forge_scene_core`
- `forge_render_core`
- `forge_authoring_core`
- `forge_authoring_egui`
- `forge_runtime_core`
- `forge_runtime_bridge`
- `forge_engine`

## Architectural rules

1. No game/project domain type belongs in the shared core.
2. `forge_runtime_core` must compile without egui.
3. `forge_scene_core` is shared by editor and runtime.
4. Editor-only scene metadata must be removable/neutralizable for runtime use.
5. Rendering is backend-driven through `forge_render_core`.
6. The egui authoring host may display fallback content but does not become the
   only render backend.
7. Applications pin certified ForgeGUI_Core revisions rather than silently
   inheriting library changes.
8. ForgeGUI Lab and ForgeAuthoring Lab are certification harnesses, not product
   shells.
