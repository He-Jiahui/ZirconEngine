---
related_code:
  - zircon_runtime/src/text/parallel
  - zircon_runtime/src/text/native_bitmap_atlas.rs
  - zircon_runtime/src/text/native_bitmap_atlas/source_cache.rs
  - zircon_runtime/src/text/render_state.rs
  - zircon_runtime/src/ui/surface/render/text_prewarm.rs
  - zircon_runtime/src/ui/surface/render/extract.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/SlateSdfGenerator.cpp
  - dev/bevy/crates/bevy_tasks/src/lib.rs
tests:
  - current-source Windows zircon_runtime text parallel tests pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime text parallel逐文件性能静态审查（2026-07-18）

## 范围与覆盖

`zircon_runtime/src/text/parallel/**`当前源4/4个Rust文件、1,110行已逐文件阅读：`mod.rs`、`shape_pool.rs`、`raster_pool.rs`与`tests.rs`。同时回查生产接线`text/render_state.rs`、`native_bitmap_atlas.rs`、`native_bitmap_atlas/source_cache.rs`与`ui/surface/render/{extract,text_prewarm}.rs`，并核对`core/runtime/tasks/parallel_for.rs`的同步语义。`tests.rs`含Text01活跃owner的face-epoch hard-cut改动，本审查只读取当前源，未覆盖或改写。

## PERF-MVP-229：raster队列与字体复制

生产`TextRenderState`以`TextRasterWorkerPoolOptions::new(worker_count)`创建worker，未设置queue depth；request channel与completion channel因此都为unbounded。worker发布结果时立即从`in_flight`删除，故尚未被render线程drain的`GlyphBitmap.data`不计入in-flight/queue-peak，也没有completion backlog bytes指标。`native_bitmap_atlas_frame`每帧把completion channel全部try-recv并全部插入source cache，没有count/byte预算。

更严重的是每个glyph miss都构造`Arc::<[u8]>::from(font.data())`。cosmic-text 0.18.2源码确认`Font::data`返回`&[u8]`，该转换会完整复制字体文件，而不是增加共享引用；同一face的G个新glyph产生O(G × font_bytes) caller-thread复制。Drop关闭request sender后join workers，而crossbeam会先消费buffered requests，极端队列可把编辑器/运行时退出变成长时间同步栅格。

## PERF-MVP-230：同步shape prewarm

UI extract在command构建前调用`prewarm_visible_owner_text`，command构建后又调用`prewarm_render_command_text`。两次都物化requests Vec与paragraphs Vec；`shape_paragraphs_with_cache`对每个请求用`pending.iter_mut().find`做批内去重，唯一miss数U时为O(U²)。存在miss时`parallel_for`进入Rayon `TaskPool::install`并同步等待，因此不是跨阶段异步pipeline，而是在render extract关键路径最多形成两次worker barrier。

该prewarm使用进程常驻、固定2线程的独立`OnceLock<TaskPool>`，不读取全局`TaskPoolOptions`，可能与engine compute/async-compute/raster workers叠加。多线程确实存在，但caller wait、oversubscription与重复barrier未被report捕获。

## 参考引擎结论

Unreal `FSlateSdfGeneratorImpl`只从有限`FreeTasks`池取任务；满载直接返回`BUSY`，需要时给placeholder。`Update`只回收已完成任务，`Flush`先取消可取消任务，再只等待已开始工作。该模型把并行数、背压、降级与shutdown语义放在同一owner内。Bevy通过全局`ComputeTaskPool`/`AsyncComputeTaskPool`用法路由线程预算，避免子系统私建固定线程数成为第二套调度政策。

## 责任计划与验收

Text09 PF-M2/PF-M3收到`failure-2026-07-18-text-parallel-queue-budget-and-font-copy.md`。raster应按face epoch共享字体bytes，request/completion按count与bytes有界，每帧只apply预算量并能取消未开始工作；shape应接共享compute预算，把owner收集与command build重叠，合并唯一miss且最多join一次，batch dedup走hash bucket并保留exact text校验。

确定性门禁包括：同一face 1/100/1k新glyph的font copied bytes≤一次face载入；request/completion backlog、单帧apply glyph/bytes与shutdown canceled/running数有明确上限；1/100/1k/10k labels的shape barriers≤1、dedup probes近O(U)、线程数不超全局budget。Cargo、workbench scroll/Console burst、退出trace与像素等价完成前，目录继续留在`pending.md`。
