# ForgeGUI_Core Gate Checklist

The internal ForgeGUI_Core PCC remains the authoritative independent library gate.

## Required for the cumulative FG-C01…C40 batch

1. Package-manifest/source governance passes after repairing the accidental Phase-1 overwrite.
2. `cargo fmt --all -- --check`
3. `cargo check --workspace --all-targets`
4. `cargo test --workspace --all-targets`
5. `cargo clippy --workspace --all-targets -- -D warnings`
6. Canonical `forge_gui_lab` builds as the only runnable visual Lab.
7. The Lab launches without native OS decorations.
8. ForgeGUI title chrome exposes Minimize + Close only and does not expose native Maximize.
9. Project-owned edge/corner resize handles retain manual window resizing.
10. All four rail presets render without clipping.
11. Rail collapse/expand and badge behavior work.
12. Asset Browser remains usable with a long virtualized list.
13. Inspector edits return the expected property IDs/values.
14. Command Palette opens, filters, invokes and closes.
15. Document-map marker lane renders and returns normalized navigation positions.
16. Focus/fill mode preserves state while hiding structural side/bottom regions.
17. Bottom tray cycles collapsed/normal/expanded.
18. Scene hierarchy validation passes.
19. Runtime clone neutralizes editor-only state.
20. Authoring mode switches between 2D / 3D / Hybrid.
21. Undo/redo round-trips scene transforms.
22. Headless runtime starts without GUI dependencies.
23. Fixed-step runtime caps substeps.
24. PIE/runtime bridge starts from an authoring scene and restores authoring state.
25. Existing 0.4.8 tests remain GREEN.
26. No Ember/Cortex/Havenwild/Subspace domain type is introduced into the core libraries.

## Visual certification expansion

Add fixtures for:

- 1366x768 compact
- 1920x1080 @ 100%
- 2560x1440 @ 125%
- 3840x2160 @ 150%
- keyboard-only rail/palette navigation
- high-contrast theme
- focus mode
- bottom tray states
- all four rail presets
- 2D authoring mode
- 3D authoring mode
- Hybrid authoring mode
