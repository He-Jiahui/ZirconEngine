---
title: Runtime GPU Scene and Editor Plugin Lookup Fast Paths 521
category: zircon_runtime
report_id: Runtime521-gpu-scene-id-allocator-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime GPU Scene and Editor Plugin Lookup Fast Paths 521

`GpuSceneIdAllocator::commit_pending_frees` previously called `sort_unstable_by_key` for every
non-empty pending-free batch. Frame-boundary frees commonly arrive in ascending slot order, so the
sort paid `O(P log P)` work even when the merge input was already ordered. The allocator now tracks
whether a newly appended span decreases the previous start key and sorts only when that invariant is
broken. The coalescing merge, deferred reuse boundary, live count, and high-water behavior are
unchanged; the flag resets after every successful merge.

`EditorPluginPanelSource::row` previously searched the sorted manager entries and then separately
searched the generation-paired sorted catalog projection. The two vectors are already published in
the same package-id order and `rows()` enforces that invariant. The single-row path now performs one
manager binary search and uses the resulting index to read the projection in `O(1)`, preserving the
package-id consistency assertion and borrowed generation lifetime.

The focused regression covers monotonic and out-of-order release sequences. The ignored Release
model `RUNTIME521_GPU_SCENE_PENDING_FREE_SORT_BENCH_V1` exercises 32,768 frames with 64 pending
spans: legacy unconditional sorting performs 32,768 sort calls while the monotonic optimized path
performs 0, avoiding 100% of sort invocations in that workload. This is a sort-work model, not an
end-to-end frame-time claim.

The Editor Release model `EDITOR521_PLUGIN_PANEL_SINGLE_SEARCH_BENCH_V1` fixes 32,768 lookups over
1,024 rows. Logical binary searches fall from 65,536 to 32,768, a 50% reduction; this is a lookup
operation model rather than an elapsed-time claim.

No direct Cargo validation was run. The coordinator owns combined Runtime/Editor Release
compilation, focused tests, ignored performance evidence, immutable manifest checks, commit/push,
and WeCom publication after all declared gates pass.

## Static checks

- TDD RED: the new regression referenced the absent `pending_free_spans_needs_sort` state before
  implementation.
- TDD GREEN: the state-aware implementation and existing span/coalescing tests are present.
- Editor TDD RED: the source contract observed `snapshot.entry()` plus the second projection search.
- Editor TDD GREEN: the row source contains one binary search and no second snapshot entry lookup.
- `rustfmt --edition 2021 --check` passed for both owned sources.
- `git diff --check` passed for both owned sources (PowerShell reports the repository LF/CRLF notice).

## Current source

- `zircon_runtime/src/graphics/scene/gpu_scene/id_allocator.rs`
  - SHA-256: `83E4B9E3B57024CCF99C8AE9ADCD2CC6BE8FBB1452419CFBB29FBC24BA01F772`
- `zircon_editor/src/core/plugin/panel_source.rs`
  - SHA-256: `4451BE7257E2B0645D045E8AAADEC198021A15FB61122C4D20CEFDC3D36A52F0`

## Acceptance gates

1. Managed Windows native Release compile and focused Runtime/Editor tests pass.
2. The ignored benchmarks print both 521 markers and confirm zero optimized sort calls for
   monotonic frees while the legacy model remains positive.
3. Unordered frees still coalesce to the same canonical spans and no deferred-reuse behavior
   changes.
4. Plugin panel row lookup preserves package order, row content, and old-generation snapshot reads.
5. Commit and push are coordinator-owned; WeCom receives both validated operation-reduction data.
