---
title: Editor Asset Summary Append Capacity
category: zircon_editor
report_id: Editor880-asset-summary-append-capacity-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor880 Asset Summary Append Capacity

## Finding

`append_asset_browser_summary_nodes` always appends the same five nodes after
updating the retained name node: continuation, type badge, type, state, and
revision. The direct pushes preserved order but did not reserve that exact
bound. A caller-owned vector whose existing prefix exhausted its capacity could
therefore grow geometrically while materializing a fixed-shape summary.

## Optimization

- Name the fixed append contract as `SUMMARY_APPENDED_NODE_COUNT = 5`.
- Reserve that exact additional count before any of the five nodes is
  materialized.
- Keep the existing five direct pushes; no temporary summary vector or collect
  stage is introduced.
- Preserve the caller-owned prefix, summary control IDs, append order, selected
  asset lookup, text, style, and thumbnail-view removal behavior.

`Vec::reserve` is a no-op when the caller already owns enough spare capacity,
so the change does not impose an allocation on the reused-capacity path.

## TDD and deterministic evidence

The Editor880 source/model contract was observed RED at `2/5` and GREEN at
`5/5`. The lower regression starts with a capacity-exhausted retained prefix,
runs the real append owner, and locks the prefix plus all five control IDs in
their existing order.

For 4,096 representative refreshes starting with one retained prefix slot and
five fixed appends, the retired geometric-growth model performs two destination
growth events per refresh (`8,192` total); the exact-reserve model performs
zero. The ignored 101-pair Release marker
`EDITOR880_ASSET_SUMMARY_APPEND_CAPACITY_BENCH_V1` emits alternating
p50/p95/p99 samples, locks both growth counts, and requires reserved p95 to stay
within 10% of the retired append model.

## Local validation boundary

- Exact-file Rustfmt and scoped `git diff --check` pass.
- Editor879/880 plus the adjacent Asset Browser static-suffix contracts pass
  `28/28` in one local Python batch.
- Lower Rust execution remains owned by asynchronous v10, submitted with
  Editor879 as PID `21968`; no per-task Cargo validation was launched.
- Local evidence does not establish Windows compilation, allocator behavior,
  or Asset Browser product p50/p95/p99 latency.
- Tooling production remains deferred for the later Rust migration.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/layouts/views/asset_browser/summary_nodes.rs` | `D8B142F20CEA72AC2AB4451FD8C7E7FF9EDC21A45CFEC425DCCF83AC373CCDF0` |
| `zircon_editor/src/ui/layouts/views/asset_browser/summary_nodes/capacity_tests.rs` | `D7E267653BB1F296BF94357EFFC7FF8335E9C2A62CE47EE9A2EC3985A7BD56A5` |
| `tools/tests/test_editor880_asset_summary_append_capacity_performance_contract.py` | `52A277189FD2EF1CA2909693C2461F7EDDD57D891D333AC776FF8F8321A99C43` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
a combined current-source Windows lane compiles Editor, executes the lower
regression and ignored Release marker, and supplies allocator plus Asset Browser
product p50/p95/p99 evidence. The deterministic growth model is not product
acceptance.
