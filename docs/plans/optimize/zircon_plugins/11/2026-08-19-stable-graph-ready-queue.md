# Plugins11 Stable Graph Ready-Queue Optimization Record

- Date: 2026-08-19
- Owner: `plugins11-stable-ready-heap-r1-01a00797-20260819`
- Source plan: `docs/plans/optimize/zircon_plugins/11-first-party-sound-source-runtime-editor-dist-catalog-mixer-spatial-reverb-timeline-product-integration-review.md`, mixer graph routing scope adjacent to NSND-P1-018
- Status: prior performance gate passed; Plugins super-batch revalidation pending

## Problem

The mixer graph topological pass stored ready tracks in a `Vec`. Every dequeue
used `remove(0)`, shifting the remaining ready set, and every newly unlocked
track sorted the complete ready set while repeatedly scanning the authored
track list for positions. A wide graph therefore paid quadratic entry moves
before any backend track allocation.

## Change

- A min-heap now stores authored track positions instead of complete track IDs.
- Ready dequeue no longer shifts the remaining set.
- A one-time track-position index replaces repeated linear position scans.
- The smallest authored position still wins whenever multiple tracks are ready,
  preserving the previous deterministic order and cycle behavior.

## Deterministic Performance Evidence

| Workload | Before | After | Reduction |
|---|---:|---:|---:|
| 50,000 independent tracks | 1,249,975,000 ready-entry moves | 0 ready-entry moves | 100% |
| Ready dequeue | O(ready tracks) | O(log ready tracks) | one complexity class |
| Newly unlocked track position lookup | O(tracks) per comparison | O(1) indexed lookup | one complexity class |

## Acceptance

- `topological_order_preserves_authored_priority_when_lower_slot_becomes_ready`
  proves that a newly unlocked lower authored slot preempts an unrelated higher
  slot exactly as before.
- `stable_ready_heap_release_benchmark_evidence` compares 21 paired,
  alternating release samples over 50,000 independent tracks against the
  previous algorithm and computes nearest-rank P50/P95.
- Timing gate: optimized P95 must be no more than 50% of legacy P95.
- Exact-file Rustfmt, Cargo regression, and release P50/P95: pending one batched
  Windows coordinator validation; no per-task Cargo command is used.

Replacement job `4b4f11bcee7c4d4fa053aa500adf6d43` reached the compiled
Sound test harness but its PowerShell log wrapper rejected a blank Cargo output
line. Run `72a2ff63b2014b068eb3e3ab908cf45a` therefore provides no accepted
test or timing result. The empty-line-safe wrapper is being rerun as job
`2bc1a5e1161e481595001d49e88113dd`; the ready-heap result remains pending
until the batch emits its terminal success marker and both performance rows.

Retry job `2bc1a5e1161e481595001d49e88113dd` completed as run
`2b85d5cec24a429e885eae6874c9269d`. Dynamic-event and graph-validation
regressions passed, and this ready-heap release stage exited successfully after
`1862.017s`. The combined batch then failed the handler benchmark's independent
P95 gate, so it emitted no terminal success marker and this stage is not yet an
accepted final performance receipt. The successor batch will rerun both tasks
and must retain both `PERF_RESULT` rows together.

Successor job `a8f45b165b1c4831a17586d39bfeff3c` uses input-manifest hash
`e389b2745bc7ce3bc4d311255be5d82d8f36ad399572d41305f05a8f66f3a4c2`;
coordinator request `bbd1b35f16864021a6e9e3e58bb12460` accepted its only run.
It is still `running`, and its persistent log is
`F:/cargo-targets/verify/a8f45b165b1c4831a17586d39bfeff3c/target/temporary/zircon-validation-plugins11-ready-dispatch-batch-r3.log`.
This is submission evidence only. Script `r3` has SHA-256
`bcdcd8bdd3bab4563f06c7b5215ac2898e6667b159739c7d480f51d3d082b02d`.
Preflight performed after launch proved that libtest may place the first
`PERF_RESULT` after a `test ...` prefix, which can make `r3` fail only during
final marker aggregation. The live Cargo run is not interrupted. Prepared
script `r4` has SHA-256
`56ecaba9cda6d7ade87a889e70086464ce68a1027591dce0aeea0947972247e3`
and normalizes those prefixed rows if a replacement run is required.

Run `54e83851e1da41cc8e48e6ae4e180d81` completed every Cargo stage for job
`a8f45b165b1c4831a17586d39bfeff3c`. Dynamic-event regressions passed `32`
tests and graph-validation regressions passed `3`; each group ignored only its
release benchmark during the debug regression pass. The ready-heap benchmark
reported legacy P50/P95 `150.2874/153.4129 ms` and optimized P50/P95
`21.2928/37.5786 ms`, reductions of `85.83%/75.50%`. Optimized P95 is
`24.50%` of legacy and passes the `50%` gate. r3 exited `1` only in its final
column-zero marker aggregation after all tests and both timing gates passed.
The corrected r4 wrapper must emit the combined success marker before this
record is terminally accepted.

Full candidate snapshot `1901` was submitted for Cargo-closure materialization
as r4 job `efa9eb1fc0ee4e0ba60a44e89bca49bd` under coordinator request
`a88f07fe275041558cd41fde62714be5`. The copy remains `materializing`, so this
line records submission only and makes no batch-level success claim.

The copy materialized with input-manifest hash
`4e862d678a5d9a9166d72815625b9e9c14298a1bcee0044f2c56809834a929fc`
and the r4 batch is now `running` under coordinator root PID `55324`. The
client timeout occurred after acceptance; terminal evidence remains pending
and the run was not resubmitted.

Run `d0d913056b4b4e64bae9ef719a5777c7` exited in the preceding handler
benchmark after `1965.948s`, before this ready-heap stage or either regression
group ran. The failure was a sequential microbenchmark load-drift result, not a
ready-heap product result. The handler harness now uses 21 alternating paired
samples with four dispatches averaged per sample while retaining its original
P95 gate. This record remains pending a fresh combined immutable batch after
the concurrent main all-target compilation completes.

Full candidate snapshot `1909` was submitted for Cargo-closure materialization
as job `f447fc1e60c143abae4d314e51b219f6` under request
`1b2e3c7344d94682ac695e1c7aef3f19`. No Cargo run has been submitted for this
copy while the main all-target successor remains CPU-intensive.

The copy subsequently materialized with input-manifest hash
`4b0361a1b3d44b6c4a84b7bf9ff5a97500d480180166dd3f4dab981ee4086f15` after the
competing main Cargo run became terminal. Coordinator request
`c65d05f5789e4951ae74a5ec3049d313` accepted the complete r4 batch. The client
timed out after acceptance and no duplicate was submitted; terminal ready-heap
P50/P95 evidence and both regression receipts are still pending.

Run `94c6b36db3494c52be9f0653054cc549` failed the preceding handler benchmark
after `2032.223s`, before this ready-heap benchmark or either regression group
ran. The handler repair now maintains its event index at registration time and
moves one drained invocation per event; this graph task remains unchanged. The
next accepted receipt must come from the combined Plugins super-batch and must
retain both Plugins11 performance rows plus the two regression groups.

## Remaining Scope

This change removes ready-set queue and position-scan amplification only. Graph
dependency collection still allocates edge vectors, and full direct/send/master
routing correctness, gain-law capture, render-thread ownership, and product
qualification remain open under NSND-P1-018 and Plugins11 G08-G10/G32.
