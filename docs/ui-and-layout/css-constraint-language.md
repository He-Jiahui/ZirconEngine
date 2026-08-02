---
related_code:
  - zircon_editor/src/ui/workbench/autolayout/css_like_constraint.rs
  - zircon_editor/src/ui/workbench/autolayout/mod.rs
  - zircon_runtime_interface/src/ui/design_tokens.rs
  - zircon_runtime_interface/src/ui/layout/style.rs
  - zircon_runtime_interface/src/ui/layout/engine.rs
  - zircon_runtime_interface/src/ui/layout/slot.rs
implementation_files:
  - zircon_editor/src/ui/workbench/autolayout/css_like_constraint.rs
  - zircon_editor/src/ui/workbench/autolayout/css_like_constraint/declaration_parser.rs
plan_sources:
  - docs/plans/zircon_editor/editor_layout/13-taffy-css-constraint-language.md
tests:
  - zircon_editor/src/ui/workbench/autolayout/css_like_constraint.rs
  - zircon_editor/src/ui/workbench/autolayout/css_like_constraint/tests.rs
  - zircon_editor/src/tests/workbench/layout/editor_layout_contracts.rs
doc_type: module-detail
---

# CSS-Like Constraint Language

`CssLikeConstraint` is the editor-authored boundary for slot content layout. Its
`from_declarations` and `apply_declaration` entry points validate an ordered CSS-like declaration
list and produce `zircon_runtime_interface::ui::layout::UiLayoutStyle`; a later declaration wins
for the same property. The runtime style mapper and Taffy bridge remain the only solver path. The retained host's recompute snapshot
builds default drawer and bottom-panel extents through
`WorkbenchSkeleton::default_region_extents_from_tokens`, converts those logical values through
`ResolutionContext`, and lets physical user-resize values override them. General `.zui` template
layout ingestion remains the S2 slot-wiring step; the runtime template builder must not depend on
the editor crate to reach it. The host caches parsed logical defaults against the current installed
V2 token snapshot; a changed token value refreshes the cache, while an unchanged recompute only
performs DPI conversion and transient-resize merging rather than rebuilding token maps. Region
priority is active drag capture, persisted drawer extent, then the token default.

## Values

Dimensions accept `auto`, finite non-negative `Npx`, normalized percentages written as `N%`,
and `$token` references. Percentages are stored as `0.0..=1.0`, so `50%` becomes `0.5` before
the shared DTO is constructed. Chrome dimensions should use tokens or relative values; naked
physical pixels remain limited to center/user content.

Token values resolve through `EditorDesignTokens::density_value_for_token_name`. The short
author aliases `$gap.xs`, `$gap.s`, `$gap.m`, `$gap.l`, `$pad.s`, and `$pad.m` map to canonical
editor density names without creating a second value table. Existing `$--left-drawer-width`,
`$--right-drawer-width`, and `$--bottom-output-height` aliases continue through the central token
registry.

| Author vocabulary | DTO output | Family |
| --- | --- | --- |
| `display:flex`, direction, wrap, grow/shrink/basis, justify/align, gap | flex fields | Flex or Wrap |
| `display:grid`, tracks, placement, size/min/max, margin/padding | grid and box fields | Grid |
| `display:block` with box constraints | box fields | Block |
| overlay/canvas/scroll/virtual | display and parent family | Zircon-owned |

Declaration values use CSS names such as `flex-direction`, `justify-content`, `row-gap`,
`grid-template-columns`, `min-width`, and `overflow-x`. Whitespace-separated edge values follow
the one/two/three/four-value CSS shorthand order; grid track splitting respects nested function
arguments, so `minmax(120px, 1fr) 25% auto` remains three tracks. `grid-row` and `grid-column`
accept `line / line`, `line / span N`, and `auto`; invalid placements fail at the author boundary.

`UiSlotKind` uses the same family choice: Linear and Splitter map to Flex, Grid maps to Grid,
Flow maps to Wrap, and Scrollable maps to Scrollable. Scale maps to Container because scale is a
render transform, not a second layout solver.

## Coverage Matrix

| Tier | Author vocabulary | Contract |
| --- | --- | --- |
| T1 | `display`, flex direction/wrap/alignment/grow/shrink/basis, grid tracks/placement, size/min/max/aspect, margin/padding, position/inset, and overflow | The constraint maps directly to `UiLayoutStyle`; Taffy owns Flex, Grid, Block, and Wrap while the named Zircon family owns overlay, canvas, scrolling, and virtualization. |
| T2 | Percent dimensions, `auto`, `align-content`, grid `fr` and `minmax()`, and `overflow: scroll` | Percent values normalize to `0.0..=1.0`; only size, flex-basis, margin, and inset accept `auto`; `align-content: baseline` and an `fr` minimum in `minmax()` are rejected; scrolling behavior still requires a Scrollable or VirtualizedList slot. |
| T3 | Viewport units, justify-items/self, implicit grid tracks/flow, `fit-content()`, `repeat()`, overflow clip margin, box sizing, RTL layout, object fit, and z-index | Each is reported as a known unsupported property or syntax. Adding one requires a DTO addition, runtime mapper support, and focused tests; it must not create a second editor-side solver. |

## Rejections

`gap` and `padding` reject `auto`; `margin` and `inset` may use it. `align-content: baseline`
is rejected before layout backend selection. A `minmax()` minimum also rejects `fr`, because the
current Taffy bridge would normalize it to `auto`; the author value must never silently lose its
meaning. Values must be finite and non-negative, and percent values cannot exceed one after
normalization.

The parser reports known unsupported diagnostics for T3 extension candidates rather than silently
accepting them: viewport units, justify-items/self, grid auto-flow/implicit tracks,
`fit-content()`/`repeat()`, `overflow:clip`, overflow clip margin, box sizing, RTL layout, object
fit, and z-index. Those candidates require their own DTO, mapper, and test milestone; they do not
add a parallel editor-side layout path.

## Verification

The module-local contracts cover declaration source-order replacement, token resolution,
percentage normalization, Flex/Grid/Block family routing, slot-family mapping, invalid `auto`
and baseline rejection, grid line/span placement, edge shorthand expansion, axis-specific
overflow replacement, immediate invalid pixel rejection, and known T3 property and value
diagnostics. Existing workbench layout contracts cover the shell region extents that consume this boundary. Milestone validation compiles and runs
the scoped `zircon_editor` checks through the coordinator; screenshot capture remains a later
visual integration stage and writes only under `docs/tests/editor`.
