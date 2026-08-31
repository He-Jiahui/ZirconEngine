---
title: Runtime200 Borrowed Dispatch Route Sharing
category: zircon_runtime
report_id: Runtime200-borrowed-dispatch-route-sharing-2026-08-31
date: 2026-08-31
session_id: root-runtime-interface03-activate-link-failure-20260831
implementation_status: implementation_complete
validation_status: managed_validation_queued
---

# Runtime200 Borrowed Dispatch Route Sharing

## Scope

Pointer and navigation dispatch previously cloned their owned route into the result and cloned it
again into handler contexts. Pointer routing also materialized candidate vectors, while the hit path
stored both root-to-leaf and bubble order for the same nodes. This slice removes those event-hot-path
copies without changing route order, preview/target/bubble behavior, stacked-hit precedence,
capture, focus, passthrough, blocking, dirty requests, damage requests, or wire compatibility.

The follow-up review found one allocation left after those copies were removed: both dispatchers
created a fresh `HashSet` for visited-node membership on every routed event. Typical UI routes are
shallow, so paying a heap allocation before visiting 1-10 nodes is the wrong shape.

## Implementation

- Dispatch contexts are event-lifetime borrowed views. Higher-ranked handler bounds accept any
  route lifetime, so callbacks cannot retain the ephemeral route.
- Dispatchers accumulate only result fields while borrowing the route, then move the one owned
  route into the terminal result exactly once.
- Pointer candidates stream an optional injected target followed by the existing candidate slice.
  Navigation candidates borrow the selected route slice. Neither path clones a candidate vector.
- `UiHitPath` retains one root-to-leaf sequence and exposes bubble order as a reversible iterator.
  `UiPointerRoutingPath::HitPath` reuses that physical sequence for ordinary dispatch and owns an
  explicit sequence only when capture or redirection changes the route.
- Manual serde projections retain the existing route fields and defaults on the wire. The borrowed
  handler contexts are no longer transport DTOs.
- Pointer and navigation dispatch now share `UiDispatchVisitedNodeSet`. Its first 16 unique node
  identities live in a fixed stack array with bounded linear membership checks. The seventeenth
  unique node promotes once to a capacity-sized `HashSet`, seeds it with the inline identities, and
  preserves deep/multi-branch route deduplication and expected hash complexity.

## Deterministic Performance Evidence

The v3 pressure model is bound to nine current-source files and six exact historical baseline blobs.
Current source is bound to `HEAD 050d8e6c36cd1bf4f3ab0d8fc4df0864c1c29a3f`; the comparison baseline
is frozen separately at `5ffc4945095a6fc734bcbb2e632958026350b760`. The combined source
manifest is `A2F50F98FC386D245ECFC4FBF2E0C1AB2570B6224233E6771109A383427A2C8C`.
The artifact is
`E:\zircon-profiles\runtime-ui-dispatch-route-sharing-pressure-20260831-r4.json`
(SHA-256 `B97ACA0D0A7F5AE561718E051955560990CAA628065D85DEC0B32CE655F5FBAA`).
The workload models 1,000,000 events at route depths
1, 10, and 100 with one and four callbacks per node phase. Both variants retain the same visited-set
work; the depth-100/four-callback case performs 100,000,000 visited-node insertions.

| Depth-100 metric, 1,000,000 events | HEAD baseline | Borrowed route | Reduction |
|---|---:|---:|---:|
| Pointer route clones | 2,000,000 | 0 | 100.000% |
| Pointer node-identity copies | 616,000,000 | 0 | 100.000% |
| Pointer clone-induced Vec allocations, lower bound | 11,000,000 | 0 | 100.000% |
| Pointer node payload copied, lower bound | 4,928,000,000 bytes | 0 | 100.000% |
| Navigation route clones | 2,000,000 | 0 | 100.000% |
| Navigation node-identity copies | 300,000,000 | 0 | 100.000% |
| Navigation clone-induced Vec allocations, lower bound | 3,000,000 | 0 | 100.000% |
| Navigation node payload copied, lower bound | 2,400,000,000 bytes | 0 | 100.000% |
| Navigation candidate-vector copies | 1,000,000 | 0 | 100.000% |

For the visited set, the shared-ancestry pointer fixture treats each stacked candidate as one new
leaf over the target ancestry. A depth-10 route with four stacked candidates therefore visits 14
unique nodes and removes 1,000,000 pointer plus 1,000,000 navigation visited-set heap allocations.
The conservative disjoint-ancestry upper bound is 40 unique pointer nodes and correctly retains the
1,000,000 HashSet fallbacks. Depth 100 also retains the fallback. The optimization removes typical
shallow allocations; it does not claim that every route fits inline.

These are deterministic copy-work lower bounds derived from the exact old and current structures.
They exclude allocator metadata, spare Vec capacity, scalar fields, callback CPU, cache locality,
RSS, and product input latency. Product P50/P95/P99 and allocator counters remain managed dynamic
acceptance evidence; this record does not infer them from the model.

## Local Validation

- `python -m unittest tools.tests.test_runtime_ui_dispatch_route_sharing_performance_contract`
  plus `tools.tests.test_runtime_ui_dispatch_route_sharing_pressure`: 14/14 passed.
- The pressure workload reproduced all copy counts, separated shared/disjoint pointer ancestry, and
  preserved the visited-node insertion invariant while removing shallow-route heap allocation.
- Python bytecode compilation, Rust 1.94.1 formatting, and scoped diff checks passed.
- Rust unit regressions cover inline deduplication and one-time deep-route promotion; they are
  authored but not Cargo-executed in this slice.
- The separate RuntimeInterface03 Clone-contract batch remains asynchronous; no Cargo result is
  reused or polled here.
- A current-source guard recovery batch updated the navigation and pointer ownership contracts from
  the removed direct `HashSet::with_capacity(...)` spelling to
  `UiDispatchVisitedNodeSet::with_expected_len(...)`. The navigation guard now also proves direct
  borrowing from `route.bubbled` / `route.root_targets`, `route: &route` handler contexts, and zero
  `route.clone()` calls. Route sharing, pointer input ownership, navigation ownership, and pointer
  trace contracts pass 25/25; Python bytecode and scoped diff checks are green.
- Static recovery ticket `90a95351009e49e399a636f92e43a81a` was queued by request
  `runtime200-route-static-recovery-20260831-r2` (coordinator receipt
  `6f42dd7480764bf9ae4274de9c9d3a64`). It is asynchronous and is not used as terminal evidence in
  this record.

## Managed Acceptance

One Windows `--locked --release --jobs 1` coordinator batch must compile and test
`zircon_runtime_interface` and `zircon_runtime`, run focused UI route behavior tests, run both Python
contracts, and reproduce the pressure model. Commit, push, optimization-record finalization, and
WeCom publication remain gated on terminal managed evidence.

The shared Runtime200 dynamic batch groups this route-sharing slice with pointer-hover hot paths.
Its single `runtime200_` behavior filter includes both inline visited-set regressions plus the hover
behavior tests; one ignored-filter pass runs the release P95 benchmark. The coordinator ticket is
`776ac58291114580ab254879d2f7fea4`, submitted by request
`runtime200-ui-route-performance-batch-20260831-r1` (receipt
`01dd978ba6064af3963ee5126cae0822`) from snapshot `2491`; its manifest is superseded by the
current record hash after this reconciliation. A final current-source refresh will be submitted
after all record text is stable.
