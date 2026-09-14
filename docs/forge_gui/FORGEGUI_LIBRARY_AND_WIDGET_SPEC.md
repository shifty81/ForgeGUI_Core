# ForgeGUI_Core Library and Widget System

Status: canonical independent-library design baseline

## Role and boundary

ForgeGUI_Core is an independently maintained, reusable GUI, authoring, scene, render,
and runtime foundation. It may be consumed by Ember, Forge, Cortex, hosted game editors,
standalone utilities, or future applications without requiring those consumers to share
one monolithic application.

The repository is broader than a widget skin:

- **ForgeGUI** owns reusable shell behavior, panels, rails, tabs, docking, widgets,
  browser primitives, inspectors, overlays, command UI, status UI, themes, input,
  accessibility, layout persistence, and GUI certification.
- **ForgeAuthoring** owns reusable authoring sessions and the universal 2D/3D/Hybrid
  authoring-surface contracts.
- **ForgeScene** owns the shared scene/entity/component/hierarchy representation used
  by both authoring and runtime.
- **ForgeRender** owns renderer-neutral 2D/3D/Hybrid render, camera, and picking contracts.
- **ForgeRuntime** owns GUI-independent runtime lifecycle, fixed-step execution, PIE,
  simulation, standalone, detached, and headless hosting contracts.

Applications own project-specific meaning. ForgeGUI_Core must not hard-code crops,
spaceships, quests, world-generation rules, or other consumer-domain semantics.

## Core panel/tab-rail system

ForgeGUI_Core supports multiple interchangeable tab-rail presentations through one
semantic model. Visual treatment is configuration, not a different navigation subsystem.

### Rail variants

1. **Slim Icon Rail**
   - Narrow edge rail.
   - Icon-first tabs.
   - Tooltip and optional badge/indicator.
   - Accent edge marker plus selected surface.
   - Ideal for global activity/navigation rails.

2. **Labeled Tool Rail**
   - Wider rail with icon + text.
   - Grouped sections and separators.
   - Large click targets.
   - Ideal for asset/tool/category navigation.

3. **Dual Rail / Context Rail**
   - Slim primary activity rail plus contextual secondary rail.
   - Primary selection changes the secondary tool set.
   - Either rail can collapse independently.
   - Ideal for complex authoring surfaces.

4. **Pill Tabs / Floating Rail**
   - Rounded tactile card/pill tabs.
   - Selected tab visually connects to the content surface.
   - Ideal for focused property/object tools and compact floating surfaces.

### Orientation and placement

Every rail variant should support left, right, top, and bottom orientation; flush,
inside-edge, inset, and floating placement; compact/normal/comfortable density;
fixed/content-sized/user-resizable sizing; and per-workspace persistence.

### Common behavior

Stable IDs, semantic icons, labels, tooltips, active/hover/focus/pressed/disabled
states, optional close buttons, pin/lock state, notification counts, dirty markers,
warning/error/success indicators, drag reorder, drag-out docking where permitted,
overflow menus, keyboard navigation, context menus, grouped sections, remembered
active tab, and shared motion tokens.

## Dock/lock policy

Appearance and placement policy are independent:

- `Structural`: permanent shell region outside the free docking tree.
- `HardLocked`: fixed to an assigned region.
- `SoftLocked`: fixed by default with an explicit unlock affordance.
- `Anchored`: starts in a preferred region but can move.
- `Free`: fully dockable/floating.

A lock state never dims the entire panel.

## Universal workbench structure

ForgeGUI_Core should be able to compose:

- application chrome/title surface
- main menu + command/search surface
- primary action toolbar
- global activity rail
- left contextual tool region
- central document/authoring workbench
- right contextual Inspector/tool region
- bottom console/Cortex/problems/build/timeline/Git region
- status bar
- overlay host

Only genuinely flexible documents/tools belong in the free docking tree.

## Universal authoring surface

The canonical authoring surface supports:

- 2D, 3D, and Hybrid modes
- infinite pan/zoom and 3D orbit/fly/navigation
- orthographic and perspective cameras
- hierarchy-aware selection and picking
- box/lasso selection
- move/rotate/scale gizmos
- pivots and coordinate spaces
- grid, rulers, guides, and snapping
- layers, visibility, and lock state
- undo/redo command transactions
- runtime preview and Play From Here
- application-provided tools and render backends

The same shared `ForgeScene` model must flow from authoring through PIE/runtime and
packaging. Editor-only metadata is layered around the scene rather than requiring a
separate runtime scene format.

## Visual system

Use semantic surfaces instead of ad-hoc colors:

- `Sunken`
- `Base`
- `Raised1`
- `Raised2`
- `Raised3`
- `Overlay`
- `Focus`

Each resolves shared tokens for fill, border, separator, shadow, radius, text,
muted text, accent, hover, active, selection, focus, warning, error, and success.

## Scrollbar + document map

ForgeGUI_Core provides themed scrollbars plus an optional IDE-style document map
for search matches, warnings/errors, bookmarks, console events, timeline density,
and other document markers.

## Certification

ForgeGUI Lab and ForgeAuthoring Lab are independent certification applications.
They are not product-specific editors. Consumer applications run their own integration
gates against a known-GREEN ForgeGUI_Core version.
