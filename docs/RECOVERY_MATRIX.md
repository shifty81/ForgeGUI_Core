# ForgeGUI 0.4.6 Recovery Matrix

0.4.6 is the recovery/hardening baseline after the early 0.4.x reconstruction compressed the richer 0.3.8 reference surface.

## Recovered reference surfaces

Project Explorer, Asset Browser, Scene/Room Canvas, Inspector, Layers, Console/Cortex, Notification Center, Problems, Timeline, Visual Graph, History, Curve Editor, Settings, command palette, menu/tool rails, status rail, finite/infinite canvas controls, minimap proof, PIE status, Play/From Here/Pause/Step/Restart/Stop controls, runtime input ownership, WASD runtime movement proof, Alt/middle pan, and cursor-oriented hover zoom are exercised by the Lab.

## Structural normalization

Left/right/bottom are ForgeGUI-owned structural rail tab groups. Only the center workspace is freely docked through the current `egui_dock` adapter. This prevents the reference host from degenerating back into an all-detached/nested-panel shell while keeping the center replaceable for future `egui_tiles` or other layout engines.

## Retained 0.4.x improvements

Exit-code-authoritative PCC native execution, single-ZIP handoff, Explorer handoff opening, GREEN fingerprint protection, semantic `IconId`, Phosphor adapter, virtual tables, OSS evaluation lane, controlled Cargo.lock repair, manifest-aware root hygiene, and egui 0.36/egui_dock 0.21 API corrections remain.

## New hardening

Duplicate panel/command registrations are fail-closed; patch intake is ordered one-transport-per-session and requires PCC restart after apply; Full Gate is check-only after one manifest-authorized recovery-package rustfmt canonicalization; package-manifest coverage is complete/fail-closed; machine-readable provider queries are restored; and OSS candidate APIs are exercised by smoke tests rather than metadata only.

## Explicitly not yet complete

Runtime `InterfacePreset` application/persistence, true floating windows, a terminal process host, RenderSurfaceHost, schema-generated data editors, generalized `GuiServices`, and production accessibility/visual-regression certification remain post-recovery roadmap work.
