# FG-C41…FG-C55 Creator Studio Foundation

This milestone moves ForgeGUI Core away from a generic dark IDE shell and establishes the reusable Creator Studio grammar intended for all ForgeGUI-powered game editors and software.

## Architecture

The Creator Studio is split into four layers:

1. **ForgeGUI Chrome** — product identity, menus, action strip, workspace tabs, tool tray and status bar.
2. **ForgeGUI Panels** — reusable panel frames, headers, sections and panel actions.
3. **ForgeGUI Workspace** — the central authoring surface and editor-local tool strip.
4. **ForgeRender Surface Host** — renderer lifecycle, sizing, frame scheduling and backend attachment.

The renderer is not an egui canvas. `RenderSurfaceHost` owns a real `RenderBackend` implementation. ForgeGUI only provides the chrome and authoring overlays around the backend surface. The current Lab intentionally uses the existing `NullRenderBackend`; game/editor consumers can attach a GPU-backed implementation without changing the workspace UI contract.

## Supported renderer families

The authoring workspace exposes profiles for:

- 2D
- 2.5D
- 3D
- Voxel
- Hybrid

These are authoring profiles. They map onto the current renderer-neutral `SurfaceDimension` contract while allowing future game-specific render extensions underneath the surface host.

## Default Creator Studio layout

The first Creator preset uses:

- Inspector on the left
- Asset/Content Browser on the right
- renderer-backed Workspace in the center
- Console / Problems / Build / Runtime / Cortex tool tray on the bottom
- product/menu/action/workspace chrome above
- status chrome below

This arrangement is a preset, not a hard-coded project assumption.

## Boundary rule

ForgeGUI Core owns reusable UI/workspace mechanics. Projects own gameplay semantics, project-specific assets, scene components and renderer extensions. Runtime and scene crates remain independent from egui.

## Certification note

This patch deliberately marks the package manifest `requires-canonicalization` because the artifact environment cannot run Rust/rustfmt. The internal PCC is expected to perform its manifest-authorized one-time `cargo fmt --all`, regenerate hashes, then run the normal locked workspace gate. Local PCC GREEN remains authoritative.
