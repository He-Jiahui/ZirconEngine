# Editor Event Listener Audit

## Scope
- Extended editor event listener deliveries so external automation can read event `source` and structured `result` without cross-referencing the journal.
- Extended listener filters beyond operation path prefix to include source selection and success/failure selection.
- Hardened unknown listener controls so delivery query and ack requests return structured errors instead of empty success payloads.
- Added a listener status query shape so external panels can inspect a listener descriptor plus pending delivery count and sequence bounds before fetching or acknowledging deliveries.

## Test Inventory
- Source/failure filter case: a listener configured for `EditorEventSource::Cli` failures should ignore headless success and CLI success, then receive the failed CLI operation control request.
- Operation group filter case: a listener configured for one `operation_group` should ignore other groups and ungrouped operations, then receive only matching grouped deliveries.
- Delivery audit case: the delivered JSON exposes `source` and `result.error`.
- Missing listener case: `QueryDeliveries`, `QueryDeliveriesSince`, and `AckDeliveriesThrough` should fail when the listener id is not registered.
- Listener status case: `QueryListenerStatus` should expose descriptor metadata, `pending_delivery_count`, `first_pending_sequence`, and `last_pending_sequence`, then reset the pending sequence bounds to `null` after all deliveries are acknowledged.
- Static checks: touched listener/test files must pass `rustfmt --edition 2021 --check`; touched docs/code must pass `git diff --check`.

## Tooling Evidence
- Windows toolchain path: focused Cargo loops used `cargo test` / `cargo check` with `--locked`, `--jobs 1`, and an isolated `D:\cargo-targets\zircon-codex-editor-listener-audit-green` target directory to avoid repo-local target churn.
- WSL toolchain: `cargo 1.94.1` and `rustc 1.94.1`; retry target was `/tmp/zircon-target-editor-listener-status` to keep Linux validation separate from Windows target outputs.
- WSL accepted path: `RUSTFLAGS="-C debuginfo=0"` was used only for the Linux evidence runs to reduce local debug-info cost; commands still used `--locked`, `--jobs 1`, and the isolated `/tmp/zircon-target-editor-listener-status` target.
- WSL process cleanup: later `ps -eo pid,ppid,etime,args | grep zircon-target-editor-listener-status` checks found no remaining listener-status Cargo/rustc process.

## Results
- TDD red: `cargo check -p zircon_editor --tests --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-component-drawer-bindings --message-format short` failed on missing `EditorEventListenerFilter::source`.
- Passed: `rustfmt --edition 2021 --check zircon_editor/src/core/editor_event/listener.rs zircon_editor/src/ui/host/editor_event_listener_control.rs zircon_editor/src/tests/editor_event/runtime.rs`.
- Passed: `cargo check -p zircon_editor --tests --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-listener-audit-green --message-format short`; only the existing `editor_meta.rs::save` dead-code warning was reported.
- Timed out: `cargo test -p zircon_editor --lib event_listener_filter_limits_delivery_by_source_and_failure_state --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-listener-audit-green --message-format short -- --test-threads=1 --nocapture` timed out after 15 minutes while linking the `zircon_editor` test binary; no Rust or assertion diagnostic was emitted, and the owned cargo/rustc processes were stopped by exact command-line match.
- Failed assertion rerun: `cargo test -p zircon_editor --lib event_listener_filter_limits_delivery_by_source_and_failure_state --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-listener-audit-green --message-format short -- --test-threads=1 --nocapture` delivered the expected filtered failure record but the test expected the wrong error-text fragment.
- Passed: `cargo test -p zircon_editor --lib event_listener_filter_limits_delivery_by_source_and_failure_state --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-listener-audit-green --message-format short -- --test-threads=1 --nocapture` with `1 passed; 0 failed`; existing warning was `editor_meta.rs::save` dead code.
- TDD red: `cargo test -p zircon_editor --lib event_listener_control_rejects_unknown_listener_queries --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-listener-audit-green --message-format short -- --test-threads=1 --nocapture` failed because `QueryDeliveries` returned `error = None` for an unknown listener.
- Passed: `cargo test -p zircon_editor --lib event_listener_control_rejects_unknown_listener_queries --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-listener-audit-green --message-format short -- --test-threads=1 --nocapture` with `1 passed; 0 failed`; existing warnings were `zircon_runtime` unused indirect draw method and `editor_meta.rs::save` dead code.
- Passed: `cargo test -p zircon_editor --lib event_listener_control --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-listener-audit-green --message-format short -- --test-threads=1 --nocapture` with `6 passed; 0 failed`; existing warning was `editor_meta.rs::save` dead code.
- TDD red: `cargo check -p zircon_editor --tests --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-listener-audit-green --message-format short` failed on missing `EditorEventListenerControlRequest::QueryListenerStatus`, proving the new status regression observed the missing API.
- Passed: `rustfmt --edition 2021 --check zircon_editor/src/core/editor_event/listener.rs zircon_editor/src/core/editor_event/mod.rs zircon_editor/src/ui/host/editor_event_listener_control.rs zircon_editor/src/tests/editor_event/runtime.rs`.
- Blocked GREEN: `cargo test -p zircon_editor --lib event_listener_control_reports_listener_status_with_pending_delivery_bounds --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-listener-audit-green --message-format short -- --test-threads=1 --nocapture` timed out after 10 minutes while linking the `zircon_editor` test binary; no Rust or assertion diagnostic was emitted, and the owned cargo/rustc processes were stopped by exact command-line match.
- Blocked compile reruns: `cargo check -p zircon_editor --tests --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-listener-audit-green --message-format short` timed out after 5 minutes and again after 15 minutes without a Rust diagnostic; `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-listener-audit-green --message-format short` timed out after 10 minutes while compiling the editor lib. Same-target owned cargo/rustc processes were stopped by exact command-line match each time.
- Blocked WSL compile: `CARGO_TARGET_DIR=/tmp/zircon-target-editor-listener-status cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short` first failed in the `zircon_runtime` dependency on unrelated private-field access errors in active runtime graphics render cutover files (`update_hybrid_gi_runtime.rs` and `update_virtual_geometry_runtime.rs`).
- Blocked WSL retry: the same WSL command later progressed into `zircon_editor` compilation but timed out after 20 minutes without producing a `libzircon_editor` rmeta/rlib or Rust diagnostic for the listener-status change.
- Passed WSL compile: `CARGO_TARGET_DIR=/tmp/zircon-target-editor-listener-status RUSTFLAGS="-C debuginfo=0" cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never` finished in 14m53s with existing warnings only (`editor_meta.rs::save` dead code and `showcase_demo_state.rs` unused variants).
- Passed WSL focused test: `CARGO_TARGET_DIR=/tmp/zircon-target-editor-listener-status RUSTFLAGS="-C debuginfo=0" cargo test -p zircon_editor --lib event_listener_control_reports_listener_status_with_pending_delivery_bounds --locked --jobs 1 --message-format short --color never -- --test-threads=1 --nocapture` finished in 17m23s; result was `1 passed; 0 failed; 886 filtered out`.
- Passed: scoped `git diff --check` for touched listener code, runtime tests, docs, acceptance, and session files with only LF/CRLF warnings.
- Pending: `event_listener_filter_limits_delivery_by_operation_group` covers exact operation-group listener filtering, but Cargo validation is waiting for the active Runtime UI/runtime-interface build queue to quiet down.

## Acceptance Decision
- Listener audit API and unknown-listener error semantics are accepted.
- Source/failure listener filtering and missing-listener control error semantics are accepted.
- Listener status query is accepted on WSL/Linux evidence: the crate-level editor lib check and focused status regression test both pass. Windows focused runs remain inconclusive under concurrent Cargo/link load, but they produced no Rust or assertion diagnostic.
- Operation group listener filtering is implemented and has a focused regression in the suite; final acceptance remains pending until the next focused Cargo run reaches test execution.
