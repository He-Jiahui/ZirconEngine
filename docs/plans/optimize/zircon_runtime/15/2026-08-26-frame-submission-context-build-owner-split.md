# Runtime 15 frame submission context build owner split

## Scope

- Target: `zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs`.
- Baseline: clean 803-line current-source owner before this slice.
- Priority sources: `docs/plans/engine-code-structure-convention.md`, `docs/plans/engine-code-review-findings-2026-06.md`, Runtime 15 M3/M4, and the existing Runtime 07 F3 source-sharing guards.
- This slice changes source ownership only. It does not reorder frame construction, alter render features, claim a frame-time or power improvement, or close Runtime 07 performance acceptance.

## Architecture review

The root owns the ordered construction of one `FrameSubmissionContext`: viewport resolution, asset-backed extract hydration, pipeline compilation, effective post process, visibility/history, advanced feature payloads, and the final context handoff. That order is a real orchestration responsibility and remains intact.

The previous tail also owned four leaf responsibilities that can change independently:

1. Budget degradation projection into render scale, compile options, and SSR settings.
2. Effective view-family, previous-particle, post-process, temporal-history, and graph state projection.
3. UI command summary projection.
4. Eight behavior/source-contract tests and their fixtures.

Existing Runtime 07 and Plan 08 guards were reviewed before the split. The shared `Arc<RenderFrameExtract>` mutation/handoff sequence and the VG/HGI automatic payload product anchors stay in `build.rs`; the guards do not need relaxation or path rewriting.

The primary local Unreal reference was `dev/UnrealEngine`:

- `Renderer/Private/SceneRendering.cpp::FSceneRenderer::PrepareViewRectsForRendering` owns resolution/view-rect policy as an explicit view setup stage.
- `Renderer/Private/SceneVisibility.cpp::BeginInitViews` and `EndInitViews` separate visibility/resource initialization stages from the top-level render route.

The Zircon split follows the same engineering direction without copying Unreal's types: the root remains the ordered frame route while budget, effective-view state, and UI projection become named leaf owners.

## Implemented layout

| Owner | Responsibility | Current lines |
|---|---|---:|
| `build.rs` | Ordered context construction and VG/HGI product payload selection | 506 |
| `build/budget_degrade.rs` | Budget-to-render/compile/effect projection | 51 |
| `build/effective_view_state.rs` | View family, history, post-process, particle previous state, graph state | 98 |
| `build/ui_submission_stats.rs` | UI command-count projection | 23 |
| `build/tests.rs` | Existing behavior/source contracts plus root ownership regression | 193 |

The parent `build_frame_submission_context/mod.rs` and all callers remain unchanged. Concurrent edits in `compile_pipeline.rs` and `subsurface_profile_extract.rs` were not touched.

## Behavior invariants

- `build_frame_submission_context_from_source` retains its original statement order and `FrameSubmissionContext::new` handoff.
- The shared effective extract still uses `Arc::make_mut` and one final `Arc::clone`; removed full-extract and large payload clone forms remain absent.
- Budget degradation still applies before initial compile and again when the resolved post-process stack/MSAA sample count are known.
- Visibility/history validation still uses the same compiled feature names, camera key, target/render sizes, and pipeline bindings.
- Authored VG remains preferred over automatic fallback; HGI payload source semantics remain unchanged.
- UI stats still count commands, quads, text, images, and clipped commands using the same predicates.

## Current evidence and status

- Scoped `rustfmt --edition 2021 --check` passed across all 17 currently owned Rust files.
- Final scoped `git diff --check` passed, apart from the repository checkout's LF/CRLF notice.
- Static migration comparison found all 25 old function/type definitions and all 8 old tests in the new owners; one structure regression was added.
- The root product/hotpath contract passed 13/13 and the four forbidden clone anchors remain absent.
- All 29 current goal files exist and contain no trailing whitespace or conflict markers; both Runtime 15 plan-path contracts have zero missing paths.
- Managed Cargo was not requested while bypassing the shared validation blocker. Status is `implemented_static_passed_managed_validation_deferred`.

No profiler or power result is attached because no algorithm changed. Later frame-build optimization must use the existing Runtime 07 phase instrumentation and fresh scale baselines before changing the orchestration.
