---
title: Runtime09H2 Borrowed Active Resource Index
category: zircon_runtime
report_id: Runtime09H2-borrowed-active-resource-index-2026-08-26
date: 2026-08-26
session_id: root-runtime09h2-borrowed-active-resources-20260826
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime09H2 borrowed active resource index

## Scope

- Parent scope: the Runtime09H2 post-process descriptor filtering path and its CPU performance qualification.
- Baseline: `8e56165c4c789416c328898d3d8937d934b52efa`, epoch `443`.
- Owners: active post-process graph resource filtering, its source/performance contract, and this record.
- This slice removes temporary ownership of active resource names only. It does not close HDR correctness, effect algorithms, GPU timing, resource metadata, quality tiers, visual oracles, or the remaining Runtime09H2 acceptance gates.

## Change

- Active post-process graph resources now use a borrowed `HashSet<&str>` instead of cloning graph-owned names into a `BTreeSet<String>`.
- The builder counts enabled-effect resource references once and reserves the hash table to that graph-derived upper bound.
- Initial, required, and produced resource names are projected through `String::as_str`; filtering and optional-provider removal consume the borrowed set directly.
- The set is used only for membership and removal. Descriptor, pass, and resource output order remain driven by their existing vectors, so hash iteration order is not observable.

For a 4,096-effect stack, the model removes all per-name allocations. The only optimized allocation is the reserved hash table.

## TDD and local evidence

- RED: `python -m unittest tools.tests.test_runtime09h2_borrowed_active_resource_index_performance_contract -v` initially failed 4/4 because the old implementation used `BTreeSet<String>`, cloned names three times, and did not reserve a graph-derived bound.
- GREEN: the same command passes 4/4 after the borrowed index is implemented.
- `rustfmt +1.94.1 --edition 2021 --config skip_children=true` completed for the owned Rust file.
- The standalone model is compiled with `rustc +1.94.1 -O`; it does not use Cargo or the shared build lane.

The deterministic Rust model measures 31 alternating legacy/optimized sample pairs, with 16 index builds and membership sweeps over 4,096 effects per sample. It retains enabled-effect filtering, duplicate resource names, optional-provider removal, membership checks, exact hit-count parity, and a process-local allocation counter. Three local runs passed the acceptance thresholds; the table records the latest run.

| Metric | Owned `BTreeSet<String>` | Borrowed preallocated `HashSet<&str>` | Change |
|---|---:|---:|---:|
| P50 | 273.3779 ms | 78.6490 ms | -71.231% |
| P95 | 400.3522 ms | 180.0531 ms | -55.026% |
| allocations / index and sweep | 13,181 | 1 | -99.992% |

The other two runs produced P50 reductions of 68.630% and 64.845%, P95 reductions of 47.326% and 69.579%, and the same 99.992% allocation reduction. These timings isolate CPU active-resource indexing and membership checks; they do not claim GPU frame time or complete post-process latency.

## Async validation

One coordinator batch must run the four Python source contracts, all 19 focused post-process route Rust tests in the real `zircon_runtime` crate, Rust formatting checks, scoped diff checks, exact model parity, and the same performance workload.

Acceptance requires 4/4 source contracts and 19/19 Rust tests to pass, exact hit-count parity, allocation reduction of at least 99%, and P50/P95 reductions of at least 40%. Integration and automatic WeCom publication remain coordinator-owned after managed validation succeeds. The WeCom message must include managed P50/P95 and allocation reductions and label them as 4,096-effect CPU active-resource-index evidence.
