---
title: Editor Input Capture Shutdown Owned Drain
category: zircon_editor
report_id: Editor877-input-capture-shutdown-owned-drain-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor877 Input Capture Shutdown Owned Drain

## Finding

`ToolInputCaptureAuthority::shutdown` first copied every capture ID into a
temporary vector, then removed each capture and each source-index entry through
ordered-tree lookup paths. Shutdown terminates every capture and owns
`&mut self`, so selection IDs and per-entry removals were unnecessary.

## Optimization

- Take ownership of the ordered capture map and clear the complete source index
  once.
- Reserve the exact result/event bounds from the captured map length and iterate
  its values in capture-ID order.
- Retain the one `ToolInputCaptureHandle` clone required because the report
  outcome and lifecycle event each own a handle.
- Preserve `Shutdown` disposition, ordered outcomes/events, empty authority
  state, source-index clearing, identity progression, and all selective
  end/release paths.

## TDD and deterministic evidence

The Editor877 source/model contract was observed RED with five failures (the
deterministic model was already true), then GREEN at `6/6`. The lower regression
creates captures whose source order differs from their capture-ID order and
verifies shutdown outcomes/events remain capture-ID ordered while every source
lookup is cleared.

For 4,096 captures, the retired path stores 4,096 capture IDs and performs 4,096
capture-tree plus 4,096 source-tree removals. The new path stores zero capture
IDs and performs zero per-entry tree removals. Both paths retain 4,096 handle
clones because outcome/event dual ownership is part of the public report
contract. The ignored 101-pair Release marker
`EDITOR877_INPUT_CAPTURE_SHUTDOWN_OWNED_DRAIN_BENCH_V1` emits alternating
p50/p95/p99 samples and requires improved model p95.

## Local validation boundary

- Exact-file Rustfmt and scoped `git diff --check` pass.
- The Editor877 source/model contract passes `6/6`; the lower Rust regression is
  wired for the managed current-source batch.
- Editor875–878 were submitted together in the asynchronous v8 multi-task
  current-source lane; no task was submitted alone.
- Local source/model evidence does not establish Windows compilation, actual
  allocator behavior, or interactive-tool shutdown p50/p95/p99 latency.
- Tooling production remains deferred for the later Rust migration.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/core/tools/input_capture.rs` | `342AD8A33FFF5DCA7E56D1F4085C345948D21558DA1334794D3557EEFAA4C763` |
| `zircon_editor/src/core/tools/optimization_batch_editor877_input_capture_shutdown_owned_drain_tests.rs` | `82FED215001BB150C3671706F24C4108A673BFB3DC886FB1B7AAB4D871FDAE4F` |
| `tools/tests/test_editor877_input_capture_shutdown_owned_drain_performance_contract.py` | `84D9A888DD23FAB1E5A363CE12231CB9AD13E43C04EBE42647A791A5B7A70DAB` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
a combined current-source Windows lane compiles Editor, executes the lower
regression and ignored Release marker, and supplies allocator plus interactive-
tool shutdown p50/p95/p99 evidence. The index-removal model is not product
acceptance.
