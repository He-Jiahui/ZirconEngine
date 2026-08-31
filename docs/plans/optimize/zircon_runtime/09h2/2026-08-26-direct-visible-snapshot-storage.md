---
title: Runtime09H2 Direct Visible Snapshot Storage
category: zircon_runtime
report_id: Runtime09H2-direct-visible-snapshot-storage-2026-08-26
date: 2026-08-26
session_id: root-runtime09h2-direct-visible-snapshot-storage-20260826
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime09H2 direct visible snapshot storage

## Scope

- Parent scope: Runtime09H2 per-frame render-framework bookkeeping, specifically publishing the immutable visible-spatial-query snapshot for a viewport.
- Baseline: `8e56165c4c789416c328898d3d8937d934b52efa`, epoch `443`; `viewport_record.rs` source blob `d64770e4066792502cd1709c01339c4f5ff9caa7`; `visible_spatial_query.rs` source blob `31c745e1c0aa97e4a799bcc394ab10d2e2d8b9a5`.
- Owners: the viewport-record visible snapshot field, its store/getter implementation and focused Rust contracts, the standalone allocation/timing model, and this record.
- This slice preserves the public owned-snapshot return type, immutable generation identity, replacement behavior, and shared ownership of the opaque query implementation. It does not change visibility construction/query algorithms, capture readback ownership, GPU effects, or the remaining Runtime09H2 acceptance gates.

## Change

- `ViewportRecord` now stores `RenderVisibleSpatialQuerySnapshot` directly instead of wrapping it in a second `Arc`.
- The snapshot's opaque `RenderVisibleSpatialQuery` remains held by its existing inner `Arc`, so cloning the public snapshot continues to share the immutable query implementation.
- Per-frame publication now allocates only the necessary inner query handle. The getter clones that handle from `Option::as_ref` without consuming viewport storage.
- A direct Rust contract retrieves the same stored generation twice, replaces it with a newer generation, and proves both previously returned owned snapshots remain valid.

The former outer `Arc` was allocated on every successful viewport publication but was immediately dereferenced and cloned into the public value on reads. No caller received or shared that outer allocation, so direct storage removes it without widening the API or changing lock-release behavior.

## TDD and local evidence

- RED: `python -m unittest tools.tests.test_runtime09h2_direct_visible_snapshot_performance_contract -v` initially failed 4/4 because direct storage, direct construction, the `as_ref().cloned()` getter, and the direct Rust contract were absent.
- GREEN: the same focused source contract passes 4/4 after implementation.
- `rustfmt +1.94.1 --edition 2021 --config skip_children=true` and scoped `git diff --check` pass.
- The standalone model is compiled with `rustc 1.94.1 -O`; it does not use Cargo or the shared build lane.

The deterministic Rust model measures 31 alternating outer-Arc/direct-storage sample pairs. Each sample publishes 262,144 immutable snapshots, retaining the necessary inner query `Arc` in both variants. Every pair produces checksum `13862435951319318528` for both representations. Four sequential local runs passed the acceptance thresholds; the table records the final run.

| Metric | Outer snapshot Arc | Direct snapshot storage | Change |
|---|---:|---:|---:|
| P50 | 41.7864 ms | 20.4227 ms | -51.126% |
| P95 | 84.7185 ms | 29.2069 ms | -65.525% |
| allocations / 262,144 publications | 524,288 | 262,144 | -50.000% |

The other three sequential runs produced P50 reductions of 51.763%, 50.206%, and 48.370%, P95 reductions of 52.795%, 53.831%, and 57.767%, and the same 50.000% allocation reduction. These timings isolate CPU snapshot publication and do not claim complete visibility extraction or frame-time improvement.

## Async validation

The coordinator validation batch must run the four focused Python source contracts, both viewport-record visible-query Rust tests in one Cargo filter, Rust formatting checks, scoped diff checks, exact model parity, and the same performance workload. The batch may combine this slice with other ready Runtime09H2 tickets; it must not serialize one Cargo request per source contract.

Acceptance requires 4/4 source contracts and 2/2 Rust tests to pass, identical checksum `13862435951319318528` for both representations, `524,288` legacy versus `262,144` direct allocations, P50/P95 reductions of at least 40%, and an allocation reduction of at least 50%. Cargo validation remains required even while the foreign tracked deletion of `zircon_runtime/src/core/framework/render/environment/skybox.rs` prevents workspace compile-time input closure planning. Integration and automatic WeCom publication remain coordinator-owned after managed validation succeeds. The WeCom message must include managed P50/P95 and allocation reductions and label them as CPU visible-snapshot publication evidence for 262,144 publications.
