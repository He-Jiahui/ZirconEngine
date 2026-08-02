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

2026-08-01 implementation state: `open / resolving_failure / non_validation_implementation_complete / managed_validation_pending`.

- Native bitmap raster work shares `Arc<[u8]>` font bytes from `NativeBitmapAtlasSourceCache`, keyed by backend font id and cleared with face invalidation. The first source copy is counted; later glyph work only clones the `Arc`.
- `TextRasterWorkerPool` bounds request count/input bytes and completion count/bytes, exposes queue/running/backlog/rejected/cancelled diagnostics, and drains only through `drain_completed_for_face_epoch(max_items, max_bytes)`. Source-cache application preserves deferred completions for a later frame and maps queue pressure to the existing placeholder/deferred path.
- Worker shutdown marks cancellation before joining, including a completion-backpressure regression case. Deterministic tests cover queue fullness, completion byte limits, budgeted drains, face-epoch discard, cancellation, and backlog release.
- Paragraph shaping hashes and deduplicates same-frame misses, uses the shared compute-pool parallelism, keeps small batches inline, and performs exactly one parallel join for a non-inline batch while reporting caller wait and generation deferral.
- Managed current-source Cargo, the requested thread/label scale matrix, and live product trace remain coordinator-owned. This is a non-validation completion record, not an acceptance claim; retain `open` until a coordinator receipt is attached.
