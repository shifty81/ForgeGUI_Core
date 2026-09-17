# ForgeGUI Core — Canvas-First Shell Standard

## Scope: optional authoring profile

This document defines the canvas-heavy authoring profile used by editor/game-tool consumers. It is **not** the universal ForgeGUI application default. The universal shell defaults to generic modular application surfaces and only enables this profile when a consumer opts into a canvas/viewport workflow.


This document is the visual and structural contract for ForgeGUI applications.
ForgeGUI is a reusable GUI core library, not a single editor skin. Consumer projects
start from the same normalized shell and enable only the chrome their workflow needs.

## 1. Ownership model

### Main application shell

The root application owns:

- borderless project-owned title chrome
- project-owned Minimize and Close controls
- File / Edit / View / Help application menus
- optional document tabs
- universal tool rail
- optional contextual right surface
- optional bottom tool tray
- status strip
- command palette and global notifications
- theme and density selection

There is deliberately no project-owned Maximize button. Dock focus/fill and manual
window sizing are the workspace expansion mechanisms.

### Authoring workspaces

An authoring workspace may consume canvas-specific chrome when it benefits the tool:

- internal canvas frame
- top and left rulers
- guides and snap indicators
- text-only layer visibility bar
- canvas HUD
- viewport status and orientation widgets
- transform/manipulation tools
- optional minimap/navigator

An authoring tool must not automatically recreate the entire application shell inside
its panel. Chrome is inherited by responsibility.

## 2. Canvas priority

The center canvas is the primary visual surface. Default ForgeGUI layouts must avoid
permanent stacked toolbars that reduce the canvas without a clear workflow need.

Default canonical Lab composition:

1. project-owned title strip
2. File / Edit / View / Help strip
3. slim universal tool rail at the left
4. large center infinite canvas
5. contextual right surface
6. bottom status strip

Document tabs and the bottom tool tray are opt-in/toggleable. The left rail and right
context surface support pinned and edge-reveal use.

## 3. Infinite-canvas chrome

The infinite canvas owns its own frame widgets rather than using a separate panel:

- subtle inset frame border
- top ruler
- left ruler
- ruler origin corner
- major/minor ticks
- coordinate labels
- text-only layer strip in the upper-left content region
- lightweight upper-right camera/render HUD
- lightweight lower-left selection/status HUD

The layer strip is intentionally not a panel. Each layer is rendered as text with a
visibility marker and hover target. Clicking a layer toggles its visibility and makes it
the active layer.

## 4. Universal tool rail

Every ForgeGUI authoring product can expose one normalized left rail. Tool definitions
are grouped into sections instead of being scattered among unrelated bars.

Recommended groups:

- Transform: Select, Move, Rotate, Scale, Pan, Zoom
- Workspace: Inspector, Assets, Outliner, Widget Gallery
- Authoring: tool-specific modes supplied by the consumer
- Runtime: Play, Pause, Stop, Step

The rail supports:

- compact icon mode
- tooltip labels
- selected state
- badges and severity state
- pinned mode
- collapsed mode where applicable
- edge-reveal/auto-hide mode
- consumer-defined sections

## 5. Context surfaces

Right-side surfaces are contextual and collapsible. The default Lab demonstrates a
single normalized host with Inspector, Assets, and Widgets tabs rather than several
simultaneous permanent panels.

Consumers may add specialized context tabs such as:

- scene hierarchy
- animation properties
- node properties
- tile properties
- audio properties
- build settings
- source-control state

The surface can be pinned or hidden. When unpinned it may reveal from the right screen
edge without changing the permanent layout contract.

## 6. Widget families

ForgeGUI should provide project-owned styling and normalized adapters for the full
application-control vocabulary. Major families are:

### Navigation

Menus, menu items, context menus, command palette, tabs, breadcrumbs, rails, tree
navigation, pagination, steppers, history, quick switchers, split buttons, overflow.

### Input

Buttons, icon buttons, toggles, checkboxes, radio controls, segmented controls, text
fields, search, password, numeric editors, sliders, range sliders, scrubbers, combo
boxes, asset/file/folder pickers, color and gradient controls, keybind capture.

### Data display

Labels, badges, chips, status pills, progress bars, radial gauges, meters, sparklines,
health/load/capacity indicators, tables, trees, lists, thumbnail grids, property rows.

### Containers

Panels, cards, frames, group boxes, split views, dock stacks, collapsible sections,
scroll areas, drawers, bottom trays, popovers, floating palettes, auto-hide surfaces.

### Feedback

Tooltips, banners, toasts, inline validation, error/warning/success states, busy states,
skeletons, operation progress, confirmation dialogs, modal and non-modal overlays.

### Authoring

Infinite canvases, rulers, guides, grids, snapping, selection/lasso, transform gizmos,
minimaps, node/graph canvas, timeline, dope sheet, curve editor, track headers, scrubber,
viewport overlays, docking guides, drag/drop targets.

### System/project operations

Console, problems, build/gate progress, task queues, patch intake, artifact cards,
source-control status, branches, health, diagnostics, notifications, Cortex/AI surfaces.

## 7. Theme contract

ForgeGUI themes are semantic token sets. Consumer code must use semantic colors and
metrics rather than hard-coded local colors wherever practical.

Built-in baseline presets:

- Forge Dark
- Midnight Mint
- Graphite
- Warm Ember
- High Contrast

Themes govern:

- shell and panel surfaces
- canvas background
- borders and focus state
- text and muted text
- accent/success/warning/danger
- button states
- scrollbar dimensions and visibility behavior
- progress/gauge state
- density metrics

Projects can define derived themes without replacing the control implementation.

## 8. Scroll contract

ForgeGUI supports both toolkit ScrollArea styling and a project-owned scrollbar
primitive for custom/virtualized surfaces. Scroll chrome must stay dark-theme aware,
compact, discoverable on hover, and consistent with the selected theme.

## 9. Library rule

The canonical Lab is a certification host. It must demonstrate the reusable pieces,
but no widget may exist only as Lab-specific drawing when it belongs in the shared
library. Consumer applications should be able to assemble a polished starting GUI by
combining ForgeGUI crates and then specialize that shell for their own workflows.
