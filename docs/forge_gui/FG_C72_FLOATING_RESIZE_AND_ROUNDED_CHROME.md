# FG-C72 — native floating resize and rounded panel chrome

**Baseline:** FG-C71R rebased source. This update does not replace the GUI Lab, change the PCC, or modify Cargo. Apply after FG-C71R, not together with superseded FG-C71.

## Root causes and fixes

1. `show_native_surface` previously submitted `.with_inner_size(preferred_size)` to `show_viewport_immediate` **every frame**. The Lab simultaneously copied the current viewport inner size into `preferred_size` every frame. A native resize therefore became a feedback loop (particularly noticeable with Windows/DPI rounding). The new `NativeSurfaceLifecycle` issues `Some(opening_size)` **once per uninterrupted floating-window lifetime**. On all later frames the window builder omits `inner_size`, and the OS remains the size authority. Hidden/redocked panels drop out of the lifecycle. Reopen uses the last saved logical size.
2. Preferred geometry is not overwritten by a maximized window, non-finite measurements, or an implausibly small inner size while minimizing. Finite sizes are bounded before persistence.
3. Floating content no longer uses an unrounded full-rect fill. An opaque shell gutter surrounds the standard `modular_surface_frame`; its border, radius, and padding are inherited from one shared theme rule. Docked surfaces also get a narrow outer gutter so their radius is visible rather than merging into the same-color parent.
4. Native windows retain `with_decorations(true)` and opaque backgrounds. The **OS owns the outer window shape and title bar**. ForgeGUI now draws visibly rounded *interior panel chrome*. True custom rounded outer corners on Windows versions that do not supply native rounding require a separate Windows-specific DWM/host implementation and are not claimed here.

## Public API detail

`show_native_surface` now accepts `opening_size: Option<[f32; 2]>` in place of a required preferred size. Its caller must use `NativeSurfaceLifecycle::retain_visible` and `opening_size`; `None` relinquishes sizing. This is an intentional pre-1.0 API correction. Search for other consumers before promoting this to a stable SDK API.

## Windows acceptance checklist

- Rebuild through the project-owned PCC. Launch the *newly built* canonical GUI Lab, not an old executable.
- Float Content or Properties. Drag each native floating window's edges and corners continuously for several seconds; resize should track the pointer without bouncing, fighting the mouse, or snapping back to the previous size.
- Resize a window, redock, float it again, and verify its last normal size returns. Hide and reopen it and repeat.
- Maximize then restore a floating window. Redock and float again: the restored non-maximized size should be used rather than the maximized monitor dimensions.
- Check at 100%, 150%, and 200% Windows display scaling and while moving between monitors. Native OS window shape and corner policy may vary by Windows version.
- Compare docked and floating panel corners, border, padding, and opaque shell gutter using each theme; no desktop pixels should leak behind the GUI.
- Confirm F10 layout reset, native close-to-redock, panel lock, toolbar floating, keyboard/mouse interactions, and no regression to active panel state.

## Evidence status

This source-only patch is tested with `git apply --check` against the FG-C71R snapshot, patch reversibility checks, and ZIP integrity. This environment has no Cargo/Rust compiler or Windows session, so Full Gate and visual/resize acceptance **must** be verified locally; do not label this revision GREEN before those checks.
