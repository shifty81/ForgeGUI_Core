# ForgeGUI Cross-Project Audit Findings

ForgeGUI is being normalized as a shared UI platform rather than a project-local docking demo. Forge/Cortex, Ember, Codename Subspace, Windstead, Havenwild, and future tools need a common shell, interaction language, panel taxonomy, diagnostics, and adapter boundary while retaining project-owned domain models.

## Recovery finding

The early 0.4.x full-source reconstruction was functionally smaller than the richer 0.3.8 reference project. The 0.4.8 recovery baseline preserves that broader Lab/contracts surface while retaining the hardened PCC, semantic icons, virtual tables, and OSS evaluation work from 0.4.x.

## Structural shell finding

`PreferredDock` cannot merely be metadata and should not be implemented as repeated nested dock splits. The recovery runtime therefore owns structural left/right/bottom rail tab groups and reserves free `egui_dock` composition for the center workspace. This is the baseline for Forge/Cortex permanent activity rails and Ember editor Studios.

## Canvas finding

The recovered Canvas must track the pinned egui input contract. For egui 0.36.2, wheel/trackpad zoom reads `InputState::smooth_scroll_delta()`; the removed `raw_scroll_delta` field is prohibited by regression scan.

Canvas pan must use per-frame pointer delta rather than cumulative drag distance. Wheel zoom must be hover-gated and centered on the pointer. Alt+primary and middle-button panning are both exercised in the reference Lab.

## Fail-closed finding

Duplicate catalog registration must be rejected before mutation. Patch intake must also be ordered: one patch per PCC session, stop on decline/failure, and restart after success before examining another transport.

## PCC native-process finding

Windows PowerShell 5.1 can represent native stderr as error records even for successful Cargo progress. ForgeGUI therefore treats stdout/stderr as diagnostic streams and uses native exit code as the sole PASS/FAIL authority.

## Certification finding

Full Gate verifies complete `PACKAGE_MANIFEST.json` coverage/hashes and is check-only after one manifest-authorized bootstrap rustfmt canonicalization. GREEN is an authoritative source fingerprint; commit/push refuses stale source.

## Remaining 1.0 work

The recovery baseline does not claim the full future system. Runtime application of `InterfacePreset`, floating-window authority, a real terminal process host, RenderSurfaceHost, schema-driven data editors, generalized service/context buses, richer generic panel templates, accessibility/DPI/localization, and full visual regression testing remain subsequent passes.
