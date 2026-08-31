---
record_kind: architecture_and_performance_research_plan
status: current_source_review_complete_measurement_instrumentation_implemented_static_correctness_prerequisite_implemented_empty_projection_fallback_bypass_removed_visual_slice_congruence_static_baseline_profile_pending_no_optimization
created_at: 2026-08-31
owner_plan: docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md
related_code:
  - zircon_runtime_interface/src/ui/surface/render/text_shape.rs
  - zircon_runtime_interface/src/ui/surface/render/text_shape/projection_profile.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/rich_text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/resolved_layout.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/resolved_layout/rich_artifact_routes.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/tests/rich_inline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/tests/rich_inline_profile.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/tests/rich_artifact_routes.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/tests/rich_projection_admission.rs
  - zircon_runtime_interface/src/ui/surface/render/text_shape/resolved_layout_tests.rs
  - zircon_runtime/tests/runtime_text_multilingual_product_framebuffer.rs
  - zircon_runtime/tests/runtime_text_multilingual_product_framebuffer/product_renderer.rs
  - zircon_runtime/tests/runtime_text_multilingual_product_framebuffer/proof_path.rs
  - tools/tests/test_runtime_text_paint_run_projection_profile_contract.py
  - tools/tests/test_runtime_text_product_framebuffer_proof_contract.py
  - tools/tests/test_runtime_text_rich_inline_geometry_profile_contract.py
  - tools/tests/test_runtime_text_paint_run_fail_closed_contract.py
design_references:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Text/TextLayout.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Text/SlateTextLayout.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/TextLayout.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/ILayoutBlock.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/ISlateRun.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/SlateImageRun.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Text/SlateImageRun.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/SlateWidgetRun.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Text/SlateWidgetRun.cpp
  - dev/Fyrox/fyrox-ui/src/formatted_text.rs
current_source_head: 14c89f9776bed828cc85e05e4b9914b3f8d1e784
review_baseline_sha256:
  rich_text_rs: bfdb8bef56c3e8e23193e907e5d7529a1f1f90538abbae8a40ac5dbf7a645443
  text_shape_rs: 2145e656627327813a1539bc0c86c8b23641f66836ce70404a444ba3d0971470
  resolved_layout_rs: ed0f3fce2facd85b6f8d0a8e3b34e81bccf1ccfbdd995115d79b1346d5cf4526
measurement_instrumentation_sha256:
  rich_text_rs: 5f2db799bd7976da57dc220772f59712256da53923fe0c5b82b537172793a7fa
  text_shape_rs: 3a7276a517fd45875d7da7854b7ea27c2f17f1421947ab665c93a047d10d12c6
  projection_profile_rs: 682fe7624142eff5e896b5bb58240a396128c2be55f027d7323e0d670d07d082
  rich_inline_tests_rs: 3490adb801ec5a42609a8f0dc57bf8304917c6623839aec1f0fb09fdea7addf2
  rich_inline_profile_rs: cf38474b3b25c729bfc3f9f483dc574075535b2fa7345475e91832fd5a871a86
---

# Rich paint-block geometry owner and profile plan

## Scope and status

This is a current-source architecture review and measurement plan, not an optimization or acceptance
record. No production algorithm is changed here, and no Cargo, WGPU, PNG, p50/p95/p99, allocation,
RSS, package-power, or Unreal matched-load result is claimed. The real framebuffer proof remains under
`docs/tests/runtime/text`; screenshots must never be written to `target`.

The existing `O(lines + runs)` statement is valid for
`rich_text_glyph_artifact_runs`: it monotonically publishes typed glyph-artifact routes and performs
O(1) exact directory access. It is not an end-to-end complexity statement for rich paint-block
geometry. Two independent geometry paths still repeat prefix work and must be measured before a
structural cutover.

## Current-source review

### Interface paint-run projection

`text_paint_runs_from_resolved_layout` walks `layout.lines -> line.runs`, but calls
`resolved_text_run_frame` independently for every run. Each call:

1. finds grapheme-aligned start/end offsets;
2. counts every grapheme in the line;
3. validates every line advance;
4. recounts both run prefixes; and
5. sums the leading and run advance slices.

For a line with `G` graphemes and `R` paint runs this is `O(R * G)` work in the worst case, despite the
outer loop being monotonic. The function then publishes that result as `UiTextPaintRun.frame`.

### Inline renderer projection

`plan_rich_text_runs` correctly reuses the typed glyph route vector, and `CompiledRichText::run_for_range`
uses `partition_point`. An inline run nevertheless takes a separate path:

1. `plan_inline_run` scans `layout.lines` from the beginning to recover the containing line;
2. `inline_layout_frame` scans that line's runs to recover the same source range;
3. it counts the complete visual prefix in graphemes; and
4. it sums the same leading advances already used to publish `UiTextPaintRun.frame`.

With many inline objects on one line, the repeated run/prefix work is `O(I * G)`; with many lines, the
line lookup adds `O(I * L)`. An alternating text/inline fixture can therefore approach quadratic work.
The fallback to `run.frame` does not remove this cost because it is used only after the searches fail.

### Ownership defect

The interface layer and renderer both derive main-axis placement from the same resolved line text,
visual range, and advance array. This is a duplicate geometry owner, not merely a slow helper. It also
creates a correctness risk: future BiDi, vertical, justification, generated-fragment, or cluster
geometry changes could update one derivation without updating the other.

## Reference-engine decision

Unreal Slate is the primary standard. `FTextLayout::CreateLineViewBlocks` creates `ILayoutBlock`
objects while it already owns run, directional block, range, measurement, baseline, and line placement.
`FSlateImageRun::OnPaint` and `FSlateWidgetRun::OnPaint` consume `Block->GetBlockSize()` and
`Block->GetLocation()` directly. Image/widget hit testing and child arrangement consume the same block;
paint does not rediscover a line or reconstruct a prefix from source text. Desired-size changes dirty
the owning widget run's layout instead of installing a second paint-time geometry authority.

Fyrox is a secondary Rust ownership check. `FormattedText::arrange` clears and rebuilds positioned
glyphs in one monotonic pass over retained lines, and render consumers read those retained bounds. Its
Unicode and rich-inline feature coverage is not the target, but it reinforces the single positioned
geometry owner.

The target Zircon boundary is therefore:

```text
compiled rich run + canonical shaped/layout artifact
  -> resolved line/run block geometry (single owner)
  -> UiTextPaintRun / typed inline block receipt
  -> image paint or ordinary child arrangement (consume only)
```

No renderer-local grapheme scan, advance sum, line search, paragraph geometry map, compatibility
facade, or second inline registry is permitted in the terminal design.

## Measurement contract

Measurement precedes the cutover. Fixed-name, low-cardinality counters must expose existing work
without storing source text, ranges, node IDs, resource IDs, or per-run events:

| Counter | Meaning |
| --- | --- |
| `rich_inline_run_count` | Inline paint runs admitted by compiled metadata. |
| `rich_inline_line_probe_count` | Resolved lines examined by renderer inline lookup. |
| `rich_inline_line_run_probe_count` | Runs examined by renderer inline range lookup. |
| `rich_inline_prefix_grapheme_count` | Graphemes visited while recovering inline main-axis offsets. |
| `rich_inline_prefix_advance_count` | Advance entries summed for those offsets. |
| `rich_inline_paint_frame_match_count` | Recomputed inline main-axis origins matching the already published paint frame within the fixed geometry tolerance. |
| `rich_inline_paint_frame_mismatch_count` | Any disagreement; this is a correctness failure and blocks optimization. |

Interface paint-run projection is measured with an isolated release benchmark around the exact
production helper plus the existing `ui_text.extract`/render planning spans. The benchmark records
line graphemes, run count, implied grapheme visits, elapsed microseconds, and working-set delta; it
must not add a public profiling field to `UiTextPaintRun` or a second retained report.

Required 31-sample lanes, all on an approved E-drive target/temp root:

| Lane | Writing/direction | Lines | Runs or inline objects | Purpose |
| --- | --- | ---:| ---:| --- |
| dense-styled-1 | Horizontal LTR | 1 | 1/100/1k/10k runs | Isolate interface run-frame prefix scaling. |
| dense-inline-1 | Horizontal LTR | 1 | 1/100/1k/10k inline objects | Expose repeated run/prefix renderer work. |
| dense-inline-rtl | Horizontal RTL | 1 | 1/100/1k/10k inline objects | Prove visual-range/main-axis equivalence. |
| dense-inline-vertical | VerticalRl | 1 | 1/100/1k/10k inline objects | Prove vertical main-axis equivalence. |
| wrapped-inline | Horizontal LTR | 1/100/1k lines | 1 inline per line | Expose repeated line lookup. |
| mixed-product | Horizontal/RTL/VerticalRl | product fixture | actual | Attribute end-to-end relevance before changing architecture. |

Each lane reports warm-up count, 31 raw samples, p50/p95/p99, allocation or retained-byte delta, RSS,
and counter totals. The matched before/after run must use the same source, style, fonts, viewport,
render mode, build profile, machine power mode, and artifact-hit state. Package power and a same-load
Unreal experience comparison remain separate acceptance evidence; an isolated microbenchmark cannot
substitute for them.

## Decision gates

1. Do not change geometry ownership before the baseline is captured.
2. Any non-zero frame mismatch blocks reuse of `UiTextPaintRun.frame`; resolve geometry semantics first.
3. If probe/grapheme/advance totals remain constant while object count grows, reject this optimization
   hypothesis and profile the next stage.
4. If totals follow the static `I * L` or `I * G` model and planning time is material, hard-cut to one
   layout-owned block geometry publication. Do not install a cache around the duplicate algorithm.
5. The post-cutover static target is `O(lines + runs + inline objects)` with no renderer grapheme or
   advance walk. The dynamic gate requires lower p50/p95/p99 in the dense lanes without regression in
   the mixed product lane, zero mismatches, bounded allocation/RSS, and no power regression.
6. Managed Cargo, real WGPU output, and PNG pixel inspection are still required before milestone
   acceptance, commit, or WeCom completion notice.

## Measurement instrumentation status

The renderer now publishes the seven fixed counters above from a request-local aggregate. Probe counts
are incremented inside the existing searches, so the measurement observes the current algorithm rather
than estimating it with a second scan. The aggregate's fields and counter publication compile only with
the `profiling` feature; the ordinary build retains a zero-sized aggregate and empty inline methods.

A profiling-feature regression builds a real resolved rich layout with three typed inline image runs,
derives expected line/run/prefix work from that layout, executes normal screen-space UI planning, and
requires all three main-axis comparisons to match the previously published paint frames. This is static
measurement infrastructure only. The test has not run under Cargo because the managed lane was not
admitted; no baseline timing or optimization claim is made.

A Windows release-only ignored renderer benchmark now prebuilds the production layouts and extracts for
dense LTR, RTL, and VerticalRl lanes at 1/100/1k/10k inline objects, plus 1/100/1k hard-line lanes. It
captures the seven work/frame counters in a separate untimed planning pass, then emits three warm-ups,
31 raw planning-time/RSS samples, and nearest-rank p50/p95/p99 per lane. Parse, layout, and profile
snapshot construction are outside the timed body. This benchmark has not run under Cargo and therefore
publishes no baseline result yet.

The Interface owner now also contains a Windows release-only ignored benchmark around the exact
production `text_paint_runs_from_resolved_layout` helper. It prebuilds one-line LTR layouts with
1/100/1k/10k style runs, performs three warm-ups and emits 31 raw nanosecond and working-set-delta
samples plus nearest-rank p50/p95/p99 and the implied full-line grapheme visits. The timed body does
not copy `resolved_text_run_frame`, mutate production behavior, or write an artifact. The test has not
run under Cargo, so these are harness capabilities rather than baseline results. At the measurement-only
checkpoint, the two dedicated measurement source contracts, product-proof contract, and three existing
Runtime Text suites passed 84/84 combined.

Before any geometry-owner cutover, the damaged-projection boundary was corrected independently. A
nonempty resolved run whose frame cannot be reconstructed now rejects the complete paint projection;
legal empty runs remain ignored. Visual ranges must be ordered, in bounds, and UTF-8 scalar-aligned before
grapheme expansion; legal scalar-aligned boundaries inside a grapheme remain supported. The rich route
builder returns typed `PaintLayoutMismatch` for cardinality/order/text/range drift, including missing or
incomplete artifact early returns, and the renderer records one command-level incomplete rejection without
emitting a generic whole-line fallback. Typed rich planning also suppresses text-dependent pre/post
decorations on structural rejection while retaining outer command chrome. Focused Rust regressions are
source-present and the new static contract passes 3/3. Combined Runtime Text static coverage is therefore
91/91. Cargo has
not run for these regressions, so this is a static correctness prerequisite rather than acceptance.

The renderer fallback audit also traced the production `text_layout: None` constructors: surface extraction
resolves missing layouts through `resolve_missing_render_command_text_layouts` before the command reaches this
planner. The raw route is consequently retained only as a compatibility boundary for genuinely unresolved
commands. Both the source command frame and its viewport intersection are finite/positive-admitted, and malformed
geometry fails closed as an incomplete command instead of entering native or SDF shaping.

Rich paint-run geometry is also preflighted once per command before materialization. This rejects non-finite
coordinates, non-finite or non-positive font/line metrics, and negative extents atomically, so a malformed run
cannot leave a partial rich background, inline resource, or text batch behind. Zero-size frames remain admissible
for control metadata; a future typed role must distinguish that case without geometry heuristics.

The subsequent glyph-artifact route is now command-atomic as well. One `O(R)` admission pass retains the route,
compiled-rich run, inline classification, and exact whole-line source-fallback provenance for each paint run.
Any non-inline Missing/Stale/Incomplete route that cannot reproduce an entire source-isomorphic visual line rejects
the command before materialization; an earlier recoverable run can no longer leave partial text or background
behind. Styled sub-runs are not labelled as complete layout lines because that marker authorizes SDF atlas span
overlays. Route/profile counters still publish on the rejected path, but no fallback-shape request is counted unless
it actually reaches materialization. This correctness boundary is independent of the block-geometry optimization
hypothesis and adds no timing, allocation, or power claim. At the artifact-admission checkpoint, focused
fail-closed contracts were 4/4 and combined Runtime Text static contracts were 92/92; Rust execution remained
managed-validation pending.

The follow-up presentation audit found a second-owner correctness defect before any geometry optimization:
layout filters invalid rich size/family overrides, while renderer paint reconstruction previously consumed the raw
compiled values. The compatibility boundary now applies the same finite-positive size and nonempty-family admission;
an empty failed projection with a present rich layout is also forced through cardinality admission instead of generic
plain batch fallback. The failing-first contracts are GREEN, bringing the focused suite to 6/6 and combined static
coverage to 94/94. The same projection boundary now also requires nonempty resolved runs to be contiguous and to
equal their exact UTF-8-safe line slices before any paint DTO is published. This is a monotonic
`O(lines + runs)` correctness admission; it does not alter the pending geometry optimization hypothesis or provide
timing, allocation, RSS, or power evidence. The
terminal design remains a Slate-aligned retained prepared block/run owned by layout, with renderer parser-artifact
style lookup removed in the same hard cut. That cut and any DTO residency optimization remain gated by the baseline
defined in `../07/2026-08-30-rich-prepared-run-current-source-review.md`; no timing or allocation result is inferred.

## Candidate cutover after evidence

The preferred implementation is to publish the final inline block frame from the same monotonic
resolved-layout projection that publishes paint runs. The renderer consumes that frame directly;
Surface consumes the same frame for widget child arrangement. If changing the public paint DTO would
create an unnecessary cross-crate contract, an internal route receipt may pair the already flattened
paint run with its resolved line/block frame. Either form must keep one geometry owner and fail closed
on cardinality/order drift.

The implementation must retain horizontal LTR/RTL, VerticalRl, baseline/center/top/bottom alignment,
ellipsis-retained inline objects, inline-only lines, viewport clipping, authored icon/image sizes, and
widget child geometry. It must add no glyph, font, source, or resource ownership to the renderer.

## WGPU product-proof status

The existing product test is the accepted evidence route, not a strategy screenshot. It creates a real
headless WGPU viewport, waits for two consecutive settled raster frames, captures framebuffer RGBA,
compares text regions against a background-only frame, and checks the imported checker image's channel
quadrants for ordinary, wrapped, RTL, ellipsized, VerticalRl, BBCode icon, and vertical-paragraph cases.
Its atomic PNG writer targets
`docs/tests/runtime/text/runtime_text_mvp_foundation_product_framebuffer_20260831.png`; path guards reject
both the workspace `target` and the configured external Cargo target. Fixture and failed-encode work also
remain under the ignored `docs/tests/runtime/text/.runtime_text_product_framebuffer_work` root, never the
Windows system temporary directory.

The current proof PNG is absent. The product test has not run after this implementation slice, so no
WGPU or pixel acceptance is claimed and no historical image may be substituted. A focused static proof
contract passes 3/3 for current evidence identity, docs-only output, external target/C-drive rejection,
real WGPU readback, framebuffer pixel deltas, and typed checker image/icon assertions.

## Current blockers and next action

The first managed Cargo attempt did not compile Runtime source because a third-party `zstd-sys`
custom-build `cl.exe` child exited without a diagnostic. Its exact syntax check passed afterward. The
single retry was not admitted because the managed Cargo CPU lane was reserved by another session; per
repository policy this task does not poll or retry the coordinator. Static inspection and measurement
instrumentation, including the isolated release benchmark, is implemented independently. Managed
focused validation request `9df75274da66456d974c3e89b2d19f58` was subsequently accepted, but its
`cargo.acquire` command had no terminal result after bounded reconciliation; it produced no Cargo/test
evidence and is not polled or duplicated. Managed
execution, baseline timings, the structural cutover, WGPU, PNG, power, commit, and WeCom remain pending;
the current static status is
`raw_fallback_frame_admission_completed / rich_run_geometry_preflight_completed /
rich_run_positive_metric_admission_completed /
rich_artifact_admission_atomic_completed / rich_renderer_style_admission_parity_static_completed /
rich_empty_projection_plain_fallback_bypass_removed /
resolved_run_visual_slice_congruence_static_completed /
managed_validation_pending`.
