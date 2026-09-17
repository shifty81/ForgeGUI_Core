# M7 — Surface recovery and opacity contract

ForgeGUI is a reusable shell. The Lab's application workspace is not an authoring canvas.

## Implemented in this pass

- This cumulative patch disables native Lab viewport transparency directly; REBASED06 is not required.
- Viewport clear alpha is 1.0; shell, application chrome, raised windows and modular
  surface backgrounds use opaque RGB even when a theme token includes alpha.
  Translucent accents and overlays remain theme controlled.
- F10 and View → Restore complete layout restore **all** core surfaces, their
  default docking/visibility/lock/size, toolbar position and visible workspace tabs.
  Unlike the prior F10 shortcut, this is a full layout reset rather than a profile toggle.
- View → Surfaces supports Show all, Hide all, Reset arrangement. When all modular
  surfaces are hidden, an opaque empty workspace provides an explicit recovery action.
- The reusable restore operation matches stable surface IDs and deliberately preserves
  registered extension surfaces and their state. Consumer data models are not reset.

## Acceptance on Windows

1. Start the freshly rebuilt Lab (not an old shortcut); hide every surface via
   View → Surfaces → Hide all surfaces. Desktop pixels must never be visible.
2. Click Restore standard layout or press F10. Confirm center, left, right, bottom,
   visible workspace tabs and toolbar all return. Data should not be deleted.
3. Float and lock a surface, change toolbar docking, hide other panels, then use
   F10. Confirm the complete arrangement returns and no drag remains active.
4. Repeat transparency checks with a floating panel, no side panels, dashboard,
   content-first profile, window resized and maximized, and after changing themes.

These are manual runtime checks; the Rust unit tests only certify the deterministic
layout reset and opaque color contracts. Full PCC Gate and GUI acceptance are required
before calling this revision GREEN.
