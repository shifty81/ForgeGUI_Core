# FG-C71R — safe current-source rebase (2026-09-17)

## Rebase cause and source authority
The original FG-C71 was authored against an older donor ZIP. The 2026-09-17 21:53 PATCH_REBASE debug bundle instead shows a newer GUI Lab with native floating-window chrome, project modular surfaces, CI, and shared font initialization. The original patch cannot be applied without overwriting unrelated newer work.

The debug bundle also contains the current `Cargo.toml`, PCC, and package manifest. They **already** target the canonical `forge_gui_lab`; no Cargo, PCC, or icon-enum file is replaced in FG-C71R. The old donor's workspace and icon-enum assumptions were not representative of the current project. Only changed files whose current bytes were included in the bundle are modified.

## Changes
- Move the shared `ForgeGuiRuntime` left/right/bottom rail containers and center into **one** `egui_dock::DockState`. Route preferred panel regions to this dock tree, allow supported panels to float inside egui, and preserve singleton focus/location after panel movement.
- Keep the newer Lab and native-window `show_native_surface` implementation intact. Add persisted modular-surface dock/visibility/lock/size state using existing eframe persistence. On restore, merge saved state by ID into registered defaults; reject unknown IDs and non-finite sizes.
- Use existing semantic `IconId` values for browser item kinds and panel headers, expose `IconId` through the widget facade, and distinguish more icon mappings. No font setup or icon-enum replacement is needed from this snapshot.
- Set `[layout].persist = true` in the project GUI config. Append explicit status/limits to the feature matrix and roadmap.

## Apply
Place **one** of the supplied transports (`.patch` OR the ZIP containing that single patch) in the ForgeGUI project root, approve in the internal PCC, run Full Gate, and run the GUI Lab. Do not re-drop the older FG-C71 patch. Test panel movement/float/redock/close/reopen, application exit/restart with saved layout, reset layout, semantic icons, and Windows-native floating behavior. If the gate fails, use its new debug bundle.

## Verification and limits
The new patch is generated from, and application-checked against, the current file snapshots inside the 2026-09-17 debug bundle; the container cannot run Cargo, Windows, or the PCC Full Gate. Patch application is not a GREEN certification.

The GUI Lab still has its own `ModularSurfaceState`, separate from the shared library `DockState`. Full docking convergence is the next architectural milestone; GPU viewport, DPI/monitor transitions, and end-to-end drag/drop certification remain outstanding. This incremental patch does not claim they are complete.
