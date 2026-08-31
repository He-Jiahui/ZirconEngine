---
title: Runtime09H2 Stable Terminal Resource Cache
category: zircon_runtime
report_id: Runtime09H2-stable-terminal-resource-cache-2026-08-26
date: 2026-08-26
session_id: root-runtime09h2-stable-terminal-cache-20260826
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime09H2 stable terminal resource cache

## Scope

- Parent scope: the Runtime09H2 terminal post-process resource cache, specifically repeated lookups of cached physical viewport-region uniforms and SMAA backing textures.
- Baseline: `8e56165c4c789416c328898d3d8937d934b52efa`, epoch `443`; source blob `e0029b38997da1021489c8386e9dd7fcea16ed90`.
- Owners: `BoundedResourceCache`, its focused source and Rust contracts, the standalone timing/movement model, and this record.
- This slice preserves resource construction, fixed cache capacities, resource identity, and exact LRU eviction. It does not close output transfer correctness, HDR display contracts, terminal UI composition, resource readiness, GPU effects, or the remaining Runtime09H2 acceptance gates.

## Change

- Cache entries now carry a monotonically increasing `last_used` epoch in stable vector slots.
- A cache hit updates its slot in place and clones only the returned `Arc`; it no longer removes the entry, shifts every later entry, and pushes the hit to the vector tail.
- A full-cache miss scans the bounded slots for the smallest epoch and replaces that one slot, preserving exact least-recently-used eviction without shifting the vector.
- A direct Rust contract warms two entries, promotes the first without changing slot order, inserts a third, and proves the true oldest entry was replaced.

The production physical-region cache is bounded at 16 entries. A recurring 16-region sequence previously shifted 15 entries on every hit after warmup; stable slots eliminate those moves while retaining the same lookup result and eviction order.

## TDD and local evidence

- RED: `python -m unittest tools.tests.test_runtime09h2_stable_terminal_cache_performance_contract -v` initially failed 4/4 because stable entry metadata, in-place hits, stable-slot eviction, and the direct LRU Rust contract were absent.
- GREEN: the same focused source contract passes 4/4 after the stable-slot cache is implemented.
- A local batch of every Runtime09H2 performance source contract passes 40/40.
- `rustfmt +1.94.1 --edition 2021 --config skip_children=true` and scoped `git diff --check` pass.
- The standalone model is compiled with `rustc 1.94.1 -O`; it does not use Cargo or the shared build lane.

The deterministic Rust model measures 31 alternating legacy/stable sample pairs. Each sample performs 65,536 rounds through 16 already-resident keys in recurring order, matching a fully warm 16-region cache under repeated multi-view use. Every pair must produce rolling checksum `17080002212837752832`. Four local runs passed the acceptance thresholds; the table records the final run.

| Metric | Remove-and-push LRU | Stable-slot LRU | Change |
|---|---:|---:|---:|
| P50 | 16.4227 ms | 10.6399 ms | -35.212% |
| P95 | 21.4326 ms | 12.5377 ms | -41.502% |
| shifted entries / sample | 15,728,640 | 0 | -100.000% |

The other three runs produced P50 reductions of 37.297%, 35.482%, and 34.442%, P95 reductions of 39.157%, 33.906%, and 34.449%, and the same complete elimination of hit-path entry shifts. These timings isolate CPU cache bookkeeping with already-resident resources; they do not claim GPU render-pass or complete frame-time improvement.

## Async validation

One coordinator batch must run the four focused Python source contracts, all six terminal-resource-cache Rust tests in one Cargo filter, Rust formatting checks, scoped diff checks, exact model parity, and the same performance workload.

Acceptance requires 4/4 source contracts and 6/6 Rust tests to pass, checksum `17080002212837752832`, exactly zero stable-path shifts versus `15,728,640` legacy shifts, and P50/P95 reductions of at least 30%. The Cargo validation remains required even while the foreign tracked deletion of `zircon_runtime/src/core/framework/render/environment/skybox.rs` prevents workspace compile-time input closure planning. Integration and automatic WeCom publication remain coordinator-owned after managed validation succeeds. The WeCom message must include managed P50/P95 reductions and the eliminated shift count and label them as CPU terminal-cache bookkeeping evidence for one warm 16-region workload.
