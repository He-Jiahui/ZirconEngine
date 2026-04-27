# Editor Operation Core Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first working slice of path-addressed editor operations so menus, remote control, CLI, and future ui.toml component editors can submit named actions through one editor runtime path.

**Architecture:** `zircon_editor::core::editor_operation` owns operation identity, descriptors, registry, requests, and journal trace metadata. `EditorEventRuntime` keeps owning execution and state mutation; an operation resolves to a canonical `EditorEvent`, then existing dispatch, journal, reflection, and undo policy handling run unchanged. `UndoableEditorOperation` is introduced as explicit metadata for operations that will participate in the operation history stack, while the first slice records undoability in the journal trace without replacing the existing scene `EditorHistory` yet.

**Tech Stack:** Rust, serde, existing `zircon_editor::core::editor_event`, existing `zircon_runtime::ui` control-plane types, Cargo tests.

---

## File Structure

- Create `zircon_editor/src/core/editor_operation/mod.rs`
  - Navigational module only, exporting focused child modules.
- Create `zircon_editor/src/core/editor_operation/path.rs`
  - `EditorOperationPath` validation and display helpers for `XXX.YYY.ZZZ` operation names.
- Create `zircon_editor/src/core/editor_operation/descriptor.rs`
  - `EditorOperationDescriptor`, `EditorOperationAction`, and `UndoableEditorOperation`.
- Create `zircon_editor/src/core/editor_operation/registry.rs`
  - `EditorOperationRegistry` with duplicate detection, lookup, and builtin operation registration.
- Create `zircon_editor/src/core/editor_operation/request.rs`
  - `EditorOperationRequest`, `EditorOperationTrace`, and `EditorOperationError`.
- Modify `zircon_editor/src/core/mod.rs`
  - Expose the new `editor_operation` module beside `editor_event`.
- Modify `zircon_editor/src/core/editor_event/types.rs`
  - Add optional operation trace metadata to `EditorEventRecord`, using serde defaults so older journals remain readable.
- Modify `zircon_editor/src/core/editor_event/runtime/editor_event_runtime_inner.rs`
  - Store an `EditorOperationRegistry` in runtime state.
- Modify `zircon_editor/src/core/editor_event/runtime/editor_event_dispatcher.rs`
  - Add `dispatch_operation(...)` to the runtime dispatcher contract.
- Modify `zircon_editor/src/ui/host/editor_event_dispatch.rs`
  - Resolve operation requests into canonical events and record operation trace metadata in the journal.
- Modify `zircon_editor/src/tests/editor_event/runtime.rs`
  - Add tests proving operation dispatch creates the same editor event as direct menu binding and stores the operation path in the journal.
- Update `docs/editor-and-tooling/ui-binding-reflection-architecture.md`
  - Document operation dispatch as the new path-addressed layer above canonical editor events.
- Update `docs/editor-and-tooling/index.md`
  - Link the operation dispatch documentation.

## Task 1: Operation Types And Registry

**Files:**
- Create: `zircon_editor/src/core/editor_operation/mod.rs`
- Create: `zircon_editor/src/core/editor_operation/path.rs`
- Create: `zircon_editor/src/core/editor_operation/descriptor.rs`
- Create: `zircon_editor/src/core/editor_operation/registry.rs`
- Create: `zircon_editor/src/core/editor_operation/request.rs`
- Modify: `zircon_editor/src/core/mod.rs`
- Test: `zircon_editor/src/tests/editor_event/runtime.rs`

- [ ] **Step 1: Write the failing registry test**

Add this test to `zircon_editor/src/tests/editor_event/runtime.rs`:

```rust
#[test]
fn editor_operation_registry_exposes_builtin_menu_operations_by_path() {
    use crate::core::editor_operation::{EditorOperationPath, EditorOperationRegistry};

    let registry = EditorOperationRegistry::with_builtin_operations();
    let reset_path = EditorOperationPath::parse("Window.Layout.Reset").unwrap();
    let reset = registry
        .descriptor(&reset_path)
        .expect("reset layout operation should be registered");

    assert_eq!(reset.path().as_str(), "Window.Layout.Reset");
    assert_eq!(reset.display_name(), "Reset Layout");
    assert_eq!(reset.menu_path(), Some("Window/Reset Layout"));
    assert!(reset.callable_from_remote());
    assert!(reset.undoable().is_some());
}
```

- [ ] **Step 2: Run the red test**

Run:

```powershell
cargo test -p zircon_editor editor_operation_registry_exposes_builtin_menu_operations_by_path --locked -- --nocapture
```

Expected: compile failure because `core::editor_operation` does not exist.

- [ ] **Step 3: Add operation path, descriptor, request, and registry types**

Implement:

```rust
pub struct EditorOperationPath(String);
pub struct EditorOperationDescriptor { ... }
pub struct UndoableEditorOperation { ... }
pub enum EditorOperationAction { StaticEvent(EditorEvent) }
pub struct EditorOperationRegistry { descriptors: BTreeMap<EditorOperationPath, EditorOperationDescriptor> }
pub struct EditorOperationRequest { path: EditorOperationPath, arguments: serde_json::Value }
pub struct EditorOperationTrace { path: EditorOperationPath, display_name: String, undoable_name: Option<String> }
pub enum EditorOperationError { InvalidPath(String), DuplicatePath(String), UnknownOperation(String) }
```

Register at least:

```rust
Window.Layout.Reset -> MenuAction::ResetLayout
Window.Layout.Save -> MenuAction::SaveLayout
File.Project.Open -> MenuAction::OpenProject
File.Project.Save -> MenuAction::SaveProject
Edit.Undo -> MenuAction::Undo
Edit.Redo -> MenuAction::Redo
View.Scene.Open -> MenuAction::OpenView("editor.scene")
View.Game.Open -> MenuAction::OpenView("editor.game")
```

- [ ] **Step 4: Run the registry test**

Run:

```powershell
cargo test -p zircon_editor editor_operation_registry_exposes_builtin_menu_operations_by_path --locked -- --nocapture
```

Expected: pass.

## Task 2: Runtime Operation Dispatch

**Files:**
- Modify: `zircon_editor/src/core/editor_event/types.rs`
- Modify: `zircon_editor/src/core/editor_event/runtime/editor_event_runtime_inner.rs`
- Modify: `zircon_editor/src/core/editor_event/runtime/editor_event_dispatcher.rs`
- Modify: `zircon_editor/src/ui/host/editor_event_dispatch.rs`
- Test: `zircon_editor/src/tests/editor_event/runtime.rs`

- [ ] **Step 1: Write the failing dispatch test**

Add this test:

```rust
#[test]
fn dispatching_operation_path_records_trace_and_reuses_canonical_event() {
    let _guard = env_lock().lock().unwrap();

    let runtime = EventRuntimeHarness::new("zircon_editor_operation_dispatch");
    let record = runtime
        .runtime
        .dispatch_operation(
            EditorEventSource::Headless,
            crate::core::editor_operation::EditorOperationRequest::parse("Window.Layout.Reset")
                .unwrap(),
        )
        .expect("operation dispatch should execute");

    assert_eq!(record.event, EditorEvent::WorkbenchMenu(MenuAction::ResetLayout));
    let operation = record.operation.expect("operation trace should be recorded");
    assert_eq!(operation.path.as_str(), "Window.Layout.Reset");
    assert_eq!(operation.display_name, "Reset Layout");
    assert_eq!(operation.undoable_name.as_deref(), Some("Reset Layout"));
    assert!(record.effects.contains(&EditorEventEffect::LayoutChanged));
}
```

- [ ] **Step 2: Run the red test**

Run:

```powershell
cargo test -p zircon_editor dispatching_operation_path_records_trace_and_reuses_canonical_event --locked -- --nocapture
```

Expected: compile failure because `dispatch_operation` and `EditorEventRecord.operation` do not exist.

- [ ] **Step 3: Implement runtime operation dispatch**

Add `operation_registry` to `EditorEventRuntimeInner`, initialize it with `EditorOperationRegistry::with_builtin_operations()`, and route `dispatch_operation` through the same normalized dispatch path used by events and bindings.

- [ ] **Step 4: Run the dispatch test**

Run:

```powershell
cargo test -p zircon_editor dispatching_operation_path_records_trace_and_reuses_canonical_event --locked -- --nocapture
```

Expected: pass.

## Task 3: Documentation And Focused Validation

**Files:**
- Modify: `docs/editor-and-tooling/ui-binding-reflection-architecture.md`
- Modify: `docs/editor-and-tooling/index.md`

- [ ] **Step 1: Document the operation layer**

Add a section explaining that `EditorOperation` is the path-addressed public action layer, while `EditorEvent` remains the canonical replay/event layer.

- [ ] **Step 2: Run focused tests**

Run:

```powershell
cargo test -p zircon_editor editor_operation --locked -- --nocapture
cargo test -p zircon_editor dispatching_operation_path_records_trace_and_reuses_canonical_event --locked -- --nocapture
```

Expected: pass.

- [ ] **Step 3: Run crate validation**

Run:

```powershell
cargo test -p zircon_editor --locked
```

Expected: pass, or report pre-existing unrelated failures with exact error lines.

## Self-Review

- Spec coverage: this first slice covers path-named editor operations, operation dispatch as editor submission, journal trace metadata, undoable naming metadata, and headless/remote callable semantics. View/Drawer extension catalogs and component custom editors are intentionally second-slice work after the operation core is stable.
- Placeholder scan: no task uses TBD, TODO, or unspecified tests.
- Type consistency: `EditorOperationPath`, `EditorOperationRegistry`, `EditorOperationRequest`, `EditorOperationTrace`, and `dispatch_operation` are named consistently across tests and implementation steps.
