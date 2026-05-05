# UI Lifecycle, Property Reflection, And Reflector Design

## Summary

- Approved scope: implement shared UI lifecycle/property/reflection/default-behavior truth plus an editor Widget Reflector-style consumer surface.
- Architecture truth remains `.ui.toml` for authored declarations and `UiSurfaceFrame` for arranged/render/hit/focus frame data.
- The new reflection model is a derived debug and authoring view. It must not become a second layout, hit-test, render, or event authority.
- Unreal Slate leads the behavior reference. Zircon translates the responsibilities into retained `.ui.toml` descriptors, runtime-interface DTOs, runtime mutation seams, and editor inspection views.

## Reference Evidence

- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Widgets/SWidget.h`: widget lifecycle shape, default unhandled event behavior, prepass, paint, arrange, children traversal, and debug child access.
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Widgets/SWidget.cpp`: base defaults for visibility, enabled, hover, tick/update flags, clipping, pixel snapping, render opacity, and registered attribute invalidation reasons.
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Types/SlateAttribute.h`: attributes update during prepass, compare cached values, and invalidate only when values change.
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Types/SlateAttributeDescriptor.h`: static attribute descriptors carry names, sort order, invalidation reason, visibility-affecting markers, and value-changed callbacks.
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Types/SlateAttributeMetaData.h`: per-widget attribute metadata exposes update paths and debug counts.
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Widgets/InvalidateWidgetReason.h`: invalidation reason taxonomy for layout, paint, visibility, child order, render transform, prepass, and attribute registration.
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Types/ReflectionMetadata.h`: reflection metadata records widget name, class/source object, owning asset, and debug path data.
- `dev/UnrealEngine/Engine/Source/Developer/SlateReflector/Private/Models/WidgetReflectorNode.h`: live and snapshot reflector nodes expose type, visibility, focus, tick, volatile, timers, invalidation root, source, attribute counts, desired size, address, enabled state, and asset data.
- `dev/UnrealEngine/Engine/Source/Runtime/UMG/Private/UserWidget.cpp`: user widget lifecycle stages distinguish initialize, pre-construct, construct, synchronize properties, rebuild, destruct, and release resources.

## Current Zircon Baseline

- Authored UI starts as `.ui.toml`, compiles into `UiTemplateNode`, then builds retained `UiTree` and `UiTreeNode` instances.
- `UiSurface` owns the live runtime tree plus `UiArrangedTree`, hit-test index, focus/navigation state, and `UiRenderExtract`.
- `UiSurfaceFrame` already packages arranged tree, render extract, hit grid, and focus state as the frame-level authority.
- `UiDirtyFlags` exists with `layout`, `hit_test`, `render`, `style`, `text`, `input`, and `visible_range`, but property mutations do not yet flow through one canonical invalidation classifier.
- `UiReflectionSnapshot` exists, but it is a separate descriptor snapshot and is not yet generated directly from the retained surface/tree plus component descriptors.
- Component defaults and behavior are spread across descriptor catalogs, template compiler defaults, inferred interaction/container behavior, material layout, render/style resolution, and component state reducer logic.
- Editor-side reflection currently has workbench reflection pieces, but not a complete Widget Reflector-style view driven by shared runtime snapshot data.

## Target Architecture

### Ownership

- `zircon_runtime_interface::ui` owns neutral lifecycle, property, invalidation, and reflection DTOs.
- `zircon_runtime::ui` owns behavior: descriptor default resolution, mutation, state reduction, dirty classification, surface rebuild integration, and reflection snapshot generation.
- `zircon_editor::ui` owns the authoring/debug consumer: Widget Reflector tree, property grid, lifecycle/dirty/focus/hit details, and snapshot export or replay hooks.
- Layout, hit-test, render, text, and invalidation performance internals remain owned by their existing runtime/editor sessions. This design only consumes their current `UiSurfaceFrame` and dirty-frame data.

### Lifecycle Model

Add a shared `UiWidgetLifecycleState` and node lifecycle metadata that can represent retained `.ui.toml` widgets without copying Unreal's object model.

Required states:

- `Declared`: authored in `.ui.toml` or template descriptors but not yet materialized into a runtime tree node.
- `Constructed`: built into `UiTreeNode` and attached to a `UiTree`.
- `PropertiesSynchronized`: descriptor defaults, authored attributes, component state, and inferred defaults have been applied.
- `Arranged`: included in the latest arranged tree pass.
- `Visible`: render-visible according to effective visibility and parent visibility.
- `Interactive`: enabled and eligible for focus, hover, click, or input based on state flags and input policy.
- `Detached`: removed from the retained tree or hidden from current roots.

Lifecycle is diagnostic state, not a scheduler. It should be updated by build, synchronization, layout, visibility, and detach operations already present in the runtime UI path.

### Property And Attribute Model

Add a shared reflected property model that records:

- property name
- value kind
- current resolved value
- authored value, when present
- descriptor default value, when present
- source: authored, descriptor default, inferred default, runtime state, binding, or system state
- readable/writable flags
- invalidation reason
- visibility-affecting marker
- validation state and message

The runtime must expose one mutation seam for properties. All property changes should pass through it, including editor property edits, component events, default action effects, and descriptor default synchronization.

The mutation seam must:

- find the target retained node
- resolve the node's component descriptor and metadata
- validate value type and bounds when descriptor schema is available
- compare old and new values before marking dirty
- update component state and retained metadata consistently
- classify dirty impact into existing `UiDirtyFlags`
- update lifecycle/reflection metadata
- return structured diagnostics for accepted, unchanged, rejected, or unsupported changes

### Invalidation Classification

Translate Slate invalidation categories into the existing Zircon flags instead of introducing a competing tree.

Mapping:

- Slate `Layout` and `Prepass` -> `UiDirtyFlags.layout`
- Slate `Paint` -> `UiDirtyFlags.render`
- Slate `Visibility` -> `UiDirtyFlags.hit_test`, `UiDirtyFlags.render`, and layout when collapsed semantics change arranged geometry
- Slate `RenderTransform` -> `UiDirtyFlags.render` plus `UiDirtyFlags.hit_test` when hit geometry changes
- Slate `ChildOrder` -> `UiDirtyFlags.layout`, `hit_test`, and `render`
- Slate `AttributeRegistration` -> reflection-only dirty plus relevant flag for the newly registered property
- Text/content value changes -> `text`, `layout`, and `render` when measured size can change
- Input policy or focusability changes -> `input` and `hit_test`
- Virtual range changes -> `visible_range`, `layout`, and `render`

### Default Behavior Model

Default behavior must become descriptor-driven and inspectable.

Rules:

- Descriptor default values remain the first default source for component properties.
- `.ui.toml` authored values override descriptor defaults.
- Inferred defaults such as input policy, focusability, hover/click flags, material metrics, and container behavior must be recorded as `inferred default` reflected properties.
- Pointer default click should continue to emit a component event, but the event should be applied through the same property mutation/state reducer path instead of becoming a detached side effect.
- Unsupported component events must be rejected with structured diagnostics rather than silently mutating state.
- Unchanged value updates should produce no dirty flags.

### Reflection Snapshot Model

Generate reflection from retained truth instead of hand-building a separate tree.

Snapshot source inputs:

- `UiTree` and `UiTreeNode`
- `UiSurfaceFrame`
- component descriptors and state
- template metadata and bindings
- dirty flags and lifecycle metadata
- focus, hover, capture, pressed, navigation state
- hit-test debug data when a point is provided

Each reflected node should include:

- node id/path and parent/children
- class/component id and display name
- lifecycle state
- visibility and effective visibility
- enabled/focusable/clickable/hoverable/pressed/checked flags
- input policy, z index, paint order, clip state, frame, desired or measured size when available
- dirty flags
- properties grouped by source
- bindings/actions
- focus/hover/capture annotations
- source asset/template path when available

Reflection snapshot generation must be read-only. It must not mutate tree state or rebuild surface data.

### Editor Reflector Surface

Add an editor-facing Reflector view that consumes shared snapshots.

Minimum UI behavior:

- tree view of reflected nodes
- selected node details
- property table with source, resolved value, writability, invalidation reason, and validation state
- lifecycle/dirty/focus/hit-test details
- snapshot export for diagnostics

The editor view may issue property edits only through the runtime mutation seam. It must not write `UiTreeNode` or component state directly.

The first Reflector view should avoid broad Slint host rewrites. It can be hosted through existing workbench reflection or inspector projection paths and should reuse current host contract patterns.

## Approaches Considered

### Option A: Core-Only Reflection Contracts

- Pros: smallest risk, mostly runtime-interface/runtime work, avoids editor overlap.
- Cons: does not satisfy the requested Reflector UI scope.
- Rejected because the approved scope includes editor Reflector UI.

### Option B: Shared Truth Plus Reflector Consumer Surface

- Pros: builds the missing architecture truth first, keeps editor UI as a consumer, avoids duplicating layout/hit/render logic, and respects active sibling sessions.
- Cons: requires touching both runtime and editor surfaces, with careful coordination.
- Selected.

### Option C: Full Slate-Style Widget System Replacement

- Pros: closest to Unreal's live `SWidget` architecture.
- Cons: contradicts Zircon's intentional `.ui.toml` retained-tree truth, risks rewriting layout/hit/render/text paths, and conflicts with active work.
- Rejected.

## Milestones

### M1 Shared Contracts

Implementation slices:

- Add lifecycle, property source, invalidation reason, and reflected property DTOs under `zircon_runtime_interface::ui`.
- Extend reflection snapshot DTOs without breaking existing tree/node descriptor semantics.
- Add contract tests for serialization defaults and representative node snapshots.

Testing stage:

- `cargo test -p zircon_runtime_interface --lib contracts --locked`
- `cargo check -p zircon_runtime_interface --lib --locked`

### M2 Runtime Mutation And Reflection Generation

Implementation slices:

- Add the canonical UI property mutation seam in `zircon_runtime::ui`.
- Route component state reducer changes and default click effects through the seam where practical.
- Add dirty classification for reflected property changes.
- Generate reflection snapshots from `UiSurface`, `UiTree`, descriptors, state, and `UiSurfaceFrame`.

Testing stage:

- `cargo test -p zircon_runtime --lib event_routing --locked`
- `cargo test -p zircon_runtime --lib component_catalog --locked`
- `cargo test -p zircon_runtime --lib shared_core --locked`
- `cargo check -p zircon_runtime --lib --locked`

### M3 Editor Reflector Consumer

Implementation slices:

- Add workbench reflection model/projection for snapshot tree and node details.
- Add the smallest Slint host or native host projection needed to display Reflector data.
- Add property edit routing back to the runtime mutation seam when existing host boundaries allow it.
- Add snapshot export path for diagnostics if it can be added without broad host rewrites.

Testing stage:

- `cargo test -p zircon_editor --lib workbench_reflection --locked`
- `cargo test -p zircon_editor --lib native_host_contract --locked`
- `cargo check -p zircon_editor --lib --locked`

### M4 Docs And Acceptance

Implementation slices:

- Update `docs/ui-and-layout/shared-ui-core-foundation.md`.
- Update or create module docs for any new runtime-interface/runtime/editor reflection modules.
- Add acceptance notes covering lifecycle/property/reflection/default behavior and Reflector snapshot export.

Testing stage:

- Run `git diff --check` on touched docs/source files.
- Run the scoped test commands from M1-M3 after final integration.

## Validation And Acceptance

Acceptance requires:

- reflected properties show authored/default/inferred/runtime sources distinctly
- unchanged property mutation produces no dirty flags
- layout-affecting property mutation marks layout-dependent dirty flags
- paint-only property mutation marks render dirty only when no layout/input/hit-test impact exists
- visibility-affecting mutation updates render and hit-test state and records the visibility reason
- default click behavior routes through component event/state mutation and is visible in reflection diagnostics
- Reflector tree displays lifecycle, visibility, state flags, dirty flags, and property data from shared snapshot DTOs
- editor Reflector property edits use the runtime mutation seam or are disabled when no safe route exists
- reflection snapshot export is read-only and reproducible

## Coordination Constraints

- Do not rewrite active hit-test files unless the runtime mutation seam exposes a lower-layer bug that blocks reflection correctness.
- Do not rewrite layout/material metrics owned by the Material layout session.
- Do not rewrite painter/text/image behavior owned by native text/input or asset/SVG/FPS sessions.
- Do not introduce a second runtime tree, second hit-test index, or editor-owned UI state authority.
- Do not add compatibility shims for old reflection paths if the new path replaces them cleanly.

## Documentation Plan

- `docs/ui-and-layout/shared-ui-core-foundation.md`: update the shared truth, lifecycle, property mutation, dirty classification, and reflection sections.
- `docs/ui-and-layout/slate-style-ui-surface-frame.md`: record that Reflector consumes `UiSurfaceFrame` and retained tree data instead of recalculating frame authority.
- New module docs under `docs/zircon_runtime_interface/ui/...`, `docs/zircon_runtime/ui/...`, and `docs/zircon_editor/ui/...` when new modules are created.
- Each module doc must include related code, implementation files, plan sources, and tests per repository documentation rules.

## Out Of Scope

- Full keyboard/text/IME system.
- Full drag/drop or popup reply/effect model.
- Text shaping, BiDi, caret, and selection implementation.
- Renderer drawcall/overdraw/material batch diagnostics.
- Complete Slate invalidation root or local retained render cache.
- World-space or 3D UI hit testing.

## Risks

- The working tree is heavily dirty and multiple active sessions are touching UI paths. Implementation must avoid reverting or rewriting sibling changes.
- Editor Reflector UI can expand into Slint host churn. The first implementation must keep it as a snapshot consumer and not a host rewrite.
- Existing reflection DTO names may already be consumed by tests. Extensions should be additive unless a hard cutover is clearly safer and coordinated.
- Property mutation can accidentally duplicate component state reducer behavior. The seam should call or wrap existing reducers rather than reimplementing every event rule.
