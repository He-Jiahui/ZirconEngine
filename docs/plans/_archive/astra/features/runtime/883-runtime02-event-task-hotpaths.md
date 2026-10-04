---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-24-runtime-events-and-task-hotpaths.md
related_records:
  - docs/plans/astra/features/runtime/882-runtime-event-unsubscribe-lock-scope.md
  - docs/plans/astra/features/runtime/884-runtime02-task-diagnostic-page-capacity.md
  - docs/plans/astra/features/runtime/885-runtime02-task-graph-dependency-fence-capacity.md
  - docs/plans/astra/features/runtime/886-runtime02-task-handle-wait-all-fast-path.md
  - docs/plans/astra/features/runtime/887-runtime530-indirect-args-capacity.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/core/runtime/events/topic.rs
  - zircon_runtime/src/core/runtime/events/publish.rs
  - zircon_runtime/src/core/runtime/tasks/parallel_for.rs
  - zircon_runtime/src/core/runtime/tasks/pools.rs
tests:
  - zircon_runtime/src/core/runtime/events/topic.rs
  - zircon_runtime/src/core/runtime/tasks/parallel_for.rs
  - zircon_runtime/src/core/runtime/tasks/pools.rs
---

# Runtime883 Runtime02 Event and Task Hotpaths

## 完成列表

| Scope | Optimization | Evidence and remaining gate |
| --- | --- | --- |
| Topic lookup | Event-bus topic reads use the shared `RwLock` read path; subscription and empty-topic removal retain the write path. | `topic.rs` keeps the ownership boundary explicit and publish/diagnostic reads avoid exclusive registry acquisition. Windows-native release throughput evidence remains pending. |
| Bulk subscriber pruning | Disconnected IDs are sorted once in a private buffer and subscriber membership uses binary search, while the single-ID path remains allocation-free. | `remove_subscribers_while_delivery_locked` source/behavior tests cover both paths; the Runtime02 probe-count target needs managed release evidence. |
| `parallel_for` small inputs | Empty slices return without installing a pool task; one-chunk inputs invoke the task directly and multi-chunk inputs retain Rayon chunking. | Focused source and behavior tests cover empty/single/multi-chunk cases; elapsed-time targets are not inferred from structure. |
| Task-pool worker accounting | Physical worker totals are resolved after per-pool minimums and reports expose the created total consistently. | Added low-host and explicit-budget regression tests in `pools.rs` that assert the reported total equals the three created pool counts; release construction and host-throughput gates remain open. |
| Scope boundary | Subscriber queue-age accounting is separately recorded in Runtime882 and is not silently folded into this Runtime02 acceptance. | No tooling migration or parent execution/shutdown architecture is claimed by this record. |

## Validation boundary

Exact Rustfmt and scoped diff checks are available for the touched Runtime
hot-path sources. The current managed Runtime package batch was reconciled as
coordinator job `bf1109dcee764aa8b4de35c434eb1155`, `released` with wrapper exit
`1`; its target and Cargo run diagnostic were not retained, so no compiler or
test conclusion is possible from that row. The pools regression tests were
added after that source seal and require another grouped current-source batch.
Until a terminal Cargo receipt and the Runtime02 release benchmark evidence
are recorded, this record is an implementation ledger only and does not claim
test or performance acceptance.

A fresh post-seal package request was submitted in execution session `65262`
to include the new `pools.rs` tests; no result is inferred before its managed
terminal receipt.

The request later reached coordinator job
`dceac29243624adab3c53aee8440d3ff` with wrapper exit `1`, but the coordinator
retained neither a Cargo run row nor a compiler/test diagnostic. The receipt
is therefore stage-unknown and does not satisfy the Runtime02 test or release
performance gate.

A replacement grouped current-source request was submitted as execution session
`70383` with verbose output and one test thread. It remains intentionally
unpolled; this submission is not a test or performance receipt.

The corresponding grouped Windows Release benchmark request was submitted as
execution session `84917` with the ignored `runtime02` filter. Its timing and
threshold output remain pending and are not inferred from the queued request.

The release dry-run also selected one `cargo check --release --tests` plus one
ignored `runtime02` test launch with one test thread. Selection output is not a
receipt. The release request was queued in parallel in execution session
`95114`, so its source closure and terminal timing/threshold output remain
independent pending evidence rather than a result of the development batch.

### Bounded grouped ledger reconciliation (2026-09-25)

One read of the coordinator ledger found the fresh Runtime development job
`42db99a91bce4a0ca4b29d389670f4d8` still `running` and the Runtime02 Release
job `5118f19a65594ffdb8550581e88721d2` still `running`; neither exposed a
`cargo_job_runs` receipt at that read. The source is not being polled while
independent work continues, and no Cargo/test/performance result is inferred.

The later bounded ledger read showed Runtime development job
`42db99a91bce4a0ca4b29d389670f4d8` and Runtime02 Release job
`5118f19a65594ffdb8550581e88721d2` both `released` with wrapper exit `1`,
without a `cargo_job_runs` row or Cargo diagnostic. Their stage and benchmark
outcome are therefore unknown; no test or performance pass is claimed.

### One-time terminal reconciliation after independent review (2026-09-25)

The next bounded ledger read confirmed the same Runtime development and
Runtime02 Release jobs remain `released` with wrapper exit `1`, still without a
Cargo run row or retained diagnostic. This preserves the stage-unknown
classification; no Runtime02 test, benchmark, or throughput threshold is
accepted from the wrapper exit.

### Fresh grouped validator launch after source stabilization (2026-09-25)

After the output-owner source guard and the Runtime pool regressions were
stabilized, a new managed `validate-matrix.ps1 -Package zircon_runtime
-LibTests -TestThreads 1 -VerboseOutput` development lane was launched together
with the Editor package lane. The validator wrappers were left running without
polling; no Cargo job id, compiler output, test count, or performance result is
claimed from launch. A parallel Runtime02 Release launch was attempted with
`-CargoProfile release -TestFilter runtime02 -IgnoredTests -NoCapture`, but the
coordinator rejected admission with `request_overloaded` before creating a
Cargo job, so no Release receipt exists from that attempt.

### Bounded terminal reconciliation after the grouped wave (2026-09-25)

The grouped Runtime development job `8f9b52c07d12401594bc678398427f72`
materialized and was later released after the coordinator's five-minute
`cargo.health_timeout`; the Runtime02 Release job `5118f19a65594ffdb8550581e88721d2`
also released with wrapper exit `1` and no retained `cargo_job_runs` row. The
development job's process tree was cleaned up by the coordinator, but neither
lane retained Cargo output, test counts, or benchmark thresholds. This is a
validator-health outcome only; no Runtime02 compiler, test, throughput, or
Release performance pass is inferred.

### Fresh four-lane submission after Editor928 (2026-09-25)

Runtime development (`-Package zircon_runtime -LibTests -TestThreads 1
-VerboseOutput`, PTY `76534`) and Runtime02 Release (`-CargoProfile release
-TestFilter runtime02 -IgnoredTests`, PTY `34527`) were submitted together
with the current Editor development and Editor09 Release lanes. The wrappers
were intentionally left unpolled while independent source work continued;
terminal-control setup emitted no Cargo job or test/benchmark receipt. Runtime
compiler, test, throughput, and Release threshold evidence remains pending.
The Editor928 cancellation-barrier extension landed after this submission, so
these receipts do not validate that final Editor source state.

The current-source resubmission uses Runtime development PTY `92163`, Editor
development PTY `57512`, Runtime02 Release PTY `65557`, and Editor09 Release
PTY `88159`; all four wrappers remain intentionally unpolled.

After Editor929, the latest grouped current-source submission uses Runtime
development PTY `22890`, Editor development PTY `85495`, Runtime02 Release PTY
`45349`, and Editor09 Release PTY `34355`; these wrappers also remain
intentionally unpolled.

Editor930 and Editor931 subsequently changed the shared source wave after those
receipts. The latest four-lane current-source submission uses Runtime
development PTY `81169`, Editor development PTY `98412`, Runtime02 Release PTY
`26311`, and Editor09 Release PTY `68990`; the wrappers remain intentionally
unpolled, so this Runtime02 record still has no compiler, test, throughput, or
Release benchmark receipt.

Editor932 changed the Editor pending dependency scratch buffer after that wave.
The resulting current-source replacement was submitted together as Runtime
development PTY `81935`, Editor development PTY `42136`, Runtime02 Release PTY
`99713`, and Editor09 Release PTY `4102`. All wrappers remain intentionally
unpolled; Runtime02 compiler, test, throughput, and Release benchmark evidence
is still pending.

Runtime884 subsequently reserved the actual diagnostic journal page capacity
after the ordered partition. The current-source replacement wave is recorded
in Runtime884 and uses Runtime development PTY `99767`, Editor development PTY
`69461`, Runtime02 Release PTY `33591`, and Editor09 Release PTY `77672`; all
wrappers remain intentionally unpolled.

Runtime885 subsequently reserved the submit-after dependency-fence capacity, so
the Runtime884 wave is stale for the current Runtime source. The replacement
four-lane batch is recorded in Runtime885 and the central log; all wrappers
remain intentionally unpolled.

Runtime886 subsequently added the empty/single `TaskHandle::wait_all` fast path,
so the Runtime885 wave is stale for current Runtime source. The replacement
four-lane batch is recorded in Runtime886 and the central log; all wrappers
remain intentionally unpolled.
