---
title: Runtime Postprocess Resource Borrow 532
category: zircon_runtime
report_id: Runtime532-postprocess-resource-borrow-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_snapshot_stale
---

# Runtime Postprocess Resource Borrow 532

Product postprocess execution previously cloned both resource-name vectors and every contained
`String` before validating graph bindings. Resource resolution is read-only, so the executor now
holds a shared GPU context and traverses the compiled node's input and output names by reference.
The resolver-backed and compatibility resource-kind routing remain unchanged.

The ignored Release evidence `RUNTIME532_POSTPROCESS_RESOURCE_BORROW_BENCH_V1` models 32,768 frames,
eight effects per frame, three inputs, and two outputs. The legacy projection performs 1,835,008
temporary owned allocations: two vector buffers plus five cloned resource strings per effect. The
borrowed projection performs zero such allocations, a 100% modeled reduction. This is an ownership
and allocation-count model, not an end-to-end render-time or GPU-time claim.

## Static evidence

- TDD RED: `required_inputs` and `produced_outputs` were cloned before both resource loops.
- TDD GREEN: both loops borrow the compiled node slices and route through read-only GPU lookup.
- Production context construction binds the resource resolver before the GPU, preserving the exact
  physical resolver carried by `RenderPassGpuExecutionContext`.
- A source contract rejects both legacy clone expressions.
- `rustfmt 1.94.1 --edition 2024` passes.
- `git diff --check` passes (PowerShell reports the repository LF/CRLF notice).
- Source SHA-256:
  `d98faff6965f2639a62297d7835aa23eaa1ebd07ad7f2b661747ccb447d32d38`.

## Acceptance gates

1. Managed Windows native Release compilation and focused Runtime tests pass.
2. Resolver-backed, transient, and external resource routing remain green.
3. The ignored evidence emits the Runtime532 marker with zero modeled temporary owned allocations.
4. Commit and push remain coordinator-owned; WeCom publication follows accepted validation only.

No direct Cargo validation, compile, commit, push, or WeCom success is claimed by this record.

## Managed validation result (2026-08-30)

The Runtime532/533 batch ticket `22c637b18dd94b03bbd074bad20abe46` stopped before Cargo in
`materialization / owned_overlay`. Job `44040289b0124c3690678d6ab2f33667` reported
`validation_copy_attribution_stale` for this session's earlier Runtime506 dependency
`zircon_runtime/src/text/font/database/system_fonts.rs`. The Runtime532 source hash remained exact;
no compile, test, performance, commit, push, or WeCom success evidence was produced. Recovery
validation is pending after exact Runtime506 attribution repair.

The first recovery ticket `689cf09c14f147b09316464d4838792e` then reached the same
`materialization / owned_overlay` stage and exposed the next historical attribution dependency,
Runtime508 `zircon_runtime/src/ui/surface/surface/default_interactions/radio.rs`. Job
`22a8067d007e45d280835537eb1771a9` stopped before Cargo. The current Runtime508 blob differs from
its original manifest only by rustfmt import ordering and is now exactly re-attributed; aggregate
ownership scans report no remaining Runtime/Editor attribution blocker.
