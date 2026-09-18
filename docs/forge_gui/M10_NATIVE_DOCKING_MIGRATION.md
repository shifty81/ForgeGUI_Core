# ForgeGUI M10 — Native detachable surface migration

Baseline: the 2026-09-17 15:48 failure debug bundle's `patch-evidence/current-source` files plus the uploaded ForgeGUI_Core rollup's unchanged `forge_gui_icons` source. This is an incremental pass, **not** a final docking certification.

## Implemented

- One reusable `forge_gui_chrome::show_native_surface` viewport helper using egui's native child viewport API (with explicit embedded fallback).
- The GUI Lab's modular floating surfaces now use separate operating-system windows instead of `egui::Window` inside the main application.
- The existing surface instance and its data models stay owned by the GUI Lab while moving; closing the child window redocks rather than closing its document.
- All current modular surface tabs have a drag gesture. The existing left/center/right/bottom drop previews remain the current placement targets.
- Surface movement records and restores the last non-floating dock; native window resize feeds the preferred dimensions back into the surface state. The defaulted new serialization field maintains compatibility with prior serialized surface state.
- Semantic Phosphor icons replace Unicode placeholders on main window controls and modular panel controls. The icon ID contract includes minimize, maximize, restore, detach, and dock.
- Existing custom Windows snapping logic is **unchanged** in this pass. It needs multi-monitor and Windows-native hit-test validation before modification.

## Remaining docking overhaul — do not claim complete

1. Converge `forge_gui_egui::ForgeGuiRuntime`'s existing `egui_dock::DockState` with `forge_gui_chrome::ModularSurfaceState`: right now two layout engines coexist. Define one panel registry and typed commands (move/split/tab/detach/close); delete the redundant GUI Lab docking loop only after parity tests pass.
2. Replace the fixed four-zone structural layout with a genuine split tree, drag insertion targets between nested groups, and tab reordering. Ensure all registered panels—including hosted third-party tools—share the same path.
3. Implement native detached-window drag-back across viewport boundaries. Current guaranteed redock path is each detached window's Dock menu or the OS close control. Dragging a tab out is a best-effort input path until cross-window pointer capture is certified on Windows.
4. Persist all window positions, sizes, monitor assignments, maximized states, dock trees, focus, tab order, panel visibility, and versioned layout migrations. This pass only maintains preferred size in live state.
5. Audit icon coverage across the full component catalog; this pass corrects window and panel chrome, not every missing icon.
6. Verify per-monitor DPI and native window drag, maximize/restore, Snap, Alt+Tab, taskbar behavior, and document close/save authority on Windows. None of those GUI behaviors can be certified from this Linux environment.

## Quality gate

Use the project-owned PCC Full Gate. Do not call GREEN from patch preflight alone. Run `cargo fmt --all -- --check`, `cargo check --workspace --all-targets`, `cargo test --workspace`, and `cargo clippy --workspace --all-targets -- -D warnings`, then exercise a native Windows GUI Lab session: detach every modular surface, edit content, resize, close/redock, lock/unlock, switch monitors, and restore layout.
