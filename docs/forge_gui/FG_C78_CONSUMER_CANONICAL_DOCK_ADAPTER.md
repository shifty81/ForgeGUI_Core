# FG-C78 — shared consumer docking migration (cumulative source candidate)

This patch is cumulative from the September 18 08:34 pre-C73 debug snapshot. It contains C73–C77 and the following C78 source edits. It is **not** a source rollup and **not** a certified Windows build.

## Concrete implementation

- `forge_gui_egui::ForgeGuiRuntime` now stores one `ModularDockTree` for main panels and one `ModularDockTree` per floating host. It no longer stores a second `egui_dock::DockState`.
- The consumer adapter uses the exact same `forge_gui_chrome::docking::{DockNode, DockDrop, ModularDockTree}` data model and transactions as the Lab. Its renderer is a separate consumer-specific adapter, not a second docking data model.
- Stable, length-prefixed panel-instance keys preserve singleton/multi-instance identity when mapping the consumer's `PanelInstanceId` to canonical string IDs. `restore_layout` and `restore_layout_with_surfaces` validate stale, duplicate, invalid, wrong-host, and hidden state through canonical model reconciliation.
- Drag a tab to another leaf center for tabs or to an edge for splits; drag outside valid leaves to create an in-app floating host. Dock splitters and floating hosts share recursion, resizing, active tab behavior and repair. Consumer in-app windows do **not** pretend to be native OS windows: OS-native multi-viewport hosting stays the responsibility of a consumer-specific viewport bridge.
- `dock_tree()`, `dock_tree_mut()`, `floating_trees()`, `surfaces()`, `dock_relative()`, `activate_instance()`, `hide_instance()` and `restore_layout()` replace the previous public `DockState`-specific accessors. **Breaking adapter API change:** consumers calling `.dock_state()` / `.dock_state_mut()` must migrate to `.dock_tree()` / `.dock_tree_mut()` and canonical operations; do not silently assume those usages compile.
- Closing/hiding an individual panel retains the registered instance and allows singleton reopening without duplication. Moving between hosts builds a valid candidate in the destination first, then releases the previous owner.
- The legacy `egui_dock` package is temporarily still declared as a dependency in `forge_gui_egui/Cargo.toml` and present in Cargo.lock to avoid an unaudited lockfile-wide dependency prune before the Windows Cargo gate. The adapter no longer imports or constructs its types. Remove the unused dependency and regenerate the lock in a later separately gated pass.
- Added eight Rust test functions in the consumer adapter: canonical layout, collision-free keys, split/tab identity, cross-host ownership, hide/reopen, host-close redocking, and host-mode restrictions, and saved-state unknown-panel rejection. The source tests have **not been executed** here.

## Validation boundary

The developer environment lacks `cargo`, `rustc`, Rustfmt, and Windows. Only patch preflight/reversal, source-byte verification and ZIP checks can be performed here. Rust compilation, unit tests, multi-window pointer event routing, mixed DPI, Windows resizing, and independent consumer integration remain mandatory local certification checks. This is intentionally a source **candidate**, not a claim of a green gate.

The current debug snapshot does not provide `crates/forge_gui_egui/Cargo.toml`. The manifest edit is based on the supplied older full source and the current `Cargo.lock` dependency inventory; the PCC must reject it safely if the current manifest differs. If so, send its rebase debug bundle rather than forcing application.

## Root-drop guidance

Keep the incompatible Cortex-targeted patch and earlier C73–C77 transport files **outside the ForgeGUI root inbox**, preserved rather than deleted. Drop only the C73–C78 cumulative ZIP unextracted into the root, approve through existing PCC, run Full Gate and attach any failure bundle. No manual extraction or external repair script is required for normal source patch intake.

After a green gate: open center/left/right/bottom and floating panels; drag center/edge into same host and across hosts; resize nested splits and floating windows; hide/reopen singletons; restart/reload; test keyboard and mixed-DPI monitors; run the independent consumer acceptance executable if available. Treat any API break in an external project as an explicit consumer migration item.
