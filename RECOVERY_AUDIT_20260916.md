# ForgeGUI_Core Recovery Audit — 2026-09-16

## Inputs audited

The reconstruction used all supplied material for provenance, but not every archive was copied into the clean source:

- `crates.zip` — authoritative full working tree. Embedded Git status was clean on `main` at `532f7e1a0dfe4d7eefc46680a1567242059640a7`.
- `certification.zip` — certification/history evidence. It contains the 2026-09-15 18:38 Full Gate PASS for the same source fingerprint later reported by the debug bundle.
- `ForgeGUI_DebugBundle_DBG-20260916-003146-PATCH_FAIL-0a788043.zip` — confirms the live tree was still clean at `532f7e1` and that M1–M6 had not landed; failures were patch-intake/path/order failures.
- `000_ForgeGUI_Core_M1-M6_MODULAR_SHELL_WINDOWING_POLISH_REBASED01_20260915.zip` — patch-history bundle containing multiple superseded M1–M6 variants. It was treated as history only, not copied into the clean source.
- latest live-tree REBASED05 delta — applied directly to the recovered green source so the clean rollup contains source changes rather than pending update transports.

## Authority decisions

1. `crates.zip` wins for source because it is a complete Git worktree and matches the latest debug bundle's clean commit.
2. `certification.zip` wins for historical GREEN evidence only; it is not source.
3. Failed and superseded root patch transports are excluded.
4. The live-tree M1–M6/PCC delta is applied directly because `git apply --check --whitespace=error-all` succeeded against the exact recovered green worktree.
5. `.git`, `artifacts`, `target`, debug bundles, transaction state, and patch queues are deliberately omitted from the clean handoff.

## Material findings

- The latest live tree before reconstruction was clean and unchanged despite repeated patch failures.
- The prior authoritative Full Gate was GREEN before M1–M6.
- The repeated M1–M6 failures were caused by stale/nonexistent PCC documentation paths and old ordered intake selecting superseded transports.
- The actual project-owned PCC implementation is `tools/pcc/ForgeGuiControl.ps1`.
- The repository already contains `docs/INTERNAL_PCC_STANDARD.md`; the failed patches incorrectly targeted `tools/pcc/INTERNAL_PCC_STANDARD.md` and later `pcc/INTERNAL_PCC_STANDARD.md`.
- CI still referenced two retired packages (`forge_gui_next_lab`, `forge_authoring_lab`) even though the current workspace contains only the canonical Lab plus Contracts Smoke, Micro Lab, and App Lab.
- `forge.gui.toml` still declared a `game_maker` default even after the architecture shifted to a generic modular application shell.
- Several documents still described Canvas-first behavior as the universal default. They are now explicitly scoped as an optional authoring profile.
- PCC provider 0.4.13 now specifies automatic main-menu return, source-authoritative Lab launch, ordered patch queues, ordinary in-process patch application, and same-console provider reload.

## Result

The accompanying clean source ZIP is a patch-free reconstructed source tree ready for one authoritative local Full Gate. It intentionally does not claim a new GREEN certification until that gate succeeds.
