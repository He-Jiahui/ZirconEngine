---
record_kind: correctness_implementation_status
status: rich_paint_run_projection_all_or_empty_static_implemented_plain_and_rich_command_rejection_fail_closed_empty_projection_plain_fallback_bypass_removed_visual_slice_congruence_static_style_admission_parity_static_managed_validation_pending
created_at: 2026-08-31
owner_plan: docs/plans/zircon_runtime/text/03-line-breaking-measure-and-layout.md
related_performance_plan: docs/plans/zircon_runtime/text/09/2026-08-31-rich-paint-block-geometry-owner-and-profile-plan.md
related_code:
  - zircon_runtime_interface/src/ui/surface/render/text_shape.rs
  - zircon_runtime_interface/src/ui/surface/render/text_shape/resolved_layout_tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/resolved_layout.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/resolved_layout/rich_artifact_routes.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/text_batches.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/rich_text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/tests/glyph_artifacts.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/tests/rich_artifact_routes.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/tests/rich_projection_admission.rs
  - tools/tests/test_runtime_text_render_batch_owner_contract.py
  - tools/tests/test_runtime_text_paint_run_fail_closed_contract.py
---

# Rich paint-run cardinality fail-closed

## Finding

`text_paint_runs_from_resolved_layout` previously skipped a nonempty run when
`resolved_text_run_frame` rejected its geometry. This published a partial paint-run vector even though
the retained layout still contained the complete ordered run set. `rich_text_glyph_artifact_runs` then
had no typed command-level representation for that disagreement, and the rich planner could leave the
command eligible for generic whole-line rendering. The fallback cannot recover per-run styles or typed
inline image/icon/widget semantics.

## Architecture decision

The retained resolved layout is the geometry authority. Projection of its nonempty runs is atomic:
publish every ordered run or publish none. Empty runs remain legal non-painting metadata and do not
participate in cardinality. The renderer distinguishes two classes of failure:

1. Missing, stale, or incomplete glyph artifacts remain typed route outcomes for a structurally coherent
   paint projection.
2. Paint/layout cardinality, order, text, source range, or visual range disagreement is the command-level
   `RichTextGlyphArtifactRouteBatch::PaintLayoutMismatch` outcome.

On `PaintLayoutMismatch`, the planner records one
`ResolvedGlyphArtifactRouteReceipt::Rejected(Incomplete)`, returns `TextPlanOutcome::Rejected`, and
emits no Native, SDF, image, or generic whole-line batch. This preserves fail-closed behavior instead of
presenting damaged rich content with silently changed semantics.

The batch owner exposes the command-wide `TextPlanOutcome::{NotHandled, Planned, Rejected}` rather than a
rich-only result or a boolean that conflates handled success with structural damage. Plain and rich routes
share this outcome without a compatibility alias. The outer command planner rolls back only text-dependent
pre-decoration vertices and skips post decorations on `Rejected`; command background/border and an
independent outer image remain intact.

Plain resolved layouts now propagate `ResolvedGlyphArtifactRouteReceipt::Rejected` through the same command
outcome. A non-source-isomorphic visual/BiDi line with a missing, stale, or incomplete artifact therefore
cannot leave selection, composition, caret, outline, or table decoration geometry without its text.
Source-isomorphic fallback, artifact-backed text, and intentional visual-only lines remain planned.

Resolved-layout batch admission now rejects non-finite font/line metrics, non-finite frame components,
non-renderable nonempty line extents, and non-finite or negative glyph advances before either plain or rich
materialization. The source-isomorphic plain fallback repeats the same predicate and cannot turn an
`Incomplete` geometry rejection into a fallback batch. This is a command-level rejection, so text-owned
decorations are rolled back with the invalid text rather than reaching Native/SDF consumers for sanitization.

A present layout with no lines is the layout owner's safe failure publication, not an invitation to shape
the command source again. `Some(empty layout)` now records one incomplete command rejection and emits no
batch; raw renderer fallback remains available only when the command has no layout at all. This closes the
renderer-side reshape bypass used by ordinary layout failures and secure TextField failure layouts.

The adjacent fallback audit found that production surface constructors commonly publish `text_layout: None`
while assembling commands, but the shared extract path invokes `resolve_missing_render_command_text_layouts`
before renderer planning and fills valid text frames with an owner layout. Therefore the remaining raw route is
an explicit compatibility path for commands that still arrive without a layout, not evidence that normal surface
text bypasses the layout owner. Its command and fallback frames now require finite coordinates and positive,
finite extents; malformed geometry records the same command-level `Rejected(Incomplete)` outcome.

Rich paint runs now receive a linear command preflight before any rich background, inline image, or text batch
is materialized. Non-finite coordinates, non-finite or non-positive font/line metrics, and negative extents fail
closed as the same incomplete command rejection. Zero-size frames remain legal for non-painting/control metadata;
their typed role remains a separate contract instead of being inferred from geometry.

Glyph-artifact admission now has the same command boundary. The renderer first builds one ordered admission
vector that pairs every paint run with its canonical artifact route, compiled-rich style identity, inline-block
classification, and exact whole-line source fallback provenance. If any non-inline run has a missing, stale, or
incomplete artifact and is not a source-isomorphic whole visual line, the complete rich command is rejected before
any run background, inline resource, or text batch is written. Recoverable runs are not partially committed ahead
of the failure. Inline blocks remain independent of glyph artifacts, while a styled sub-run is deliberately not
promoted to the whole-line provenance required by SDF atlas span overlays. The preflight is `O(R)` after route
construction and does not add renderer-side shaping; timing/allocation impact remains profile-pending.

Missing or incomplete artifact early returns perform the same linear layout/paint congruence check before
publishing per-run rejection routes. The normal artifact path retains its existing single ordered traversal.
Route construction is isolated in the 148-line `resolved_layout/rich_artifact_routes.rs` owner, while the
358-line `render/text_batches.rs` owner holds batch DTOs, route context, command outcome, and batch
materialization; the orchestration/report root is 711 lines.

Visual ranges are admitted before grapheme expansion: they must be ordered, in bounds, and aligned to
UTF-8 scalar boundaries. This removes the former `min`/floor/ceil recovery for malformed byte offsets.
Scalar-aligned style boundaries may still split a grapheme, such as a base scalar and combining mark in
different style runs; both retain the complete shared grapheme frame.

This boundary follows Unreal's retained layout model: line views and positioned blocks form a coherent
paint input, rather than allowing paint to rediscover or partially reconstruct layout state. It does not
copy Unreal APIs and does not perform the planned block-geometry ownership cutover.

## Tests and current evidence

- Interface regression: one valid line followed by one line with invalid advances rejects the complete
  projection; valid nonempty plus legal empty runs still publishes the valid run.
- Interface range regressions: reversed, out-of-bounds, and scalar-split visual ranges reject the complete
  projection, while a scalar-aligned combining-mark style boundary remains valid.
- Renderer regressions: a real Markdown layout with damaged advances produces no Native, SDF, or image
  batches, no table pre/post decorations, and records one command-level incomplete rejection with both
  present and removed glyph artifacts. Command background chrome remains rendered.
- Plain renderer regression source: a non-source-isomorphic BiDi layout without an artifact produces no
  Native/SDF text and suppresses its selection/caret decoration draws. This Rust regression is source-present
  but has not run under managed Cargo.
- Plain geometry regression source: a source-isomorphic line with a `NaN` advance is rejected before fallback,
  emits no Native/SDF batch or selection/caret draw, and records one incomplete command rejection. This Rust
  regression is source-present but has not run under managed Cargo.
- Safe-layout regression source: nonempty command text paired with a present empty failure layout emits no
  Auto/Native/SDF batch and records one incomplete command rejection instead of renderer-side reshaping.
- Raw-fallback geometry regression source: a command with a non-finite source frame is rejected before the
  compatibility route can reshape or emit a batch; the route report records one incomplete command rejection.
- Rich-run geometry regression source: a non-finite paint-run frame or non-positive font/line metric is rejected
  before partial rich background, inline, or text materialization; command chrome remains the only emitted
  geometry.
- Rich artifact-admission regression source: a two-line command whose first line has a legal source fallback and
  second line has a non-isomorphic missing-artifact route emits no text/image/box/decorations and records one
  command rejection. A separate three-style-run fixture now rejects missing canonical artifacts instead of
  reshaping each run and falsely marking styled sub-runs as complete layout lines.
- Rich presentation parity regression source: invalid rich size/family overrides use the same admission as layout
  and preserve the already-laid-out base size, line height, and family instead of reopening divergent paint metrics.
- Empty-projection regression source: a valid-geometry rich layout with one invalid visual run produces an empty
  transient projection, then rejects through `PaintLayoutMismatch`; it cannot fall through to generic plain layout
  batches. Only the no-layout compatibility path retains `NotHandled`.
- Resolved-run congruence regression source: every nonempty run must begin at the prior run's visual end, equal its
  exact UTF-8-safe `line.text` slice, and collectively reach the line visual end. Legal empty metadata runs remain
  ignored, and scalar-aligned style boundaries inside one grapheme remain accepted. Admission is one monotonic
  `O(lines + runs)` pass and does not add a renderer cache or a second geometry owner.
- Static fail-closed contract: 6/6 passed.
- Combined Runtime Text static contracts: 94/94 passed, including the text-batch owner and plain
  rejection-decoration contracts.
- Rust 2024 rustfmt and scoped `git diff --check`: passed.

Managed Cargo did not run for this slice. The earlier managed attempt stopped in third-party `zstd-sys`
before Runtime compilation, and the one retry was not admitted by the managed CPU lane. Per repository
policy this work does not poll or retry the coordinator while the lane is reserved. A focused validation
request for the command-atomic rich admission regression was accepted as
`9df75274da66456d974c3e89b2d19f58`, but `cargo.acquire` had no terminal result after the validator's bounded
post-response reconciliation. No Cargo command or test result was produced, so this remains pending rather
than passed or failed and is not retried in parallel.

## Remaining acceptance work

- Run the focused Interface and Runtime Rust regressions through the managed Windows Cargo validator.
- Capture the pre-cutover 31-sample paint projection and inline geometry baselines before optimization.
- Run the real WGPU product framebuffer test and inspect
  `docs/tests/runtime/text/runtime_text_mvp_foundation_product_framebuffer_20260831.png`.
- Collect allocation/RSS and package-power evidence; do not claim performance improvement before it exists.
- Commit and send the quantified WeCom milestone only after accepted validation and a clean coordinator
  ownership/attribution preview.

Current status is `rich_paint_run_projection_all_or_empty_static_implemented /
plain_and_rich_command_rejection_fail_closed / text_plan_outcome_hard_cut /
non_finite_layout_geometry_rejected_before_batch_materialization /
safe_empty_layout_reshape_bypass_removed / raw_fallback_frame_admission_completed /
rich_run_geometry_preflight_completed / rich_run_positive_metric_admission_completed /
rich_artifact_admission_atomic_completed / rich_renderer_style_admission_parity_static_completed /
rich_empty_projection_plain_fallback_bypass_removed /
resolved_run_visual_slice_congruence_static_completed /
managed_validation_pending`.
