---
title: Editor23 Matched Style Path Single Projection
category: zircon_editor
report_id: Editor23-matched-style-path-single-projection-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor23 Matched Style Path Single Projection

## Scope

This slice removes the intermediate document-node path allocation from matched-style inspection.
Target lookup, root host semantics, component/control/class/state references, selector matching,
missing-node behavior, cascade ordering, and declaration projection remain unchanged.

## Change

- Build borrowed `StyleMatchNode` entries directly during the document DFS.
- Pass active states and root-host identity into traversal rather than remapping a completed tuple
  path.
- Preserve traversal backtracking by popping the direct match entry when a branch misses.

## Deterministic Performance Evidence

| Depth 2,048, 128 path builds per sample | Before | After |
|---|---:|---:|
| Path allocations per sample | 256 | 128 |
| Intermediate tuple entries written | 262,144 | 0 |
| Final match entries written | 262,144 | 262,144 |
| Document DFS passes | 128 | 128 |

The ignored release gate runs 17 alternating sample pairs and emits
`EDITOR23_MATCHED_STYLE_PATH_SINGLE_PROJECTION_BENCH_V1`. Acceptance requires direct projection P95
to be at least 15% below tuple-path remapping. Exact Windows timings remain pending the coordinator
run.

## Acceptance

- `optimization_batch_20260826au_matched_style_path_preserves_host_and_descendant_matching` covers
  host/child/descendant/state matching plus missing-node behavior.
- `optimization_batch_20260826au_matched_style_path_uses_single_projection` requires the direct
  match-path builder and rejects the intermediate tuple path.
- `optimization_batch_20260826au_matched_style_path_single_projection_p95` reports paired P50/P95
  samples and enforces the 15% P95 reduction gate.

## Remaining Parent-plan Work

Editor23 still owns schema-backed preview data, typed diagnostics, incremental validation, preview
fidelity, bindings, transactions, cook artifacts, and large-asset gates. This slice only converges
matched-style path construction.
