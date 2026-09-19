# FG-C75: validated nested docking model and native drag lifecycle

## Source authority and scope

This pass is based on the exact current-source snapshot in
`ForgeGUI_DebugBundle_DBG-20260918-083411-PATCH_FAIL-bc24074f.zip`, plus the
entire C73/C74 cumulative series. It is a *source patch*, not a full rollup or
Windows-certified executable. The latest supplied debug bundle precedes C73;
there is no evidence the user's local project has applied C74.

## Implemented

- `forge_gui_chrome::docking` exports a renderer-neutral, serialized
  `ModularDockTree` with `DockNode::{Tabs,Split}` and horizontal/vertical
  splitting. The model supports target-relative center/edge placement,
  nested splits, tab activation, ratio updates, native attach/detach,
  collapsed empty leaves and versioned saved-layout reconciliation.
- Ingestion rejects unknown, duplicate, hidden and floating panel IDs;
  drops stale active references; bounds non-finite split ratios, and caps
  depth to avoid malformed saved trees. Mutations use proposals so
  prohibited, invalid, or over-depth moves leave the source unchanged.
- Leaf-local geometric drop zones and matching previews are exported for
  the upcoming renderer. They are **not** advertised in the current Lab
  until the actual split renderer is connected.
- The *existing* floating Lab fixes one real drag bug: a floating tab cannot
  drop into its own native host and silently reorder itself. Only visible,
  unlocked panels in other hosts are eligible; an actual release in the
  source native viewport clears stale drag state even if pointer-hover
  continues to report a position.
- Adds Rust unit tests for tree bootstrap, nested edge/tab placement,
  empty-leaf pruning, corrupt save recovery, version migration, locked/self
  drop refusal, attach/detach idempotency, split ratio clamping,
  precise hit zones and native self-host guard.

## Not implemented or certified

The Lab **still renders its fixed left/right/bottom/center groups**. The new
model is a reusable, tested-in-source data layer but **not yet the live Lab
layout authority**. C76 must bridge Lab surface identities into this tree,
render arbitrary nested split leaves and tab groups, persist ratios, and
coordinate physical native viewport moves atomically with model commits.
Do not enable edge split previews before actual split render behavior exists.

No Rust/Cargo/Windows build is available in this authoring environment. A
unit test written in source is not a passing test until the PCC actually runs
it. Existing C72 resize behavior and C73 frameless windows remain untouched.

## Acceptance after applying this cumulative patch

1. Run PCC Full Gate, including `forge_gui_chrome` and `forge_gui_lab` tests.
2. Float Activity and Properties, then merge them into one native host.
3. Drag a tab onto its own host: it should not unexpectedly reorder/re-host.
4. Drag an unlocked different panel between floating hosts and release.
5. Begin a tab drag and release over its origin: the next ordinary click must
   not unexpectedly dock a stale surface.
6. Resize/maximize/restore floating windows, dock them, restart the Lab and
   verify persistence. Report debug bundle if behavior disagrees.

Next work: C76 live dock-tree renderer and persisted split path/ratio bridge;
C77 drag across native viewports and multi-monitor certification; C78-80
refine chrome, DPI, accessibility and visual regression.
