# Runtime UI Complex Components Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn `VirtualList`, `PagedList`, and `WorldSpaceSurface` from descriptor/projection metadata into explicit retained Runtime UI component behavior.

**Architecture:** `zircon_runtime::ui::component` owns typed events, normalization, reducer behavior, and descriptor event coverage. `zircon_editor` remains a generic projection consumer and must not own virtual-window, page, or world-space semantics. Graphics/RHI and asset-editor watcher areas stay out of scope.

**Tech Stack:** Rust, Serde, TOML-backed `UiValue`, existing Runtime UI component descriptor/state tests, existing editor Slint host-contract projection tests, Markdown docs.

---

## Execution Policy

- Work directly on `main` in the existing checkout.
- Do not create worktrees or feature branches.
- Do not commit unless the user explicitly asks for a commit.
- During implementation slices, add production code, focused test code, comments, and docs without forcing per-slice build/test loops.
- Run compile/unit-test commands only in the milestone testing stage unless a blocker requires earlier evidence.
- Do not edit graphics/GI/VG/plugin files, `zircon_editor/src/ui/host/asset_editor_sessions/**`, or unrelated UI Asset Editor watcher/session files.

## Design Source

- `docs/superpowers/specs/2026-05-01-runtime-ui-complex-components-design.md`
- `docs/ui-and-layout/runtime-ui-component-showcase.md`
- `.codex/sessions/20260429-0719-runtime-ui-showcase-schema-panel.md`
- `.codex/sessions/20260501-1518-ui-foundation-m10-m12-m13-design.md`

## Affected Files

### Runtime Component Contract

- Modify `zircon_runtime/src/ui/component/event.rs` for new typed complex-component event kinds and error variants.
- Modify `zircon_runtime/src/ui/component/state.rs` for reducer normalization and validation helpers.
- Modify `zircon_runtime/src/ui/component/catalog/editor_showcase.rs` so descriptors advertise the new event kinds.
- Modify `zircon_runtime/src/ui/tests/component_catalog.rs` for descriptor event coverage and reducer tests unless the file crosses the large-file threshold.
- If test size grows, create `zircon_runtime/src/ui/tests/component_catalog/complex_components.rs` and modify `zircon_runtime/src/ui/tests/component_catalog.rs` only as a structural test-module entry.

### Editor Projection Consumer

- Modify `zircon_editor/src/ui/slint_host/ui/pane_data_conversion/pane_component_projection/mod.rs` only for host projection normalization parity and focused projection tests.
- Do not add editor-owned reducers or editor-only semantics for the complex components.

### Documentation And Coordination

- Update `docs/ui-and-layout/runtime-ui-component-showcase.md` with related code, implementation files, plan/spec sources, tests, behavior notes, and validation evidence.
- Update `.codex/sessions/20260429-0719-runtime-ui-showcase-schema-panel.md` with the complex component slice state, validation evidence, and any blockers.

## Core Type Shapes

Add these event kinds to `UiComponentEventKind`:

```rust
SetVisibleRange,
SetPage,
SetWorldTransform,
SetWorldSurface,
```

Add these variants to `UiComponentEvent`:

```rust
SetVisibleRange {
    start: i64,
    count: i64,
},
SetPage {
    page_index: i64,
    page_size: i64,
},
SetWorldTransform {
    position: [f64; 3],
    rotation: [f64; 3],
    scale: [f64; 3],
},
SetWorldSurface {
    size: [f64; 2],
    pixels_per_meter: f64,
    billboard: bool,
    depth_test: bool,
    render_order: i64,
    camera_target: String,
},
```

Add these `UiComponentEventError` variants:

```rust
#[error("event {event_kind:?} requires component {expected_component_id}")]
UnsupportedComponentForEvent {
    expected_component_id: String,
    event_kind: UiComponentEventKind,
},
#[error("invalid complex component value {property}={value}")]
InvalidComplexValue {
    property: String,
    value: String,
},
```

## Milestone 1: Runtime Complex Component Reducers

- Goal: `VirtualList`, `PagedList`, and `WorldSpaceSurface` have typed retained-event behavior in `UiComponentState`.
- In-scope behaviors: event-kind declarations, descriptor event support, range/page/world reducer paths, value normalization, invalid-value errors, and focused runtime tests.
- Dependencies: existing `UiComponentEvent`, `UiComponentState`, `UiComponentDescriptorRegistry::editor_showcase()`, `UiValue`, and descriptor prop ranges.

### Implementation Slices

- [ ] Extend `UiComponentEventKind`, `UiComponentEvent`, and `UiComponentEvent::kind()` support if `kind()` lives outside `event.rs`.
- [ ] Add `UnsupportedComponentForEvent` and `InvalidComplexValue` errors to `UiComponentEventError`.
- [ ] Add helper methods in `UiComponentState`:

```rust
fn ensure_component_event(
    descriptor: &UiComponentDescriptor,
    expected_component_id: &str,
    event_kind: UiComponentEventKind,
) -> Result<(), UiComponentEventError>
```

```rust
fn apply_visible_range(
    &mut self,
    descriptor: &UiComponentDescriptor,
    start: i64,
    count: i64,
) -> Result<(), UiComponentEventError>
```

```rust
fn apply_page(
    &mut self,
    descriptor: &UiComponentDescriptor,
    page_index: i64,
    page_size: i64,
) -> Result<(), UiComponentEventError>
```

```rust
fn apply_world_transform(
    &mut self,
    descriptor: &UiComponentDescriptor,
    position: [f64; 3],
    rotation: [f64; 3],
    scale: [f64; 3],
) -> Result<(), UiComponentEventError>
```

```rust
fn apply_world_surface(
    &mut self,
    descriptor: &UiComponentDescriptor,
    size: [f64; 2],
    pixels_per_meter: f64,
    billboard: bool,
    depth_test: bool,
    render_order: i64,
    camera_target: String,
) -> Result<(), UiComponentEventError>
```

- [ ] Implement `SetVisibleRange` normalization: `viewport_start = max(start, 0)`, `viewport_count = max(count, 0)`, and when `total_count > 0`, clamp start and count inside the total range.
- [ ] Implement `SetPage` normalization: `page_size = max(page_size, 1)`, derive `page_count = ceil(total_count / page_size)` when `total_count > 0`, clamp page index into the derived page range, and store `page_index`, `page_size`, and `page_count`.
- [ ] Implement `SetWorldTransform`: store `world_position`, `world_rotation`, and `world_scale` as `UiValue::Vec3`, reject non-positive scale components with `InvalidComplexValue`.
- [ ] Implement `SetWorldSurface`: store `world_size` as `UiValue::Vec2`, clamp `pixels_per_meter` through the descriptor prop range, store bool/int/string metadata, and reject non-positive size components with `InvalidComplexValue`.
- [ ] Update `editor_showcase.rs` descriptors so `VirtualList` advertises `SetVisibleRange`, `PagedList` advertises `SetPage`, and `WorldSpaceSurface` advertises `SetWorldTransform` plus `SetWorldSurface`.
- [ ] Add runtime tests covering descriptor event support and reducer behavior for normal, negative, and overflow-like boundaries.

### Testing Stage: Runtime Reducer Gate

Run targeted formatting for changed Runtime UI files:

```powershell
rustfmt --edition 2021 --check zircon_runtime/src/ui/component/event.rs zircon_runtime/src/ui/component/state.rs zircon_runtime/src/ui/component/catalog/editor_showcase.rs zircon_runtime/src/ui/tests/component_catalog.rs
```

Run focused runtime component tests:

```powershell
cargo test -p zircon_runtime --lib component_catalog --locked --jobs 1 --target-dir target\codex-runtime-ui-complex-components --message-format short --color never
```

Run scoped runtime type check:

```powershell
cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir target\codex-runtime-ui-complex-components --message-format short --color never
```

### Debug / Correction Loop

- If descriptor tests fail, fix `editor_showcase.rs` event registration before editing reducer behavior.
- If reducer tests fail on clamping, inspect the stored `UiValue` values before changing editor projection.
- If runtime compile fails in graphics/plugin modules, read `.codex/sessions/20260501-1609-runtime-graphics-plugin-validation-blocker.md` before editing anything outside Runtime UI.

### Exit Evidence

- Runtime component catalog tests cover the new event kinds and reducer behavior.
- Runtime type check passes or an unrelated active-session blocker is recorded with the exact compiler diagnostic.

## Milestone 2: Editor Projection Parity

- Goal: Editor host projection remains generic and preserves normalized complex-component state without owning semantics.
- In-scope behaviors: visible collection item slicing boundaries, normalized virtual range metadata, page metadata, and world-space metadata projection.
- Dependencies: Milestone 1 reducer behavior and existing `TemplatePaneNodeData` fields.

### Implementation Slices

- [ ] Keep `TemplatePaneNodeData` unchanged unless a field is required by Milestone 1 tests and missing from the current host contract.
- [ ] Update `visible_collection_items(...)` only if needed to mirror runtime clamping exactly for negative starts, zero counts, and overscan larger than the item set.
- [ ] Add or update editor tests in `pane_component_projection/mod.rs` for negative visible start, zero visible count, oversized overscan, clamped page metadata, and world-space metadata projection.
- [ ] Ensure projection tests construct `SlintUiHostNodeProjection` with attributes that match the Runtime UI retained property names: `viewport_start`, `viewport_count`, `overscan`, `total_count`, `page_index`, `page_size`, `page_count`, `world_position`, `world_rotation`, `world_scale`, `world_size`, `pixels_per_meter`, `billboard`, `depth_test`, `render_order`, and `camera_target`.

### Testing Stage: Editor Projection Gate

Run targeted formatting for changed editor projection files:

```powershell
rustfmt --edition 2021 --check zircon_editor/src/ui/slint_host/ui/pane_data_conversion/pane_component_projection/mod.rs
```

Run focused editor projection tests:

```powershell
cargo test -p zircon_editor --lib runtime_component_projection --locked --jobs 1 --target-dir target\codex-runtime-ui-complex-components --message-format short --color never
```

Run scoped editor type check:

```powershell
cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir target\codex-runtime-ui-complex-components --message-format short --color never
```

### Debug / Correction Loop

- If projection tests fail on visible items, compare expected indices to the runtime reducer's normalized `viewport_start`, `viewport_count`, and `overscan` values.
- If editor compile fails in asset-editor watcher/session files, do not edit those files; coordinate with the active UI Asset Editor session note.

### Exit Evidence

- Editor projection tests prove complex-component metadata survives the generic host contract.
- Editor type check passes or an unrelated active-session blocker is recorded with the exact compiler diagnostic.

## Milestone 3: Documentation And Closeout

- Goal: Record the complex-component contract, touched files, validation evidence, and remaining blockers.
- In-scope behaviors: docs update, coordination note update, whitespace checks, and final risk report.
- Dependencies: Milestones 1 and 2 implementation slices.

### Implementation Slices

- [ ] Update `docs/ui-and-layout/runtime-ui-component-showcase.md` frontmatter and body with complex-component event/reducer behavior, projection behavior, tests, and validation commands.
- [ ] Update `.codex/sessions/20260429-0719-runtime-ui-showcase-schema-panel.md` with current step, touched modules, validation evidence, blockers, and next update.
- [ ] Keep `docs/superpowers/specs/2026-05-01-runtime-ui-complex-components-design.md` and this plan listed as plan sources in the docs header.

### Testing Stage: Documentation Gate

Run whitespace checks for touched docs and Rust files:

```powershell
git diff --check -- docs/superpowers/specs/2026-05-01-runtime-ui-complex-components-design.md docs/superpowers/plans/2026-05-01-runtime-ui-complex-components.md docs/ui-and-layout/runtime-ui-component-showcase.md .codex/sessions/20260429-0719-runtime-ui-showcase-schema-panel.md zircon_runtime/src/ui/component/event.rs zircon_runtime/src/ui/component/state.rs zircon_runtime/src/ui/component/catalog/editor_showcase.rs zircon_runtime/src/ui/tests/component_catalog.rs zircon_editor/src/ui/slint_host/ui/pane_data_conversion/pane_component_projection/mod.rs
```

Run final focused acceptance commands from Milestones 1 and 2 after fixes settle.

### Exit Evidence

- Docs list related code, implementation files, plan sources, tests, and validation evidence.
- Session note records active blockers and avoids claiming workspace-wide green unless the workspace validator actually ran and passed.
