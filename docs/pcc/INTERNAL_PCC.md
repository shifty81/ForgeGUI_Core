# ForgeGUI Internal PCC

ForgeGUI owns a first-class `forge.internal_pcc.v1` provider.

Normal menu:

1. **FULL QUALITY GATE / CERTIFY GREEN** — runs the real Cargo metadata, format, check, test, Clippy, and release-build commands; verifies the release Lab executable; then persists `artifacts/certification/current-green.json` containing the certified source fingerprint.
2. **COMMIT + PUSH CURRENT GREEN** — refuses to run unless the current authoritative source fingerprint exactly matches the last GREEN receipt and no root patch transports are pending. If an extracted full-source tree has no `.git`, the PCC can initialize Git and default to the canonical `https://github.com/shifty81/ForgeGUI_Core.git` remote (or accept an explicitly supplied alternative) before the first push.
3. **Run & play / ForgeGUI_Core Lab** — executes `cargo run --locked -p forge_gui_lab`.
4. Patch status / receipts.
5. Project status / health.
6. Package debug handoff + open folder.
7. Advanced/provider operations (format repair, root hygiene, GREEN receipt, Git setup, Cargo.lock regeneration).

Patch intake runs before root hygiene. Root hygiene is non-fatal. A successfully applied patch invalidates prior GREEN certification.
