---
related_code:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/native_windows.rs
  - zircon_editor/src/ui/retained_host/app/native_windows/target.rs
  - zircon_editor/src/ui/retained_host/app/native_windows/presentation.rs
  - zircon_editor/src/ui/retained_host/app/native_windows/store.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/native_window_presenters.rs
  - zircon_editor/src/ui/retained_host/app/native_window_close.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/app/native_windows.rs
  - zircon_editor/src/ui/retained_host/app/native_windows/target.rs
  - zircon_editor/src/ui/retained_host/app/native_windows/presentation.rs
  - zircon_editor/src/ui/retained_host/app/native_windows/store.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - app native-window target/presentation/store ownership scan
  - app host-lifecycle native-window presenter subowner ownership scan
  - scoped git diff --check
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Retained Host Native Window Presenters

`app/native_windows.rs` is the structural entry for native floating window presenter support. It exposes the native floating-window target DTO, target collection helper, per-window presenter store, and presentation applicator used by retained-host lifecycle code.

## Target Collection

`app/native_windows/target.rs` owns `NativeFloatingWindowTarget` and `collect_native_floating_window_targets(...)`. It reads the Workbench model plus the floating-window projection bundle and only returns floating windows whose projection frames request native host presentation and have a runtime surface tree id.

## Presentation

`app/native_windows/presentation.rs` owns `configure_native_floating_window_presentation(...)`. It writes native floating-window mode, window id, surface tree id, title, and bounds into the host presentation, mirrors the same data into native floating surface data, and applies the platform window position and size.

## Store

`app/native_windows/store.rs` owns `NativeWindowPresenterStore`. It creates missing `UiHostWindow` instances, registers the default keep-shown close policy before host callbacks are attached, hides stale windows whose targets disappeared, applies per-target presentation updates, and exposes test/window lookup helpers.

## Boundary Rules

- Keep `app/native_windows.rs` as a structural module entry and re-export point only.
- Re-export `NativeFloatingWindowTarget` from `app/native_windows.rs` so lifecycle child modules can type native presenter callback and presentation helpers without reaching into `native_windows/target.rs`.
- Keep Workbench/projection target filtering in `target.rs`.
- Keep host-presentation and platform window position/size writes in `presentation.rs`.
- Keep presenter lifecycle, stale-window hiding, callback creation hook invocation, and window lookup in `store.rs`.
- Keep close-request policy and dirty document prompts in `app/native_window_close.rs`; native presenter storage must not own document lifecycle decisions.
- Keep model recompute and payload assembly in `app/host_lifecycle/native_window_presenters.rs`; native window children should stay focused on target, presentation, and presenter storage primitives.

## Validation Notes

The 2026-06-19 native-window split reduced `native_windows.rs` from 131 lines to 8 lines. `native_windows/target.rs` is 36 lines and owns native floating-window target DTO/filtering. `native_windows/presentation.rs` is 37 lines and owns host-presentation/platform-window writes. `native_windows/store.rs` is 61 lines and owns presenter map synchronization.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app native-window target/presentation/store ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 142 warnings, `zircon_editor` 63 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 native-window presenter subowner split kept `native_windows.rs` as a 6-line structural entry and widened the `NativeFloatingWindowTarget` re-export from test-only to app-internal. This lets `host_lifecycle/native_window_presenters/callbacks.rs` and `presentation.rs` accept typed native-window targets while the DTO remains owned by `native_windows/target.rs`.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app host-lifecycle native-window presenter subowner ownership scan, and scoped `git diff --check`. Focused `cargo check` is covered by the lifecycle slice validation record when the compile lane is available.
