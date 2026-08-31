---
title: Runtime Transmission Executor Direct Match 533
category: zircon_runtime
report_id: Runtime533-transmission-executor-direct-match-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_snapshot_stale
---

# Runtime Transmission Executor Direct Match 533

Advanced PBR transmission pass routing previously scanned each four-entry executor-ID array to
recover its fixed step index. Both ID families are compile-time contracts, so routing now uses
exhaustive string matches and preserves `None` for every unknown ID.

The ignored Release evidence `RUNTIME533_TRANSMISSION_EXECUTOR_DIRECT_MATCH_BENCH_V1` models 32,768
frames and all eight terminal transmission lookups per frame. Against the legacy worst-case
four-candidate scan, the contract model drops from 1,048,576 candidate checks to 262,144 direct
match decisions, a 75% reduction. This is a dispatch-decision model, not an end-to-end render-time
or GPU-time claim.

## Static evidence

- TDD RED: both step-index helpers used `iter().position(...)`.
- TDD GREEN: both fixed ID families use exhaustive direct matches.
- A regression enumerates the exported constant arrays and proves every ID retains its exact index.
- Unknown step-four IDs remain rejected.
- `rustfmt 1.94.1 --edition 2024` passes.
- `git diff --check` passes (PowerShell reports the repository LF/CRLF notice).
- Source SHA-256:
  `cdae938fc275a2277a03d6864ce8c8f51520f1fba8389584c889adc0dbb7d9f5`.

## Acceptance gates

1. Managed Windows native Release compilation and focused Runtime tests pass.
2. Every declared transmission executor ID maps to its existing step index.
3. The ignored evidence emits the Runtime533 marker with at least 75% modeled check reduction.
4. Commit and push remain coordinator-owned; WeCom publication follows accepted validation only.

No direct Cargo validation, compile, commit, push, or WeCom success is claimed by this record.

## Managed validation result (2026-08-30)

The Runtime532/533 batch ticket `22c637b18dd94b03bbd074bad20abe46` stopped before Cargo in
`materialization / owned_overlay`. Job `44040289b0124c3690678d6ab2f33667` reported
`validation_copy_attribution_stale` for this session's earlier Runtime506 dependency
`zircon_runtime/src/text/font/database/system_fonts.rs`. The Runtime533 source hash remained exact;
no compile, test, performance, commit, push, or WeCom success evidence was produced. Recovery
validation is pending after exact Runtime506 attribution repair.

The first recovery ticket `689cf09c14f147b09316464d4838792e` then reached the same
`materialization / owned_overlay` stage and exposed the next historical attribution dependency,
Runtime508 `zircon_runtime/src/ui/surface/surface/default_interactions/radio.rs`. Job
`22a8067d007e45d280835537eb1771a9` stopped before Cargo. The current Runtime508 blob differs from
its original manifest only by rustfmt import ordering and is now exactly re-attributed; aggregate
ownership scans report no remaining Runtime/Editor attribution blocker.
