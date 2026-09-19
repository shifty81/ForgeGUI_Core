# FG-C73 — Frameless floating hosts and tab-group foundation

**Baseline:** FG-C72 after FG-C71R. This is an incremental source patch, not a replacement rollup. No PCC changes.

## Source changes

- A detached host is a genuine OS viewport but uses ForgeGUI's own rounded interior frame; the duplicate operating-system caption/title/buttons are disabled. Resize grips remain active on all viewport edges and corners. The OS continues to own live dimensions; the one-shot opening-size policy from C72 is preserved.
- Removes the per-panel **Dock** menus from docked and detached chrome. The compact Hide icon lives at the right edge. The View/Surfaces configuration menu remains as an accessible recovery path for keyboard users and locked panels.
- A floating host can own multiple panel IDs with an independent active tab. Drag the floating **tab**, not the window-move grip, to initiate docking. Drop a dragged tab over another floating host to join that host; hide affects only the active tab. Closing a host redocks its tabs to their own previous destinations.
- `floating_host` is an optional persisted field, backwards compatible with existing JSON. The original panel ID is the default host; invalid empty host names are discarded on restore. Sibling tabs share their last normal floating dimensions to avoid resize jumps when switching/hiding.
- Three model tests cover host grouping, locked panels, legacy host IDs; two GUI Lab tests cover persisted host IDs and stale host sanitation.

## Important remaining work — do not call docking complete

This pass addresses the visible popup chrome and gives floating tab hosts a shared group model. **It does not replace the Lab's four fixed left/center/right/bottom regions with a canonical recursive dock tree.** `forge_gui_egui::ForgeGuiRuntime` already has `egui_dock::DockState` with native split/tab operations, but the Lab currently bypasses it and maintains its own regions. A follow-up must migrate all panel rendering to one authoritative runtime, bridge native viewport hosts to that runtime, support split-docking inside existing panels on all four sides and center-to-tab, use precise target rects instead of whole-shell heuristics, and restore nested split proportions and tab order. The project must not ship two competing docking state machines.

The cross-viewport tab handoff path is a candidate implementation, **not Windows certified**. It depends on OS/egui pointer capture and event routing across native windows; it must be tested with tabs dragged from main→float, float→main and float→float. If focus/release is unreliable, add a host-level drag transaction/event bridge rather than relying on pointer state inferred from the main viewport.

## Acceptance checklist (Windows)

1. Apply only this `.patch` or ZIP transport via the PCC after FG-C72. Full Gate must be green before accepting.
2. Float Activity. It must display one rounded ForgeGUI panel with no second Windows caption, a compact right-aligned Hide icon, a draggable window grip, a draggable content tab and functional edge/corner resize handles without C72 jitter.
3. Float Content; drag its tab onto Activity. Confirm one native window with two tabs, the same live content/selection and no duplicate document instance. Switch tabs and resize the host.
4. Drag a nested tab into the main app and back to another floating host; confirm state is preserved and locked tabs cannot move.
5. Hide one grouped tab, reopen it from View/Surfaces, close the host via Alt+F4, restart the app, and verify layout persistence and previous-dock restoration.
6. Repeat at 100%/150% DPI and across two monitors; record any failed transfer and use the debug bundle for a follow-up.

**Verification in authoring container:** Git whitespace and exact-baseline apply/reverse checks can be run. Cargo/Rust/Windows GUI are not available here; no compile or interaction PASS can be claimed.
