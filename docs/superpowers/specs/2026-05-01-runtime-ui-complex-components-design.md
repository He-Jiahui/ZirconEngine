# Runtime UI Complex Components Design

## Summary

This spec defines the next Runtime UI component-library slice for `VirtualList`, `PagedList`, and `WorldSpaceSurface`. The current code already exposes descriptors, retained props/states, and editor host projection metadata for these components. This slice turns those declarations into explicit Runtime UI component behavior: typed events, reducer invariants, boundary validation, and focused projection evidence.

The owner boundary stays in `zircon_runtime::ui::component`. `zircon_editor` may project and display the resulting state, but it must not become the authority for virtual-window, page, or world-space semantics.

## Goals

- Add explicit component event semantics for complex-list and world-space surfaces instead of relying only on generic string property updates.
- Keep large-list behavior in retained Runtime UI state: visible window, total count, item extent, overscan, and visible item slicing must have deterministic boundaries.
- Keep paged-list behavior in retained Runtime UI state: page index, page size, page count, total count, and item/page values must stay consistent and clamped.
- Keep world-space behavior as metadata and retained state only. Do not wire it into graphics/RHI in this slice.
- Preserve existing editor host projection fields while moving validation and normalization rules to the runtime component layer where practical.

## Non-Goals

- Do not touch graphics/GI/VG/plugin code or real world-space rendering.
- Do not touch `zircon_editor/src/ui/host/asset_editor_sessions/**` watcher/session ownership.
- Do not redesign the descriptor registry, M13 palette work, or UI asset invalidation plan.
- Do not introduce editor-only shortcuts that make `VirtualList`, `PagedList`, or `WorldSpaceSurface` behave differently from runtime component state.

## Architecture

`zircon_runtime::ui::component` remains the canonical layer for component declarations and retained events. The new behavior should extend the existing `UiComponentEventKind`, `UiComponentEvent`, and `UiComponentState::apply_event(...)` reducer instead of adding a separate complex-component dispatcher.

The intended event additions are:

- `SetVisibleRange { start, count }` for `VirtualList`.
- `SetPage { page_index, page_size }` for `PagedList`.
- `SetWorldTransform { position, rotation, scale }` and `SetWorldSurface { size, pixels_per_meter, billboard, depth_test, render_order, camera_target }` for `WorldSpaceSurface`.

If implementation evidence shows fewer event variants are cleaner, equivalent typed variants are acceptable as long as the reducer no longer depends on ad hoc property-name strings for these core behaviors.

## Runtime Behavior

`VirtualList` normalization:

- `viewport_start` is clamped to `0..=total_count`.
- `viewport_count` is clamped to `0..=remaining_count` unless `total_count` is unknown or zero.
- `overscan` is non-negative.
- Visible item projection uses `start - overscan` through `start + count + overscan`, clamped to available authored/retained items.
- `items` and `total_count` stay separate because virtual lists may represent remote or lazy data that is not fully materialized.

`PagedList` normalization:

- `page_size` is at least `1`.
- `page_count` is derived from `total_count` and `page_size` when not explicitly retained, and explicit values are clamped to a non-negative range.
- `page_index` is clamped to `0..page_count.saturating_sub(1)`.
- Page item values stay separate from `total_count`, matching lazy or remote page data.

`WorldSpaceSurface` normalization:

- `world_position`, `world_rotation`, and `world_scale` remain Vec3 values.
- `world_size` remains Vec2 and rejects invalid shape/kind updates.
- `pixels_per_meter` is clamped to the descriptor range.
- `render_order` remains an integer retained value.
- `camera_target` remains an optional string value.
- This slice only records host/render metadata; graphics integration is deferred.

## Editor Projection

`zircon_editor::ui::slint_host::ui::pane_data_conversion::pane_component_projection` may keep projecting the existing `TemplatePaneNodeData` fields, but projection should consume normalized runtime state where available. The editor tests should continue to prove that virtualization, pagination, visible collection windows, and world-space metadata survive the generic host contract.

The current `visible_collection_items(...)` behavior is accepted as the host-contract projection shape, but runtime tests must cover the same boundary cases so the projection is not the only place where virtual-list semantics exist.

## Tests

Focused runtime tests should cover:

- `VirtualList` visible range clamping, overscan clamping, visible item window slicing, and separation of `items` from `total_count`.
- `PagedList` page-size minimum, derived page count, page-index clamping, and retained page item values.
- `WorldSpaceSurface` transform/surface updates, invalid value-kind rejection, descriptor range clamping for `pixels_per_meter`, and retained metadata values.
- Descriptor event coverage for the new typed event kinds.

Focused editor tests should cover:

- Existing host projection for virtualization and pagination metadata still passes after runtime normalization.
- Visible collection item projection handles negative starts, overscan larger than the item set, and zero visible counts deterministically.
- World-space host metadata projection remains data-only and does not require graphics/RHI code.

## Documentation

Implementation must update `docs/ui-and-layout/runtime-ui-component-showcase.md` or create a more focused Runtime UI complex-components document if the behavior grows beyond the showcase doc. The doc header must list touched runtime/editor files, tests, this spec, the implementation plan, and validation evidence.

## Acceptance Criteria

- Runtime component events explicitly represent complex-component state changes for virtual range, page selection, and world-space surface metadata.
- Reducer behavior normalizes or rejects invalid complex-component values at the runtime component layer.
- Editor projection remains generic and consumes the normalized retained state without becoming the semantic owner.
- Focused runtime and editor tests pass or blockers are documented as unrelated active-session drift.
- No graphics/plugin/watcher files are modified.
