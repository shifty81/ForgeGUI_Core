# Changelog

## 0.4.8 — egui 0.36 Canvas input compatibility

- Fixed the next authoritative Rust gate blocker in the recovered Canvas: egui 0.36.2 no longer exposes `InputState::raw_scroll_delta`.
- Canvas zoom now reads `InputState::smooth_scroll_delta()` and keeps hover-gated, pointer-centered zoom behavior.
- Zoom arithmetic is explicitly `f32`, eliminating the follow-on ambiguous-float `clamp` error.
- Re-scanned the recovered Lab for other uses of the removed scroll field; this was the only occurrence.
- Preserved the full 0.4.7 recovery baseline, hardened PCC/provider contract, structural rails, semantic icons, OSS certification lane, and manifest authority.

## 0.4.7 — Forward-compatible semantic icon adapter

- Fixed the first real 0.4.6 Rust gate blocker: `forge_gui_widgets` now handles the public `#[non_exhaustive] IconId` contract with a stable fallback arm.
- Preserved explicit Phosphor mappings for known semantic icons while allowing future icon variants to degrade to their renderer-neutral text fallback instead of causing downstream E0004 compile failures.
- Re-audited all `#[non_exhaustive]` declarations across the workspace; `IconId` is currently the only such public enum and this was the only illegal exhaustive cross-crate match.
- Preserved the recovered structural rails, richer reference Lab, hardened PCC/provider contract, OSS certification lane, manifest authority, and transactional patch intake from 0.4.6.

## 0.4.6 — Recovery / foundation hardening

### Recovered GUI surface
- Restored the broader reference Lab: Project Explorer, Asset Browser, Scene/Room Canvas, Inspector, Layers, Console/Cortex, Notification Center, Problems, Timeline, Visual Graph, History, Curve Editor, Settings, command palette, PIE controls, and status/tool rails.
- Replaced repeated dock-tree splits with true ForgeGUI structural left/right/bottom rails; only the center document workspace uses free `egui_dock` docking.
- Restored finite/infinite canvas behavior, adaptive grid/snap, minimap proof, Alt/middle panning, cursor-oriented hover-gated zoom, and PIE/runtime input proof.
- Made `PreferredDock` affect runtime placement and added singleton/multi-instance enforcement.
- Added panel-instance cleanup when center tabs close.

### Contracts and adapters
- Restored namespaced stable IDs, richer panel definitions, interface presets, dock policies, selection/property contracts, capabilities, contribution manifests, and contract testkit helpers.
- Made duplicate panel/command/preset registration fail closed without replacing existing entries.
- Normalized public backend/service failures on typed `GuiResult` / `ForgeGuiError`.
- Preserved the canonical `forge_gui` consumer facade and kept `egui_dock` implementation details out of that facade.
- Mapped semantic `IconId` values to actual Phosphor glyphs.
- Retained virtualized `egui_extras` table support.
- Added real compile/smoke coverage for `egui_ltreeview`, `egui_tiles`, `egui_kittest`, semantic icons, and the virtual table adapter.

### PCC and source authority
- Fixed Windows PowerShell 5.1 native execution so stderr progress cannot become a false failure; only native exit code decides PASS/FAIL.
- Added complete package-manifest coverage/hash verification and PCC self-tests.
- Full Gate is read-only after one manifest-authorized bootstrap rustfmt canonicalization.
- Added controlled Cargo.lock generation/stale-lock repair followed immediately by locked recertification.
- Restored `1 = Full Gate`, `2 = Commit + Push Current GREEN`, real Lab launch, automatic single-ZIP handoffs, Explorer selection, and GREEN source-fingerprint enforcement.
- Ordered patch intake now applies at most one transport per PCC session, stops on decline/failure, backs up touched preimages, writes postimage-safe receipts, supports rollback, and requires provider restart after successful apply.
- Restored machine-readable Forge/Cortex provider queries for status, health, capabilities, and operations.
- Restored canonical `shifty81/ForgeGUI_Core` repository bootstrap for a virgin certified source tree.

## 0.4.5
- Corrected the Lab consumer boundary to use the public `forge_gui::egui` re-export.

## 0.4.4
- Reworked native-process capture around exit-code authority and cleaner Windows PowerShell diagnostics.

## 0.4.3
- Added controlled missing/stale Cargo.lock recovery while retaining locked certification.

## 0.4.2
- Standardized automatic single-ZIP handoff and Explorer-open behavior.

## 0.4.1
- Fixed PCC argument forwarding, restored GREEN commit/push, and corrected the required `egui_dock::TabViewer::id` implementation.

## 0.4.0
- Introduced semantic icon/widget foundations, the OSS evaluation lane, and manifest-aware root hygiene.
