---
title: Runtime Hybrid GI Sideband Binary Lookup 531
category: zircon_runtime
report_id: Runtime531-hybrid-gi-sideband-binary-lookup-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_snapshot_stale
---

# Runtime Hybrid GI Sideband Binary Lookup 531

Hybrid GI probe encoding previously scanned both prepared probe sideband arrays for every resident
probe. The provider constructs these arrays from a sorted `BTreeSet` of probe IDs, so the renderer
now verifies strict ordering once per prepared frame and uses binary lookup for the canonical path.
Reordered or duplicate sidebands retain the prior linear first-match behavior.

The ignored Release evidence `RUNTIME531_HYBRID_GI_SIDEBAND_BINARY_LOOKUP_BENCH_V1` models 32,768
full probe frames with 16 resident probes and two 16-entry sidebands. Including both ordering scans,
the comparison model drops from 8,912,896 legacy candidate checks to an indexed upper bound of
6,225,920, a 30.14% reduction. This is a comparison-count model, not an end-to-end render-time or
GPU-time claim.

## Static evidence

- TDD RED: both per-probe sideband lookups used `iter().find(...)`.
- TDD GREEN: canonical sidebands use `binary_search_by_key` after one strict-order scan.
- A regression proves reordered duplicate IDs still return the first entry, matching legacy
  behavior.
- The production provider collects relevant probe IDs through `BTreeSet`, establishing the
  canonical sorted fast path.
- `rustfmt 1.94.1 --edition 2024` passes.
- `git diff --check` passes (PowerShell reports the repository LF/CRLF notice).
- Source SHA-256:
  `992d6e3bc6b9160a70c8b41385a6aaf0e92c803f4f828b4d029bdb9a468611be`.

## Acceptance gates

1. Managed Windows native Release compilation and focused Runtime tests pass.
2. Canonical and reordered/duplicate sideband behavior regressions remain green.
3. The ignored evidence emits the Runtime531 marker and at least 30% modeled comparison reduction.
4. Commit and push remain coordinator-owned; WeCom publication follows accepted validation only.

No direct Cargo validation, compile, commit, push, or WeCom success is claimed by this record.

## Managed validation result (2026-08-30)

The Runtime530/531 batch ticket `5d35546c649e4e16aea4544e2b7a782e` stopped before Cargo in
`materialization / owned_overlay`. Job `376b92b4427c4058abc45a89c18b7da7` reported
`validation_copy_attribution_stale` for this session's earlier Runtime506 dependency
`zircon_runtime/src/text/font/database/system_fonts.rs`. The Runtime531 source hash remained exact;
no compile, test, performance, commit, push, or WeCom success evidence was produced. Recovery
validation is pending after exact Runtime506 attribution repair.

The first recovery ticket `b61165051e4a43ba8448974c85716a6c` then reached the same
`materialization / owned_overlay` stage and exposed the next historical attribution dependency,
Runtime508 `zircon_runtime/src/ui/surface/surface/default_interactions/radio.rs`. Job
`99ab1ce5fa27435896e247762e438105` stopped before Cargo. The current Runtime508 blob differs from
its original manifest only by rustfmt import ordering and is now exactly re-attributed; aggregate
ownership scans report no remaining Runtime/Editor attribution blocker.
