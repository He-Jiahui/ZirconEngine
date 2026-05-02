# Runtime UI Real Data Source Adapter Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a shared Runtime UI component data-source/action adapter contract and prove it with showcase and inspector-backed editor adapters.

**Architecture:** The shared contract lives in `zircon_runtime::ui::component::data_binding` and reuses existing typed `UiComponentEvent`, `UiValue`, and validation types. Editor-owned adapters live under `zircon_editor::ui::template_runtime::component_adapter`, wrap the existing showcase reducer, and route inspector value/commit events into real `EditorState` draft fields without moving editor concrete types into runtime.

**Tech Stack:** Rust workspace on `main`, `serde`, `thiserror`, existing `zircon_runtime::ui::component` contracts, existing `zircon_editor` template runtime and inspector binding dispatch, milestone-first validation cadence.

---

## File Structure

Create shared runtime contract files:

- `zircon_runtime/src/ui/component/data_binding/mod.rs`: structural re-export module.
- `zircon_runtime/src/ui/component/data_binding/binding_target.rs`: `UiComponentBindingTarget` declaration and constructors.
- `zircon_runtime/src/ui/component/data_binding/event_envelope.rs`: `UiComponentEventEnvelope` declaration and builders.
- `zircon_runtime/src/ui/component/data_binding/projection_patch.rs`: `UiComponentProjectionPatch` declaration and patch helpers.
- `zircon_runtime/src/ui/component/data_binding/adapter_result.rs`: `UiComponentAdapterResult` declaration and result helpers.
- `zircon_runtime/src/ui/component/data_binding/adapter_error.rs`: `UiComponentAdapterError` declaration.

Modify structural runtime exports and tests:

- `zircon_runtime/src/ui/component/mod.rs`: add `mod data_binding;` and public re-exports only.
- `zircon_runtime/src/ui/tests/component_catalog.rs`: add `mod data_binding;` only.
- `zircon_runtime/src/ui/tests/component_catalog/data_binding.rs`: focused unit tests for envelope, error, and projection patch behavior.

Create editor adapter files:

- `zircon_editor/src/ui/template_runtime/component_adapter/mod.rs`: structural re-export module.
- `zircon_editor/src/ui/template_runtime/component_adapter/inspector.rs`: inspector adapter from `UiComponentEventEnvelope` to `EditorState` draft fields.
- `zircon_editor/src/ui/template_runtime/component_adapter/registry.rs`: small domain dispatcher for editor adapters.
- `zircon_editor/src/ui/template_runtime/component_adapter/showcase.rs`: showcase adapter wrapper around existing retained showcase state.

Modify editor runtime and host wiring:

- `zircon_editor/src/ui/template_runtime/mod.rs`: add `pub(crate) mod component_adapter;` only.
- `zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs`: route `apply_showcase_demo_binding` through the showcase adapter wrapper.
- `zircon_editor/src/ui/host/editor_event_runtime_access.rs`: add a crate-local method that locks `EditorState`, dispatches the adapter envelope, refreshes reflection on success, and returns `UiComponentAdapterResult`.
- `zircon_editor/src/ui/slint_host/app/inspector.rs`: route `dispatch_inspector_control_changed` through the adapter envelope for migrated inspector fields; leave click/batch dispatch unchanged.

Modify editor tests:

- `zircon_editor/src/tests/ui/mod.rs`: add `mod component_adapter;` only.
- `zircon_editor/src/tests/ui/component_adapter.rs`: tests for inspector real-data adapter behavior.
- `zircon_editor/src/tests/host/template_runtime/component_showcase_state.rs`: add a narrow assertion that showcase binding dispatch now returns adapter result evidence through the runtime wrapper.

Update docs:

- `docs/ui-and-layout/runtime-ui-component-showcase.md`: include the new shared adapter module in frontmatter and document the showcase/inspector adapter path plus validation commands.
- `docs/superpowers/specs/2026-05-01-runtime-ui-real-data-source-adapter-design.md`: no required behavior change unless implementation materially diverges.

Update coordination:

- `.codex/sessions/20260501-0236-runtime-ui-real-data-source-design.md`: update current step, touched modules, checks, and blockers as implementation proceeds.

## Milestone 1: Shared Adapter Contract

### Implementation Slices

- [ ] Create `zircon_runtime/src/ui/component/data_binding/binding_target.rs` with:

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct UiComponentBindingTarget {
    pub domain: String,
    pub subject: Option<String>,
    pub path: String,
}

impl UiComponentBindingTarget {
    pub fn new(domain: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            domain: domain.into(),
            subject: None,
            path: path.into(),
        }
    }

    pub fn with_subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    pub fn inspector(subject: impl Into<String>, field_path: impl Into<String>) -> Self {
        Self::new("inspector", field_path).with_subject(subject)
    }

    pub fn showcase(control_id: impl Into<String>) -> Self {
        let control_id = control_id.into();
        Self::new("showcase", control_id)
    }
}
```

- [ ] Create `event_envelope.rs` with `UiComponentEventEnvelope` carrying `document_id`, `control_id`, `component_id`, `target`, `event_kind`, `event`, and `source`. The constructor must set `event_kind` from `event.kind()`.

```rust
use serde::{Deserialize, Serialize};

use super::UiComponentBindingTarget;
use crate::ui::component::{UiComponentEvent, UiComponentEventKind, UiDragSourceMetadata};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiComponentEventEnvelope {
    pub document_id: String,
    pub control_id: String,
    pub component_id: Option<String>,
    pub target: UiComponentBindingTarget,
    pub event_kind: UiComponentEventKind,
    pub event: UiComponentEvent,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<UiDragSourceMetadata>,
}

impl UiComponentEventEnvelope {
    pub fn new(
        document_id: impl Into<String>,
        control_id: impl Into<String>,
        target: UiComponentBindingTarget,
        event: UiComponentEvent,
    ) -> Self {
        let event_kind = event.kind();
        Self {
            document_id: document_id.into(),
            control_id: control_id.into(),
            component_id: None,
            target,
            event_kind,
            event,
            source: None,
        }
    }

    pub fn with_component_id(mut self, component_id: impl Into<String>) -> Self {
        self.component_id = Some(component_id.into());
        self
    }

    pub fn with_source(mut self, source: UiDragSourceMetadata) -> Self {
        self.source = Some(source);
        self
    }
}
```

- [ ] Create `projection_patch.rs` with data-only patch payloads using `UiValue`.

```rust
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::ui::component::UiValue;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiComponentProjectionPatch {
    pub control_id: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub attributes: BTreeMap<String, UiValue>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub state_values: BTreeMap<String, UiValue>,
}

impl UiComponentProjectionPatch {
    pub fn new(control_id: impl Into<String>) -> Self {
        Self {
            control_id: control_id.into(),
            attributes: BTreeMap::new(),
            state_values: BTreeMap::new(),
        }
    }

    pub fn with_attribute(mut self, key: impl Into<String>, value: UiValue) -> Self {
        self.attributes.insert(key.into(), value);
        self
    }

    pub fn with_state_value(mut self, key: impl Into<String>, value: UiValue) -> Self {
        self.state_values.insert(key.into(), value);
        self
    }
}
```

- [ ] Create `adapter_result.rs` and `adapter_error.rs` with structured result/error types.

```rust
use serde::{Deserialize, Serialize};

use super::UiComponentProjectionPatch;
use crate::ui::component::UiValidationState;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiComponentAdapterResult {
    pub changed: bool,
    pub refresh_projection: bool,
    pub status_text: Option<String>,
    pub validation: Option<UiValidationState>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub patches: Vec<UiComponentProjectionPatch>,
}

impl UiComponentAdapterResult {
    pub fn unchanged() -> Self {
        Self::default()
    }

    pub fn changed() -> Self {
        Self {
            changed: true,
            refresh_projection: true,
            ..Self::default()
        }
    }

    pub fn with_status(mut self, status_text: impl Into<String>) -> Self {
        self.status_text = Some(status_text.into());
        self
    }

    pub fn with_patch(mut self, patch: UiComponentProjectionPatch) -> Self {
        self.patches.push(patch);
        self
    }
}
```

```rust
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::ui::component::{UiComponentEventKind, UiValueKind};

#[derive(Clone, Debug, Error, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiComponentAdapterError {
    #[error("unsupported Runtime UI component target domain {domain}")]
    UnsupportedTargetDomain { domain: String },
    #[error("unsupported Runtime UI component target path {path} in domain {domain}")]
    UnsupportedTargetPath { domain: String, path: String },
    #[error("Runtime UI component target {domain}:{path} is missing source {source_name}")]
    MissingSource { domain: String, path: String, source_name: String },
    #[error("component target {domain}:{path} does not support event {event_kind:?}")]
    UnsupportedEvent { domain: String, path: String, event_kind: UiComponentEventKind },
    #[error("invalid value kind {actual:?} for component target {domain}:{path}; expected {expected:?}")]
    InvalidValueKind { domain: String, path: String, expected: UiValueKind, actual: UiValueKind },
    #[error("component target {domain}:{path} rejected input: {reason}")]
    RejectedInput { domain: String, path: String, reason: String },
    #[error("component target {domain}:{path} mutation failed: {reason}")]
    HostMutation { domain: String, path: String, reason: String },
}
```

- [ ] Create `mod.rs` that declares child modules and re-exports only.

- [ ] Update `zircon_runtime/src/ui/component/mod.rs` to re-export all new shared contract types.

### Tests To Add During Implementation

- [ ] Add `zircon_runtime/src/ui/tests/component_catalog/data_binding.rs` with tests named:
  - `component_event_envelope_preserves_typed_event_and_target`
  - `component_projection_patch_keeps_attribute_and_state_values_separate`
  - `component_adapter_error_reports_unsupported_target_without_editor_types`

### Milestone 1 Testing Stage

- [ ] Run `cargo test -p zircon_runtime --lib component_event_envelope_preserves_typed_event_and_target --locked --jobs 1 --target-dir target\codex-runtime-ui-real-data-source-adapter --message-format short --color never`.
- [ ] Run `cargo test -p zircon_runtime --lib component_projection_patch_keeps_attribute_and_state_values_separate --locked --jobs 1 --target-dir target\codex-runtime-ui-real-data-source-adapter --message-format short --color never`.

## Milestone 2: Editor Registry And Showcase Adapter

### Implementation Slices

- [ ] Create `zircon_editor/src/ui/template_runtime/component_adapter/showcase.rs` with a wrapper that resolves the existing showcase binding into an envelope before applying the existing retained state mutation. The public crate-local function should be:

```rust
pub(crate) fn apply_showcase_component_binding(
    state: &mut UiComponentShowcaseDemoState,
    binding: &EditorUiBinding,
    input: UiComponentShowcaseDemoEventInput,
) -> Result<UiComponentAdapterResult, UiComponentShowcaseDemoError>
```

- [ ] Refactor `UiComponentShowcaseDemoState::apply_binding(...)` into a helper that either delegates to the wrapper or exposes an internal `apply_component_event_envelope(...)`. Preserve existing public test helpers and existing behavior.

- [ ] Update `EditorUiHostRuntime::apply_showcase_demo_binding(...)` to return `UiComponentAdapterResult` instead of `()` and route through the showcase adapter.

- [ ] Update call sites in `zircon_editor/src/ui/slint_host/app/pane_surface_actions.rs` and tests to accept the result and keep existing status text behavior.

- [ ] Create `zircon_editor/src/ui/template_runtime/component_adapter/registry.rs` with a small dispatcher that checks `envelope.target.domain` and routes `inspector` to the inspector adapter. Leave `showcase` handled by the showcase wrapper because it needs existing demo input conversion.

### Tests To Add During Implementation

- [ ] Extend `component_showcase_state.rs` so at least one existing helper captures the returned `UiComponentAdapterResult` and asserts `changed == true`, `refresh_projection == true`, and that a patch references the changed control id.

### Milestone 2 Testing Stage

- [ ] Run `cargo test -p zircon_editor --lib showcase_demo_state_applies_projected_bindings_to_retained_values_and_log --locked --jobs 1 --target-dir target\codex-runtime-ui-real-data-source-adapter --message-format short --color never`.

## Milestone 3: Inspector Real Data Adapter

### Implementation Slices

- [ ] Create `zircon_editor/src/ui/template_runtime/component_adapter/inspector.rs` with `apply_inspector_component_envelope(state: &mut EditorState, envelope: &UiComponentEventEnvelope) -> Result<UiComponentAdapterResult, UiComponentAdapterError>`.

- [ ] In `apply_inspector_component_envelope`, require `envelope.target.domain == "inspector"`, require `envelope.target.subject == Some("entity://selected")`, and support only `ValueChanged` and `Commit` events for the first slice. Missing-subject errors use the Rust field name `source_name` because `thiserror` treats a field literally named `source` as an error source.

- [ ] Convert event values to field strings with a helper that accepts `UiValue::String`, `UiValue::Int`, and `UiValue::Float`, then rejects other kinds with `UiComponentAdapterError::InvalidValueKind`.

- [ ] Reuse `crate::ui::binding_dispatch::inspector::apply_inspector_draft_field` so the adapter mutates the same draft state as existing inspector dispatch.

- [ ] Return a `UiComponentAdapterResult::changed()` with a `UiComponentProjectionPatch` that sets `value` and the target field path to the submitted value.

- [ ] Add `EditorEventRuntime::dispatch_ui_component_adapter_event(...)` to `editor_event_runtime_access.rs`. It should lock the runtime state, dispatch through `EditorUiComponentAdapterRegistry`, refresh reflection only when `result.refresh_projection` is true, and return the adapter result.

- [ ] Update `SlintEditorHost::dispatch_inspector_control_changed(...)` to build a `UiComponentEventEnvelope` with target `UiComponentBindingTarget::inspector("entity://selected", field_id)` and event `UiComponentEvent::ValueChanged { property: "value", value: UiValue::String(value.to_string()) }`, then call the runtime adapter method. On success set status from result or `Inspector field updated: {field_id}` and mark presentation dirty when requested. On error set the status line and do not call the old binding dispatch for migrated fields.

### Tests To Add During Implementation

- [ ] Add `zircon_editor/src/tests/ui/component_adapter.rs` with tests named:
  - `inspector_component_adapter_value_changed_updates_selected_name_draft`
  - `inspector_component_adapter_commit_updates_transform_draft`
  - `inspector_component_adapter_rejects_missing_selection_without_mutation`
  - `inspector_component_adapter_rejects_unsupported_field`

### Milestone 3 Testing Stage

- [ ] Run `cargo test -p zircon_editor --lib inspector_component_adapter --locked --jobs 1 --target-dir target\codex-runtime-ui-real-data-source-adapter --message-format short --color never`.
- [ ] Run `cargo test -p zircon_editor --lib inspector --locked --jobs 1 --target-dir target\codex-runtime-ui-real-data-source-adapter --message-format short --color never`.
- [ ] Run `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir target\codex-runtime-ui-real-data-source-adapter --message-format short --color never`.

## Milestone 4: Documentation And Final Validation

### Implementation Slices

- [ ] Update `docs/ui-and-layout/runtime-ui-component-showcase.md` frontmatter to include the new runtime data-binding files, editor component adapter files, and new tests.

- [ ] Add a section explaining the shared adapter envelope, showcase adapter wrapper, and inspector-backed real data-source proof.

- [ ] Update `.codex/sessions/20260501-0236-runtime-ui-real-data-source-design.md` with implementation files, tests, blockers, and validation evidence.

### Milestone 4 Testing Stage

- [ ] Run targeted `rustfmt --edition 2021 --check` for touched Rust files.
- [ ] Run `git diff --check --` for touched files.
- [ ] If earlier focused tests pass, run `cargo test -p zircon_runtime --lib ui::tests::component_catalog --locked --jobs 1 --target-dir target\codex-runtime-ui-real-data-source-adapter --message-format short --color never`.
- [ ] If editor focused tests pass, run `cargo test -p zircon_editor --lib component_showcase --locked --jobs 1 --target-dir target\codex-runtime-ui-real-data-source-adapter --message-format short --color never`.

## Self-Review Checklist

- [ ] Shared runtime files contain no `zircon_editor` imports.
- [ ] Root `mod.rs` files are structural only.
- [ ] Showcase still uses existing retained reducer semantics and existing tests remain meaningful.
- [ ] Inspector adapter uses `EditorState` authority through existing draft mutation, not direct Slint state.
- [ ] No `asset_editor_sessions/**`, graphics/RHI/render graph, or plugin files are touched.
- [ ] Docs list every new implementation file and test file.
