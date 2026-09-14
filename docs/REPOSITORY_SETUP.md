# ForgeGUI_Core Repository Setup

## Canonical target

`https://github.com/shifty81/ForgeGUI_Core.git`

At clean-source packaging time, the connected GitHub installation did not expose a repository named `shifty81/ForgeGUI_Core`. Create the empty GitHub repository if it does not yet exist.

## Clean local reset

1. Move/archive the old local ForgeGUI folder.
2. Extract the clean source ZIP into a fresh folder named `ForgeGUI_Core`.
3. Do not copy the old `.git`, `target`, `artifacts`, patch transactions, or debug bundles into the new folder.
4. Run `SETUP_REPOSITORY.cmd`.
5. Run `PROJECT_CONTROL_CENTER.cmd`.
6. Select **1. FULL QUALITY GATE / CERTIFY GREEN**.
7. Resolve any real Rust/test/Clippy failure until GREEN.
8. Select **2. COMMIT + PUSH CURRENT GREEN**.

The PCC initializes `main`, configures Git identity if missing, and defaults the remote to the canonical ForgeGUI_Core URL.

## Repository policy

- `main` is the certified integration branch.
- New development should normally occur on feature/update branches.
- Consumers should pin ForgeGUI_Core to a known-GREEN tag or commit.
- `target/` and `artifacts/` are local/generated and never committed.
- `Cargo.lock` is committed.
- root-drop patch ZIPs are transport only and are ignored by Git.
- `PACKAGE_MANIFEST.json` governs clean source payload bytes; it is self-excluded.
- a GREEN receipt is local certification evidence under `artifacts/` and is not repository source.

## Suggested initial GitHub settings

Once the repository exists:

- default branch: `main`
- require the ForgeGUI_Core CI workflow before merging when branch rules are enabled
- use pull requests for non-trivial library/API changes
- tag known-GREEN releases (`v0.4.8-core-reset`, later normal semver tags)
- consumers pin tags/commits rather than floating branch heads
