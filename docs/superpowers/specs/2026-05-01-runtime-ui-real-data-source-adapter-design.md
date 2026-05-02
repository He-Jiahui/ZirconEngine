# Runtime UI Real Data Source Adapter Design

## Context

Runtime UI Component Showcase now has a typed component event/state path for value changes, commits, drag/drop, popup, option selection, arrays, maps, and references. The remaining gap is authority: showcase state is still transient demo state, while real editor and runtime surfaces own their data elsewhere.

Current authoritative sources are split by layer:

- `zircon_runtime::ui::component` owns neutral component semantics through `UiComponentEvent`, `UiComponentState`, descriptors, values, validation, and drag payloads.
- `zircon_editor::ui::workbench::state::EditorState` and `EditorDataSnapshot` own editor scene, inspector, asset browser, and workbench projection facts.
- `zircon_editor::ui::host::editor_asset_manager::EditorAssetManager` owns editor asset catalog/details/change snapshots over runtime project asset truth.
- `zircon_editor::ui::asset_editor::session::UiAssetEditorSession` owns UI asset document/session/presentation state.
- `zircon_runtime::ui::runtime_ui::RuntimeUiManager` owns runtime fixture loading, shared `UiSurface`, input dispatch, and frame extraction, but does not yet consume a real game UI model.

The user selected the first milestone direction: add a shared adapter contract before wiring individual real data sources. This prevents editor-only or showcase-only special cases from becoming the next long-lived UI binding path.

## Approved Direction

Implement a shared Runtime UI component data-source/action adapter contract, then prove it with one low-conflict editor adapter slice.

The first milestone should add neutral envelopes and adapter result types around the existing `UiComponentEvent` and `UiComponentState` semantics. Editor and runtime hosts should use this contract to translate component events into authoritative source mutations and projection patches.

Preferred first vertical proof is the existing editor inspector draft surface, because it already has real mutable state and a dispatch path. Asset browser search/reference binding is acceptable as a second proof. UI Asset Editor session wiring and runtime gameplay fixtures should wait for follow-up milestones because active sessions currently own nearby files and runtime visual integration has a known plugin-loader blocker.

## Scope

In scope:

- Shared neutral adapter DTOs in `zircon_runtime::ui::component` or a child module such as `data_binding`.
- A host-side adapter trait or trait-like command surface that maps a component event envelope to a source-specific mutation/result.
- Editor adapter registry/wiring at the template-runtime or Slint-host boundary.
- One real editor data-source adapter, preferably inspector draft fields.
- Projection patch data that can refresh a `TemplatePane`/host model without making Slint a business source.
- Tests for shared contract behavior, fake adapter behavior, and the chosen real editor adapter.
- Docs updates for Runtime UI component data binding and the chosen editor integration.

Out of scope:

- New `.ui.toml` schema for full declarative data-source bindings.
- UI Asset Editor hot-reload/conflict work owned by the active M5 watcher session.
- Runtime graphics/RHI/render graph changes.
- Final runtime HUD/pause/settings/inventory gameplay model integration.
- Full generic expression binding language.
- Replacing existing inspector, asset browser, or UI Asset Editor command surfaces in one step.

## Architecture

### Ownership

The shared contract belongs under `zircon_runtime::ui::component` because `UiComponentEvent`, `UiComponentState`, `UiValue`, validation, and drag payloads already live there and are editor-neutral. This layer may define neutral data-source envelopes, stable component/control identity, source paths, adapter results, and projection patch payloads.

The editor owns authoring adapters in `zircon_editor::ui`. Editor adapters may read and mutate `EditorState`, `EditorDataSnapshot`, editor asset manager snapshots, or UI asset sessions, but those concrete types must not enter `zircon_runtime`.

Runtime gameplay adapters belong later under `zircon_runtime::ui::runtime_ui`. They should map HUD, pause, settings, and inventory component events to runtime game/config/inventory models through the same shared envelope, not through a fixture-specific route table.

### Contract Shape

The shared layer should introduce the smallest neutral vocabulary needed to move from showcase-only state to real source-backed state.

Expected concepts:

- `UiComponentBindingTarget`: identifies the host-owned source domain and path, such as inspector field, asset filter, asset reference, UI asset field, or runtime HUD field.
- `UiComponentEventEnvelope`: carries document id, control id, optional component id, target, event kind, typed `UiComponentEvent`, and optional source metadata.
- `UiComponentAdapterResult`: reports whether the source changed, which component state/properties changed, optional validation/error text, optional status text, and whether projection must refresh.
- `UiComponentProjectionPatch`: describes host-model attributes or component-state values to overlay after the source mutation.
- `UiComponentAdapterError`: structured failure for unsupported target, missing source, invalid value kind, rejected event, stale source, or host mutation failure.

The contract should reuse `UiComponentEvent`, `UiComponentEventKind`, `UiComponentState`, `UiValue`, `UiDragPayload`, and `UiDragSourceMetadata`. It should not duplicate event semantics or introduce string-only value dispatch.

### Editor Adapter Registry

Editor host code should register adapters by target domain rather than by showcase action id.

Initial domains:

- `inspector`: selected object draft fields such as `name`, `parent`, and `transform.translation.x/y/z`.
- `asset_browser`: search/filter/view/selection and real reference payloads, if chosen as the second adapter.
- `showcase`: existing demo state retained as a compatibility test/demo adapter, not as the architecture authority.

The registry should live near existing editor UI dispatch/projection boundaries, not inside `workbench.slint` or generic Slint host files. Likely implementation paths are `zircon_editor/src/ui/template_runtime` for projection-side integration and `zircon_editor/src/ui/slint_host/app` for callback-side routing.

### First Real Adapter

The recommended first real adapter is inspector draft mutation.

Current inspector dispatch hardcodes control id to field id in `zircon_editor/src/ui/slint_host/app/inspector.rs`, then mutates `EditorState` through `zircon_editor/src/ui/binding_dispatch/inspector/apply.rs`. The adapter milestone should preserve the same authority and behavior while routing through the shared component event envelope.

The first adapter should prove this flow:

`TemplatePane control/action -> UiComponentEventEnvelope -> inspector adapter -> EditorState draft mutation -> EditorDataSnapshot/host projection refresh -> status/result evidence`

The adapter does not need to convert every inspector field or every editor pane. It needs enough coverage to prove typed component `ValueChanged` and `Commit` events mutate real editor state without relying on showcase demo storage.

### Projection Model

Projection patches should be data-only overlays, not direct Slint property writes.

Allowed patch targets:

- host node attributes that already represent runtime UI component properties;
- component state values keyed by property;
- validation text or status text;
- source summary metadata for reference/drop controls.

Disallowed patch targets:

- control-specific Slint frame getters/setters;
- editor-only concrete state types inside `zircon_runtime`;
- renderer-specific commands;
- new showcase-only branches in shared foundations.

### Runtime Integration Path

Runtime game UI should consume the same envelope later through `RuntimeUiManager` or a runtime UI service registered by `UiModule`.

The first spec does not require production service registration, because the current `RuntimeUiManager` is crate-private and fixture-focused. The contract should nevertheless keep runtime suitability by using neutral target domains and typed values rather than editor concrete paths.

## Data Flow

Editor flow for the first milestone:

`TOML binding/action id -> host callback -> UiComponentEventEnvelope -> EditorUiComponentAdapterRegistry -> inspector adapter -> EditorState draft field update -> snapshot/projection rebuild -> TemplatePane node attribute patch`

Runtime flow for later milestones:

`runtime fixture control/action -> UiComponentEventEnvelope -> runtime game adapter -> HUD/settings/inventory model mutation -> UiSurface refresh -> UiRenderExtract -> runtime frame`

Showcase flow after the adapter contract:

`showcase control/action -> UiComponentEventEnvelope -> showcase adapter -> existing UiComponentShowcaseDemoState -> host model overlay`

This keeps showcase useful as a coverage fixture while demoting it from the only component-state owner.

## Reference Alignment

Slint reference evidence:

- `dev/slint/tools/lsp/preview/ui/property_view.rs` maps compiler property information into UI-facing property declarations and values, separating source metadata from rendered controls.
- `dev/slint/tools/lsp/preview/ui/search_model.rs` wraps a source model with a filtered model and explicit search text state, supporting an adapter-style separation between source truth and UI projection.

Godot reference evidence:

- `dev/godot/editor/inspector/editor_inspector.h` separates `EditorProperty` controls from edited object/property identity.
- `dev/godot/editor/inspector/editor_inspector.cpp` emits `property_changed` from editor property controls and lets the inspector/editor object path apply the mutation.

Bevy reference evidence:

- `dev/bevy/crates/bevy_asset/src/event.rs` uses typed asset events with stable ids and untyped conversion for generic consumers, which supports using typed component events plus neutral envelopes instead of string-only dispatch.

Zircon repository evidence:

- `zircon_runtime/src/ui/component/event.rs` already defines the typed event vocabulary.
- `zircon_runtime/src/ui/component/state.rs` already applies descriptor-validated events and stores values/flags/reference metadata.
- `zircon_editor/src/ui/template_runtime/showcase_demo_state.rs` proves the typed events can drive retained state, but currently only for demo state.
- `zircon_editor/src/ui/slint_host/app/inspector.rs` and `zircon_editor/src/ui/binding_dispatch/inspector/apply.rs` prove real inspector mutation exists but is hardcoded to inspector controls.
- `zircon_runtime/src/ui/runtime_ui/runtime_ui_manager.rs` proves runtime fixtures already load through shared `UiSurface`, but fixture data remains static.

Deliberate divergence:

- Unlike Slint's property binding runtime, Zircon should not make the toolkit host the binding authority. The binding authority stays in runtime/editor data-source adapters because `.ui.toml -> UiSurface -> host projection` is the repository cutover rule.
- Unlike Godot's object/Variant inspector path, Zircon should keep `UiValue` and `UiComponentEvent` typed and descriptor-validated before reaching host mutations.
- Unlike Bevy's ECS asset events, this adapter contract is not an ECS schedule event yet. It is a UI component event envelope that can later be bridged into runtime schedules if needed.

## Error Handling

Adapter errors should be structured and user-visible through status/projection diagnostics where applicable.

Required failure classes:

- unsupported target domain;
- missing selected subject or missing source path;
- unsupported component event kind for a target;
- invalid value kind or parse failure;
- rejected reference/drop payload;
- stale source after a host snapshot changed;
- host mutation failure from `EditorState`, asset manager, UI asset session, or runtime model.

Failure behavior:

- Do not mutate source state after validation fails.
- Do not fall back to showcase demo state for a real target.
- Do not panic on unknown control ids, unknown targets, or missing selected objects.
- Return projection/status evidence so focused tests can assert the failure path.
- Preserve existing inspector behavior for unsupported fields until a field is explicitly migrated.

## Milestones

### Milestone 1: Shared Adapter Contract

Implementation slices:

- Add shared envelope, target, result, patch, and error types under `zircon_runtime::ui::component` or `component::data_binding`.
- Add conversion helpers from existing binding/action facts into `UiComponentEventEnvelope` without changing `.ui.toml` schema.
- Add unit tests for typed value preservation, unsupported target errors, and projection patch serialization if the types are serializable.

Testing stage:

- Run focused `zircon_runtime` component tests.
- Run `cargo check -p zircon_runtime --lib --locked` if the milestone changes runtime public module exports.

Acceptance evidence:

- Shared contract compiles without editor dependencies.
- No string-only replacement for `UiComponentEvent` is introduced.

### Milestone 2: Editor Adapter Registry And Showcase Adapter

Implementation slices:

- Add editor adapter registry/wiring around existing template-runtime or Slint-host callback dispatch.
- Wrap existing `UiComponentShowcaseDemoState` behind the adapter path.
- Keep old showcase behavior green while proving events now pass through the same envelope shape intended for real sources.

Testing stage:

- Run focused `zircon_editor` component showcase tests.
- Run projection tests that assert control id, target, event kind, and typed values survive the registry path.

Acceptance evidence:

- Showcase remains a consumer of the adapter contract, not a parallel architecture path.

### Milestone 3: Inspector Real Data Adapter

Implementation slices:

- Define inspector target mapping for selected-object draft fields.
- Route `ValueChanged` and `Commit` envelopes to existing inspector draft mutation behavior.
- Produce projection/status patches after real `EditorState` mutation.
- Add negative coverage for missing selection, unsupported field, and invalid numeric/string values.

Testing stage:

- Run focused inspector binding/dispatch tests.
- Run selected template-runtime or Slint-host tests that exercise the new adapter path.
- Run `cargo check -p zircon_editor --lib --locked` after fixing focused failures.

Acceptance evidence:

- At least one Runtime UI component event mutates real inspector draft state.
- The same path updates projection without touching Slint business authority.
- Existing hardcoded inspector dispatch remains only as an unmigrated fallback for fields not covered by this milestone.

### Milestone 4: Asset Browser Or Runtime Follow-Up Design

This spec does not implement the next consumer. After Milestone 3, choose one follow-up:

- asset browser search/reference source adapter;
- UI Asset Editor session adapter after the watcher session completes;
- runtime gameplay HUD/settings/inventory adapter after runtime visual/plugin blockers clear.

The follow-up should reuse the same contract and add only target-specific adapters.

## Testing And Acceptance

Contract tests:

- envelope carries document id, control id, component id, target, event kind, and typed value without lossy string conversion;
- unsupported target reports structured error;
- projection patch can overlay attributes or values without depending on editor concrete types.

Showcase adapter tests:

- existing value/commit/drop/select events still mutate retained showcase state through the adapter path;
- event log remains stable where existing tests require it;
- no real target silently falls back to showcase state.

Inspector adapter tests:

- `ValueChanged` for selected object name updates the inspector draft name;
- `Commit` for a transform field updates the draft transform field through the adapter path;
- missing selection returns a structured adapter error and leaves state unchanged;
- unsupported field returns structured unsupported-target or unsupported-field error;
- projection/status marks the presentation dirty only after successful mutation.

Validation commands should be declared in the implementation plan. Expected first set:

- `cargo test -p zircon_runtime --lib ui::tests::component_catalog --locked --jobs 1`
- `cargo test -p zircon_editor --lib component_showcase --locked --jobs 1`
- `cargo test -p zircon_editor --lib inspector --locked --jobs 1`
- `cargo check -p zircon_editor --lib --locked --jobs 1`

Milestone-first policy applies: implementation slices may add tests, but build/test execution belongs to the milestone testing stage unless a concrete blocker requires earlier evidence.

## Documentation

Update `docs/ui-and-layout/runtime-ui-component-showcase.md` after implementation to explain:

- shared component adapter envelope;
- showcase as a demo adapter rather than the architecture authority;
- first real inspector-backed adapter path;
- focused validation commands.

Add or update a module document under `docs/ui-and-layout/` or `docs/zircon_runtime/ui/` for the shared adapter contract if implementation creates a new `component::data_binding` module.

Update `docs/editor-and-tooling/` only if the first real adapter changes documented inspector/editor host behavior.

## Coordination Notes

Avoid these active-session areas unless the implementation plan explicitly coordinates them:

- `zircon_editor/src/ui/host/asset_editor_sessions/**`, owned by the UI Asset Editor workspace watcher session.
- `zircon_runtime::graphics::{rhi,rhi_wgpu,render_graph}` and broad renderer internals, owned by SRP/RHI and runtime visual sessions.
- Runtime UI visual integration tests currently blocked by native plugin loader drift in active render/plugin work.
- Showcase schema/category/value-state files should only be touched when wrapping the existing showcase path behind the adapter contract.

## Review Gate

This design is approved for spec capture only. Implementation should not begin until an implementation plan is written from this spec and reviewed.

The implementation plan should keep the order bottom-up: shared contract, editor registry, showcase adapter reuse, inspector real-data adapter, docs, milestone validation.
