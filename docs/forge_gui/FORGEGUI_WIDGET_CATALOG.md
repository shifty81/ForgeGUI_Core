# ForgeGUI_Core Widget Catalog

This catalog defines the reusable GUI surface available to any ForgeGUI_Core consumer.
The optional canvas-authoring shell contract is documented in `FORGEGUI_CANVAS_FIRST_SHELL_STANDARD.md`; it is a consumer profile, not the universal ForgeGUI default.

## Current certified baseline

The canonical Lab now directly certifies project-owned title/menu chrome, theme presets,
a grouped universal tool rail, edge-reveal side surfaces, optional document/bottom trays,
status chrome, infinite-canvas frame/rulers/text-layer HUD, Inspector/Asset context hosting,
command palette, progress bars, radial gauges, themed native scroll areas, and a
ForgeGUI-owned scrollbar primitive. Remaining entries below are the normalized target
widget vocabulary and should be implemented behind shared ForgeGUI adapters rather than
as consumer-specific one-offs.

## Navigation and commands

Command palette/omnibox, quick document/asset switcher, breadcrumbs, back/forward/up
history, activity rails, configurable tab rails, document tabs, segmented navigation,
recent/history popup, searchable action menu, keyboard shortcut overlay.

## Buttons and compact controls

Primary/secondary/destructive/subtle buttons, icon buttons, split/menu/toolbar buttons,
toggle buttons, segmented buttons, chips/tags/filter buttons, disclosure controls,
repeat/hold buttons.

## Value editors

Text/search/password fields, multiline editor, numeric scrub/stepper, checkbox, radio,
switch, slider/range slider, Vector2/3/4, angle/rotation, knob/dial, combo/select,
editable combo, enum picker, asset/file/folder picker, color picker, gradient editor,
date/time, keybinding recorder, tags, and path fields.

## Inspector/property system

Property grid, collapsible groups, multi-select mixed values, reset/revert,
provenance/source indicator, validation messages, read-only/computed state,
advanced-property disclosure, units/ranges, bindings, and asset/object references.

## Browsers, trees, lists, and tables

Virtualized tree/list/table, thumbnail grid, tree-table, sortable/resizable/reorderable
columns, saved searches, favorites/recent, filters/tags, multi-selection, inline rename,
drag/drop, context menus, source-control/status/validation overlays, empty/loading/error
states, skeleton rows, and async thumbnails.

## Panel/layout surfaces

Raised/sunken/flat panels, cards, split panes, dock tab stacks, collapsible sidebars,
accordions, scroll containers, auto-hide edge panels, slide-out drawers, bottom trays,
focus/fill document mode, floating tool windows, and contextual rails.

## Overlays

Rich tooltips, context/popup menus, popovers, command palette, modal/non-modal dialogs,
sheets, drawers, toasts, banners, progress overlays, confirmation panels, drag previews,
drop targets, and docking guides.

## Status and telemetry

Status pills, severity badges, linear/radial progress, meters/gauges, sparklines,
consumer-supplied telemetry, connection/sync indicators, operation queues, gate/build
stages, and notification badges.

## Universal authoring/canvas controls

Infinite 2D/3D/Hybrid authoring surface, pan/zoom/orbit navigation, minimap/navigator,
rulers, guides, snapping HUD, grids, selection rectangle/lasso, transform gizmos,
viewport toolbar, breadcrumbs, zoom presets, focus-selection, axes/orientation,
overlay legend, and viewport status HUD.

## Timeline / animation / graph

Timeline ruler, tracks, frame/cel strips, keyframes, dope sheet, event markers,
playhead/scrubber, transport controls, curve editor, graph canvas, node cards,
ports/sockets, wires, graph minimap, selection box, and subgraph breadcrumbs.

## Console / build / problems

Streaming console, severity/category filters, search, structured operation groups,
Problems list, build stages, clickable file references, copy/export, follow-tail,
and command input when the consumer exposes execution.

## Project operations

Drag/drop intake, intake queue, compatibility/review cards, before/after comparison,
approval bar, operation receipts, artifact cards, project health, and source-control
branch/status widgets.

## High-value polish

Dock compass, tab preview thumbnails, peek panel, command chips, inline diff,
palette/history strip, smart empty states, breadcrumb overflow, sticky headers,
splitter-hover feedback, drag ghost/target preview, pinned popovers, Focus Mode,
panel quick-actions, workspace presets, search-highlight lane, inline undo feedback,
contextual footer, command-progress pill, notification inbox, and keyboard-first
discoverability.
