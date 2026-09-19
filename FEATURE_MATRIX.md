# Feature Matrix

| Area | 0.4.8 status | Notes |
|---|---|---|
| Renderer-neutral panel/core contracts | Ready | Namespaced IDs, roles, scope, instance policies, capabilities |
| Universal modular surface system | Foundation ready | Independent left/center/right/bottom/floating surfaces, tab stacks, hide/show, docking and lock-in-place controls; Canvas-specific rail remains optional |
| Project-owned application chrome | Ready foundation | Borderless title surface, Minimize/Maximize-Restore/Close, double-click behavior, edge-snap/maximize fallback, distinct File/Edit/View/Settings/Help strip and manual resize handles |
| Structural panel widgets | Foundation ready | Shared panel frame/header/action primitives added |
| Workspace layout schema | Foundation ready | `forge.workspace_layout.v1` plus Creator/World/Animation/Scripting presets |
| Optional renderer workspace | Ready foundation | Canvas / Viewport is an opt-in workspace; generic application content is the default Lab identity |
| Universal renderer families | Foundation ready | 2D / 2.5D / 3D / Voxel / Hybrid authoring profiles |
| GPU surface presentation | Next | Renderer output still needs backend-specific texture/swapchain presentation bridge |
| Modular docking/floating | Foundation ready | Generic surfaces move between left/center/right/bottom/floating hosts and may be locked; direct drag-drop dock compass remains a later depth pass |
| Semantic icons | Ready | `IconId` -> Phosphor adapter |
| Core chrome widgets | Ready foundation | Compact/labeled controls, badges, progress, radial gauge, themed native scroll areas, owned scrollbar, segmented choice |
| Virtual data table | Ready | `egui_extras` virtualized rows |
| 2D canvas contracts | Ready | finite/infinite, pan/zoom/grid/snap/minimap proof |
| Console / Cortex data model | Ready | output/cortex/terminal/activity channel kinds; no terminal process host yet |
| Terminal process host | Planned | capability is intentionally false in `forge.gui.toml` |
| Notifications | Ready | durable notification center model |
| PIE contracts | Ready | play/from-here/pause/step/restart/stop reference proof |
| Context Inspector/Assets/Widgets host | Ready foundation | One collapsible/pinnable context surface replaces permanently competing side panels |
| Timeline / Curve / Graph surfaces | Reference proof | shared contracts/templates still to deepen |
| OSS tree/layout candidates | Evaluation | compile/smoke certified, not public durable contracts |
| Interaction testing | Foundation | `egui_kittest` smoke lane established |
| Schema-driven data editors | Planned | intended for game/project data tooling |
| Accessibility/DPI/localization hardening | Planned | Creator Studio certification lane |
| PCC native runner | Ready | PowerShell 5.1 stderr-safe, exit-code authoritative |
| Automatic handoff ZIP | Ready | failure/manual + PASS checkpoint bundles |
| Transactional patch intake | Ready | one per session, backup/receipt/rollback |
| GREEN commit/push authority | Ready | exact certified source fingerprint required |


## FG-C56..FG-C70 — Canonical Creator Studio / chrome normalization

- C56 canonical `forge_gui_lab` promoted to the Creator Studio shell.
- C57 ForgeGUI facade now exposes browser / inspector / command creator services.
- C58 compact Creator density and chrome metrics.
- C59 flush structural panel frame and header treatment.
- C60 reusable panel tab strips.
- C61 Asset Browser search / favorites / tree-row visual pass.
- C62 Inspector object header and compact property-grid pass.
- C63 bottom tool tray tab normalization and reduced default tray height.
- C64 menu / product / action chrome normalization.
- C65 compact Creator widget family and semantic tabs.
- C66 workspace document header and local toolbar normalization.
- C67 renderer-family selector moved into workspace-local chrome.
- C68 renderer surface remains the authoritative center view; null presenter remains Lab-only.
- C69 Creator Studio reference preview and selection chrome tightened.
- C70 canonical Lab parity: PCC `run.lab` now reaches the Creator Studio because `forge_gui_lab` itself is the promoted target.


## FG-C71..FG-C75 — Project-owned shell normalization

- C71 native OS title-bar decorations removed from the canonical GUI Lab.
- C72 ForgeGUI-owned draggable title surface established; current shell normalizes it to Minimize / Maximize-Restore / Close with double-click maximize/restore.
- C73 project-owned edge/corner resize handles preserve manual desktop sizing without native chrome.
- C74 duplicate `forge_gui_next_lab` and `forge_authoring_lab` executables retired; `forge_gui_lab` is the one visible Lab.
- C75 reference concept images remain the visual-direction authority; dock/workbench focus-fill remains available as a workspace behavior while the desktop shell now also supports native-style maximize/restore.

## FG-C76..FG-C82 — Canvas-first GUI core normalization

- C76 canonical Lab reduced to project-owned title + File/Edit/View/Help + status as permanent app bars.
- C77 universal left tool rail grouped by Transform / Workspace / Runtime.
- C78 left rail and right context surface support pinned or hover-edge reveal behavior.
- C79 renderer workspace owns the internal frame, rulers and lightweight HUD; current normalization reduces the frame/ruler footprint and hides rulers where 3D/Voxel profiles do not need them.
- C80 layer visibility remains built into the workspace rather than a permanent panel; current normalization presents it as a compact vertical layer stack.
- C81 built-in Forge Dark / Midnight Mint / Graphite / Warm Ember / High Contrast presets plus themed ScrollArea chrome.
- C82 reusable progress bar, radial gauge, owned scrollbar and segmented-choice primitives plus canonical widget gallery.


## M1–M6 — Universal modular shell normalization

- **M1 PCC continuity:** ordered patch queue continues after ordinary updates; PCC-source updates auto-relaunch the provider instead of asking for a manual restart.
- **M2 window behavior:** borderless title dragging retains native drag commands and adds ForgeGUI top/left/right edge maximize/snap fallback.
- **M3 modular surfaces:** generic independent surfaces can live left, center, right, bottom or floating; tab stacks, visibility, hide, lock and dock controls are first-class.
- **M4 tabs/toolbars:** workspace tabs are visible by default and toolbars can dock to any edge, float, hide and lock.
- **M5 shell profiles:** Minimal, Content First, Standard, Workspace and Kiosk profiles control global chrome; the canonical Lab defaults to generic Application/Dashboard/Canvas workspaces.
- **M6 interaction polish:** hover, pressed, selected and focus behavior plus surface/control rounding are semantic theme settings, exposed live from Settings.

| PCC menu continuity | Candidate M1-M6+ | All interactive actions return to main menu; normal patch apply stays in-process; provider changes self-reload |
| Drag docking certification | Candidate M1-M6+ | Active surface and toolbar grips provide live dock previews and drag re-docking in the canonical Lab |

## M8 universal consumer runtime

| Capability | Public framework status | Certification |
|---|---|---|
| Reusable application shell | Implemented in `forge_gui_shell` | `forge_gui_consumer_starter` |
| Surface registration | Implemented | unit tests + starter |
| Left/center/right/bottom docking | Implemented | starter |
| Floating surfaces | Implemented | starter |
| Surface tab stacks | Implemented | starter |
| Surface lock/hide/show | Implemented | unit tests + starter |
| Surface drag/drop dock preview | Implemented | starter visual acceptance |
| Toolbar placement/locking | Implemented | starter visual acceptance |
| JSON shell layout persistence | Implemented | unit test |
| ForgeGUI Lab | Showcase only | not framework authority |

## FG-C71R source-aware correction (requires local Full Gate)
Shared `forge_gui_egui` docking is unified; the newer native-window GUI Lab is preserved and persists modular surface placement through eframe storage. The Lab currently has a separate `ModularSurfaceState` layout authority; merging both systems under one authoritative docking interface is still required before claiming universal docking completion. Semantic browser and panel icons are improved. CI and icon-font initialization already existed in this source and are not overwritten.

## FG-C74 measured docking candidate (Windows Full Gate pending)

| Capability | Source implementation | Still needed |
|---|---|---|
| Main dock-drop hit testing | Actual visible panel rectangles with center fallback only inside shell | Full nested split tree |
| Main dock-drop preview | Per-panel tab target and corrected label | Split-edge previews once split operations exist |
| Floating tab selection/order | Separate persisted active-tab map and stable ordered IDs | Multi-monitor/native drag certification |
| FG-C72 resize and FG-C73 frameless hosts | Preserved in cumulative source | Full Windows interaction gate |

## FG-C75 nested docking model (Windows Full Gate pending)

| Capability | Source status | Remaining |
|---|---|---|
| Reusable target-relative nested split/tab model | Versioned `forge_gui_chrome::docking::ModularDockTree`, atomic move/attach/detach and reconciliation implemented | Connect actual Lab rendering and persisted split ratios |
| Floating self-target protection | Live Lab filters same-host/hidden/locked sources; release clears stale origin drag | Certify native cross-viewport transfer events on Windows |
| Split drop geometry | Leaf-specific target zone and preview helpers, with unit tests | Show zones only after real nested split renderer is wired |
| Release certification | Patch and archive checks only | Run PCC Full Gate, GUI drag and multi-monitor tests |

## FG-C76 live GUI Lab dock-tree candidate (Windows Full Gate pending)

| Capability | C76 implementation | Outstanding certification |
|---|---|---|
| Main GUI Lab layout | Replaced four fixed dock regions with recursive `ModularDockTree` rendering | Windows visual interaction and mixed-DPI |
| Nested splits | Recursive horizontal/vertical layout and draggable ratio handles | Manual resizing, edge cases and persistence test |
| Tab and split drop | Measured leaf-local center/edge drop zone, model-backed commit, live preview | Native pointer crossing between OS windows |
| Persistence | Versioned `forgegui.core.dock_tree.v1` restored/reconciled with catalog | Local restart and corrupted saved layout smoke test |
| Floating host | C73–C75 native frameless grouping retained; shell tree membership updated | Recursive splits within floating hosts still absent |
| Shared consumer shell | Independent runtime remains, no migration claimed | Adapter unification and external consumer certification |

## FG-C77 native-host tree candidate (Windows Full Gate pending)

| Capability | Source candidate | Unverified |
|---|---|---|
| Floating-host layouts | Independent versioned `ModularDockTree` per native host, saved and reconciled | Windows restart, DPI and multiple monitors |
| Recursive floating panels | Common shell renderer supports floating tab leaves and draggable nested split dividers | Live native resize and all window-manager behaviors |
| Floating drag/drop | Per-leaf center/edge previews, candidate-first transaction and old-host detach | End-to-end cross-OS-window pointer routing |
| Consumer adapter | Unchanged; Lab-only migration | Migrate `forge_gui_egui` to the same layout model |

## FG-C78 consumer docking convergence — candidate, NOT certified

| Area | Source state | Certification still required |
|---|---|---|
| `forge_gui_egui` canonical layout | Main and floating use shared `ModularDockTree`; `DockState` source removed | Windows `cargo check`, tests, Clippy |
| Consumer view adapter | Recursive tab/split rendering and leaf-local drop; in-app floating windows | Actual GUI pointer and resize tests |
| Instance ownership | Stable encoded IDs, candidate-first host transfer, hide/reopen, validated saved-surface restore | Cross-host and multi-instance live tests |
| Public consumer API | `dock_tree` / `restore_layout` / `dock_relative` | Migrate old `.dock_state()` callers; independent consumer build |
| Native OS float | Existing Lab native hosting preserved, consumer only in-app egui hosts | Explicit native bridge and mixed-DPI |
| Dependency pruning | Old `egui_dock` declaration remains unused pending lock regeneration | Windows Cargo lock validation |
