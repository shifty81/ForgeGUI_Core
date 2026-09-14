# Handoff Standard

ForgeGUI uses a single user-facing handoff lane: `artifacts/handoff/`.

Automatic failure boundaries and menu option 6 produce one `ForgeGUI_DebugBundle_*.zip`. The PCC prints the exact `[HANDOFF]` path and opens Windows Explorer with the bundle selected.

A handoff bundle carries the authoritative session log plus recent PCC logs, project/PCC manifests, GREEN receipt when present, recent patch receipts, environment/toolchain details, source fingerprint, Git state, pending patch names, and the failure reason.

`latest-handoff.json` records the most recent bundle for Cortex/Forge ingestion. Handoff generation never changes source authority or certifies GREEN.
Native-process failures include the tail of actual native output in `failure.txt`/context. Informational stderr from successful native tools is retained in the session log but is not treated as failure. Empty pending-patch collections serialize as `[]`.
