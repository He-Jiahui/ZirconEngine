---
title: Runtime09H2 Borrowed Pass Resource Index
category: zircon_runtime
report_id: Runtime09H2-borrowed-pass-resource-index-2026-08-26
date: 2026-08-26
session_id: root-runtime09h2-borrowed-pass-resources-20260826
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime09H2 borrowed pass resource index

## Scope

- Parent scope: the CPU pass-graph execution path in the Runtime09H2 focused production set and its performance/budget qualification.
- Baseline: `8e56165c4c789416c328898d3d8937d934b52efa`, epoch `443`.
- Owners: the post-process pass-graph fallback executor, its source/performance contract, and this record.
- This slice removes per-frame resource-name ownership from the fallback executor only. It does not close HDR correctness, effect algorithms, GPU timing, resource metadata, quality tiers, visual oracles, or the remaining Runtime09H2 acceptance gates.

## Change

- The fallback executor now stores borrowed `&str` resource keys instead of cloning graph-owned names into two `BTreeSet<String>` instances.
- It counts produced references and total resource references once, then reserves both `HashSet` tables to their graph-derived upper bounds.
- External resources are checked when admitted initially and produced resources when admitted after a node executes. Consumer nodes now use the available set directly instead of repeating the immutable binding lookup for every edge.
- Membership remains independent of set iteration order. Nodes are still evaluated and recorded strictly in `graph.nodes` order, so pass ordering and execution receipts are unchanged.
- The already optimized executed-executor mask path is unchanged.

For a 1,024-node graph with one external input and one predecessor edge per node, owned resource-name insertions are removed. The only remaining model allocations are the two reserved hash tables.

## TDD and local evidence

- RED: direct execution of `test_fallback_resource_sets_borrow_graph_names` failed because the old implementation imported `BTreeSet` and cloned graph resource names.
- GREEN: `python -m unittest tools.tests.test_runtime09h2_borrowed_pass_resources_performance_contract -v` passes 4/4.
- `rustfmt --edition 2021 --config skip_children=true` completed for the owned Rust file.
- The standalone model is compiled with `rustc -O`; it does not use Cargo or the shared build lane.

The deterministic Rust model measures 31 alternating legacy/optimized sample pairs, with 32 executions of a 1,024-node graph per sample. A process-local global allocator counts allocation and reallocation calls. Two final local runs of the complete borrowed/admit-once implementation exceeded the acceptance thresholds; the table records the latest isolated run.

| Metric | Owned `BTreeSet<String>` | Borrowed preallocated `HashSet<&str>` | Change |
|---|---:|---:|---:|
| P50 | 95.7691 ms | 20.4878 ms | -78.607% |
| P95 | 193.0320 ms | 60.3851 ms | -68.718% |
| allocations / graph execution | 3,452 | 2 | -99.942% |

The other final run produced a 75.373% P50 reduction, a 65.930% P95 reduction, and the same 99.942% allocation reduction. These timings isolate CPU fallback resource readiness bookkeeping; they do not claim GPU frame time or full post-process latency.

## Async validation

One coordinator batch must run the four Python source contracts, all five focused Rust tests in the real `zircon_runtime` module, Rust formatting checks, scoped diff checks, model parity, and the same performance workload.

Acceptance requires 4/4 source contracts and 5/5 Rust tests to pass, exact execution-count parity, allocation reduction of at least 99%, and P50/P95 reductions of at least 50%. Integration and automatic WeCom publication remain coordinator-owned after managed validation succeeds. The WeCom message must include managed P50/P95 and allocation reductions and label them as 1,024-node CPU fallback resource-index evidence.
