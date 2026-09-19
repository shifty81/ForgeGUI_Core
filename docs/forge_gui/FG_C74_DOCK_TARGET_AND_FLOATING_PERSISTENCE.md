# FG-C74 — Measured dock targets and floating-tab persistence

**Scope:** The current GUI Lab native-window lane, following FG-C73. This is a source change, not a Full Gate certification.

## Implemented

- Dock drops now use measured visible panel frame rectangles, rather than approximating all targets using fixed fractions of the entire application window. The smallest containing panel wins if rectangles overlap. Empty central workspace still resolves to the center.
- A drop over application title, menus, status or outside the shell cannot silently fall back to the center; a main-origin drag without a valid target becomes a detached floating panel. Native floating-origin releases outside the main viewport remain host-owned.
- Previews are drawn within the measured destination panel and state "Nest as <region> tab" rather than implying a structural split that the legacy Lab cannot perform.
- A floating-host tab target shows an accent border while hovered, not solely on the release frame. The existing C73 native host and C72 single-opening-size/no resize-feedback behavior are preserved.
- Persist active floating tabs and their order under new dedicated eframe storage keys while keeping the existing surface v1 layout key readable. Invalid/stale tab IDs are filtered, duplicates removed, and new surface IDs appended deterministically. Moving a tab into a group appends that tab once.
- Adds focused tests for panel-target resolution and saved-tab-order normalization.

## Explicit boundaries

The Lab **still uses fixed left/right/bottom/center structural regions**. Dragging a tab onto an existing region nests it as another tab; arbitrary nested *splits within a panel* do not yet exist. `forge_gui_egui::ForgeGuiRuntime` has a separate egui_dock tree. Migrating the Lab and the shared shell to one persisted authority without regressing independent native viewport support is the next structural milestone. Cross-native-window drag/release behavior must still be certified on Windows and multiple monitors. Do not report the project as having a fully unified dock tree on account of this pass.

## Compatibility and intake

The cumulative FG-C73+C74 patch is based on exact `DBG-20260918-083411-PATCH_FAIL-bc24074f` source snapshot, which was before C73. The incremental FG-C74-only patch is based on the same snapshot with C73 applied. **Use exactly one patch for the matching local baseline.** Neither requires earlier C71/C72 packages to be replayed: those changes are already in the referenced debug snapshot. Do not apply a cumulative and incremental patch together.

Before intake, preserve and move the confirmed wrong-target `ForgeGUI_Cortex_FirstClass_Surface_CTXFG02_20260918.patch` outside root intake, following `ForgeGUI_C73_Intake_Recovery_Handoff.md` or the earlier recovery tool. Do not delete or apply that Cortex patch to ForgeGUI. Remove superseded unconsumed C73 transport from root when selecting the cumulative patch; archive it outside intake. Avoid auto-approving patches.

## Acceptance checks on the user's Windows machine

1. Run PCC Full Gate and attach any debug bundle on failure. Verify Cargo format/check/tests/Clippy/release.
2. Drag Activity into Properties and Content: highlighted rectangle must match the visible panel, and the tab must appear in that group. Drop over main title/menu/status: no silent center redock.
3. Float Activity, drag Content into it, change the active tab, rearrange host membership, restart Lab: active tab, tab order, window size and dock destinations must persist correctly.
4. Resize floating groups on every side/corner, minimize/restore, test 100/150% DPI and a second monitor; watch for jitter, missing rounded inner chrome, input loss and stale floating windows.
5. Inspect native-window dragging/merging and report cross-window failures separately. Full nested edge splitting is NOT part of this pass.

## Local authoring verification

Git patch check/apply/reverse and ZIP integrity can be verified in the authoring container. Cargo, rustfmt, Windows PCC and GUI interaction checks require the Windows environment and are not claimed to pass here.
