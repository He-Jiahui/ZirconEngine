# Runtime09E Indexed Shadow Preemption Lookup

Plan: docs/plans/optimize/zircon_runtime/09e-direct-lighting-clustered-shadow-review.md
Milestone: M10 focused CPU-budget slice
Status: release_validation_submitted
Files: ["docs/plans/optimize/zircon_runtime/09e/2026-08-26-indexed-shadow-preemption-lookup.md","tools/tests/test_runtime09e_shadow_atlas_preemption_performance_contract.py","zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/allocator.rs","zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/allocator/tests.rs"]

- Date: 2026-08-31
- Baseline: `0aeb32c037cf30028d7a8950ce373ae052c97c38` / epoch `576`
- Integration owner: `root-runtime09e-indexed-preemption-release-r2-20260831`
- Validation request: `af14f40695834246a376bb4a34752bc7`
- Plan items: allocator retention/preemption policy kernel, P1-16 stable-frame CPU work, M10
  `1/100/10k lights` and atlas-pressure performance evidence

## Problem

On every oversubscribed shadow-atlas frame, `update_preemption_contention(...)` visited every
retained slot and linearly searched the complete `planned` slice for its incumbent. It then scanned
the complete slice a second time for challengers even when no challenger could reach the required
priority margin. The frame constructed a `planned_by_key` map immediately afterward and sorted the
same planned slice before allocation, so both avoidable scans duplicated work that the frame already
paid for.

For `N` retained planned slots with equal priority and the default `1.25` preemption multiplier, the
old contention kernel performs `N * (N + 1) / 2` incumbent comparisons plus `N * N` challenger
visits and produces no active pair. This is a common stable pressure case rather than a synthetic
semantic exception: equal-priority slots cannot preempt one another at a 25% margin.

## Scope Delivered

- The existing `compare_planned_slots` sort moves before contention. It remains the frame's only
  planned-slot sort and preserves the later allocation order.
- The existing `planned_by_key` map is constructed once after sorting and is shared by contention
  incumbent lookup and retained-slot projection.
- Contention incumbent resolution uses one hash lookup per retained slot instead of a linear scan.
- Because the planned slice is ordered by normalized priority descending, challenger traversal stops
  at the first entry below the incumbent's required threshold. Tier and self-pair filters remain
  unchanged within the qualifying prefix.
- A multi-incumbent behavior test records all ten qualifying pairs for priorities `1..=5`, guarding
  against an early-break regression.

## TDD And Verification Evidence

- RED: the focused Python contract failed `2/2` checks on the prior source. It observed that
  `planned_by_key` was built after contention and that the helper still used `planned.iter().find`.
- GREEN after the implementation: the focused contract passed `2/2` source checks.
- A benchmark-presence/P95-gate contract was then added RED and became GREEN after the ignored Rust
  release benchmark was added; the final focused contract passes `3/3`.
- `rustfmt +1.94.1` completed for both owned Rust files.
- The existing confirmed-priority preemption test remains in the same allocator suite. The new
  multi-incumbent behavior test and ignored crate release benchmark are pending managed coordinator
  compilation because direct Cargo execution is prohibited.

## Performance Evidence

The independent release-mode Rust model under `.codex/state/session-coordinator` reproduces the old
linear incumbent lookup and full challenger scan versus the new indexed lookup and bounded priority
prefix. It uses 4,096 retained/planned slots, equal priority, default multiplier `1.25`, three warmup
pairs, and 21 alternating legacy/indexed sample pairs.

Deterministic structural work per modeled frame:

- incumbent lookup: `8,390,656` linear comparisons -> `4,096` hash probes;
- challenger traversal: `16,777,216` visits -> `4,096` visits;
- combined modeled visits/probes: `25,167,872 -> 8,192`, a `99.9675%` reduction;
- active preemption pairs: `0 -> 0`.

Two clean `rustc +1.94.1 -O` preflight executions on 2026-08-26 produced:

| Run | Legacy P50 | Indexed P50 | P50 reduction | Legacy P95 | Indexed P95 | P95 reduction |
|---|---:|---:|---:|---:|---:|---:|
| A | 38.184 ms | 0.338 ms | 99.1159% | 84.150 ms | 0.535 ms | 99.3638% |
| B | 31.558 ms | 0.396 ms | 98.7439% | 48.720 ms | 0.593 ms | 98.7835% |

The owned ignored benchmark uses the real `PlannedShadowSlot`, `RetainedShadowSlot`, comparator, and
optimized allocator helper with the same workload. Its acceptance gate requires
`indexed_p95_ns * 20 <= legacy_p95_ns`, or at least 95% lower P95. The independent model clears that
gate in both runs. Those runs are historical preflight evidence, not terminal current-source
acceptance. Coordinator request `af14f40695834246a376bb4a34752bc7` will run the static contract and
the two filtered Rust tests, including the ignored crate release benchmark, against the exact
four-path snapshot. The terminal marker must be
`RUNTIME09E_SHADOW_PREEMPTION_BENCH_V1` and must satisfy the same 95% P95 gate before this slice can
be integrated.

## Remaining Scope

This slice removes redundant CPU work inside the existing allocator policy kernel. It does not close
Runtime09E's authoring, unified shadow-plan authority, visibility, persistent depth cache, per-frame
allocation, GPU submission, atlas gutter, device-loss, same-quality Unreal comparison, or full
1/100/10k light product matrix requirements. Those remain open and must not be inferred from this
focused kernel result.
