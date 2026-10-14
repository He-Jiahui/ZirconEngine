---
title: Runtime44 Disabled Log Output Shutdown Record
category: zircon_runtime
report_id: Runtime44-disabled-log-output-shutdown-2026-10-13
date: 2026-10-13
implementation_status: source_reviewed
validation_status: combined_native_validation_pending
performance_status: no_performance_claim
---

# Runtime44 Disabled Log Output Shutdown Record

## Problem and behavior

`DiagnosticLogState` retains the requested `console_enabled` and `file_enabled` settings even when no `SinkRuntime` exists. Before this change, `outputs_succeeded()` treated every absent worker as failure. A process configured with both outputs explicitly disabled therefore shut down, unpublished its state, saved `Some(false)`, and returned false on every later idle shutdown call even though there was no configured output to flush or join.

The fix keeps the real worker result authoritative:

```rust
Some(sink) => sink.outputs_succeeded(),
None => !self.console_enabled && !self.file_enabled,
```

An absent worker is successful only when both outputs were disabled. If console, file, or both were requested but no worker exists, shutdown remains failed and the terminal `Some(false)` result stays sticky. Real worker errors still flow through `SinkRuntime::outputs_succeeded()`.

Both-disabled output settings are a valid public configuration through `initialize_process_log_with_settings`. The shipped CLI initializers use defaults that enable both outputs; the reviewed change does not claim that the default CLI is broken. A file-open failure after exhausting candidates or a worker-spawn failure can leave a requested output without a worker, but this change does not simulate or claim an actual OS file-open or thread-spawn failure. It also does not redefine fallback when another real output worker exists.

## Regression coverage and preserved lifecycle

The first new ordinary test constructs a real `DiagnosticLogSettings` with both outputs disabled, initializes an isolated `ProcessLogController`, and verifies the resulting real state has no worker, file, or snapshot. A lazy message remains unevaluated; shutdown unpublishes the state, records `Some(true)`, and repeated shutdown calls remain true.

The second new ordinary test injects the existing production postcondition of a missing worker while console-only, file-only, and both-output request flags are set. It then exercises the real publish and controller-shutdown path and verifies unpublication, `Some(false)`, and repeated false results. This is a deterministic state test, not an OS failure test. Existing `Some(worker)` terminal-output failure coverage remains selected, so the false worker result cannot be replaced by request flags.

Worker ownership, actual join behavior, timeout and retry authority, dynamic lease counts, terminal-result reset on a fresh generation, queue admission/cancel/backpressure, flush, durability and output-error counters are unchanged. No fields, production seams, public settings/API, JSON/schema or dependencies were added. Public flush behavior is unchanged.

## Next combined validation floor

The next combined native contract is ordinal 2, Debug profile, with the previous 66-test floor retained in full. It adds the two new ordinary cases and selects five existing cases that were not in the prior literal filter: the no-active-sink case and four stable terminal-result cases (repeated failure, repeated success, reset after fresh publication, and timeout retention). The resulting minimum selected gate is **73**. The exact selector list is retained in the linked validation contract. The existing affected timeout, worker join, durability, error-count and partial-console-success cases remain selected. No per-case compile arms are added, and the separate ignored ordinal-9 floor remains 16.

The contract's Cargo prefix is:

```text
cargo test -p zircon_runtime --no-default-features --features target-editor-host,test-support --lib --locked -- --test-threads=1
```

The two added exact selectors are:

- `diagnostic_log::sink::tests::lifecycle::process_shutdown_explicitly_disabled_outputs_completes_without_worker`
- `diagnostic_log::sink::tests::lifecycle::process_shutdown_missing_requested_sink_remains_failed`

The five newly selected existing exact selectors are:

- `diagnostic_log::sink::tests::lifecycle::process_shutdown_without_an_active_sink_is_already_complete`
- `diagnostic_log::sink::tests::lifecycle::process_shutdown_repeats_completed_worker_output_failures`
- `diagnostic_log::sink::tests::lifecycle::process_shutdown_repeats_completed_success_without_restarting_worker`
- `diagnostic_log::sink::tests::lifecycle::process_shutdown_terminal_result_resets_only_after_fresh_generation_publication`
- `diagnostic_log::sink::tests::lifecycle::process_shutdown_timeout_retains_generation_until_worker_completion`

## Review evidence

- Frozen patch: [runtime-disabled-log-shutdown.patch](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime-disabled-log-shutdown.patch), SHA-256 `ae281fc5f0d27d7870069acc19516f72d0eb575382507505902b8abb3dd5e0e8`.
- Source and validation evidence: [runtime-disabled-log-shutdown-evidence.json](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime-disabled-log-shutdown-evidence.json), SHA-256 `459e6218112e4e6601fa6f884267eb6cfc8060662f041b5a23b6f3da3da428f9`.
- Exact root guard: [runtime-disabled-log-shutdown-guard.json](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime-disabled-log-shutdown-guard.json), SHA-256 `5c4657cb1080ead2195c9939163b0e7af9e99860d22943b16e9124cba742c65a`; passed with zero errors, warnings, and suppressed findings.
- Previous batch10 baseline contract: [batch10-validation-contract.json](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/batch10-validation-contract.json), SHA-256 `e2b75b4a129494d2007794fc02e76d160fa415a50c714727129ca5408b9b95a1`. It supplies the old 66-test arm; it does not contain the seven new Runtime44 selectors.
- Runtime44 next-selection evidence: [runtime-disabled-log-shutdown-evidence.json](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime-disabled-log-shutdown-evidence.json), SHA-256 `459e6218112e4e6601fa6f884267eb6cfc8060662f041b5a23b6f3da3da428f9`; `next_combined_native_contract` records the added selectors and minimum floor 73.
- The batch10 handoff metadata was retained at [batch10-handoff.json](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/batch10-handoff.json) (SHA-256 `00a526a7e8b918a5d002b846ef54648c2159d54a878204730431933612da93ec`) and [batch10-handoff.md](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/batch10-handoff.md) (SHA-256 `a9870385c4b7895dc01621a99ad88b579b199c376477a0dbc638c96b89e1b578`). The F0 source repair followed that handoff; these files do not stand in for native acceptance.

The two frozen source paths are `zircon_runtime/src/diagnostic_log/sink.rs` (before SHA-256 `a597874d7d8ea31a459590e1022d0ce32d9a50c86bfd33508ac74d247c73f686`, after `cd7996dfd2e032293bbbf5ba879c5508a0cc83a79e6a62656e6c396ae9969e93`) and `zircon_runtime/src/diagnostic_log/sink/tests/lifecycle.rs` (before `7a355004460c8ee1b1e1d91915059d33ac965cb47822d280508cd0c0366515ac`, after `589cc3e42c96810ab13874920fac157556efe09c13e05afc889d4d69bb3127db`). Their literal test wiring is through `sink.rs` → `sink/tests/mod.rs` → `lifecycle.rs`.

Affected-file rustfmt and the named-base whitespace check passed. Static source comparison confirms only `outputs_succeeded()` differs in the production file and the existing lifecycle cases remain. The exact guard passed. Native Rust execution is pending the combined managed batch; performance and product validation remain open. No native result or performance result is claimed here. Batch11 has not been formally handed off.
