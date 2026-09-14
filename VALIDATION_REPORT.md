# Validation Report — ForgeGUI_Core 0.4.8

## Scope

This report records static/package validation performed while constructing the recovery baseline. The artifact environment used to build this ZIP does not provide the Rust toolchain, so only the user's Windows project-owned PCC can produce the authoritative Rust GREEN result.

## Recovered and corrected

- Broader 13-panel reference Lab restored.
- Structural left/right/bottom rails separated from the center docking workspace.
- Consumer Lab uses the canonical `forge_gui::egui` facade rather than a direct egui dependency.
- `egui_dock 0.21.1` `TabViewer::id()` implemented with stable `PanelInstanceId` hashing.
- Panel factory consumer path takes an owned `PanelInstanceId`, avoiding the prior closure/HRTB inference failure.
- Duplicate panel/command registration is fail-closed.
- Semantic `IconId` mapping resolves through the Phosphor adapter.
- Canvas pan/zoom input semantics corrected.
- Real OSS smoke tests exercise ltreeview, tiles, kittest, and the virtual table adapter.

## PCC authority

- `1 = FULL QUALITY GATE / CERTIFY GREEN`.
- `2 = COMMIT + PUSH CURRENT GREEN` requires the exact certified source fingerprint.
- Patch intake precedes hygiene and processes at most one transport per PCC session.
- Touched preimages are transactionally backed up; receipts store postimages; rollback fails closed on postimage drift.
- Native PASS/FAIL uses process exit code only; stderr remains diagnostic output.
- Missing/stale Cargo.lock has a controlled repair lane and then returns immediately to `--locked` certification.
- Full Gate verifies complete package-manifest coverage and PCC self-tests before Rust certification.
- Full Gate is check-only after one manifest-proven bootstrap formatting pass.
- Gate/runtime/patch/commit failures produce a single structured handoff ZIP and open/select it in Explorer for interactive runs.
- Forge/Cortex can query non-interactive JSON status, health, capabilities, and operations.

## Static package validation before ZIP freeze

The final package generation step validates JSON/TOML parsing, package-manifest coverage, manifest SHA-256/byte counts, provider/version consistency, prohibited repair residue, and ZIP CRC/integrity. `target/`, `.git/`, artifacts, and root patch transports are intentionally absent from the shipped clean baseline. `Cargo.lock` is retained for reproducibility and is intentionally excluded from PACKAGE_MANIFEST.json governance.

## 0.4.8 recovery evidence

The 0.4.7 handoff reached `cargo check --locked --workspace --all-targets` and compiled every workspace crate through the ForgeGUI facade before stopping in the reference Lab Canvas. egui 0.36.2 exposes `smooth_scroll_delta()` instead of the removed `raw_scroll_delta` field. 0.4.8 updates the Canvas input adapter and uses explicit `f32` zoom arithmetic.

## Authoritative next test

Extract into a clean folder, verify the PCC banner reports provider `0.4.8`, and run option **1**. The first gate may perform the one-time manifest-authorized rustfmt canonicalization and generate `Cargo.lock`. Any genuine Rust/test/Clippy failure should produce one `artifacts/handoff/ForgeGUI_DebugBundle_*.zip`; that ZIP is the preferred next handoff.

## 0.4.7 recovery evidence

The 0.4.6 handoff reached real `cargo check --locked --workspace --all-targets` after bootstrap formatting and lock refresh. The first source failure was Rust E0004 in `forge_gui_widgets`: `IconId` is intentionally `#[non_exhaustive]`, so the downstream adapter requires a wildcard arm. 0.4.8 adds a forward-compatible fallback and retains all explicit Phosphor mappings.
