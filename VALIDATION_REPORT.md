# Validation Report — ForgeGUI_Core 0.4.8 / PCC 0.4.13

## Reconstruction authority

This clean source was reconstructed from the user's supplied `crates.zip`, whose embedded Git worktree was clean on `main` at commit:

`532f7e1a0dfe4d7eefc46680a1567242059640a7`

The supplied certification archive contains the authoritative prior Full Gate pass:

`QG-20260915-183828-full-c5c3f4b8`

with source fingerprint:

`5d6c9f6af46d4d9021ccbfb3e8124826d2043ba2ff73ae49fbdae5e75bbebd51`

The later debug bundle also reported the same clean commit and fingerprint, confirming the repeated failed M1–M6 patch attempts had not modified governed source.

## Recovered forward changes

The clean reconstruction applies the latest live-tree M1–M6 source delta directly instead of shipping pending patch transports. It includes:

- universal modular Application / Dashboard / Canvas workspace model
- generic left/center/right/bottom/floating surface hosts and tab stacks
- movable/dockable/floating/lockable toolbar state
- generic Application workspace as the canonical Lab default
- optional Canvas / Viewport authoring workspace
- project-owned borderless chrome with maximize/restore and edge-snap fallback
- configurable surface/control rounding and semantic hover/pressed/selected/focus treatment
- PCC provider 0.4.13 with ordered patch queues, same-session ordinary patch application, automatic provider reload, source-authoritative Lab launch, and automatic return to the main menu

## Audit repairs included in this clean source

- Removed CI references to the retired `forge_gui_next_lab` and `forge_authoring_lab` packages.
- CI now explicitly release-builds the four current certification applications.
- Updated the Internal PCC Standard from provider 0.4.10 behavior to 0.4.13 behavior.
- Reframed Canvas-first documentation as an optional authoring profile rather than the universal GUI default.
- Changed `forge.gui.toml` from the old `game_maker` default to the generic `universal_modular` / `forgegui.core.application` profile.
- Regenerated `PACKAGE_MANIFEST.json` after reconstruction so the shipped hashes describe this source exactly.
- Removed `.git/`, `artifacts/`, `target/`, debug bundles, patch transactions, and root patch transports from the clean source package.

## Static validation performed during reconstruction

The artifact environment does not provide Rust/Cargo, so it cannot replace the user's Windows PCC Full Gate. Before packaging, the reconstruction verifies:

- all JSON files parse
- all TOML files parse
- GitHub workflow YAML parses
- every Cargo workspace member directory and `Cargo.toml` exists
- every explicit CI certification package exists in the workspace
- no retired Lab package remains in the CI workflow
- `project.control.json` reports PCC provider 0.4.13 and no-normal-restart patch policy
- `forge.gui.toml` reports the universal modular application profile
- `git diff --check` reports no whitespace errors against the recovered green Git baseline
- the clean-source ZIP contains no `.git/`, `artifacts/`, `target/`, or root patch transports
- `PACKAGE_MANIFEST.json` covers every governed source file except itself and `Cargo.lock`, matching PCC policy

## Certification status

The **base** source is previously certified GREEN. The reconstructed M1–M6/PCC 0.4.13 changes are intentionally marked **NEEDS FULL GATE** until validated by the user's local Rust 1.95.0 toolchain.

After extraction, run:

`PROJECT_CONTROL_CENTER.cmd`

then choose:

`1. FULL QUALITY GATE / CERTIFY GREEN`

Do not treat the reconstructed source as a new GREEN baseline until that gate passes.
