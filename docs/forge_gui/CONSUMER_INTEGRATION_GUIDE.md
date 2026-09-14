# ForgeGUI_Core Consumer Integration Guide

ForgeGUI_Core is an independent library repository. Consumers should depend on
the smallest module set they need and must not modify the library in-place to
add application-specific behavior.

## GUI-only consumer

Typical dependencies:

```toml
forge_gui = { git = "<ForgeGUI_Core repo>", rev = "<certified commit>" }
```

or, during local development:

```toml
forge_gui = { path = "../ForgeGUI_Core/crates/forge_gui" }
```

## Universal authoring consumer

Add the authoring/scene/render contracts:

```text
forge_scene_core
forge_render_core
forge_authoring_core
forge_authoring_egui   # only when using egui editor hosting
```

## Runtime consumer

A shipped runtime or headless test host can use:

```text
forge_scene_core
forge_render_core
forge_runtime_core
```

without any GUI dependency.

## PIE/editor runtime consumer

Add:

```text
forge_runtime_bridge
```

The consuming editor owns project-specific spawn rules, gameplay systems,
asset databases, scripting, physics, networking and renderer backends.

## Version policy

Applications should pin a known-GREEN ForgeGUI_Core commit/tag. Library
upgrades are explicit integration events and must pass the consumer's own
integration gate before promotion.
