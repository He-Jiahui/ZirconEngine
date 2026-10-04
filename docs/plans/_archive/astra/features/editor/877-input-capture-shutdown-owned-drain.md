---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/125/2026-09-21-input-capture-shutdown-owned-drain.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/editor/876-tool-scheduler-shutdown-owned-drain.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/core/tools/input_capture.rs
tests:
  - zircon_editor/src/core/tools/optimization_batch_editor877_input_capture_shutdown_owned_drain_tests.rs
  - tools/tests/test_editor877_input_capture_shutdown_owned_drain_performance_contract.py
---

# Editor877 Input Capture Shutdown Owned Drain

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Editor125 input-capture shutdown drain | Take the ordered capture map, clear the source index once, and publish exact-capacity outcomes/events directly in capture-ID order. This removes the ID snapshot and per-entry tree removals while retaining the handle clone required by dual report/event ownership. | Intentional RED `1/6` → GREEN `6/6`; lower differing-source/capture-order regression and ignored `EDITOR877_INPUT_CAPTURE_SHUTDOWN_OWNED_DRAIN_BENCH_V1` marker are wired. The 4,096-capture model changes ID scratch slots `4096→0` and per-entry tree removals `8192→0`; managed Cargo/Release, allocator, and product p50/p95/p99 evidence remain pending. | implemented_pending_validation |

## Scope boundary

This slice changes only all-capture shutdown. Selective owner/lease/window end
paths, priority/preemption, capacity, identity allocation, lifecycle schema,
resource ownership, and tooling production remain unchanged.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/core/tools/input_capture.rs` | `342AD8A33FFF5DCA7E56D1F4085C345948D21558DA1334794D3557EEFAA4C763` |
| `zircon_editor/src/core/tools/optimization_batch_editor877_input_capture_shutdown_owned_drain_tests.rs` | `82FED215001BB150C3671706F24C4108A673BFB3DC886FB1B7AAB4D871FDAE4F` |
| `tools/tests/test_editor877_input_capture_shutdown_owned_drain_performance_contract.py` | `84D9A888DD23FAB1E5A363CE12231CB9AD13E43C04EBE42647A791A5B7A70DAB` |

## Managed gate

Editor875–878 were submitted together in asynchronous v8 and were not submitted
individually. Keep this entry pending until that current-source lane
provides Editor compilation, lower/ignored Release execution, allocator
evidence, and interactive-tool shutdown percentiles.
