# ForgeGUI_Core Root-Drop Update Intake

ForgeGUI_Core supports two update transports:

1. A raw `.patch` file dropped directly into the project root.
2. A root-drop `.zip` containing exactly one `.patch` payload.

On interactive PCC startup, ZIP transport staging runs before ordinary patch
scanning. A qualifying ZIP is extracted transactionally into a root `.patch`,
then the ZIP transport is archived under `artifacts/patches/transports/`.

The normal ordered intake rule still applies: exactly one patch is approved and
applied per PCC session, then the PCC exits and must be restarted before any
later transport is considered.

ZIP intake is fail-closed:

- ZIPs with no `.patch` are ignored.
- ZIPs with more than one `.patch` are ignored.
- A same-name root patch with different contents blocks ZIP staging.
- Patch application still uses strict `git apply --check --whitespace=error-all`.
- Transaction backup, receipts, rollback, GREEN invalidation, and postimage
  checks remain owned by the existing patch pipeline.

## Bootstrap note

The PCC state before this change does not understand ZIP transports. Therefore,
the FG-C01-C40 BOOTSTRAP R2 update itself must be dropped into the root as the
raw `.patch` file once. After that update is applied and the PCC is restarted,
future ForgeGUI_Core root-drop ZIP packages can be used directly.
