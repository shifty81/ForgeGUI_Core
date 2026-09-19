# FG-C77 — native floating-host dock trees (source candidate)

## Base and scope

Cumulative from the `2026-09-18 08:34` PATCH_FAIL debug source, including FG-C73, C74, C75, C76. This is **not** a full source rollup. No Windows Full Gate or compiled Rust testing was available when authored.

## Implementation

- A floating host owns a versioned `ModularDockTree` keyed by its native window identity, persisted under `forgegui.core.floating_dock_trees.v1`; the existing main-window tree remains the only authority for shell panels.
- All hosts reuse the shell recursive tab/split renderer, draggable splitter ratios, panel hide behavior, and center/edge preview geometry. Floating host tree restoration validates identity, repairs corrupt saved ratios and duplicates, and rejects panels owned by another host.
- Dragging a tab onto another leaf in the **same floating host** can form tabs or nested splits. A different host may attach a panel to a specified target leaf only after a candidate model validates; on commit, the previous host/shell tree drops the old ID. A self-drop and locked source are rejected.
- Native window sizing stays OS-owned and unchanged from FG-C72. Closing a native window redocks its panels instead of closing documents. No extra Dock buttons or native OS title bar have been introduced.
- Legacy floating active-tab and order keys remain readable for migration; new host trees carry the canonical ordered topology.

## Certification boundaries

The Rust tests added in this source have **not** been executed. This environment has no Rust toolchain or Windows GUI. Cross-native pointer event delivery depends on the host integration and has **not** been observed on Windows. The source supports the routing but does not establish end-to-end functional certification. Do not claim multi-monitor, DPI, or cross-window drag success without device testing.

The next pass should migrate the separately exported `forge_gui_egui` consumer adapter onto `ModularDockTree` (instead of maintaining an `egui_dock` authority) and build an automated Windows interaction harness. GPU canvas presentation is still separate work.

## When home

Place only the newest root-drop cumulative ZIP, unextracted, into ForgeGUI's root. Preserve and move the incompatible Cortex-targeted patch **outside root intake** before launching the PCC. Approve the cumulative patch, run Full Gate and send the debug bundle if any check fails. Test shell tab drag/splits, float Content and Properties, split them inside one floating host, resize/close/reopen, restart to check layout, then move tabs between windows and monitors. Do not force apply if the source hash changed.

The native-source release handler preserves a drag if the main-window pointer is
inside a measured shell leaf, giving the subsequent shell dispatcher its chance
to commit. It cancels stale source releases elsewhere. This is source-level
routing only until the actual Windows viewport event sequence is observed.
