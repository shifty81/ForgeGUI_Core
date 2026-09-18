# ForgeGUI_Core v0.4.8

ForgeGUI_Core is an independently maintained Rust foundation for a modular development-system ecosystem.

It provides reusable GUI, workbench, authoring, scene, render, runtime, PIE, diagnostics, and application-integration infrastructure that can be consumed by Ember, Forge, Cortex, game editors, standalone utilities, and future applications without forcing them into one monolithic executable.

## Universal modular GUI core

ForgeGUI is a **general desktop GUI system first**. Applications compose independent
surfaces, tab stacks, toolbars, menus, status chrome, dialogs, data views and optional
custom-render surfaces without inheriting an editor-shaped layout. The canonical GUI Lab
opens in a generic modular application profile; its Canvas / Viewport tab is only one
optional authoring workspace used to certify 2D/3D/voxel/hybrid consumers.

The shell supports named profiles, content-first/full-window presentation, configurable
left/center/right/bottom/floating surfaces, surface visibility/locking, movable toolbar
docking, persistent visible workspace tabs, and semantic hover/pressed/selected/focus
styling. File/Edit/View/Settings/Help remains global application chrome and is visually
separated from consumer content.

## Repository architecture

```text
ForgeGUI_Core
├─ ForgeGUI       reusable application/workbench GUI
├─ ForgeAuthoring universal 2D / 3D / Hybrid authoring
├─ ForgeScene     shared authoring/runtime scene model
├─ ForgeRender    renderer-neutral render/camera/picking contracts
├─ ForgeRuntime   GUI-independent runtime + PIE/simulation hosting
└─ GUI Lab/Testkit canonical visual lab + library certification
```

Consumer applications own their domain models and project-specific tools. ForgeGUI_Core owns reusable mechanics and contracts.

## Current baseline

The current source contains the cumulative **FG-C01 through FG-C82** foundation plus the **M1–M6 universal modular shell normalization**:

- mature ForgeGUI 0.4.8 contracts and structural workbench
- semantic themes/elevation and reusable widget infrastructure
- Slim Icon, Labeled Tool, Dual Context, and Floating Pill rail families
- browser, Inspector, command-palette, console/notifications, document-map foundations
- shared 2D/3D scene transforms, hierarchy, layers/tags, and editor metadata
- renderer-neutral 2D / 3D / Hybrid render and camera contracts
- universal authoring-surface state, selection, grid/snap, gizmo, and undo foundations
- fixed-step headless/rendered/editor-preview runtime contracts
- Simulate / PIE / Play From Here / Detached / Standalone / Headless runtime bridge
- one canonical `forge_gui_lab` visual certification target; authoring/runtime coverage lives in shared crates and tests
- project-owned borderless shell with Minimize / Maximize-Restore / Close, draggable + double-click behavior, and ForgeGUI edge-snap/maximize fallback
- distinct File/Edit/View/Settings/Help global chrome separated from the content/workspace region
- modular left/center/right/bottom/floating surface hosts with tab stacks, visibility, docking and lock-in-place controls
- movable/dockable/floating/lockable universal toolbar and shell profiles from Minimal through Workspace/Kiosk
- optional Canvas / Viewport workspace with thin frame, rulers, vertical layers and HUD rather than a mandatory center-editor identity
- configurable semantic hover/pressed/selected/focus treatments plus theme-controlled surface/control rounding
- five built-in visual themes plus shared progress, gauge, scrollbar and segmented-choice primitives

## Clean reset

This package intentionally contains **no**:

- `target/`
- `.git/`
- `artifacts/`
- old debug bundles/logs
- applied-patch transaction backups
- stale Ember migration manifests
- pending patch transports

`Cargo.lock` is retained as the current workspace lockfile. It is excluded from `PACKAGE_MANIFEST.json` by policy but participates in the source fingerprint and should be committed.

## Project Control Center

Run:

```text
PROJECT_CONTROL_CENTER.cmd
```

Normal workflow:

1. **FULL QUALITY GATE / CERTIFY GREEN**
2. **COMMIT + PUSH CURRENT GREEN**
3. **Run & play / ForgeGUI_Core Lab**
4. Patch status / receipts
5. Project status / health
6. Package debug handoff
7. Advanced/provider operations

Patch intake accepts a raw root `.patch` or a root-drop `.zip` containing exactly one `.patch`. A successful update invalidates previous GREEN certification. Ordinary source patches now continue in-process; if a patch updates the PCC provider itself, the provider automatically relaunches so a manual restart is not part of normal intake.

## New repository setup

The clean source is configured for:

```text
https://github.com/shifty81/ForgeGUI_Core.git
```

The connected GitHub account did not expose a repository with that name at package time, so create the empty repository first if needed.

Then either run:

```text
SETUP_REPOSITORY.cmd
```

or use **Advanced → Git setup / remote status** in the PCC.

The PCC initializes `main` and defaults `origin` to the ForgeGUI_Core URL. The first source commit/push is intentionally performed only by option **2** after option **1** certifies the exact source GREEN.

## Certification authority

`PACKAGE_MANIFEST.json` is the clean-source byte/hash authority. `Cargo.lock` is separately retained for reproducible Cargo resolution.

The first gate may perform the one manifest-authorized `rustfmt` canonicalization because the artifact environment used to assemble this source does not provide Rust/Cargo. After that bootstrap, Full Gate is check-only unless an explicit repair operation is requested.

See:

- `docs/forge_gui/PROJECT_OWNED_CHROME_AND_VISUAL_TARGET.md`
- `docs/FORGEGUI_CORE_ARCHITECTURE.md`
- `docs/FORGE_UNIVERSAL_AUTHORING_RUNTIME.md`
- `docs/forge_gui/IMPLEMENTATION_ROADMAP.md`
- `docs/ROOT_DROP_UPDATE_INTAKE.md`
- `docs/INTERNAL_PCC_STANDARD.md`


### Modular surface interaction

The canonical Lab now defaults to a generic application workspace rather than an editor canvas. Surface title grips can be dragged between left, center, right, and bottom dock zones; matching zones form tab stacks. Surfaces can be floated, hidden, resized, or locked. The toolbar is independently dockable/floating/lockable. Canvas/viewport tooling remains an optional certification workspace rather than the framework's default identity.

The internal PCC uses source-authoritative `cargo run` for the Lab, displays the detected Lab generation on its main menu, returns completed interactive operations to that menu, and does not require manual restarts for ordinary source patches.

## Public drop-in application shell

ForgeGUI Lab is only the visual showcase. Consumer applications should build on the public
`forge_gui_shell` runtime instead of copying Lab code.

The minimal Rust/egui integration is:

```rust
let mut shell = ForgeShellState::new(
    ShellProfile::Standard,
    vec![ModularSurfaceState::new("home", "Home", SurfaceDock::Center)],
);
show_application_shell(root, &ctx, &spec, &mut shell, &theme, &mut content);
```

`ForgeShellContent` supplies application-owned menu, toolbar, status and surface contents.
ForgeGUI owns the shared application frame, dock zones, tab stacks, floating windows, locking,
visibility, resizable dock groups and JSON layout persistence.

Run `cargo run -p forge_gui_consumer_starter` for the independent consumer certification app.
It intentionally contains no copied ForgeGUI Lab shell implementation.
