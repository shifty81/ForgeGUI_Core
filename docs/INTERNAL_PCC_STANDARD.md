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
