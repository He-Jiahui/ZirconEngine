---
title: Runtime Virtual Geometry Cluster Ordinal Fast Path 522
category: zircon_runtime
report_id: Runtime522-virtual-geometry-cluster-ordinal-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime Virtual Geometry Cluster Ordinal Fast Path 522

`cluster_ids_for_stable_instance_key` already returns cluster IDs in sorted, deduplicated order.
`virtual_geometry_cluster_ordinal` nevertheless performed a linear `position` scan for every
cluster emitted into a virtual-geometry plan. The ordinal lookup now uses the established ordering
invariant through `binary_search`, reducing each lookup from O(C) to O(log C). The existing missing
cluster behavior remains ordinal 0 through `unwrap_or_default`.

The ignored Windows Release model
`RUNTIME522_VIRTUAL_GEOMETRY_CLUSTER_ORDINAL_BENCH_V1` fixes 32,768 lookups over 4,096 clusters.
Worst-case comparison bounds fall from 134,217,728 to 425,984, a 9,968 basis-point reduction. This
is a comparison-count model rather than an end-to-end frame-time claim.

## Static evidence

- TDD RED: the production source lacked the sorted-slice `binary_search` binding.
- TDD GREEN: the production segment contains `binary_search` and no linear `position`.
- The focused source regression also preserves the missing-ID default ordinal.
- `rustfmt +1.94.1 --edition 2021 --check` passes.
- `git diff --check` passes (PowerShell reports the repository LF/CRLF notice).
- Source SHA-256:
  `48c6b93eb8969b4d82b9cba7df56f5b22c6548e681a7d2122c0c08b15407144d`.

## Acceptance gates

1. Managed Windows native Release compilation and the focused Runtime test pass.
2. The ignored evidence emits the Runtime522 marker and exact comparison bounds.
3. Existing virtual-geometry plan ordering and missing-cluster fallback behavior remain unchanged.
4. Commit and push remain coordinator-owned; WeCom publication follows accepted validation only.

No direct Cargo validation, compile, commit, push, or WeCom success is claimed by this record.
