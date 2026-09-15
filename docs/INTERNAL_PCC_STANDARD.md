# Internal PCC Standard

ForgeGUI follows the project-owned PCC convention used across the tool/game fleet:

- `1` = Full Quality Gate / certify GREEN.
- `2` = Commit + push **current certified GREEN only**.
- `3` = Run/play the project validation target.
- Patch intake occurs before hygiene or certification.
- GREEN is a persisted source fingerprint, not a console label.
- Any source-changing patch or repair invalidates previous GREEN.
- Commit/push fails closed if source changed after certification.
- Gate and runtime failures automatically create one debug handoff ZIP.
- Debug handoffs live under `artifacts/handoff`, print a `[HANDOFF]` path, and open Explorer selecting the generated ZIP.
- Manual option 6 creates the same handoff bundle and opens Explorer.
- Handoff bundles include the current session log, recent PCC logs, GREEN state, PCC providers, patch receipts, environment data, and failure context.
- `artifacts/handoff/latest-handoff.json` points to the most recent handoff artifact.
- Extracted standalone source can be initialized as Git from the PCC. The canonical default remote is `https://github.com/shifty81/ForgeGUI_Core.git`; a different remote can still be supplied explicitly.
- Native tool stdout/stderr are diagnostic streams; only the native process exit code determines PASS/FAIL. Normal Cargo progress on stderr must never become a PCC failure.

- Machine-readable status/health/capability/operation provider queries are non-interactive and never trigger patch approval prompts.
- Full Gate emits a non-opening PASS handoff bundle as checkpoint evidence as well as automatic opened bundles on failures.

## Automatic patch normalization

A successful root patch transaction owns its complete preparation lifecycle:

- Patch application remains explicitly approved by the user and transactional.
- Rust source changes automatically run `cargo fmt --all` followed by `cargo fmt --all --check`.
- Cargo workspace/manifest changes automatically probe `cargo metadata --locked`; Cargo.lock is refreshed only when Cargo explicitly reports the lock as stale or missing.
- Any controlled formatter/lock transformation is followed by full `PACKAGE_MANIFEST.json` regeneration and verification.
- The patch receipt is written only after normalization succeeds; normalization failure rolls the transaction back and produces a debug handoff.
- A successful update enters `PATCH_APPLIED_NEEDS_GATE`. The PCC exits/restarts cleanly, but no separate rustfmt or Cargo.lock repair should normally be required.
- `Run & play` is blocked until the current source has completed a Full Quality Gate and has a current GREEN fingerprint.
- Full Gate remains explicit. Patch application never certifies GREEN automatically.
- Manual rustfmt and Cargo.lock actions remain available only as repair/fallback operations.
- Older failed root patches that are fully covered by a newer successfully applied cumulative patch may be archived as superseded when the older transport predates the newer one and no longer applies.

## Startup formatting authority recovery

Provider 0.4.10 closes a provider-transition edge case discovered while promoting the Creator
Studio. `PACKAGE_MANIFEST.json` being marked `canonical` is not by itself sufficient proof that
Rust source written by an older provider has passed through the current `rustfmt`.

On normal PCC startup, when no explicit normalization recovery is already pending, the provider
runs `cargo fmt --all --check` as the formatting authority probe. If formatting drift is detected,
the PCC automatically performs the governed post-patch normalization transaction:

- `cargo fmt --all`
- `cargo fmt --all --check`
- controlled Cargo.lock reconciliation
- `PACKAGE_MANIFEST.json` regeneration and verification
- `PATCH_APPLIED_NEEDS_GATE`

The recovery never certifies GREEN. It returns the project to the user ready for an explicit
Full Quality Gate.
