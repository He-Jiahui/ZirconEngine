# Editor Extension Registration Atomicity

## Scope
- `EditorEventRuntime::register_editor_extension(...)` must reject invalid extension contributions before mutating live editor operation discovery or Workbench view state.
- Operation descriptors and auto-generated View open operations are first merged into a scratch `EditorOperationRegistry`.
- Extension views are preflighted against the Workbench view registry before the scratch registry is committed to live runtime state.
- Already-registered extension drawer ids, menu paths, component drawer component types, and UI template ids are treated as live contribution ids, so later extensions cannot publish conflicting metadata under the same id.

## Test Inventory
- Negative case: `editor_runtime_rejects_duplicate_extension_view_without_registering_operations` first registers a plugin view, then attempts a second extension with the same view id plus a new operation contribution.
- The second registration must return the duplicate view diagnostic and the contributed operation must not appear in `ListOperations`.
- Negative case: `editor_runtime_rejects_duplicate_extension_menu_paths_without_registering_operations` first registers a plugin menu path, then attempts a second extension with the same menu path plus a new operation contribution.
- The duplicate menu registration must return the duplicate menu diagnostic and the contributed operation must not appear in `ListOperations`.
- Existing positive coverage continues through `editor_runtime_registers_plugin_views_as_activity_descriptors`, `editor_runtime_projects_plugin_views_into_view_menu_operations`, and plugin menu/component-drawer registration tests.

## Results
- Passed static gate: `rustfmt --edition 2021 --check zircon_editor/src/ui/host/editor_extension_views.rs zircon_editor/src/ui/host/editor_extension_registration.rs zircon_editor/src/tests/editor_event/runtime.rs zircon_editor/src/core/editor_operation.rs zircon_editor/src/core/editor_extension.rs zircon_app/src/entry/entry_runner/editor.rs`.
- Passed static gate: scoped `git diff --check` for extension registration/view code, runtime regression, docs, acceptance notes, and active session note; only LF/CRLF conversion warnings were reported.
- Passed static gate after cross-extension contribution checks: `rustfmt --edition 2021 --check zircon_editor/src/ui/host/editor_extension_registration.rs zircon_editor/src/ui/host/editor_extension_views.rs zircon_editor/src/tests/editor_event/runtime.rs zircon_editor/src/core/editor_operation.rs zircon_editor/src/core/editor_extension.rs zircon_app/src/entry/entry_runner/editor.rs`.
- Passed static gate after cross-extension contribution checks: scoped `git diff --check` for extension registration/view code, runtime regressions, docs, acceptance notes, and active session note; only LF/CRLF conversion warnings were reported.
- Pending focused test: `cargo test -p zircon_editor --lib editor_runtime_rejects_duplicate_extension_view_without_registering_operations --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-extension-registration-atomicity --message-format short --color never -- --test-threads=1 --nocapture`.
- Pending focused test: `cargo test -p zircon_editor --lib editor_runtime_rejects_duplicate_extension_menu_paths_without_registering_operations --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-extension-registration-atomicity --message-format short --color never -- --test-threads=1 --nocapture`.
- Deferred Cargo execution: active runtime/UI and physics Cargo jobs are currently compiling in other coordination lanes, so this slice keeps Cargo focused tests pending instead of adding another writer.

## Acceptance Decision
- Extension registration now uses scratch operation registration, contribution-id conflict checks, and view preflight to prevent failed extension registration from polluting live operation discovery.
- Final acceptance remains pending until the focused editor test can execute after active UI/runtime-interface Cargo blockers clear.
