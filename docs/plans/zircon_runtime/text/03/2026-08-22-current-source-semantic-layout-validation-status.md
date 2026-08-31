---
record_kind: execution_status
status: resolving_failure
recorded_at: 2026-08-22
plan: docs/plans/zircon_runtime/text/03-line-breaking-measure-and-layout.md
related_failure: docs/plans/zircon_runtime/text/03/failure-2026-07-18-layout-prefix-and-grapheme-remeasurement.md
session: text03-current-proof-r1-bee4c707-20260822
base_head: bee4c707b714738346b49bba15c59468b8bd9b39
---

# Text03 Current-Source Semantic Layout Status

## Completed Current-Source Review

- `text/shaping/horizontal/direct.rs` and `text/shaping/vertical/direct.rs` enumerate real hard lines, itemize each complete logical semantic segment, and shape that segment once. They do not partition source ranges at a byte-size threshold.
- `text/shaping/work_budget.rs` makes the 64 KiB value an execution-classification threshold only. Its contract explicitly forbids using it as a source, script, glyph-cluster, or visual-line boundary; the synchronous fallback retains the complete semantic context.
- `text/hard_line.rs` remains the common owner for CR, CRLF, LF, VT, FF, NEL, LS, and PS separators. The layout and shaping paths therefore share hard-line source ranges instead of reconstructing incompatible local lines.
- The neutral `SharedTextLayoutService` projects canonical shaped output to framework DTOs after shaping. A current-source search found no graphics-layer call to `hard_lines`, `logical_segments_for_line`, `TextShapingWorkBudget`, or the direct text shapers. This preserves the Text03 backend-neutral ownership boundary required by `engine-code-structure-convention.md`.
- The Text03 indexed-layout implementation and its existing scale/evidence entry points remain in the correct owners: grapheme projection in `text/layout`, rich advance indexing in `text/layout/rich_advance_index`, and UI layout projection in `ui/text/layout_engine`. No renderer-side alternate layout path was introduced.
- The current hot paths match the research gate rather than its previously suspected quadratic shape: grapheme projection uses two monotonic `partition_point` searches per glyph, rich text coalesces contiguous equal-style parser runs before shaping, and boundary correction limits contextual remeasurement to an eight-grapheme prefix and suffix. There is no per-grapheme whole-run shaping or growing candidate-string loop in these paths.

This review confirms the correction required by the 2026-06 review finding: a backend work budget cannot split Arabic, Indic, grapheme, or ligature context. It does not claim a new optimization result. The existing performance research gate remains authoritative: measure `O(G + N log G + I)` grapheme projection before considering a cluster-prefix enhancement, and do not substitute scalar advances, unbounded prefix memoization, or renderer-owned layout.

## Managed Validation Status

The following evidence remains required and has not run in this session:

- focused and upward `zircon_runtime` behavior regressions for semantic long-line shaping, rich index topology, boundary correction, and soft hyphen;
- ignored 31-sample layout/cache, grapheme-projection, and long-semantic-request p50/p95 evidence using the profiling lane;
- the real WGPU product-framebuffer test and fresh PNG visual inspection under `docs/tests/runtime/text`.

No Cargo command was run outside the managed validation script. A previously accepted Cargo acquire request created no process and no command, and was released through the coordinator before retrying. Therefore it provides no test or performance evidence.

## Validation Infrastructure And Pool Status

The default managed-script timeout first prevented submission. The observed tool contract drift is between `tools/zircon-session.ps1` and the running Session Coordinator:

1. `Test-CoordinatorHealthy` requests `/health` without an `Authorization: Bearer <runtime token>` header.
2. The running coordinator rejects that request as unauthorized; its server contract requires the bearer token.
3. The wrapper consequently treats a healthy coordinator as unavailable and enters its startup gate. With its default 15-second command deadline, a later coordinator command reached `command_preflight_timeout` with `submission: not_submitted`.

An explicitly scoped 60-second deadline was then used only for the managed script process. It registered the validation session and reached `cargo acquire`, proving that no Text03 source error occurs before the managed Cargo lane. Its first acquire returned `cargo_reuse_pool_busy` because compatible job `aa987bc74f1d451c94e2b329c9ddaffa` was compiling the same focused semantic-long-line test for another Session. That other job later became `orphaned` without an exit code and is not evidence for Text03. A second acquire is currently queued behind compatible job `abb01ecff3df4ab288496798b2560a3a`, an active UI acceptance build with a live `rustc` process. No Text03 Cargo process, test result, performance sample, or screenshot has been created in this Session.

The unauthenticated health probe remains a tooling defect for the Session Coordinator owner. Tool-level acceptance is to authenticate the health probe from the runtime descriptor and prove that `validate-matrix.ps1` registers a validation session, acquires one Cargo job, starts it, finishes it, and releases it without duplicate daemon launch or a preflight timeout. Text03 validation must wait for the active compatible pool job to release, then reacquire through the managed script; it must not bypass the pool or reuse another Session's result as Text03 acceptance.

Until that repair is available, Text03 remains `implementation_complete / resolving_failure / managed_validation_pending`. The failure is deliberately not closed, no p50/p95 value is recorded, and no existing text-only or stale PNG is reused as WGPU acceptance. The coordinator-supported `-Ephemeral` mode was considered and intentionally deferred: concurrent compilation would contend with the active roughly 4 GB Rust build and make the performance measurements non-comparable.

## External Artifact-Governance Blocker

After the compatible pool became available, the current Text03 managed validation was rejected before Cargo start with `unmanaged_artifacts_detected`. The coordinator identified `F:\cargo-targets\zircon-engine`; read-only inspection showed that the directory was newly created at 2026-08-22 16:58 and contains `rhi90-validation`. The matching `rhi90-operation-capability-contract-m0-r2-bee4c707-20260822` primary Session is registered for the RHI90 capability plan, but its F-drive output is not registered as a coordinator-managed job.

This Session neither created nor owns that artifact, so it must not delete, move, register, or otherwise mutate it. The RHI90 owner/coordinator must reconcile or safely clean its own output, after which Text03 can rerun the same managed command. This rejection occurred before a Text03 Cargo process, test, performance sample, or product PNG existed; it is not a source failure and creates no new Text03 validation evidence.

The external directory was subsequently removed by its owner, so it no longer blocks the coordinator. Two early Text03 ephemeral attempts (`86b10c4bc65042bebeae4c9155bd46f6` and `f8257741a2ad4fe99660b7bd2df48d90`) became `orphaned` after their caller-side 90-second and 30-minute observation windows ended before the managed script could send its final `finish`; both target directories were automatically deleted and neither provides a result. A later isolated `-SkipBuild` run (`4fc447411baa4915bac920f22425e56c`) completed its managed lifecycle and returned Cargo exit 101 after 18m47s. It is the first real dynamic failure for this Session, but the ephemeral cleanup removed its full compiler log before diagnosis could be preserved. The next diagnostic rerun must use the shared managed cache with verbose output disabled so the exact `error[...]` and source location survives the client response; no Text03 source change is authorized until that lower-layer cause is known.

That shared diagnostic rerun completed as coordinator job `6dabb1edfc62419b84bfb9131394ba6b` in 10m41s, with the wrapper exiting 1 because Cargo returned 101. It established the lower-layer gate: `zircon_runtime/src/ui/template/asset/compiler/binding_program.rs` imported a missing public `UiCompiledAssetId`, and `ui/tests/asset_surface_index/binding_ownership_performance.rs` referenced an out-of-scope `TARGET_BINDING_COUNT`; the compilation reported 20 errors before any Text03 test executed. The canonical type and test belong to Runtime74 binding reload, so the actionable handoff is `docs/plans/optimize/zircon_runtime/74/failure-2026-08-22-text03-compiled-binding-contract-compile.md`.

2026-08-22 repair status: Runtime74 has now added the canonical `UiCompiledAssetId` public re-export, confirmed by current-source inspection. The remaining test-owner scope repair was submitted to the coordinator as deferred patch `#127`; it moves the expected target-binding count to the shared module scope without weakening the 4,096/16/128 workload or its assertion. This is only a durable handoff receipt, not a compile result. Text03 must not run layout p50/p95 or emit a WGPU PNG until Runtime74's patch is applied and the named text test actually runs through the managed Cargo lane.

## 2026-08-22 Cache-Key Repair And Architecture Re-review

The completed cache-key repair removes `UiWidthBucket` from
`ui/text/measure_cache.rs`.  The retired key component called
`measure_line_width("n", ...)` before every `resolve_or_shape(...)` cache lookup;
that direct auxiliary shaping bypassed `SharedTextLayoutSession`, including on
otherwise valid frame and persistent cache hits.  The key now retains only real
request inputs: exact frame and clip geometry, content, viewport, style, and font
generation.  `TextLayoutCache` continues to enforce its exact
`TextLayoutWidthValidity` predicate.  The source regression rejects both the
retired type and the auxiliary sample, so the correction cannot silently return as
a cache-key heuristic.

This is a measured-work removal with a precise complexity claim, not a benchmark
claim: it avoids one uncached auxiliary shaping call per wrapping
`resolve_or_shape(...)` entry.  Focused cache behavior, the 31-sample p50/p95
reporters, and product WGPU capture remain unrun behind the Runtime74 compile
gate, so no CPU time, energy, or cross-engine comparison is recorded.

The required structural re-review compared the live Slate text-layout source with
Text09's architecture contract.  Slate's `FLineModel` / `FLineView` separation,
estimated offsets, and visible-view materialization are valid reference evidence
for a future document/viewport optimization.  They do not authorize a speculative
Zircon cache rewrite: the present `UiResolvedTextLayout` exposes absolute line and
box frames, and the current regression deliberately proves that a frame-origin
change re-resolves that geometry.  Text09 explicitly rejects reuse across changed
frame, viewport, writing mode, or wrap width and requires an M0 trace to select
an M1 leaf-stage change.  Therefore no frame-origin re-key, second layout cache,
facade, renderer-side layout, or model/view split is included in Text03 before
profile evidence identifies `ui_text.layout_resolve` as the bottleneck.

Completed in this status update: current-module and Slate re-review, the
cache-key auxiliary-work removal, cache-key source regression, scoped formatting,
and whitespace checks.  Remaining: Runtime74 compile recovery; managed focused
Text03/cache regressions; M0 p50/p95 and WGPU timestamp evidence; and one fresh
real-framebuffer export plus visual inspection under `docs/tests/runtime/text`.
Status remains `implementation_complete / resolving_failure /
managed_validation_pending`; no Cargo, performance, WGPU, PNG, milestone,
commit, or WeCom result is claimed.
