# UI Debug Observatory Acceptance

Plan source: `docs/superpowers/plans/2026-05-07-debug-observatory-m0-m1.md`
M2 plan source: `docs/superpowers/plans/2026-05-08-debug-observatory-m2.md`
Design source: `docs/superpowers/specs/2026-05-07-debug-observatory-design.md`

## Scope

- M0 records the current Debug Reflector baseline and active coordination constraints.
- M1 connects Runtime Diagnostics pane payload construction to an explicitly supplied active `UiSurfaceDebugSnapshot`.
- M1 preserves the existing no-active-surface fallback when no snapshot is supplied.
- M2 adds runtime-owned bounded snapshot timeline contracts/storage and registers the Debug Observatory Window-surface tool.
- M2 does not implement hit-test explanation expansion, invalidation overlays, diff, replay, or property editing.

## M0 Baseline

- Branch policy: work remains on `main`; no worktree or feature branch.
- Coordination: active Slate M8 and drawer/window/menu lanes were present during planning.
- Existing known risk: broad workspace validation can still expose sibling-owned dirty-tree failures.

## M1 Evidence

Implemented source changes:

- `PanePayloadBuildContext` can carry an active borrowed `UiSurfaceDebugSnapshot`.
- Runtime Diagnostics payload builder uses the active snapshot when present and preserves no-active fallback when absent.
- Runtime Diagnostics payload builder derives overlay primitives through `EditorUiDebugReflectorOverlayState` so the same shared overlay filters are used by pane payloads and tests.
- Host Runtime Diagnostics conversion marks active-snapshot payloads so the later body-surface Debug Reflector refresh does not overwrite the supplied shared snapshot data.
- Focused tests cover active snapshot projection, fallback behavior, template root attributes, payload overlay host conversion, and active-payload preservation through the production refresh seam.

Review correction:

- Code review found that `to_host_contract_pane(...)` converted payload-derived Runtime Diagnostics reflector rows/overlays and then unconditionally refreshed them from the pane's own body surface.
- The regression `runtime_diagnostics_body_refresh_preserves_active_payload_reflector` first failed with `active payload reflector should not be replaced`, then passed after adding the active-payload preservation flag.

Validation commands:

- `rustfmt --edition 2021 --check "zircon_editor/src/ui/workbench/debug_reflector/mod.rs" "zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_presentation.rs" "zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload.rs" "zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/runtime_diagnostics.rs" "zircon_editor/src/ui/slint_host/host_contract/data/panes.rs" "zircon_editor/src/ui/slint_host/ui/pane_data_conversion/runtime_diagnostics.rs" "zircon_editor/src/tests/host/pane_presentation.rs" "zircon_editor/src/tests/host/template_runtime/pane_payload_projection.rs" "zircon_editor/src/tests/host/slint_window/ui_debug_reflector.rs"`: passed with no output after applying `rustfmt` to the same touched Rust files.
- `cargo test -p zircon_editor --lib pane_payload_builders_emit_stable_body_metadata_for_first_wave_views --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-shared" --message-format short --color never`: passed, 1 test.
- `cargo test -p zircon_editor --lib runtime_diagnostics_payload_uses_active_ui_debug_snapshot_when_available --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-shared" --message-format short --color never`: passed, 1 test.
- `cargo test -p zircon_editor --lib pane_payload_projection --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-shared" --message-format short --color never`: passed, 2 tests.
- `cargo test -p zircon_editor --lib runtime_diagnostics_body_refresh_preserves_active_payload_reflector --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-shared" --message-format short --color never`: failed before the guard fix, then passed after the guard fix, 1 test.
- `cargo test -p zircon_editor --lib runtime_diagnostics_live_body_surface_populates_debug_reflector_rows_and_overlays --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-shared" --message-format short --color never`: passed, 1 test.
- `cargo test -p zircon_editor --lib ui_debug_reflector --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-shared" --message-format short --color never`: passed, 16 tests.
- `cargo test -p zircon_runtime --lib diagnostics --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-shared" --message-format short --color never`: passed, 17 tests.
- `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-shared" --message-format short --color never`: passed with existing warning noise.

Validation scope:

- Focused M1 package validation completed against `D:\cargo-targets\zircon-shared` with `--locked --jobs 1`.
- Full workspace build/test was not run in this M1 slice; active sibling Slate and drawer/window/menu lanes remain responsible for their broader validation scope.

## M2 Evidence

Implemented source changes:

- `zircon_runtime_interface::ui::surface::timeline` defines serializable timeline frame handles, summaries, retention metadata, and timeline snapshots.
- `zircon_runtime::ui::surface::UiDebugTimelineStore` retains authoritative `UiSurfaceDebugSnapshot` values in a bounded runtime-owned `VecDeque`, assigns stable handles, reports dropped frames, and keeps historical selection as timeline cursor state only.
- `zircon_editor::ui::workbench::debug_reflector::timeline` projects shared timeline snapshots into retention labels, selected/latest labels, previous/next handles, frame rows, and selected-frame reflector data.
- `editor.debug_observatory` is registered as a `Debug Observatory` `ActivityWindow` that reuses `RuntimeDiagnosticsV1` payloads and the diagnostics route namespace.
- Window menu, editor operation registry, host fallback menu bindings, and reflection naming now expose `OpenView.editor.debug_observatory` through `Window.DebugObservatory.Open`.
- Focused tests cover runtime retention/selection/no-mutation behavior, editor timeline read-model projection, built-in descriptor registration, Window menu projection, host menu dispatch binding, capability gating, pane template classification, and operation registry exposure.

Validation commands:

- `rustfmt --edition 2021 --check` over the M2 touched Rust files: initially reported formatting diffs and one typo in the command path; after formatting the M2 touched files and rerunning with the corrected path, passed with no output.
- `cargo check -p zircon_runtime_interface --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-shared" --message-format short --color never`: passed, validating the shared timeline DTO crate.
- `cargo test -p zircon_runtime --lib timeline --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-shared" --message-format short --color never`: passed, 3 tests.
- `cargo test -p zircon_editor --lib debug_observatory --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-shared" --message-format short --color never`: blocked before editor tests by sibling-owned runtime diagnostics/log compile drift, `zircon_runtime/src/diagnostic_log/sink.rs:84` borrow of moved `filter`.
- `cargo test -p zircon_editor --lib ui_debug_timeline --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-shared" --message-format short --color never`: blocked by the same sibling-owned `zircon_runtime/src/diagnostic_log/sink.rs:84` compile drift.
- `cargo test -p zircon_editor --lib builtin_activity_windows_expose_window_template_documents --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-shared" --message-format short --color never`: blocked by the same sibling-owned `zircon_runtime/src/diagnostic_log/sink.rs:84` compile drift.
- `cargo test -p zircon_editor --lib workbench_view_model_projects_menu_strip_drawers_and_status --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-shared" --message-format short --color never`: blocked before the filtered test by sibling-owned editor host presentation drift, `zircon_editor/src/ui/slint_host/ui/apply_presentation.rs:136` missing `text_input_focus` in `HostWindowPresentationData` initializer.
- `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-shared" --message-format short --color never`: blocked before editor lib acceptance by sibling-owned runtime asset facade drift, `zircon_runtime/src/asset/facade/event.rs:96` missing `crossbeam_channel::Sender::is_disconnected`.

Validation scope:

- M2.T validation is scoped to runtime timeline storage and editor Debug Observatory registration. Runtime interface DTOs and runtime timeline storage passed. Editor-side tests could not execute to completion because active sibling lanes currently block `zircon_editor` compilation before M2 filters run.
- Full workspace build/test was not run; current blockers are attributed to active peripheral diagnostics/log, asset facade, and editor input/host presentation lanes rather than Debug Observatory M2.

## Remaining Out Of Scope

- Detailed hit-test explanation beyond current snapshot projection.
- Invalidation/damage cause overlays beyond existing snapshot fields.
- Snapshot diff/export package/replay.
- Guarded property editing.
