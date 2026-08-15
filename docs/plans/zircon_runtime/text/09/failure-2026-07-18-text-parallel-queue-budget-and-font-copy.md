---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: text-parallel-queue-budget-and-font-copy
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/text/09
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/text/parallel
  - zircon_runtime/src/text/native_bitmap_atlas.rs
  - zircon_runtime/src/text/native_bitmap_atlas/source_cache.rs
  - zircon_runtime/src/text/render_state.rs
  - zircon_runtime/src/ui/surface/render/text_prewarm.rs
  - zircon_runtime/src/ui/surface/render/extract.rs
---

# Text parallel队列预算、字体复制与同步barrier

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_runtime/src/text/parallel/**`当前源4/4 Rust文件及生产接线回查
- 修复责任计划：`docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md`
- 责任切片：PF-M2/PF-M3。
- 交接原因：线程预算、raster backpressure、首帧降级、shape调度与退出语义由Text09拥有；相关source cache和Text09主计划正由活跃owner修改，本审查不并发覆盖。

## 失败现象与复现证据

PERF-MVP-229：生产raster request/completion channel默认unbounded；completion发布后即从in-flight删除，未drain bitmap backlog不可见；frame入口无预算地drain/apply全部完成结果，Drop会等待buffered requests全部执行。每个glyph miss还执行`Arc::<[u8]>::from(font.data())`，而cosmic-text的`font.data()`为`&[u8]`，所以同一字体文件按新glyph数完整复制。

PERF-MVP-230：UI extract在commands前后各做一次prewarm；每批重新物化requests/paragraphs，唯一miss用Vec线性find去重；miss通过Rayon `TaskPool::install`同步等待。固定2线程prewarm pool不消费全局TaskPoolOptions，changed/scroll帧可出现两次render-thread barrier与线程预算叠加。静态证据见`docs/plans/performance/01/2026-07-18-text-parallel-static-review.md`。

## 最低共享层根因

Text09已经定义“主线程只装配/上传”和每帧≤256新glyph、≤2 MiB上传预算，但worker API只约束线程数，没有约束input/output backlog、结果apply、字体payload ownership、取消或caller join。shape report只计请求/命中/worker parallelism，不计barrier、caller wait、dedup probes与实际全局线程预算。多线程实现因此可能把同步工作转换为不可见积压，而非稳定帧预算。

## 架构修复验收

- 按`face_epoch + backend font id`缓存共享font bytes；同一face的work item只clone Arc。若FontDb source首次需要复制，最多一次/face epoch，并记录copied bytes。
- request queue按glyph count与估算输入bytes有界；饱和返回typed busy/rejected，由现有approximate/transparent placeholder路径降级，不同步raster、不panic。
- completion backlog按count与bitmap bytes有界；提供`drain_completed_for_face_epoch(max_items, max_bytes)`或等价budget API，剩余结果跨帧保留。queued/running/completed backlog/rejected/canceled/apply bytes分别诊断。
- shutdown发取消信号并丢弃未开始请求，只有已运行任务允许有界join；不得因为历史queue depth处理全部glyph后才退出。
- UI shape接共享compute pool预算；owner prewarm可与command build重叠，command requests合并去重后只在layout依赖点join一次。小批量按标定阈值inline，禁止固定额外2线程政策。
- batch dedup使用hash bucket + exact text比较，和PERF-MVP-228 indexed shaped cache协同；保持输出顺序、collision/direction alias与report语义。
- 门禁覆盖1/100/1k glyph、1/100/1k/10k labels、1/2/4/8 available threads；记录font copied bytes、queue/backlog bytes、apply budget、barriers、dedup probes、caller wait与shutdown canceled/running。

## 禁止临时方案

- 不得只把request channel改成很大的固定容量，同时保留无界completion与全量drain。
- 不得在线程池满时回退到caller-thread同步raster/shape；应走已有placeholder或下一帧重试。
- 不得让bounded completion send与Drop join互相等待；取消/关闭协议必须可终止。
- 不得通过增加固定线程数掩盖两次同步barrier或O(U²)去重，也不得把test wall-clock当作产品主线程预算。

## 修复结果与回传

2026-08-10 implementation state: `open / resolving_failure / non_validation_implementation_complete / secondary_review_complete / managed_validation_pending`.

- Native bitmap raster work shares `Arc<[u8]>` font bytes from `NativeBitmapAtlasSourceCache`, keyed by backend font id and cleared with face invalidation. The first source copy is counted; later glyph work only clones the `Arc`.
- `TextRasterWorkerPool` bounds request count/input bytes and completion count/bytes, exposes queue/running/completed/failed/peak/backlog/rejected/cancelled diagnostics, and drains only through `drain_completed_for_face_epoch(max_items, max_bytes)`. Source-cache application preserves budget-deferred completions for a later frame, clears pending work on typed worker failure, and maps request pressure to the existing placeholder/deferred path.
- Worker shutdown marks cancellation before joining, including a completion-backpressure regression case. Deterministic tests cover queue fullness, completion byte limits, budgeted drains, face-epoch discard, cancellation, and backlog release.
- Paragraph shaping hashes and deduplicates same-frame misses, uses the shared compute-pool parallelism, keeps small batches inline, and performs exactly one parallel join for a non-inline batch while reporting caller wait and generation deferral.
- Managed current-source Cargo, profiler execution, and live product trace remain coordinator-owned. The source now contains the requested 1/100/1k glyph by 1/2/4/8 worker matrix, the 1/100/1k/10k label product matrix, and a real source-cache retry regression after strict completion-byte rejection. This is a non-validation completion record, not an acceptance claim; retain `open` until a coordinator receipt is attached.

2026-08-10 forward-fix update:

- Completion admission is now nonblocking and strict: a raster bitmap that cannot reserve its exact byte count is released immediately, while a zero-byte failure completion clears the source-cache pending id so a later frame can retry. Worker-side budget waiting was removed; queued completion bytes cannot exceed the configured pool-wide budget. A separately documented drain-side liveness exception accepts one already-admitted oversized queue head at the start of a frame, then defers every following completion.
- Production frame reports now carry font-copy, request backpressure/cancellation, completion drained/deferred/applied bytes, and worker-pool thread/in-flight/queued/running/completed/failed/queue-peak/backlog/rejection totals. These values are projected into `ScreenSpaceUiTextRasterUploadReport` and fixed profiling counters on both text and idle frames.
- Owner prewarm requests are collected before render-command construction. Batches at or above the calibrated parallel threshold use the process shared compute pool's caller-preserving in-place scope: only owner shaping moves to a worker, while render-command construction and its frame-local profiling remain on the render caller. The worker receives an explicit captured frame context, the caller joins once at the layout dependency point, and the post-build phase publishes no duplicate empty prewarm sample.
- Component painters that can emit their own text, including text fields and open popup rows, conservatively disable owner overlap so their eager layout keeps the shared cache. Remaining owner and popup requests merge into one post-command-collection prewarm/shape batch. The cold-frame profiling regression requires `owner_overlap_joins=0`, exactly one non-empty prewarm sample/span and one non-empty shape-batch sample/span, all attached to the caller frame; the stable frame requires cache hits and resolved layouts for both component routes. Small owner-only batches retain the existing inline path, and true viewport-partial plain documents remain excluded from full-source prewarm.
- The 10k-label managed baseline still creates 10,000 distinct nodes and payloads, but cycles through 512 stable text identities so the 1,024-entry shaped-run cache can converge. The 1/100/1k cases remain fully unique, while the 10k case continues to exercise large command/layout/atlas draw pressure without an impossible perpetual shape-cache-thrash zero-work gate.
- Direct `rustfmt`, scoped `git diff --check`, and the 800-line owner threshold pass for the touched implementation. Independent secondary review found no P0/P1/P2 after the bounded 10k identity and mixed-component profiling repairs. Coordinator-owned Cargo/profiling/WGPU gates remain pending, so no runtime or screenshot acceptance is claimed by this update.

2026-08-10 measurement-contract forward-fix:

- The production static-label profiler now separates forced surface-projection rebuilds from clean
  retained frames and exposes Plain owner document resolves that bypass the persistent layout
  cache. Its WGPU gate requires the exact `runtime-ui` / `ui.screen-space` pass and that pass's own
  GPU timestamp on every measured generation.
- A separate persistent layout-cache pressure profiler preserves the original 1/100/1k settled
  hit and 10k capacity-miss contract without misrepresenting the production owner bypass. The 10k
  row uses unique layout keys but only 512 shaped-text identities, so the experiment isolates
  layout-cache capacity rather than shaped-cache thrash.
- Focused profiling tests now require zero persistent layout-cache hits and misses for the Plain
  document bypass, plus a rich-owner negative case that reports zero bypasses and one real layout
  miss. Production resolution and profiling share one request-owned bypass predicate, so the
  diagnostic cannot drift from routing. Independent secondary review found no P0/P1; its final P2
  was incomplete Rust 2021 import formatting in the measurement-scope `extract.rs` and
  `render_profiling.rs` owners. That P2 is forward-fixed and the complete M0 measurement file set
  passes `rustfmt --edition 2021 --check`. Managed profiler/Cargo/WGPU execution remains pending.

2026-08-10 completion-drain liveness forward-fix (secondary review complete):

- A pool-wide completion byte budget scales with worker count, while native atlas drain retains a
  fixed per-frame byte allowance. A completion admitted by the former could otherwise remain at
  the deferred queue head forever when it exceeds the latter. The drain now applies one such
  completion only when its frame has applied no bytes, records the explicit oversized acceptance,
  and still defers every following completion. The focused two-worker regression covers the
  oversized head plus a following completion; the profiler mapping regression asserts the fixed
  counter name and value. Managed Cargo, profiler, and WGPU validation remain pending.
- Independent static review found no P0/P1/P2. The completion-byte budget remains the documented
  admitted-backlog bound; it deliberately does not claim a pool-wide cap over transient worker
  raster allocations.

2026-08-11 viewport-routing and measurement-contract forward-fix:

- The retained-document route now bypasses persistent layout reuse only when the shared hard-line
  query selects a strict Plain/HorizontalTb/None/Clip subset. Complete viewports and vertical
  owner text use the persistent cache, while partial geometry remains same-frame deduplicated.
  `uncached_document_resolves` is sampled from the actual partial branch rather than request
  metadata.
- The 300-frame forced-label baseline now requires zero uncached-document resolves. Its 1/100/1k
  document-key rows require settled layout-cache hits; its 10k row exceeds the 2,048-entry cache
  and therefore requires deterministic cache misses while retaining 512 shaped-text identities.
  This preserves the source-cache convergence gate without claiming a false steady layout-cache
  state. Managed validation remains pending, so this update does not change the handoff status.

2026-08-11 M5 viewport hot-path forward-fix:

- Canonical hard-line infrastructure now exposes an allocation-free multiple-line predicate. The
  strict Plain/HorizontalTb/None/Clip owner path invokes the retained parsed-document/index route
  only when a source separator or the 64-KiB shaping-cap can produce more than one canonical line.
  Ordinary one-line labels remain on the complete prewarm/layout path without churning the bounded
  16-entry retained-document and hard-line-index owners.
- Persistent layout-cache lookup now runs before the exact viewport classification. Because the
  layout key includes viewport geometry, a complete cached viewport can return without a fresh
  hard-line probe while a new partial viewport still misses and follows the fail-closed shared
  query. Static regressions cover CRLF, Unicode separators, cap splitting, and cross-frame cache
  reuse without an additional index hit. Scoped Rustfmt and diff checks pass; managed Cargo,
  profiler, WGPU, and screenshot validation remain pending.

2026-08-13 source-cache report owner hard-cut:

- State: `implementation_complete / scoped_static_validation_complete /
  secondary_review_complete / managed_validation_pending`.
- The frame diagnostics DTO and worker-pool projection now belong to the folder-backed
  `native_bitmap_atlas/source_cache/report.rs` owner. The source-cache parent remains responsible
  for cache state, request/completion application, face-epoch invalidation, and frame-report
  population; the crate-local `source_cache::NativeBitmapAtlasSourceCacheFrameReport` re-export is
  unchanged for existing consumers.
- This structure-only hard cut reduces the production source-cache owner from 798 to 718 lines and
  creates an 83-line report owner. It does not change cache keys, budgets, queue admission,
  eviction, rasterization, or fallback behavior.
- Direct Rust 2021 formatting and scoped `git diff --check` pass. Coordinator-owned Cargo,
  profiling, WGPU, and screenshot validation remain pending, so this update is not an acceptance
  claim and does not close the failure handoff.
- Independent read-only secondary review checked all 53 report fields, all 15 worker-pool
  diagnostics mappings, active/idle consumers, crate-local re-export visibility, and the ownership
  budgets, and found no P0/P1/P2. The review did not run Cargo or produce visual evidence.

2026-08-13 completion-application telemetry forward-fix:

- State: `implementation_complete / scoped_static_validation_complete /
  secondary_review_complete / managed_validation_pending`.
- `worker_completion_applied_byte_count` now advances only after the completion image is admitted
  into `NativeBitmapAtlasSourceCache`. A completion rejected by the source-cache entry or byte
  budget remains represented by its drained bytes and budget-rejection counter, but is no longer
  misreported as applied work.
- The complete prepare-report mapping fixture now supplies and asserts a non-zero
  `persistent_raster_key_count`. This keeps the source-cache report to render-statistics telemetry
  projection compile-complete and prevents the new field from being silently defaulted in the only
  explicit report expectation.
- The deterministic source-cache regression sends an 8-byte worker completion into a 4-byte cache
  and requires `insert_count=0`, `applied_bytes=0`, `drained_bytes=8`, a single budget rejection,
  and no leaked pending work. This is a forward fix to report semantics only; it does not tune
  queue depth, cache capacity, rasterization, or fallback behavior.
- Scoped Rust 2021 formatting and `git diff --check` pass. An independent read-only secondary
  review found no P0/P1/P2: applied bytes, insert counts, budget rejections, drained bytes,
  pending cleanup, persistent raster-key binding, and the prepare-report fixture projection remain
  semantically distinct. This update has not run Cargo, WGPU, profiling, or screenshot capture;
  the handoff remains `open` until the coordinator records the declared managed validation
  evidence.
