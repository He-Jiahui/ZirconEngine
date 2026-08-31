---
title: Runtime09F1 Borrowed IBL Artifact Dispatch
category: zircon_runtime
report_id: Runtime09F1-borrowed-ibl-artifact-dispatch-2026-08-27
date: 2026-08-27
session_id: root-runtime09f1-borrowed-ibl-dispatch-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime09F1 Borrowed IBL Artifact Dispatch

## Scope

`resolve_ibl_bake_artifact_runtime_dispatch` previously cloned every asset-derived IBL blob into an
owned candidate list, cloned a runtime-cache hit into the same list, and then cloned the selected
blob again into the resolved payload. An IBL blob owns its raw `Vec<u8>` PMREM/SH9/IEM payload, so
each candidate clone copied the complete artifact.

The runtime dispatch now borrows the input blobs. It projects only their copy-sized descriptors for
the existing canonical selector and clones exactly the selected blob into the owned result. Asset
priority, runtime-cache fallback, rejected-candidate accounting, current-descriptor checks, and the
runtime-compute fallback are unchanged. The existing public owned-candidate resolver remains
available for callers that already own candidates.

## Behavior Evidence

- `borrowed_blob_resolution_preserves_asset_priority_and_rejection_count` covers a stale asset, a
  current asset, and a current runtime cache while preserving first-tier asset selection.
- `borrowed_blob_resolution_uses_current_runtime_cache_after_stale_assets` covers cache fallback and
  rejected-candidate accounting.
- `borrowed_blob_resolution_requests_compute_when_every_blob_is_stale` covers the terminal fallback
  and dispatch count.
- `test_runtime09f1_borrowed_ibl_artifact_dispatch_performance_contract.py` locks the borrowed
  signature, descriptor-only projection, complete cache-read matching, and removal of candidate blob
  clones from runtime dispatch.

## Deterministic Performance Model

The release model uses seven asset-derived candidates plus one runtime-cache candidate. Every blob
owns a 256 KiB payload, the current asset is last in asset order, and each timed sample executes 32
complete selections. Twenty-one alternating sample pairs are used per run.

| Metric | Owned candidate path | Borrowed candidate path | Reduction |
|---|---:|---:|---:|
| allocations per dispatch | 11 | 2 | 81.818% |
| allocated bytes per dispatch | 2,360,136 | 262,272 | 88.887% |

| Run | Owned P50 ns | Borrowed P50 ns | Reduction | Owned P95 ns | Borrowed P95 ns | Reduction |
|---:|---:|---:|---:|---:|---:|---:|
| 1 | 75,521,100 | 1,914,500 | 97.465% | 222,833,800 | 7,448,900 | 96.657% |
| 2 | 102,790,500 | 5,029,900 | 95.107% | 296,292,600 | 56,984,500 | 80.767% |
| 3 | 48,534,500 | 1,851,400 | 96.185% | 65,686,900 | 3,082,200 | 95.308% |

The three-run worst case reduces P50 by 95.107% and P95 by 80.767%. The parity checksum is
`18440808946618895332`. The model gate requires at least 75% fewer allocations, 75% fewer allocated
bytes, 60% lower P50, and 45% lower P95.

## Validation

Passed locally without Cargo:

- 3/3 Python source contracts;
- Rust 1.94.1 formatting checks for the owned Rust paths;
- scoped `git diff --check`;
- three independent optimized model runs with identical nonzero checksum and all gates satisfied.

The managed coordinator validation must run the three focused Rust behavior tests, the source
contracts, formatting, scoped diff, and the release performance model in one batch. Cargo validation
is not claimed until that asynchronous ticket reaches a passing terminal state.

## Remaining Parent-Plan Work

This slice removes full-payload candidate cloning only. Runtime cache I/O, manifest-first warm hits,
chunked/platform-specific artifacts, submission-thread I/O removal, and GPU IBL scheduling remain
owned by the Runtime09F1 parent plan.
