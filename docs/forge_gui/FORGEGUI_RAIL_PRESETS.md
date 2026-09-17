# ForgeGUI_Core Rail Presets

The four approved rail concepts are first-class presentation presets over one shared
semantic rail model.

## `RailPreset::SlimIcon`

Compact global/activity navigation with icon-first tabs, consistent hit targets,
tooltips, optional badges, and an accent edge marker.

## `RailPreset::LabeledTool`

Discoverable tool/category navigation with icons + labels, section headers,
larger selected surfaces, optional counts, keyboard navigation, and resizable width.

## `RailPreset::DualContext`

A primary mode/domain rail plus a secondary contextual rail. Either rail can collapse
independently and both retain state per workspace.

## `RailPreset::FloatingPill`

Rounded card/pill tabs inset from the panel edge. The active tab visually connects to
the content surface while inactive tabs remain individually tactile.

## Shared model

All presets share stable IDs, semantic commands/state, orientation, placement, density,
badges, dirty/warning/error state, keyboard navigation, drag/reorder, overflow,
persistence, and lock policy.

They are rendering/configuration variants, not four separate navigation systems.
