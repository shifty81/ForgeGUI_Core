# FG-C76 — live nested dock-tree renderer (cumulative source candidate)

## Authority and provenance

This patch is cumulatively based on `patch-evidence/current-source/` in
`ForgeGUI_DebugBundle_DBG-20260918-083411-PATCH_FAIL-bc24074f.zip` and contains
all C73, C74, C75, and C76 source changes. The newer C71R/C72 changes already
present in that exact debug snapshot are left intact. This is **not** a full
source rollup or a Windows-certified release.

## Implemented in C76

- The GUI Lab's four separate fixed `egui::Panel` dock regions are replaced by
  a single, recursive `ModularDockTree` renderer within the central workspace.
  Existing application bars and the independently placeable universal toolbar
  remain separate shell chrome, not editor panels.
- `DockNode::Split` draws two recursively nested child rectangles and a
  draggable splitter. Split ratios are clamped in the core model and stored.
- `DockNode::Tabs` renders live panel headers and tab controls. Tab selection,
  dragging, and visibility resolve via stable panel IDs, not region-specific
  `active_left/right/center/bottom` variables.
- Measured leaf rectangles resolve the exact source-to-target operation:
  center creates tabs; edges create left/right/top/bottom splits. Preview
  geometry uses the same `drop_zone` and `drop_preview_rect` functions as
  the committed layout model. Self-targeting does not float a panel.
- Dragging a docked tab outside the workspace detaches it to a native host.
  Dragging a native tab into a recognized shell leaf calls `attach`; moving
  an existing docked tab calls `move_relative`. Floating-host tab merges
  detach sources from the in-shell tree after host reassignment.
- Persist the versioned tree under `forgegui.core.dock_tree.v1`; legacy
  `forgegui.core.modular_surfaces.v1` saves bootstrap a compatible tree. Saved
  unknown, hidden, floating, and duplicate IDs are pruned at restore and
  on every frame after menu/native-host changes. Restore Standard resets the
  tree and clears stale drag state.

## Explicit remaining limitations

- The shared `forge_gui_egui`/consumer shell still has its own historical dock
  runtime; C76 makes the **GUI Lab** use the C75 canonical nested model, but
  a project-wide adapter migration remains necessary. No claim that every
  consumer already uses this model.
- Floating windows still support **tab grouping**, not recursive split
  rendering within each floating native host. Cross-native viewport pointer
  capture, native-to-main drops, multi-monitor DPI, and resizing require the
  next bridge/Windows test passes.
- The application-level universal tool rail, toolbar and product/menu/status
  chrome intentionally stay outside the editor panel dock tree.
- A null render backend remains for the canvas example; no GPU certification
  is implied by the docking change.
- Cargo/rustc, Windows GUI and PCC are unavailable in this authoring
  environment. Source tests were added; their execution is pending the real
  project-owned Full Gate.

## Windows acceptance checklist

1. Ensure wrong-target Cortex patch is safely held outside ForgeGUI's root
   intake and superseded C73/C74/C75 transport files are not queued.
2. Place only this cumulative patch ZIP OR its standalone patch in the
   ForgeGUI root, approve via PCC and run Full Gate. Do not manually extract.
3. Drag Activity onto the middle of Properties: verify one tab group.
4. Drag Content onto each of the four edges of a destination leaf: verify
   nested splits in the correct direction, without replacing another leaf.
5. Resize all split handles; restart Lab: ratios, tab selection and arrangement
   must remain. Reset layout (F10) must restore a clean default tree.
6. Hide and reopen a panel; verify no duplicate tabs. Float, resize, and
   redock a panel; inspect for size jitter, transparent corners, duplicate
   title chrome and stale pointer events.
7. If the Windows build or drag test fails, capture the new PCC debug bundle
   before another patch is authored. Do not force a failed preimage apply.

Next: C77 stable cross-native viewport drag bridge and native floating split
hosts; C78 render and DPI certification; C79–80 library-wide dock authority
adapter and release readiness. No local build is claimed here.
