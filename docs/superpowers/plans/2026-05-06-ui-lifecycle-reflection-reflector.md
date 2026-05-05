# UI Lifecycle Reflection Reflector Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the shared UI lifecycle/property/reflection/default-behavior truth and expose it through a minimal editor Widget Reflector-style snapshot consumer.

**Architecture:** `.ui.toml` remains the authored truth and `UiSurfaceFrame` remains the arranged/render/hit/focus frame truth. `zircon_runtime_interface::ui` defines neutral DTOs, `zircon_runtime::ui` owns mutation and snapshot generation, and `zircon_editor::ui` only consumes snapshots for Reflector display/export.

**Tech Stack:** Rust workspace on `main`, `serde`, `serde_json`, existing `zircon_runtime_interface`, `zircon_runtime`, `zircon_editor`, existing Cargo test/check commands with milestone-first testing stages.

---

## Current Baseline

- The workspace is intentionally dirty with extensive active UI edits. Do not revert unrelated changes.
- Active sibling sessions own hit-test, invalidation/performance, Material layout, native text/input, and painter/media/SVG/FPS paths.
- Existing shared DTOs include `UiDirtyFlags`, `UiTreeNode`, `UiTemplateNodeMetadata`, `UiSurfaceFrame`, `UiComponentDescriptor`, `UiComponentState`, and `UiReflectionSnapshot`.
- Existing runtime behavior includes `UiSurface`, `apply_component_event`, component descriptor registry, pointer component events, and surface dirty rebuild.
- Existing editor reflection path lives under `zircon_editor/src/ui/workbench/reflection` and should be extended as a consumer, not made authoritative.

## File Structure

### Shared Contracts

- Modify: `zircon_runtime_interface/src/ui/event_ui/reflection.rs`
  - Add lifecycle state, property source, invalidation reason, reflected property, reflector node, reflector hit context, and reflector snapshot DTOs.
  - Keep existing `UiReflectionSnapshot` compatible.
- Modify: `zircon_runtime_interface/src/ui/event_ui/mod.rs`
  - Re-export new DTOs.
- Modify: `zircon_runtime_interface/src/tests/contracts.rs`
  - Add serialization/contract tests for reflected lifecycle/property/reflector DTOs.

### Runtime Truth And Mutation

- Create: `zircon_runtime/src/ui/surface/property_mutation.rs`
  - Define `UiPropertyMutationRequest`, `UiPropertyMutationStatus`, `UiPropertyMutationReport`, and property update helpers.
- Create: `zircon_runtime/src/ui/surface/reflection_snapshot.rs`
  - Generate `UiReflectorSnapshot` from `UiSurface`, descriptors, metadata, state flags, dirty flags, focus state, and optional hit-test point.
- Modify: `zircon_runtime/src/ui/surface/mod.rs`
  - Export mutation and reflection helpers.
- Modify: `zircon_runtime/src/ui/surface/surface.rs`
  - Add `mutate_property(...)` and `reflector_snapshot(...)` methods.
- Modify: `zircon_runtime/src/ui/tests/event_routing.rs`
  - Add default-click/reflection behavior coverage if it fits the current surface tests.
- Modify: `zircon_runtime/src/ui/tests/shared_core.rs`
  - Add mutation dirty-classification and snapshot-generation coverage.
- Modify: `zircon_runtime/src/ui/tests/mod.rs`
  - Wire any new focused runtime test module if created.

### Editor Reflector Consumer

- Create: `zircon_editor/src/ui/workbench/reflection/widget_reflector.rs`
  - Define a minimal editor-side `WorkbenchWidgetReflectorModel` that consumes shared `UiReflectorSnapshot` and exposes selected-node details.
- Modify: `zircon_editor/src/ui/workbench/reflection/mod.rs`
  - Export `widget_reflector` types without changing existing route registration behavior.
- Add or modify focused editor tests under the existing editor test tree if a matching workbench reflection test module exists.

### Documentation

- Modify: `docs/ui-and-layout/shared-ui-core-foundation.md`
  - Document lifecycle/property mutation/dirty classification/reflection truth.
- Modify: `docs/ui-and-layout/slate-style-ui-surface-frame.md`
  - Document that Reflector consumes `UiSurfaceFrame` instead of recalculating frame authority.
- Create or update source-mirrored docs for any new runtime/editor modules if no existing UI doc already owns the behavior.

## Milestone 1: Shared Reflection Contracts

- Goal: Add neutral DTOs for lifecycle, property source, invalidation reason, reflected properties, and Widget Reflector snapshots.
- In-scope behaviors: serde-friendly defaults, stable read-only snapshot data, compatibility with existing `UiReflectionSnapshot`, explicit dirty/invalidation taxonomy.
- Dependencies: current `UiDirtyFlags`, `UiValueType`, `UiNodeId`, `UiNodePath`, `UiTreeId`, `UiVisibility`, and `UiStateFlags`.
- Implementation slices:
  - [ ] Add `UiWidgetLifecycleState` with `Declared`, `Constructed`, `PropertiesSynchronized`, `Arranged`, `Visible`, `Interactive`, and `Detached` variants.
  - [ ] Add `UiReflectedPropertySource` with authored, descriptor default, inferred default, runtime state, binding, and system state variants.
  - [ ] Add `UiPropertyInvalidationReason` fields that map to existing `UiDirtyFlags` plus a `reflection` flag.
  - [ ] Add `UiReflectedProperty`, `UiReflectorNode`, `UiReflectorHitContext`, and `UiReflectorSnapshot` DTOs in `reflection.rs`.
  - [ ] Re-export new DTOs from `event_ui/mod.rs`.
  - [ ] Add contract tests showing default lifecycle/property values serialize and carry dirty mappings.
- Testing stage:
  - Run `cargo test -p zircon_runtime_interface --lib contracts --locked`.
  - Run `cargo check -p zircon_runtime_interface --lib --locked`.
  - Debug/correct compile or assertion failures before promoting.
- Lightweight checks: use scoped `cargo check -p zircon_runtime_interface --lib --locked` only if type uncertainty blocks later slices.
- Exit evidence: contract tests prove new DTOs serialize, default correctly, and do not remove existing reflection DTOs.

## Milestone 2: Runtime Mutation And Snapshot Truth

- Goal: Add one canonical runtime property mutation seam and generate Reflector snapshots from retained `UiSurface` truth.
- In-scope behaviors: accepted/unchanged/rejected mutation reports, descriptor-aware property source inference, dirty classification, read-only snapshot generation, default click visibility in reflection.
- Dependencies: Milestone 1 DTOs, existing `UiSurface`, `UiTreeNode`, `UiTemplateNodeMetadata`, `UiComponentDescriptorRegistry`, and `UiComponentStateRuntimeExt`.
- Implementation slices:
  - [ ] Add `surface/property_mutation.rs` with mutation request/report/status types and dirty-classification helpers.
  - [ ] Implement mutation for node state/metadata properties that can be safely represented today: visibility, enabled, clickable, hoverable, focusable, pressed, checked, and metadata attributes.
  - [ ] Ensure unchanged values return `Unchanged` and do not set dirty flags.
  - [ ] Ensure unsupported or invalid value kinds return `Rejected` diagnostics without mutating the node.
  - [ ] Add `surface/reflection_snapshot.rs` to build `UiReflectorSnapshot` from `UiSurface` without rebuilding layout/hit/render state.
  - [ ] Add `UiSurface::mutate_property(...)` and `UiSurface::reflector_snapshot(...)` wrappers.
  - [ ] Add runtime unit tests for dirty classification and snapshot fields.
- Testing stage:
  - Run `cargo test -p zircon_runtime --lib event_routing --locked`.
  - Run `cargo test -p zircon_runtime --lib shared_core --locked`.
  - Run `cargo test -p zircon_runtime --lib component_catalog --locked`.
  - Run `cargo check -p zircon_runtime --lib --locked`.
  - Debug/correct lower-layer mutation or DTO failures before promoting.
- Lightweight checks: use scoped `cargo check -p zircon_runtime --lib --locked` if Rust type errors block editor integration.
- Exit evidence: runtime tests prove mutation reports, dirty flags, and snapshots are generated from retained truth.

## Milestone 3: Editor Reflector Consumer

- Goal: Add a minimal editor-side Widget Reflector model that consumes shared snapshots and exposes selected-node data.
- In-scope behaviors: tree rows, selected node details, reflected properties, lifecycle/dirty/focus visibility, read-only snapshot export data shape.
- Dependencies: Milestone 1 DTOs and Milestone 2 snapshot generation.
- Implementation slices:
  - [ ] Add `zircon_editor/src/ui/workbench/reflection/widget_reflector.rs` with a focused model over `UiReflectorSnapshot`.
  - [ ] Export the model from `zircon_editor/src/ui/workbench/reflection/mod.rs`.
  - [ ] Add tests for selecting a node, reading lifecycle/dirty/property fields, and rejecting missing selected nodes cleanly.
  - [ ] Keep this layer read-only unless a safe existing host route to `UiSurface::mutate_property(...)` already exists.
- Testing stage:
  - Run `cargo test -p zircon_editor --lib workbench_reflection --locked` if this target exists in the current test filter.
  - Run `cargo test -p zircon_editor --lib native_host_contract --locked` if touched host projection code requires it.
  - Run `cargo check -p zircon_editor --lib --locked`.
  - Debug/correct editor compile or model test failures before promoting.
- Lightweight checks: use scoped `cargo check -p zircon_editor --lib --locked` if module wiring is uncertain.
- Exit evidence: editor code can consume `UiReflectorSnapshot` without owning runtime UI state.

## Milestone 4: Docs, Acceptance, And Integration Validation

- Goal: Keep code/docs synchronized and prove the implemented layers work together.
- In-scope behaviors: UI docs, module docs, acceptance evidence, scoped integration commands, and whitespace checks.
- Dependencies: Milestones 1-3 implemented.
- Implementation slices:
  - [ ] Update `docs/ui-and-layout/shared-ui-core-foundation.md` for lifecycle, mutation, dirty classification, and reflection snapshot source of truth.
  - [ ] Update `docs/ui-and-layout/slate-style-ui-surface-frame.md` for Reflector snapshot consumption of `UiSurfaceFrame`.
  - [ ] Create source-mirrored docs for new modules if existing docs do not already own the behavior.
  - [ ] Update `.codex/sessions/20260506-0445-ui-lifecycle-reflection-reflector.md` with implementation files and validation evidence.
- Testing stage:
  - Run `git diff --check`.
  - Run the Milestone 1-3 command set that remains applicable after implementation.
  - If a workspace-level shared contract change causes broad type impact, run `cargo check --workspace --locked` or the repository validator if time and disk state allow it.
  - Debug/correct failures from the lowest shared layer upward.
- Lightweight checks: none beyond focused `cargo check` when blocked.
- Exit evidence: docs list implementation files, plan sources, tests, and behavior; validation commands and any remaining risks are reported.

## Coordination Rules

- Do not rewrite hit-test internals unless snapshot correctness proves a lower-layer blocker.
- Do not rewrite Material layout metrics or text/painter/image paths.
- Do not add a second UI tree, second hit-test index, or editor-owned UI authority.
- Do not commit unless explicitly requested.
