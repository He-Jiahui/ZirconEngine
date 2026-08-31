---
title: Runtime94 HZB Params Hash Workspace
category: zircon_runtime
report_id: Runtime94-hzb-params-hash-workspace-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime94 HZB Params Hash Workspace

## Scope

This slice replaces the HZB occlusion-parameter workspace owner with `HashMap`. Every prepare call
resolves its stable workspace ID through expected constant-time lookup before deciding whether to
create a GPU buffer or upload changed arguments.

The private owner has no iterator, snapshot, Debug, or serialization contract. Per-workspace
buffer identity, argument-change upload suppression, initialization accounting, and GPU resource
lifetime are unchanged.

## Performance Workload

The release workload fills 4,096 workspace IDs and performs 4,096 stable hits for the final ID.

| Work per workload | Before | After |
|---|---:|---:|
| Ordered workspace lookups | 4,096 | 0 |
| Hash workspace lookups | 0 | 4,096 |
| Workspace-order policy changes | 0 | 0 |
| Allocations on workspace hits | 0 | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`RUNTIME94_HZB_PARAMS_HASH_WORKSPACE_BENCH_V1`. Acceptance requires hash lookup P95 to be at least
30% below the legacy `BTreeMap` path. Exact Windows P50/P95 timings remain pending the coordinator
run.

## Acceptance

- `runtime94_hzb_params_hash_workspace_reuses_each_workspace_buffer` covers
  same-ID reuse, separate-ID isolation, and post-admission-commit upload suppression under the
  current two-phase prepare/commit protocol.
- `runtime94_hzb_params_hash_workspace_has_no_order_contract` locks the
  unordered owner and absence of traversal.
- `runtime94_hzb_params_hash_workspace_p95` reports paired release P50/P95
  samples and enforces the 30% P95 reduction gate.
- These checks remain pending the copy-complete HZB two-phase upload union. The current workspace
  signature and commit protocol require its culler, graph-execution, and renderer submission
  callers, so this slice is intentionally excluded from the self-contained Runtime94 index batch.

## Remaining Parent-plan Work

Runtime94 still owns canonical bounds, persistent render-scene lifecycle, per-view HZB history,
two-phase retest, phase coverage, GPU timings, and product qualification. This slice only
converges the HZB parameter workspace index.
