---
title: Runtime206 Incremental Project Resource Publication Capacity
category: zircon_runtime
report_id: Runtime206-incremental-publication-capacity-2026-09-15
date: 2026-09-15
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime206 Incremental Project Resource Publication Capacity

## Scope

The project-asset-manager incremental watch/import projection builds four bounded temporary
collections from the changed and previous resource slices. They previously started empty and grew
geometrically even though each input slice already supplied a safe upper bound.

## Implementation

- Reserve `updated_records.len()` for the updated-record identity map.
- Reserve `previous_source_records.len()` for removed locators and source-path removals.
- Reserve the deduplicated updated-record count for source-path updates.
- Preserve record-ID deduplication, source-path filtering, hash-set uniqueness, and the existing
  unordered materialization boundary for the two map/set projections.
- Add a lower capacity regression and the ignored
  `RUNTIME786_INCREMENTAL_PUBLICATION_CAPACITY_BENCH_V1` marker.

No persistence, generation, publication ordering, or watch authority semantics were changed.

## Deterministic performance model

For four collection families and input lengths 65,536, 32,768, 16,384, and 8,192, the legacy
geometric-growth model records 216 growth events (four families across the four lengths); the
capacity-hinted path records zero growth events. This is allocation-shape evidence only. It is not
a managed allocator, CPU/RSS, or product p50/p95/p99 acceptance result.

## Local validation

- TDD source contract `tools/tests/test_runtime206_incremental_publication_capacity_performance_contract.py`
  was intentionally RED before the implementation and is GREEN at `3/3`.
- The focused Runtime206/Runtime85 source-contract batch passes `23/23` in one process, including
  the existing type-posting, secondary-query, referencer, registry-build, and project-root guards.
- The refreshed non-tooling Runtime/Editor performance-contract loader covers `552` modules and
  passes `1975/1975` tests in one process (`4.790s`); this is local source/model evidence only.
- The broader non-tooling Runtime/Editor Python regression discovery passes `3724/3724` across
  `915` modules in one process (`326.952s`), with zero failures, errors, or skips.
- The lower module contains the upper-bound regression and an ignored Release benchmark marker.
- The new contract is queued for the existing one-process Runtime/Editor non-tooling batch; no
  standalone Cargo process or coordinator status query was issued.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/asset/pipeline/manager/project_asset_manager/resource_publication.rs` | `DCF6B806E563EE25A9FD1E3CBD8497A3A402A07C16C4501DDB809C34A8DDCB84` |
| `zircon_runtime/src/asset/pipeline/manager/project_asset_manager/resource_publication/incremental_capacity_tests.rs` | `4CBB9B1892CC1AFC5AD89338219D946A1331D3D3B7BACD6631FE1048FDA74BC1` |
| `tools/tests/test_runtime206_incremental_publication_capacity_performance_contract.py` | `21B58A3362BDEDE58EF14B2AA47AF4A581C25E07DC0D182045AAC83962F0EB8C` |

## Managed validation boundary

This slice joins the owner-attributed Runtime/Editor Windows batch. Managed Cargo/Release
compilation, ignored benchmark execution, allocator evidence, and product percentile evidence
remain pending under the existing external `E:\Git\zr_vm` dirty-worktree admission boundary.
