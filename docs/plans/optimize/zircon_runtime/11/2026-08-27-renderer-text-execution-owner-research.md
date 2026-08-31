# Runtime11 Renderer Text Execution Owner Research And Profiling Plan

- Date: 2026-08-27
- Owner plans: `docs/plans/zircon_runtime/runtime/11-job-system-task-model.md`, Runtime79, Runtime80
- Status: `research_complete_product_profile_required_before_code`

## Decision Boundary

This record reviews the product text worker topology before changing its
algorithm. It does not claim a performance, power, or shutdown improvement and
does not authorize replacing the raster worker implementation until the
current product path has a source-bound Windows profile.

The offline `zircon_font_sdf_bake` executable intentionally uses the process
compute owner and is not a Runtime-lifetime defect. The product renderer and UI
surface paths are the scope of this review.

## Current Owner Graph

```text
CoreRuntime TaskPools
  |-- Compute handle passed to WgpuRenderFramework only after SceneRenderer construction
  |-- AsyncCompute worker count copied as a number
  `-- no execution context reaches UiSurface extraction

SceneRenderer -> ScreenSpaceUiRenderer -> ScreenSpaceUiTextSystem
  |-- TextRenderState SDF scheduler -> process-default Compute pool
  `-- TextRasterWorkerPool -> N private zircon-text-raster-* threads

UiSurface rebuild -> render extract -> shape prewarm
  `-- process-default Compute pool
```

The central defect is ownership, not a small loop implementation:

- `TextRenderState::process_raster_worker_options` reads the Runtime-style
  `async_compute_threads` count, but `TextRasterWorkerPool::new` then creates
  that many additional operating-system threads. The diagnostic name
  `TaskPoolAsyncCompute` describes the numeric budget source, not shared
  execution ownership.
- `WgpuRenderFramework` already receives the product Runtime compute pool, but
  constructs `SceneRenderer` before storing that handle. The renderer's text
  subsystem therefore cannot consume it.
- `UiSurface` owns shape prewarm without an execution context and reaches the
  process pool independently of the graphics renderer.
- The private raster pool has bounded request count/bytes, bounded completion
  count/bytes, batching, cancellation, diagnostics, and synchronous Drop join.
  These semantics are valuable and must not be discarded merely to remove
  threads.

For Runtime configured with `A` async-compute workers and text configured from
that budget, the current process can own the Runtime's `A` workers plus another
`A` raster workers. The physical thread budget is therefore not conserved even
though diagnostics report the same budget source.

## Unreal Reference

`dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/SlateSdfGenerator.cpp`
uses `FAsyncTask<FSdfGeneratorTask>` on the engine background task system. It
does not create one private OS thread per SDF lane.

The important engineering properties are:

- `SlateSdfText.GeneratorPoolSize` bounds reusable task objects and defaults to
  one concurrent task;
- admission returns `BUSY` when no task object is free instead of creating
  more workers or an unbounded queue;
- `Update` harvests completed tasks and returns task objects to the free list;
- `Flush` first cancels work that has not started, then ensures completion of
  remaining tasks before clearing the started set;
- the font cache calls SDF flush during cache teardown.

The transferable rule is shared engine worker ownership plus subsystem-local
bounded admission and scratch/task-object reuse. Zircon should not copy
Unreal's exact pool size or task wrapper without product measurements.

## Candidate Architecture After Measurement

1. Introduce a small `RenderExecutionResources` construction input containing
   Runtime `Compute` and `AsyncCompute` handles. The product graphics module
   creates it before `SceneRenderer`; standalone renderer constructors create
   an explicit standalone owner.
2. Inject `Compute` into `TextRenderState` for SDF scheduling. Remove the
   product `new_with_process_raster_worker_budget` route rather than retaining
   a fallback inside the renderer.
3. Replace private raster threads with a bounded raster lane on Runtime
   `AsyncCompute`. Preserve the current request/completion byte budgets,
   duplicate suppression, cancellation, face-epoch invalidation, batch size,
   and diagnostics.
4. Keep rasterizer reuse explicit. Compare a bounded scratch-object pool with
   worker-local state keyed by Runtime generation; reject a design that adds a
   mutex or rasterizer allocation per glyph.
5. Give `UiSurface` a runtime/product-owned shape execution route, supplied by
   `UiRuntimeDriver` or its owning surface service. Do not let render extraction
   discover a global pool.
6. Shutdown order is UI surface admission close, renderer text flush/drain,
   graphics teardown, then `ExecutionRuntime` worker close/join. The text
   receipt must expose queued/running/completed/cancelled counts and retained
   completion bytes.

## Complexity Target

For `G` admitted glyphs, `W` Runtime async workers, and configured batch size
`B`:

- admission and duplicate lookup remain average `O(1)` per glyph;
- raster work remains `O(G)` plus glyph algorithm cost;
- worker/task fanout is bounded by `min(W, configured_text_concurrency)`;
- scratch/rasterizer state is `O(W)`, not `O(G)`;
- retained request and completion memory stays within the existing explicit
  byte limits;
- no per-frame polling thread and no additional OS workers are permitted.

## Required Windows Product Profile

Use one locked profiling build and record both idle and active intervals with
WPR/WPA or the repository's equivalent Windows profiler. Store traces and
reports under an E-drive target directory.

Profile these scenarios:

| Scenario | Workload |
|---|---|
| Idle | UI visible for 60 seconds with no invalidation |
| Cold Latin | 1, 100, 1,000, and 10,000 previously unseen glyph requests |
| Cold CJK | the same counts with multi-page atlas pressure |
| Color/variation | mixed bitmap/color/variation faces with fallback |
| Steady scroll | repeated visible-set turnover with cache hits and misses |
| Reload | one face generation change with pending raster/SDF work |
| Teardown | immediate close with empty, queued, and running work |

Record:

- process and named-thread count, worker active/parked time, wakeups, context
  switches, migrations, and CPU cycles;
- package/process power and idle CPU over the fixed 60-second interval;
- frame p50/p95/p99/max, render-extract join wait, raster queue wait, execution
  time, completion-drain time, and SDF latency;
- admitted/rejected/duplicate/cancelled/failed glyphs, batch-size histogram,
  request/completion high-water bytes, and cache hit/miss counts;
- teardown p50/p95/max and zero remaining text-owned threads or work after the
  receipt.

## Implementation Gate

Implementation may start only when the baseline trace identifies thread
oversubscription, wakeup/power cost, queue latency, or shutdown ownership as a
measured bottleneck and the chosen candidate includes a comparison trace.
Acceptance requires no private product text OS threads, conserved Runtime
worker counts, no regression in glyph correctness/budget semantics, no
statistically material steady-state frame regression, and a complete teardown
receipt. Until then the source defect is documented but not performance-closed.
