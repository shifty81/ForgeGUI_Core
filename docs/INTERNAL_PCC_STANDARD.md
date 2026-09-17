# Internal PCC Standard

ForgeGUI follows the project-owned PCC convention used across the tool/game fleet.

- `1` = Full Quality Gate / certify GREEN.
- `2` = Commit + push **current certified GREEN only**.
- `3` = Run/play the source-authoritative ForgeGUI Lab.
- Every completed interactive action returns automatically to the main PCC menu.
- Patch intake occurs before hygiene or certification.
- GREEN is a persisted source fingerprint, not a console label.
- Any source-changing patch or repair invalidates previous GREEN.
- Commit/push fails closed if source changed after certification.
- Gate and runtime failures automatically create one structured debug handoff ZIP.
- Debug handoffs live under `artifacts/handoff`, print a `[HANDOFF]` path, and open Explorer selecting the generated ZIP for interactive runs.
- Machine-readable status/health/capability/operation queries are non-interactive and never trigger patch prompts.
- Native stdout/stderr are diagnostic streams; only the native process exit code determines PASS/FAIL.

## Provider baseline

Current provider: `forge.internal_pcc.v1 / 0.4.13`.

The provider is project-owned and remains usable without Forge/Cortex. External Forge/Cortex integration is optional and queries the same project-owned provider operations.

## Automatic patch intake and normalization

A root patch transaction owns its complete preparation lifecycle:

- Root `.zip` transports are staged when they contain exactly one `.patch`.
- Patch application is explicitly approved by the user and transactional.
- Multiple pending patches are handled as an ordered queue.
- Ordinary source patches continue in the same PCC process; a manual restart is not required.
- If a patch changes the PCC provider source, the updated provider is relaunched automatically in the same console.
- Older failing root patches may be quarantined as superseded only when a newer viable cumulative patch covers every path touched by the older patch.
- Rust source changes run `cargo fmt --all` followed by `cargo fmt --all --check`.
- Cargo workspace/manifest changes probe `cargo metadata --locked`; `Cargo.lock` is refreshed only when Cargo explicitly reports the lock as stale or missing.
- Any controlled formatter/lock transformation is followed by full `PACKAGE_MANIFEST.json` regeneration and verification.
- The patch receipt is written only after normalization succeeds; normalization failure rolls the transaction back and produces a debug handoff.
- A successful update enters `PATCH_APPLIED_NEEDS_GATE`. Full Gate remains explicit and patch application never certifies GREEN automatically.

## Main-menu continuity

Interactive operations do not strand the user in submenus or terminate the PCC after normal completion.

- Full Gate returns to the main menu.
- Commit/push returns to the main menu.
- Run Lab returns to the main menu when the Lab closes.
- Patch status, project health, debug handoff, and advanced actions return to the main menu.
- The main menu retains a short `LastAction` summary and displays the detected Lab generation.
- `run.lab` uses `cargo run --locked -p forge_gui_lab`, making the source authoritative and preventing a stale executable from silently representing the current GUI.

## Startup formatting authority recovery

The PCC runs `cargo fmt --all --check` as a formatting authority probe. If a pristine package is explicitly marked `requires-canonicalization`, the provider may perform the governed one-time bootstrap formatting transaction, refresh the package manifest, and return the project to `NEEDS_GATE`.

The recovery never certifies GREEN. Certification remains an explicit Full Quality Gate.
