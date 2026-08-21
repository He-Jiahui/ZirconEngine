# Plugins11 Preordered Dynamic-Event Handler Optimization Record

- Date: 2026-08-19
- Owner: `plugins11-stable-ready-heap-r1-01a00797-20260819`
- Source plan: `docs/plans/optimize/zircon_plugins/11-first-party-sound-source-runtime-editor-dist-catalog-mixer-spatial-reverb-timeline-product-integration-review.md`, NSND-P1-033
- Status: persistent handler snapshot repair complete; Plugins super-batch validation pending

## Problem

Every pending dynamic event cloned all matching handler descriptors into a
temporary vector and sorted that vector by priority/plugin/handler before
creating deliveries. Both dispatch service entry points also cloned the full
handler registry solely to work around field borrowing. A burst of E events
therefore performed E sorts and E temporary allocations while holding the
manager lock.

## Change

- Handler registration and replacement now maintain the established stable
  priority-descending, plugin-ascending, handler-ascending dispatch order.
- Registration, replacement, removal, and catalog cleanup maintain a persistent
  per-event index over the ordered registry. Dispatch drains pending events
  through constant-time event lookups without rebuilding the index or creating
  per-event matching vectors.
- The output vector reserves the exact delivery count once before fanout.
- The final delivery for each event takes ownership of the drained invocation,
  avoiding one deep invocation clone per event without changing the owned DTO.
- Dispatch and execution service entry points borrow the handler registry and
  pending queue as disjoint fields, removing their full-registry snapshot clone.
- Delivery order, owned delivery payloads, drain-all behavior, and executor
  isolation remain unchanged.

## Deterministic Performance Evidence

| Workload | Before | After | Reduction |
|---|---:|---:|---:|
| 2,048 events across 16 IDs x 4 matching handlers among 64 registered | 2,048 handler sorts | 0 handler sorts | 100% |
| Same burst | 2,048 temporary matching vectors | 0 temporary matching vectors | 100% |
| Handler matching for the same burst | 131,072 string comparisons | 64 persistent index entries + 2,048 lookups across 16 buckets | removes repeated full-registry scans |
| One dispatch service call | 1 full handler-registry clone | 0 full handler-registry clones | 100% |
| Pending staging | 1 complete pending-vector allocation | 0 staging allocations | 100% |
| Invocation fanout | 8,192 deep clones | 6,144 deep clones + 2,048 moves | 25% fewer deep clones |

## Acceptance

- `handler_registration_maintains_dispatch_order_after_insert_and_update`
  proves insert and replacement keep the established fanout order.
- `preordered_dispatch_index_preserves_per_event_order` proves the borrowed
  event index retains priority/plugin/handler order across multiple event IDs.
- Existing fanout ordering, queue drain, cleanup, and executor regressions remain
  in the combined Sound test batch.
- `handler_retain_rebuilds_event_indices` proves catalog cleanup cannot leave
  stale event buckets or handler indices.
- `preordered_handler_dispatch_release_benchmark_evidence` compares 21 paired,
  alternating release samples for 2,048 events across 16 event IDs and 64
  registered handlers, with 4 matching handlers per event, and computes
  nearest-rank P50/P95. This preserves the full
  131,072-comparison legacy scan while isolating dispatch planning from the
  unchanged cost of materializing owned delivery payloads.
- Timing gate: optimized P95 must be no more than 75% of legacy P95.
- Exact-file Rustfmt, Cargo regression, and release P50/P95: pending one batched
  Windows coordinator validation together with the graph ready-queue task.

Initial coordinator run `8482143959024079b5a24e2e64b780d3` passed the two
focused regression stages and the stable ready-heap release benchmark, then
failed in `preordered_handler_release_benchmark`. The old PowerShell wrapper
captured the child output before throwing, so it retained neither the timing
line nor the assertion text. The remaining repeated handler-table scan and
delivery-vector growth were then removed as described above. The replacement
batch uses a persistent external log and must rerun both task benchmarks before
either record is accepted.

The first replacement job `4b4f11bcee7c4d4fa053aa500adf6d43` completed as
run `72a2ff63b2014b068eb3e3ab908cf45a` after the Sound test harness compiled.
Its validation wrapper rejected a blank Cargo output line while copying output
to the persistent log, so run exit code `1` is harness evidence rather than a
product-test or performance result. The wrapper now accepts empty lines and
passes PowerShell parsing; retry job `2bc1a5e1161e481595001d49e88113dd`
completed as run `2b85d5cec24a429e885eae6874c9269d`. Dynamic-event and
graph-validation regressions passed, and the stable ready-heap benchmark
passed. The handler benchmark reported legacy P50/P95 `80.9564/81.8931 ms`
and optimized P50/P95 `67.3405/67.4067 ms`; the `82.31%` P95 ratio missed the
`75%` gate because the original all-64-matching workload materialized 131,072
owned deliveries in both implementations. The revised workload above retains
the same legacy registry-scan count, sort count, and temporary-vector count
while reducing unchanged delivery materialization to 16,384 rows. A successor
combined batch must still emit both accepted `PERF_RESULT` rows and the
terminal success marker.

The revised two-task batch was materialized as job
`a8f45b165b1c4831a17586d39bfeff3c` with input-manifest hash
`e389b2745bc7ce3bc4d311255be5d82d8f36ad399572d41305f05a8f66f3a4c2`.
Coordinator request `bbd1b35f16864021a6e9e3e58bb12460` accepted its only run;
the copy is still `running`, so this is an asynchronous submission log rather
than test or timing acceptance. Script `r3` has SHA-256
`bcdcd8bdd3bab4563f06c7b5215ac2898e6667b159739c7d480f51d3d082b02d`
and writes its full stream to
`F:/cargo-targets/verify/a8f45b165b1c4831a17586d39bfeff3c/target/temporary/zircon-validation-plugins11-ready-dispatch-batch-r3.log`.
After submission, validation preflight found that libtest can prefix the first
performance row with `test ...`; `r3` may therefore reject otherwise valid
Cargo output at its final aggregation step. No running Cargo process is
terminated. Replacement `r4` (SHA-256
`56ecaba9cda6d7ade87a889e70086464ce68a1027591dce0aeea0947972247e3`)
extracts markers from anywhere in a line and is reserved only if the terminal
receipt confirms that harness-only failure.

Run `54e83851e1da41cc8e48e6ae4e180d81` completed all four Cargo stages for
job `a8f45b165b1c4831a17586d39bfeff3c`. The dynamic-event regression stage
passed `32` tests with one release-only benchmark ignored, and the graph
validation stage passed `3` tests with one release-only benchmark ignored.
The revised handler benchmark reported legacy P50/P95 `11.3807/11.8656 ms`
and optimized P50/P95 `7.9777/8.4982 ms`, reductions of `29.90%/28.38%`.
The optimized P95 is `71.62%` of legacy and therefore passes the `75%` gate.
The run exited `1` only after those successful stages because r3 line 68
required a marker at column zero while libtest emitted `test ... PERF_RESULT`.
The corrected r4 aggregation remains required before the combined batch is
accepted; no product source changed in response to this harness-only failure.

Full candidate snapshot `1901` was submitted for Cargo-closure materialization
as r4 job `efa9eb1fc0ee4e0ba60a44e89bca49bd` under coordinator request
`a88f07fe275041558cd41fde62714be5`. The copy is still `materializing`; this is
an asynchronous submission receipt, not a corrected batch result.

Materialization completed with input-manifest hash
`4e862d678a5d9a9166d72815625b9e9c14298a1bcee0044f2c56809834a929fc`.
The r4 batch is now `running` under coordinator root PID `55324`; the client
timed out only after the server accepted the run. Terminal evidence is still
required and no duplicate run was submitted.

Run `d0d913056b4b4e64bae9ef719a5777c7` then stopped in its first release
benchmark after `1965.948s`. Under concurrent all-target compilation, the
0.16-second sequential five-sample window reported legacy P50/P95
`10.9774/11.2230 ms` and optimized P50/P95 `13.6600/16.6274 ms`; no regression
stage or ready-heap benchmark ran. This conflicts with r3's passing result on
the identical product code and exposes a measurement-order flaw: the harness
measured all legacy samples before all optimized samples, so scheduler/load
drift was attributed to the implementation.

The benchmark now collects `21` paired samples, alternates which implementation
runs first in each pair, and averages `4` dispatches per sample before computing
absolute P50/P95. The same `75%` optimized-P95 gate remains. This change affects
only release evidence code; dispatch behavior and the deterministic scan/sort
counters are unchanged. Exact Rust 1.94.1 formatting and scoped diff checks
pass. A fresh immutable batch is required after the main all-target compile no
longer competes for CPU.

Full candidate snapshot `1909` was submitted for Cargo-closure materialization
as job `f447fc1e60c143abae4d314e51b219f6` under coordinator request
`1b2e3c7344d94682ac695e1c7aef3f19`. The copy is materializing only; its Cargo
batch will not start while the main all-target successor is CPU-intensive.

The copy subsequently materialized with input-manifest hash
`4b0361a1b3d44b6c4a84b7bf9ff5a97500d480180166dd3f4dab981ee4086f15` after the
competing main Cargo run became terminal. Coordinator request
`c65d05f5789e4951ae74a5ec3049d313` accepted the complete r4 batch. The client
timed out after acceptance and no duplicate was submitted; a terminal run ID,
regression counts, and the alternating-sample P50/P95 result remain required
before this optimization is accepted.

Run `94c6b36db3494c52be9f0653054cc549` completed the cold release build and
failed its first benchmark after `2032.223s`. Its 21 alternating paired samples
reported legacy P50/P95 `13.1261/19.0608 ms` and optimized P50/P95
`10.7260/16.0451 ms`; the `84.18%` P95 ratio missed the `75%` gate, so no later
regression or ready-heap stage ran. The evidence showed the hot path still
rebuilt its supposedly stable event index on every dispatch and deep-cloned the
drained invocation for every delivery. The registry now owns and refreshes the
index only on handler mutations, and the last matching delivery moves the
invocation. The next evidence comes from the combined Plugins super-batch,
which reruns both Plugins11 benchmarks and regressions together rather than
starting another single-task Cargo job.

## Remaining Scope

The pending event queue is still unbounded and dispatch still drains the entire
queue. Delivery ownership still clones each handler descriptor and all but the
final invocation in each fanout group. This record closes repeated registry
scans, per-event sorting, temporary matching vectors, and staging amplification;
bounded MPSC admission, frame budget, fairness, drop reasons, deadlines,
cancellation, and the unique product pump remain open under NSND-P1-033/034 and
Plugins11 G19-G20.
