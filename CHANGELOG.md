# Changelog
- PCC provider 0.4.13 stages every valid root ZIP in one intake pass, auto-quarantines safely superseded stale root patches before ordered intake, reloads provider changes in the same console, and returns automatically to the main menu without an extra Enter prompt.

## Unreleased — M1–M6 universal modular shell normalization

- PCC provider 0.4.12 now returns every completed interactive action to the main menu, keeps ordinary patch application in-process, self-reloads only when provider source changes, and reports the active Lab source generation.
- `run.lab` is explicitly source-authoritative through Cargo so a stale pre-patch executable cannot masquerade as the current GUI.
- Modular surface and toolbar headers now expose drag grips with live dock previews; surfaces can be re-docked by drag and tab-stack automatically when they share a dock.

- Removed the normal manual-PCC-restart requirement after patch application; ordinary patches continue in-process and PCC-provider updates request an automatic provider reload.
- Added borderless-window top/side snap handling around the existing draggable/maximize/restore chrome.
- Reframed the canonical Lab around generic Application / Dashboard / Canvas workspaces, with Canvas explicitly optional.
- Added configurable left/center/right/bottom/floating modular surface hosts with visible tab stacks, hide/show, locking and dock controls.
- Added top/bottom/left/right/floating/hidden toolbar placement with lock policy.
- Added Shell Profile controls for Minimal, Content First, Standard, Workspace and Kiosk presentation.
- Promoted Settings to global application chrome and exposed theme, density, interaction highlight modes, rounding and window-snap policy.
- Expanded semantic interaction tokens for hover/pressed/selected/focus presentation and normalized rounded ForgeGUI surfaces.


## Unreleased — Project-owned shell / single GUI Lab normalization

- Made the canonical `forge_gui_lab` launch without native operating-system window decorations.
- Added ForgeGUI-owned draggable title chrome with Minimize, Maximize/Restore, and Close controls plus double-click-title maximize/restore behavior.
- Added invisible project-owned edge/corner resize handles so manual window placement and sizing remain available without a Windows title frame.
- Retired the duplicate `forge_gui_next_lab` and `forge_authoring_lab` executables from the workspace and Full Gate; one canonical GUI Lab is now the visual certification surface.
- Raised the project title strip and base corner radius slightly to better match the approved dark graphite / mint concept references.
- Installed the shared icon font from the chrome visual bootstrap so the canonical Lab cannot silently render missing semantic glyphs.
- Locked the four supplied GUI concept images as visual-direction references while retaining dock/workbench focus-fill as a workspace action separate from native-style window maximize/restore.

- Normalized the permanent application menu to File / Edit / View / Help.
- Made the center infinite canvas the default-priority surface and made document tabs/bottom tray optional.
- Added grouped universal left rail and pinned/hover-edge reveal behavior for left/right workspace chrome.
- Slimmed the canvas frame/rulers, converted layer controls to a compact built-in vertical stack, and retained the lightweight canvas HUD.
- Added Forge Dark, Midnight Mint, Graphite, Warm Ember, and High Contrast theme presets.
- Added themed native scrolling plus reusable project-owned progress, radial gauge, scrollbar, and segmented-choice widgets.
- Added the canvas-first shell standard and expanded widget catalog/roadmap for reusable consumer adoption.
- Separated File / Edit / View / Help as stronger global application chrome with its own lower separator.
- Increased the bottom status strip for clearer status text and added active-layer feedback.
- Added subtle rounded outer application corners through transparent borderless shell chrome.
- Wired the demo layer visibility controls to World, Entities, Lighting, and Guides preview content instead of leaving them cosmetic-only.

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

## FG-C75 (2026-09-18) — cumulative nested-docking foundation

Added an exported, versioned nested tab/split docking model with safe saved-layout reconciliation and target-relative move operations, plus model unit tests. Hardened native floating transfer against dropping a panel onto its own host and against a stale drag surviving a real pointer release. These changes are bundled cumulatively with C73/C74; arbitrary nested panes are not yet rendered by the Lab and the Windows Full Gate has not run in this authoring environment. See `docs/forge_gui/FG_C75_NESTED_DOCK_MODEL.md`.

## FG-C76 — live nested dock renderer (2026-09-18, source candidate)
- Replace the GUI Lab fixed region layout with live tree-driven tabs, arbitrary recursive splits, draggable ratios and exact leaf-local center/edge drop previews.
- Persist the versioned dock tree independently of legacy per-panel storage; reconcile hide/reopen/native hosts without duplicating panel IDs.
- Retain C73–C75 frameless native floating hosts, group tabs, resize safeguards and prior source fixes. Windows Full Gate remains pending.

## FG-C77 — native host nested docking (2026-09-18, cumulative source candidate)
- Added validated, persisted native-host dock subtrees and reused the same recursive tab/split renderer inside floating windows.
- Added leaf-targeted same-host and cross-host tab/split transactions with source-tree detach after candidate commit, plus model and GUI Lab source tests.
- Preserved C73–C76 and C72 size ownership. Native drag and Windows Full Gate still require testing.

## FG-C78 — canonical consumer adapter source candidate (Windows gate pending)

- Migrated `forge_gui_egui::ForgeGuiRuntime` from independent `egui_dock::DockState` ownership to the shared, versioned `ModularDockTree` used by GUI Lab.
- Added canonical consumer tree rendering, nested split resizing, tab/edge drag transactions, floating group model, instance identity mapping, and eight unexecuted Rust test functions.
- Added `forge_gui_chrome` to the consumer Cargo dependency/lock entry. Legacy `egui_dock` package remains declared temporarily pending certified lockfile pruning, but its data model is no longer used by consumer source.
- Preserves cumulative C73–C77 work and all earlier native GUI Lab features. External consumer API migration from `.dock_state()` to `.dock_tree()` is necessary. Windows Cargo/Full Gate pending.
