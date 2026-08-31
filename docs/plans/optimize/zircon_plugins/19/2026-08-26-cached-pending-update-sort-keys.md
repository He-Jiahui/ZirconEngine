---
title: Plugins19 Cached Pending-Update Sort Keys
category: zircon_plugins
report_id: Plugins19-cached-pending-sort-keys-2026-08-26
date: 2026-08-26
session_id: root-runtime-interface03-activate-link-failure-20260831
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Plugins19 cached pending-update sort keys

## Scope

- Parent scope: Plugins19 Hybrid GI frame preparation, specifically pending probe-update priority ordering.
- Baseline: `601472078e848164d2221967c55a77fea2452928`, epoch `447`.
- Owned paths: `prepare_frame/collect_pending_updates.rs`, its focused source contract, the shared projection/sort/discovery pressure model and contract, and this record.
- This is a bounded response to the repeated full-rebuild costs described by `HGI-V-P1-19`. It preserves the existing lineage-support, resident-descendant, descendant-count, depth, generation, and probe-ID priority tuple. It does not claim to close persistent slot/page indexing, GPU residency, readback, denoise, or product-quality gates.

## Change

Pending update ordering now uses `sort_by_cached_key` instead of `sort_by_key`. The previous comparator recomputed the full tuple throughout sorting. Each recomputation can rebuild scheduled trace-region data, walk the probe lineage, and construct descendant vectors and visited sets twice. The cached sort evaluates that exact tuple once per retained update and sorts the cached scalar tuple without changing priority semantics.

The direct Rust contract covers generation/probe-ID ordering and ray-budget projection for otherwise equal priority dimensions. The source contract additionally freezes every expensive priority dimension inside the cached key and rejects restoration of the repeated-key comparator.

## TDD and local evidence

- RED: `python -m unittest tools.tests.test_plugins19_cached_pending_sort_keys_performance_contract -v` produced 3 failures and 1 error because the old source used `sort_by_key`, had no cached-key body, and had no direct Rust contract.
- GREEN: the focused source contract now passes 4/4.
- `rustfmt +1.94.1 --edition 2021 --check --config skip_children=true` passes for `collect_pending_updates.rs`.
- Scoped `git diff --check` passes.
- The standalone model compiles with `rustc 1.94.1 -O`; it does not use Cargo or a shared build target.

The deterministic model measures 31 alternating legacy/cached sample pairs over 1,024 pending updates in a binary probe lineage with 64 scheduled trace regions. Runtime-model construction is outside the timed and allocation-counted region. Both algorithms produced checksum `1123984918402528105` in all four runs.

| Metric | Recomputed sort key | Cached sort key | Change |
|---|---:|---:|---:|
| P50 | 77.7146 ms | 3.6246 ms | -95.3360% |
| P95 | 165.3016 ms | 6.0630 ms | -96.3322% |
| allocations / sort | 329,884 | 15,974 | -95.157692% |

The other three runs produced P50 reductions of 95.1057%, 95.4647%, and 95.2740%, and P95 reductions of 94.9174%, 94.0285%, and 93.5188%, with identical allocation counts. These values cover CPU pending-update priority sorting only and do not claim GPU frame-time or image-quality improvement.

## Async validation

### Current-source convergence receipt

- Ownership transfer preview request: `d7b0d0d9f6674ac29b4a1c63800c853c`.
- Ownership transfer apply request: `c776f5230c1449a7ad53d90d3b5ece4a`.
- Applied fingerprint: `0caacce1f20a3d0c48dd892678b0040ce9a71872faa7cc03e9a89dee4d792f0f`.
- Current session: `root-runtime-interface03-activate-link-failure-20260831`.
- Shared static/model ticket: `4c6aa5481e1440819e427ac1568979ab` (queued, 20 Python tests).
- Plugins19 Cargo behavior ticket: `bca76f8bab654428bd5d2a0c0faf3f76` (queued; exact priority-order test).
- Shared model: `tools/plugins_projection_sort_discovery_pressure.py`, source manifest `ECAB605B9EC8C342B696C4CEB59AE2FB270F75976140B7086BFDD497CC18CC05`.
- Current source hashes: `collect_pending_updates.rs` `1895DC44A0A11E45AE98E625FF80100F7AA65A059EEE4E9550692008AE0129F1`; shared model `2F51A8F953D139FE68E7DB1AB87653DFCC769DA01571BC6CC429040974D34323`.

The current-source model is deterministic structural evidence, not wall-clock timing. With 1,024 updates and an explicit 10,240-comparison workload, cached evaluation changes modeled priority-key evaluations `20,480 -> 1,024` (`-95%`) and expensive graph queries `81,920 -> 4,096`, while preserving all six tuple dimensions. Historical alternating release evidence remains the performance claim: P50 `77.7146 -> 3.6246 ms` (`-95.3360%`), P95 `165.3016 -> 6.0630 ms` (`-96.3322%`), allocations `329,884 -> 15,974` (`-95.157692%`), checksum `1123984918402528105`.

The coordinator must run the four focused source contracts, the exact Rust test `hybrid_gi::prepare_frame::collect_pending_updates::tests::cached_pending_update_sort_preserves_priority_order`, Rust formatting, scoped diff checks, checksum parity, and the standalone model. Acceptance requires 4/4 source contracts, exactly one filtered Rust test, checksum `1123984918402528105`, P50 reduction of at least 90%, P95 reduction of at least 80%, and allocation reduction of at least 90%.

The ticket joins the outstanding optimization validation batch and this business Session does not wait for it before subsequent work. Cargo remains coordinator-owned. At the time of this record, foreign unmanaged wave85-wave87 build directories can stop managed copies at `artifact_governance`, and the foreign tracked deletion of `zircon_runtime/src/core/framework/render/environment/skybox.rs` can stop Cargo input-closure planning. Neither path is part of this candidate.

Integration and automatic WeCom publication remain coordinator-owned after managed validation and independent review succeed. The WeCom message must include the managed P50/P95 and allocation reductions and label them as CPU Hybrid GI pending-update priority-sort evidence for the 1,024-update/64-region workload.
