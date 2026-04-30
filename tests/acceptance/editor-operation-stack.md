# Editor Operation Stack

## Scope
- Changed `EditorOperationStack` so successful built-in Undo/Redo operations move existing named operation entries between undo and redo stacks.
- Added operation group metadata so repeated continuous invocations with the same operation id and group collapse into one named stack entry while every dispatch remains independently journaled.
- Affected layers: editor operation registry, editor event dispatch, journal/operation stack query, and editor event runtime tests.

## Baseline
- Before this slice, built-in `Edit.History.Undo` and `Edit.History.Redo` were declared with `UndoableEditorOperation`, so invoking them through the operation layer could add Undo/Redo themselves to the named stack instead of moving the previous operation entry.
- The workspace had multiple active unrelated Cargo/Rustc sessions during validation, mainly runtime/editor/app checks owned by other coordination sessions.

## Test Inventory
- Focused positive case: create cube through `Scene.Node.CreateCube`, invoke `Edit.History.Undo`, then invoke `Edit.History.Redo`; assert stack transitions `undo -> redo -> undo`.
- Boundary case: Undo/Redo are non-undoable dispatcher commands and must not create stack entries for themselves.
- Query case: `EditorOperationControlRequest::QueryOperationStack` must expose the moved operation entry after redo.
- Source metadata case: stack entries record the original `EditorEventSource`, query responses expose it, and Undo/Redo movement preserves it.
- Operation group case: two undoable invocations with the same `operation_group` should produce two journal records but one undo stack entry whose sequence points at the latest grouped dispatch.
- Formatting/static checks: touched Rust files must pass `rustfmt --edition 2021 --check`; touched files must pass `git diff --check`.

## Tooling Evidence
- Windows Cargo 1.94.1 and Rust 1.94.1 were available, but package cache locks from other active sessions repeatedly blocked or interrupted focused validation.
- WSL Ubuntu 22.04 had Cargo 1.94.1 and Rust 1.94.1 available. A WSL focused run was attempted with an isolated `/tmp` target directory to bypass Windows Cargo locks.
- Operation-group validation used WSL with `CARGO_TARGET_DIR=/tmp/zircon-target-editor-listener-status`, `RUSTFLAGS="-C debuginfo=0"`, `--locked`, and `--jobs 1`.

## Results
- Passed: `rustfmt --edition 2021 --check zircon_editor/src/core/editor_operation.rs zircon_editor/src/ui/host/editor_event_dispatch.rs zircon_editor/src/ui/host/editor_operation_dispatch.rs zircon_editor/src/tests/editor_event/runtime.rs zircon_editor/src/ui/slint_host/ui/tests.rs`.
- Passed: `git diff --check -- zircon_editor/src/core/editor_operation.rs zircon_editor/src/ui/host/editor_event_dispatch.rs zircon_editor/src/ui/host/editor_operation_dispatch.rs zircon_editor/src/tests/editor_event/runtime.rs zircon_editor/src/ui/slint_host/ui/tests.rs docs/editor-and-tooling/ui-binding-reflection-architecture.md tests/acceptance/editor-operation-stack.md .codex/sessions/archive/20260428-1955-editor-operation-stack-source.md`.
- Passed: `cargo test -p zircon_editor --lib operation_stack_moves_entries_across_undo_and_redo_operations --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-component-drawer-bindings --message-format short -- --test-threads=1 --nocapture` with `1 passed; 0 failed`.
- Passed: `cargo test -p zircon_editor --lib tests::editor_event::runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-component-drawer-bindings -- --test-threads=1` with `42 passed; 0 failed`.
- Passed: `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-component-drawer-bindings` with `EXITCODE=0`; only existing warnings were reported for `editor_meta.rs::save` dead code and unused Runtime UI showcase variants.
- Fixed during validation: `zircon_editor/src/ui/slint_host/ui/tests.rs` now imports the sibling `pane_data_conversion` module so nested component showcase tests can resolve `super::pane_data_conversion`.
- TDD red: `cargo test -p zircon_editor --lib operation_stack_preserves_original_source_across_undo_and_redo --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-component-drawer-bindings --message-format short -- --test-threads=1 --nocapture` failed on the expected missing `EditorOperationStackEntry.source` field, then also surfaced unrelated active Runtime UI showcase compile errors.
- Retried: `cargo test -p zircon_editor --lib operation_stack_preserves_original_source_across_undo_and_redo --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-component-drawer-bindings --message-format short -- --test-threads=1 --nocapture` timed out after 15 minutes while compiling/linking the `zircon_editor` test profile; no Rust or assertion diagnostic was emitted, and no same-target cargo/rustc process remained afterward.
- Passed: direct execution of the current editor test binary `D:\cargo-targets\zircon-codex-editor-listener-audit-green\debug\deps\zircon_editor-86a81a58131e5a21.exe operation_stack_preserves_original_source_across_undo_and_redo --test-threads=1 --nocapture` with `1 passed; 0 failed; 879 filtered out`.
- Earlier blocked attempts: Windows focused runs hit package cache/build contention or timed out before final diagnostics; WSL focused run timed out during cold dependency compilation.
- TDD red: `CARGO_TARGET_DIR=/tmp/zircon-target-editor-listener-status RUSTFLAGS="-C debuginfo=0" cargo check -p zircon_editor --tests --locked --jobs 1 --message-format short --color never` failed on the expected missing `EditorOperationInvocation::with_operation_group` method and missing `EditorOperationStackEntry.operation_group` field.
- Fixed during validation: `zircon_editor/src/tests/host/slint_event_bridge/support.rs` now initializes `EditorEventRecord.operation_group` for the test-support record literal.
- Passed: `CARGO_TARGET_DIR=/tmp/zircon-target-editor-listener-status RUSTFLAGS="-C debuginfo=0" cargo test -p zircon_editor --lib operation_stack_merges_continuous_invocations_with_same_operation_group --locked --jobs 1 --message-format short --color never -- --test-threads=1 --nocapture` with `1 passed; 0 failed; 887 filtered out`.
- Passed: `CARGO_TARGET_DIR=/tmp/zircon-target-editor-listener-status RUSTFLAGS="-C debuginfo=0" cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`; only existing warnings were reported for `editor_meta.rs::save` dead code and unused Runtime UI showcase variants.

## Acceptance Decision
- Stack movement slice accepted.
- Source metadata preservation accepted for the current editor test binary: the focused source-preservation regression passed when run directly from the freshly linked `zircon_editor` test executable, while the Cargo wrapper retry remains classified as local compile/link queue timeout rather than a test failure.
- Operation group merging accepted on WSL/Linux evidence: grouped undoable operation invocations keep per-dispatch journal records but collapse to one `EditorOperationStack` entry and expose `operation_group` through stack query JSON.
- The focused stack movement regression and the broader editor event runtime suite both pass with the direct toolchain `RUSTC` path.
- Remaining risk: full workspace validation was not run because multiple unrelated active sessions were still building runtime, app, physics, editor chrome, and graphics targets.
