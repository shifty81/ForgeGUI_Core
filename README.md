# ForgeGUI_Core v0.4.8

ForgeGUI_Core is an independently maintained Rust foundation for a modular development-system ecosystem.

It provides reusable GUI, workbench, authoring, scene, render, runtime, PIE, diagnostics, and application-integration infrastructure that can be consumed by Ember, Forge, Cortex, game editors, standalone utilities, and future applications without forcing them into one monolithic executable.

## Canvas-first GUI Core

ForgeGUI now treats the center authoring canvas as the priority surface. The canonical Lab
uses fully project-owned borderless chrome, File/Edit/View/Help menus, a grouped universal
left tool rail, a collapsible/edge-reveal right context surface, optional secondary trays,
and internal infinite-canvas slim ruler/vertical-layer chrome. Built-in theme presets and reusable
progress, gauge, scrollbar, command, Inspector, browser, rail, and panel primitives are
intended to let other projects begin from a polished GUI foundation and specialize only
what their workflows require. See `docs/forge_gui/FORGEGUI_CANVAS_FIRST_SHELL_STANDARD.md`.

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

The current source contains the cumulative **FG-C01 through FG-C82** foundation:

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
- project-owned borderless shell with Minimize / Maximize-Restore / Close, draggable + double-click title behavior, and a distinct File/Edit/View/Help application menu strip
- canvas-first workspace with a thin internal frame, slim context-aware rulers, compact vertical layer stack, HUD, grouped universal rail and collapsible context surface
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

Patch intake accepts a raw root `.patch` or a root-drop `.zip` containing exactly one `.patch`. A successful update invalidates previous GREEN certification and requires PCC restart before the next transport.

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
