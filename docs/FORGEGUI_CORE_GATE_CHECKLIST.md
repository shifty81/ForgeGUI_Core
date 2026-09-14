# ForgeGUI_Core Gate Checklist

The internal ForgeGUI_Core PCC remains the authoritative independent library gate.

## Required for the cumulative FG-C01…C40 batch

1. Package-manifest/source governance passes after repairing the accidental Phase-1 overwrite.
2. `cargo fmt --all -- --check`
3. `cargo check --workspace --all-targets`
4. `cargo test --workspace --all-targets`
5. `cargo clippy --workspace --all-targets -- -D warnings`
6. Existing `forge_gui_lab` builds.
7. `forge_gui_next_lab` builds.
8. `forge_authoring_lab` builds.
9. All four rail presets render without clipping.
10. Rail collapse/expand and badge behavior work.
11. Asset Browser remains usable with a long virtualized list.
12. Inspector edits return the expected property IDs/values.
13. Command Palette opens, filters, invokes and closes.
14. Document-map marker lane renders and returns normalized navigation positions.
15. Focus mode preserves state while hiding structural side/bottom regions.
16. Bottom tray cycles collapsed/normal/expanded.
17. Scene hierarchy validation passes.
18. Runtime clone neutralizes editor-only state.
19. Authoring mode switches between 2D / 3D / Hybrid.
20. Undo/redo round-trips scene transforms.
21. Headless runtime starts without GUI dependencies.
22. Fixed-step runtime caps substeps.
23. PIE/runtime bridge starts from an authoring scene and restores authoring state.
24. Existing 0.4.8 tests remain GREEN.
25. No Ember/Cortex/Havenwild/Subspace domain type is introduced into the core libraries.

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
