# UI Debug Observatory Acceptance

Plan source: `docs/superpowers/plans/2026-05-07-debug-observatory-m0-m1.md`
Design source: `docs/superpowers/specs/2026-05-07-debug-observatory-design.md`

## Scope

- M0 records the current Debug Reflector baseline and active coordination constraints.
- M1 connects Runtime Diagnostics pane payload construction to an explicitly supplied active `UiSurfaceDebugSnapshot`.
- M1 preserves the existing no-active-surface fallback when no snapshot is supplied.
- M1 does not implement timeline, hit-test explanation expansion, invalidation overlays, diff, replay, or property editing.

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

## Remaining Out Of Scope

- Snapshot timeline/history.
- Detailed hit-test explanation beyond current snapshot projection.
- Invalidation/damage cause overlays beyond existing snapshot fields.
- Snapshot diff/export package/replay.
- Guarded property editing.
