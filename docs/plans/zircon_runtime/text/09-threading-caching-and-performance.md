---
related_code:
  - zircon_runtime/src/text/cache/mod.rs
  - zircon_runtime/src/text/cache/frame_dedup.rs
  - zircon_runtime/src/text/cache/layout_cache.rs
  - zircon_runtime/src/text/cache/measure_cache.rs
  - zircon_runtime/src/text/cache/shaped_cache.rs
  - zircon_runtime/src/text/cache/tests.rs
  - zircon_runtime/src/text/font/shared.rs
  - zircon_runtime/src/text/font/runtime_asset.rs
  - zircon_runtime/src/text/font/shared/tests.rs
  - zircon_runtime/src/text/font/database/equivalence.rs
  - zircon_runtime/src/text/sdf/font_bake/tests/cache_generation.rs
  - zircon_runtime/src/text/parallel/mod.rs
  - zircon_runtime/src/text/parallel/shape_pool.rs
  - zircon_runtime/src/text/parallel/raster_pool.rs
  - zircon_runtime/src/text/parallel/tests.rs
  - zircon_runtime/src/text/hard_line.rs
  - zircon_runtime/src/text/shaping/work_budget.rs
  - zircon_runtime/src/text/shaping/tests.rs
  - zircon_runtime/src/ui/text/measure_cache.rs
  - zircon_runtime/src/ui/surface/render/extract.rs
  - zircon_runtime/src/ui/surface/render/text_prewarm/tests.rs
  - zircon_runtime/src/ui/tests/text_pipeline
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_renderer/renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_renderer/tests.rs
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/font_assets.rs
  - zircon_runtime/src/text/native_bitmap_atlas.rs
  - zircon_runtime/src/text/native_bitmap_atlas/source_cache.rs
  - zircon_runtime/src/text/native_bitmap_atlas/tests/source_cache.rs
  - zircon_runtime/src/text/native_bitmap_atlas/storage.rs
  - zircon_runtime/src/text/atlas/bitmap_run.rs
  - zircon_runtime/src/text/atlas/bitmap_run/tests/persistent_slots.rs
  - zircon_runtime/src/text/atlas/bitmap_run/types.rs
  - zircon_runtime/src/text/atlas/page.rs
  - zircon_runtime/src/text/atlas/raster_key/mod.rs
  - zircon_runtime/src/text/atlas/render_submission/report.rs
  - zircon_runtime/src/text/atlas/slot_cache.rs
  - zircon_runtime/src/text/atlas/slot_cache/tests.rs
  - zircon_runtime/src/ui/surface/render/font_dependencies.rs
  - zircon_runtime/src/dynamic_api/session/runtime_ui.rs
  - zircon_runtime/src/dynamic_api/session/runtime_ui/font_admission.rs
  - zircon_runtime/src/text/document/edit.rs
  - zircon_runtime/src/text/document/index.rs
  - zircon_runtime/src/text/document/index_profile.rs
  - zircon_runtime/src/text/document/storage.rs
  - zircon_runtime/src/text/document/tests.rs
  - tools/tests/test_runtime_text_document_incremental_index_contract.py
design_references:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/ShapedTextCache.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/SlateSdfGenerator.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Fonts/FontCache.h
  - dev/bevy/crates/bevy_text/src/pipeline.rs
  - dev/bevy/crates/bevy_text/src/font_atlas_set.rs
  - dev/bevy/crates/bevy_tasks/src/lib.rs
  - dev/godot/core/object/worker_thread_pool.cpp
plan_sources:
  - docs/plans/zircon_runtime/text/index.md
  - docs/plans/zircon_runtime/text/02-shaping-unicode-and-bidi.md
  - docs/plans/zircon_runtime/text/04-glyph-atlas-and-rasterization.md
  - docs/plans/zircon_runtime/render/14-2d-stack.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
status: in_progress
---

# 09 多线程 / 缓存体系 / 性能与精度预算

> 本计划把 `01–05` 的同步实现并行化、异步化,定稿三级缓存契约,把文本性能纳入 `render/17` 的确定性计数验收。目标:大段文本/多 TextField/滚动列表下不掉帧,高精度不以性能为代价。

## 1. 目标

1. **多线程 shaping/栅格**:并行 per-paragraph shaping、并行 glyph 栅格(SDF/MSDF 生成 CPU 密集);复用 `asset/pipeline/worker_pool` 或 rayon;主线程只装配/上传。
2. **异步栅格上传**:glyph 栅格在 worker,GPU 上传在渲染线程合并;首帧缺字降级(占位/低模)避免阻塞。
3. **三级缓存契约**:shaped run cache(`02`)、measure cache(`03`)、glyph atlas cache(`04`)——键、容量、逐出、帧戳统一定义;同帧去重(避免重复 shaping)。
4. **性能与精度预算**:确定性计数(shape 次数/栅格次数/上传字节/缓存命中率)进 `render_perf_text_*` 测试;精度桶(scale 量化)与性能权衡书面化。

## 2. 现状与差距

- 文本全链单线程:shaping(启发式)、SDF 烘焙、上传都在主/渲染线程。
- `measure_cache.rs`:宽度桶缓存在(喂启发式),无 shaped run cache、无 atlas 命中统计。
- `worker_pool.rs`:资产管线有 worker pool,文本未用。
- 缺口:无并行 shaping/栅格、无异步上传、无统一缓存契约、无同帧去重、无性能计数。

### 2026-08-24 当前架构复核与性能基线门

本轮已完成 Text02/Text03 到 UI cache/publication 的 M2b typed-outcome 硬切换，尚未执行受管 Cargo、WGPU 或功耗采样，因此**不能**把下列结论表述为已测得的性能瓶颈。原先的结构问题是 `TextShapeRunProvider`、量度/换行及其上层布局消费者会把失败折叠成空 `ShapedGlyphRun`，使零宽 break chunk、空 glyph artifact 或 UI layout cache 复用成为可能；该问题现由单一 `Ready`/`Deferred`/`Failed` 路径收束，失败不会生成 run、artifact 或缓存条目。这是已完成的正确性与缓存准入修复，不是可凭静态审查量化的性能结论。

后续实现必须以一次硬切换收束，不保留兼容 provider 或 renderer-side shaper：

1. canonical shape 与 direct/session provider 统一返回保留 `Ready`/`Deferred`/`Failed` 的结果；parallel prewarm 已作为 Ready-only cache-admission stage 收敛，只有 `Ready` 可进入 shaped-run、measure 和 layout cache。
2. line metrics、grapheme advance、line break/boundary correction、rich/vertical materialization 向上传播该结果，禁止以空运行替代真实几何。
3. UI layout publication 在唯一边界应用显式安全策略：`UiTextMeasureKey` 已包含 font-database generation，故不得把上一 generation 的 ready layout 伪装成同 key 当前几何。当前 generation 的 exact ready 会在 shape 前直接命中；若 shape 仍返回 deferred/failed，则不发布虚假或陈旧几何、不写入 frame/persistent cache，并发布既有 overflow-clipped safe layout 与低基数诊断。render extract 不缓存非 ready 结果。
4. 完成这一正确性切换后，才运行 1/100/1k/10k 行的纯 Latin/CJK/Arabic/RTL/emoji workload，并同时记录 `text.shape_batch`、shaped/layout/measure cache hit/miss、caller wait、分配/RSS、p50/p95/p99 和进程功耗。对比基线必须使用相同字体、DPI、窗口、warm-up、device 和 frame 数；计数零漂移与真实 framebuffer 均为独立验收项。

secure TextField 原子切换后，基线还必须单独记录 `UiSecureTextPresentation` 的 grapheme 数、`BidiLineSignature` scalar 数、构建时间与临时/常驻字节，以及实际物理行重放 L1/L2 的时间；记录只允许使用计数和低基数类别，禁止把密码原文写入 profile/diagnostic。signature 的物理行解析必须以 source-order cluster/scalar 的单次前向扫描完成，禁止 per-grapheme 回扫形成二次复杂度；这是一项接线前的算法契约，不是未测 profile 的性能结论。该 signature 是正确性元数据，不得在没有这组基线前擅自压缩、量化或把 source offset map 缓存在共享 layout cache。

同一安全路径的 artifact 投影也完成了静态复杂度复核：`presentation_glyphs_for_line` 的输入是已经按 physical visual order shaping 的 mask glyph 与一 grapheme 一 run 的 presentation map。旧实现为每个 glyph 对全部 run 作位置搜索，最坏为 `O(G*R)`；这是源码可证明的结构风险，但在尚未获准运行 profile 前不能称为已测热点。实现必须先校验 run 的连续 visual coverage，再以单调 glyph visual range 驱动一个 run cursor，达到 `O(G+R)`；range 回退、跨 run glyph、缺失 glyph 或不完整 coverage 必须拒绝整份 artifact，绝不能回退为原文或猜测 source range。后续 secure 基线增加 `secure_artifact_projection_glyph_count`、`secure_artifact_projection_run_count`、`secure_artifact_projection_elapsed_ns` 与拒绝类别计数，并在 1/100/1k/10k grapheme 样本上确认时间与 `G+R` 同阶。

secure layout 的 display-range replay 也不得因软换行放大复杂度：presentation hard-line 与 cluster ranges 在构造时按 display/source offset 严格单调，所以 `bidi_for_display_range`、layout adapter 的 cluster lookup 与 atomic source/display caret-boundary conversion 都必须通过二分边界定位，而不是每个物理行或编辑边界从首 cluster `find/position`。物理行仍只构造自身不相交的 `logical_ranges`，总 scalar/range 重放工作保持对实际显示 cluster 数线性；跨 hard line、非原子边界或 separator range 继续 fail closed。后续 secure 基线将把 `secure_replay_physical_line_count`、`secure_replay_cluster_count`、`secure_replay_lookup_elapsed_ns` 与 reject 类别分开记录，确保 lookup 不是窄宽长行的隐藏二次项。

M2b 的代码切换边界已经完成：`TextShapeRunProvider` 的 horizontal/vertical 请求和 `text/layout/{measure,advance_index,line_break,boundary_correction,rich,rich_advance_index,rich_vertical}` 均向上传播 typed outcome，不在局部以零宽替代；`ui/text/layout_engine` 的 plain/rich/vertical/ellipsis/paragraph/table 入口同样一次返回失败，`UiTextMeasureCache::resolve_or_shape` 是唯一 publication/cache-admission 边界。该边界只允许 exact Ready 进入 frame/persistent cache 或 glyph artifact；deferred/failed 不进入任一 cache，且只得到 overflow-clipped safe layout 与低基数 error counter。因为 key 已含 font generation，这个 safe result 绝不跨 generation 复用旧 layout；`shape_fallback_for_error` 和 provider/direct path 的空 run 已删除，未新增兼容 provider、陈旧-layout cache 或 renderer-side shaper。

当前基础契约：`text/shaping/outcome.rs` 已把原先仅承载 `Arc<ShapedGlyphRun>` 的 outcome 提升为泛型 `TextLayoutOutcome<T>`（shaping 继续使用 `TextShapingOutcome` 默认类型），并保留 `Ready` map/chain 与 deferred/failed disposition。`TextShapeRunProvider`、所有布局消费者、富表格递归和 UI 缓存准入均已迁移；`failed_shape_is_neither_cached_nor_published_as_empty_geometry` 锁定 zero-geometry path 不可发布或缓存。状态为 `m2b_non_validation_implementation_complete / managed_validation_pending`，不是 Text09 或 Text05 acceptance。

`m2b_artifact_ready_only_implemented`：`SharedTextLayoutSession` 与 `ResolvedTextGlyphArtifact` 仅在每个 shaping outcome 为 `Ready` 时构建和发布；任一 `Deferred`/`Failed` 会记录既有低基数 layout error 并拒绝整份 artifact，不能把空 run 或部分失败结果发布为 glyph artifact。视觉/synthetic 行的既有按行 fallback 不等同于 typed failure，仍保留其原有行为；provider/direct、measure/line-break、rich/vertical/table 和 UI layout cache 的旧 fallback 已随 M2b 删除。

### 2026-08-29 Core font admission 与 prepare-time fallback

`TextModule` 在 Core `Services` 层发布唯一 `FontCollectionService`；Graphics renderer 与动态 Runtime UI
surface builder 都从该服务取得集合，避免同一进程多 project/session 共享可变 fallback registry。动态 UI
加载先构造 retained surface、汇总默认及显式 `font` asset URI，再在首次布局前执行一次 admission/publish。
布局 cache 因此从首帧开始使用正确的 collection revision；字体缺失保持内置 last-resort 可用并只增加低
基数 admission failure 计数。

Screen-space plan 阶段只保留 raw text batch；字体依赖准备完成后，renderer-owned collection 才执行
canonical fallback shaping，避免 plan 期间误用 `shared_text_layout_service()`。已有 glyph artifact
仍走 zero-reshape lease 路径。`UiSurface::compute_layout` 现在在布局入口接通 text-font generation
失效钩子，确保外部发布会使 surface text dirty。上述为静态实现与源码回归范围；受管 Cargo、真实 WGPU
framebuffer/PNG、profile/RSS/power 及 Unreal 对拍仍是 validation pending，不能把计数或策略文本当验收证据。

字体 owner 的 project/session 回收尚未关闭：现有 renderer cache 可重复 admission，但不发布跨 UI consumer
的 scope lease，也不会在 project switch 的首次布局前可靠裁剪全部旧 owner。按单个 dynamic surface 集合
直接删除会误伤 HUD/菜单等消费者，因此正确后续是 collection-owned claim/release transaction，以一次
generation publish 更新 active owner set；门禁需覆盖 project switch、hot reload、缺失 asset、共享 face
去重、fallback 隔离、驻留上限与稳定帧零 mutation。

### 2026-08-28 retained document 结构优化与实测收口

本轮先复核 `text/document` 全模块及 Unreal Slate retained line/edit transaction，再对同一套生产源码直引 harness 运行 17 场景、每场景 31 样本的优化前矩阵。基线证明两个结构热点：每次 replacement 建立独立 addition chunk 导致顺序输入反复重建增长中的 piece list；separator-neutral 编辑仍构造并重解析完整 hard-line envelope，使百万字符单行的每次字符编辑复制约 2 MB。该结论来自 wall time、计数分配、RSS、snapshot/index 分相统计，不是静态猜测。

实现已硬切到单一 append-only addition source，piece 只保留该逻辑 source 的 byte range；separator-neutral 且完全落在一个 hard-line content range 内的编辑只更新该 stable line model 的 content length，触碰或引入 CR/LF（包括 CRLF 中间插入）仍走 separator-aware reparse。prepare/expected-revision/commit、旧 snapshot lease、stable hard-line ID、dirty receipt 与显式 store admission 均保留，没有引入 rope/gap buffer/tree、猜测 compaction interval 或第二文档 authority。

同一矩阵的优化后数据已写入 `docs/tests/runtime/text`：10k ASCII 尾插 p50 从 1,710.706 ms 降到 4.508 ms（379.46x），计数分配从 8.127 GB 降到 3.643 MB（2,231.08x），最终为 1 addition source / 2 pieces；百万字符 base 的 100 次尾插从 711.913 ms 降到 0.061 ms，100 次中间插入从 799.927 ms 降到 0.034 ms。CJK/combining/ZWJ emoji 的 1k 尾插同样保持 1 source / 2 pieces。52/52 direct-source 测试为 49 项 production document/hard-line/store 用例加 3 项结构守卫；完整原始 JSONL、summary、comparison 与 SHA-256 由 `docs/plans/optimize/zircon_runtime/82/2026-08-28-retained-document-edit-baseline-and-structural-direction.md` 持有。

当前 Windows policy 以 `0xc5585011` 拒绝 WPR CPU profile，所以 sampled stack、energy/package power 与 matched Unreal runtime 数据仍明确开放；这些缺口不推翻已经在相同 workload 中消失的两个算法热点，也不允许声称功耗或 Unreal 性能已对齐。Surface/session document authority、delta history 与 focus-loss owner 已实现但未受管验收；产品阈值、完整 Runtime module graph、真实 WGPU framebuffer 与 PNG 仍是后续接线/验收项。不得用源码计数、旧日志或策略文本截图替代真实基线或 framebuffer，也不得向 `target/` 写验证产物。

本计划状态为 `architecture_review_complete / m2b_non_validation_implementation_complete / retained_document_structural_profile_and_optimization_complete / surface_document_session_history_and_focus_owner_implemented_unvalidated / managed_runtime_and_product_validation_pending / power_and_matched_unreal_pending`。

## 3. 参考代码

| 文件 | 应重点阅读 |
|------|-----------|
| `dev/UnrealEngine/.../Framework/Text/ShapedTextCache.h` | `FShapedTextCache`:`FCachedShapedTextKey { TextRange, Scale, TextContext, FontInfo }`、LRU、`GetShapedText`/`FindShapedText`——**shaped run 缓存键与查找主样板**(`render/14` 已改造为不持引用键) |
| `dev/UnrealEngine/.../Fonts/SlateSdfGenerator.cpp` | SDF **异步生成**:请求队列、worker 生成、完成回调、生成中占位——**异步栅格路径样板** |
| `dev/UnrealEngine/.../Fonts/FontCache.h` | `FSlateFontCache::FlushCache`/帧末 trim、atlas dirty 跟踪 |
| `dev/bevy/crates/bevy_text/src/pipeline.rs` | `TextPipeline` 的 per-block shaping、缓存复用;`ComputedTextBlock` 脏标记 |
| `dev/bevy/crates/bevy_text/src/font_atlas_set.rs` | `FontAtlasSet` 按 key 复用、miss 才栅格 |

**Rust/wgpu 落地**:rayon `par_iter`(per-paragraph 并行 shape/raster);`crossbeam` 通道(worker→上传);复用 `asset/pipeline/worker_pool`。cosmic-text 的 `FontSystem` 非 `Sync`,需 per-worker 实例或 `Mutex` 包裹——隔离层处理。

## 4. 目标架构

```
帧 N: 收集脏文本节点(内容/样式/scale 变)→
  并行 shape(rayon, per-paragraph;cache 命中跳过)→ ShapedGlyphRun(Arc, 缓存)→
  并行 raster 缺失 glyph(worker;SDF/MSDF CPU 密集)→ GlyphBitmap →
  渲染线程: shelf alloc + 合并脏矩形 + 每页≤1 次上传 → atlas →
  measure 闭包(taffy): 命中 measure/shaped cache,绝不重复 shape
帧末: 三级缓存 LRU trim(未引用项),记录 perf 计数
```

缓存键全部不持引用(`render/14` 已定 `ShapedTextCacheKey`)。同帧去重:measure 阶段与 layout 阶段共享 shaped cache,同一 (text,style) 只 shape 一次。

**缓存两级裁决(2026-07-02 评审收口,D6)**:shaping 结果与 wrap 无关(`02` 交付无宽度约束整形),`ShapedRunCache` 键**不含 wrap**——measure 以 ∞ 宽试测、layout 用实宽时命中同一 shaped run,这是"只 shape 一次"不变量成立的前提。断行/对齐结果另立 `LayoutCache`(键=shaped key + wrap + align/overflow/max_lines)缓存,见下方缓存契约表。命中后必须对缓存值内存文本副本(`Arc<str>`)做等值比较防 hash 碰撞,禁止裸 hash 直取。

## 5. 里程碑

### PF-M1 三级缓存契约定稿 + 同帧去重

实施切片:
1. `text/cache/`:`ShapedRunCache`(`02`)、`MeasureCache`(`03`,承接 `measure_cache.rs`)、atlas cache(`04` 内)——统一键/容量/帧戳/LRU。
2. 同帧去重:measure 闭包与 full layout 走同一 `ShapedRunCache`;命中计数。

测试:`render_perf_text_measure_then_layout_shapes_once`、`text_cache_lru_trims_unreferenced_at_frame_end`。

当前完成口径(2026-07-08):`UiTextMeasureCache` 已把 generic `TextMeasureCache`、`TextLayoutCache`、`TextFrameDedup` 与 `ShapedRunCache` 接到生产 UI measurement / full layout 请求路径；同一帧内 exact key + exact text 的重复 natural-size/full-layout 请求先命中 frame dedup,measurement miss 与 full-layout miss 则通过 `TextShapeRunProvider` 共享同一个 `ShapedRunCache`。`text/layout` 的 line metrics、line break、ellipsis、grapheme advances 与 source-range measure 已有 provider path,且普通非 Tab 文本不再为了 tab alignment 额外 shape `" "`。`render_perf_text_measure_then_layout_shapes_once` 在预热稳定 `"Hg"` line metrics 后断言 `editor base.zui` 的 measure+layout 只插入/ miss 一个真实 source shaped run,避免为解决小字号左右落点问题而改用真实字符串行高；`render_perf_text_scroll_list_reuses_cache` 已覆盖滚动列表首段:首屏 5 行各 shape 一次,滚动 3 行后只为 3 个新进入视口的 row 增加 shaped-run miss/insert,重叠 row 必须命中 shaped cache。2026-07-08 `text/parallel/shape_pool.rs` 新增 owned paragraph batch + `TaskPool` 并行 shape 数据面,先查 `ShapedRunCache`,再只把未命中的唯一 paragraph 交给 worker chunk；2026-08-24 收敛为预热专用路径，仅把 `Ready` 写入 cache，`Deferred`/`Failed` 只保留在 batch report，绝不返回或分发空 `ShapedGlyphRun`。同日 follow-up 用显式 `Vec<PendingShapeJob>` pending queue 类型解除验证时暴露的类型推断编译阻塞,`render_perf_text_parallel_shape_count` focused Cargo 已通过 1/1。随后 `UiTextShapePrewarmRequest` 与 `UiTextMeasureCache::prewarm_horizontal_paragraphs(...)` 把这一数据面接到 UI cache owner:可见 editor row 可在布局前按 batch 预热到同一个 `ShapedRunCache`,后续 full layout 不再为这些行重复 shape,且 absolute layout 仍按 frame miss。surface render owner-text 自动收集/调度已接入 `ui/surface/render/text_prewarm.rs`;组件 painter 生成的 `Text` command 现在也在 command generation 后统一 prewarm,并在返回 `UiRenderExtract` 前补齐 `text_layout`,避免 retained-host 继续走裸文本 fallback。rich/vertical 文本预热已通过 `from_layout_source(...)` 关闭。2026-07-10 追加 retained framebuffer proof PNG 像素指标复查,确认 full/narrow label 的 ink coverage 与内部空列稳定,但实时 editor-window typography QA 因当前 active cargo/rustc/link 队列 12-15 暂未启动；同日 `ScreenSpaceUiTextPrepareReport.raster_upload` 已接入 native raster/upload prepare-report 计数 surface。2026-08-03 已把 persistent-slot 的滚动 source-cache、atlas-slot hit/miss/insert 与 upload copy/byte 计数投影为该 report，并由底层滚动窗口与 report 映射回归共同锁定；实际 editor-window typography QA 与完整 glyphon `TextAtlas` cutover 仍 open。

2026-07-10 补记:`scene_renderer/ui/text.rs` 新增 `ScreenSpaceUiTextRasterUploadReport`,由 `text_prepare_report(...)` 汇总 `NativeBitmapAtlasPrepareReport` 与 `GlyphAtlasBitmapRendererPrepareReport` 的 source/cache/worker/upload/requeue/failure 计数。2026-08-03 继续把 persistent-slot 的 atlas slot hit/miss/insert 与精确 copy-byte 计数投影到同一 report；`render_perf_text_scroll_list_reuses_raster_slots_and_uploads_only_entering_rows` 锁定滚动后仅新行有 copy work，`text_prepare_report_exposes_raster_upload_scroll_counters` 锁定 UI report 不丢失这组增量。这是 PF-M4 的确定性源码计数闭合，不改变 renderer 行为；实际工作台标定与受管 Cargo/WGPU 仍待后续验证。

2026-07-08 补记:`UiTextStyleKey` 已把 `UiTextWritingMode` 纳入 full-layout cache 与 same-frame dedup key。此前 HorizontalTb 与 VerticalRl 在相同 text/frame/style 其它字段下可能共享 `UiTextMeasureKey`,导致竖排 layout 误用水平行缓存。`resolved_layout.rs::style_key_encodes_text_writing_mode` 与 `ui/tests/text_pipeline/measure_cache.rs::text_measure_cache_separates_layouts_by_writing_mode` 覆盖 persistent layout miss 与 same-frame dedup miss。该切片只修 key 维度,不增加 UI/root text facade、字体 token、letter-spacing 或 renderer shortcut。

2026-07-07 补记:`GlyphAtlasBitmapTextureUploadRequestPlan` 已把 page-generation stale/missing 与 face invalidation 从"跳过上传"提升为显式 `GlyphAtlasBitmapRequeuedUpload` 报告。`glyph_atlas_bitmap_texture_upload_request_plan_with_atlas_and_face_validity(...)` 只在 live atlas generation 匹配且 face 仍有效时输出 texture upload request；否则记录 `PageGenerationMismatch`、`MissingPage` 或 `FaceInvalidated`,为 PF-M3 async raster/upload 的 miss/requeue 路径补上可观测数据面。真实 worker、per-face artifact validity source、global glyph slot invalidation 和完整 `TextAtlas` cutover 仍 open。

2026-07-07 补记:`scene_renderer/ui/atlas_texture_upload/frame.rs` 已消费上面的低层 requeue plan,把 `requeued_upload_count`、`missing_page_requeue_count`、`page_generation_mismatch_requeue_count` 与 `face_invalidated_count` 投影进 `GlyphAtlasBitmapTextureUploadFrameReport`。新增 `glyph_atlas_bitmap_texture_upload_frame_plan_for_atlas_and_face_validity(...)`,当 missing page、page generation mismatch 或 face invalidated 出现时帧计划保持 fail-closed,不进入 WGPU texture write。真实 worker、per-face artifact validity source、global glyph slot invalidation、focused Cargo green、live editor-window typography QA 和完整 `TextAtlas` cutover 仍 open。

2026-07-07 补记:`scene_renderer/ui/atlas_renderer/renderer.rs` 已把 renderer prepare telemetry 接上 requeue frame report。生产 bitmap atlas renderer 现在从 `GlyphAtlasBitmapRenderSubmissionPlan.run.atlas` 读取 live atlas generation,调用 `glyph_atlas_bitmap_texture_upload_frame_plan_for_atlas(...)` 后再写 texture；`GlyphAtlasBitmapRendererPrepareReport` 汇总 requeued/missing-page/page-generation-mismatch/face-invalidated upload counters,`upload_failure_count` 也把 requeued uploads 计入失败口径。per-face artifact validity source、真实 async worker、global glyph slot invalidation、focused Cargo green、live editor-window typography QA 和完整 `TextAtlas` cutover 仍 open。

2026-07-07 补记:`GlyphAtlasBitmapRunPlan` 已新增 slot invalidation report,且 bitmap run/render submission/retry driver 可以消费上一帧 `GlyphAtlasSet`。生产 `ScreenSpaceUiTextBackend` 现在跨非空 native bitmap frame 保留主 submission atlas,并在 face invalidation 或 idle native frame 清空；未引用 page 被重建时记录 `GlyphAtlasBitmapSlotInvalidation`、递增 page generation 并整页标脏。该切片关闭主 native bitmap atlas 路径的 slot invalidation state 首段,让 stale upload requeue guard 有真实跨帧 atlas state 可比较；mixed-storage persistent atlas 后续状态见下一条补记,真实 async worker、完整 glyph slot owner、focused Cargo green、live editor-window typography QA 和完整 `TextAtlas` cutover 仍 open。

2026-07-07 补记:`text/native_bitmap_atlas/storage.rs` 已接管 mixed-storage partition/submission owner,且 `NativeBitmapAtlasFrame::storage_submissions()` 将主 frame `self.submission.run.atlas.clone()` 传入每个 storage submission。这样 mixed R8/RGBA frame 的 per-storage render submission 不再从 default `GlyphAtlasSet` 重建,而是继承 persistent frame atlas 与 page generation,关闭 mixed-storage persistent atlas 的 default-atlas reset 缺口。true async raster worker、完整 glyph slot owner/reuse、focused Cargo green、live editor-window typography QA 和完整 `TextAtlas` cutover 仍 open。

2026-07-07 补记:`text/native_bitmap_atlas.rs` 现在在 `source_cache.image(...)` 返回 `None` 时累计 `missing_raster_image_count`,并把该计数写入 `NativeBitmapAtlasPrepareReport`。`native_bitmap_atlas/handoff.rs` 新增 `MissingRasterImage` fallback reason,且只要该计数非 0,native bitmap atlas 即使 source/visible 计数看起来匹配也不能替代 glyphon。该切片关闭 PF-M3 首帧缺失 raster 图像时静默跳过但仍接管 glyphon 的风险,属于 fail-closed 降级前置项；true async raster worker、占位/近似桶首帧降级、完整 glyph slot owner/reuse、live editor-window typography QA 和完整 `TextAtlas` cutover 仍 open。

2026-07-07 补记:`text/parallel/raster_pool.rs` 现在落地真实 swash CPU glyph raster worker queue。`TextRasterWorkerPool` 按显式 worker count 或 `TaskPoolOptions` 的 async-compute budget 创建 `zircon-text-raster-*` worker；每个 worker 持有独立 `SwashRasterizer`,只消费 `Arc<[u8]>` 字体数据与 `SwashRasterRequest`,输出 `GlyphBitmap` completion。提交端维护 in-flight work id 去重、可选有界队列 backpressure 与诊断计数。2026-07-17 owner hard cut 把 completion 失效条件收敛为 `face_epoch`：raster source 在 page allocation 前生成，atlas page churn 不得使可复用 bitmap 失效；page generation 仍在后续 staging/upload boundary fail-closed。per-page upload merge、scroll raster/upload perf counters、live editor-window typography QA 和完整 `TextAtlas` cutover 仍 open。

2026-07-07 补记:`TextRasterCompletionDrain` 携带 face-invalidated work id,让主线程 owner 能清理被拒收 completion 的 pending 映射。`NativeBitmapAtlasSourceCache` 新增 `register_worker_request(...)` 与 `apply_worker_completion_drain(...)`,accepted `GlyphBitmap` 会转换为 source cache 可复用的 `SwashContent`/bearing/size/bytes；failed、unknown、invalid bitmap、face-invalidated 与 pending worker 数进入 source-cache frame report。2026-07-17 删除 stale-page work id/count，因为 raster source 不拥有 atlas page；历史 direct lib-test 只证明旧边界，新的 owner hard cut 仍待 current-source focused Cargo。本切片为非视觉数据面,不生成 PNG。

2026-07-07 补记为历史 worker 路径。2026-08-24 native input hard cut 已使 `scene_renderer/ui/text.rs`、`text/native_bitmap_atlas.rs` 与 `native_bitmap_atlas/source_cache.rs` 只处理由 canonical shaped glyph 投影的 `GlyphRasterKey`。每帧按 face epoch drain completion；cache miss 使用当前 `FontDatabase` face index、shared font bytes 和 variation coordinates 构造 `SwashRasterRequest::native_bitmap_atlas_glyph(...)`，并以 `GlyphRasterKey` pending map 去重。当前 hard cut 仍待受管 Cargo、WGPU 和性能采样，且不生成占位截图。

2026-07-08 的 glyphon `CacheKey.x_bin` 归一化为历史行为。当前 phase policy 由 `GlyphRasterKey::{subpixel_bin, vertical_subpixel_bin}` 明确表达；worker registration、cache lookup、approximate lookup、pending check 与 insert 统一使用这一 identity。纵向近似保持最多三个直接 HashMap probe，不做线性扫描。历史 Cargo 记录不构成当前 hard-cut 验收。

2026-07-07 补记:`text/atlas/render_submission/plan.rs`、`text/native_bitmap_atlas.rs`、`native_bitmap_atlas/handoff.rs` 与 `scene_renderer/ui/text.rs` 已把 worker-pending miss 从 glyphon fallback 改为透明占位。native frame 遇到已经 pending 或刚提交的 worker glyph 时追加 `GlyphAtlasBitmapPlaceholderMode::TransparentQuad`,handoff owner 返回 `TransparentPlaceholder`,root backend 走 native bitmap atlas renderer prepare 并关闭 glyphon。这满足 PF-M3 “缺 glyph 首帧降级占位、不阻塞、不 panic”的当前切片,避免首帧 render path 抖动；真实 glyph 仍由后续 worker completion/source cache 回流。Focused runtime tests 三条均通过 1/1,日志 SHA256 `2CD98E6D27157649062B106EF0D77AB3C8625D9009726BD8959F77118B7EB262` / `597021BF1AB905AEC5733FAA2E77242CC52CFCE358B50239B021D206C99E90B1` / `E1E2F44433689A234F5701D8B05AEFFA5D558D7D64CB2D467896A89643EE42BD`;本切片为非视觉数据面,不生成 PNG。per-page upload merge、scroll raster/upload perf counters、live editor-window typography QA 与完整 `TextAtlas` cutover 仍 open。

2026-07-07 补记:`text/native_bitmap_atlas/source_cache.rs`、`native_bitmap_atlas.rs` 与 `native_bitmap_atlas/handoff.rs` 已落地 PF-M3 的已有近似桶首帧替代。source cache 在 exact `CacheKey` miss 后可保守查找同 font/glyph/size/weight/flags 且仅 subpixel bin 不同的缓存图像,并记录 `approximate_hit_count`;native frame 一边继续提交 exact worker request,一边用近似 source image 生成当前帧 atlas submission,不再透明占位;prepare report 暴露 `approximate_raster_image_count`,first-frame degradation 投影为 `ApproximateBucketReplacement`。静态验证通过 rustfmt/diff/字段覆盖扫描,日志 SHA256 `ADDE952DA2030E7AB8246E555A370814A31CA0E4454C181C27798D7B64FE826C`;Windows focused Cargo 与 WSL `/tmp` target 均在编译阶段超时,日志 SHA256 `6E3EA51A1E2FAA3764DA86EC0295B0A6514C7068E20DAD6D7BBC7355215CAB61` / `CDBF68EA4358BD9212588FF74A379FBE660B0DBFDCE1BA46AF2C5E7C79CCA447`,不声明 focused green。本切片为非视觉数据面,不生成 PNG。per-page upload merge、scroll raster/upload perf counters、live editor-window typography QA 与完整 `TextAtlas` cutover 仍 open。

2026-07-07 补记:`zircon_runtime/src/ui/tests/text_pipeline` 已新增 PF-M4 首个滚动列表 cache 复用 guard `render_perf_text_scroll_list_reuses_cache`。测试先预热稳定 `"Hg"` metrics run,再渲染 5 行编辑器式资源 label 首屏；滚动 3 行后,absolute layout 因 row y 变化仍应 miss,但 `ShapedRunCache` 只允许 3 个新 row 产生 miss/insert,2 个重叠 row 必须命中。该切片锁住滚动列表 shape/layout 增量有界,不改 UI 字体、letter-spacing、ZUI token、root painter、atlas handoff 或 raster worker。raster/upload 字节与命中率计数还没有在 UI scroll test 中暴露,所以 PF-M4 raster 部分仍 open。静态验证 `runtime_text_scroll_cache_reuse_perf_rustfmt_check_20260707.log` SHA256 `E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855`,diff-check SHA256 `22FD7413CC13074CC1E6687BCD6B088A4C9EA15A00B765AC4A5F1739E0495A05`; final scoped diff-check SHA256 `0C2C4EF8AAB3E3943C4674E009561D3F5E8FAB3429CC0B08C123E71ADF59EBC3`,target/cargo-target PNG scan 0 SHA256 `E99A2829ECABA7855E71B61F879991A86D2DF3070B1CD2A23AD2D4242C975B7D`;active cargo/rustc lanes SHA256 `FF8B1878275A1D2AA7843370F3664834CF45966B479F68615D6BA4A18FF93D2D`,因此未启动新的 Cargo。本切片为非视觉 perf counter guard,不生成 PNG。

2026-07-17 补记：PF-M4 的 per-page upload merge 不是缺失的 renderer 行为。`GlyphAtlasDirtyPage` 已按 page union glyph rect，`bitmap_upload_commands(...)` 每页只产一个 command，staging 与 texture request 延续这一基数。新增精确 guard `render_perf_text_async_upload_merges_per_page` 用同页两个 alpha glyph 锁定 `dirty page/upload command/staging page/staged upload/texture request = 1/1/1/1/1`，同时断言合并 extent。实现与静态门已完成，focused Cargo 仍为 `validation_pending`；scroll 场景 raster/upload bytes 增量与命中率计数继续 open。

2026-07-19 补记：PF-M3 的“帧新栅格 glyph <=256”预算已接到最低 `NativeBitmapAtlasSourceCache::request_worker_image(...)` owner，而不是限制 atlas 的全部 visible source。每帧只允许 256 个成功提交的唯一 worker raster request；cache hit、既有 pending 与 worker unavailable 不消耗该成功预算，第 257 个及后续请求返回显式 `DeferredByFrameBudget`，当前帧继续走透明 placeholder，并在下一可见帧重新请求。`worker_request_deferred_count` 已进入 source-cache frame report 与 `ScreenSpaceUiTextRasterUploadReport`，连续两帧 guard 覆盖首帧 `256 submitted + 1 deferred` 以及次帧 deferred key 继续提交；稳定 atlas/cache-hit glyph 不被限流。Rust 1.94.1 rustfmt、scoped diff-check 与静态 owner/telemetry guards 已通过，fresh managed focused Cargo 仍为 `validation_pending`；真实 scroll raster/upload bytes 增量、2 MiB 图集上传字节预算和 live editor-window typography QA 继续 open。

2026-07-19 补记：PF-M4 的 2 MiB 图集上传字节初始标定已新增确定性 contract code。`render_perf_text_typical_256_glyph_frame_stays_within_upload_budget` 使用 192 个 alpha glyph 与 64 个 RGBA glyph 模拟典型 64px/256-glyph 首帧，在四个 512x512 page 上断言每页每帧一次 full-page upload，总上传字节精确为 1,835,008 bytes（1.75 MiB），低于 2 MiB 初始预算。该测试是典型工作台的 synthetic proxy / 确定性基线，不把 2 MiB 误实现为 WGPU writer 末端静默丢写，也不改变现有 256-glyph 回压行为；Asset Browser + Console 万行 + Inspector + `text_corpus` 的真实工作台标定、scoped rustfmt/diff-check 与 fresh managed focused Cargo 仍待本里程碑 testing stage，因此状态为 `contract_implemented / managed_validation_pending`，真实 scroll raster/upload 增量与 live editor-window typography QA 继续 open。

### PF-M2 并行 shaping + 并行栅格

实施切片:
1. per-paragraph 并行 shape(rayon;cosmic-text `FontSystem` per-worker 或 Mutex 隔离);Arc 结果入缓存。
2. 并行 glyph 栅格(SDF/MSDF 生成下 worker);渲染线程收集 + 上传。

测试:`render_perf_text_parallel_shape_count`(N 段并行,shape 次数=未命中段数)、`text_parallel_raster_deterministic`(并行结果与串行一致)。

2026-07-08 补记:`text/parallel/shape_pool.rs` 已落地 PF-M2 的 paragraph shaping 数据面。`TextShapeParagraph` 拥有文本、resolved style、range、direction/orientation 与 kerning 输入;`shape_paragraphs_with_cache(...)` 先按 `ShapedRunCacheKey + exact text` 查 shared `ShapedRunCache`,再把未命中的唯一 paragraph 交给 `TaskPool`/`parallel_for` chunk 执行；2026-08-24 该 API 已硬切为预热专用报告，只有 `Ready` 可写入 `ShapedRunCache`，`Deferred`/`Failed` 不再被折叠为空运行或作为批次结果分发。同批重复 miss 只 shape 一次,第二批相同请求命中 shaped cache。后续 editor property/axis 截图验证暴露 `pending` 队列类型推断失败后,本 owner 仅补 `Vec<PendingShapeJob>` 显式类型并复跑 `render_perf_text_parallel_shape_count` 1/1 passed。随后 `UiTextMeasureCache::prewarm_horizontal_paragraphs(...)` 用 `UiTextShapePrewarmRequest` 将可见 UI 段落批量预热到同一个 shaped-run cache,预热请求使用与 measure/layout 一致的 `UiTextDirection::Auto` 和完整 source range,避免预热 key 与后续布局 key 分叉。该 UI cache 入口已通过 focused Cargo,后续 surface render owner-text 自动 collection 已由下一条关闭；live editor-window typography QA、scroll raster/upload counters、per-page upload merge 与 full glyphon `TextAtlas` cutover 仍 open;本切片不生成视觉 PNG。

2026-07-08 补记:`ui/surface/render/text_prewarm.rs` 已把 PF-M2 shape pool 接到 `UiSurface` render extract owner-text 自动收集路径。`prewarm_visible_owner_text(...)` 在生成 render commands 前遍历当前 arranged draw order,按组件 painter suppress 规则收集可见 owner text,再通过 render-local `TaskPool` 调用 `UiTextMeasureCache::prewarm_horizontal_paragraphs(...)`。`UiTextMeasureCache::frame_shape_prewarm_report()` 暴露本帧 prewarm telemetry,`render_extract_automatically_prewarms_visible_owner_text_before_layout` 覆盖 visible/hidden/duplicate editor labels:extract 自动 requested 3、miss/insert 2、batch duplicate 1,后续 layout 对这些 labels 命中 shaped cache。该切片关闭 surface render owner-text 自动 paragraph collection/scheduling;组件 painter 生成的 Text command 已由下一条补记关闭。rich text 与 vertical writing mode 预热已由后续 `from_layout_source(...)` 补记关闭；scroll raster/upload counters、live editor-window typography QA、per-page upload merge 与 full glyphon `TextAtlas` cutover 仍 open。本切片非视觉,不生成 PNG。

2026-07-08 补记:`ui/surface/render/text_prewarm.rs` 同日继续关闭组件 painter 生成 `Text` command 的预热/布局缺口。`prewarm_render_command_text(...)` 在所有 render commands 生成后收集仍缺 `text_layout` 的可见命令,通过同一个 render-local shape pool 预热 shared `ShapedRunCache`;`resolve_missing_render_command_text_layouts(...)` 随后用 `UiTextMeasureCache::resolve_or_shape(...)` 补齐这些命令的 resolved layout。`UiTextMeasureCache::frame_shape_prewarm_report()` 改为同帧累计,因此 owner-text prewarm 与 command-text prewarm 不会互相覆盖 telemetry。`render_extract_prewarms_and_layouts_component_text_commands` 覆盖 Button 组件生成的 `editor base.zui` / `folder-open-outline.svg` 文本:visible text commands=3,prewarm requested=3,miss/insert=2,batch duplicate=1,且所有组件 Text command 在 retained-host fallback 前已有 `text_layout`。本切片非视觉,不生成 PNG;rich/vertical 文本预热已由下一条关闭；scroll raster/upload counters、live editor-window typography QA、per-page upload merge 与 full glyphon `TextAtlas` cutover 仍 open。

2026-07-08 补记:`UiTextShapePrewarmRequest::from_layout_source(...)` 已关闭 rich/vertical 预热缺口。rich text 先通过 `parse_source_runs(text, true)` 投影成 layout 可见文本,避免 `**editor base.zui**` 这类 markup 字节进入 shaped-run key；vertical writing mode 保留原 `UiResolvedStyle`,由写入 `UiTextStyleKey` 的 `UiTextWritingMode` 区分缓存。`prewarm_visible_owner_text(...)` 与 `prewarm_render_command_text(...)` 现在都走该入口,`prewarm_render_command_text_accepts_rich_and_vertical_commands` 与 `render_extract_prewarms_rich_and_vertical_owner_text_before_layout` focused Cargo 2/2 passed,日志 `docs/tests/runtime/text/runtime_text_rich_vertical_prewarm_focused_cargo_20260708.log`。本切片为非视觉 cache/prewarm 数据面,不生成 PNG;scroll raster/upload counters、per-page upload merge、live editor-window typography QA 与 full glyphon `TextAtlas` cutover 仍 open。

### PF-M3 异步上传 + 首帧降级

实施切片:
1. glyph 栅格异步,GPU 上传渲染线程合并;栅格未完成的 glyph 首帧降级(占位/已有近似桶)避免阻塞。
2. 帧预算:超预算时限制本帧新栅格 glyph 数,余下下帧补(滚动大列表不卡)。

测试:`render_perf_text_async_upload_merges_per_page`、`text_first_frame_missing_glyph_degrades_not_blocks`。

### PF-M4 性能计数进测试(接 render/17)

实施切片:
1. `render_perf_text_*`:shape 次数、raster 次数、上传字节、缓存命中率确定性断言(时间类只观测);接 `render/17` PF 观测底座。

测试:`render_perf_text_scroll_list_reuses_cache`(滚动复用;shape/layout 首段增量有界,raster/upload counter 待接入)。

### PF-M5 超长文本分段与虚拟化(2026-07-02 评审收口新增)

并行与缓存只解决"多而小"的文本;百 KB 级 Console log / 长文档是"单个巨大"文本,没有增量策略必掉帧。实施切片:

1. **段落级脏跟踪**:文本按段落(hard break)切分为独立 shaping 单元;编辑/追加只重 shape 受影响段,其余段命中缓存。
2. **可视区惰性布局**:仅 shape/layout 可视区 ±N 段(N 默认 2 屏);视口外行高用估算值(已 shape 段用真实值),滚动进入时精化并修正滚动偏移。
3. **单 run 执行预算与保护**:`TextShapingWorkBudget`以默认 64 KiB 区分 inline/oversized-synchronous 工作，生产 retained session cache miss 与 parallel unique pending job 已发布 request count、总输入字节与最大请求字节；该阈值不得成为 source line、script run 或 cluster 边界。typed defer/cancel 尚未接入前，超限请求必须保留完整语义并同步完成；后续 scheduler 只能调度完整 work unit，不能靠伪造 layout line 规避大请求。

测试:`render_perf_text_huge_log_shapes_visible_only`(万行 log 首帧只 shape 可视区)、`text_paragraph_dirty_reshapes_edited_only`(编辑单段只重 shape 该段)、`text_oversized_run_keeps_one_logical_shaped_line`与`text_semantic_context_preserves_a_ligature_crossing_the_work_boundary`。

## 6. 工程落地细化(实施权威)

### 模块与文件落点

实现层 `zircon_runtime/src/text/cache/`:

| 文件 | 内容 |
|------|------|
| `mod.rs` | 缓存装配(薄) |
| `shaped_cache.rs` | `ShapedRunCache`(`ShapedTextCacheKey → Arc<ShapedGlyphRun>`,LRU + 帧戳;`render/14` 键,**无 wrap**,见缓存契约表) |
| `layout_cache.rs` | `LayoutCache`(断行/对齐结果缓存,键含 wrap;2026-07-02 评审收口新增) |
| `measure_cache.rs` | measure 结果缓存(承接 `ui/text/measure_cache.rs`;桶化限制见缓存契约) |
| `frame_dedup.rs` | 同帧去重(measure↔layout 共享);命中计数 |

并行:`text/parallel/`:

| 文件 | 内容 |
|------|------|
| `shape_pool.rs` | per-paragraph 并行 shape(rayon;`FontSystem` per-worker 隔离) |
| `raster_pool.rs` | 并行 glyph 栅格 → 上传队列(crossbeam 通道) |

### 缓存契约(统一)

| 缓存 | 键 | 容量/逐出 | 帧戳 | 失效来源(2026-07-02 评审收口) |
|------|----|-----------|------|------|
| ShapedRunCache | `font_id + size_bits + features_hash + language + text_hash`(**无 wrap**,不持引用;命中后等值比较文本副本防碰撞) | 1024 run / 8 MiB LRU | 本帧引用打戳 | 字体失效(`01` 失效级联,按 font_id 剔除)、显式 flush |
| LayoutCache(2026-07-02 评审收口新增) | shaped key + 精确 wrap 宽度**或**结果有效宽度区间 `[min,max)` + align/overflow/max_lines | 2048 项 LRU | 同上 | ShapedRunCache 条目失效连带剔除 |
| MeasureCache | shaped key + wrap_width(桶化限制见下) | 4096 项 LRU | 同上 | 同 LayoutCache |
| GlyphAtlas(04) | `GlyphRasterKey` | 页级 LRU,8 页/格式 | 本帧引用页不可逐出 | 字体失效按 face 剔除 slot、页 LRU 逐出(携 page_generation) |

**帧戳语义修正(2026-07-02 评审收口)**:容量超限时按 LRU 逐出**最久未引用**项;帧末**仅更新帧戳,不主动清空未引用项**——否则离屏/折叠面板文本缓存每帧清零,LRU 容量上限形同虚设。trim 触发条件=容量超限 / 显式 flush / 字体失效级联(`01`),三者之外不清缓存。`text_cache_lru_trims_unreferenced_at_frame_end` 的断言口径同步改为"容量超限时最久未引用项先被逐出"。

**native bitmap atlas 空帧 flush(2026-07-05，2026-08-24 硬切换同步)**:PF-M1/AT-M3 的 swash source-image cache 采用持久跨帧复用 + 容量 LRU,但 native text 输入为空代表当前 renderer 已无 native glyph 需求。本轮把该状态定义为显式 flush:`ScreenSpaceUiTextBackend` 在 `texts.is_empty()` 分支调用 `native_bitmap_atlas_idle_prepare_report(...)`,清空 `NativeBitmapAtlasSourceCache` 并把 `evicted_count/entry_count` 写入 `NativeBitmapAtlasPrepareReport.source_cache`。native atlas 不再保有或清理 glyphon `TextAtlas`。这不改变普通非空帧的 LRU 语义,也不引入每帧清空未引用项。

**native bitmap atlas 帧戳输入修正(2026-07-05)**:PF-M1/AT-M3 的 atlas retry、page residency 与后续 async raster 产物失效都依赖真实 frame index。旧生产 native path 用固定 `BITMAP_ATLAS_FRAME_INDEX = 1`,会让 blocked retry 永远报告 next frame 2,也让后续 frame-loop telemetry 无法区分连续帧。现在 `ScreenSpaceUiTextBackend` 对非空 native atlas frame 递增 `bitmap_atlas_frame_index`,并把该值传入 `native_bitmap_atlas_frame(...)`、prepare report 与 per-storage submission。该修正不等同于真实 retry-frame state execution 或全局 glyph cache slot invalidation,但关闭了它们接线前的固定帧号阻塞。

**native bitmap atlas retry face-invalidation report(2026-07-05)**:PF-M1/AT-M3 的 retry-frame state 现在把 face invalidation 导致的 blocked retry glyph 清理作为显式 telemetry 暴露。`GlyphAtlasBitmapRetryFrameState::discard_all_for_face_invalidation()` 清空 blocked queue 并累计 `pending_invalidated_blocked_glyph_count`;`apply_submission_plan(...)` 与 `native_bitmap_atlas_idle_prepare_report(...)` 通过 `take_report()` 在当前 visible frame 或 idle frame 写出 `NativeBitmapAtlasPrepareReport.retry_state.invalidated_blocked_glyph_count` 后清零。这样字体 face 变化不会把待重试 glyph 静默丢弃,也不会让空 native text frame 把 invalidation 计数滞留到下一次非空帧。该切片仍不等同于异步产物 face-validity requeue 或全局 glyph cache slot invalidation。

**native bitmap atlas renderer face-invalidation storage-pass telemetry(2026-07-05)**:PF-M1/AT-M3 继续把 face invalidation 推到 renderer-local storage pass 状态。`GlyphAtlasBitmapRenderer::discard_all_for_face_invalidation()` 清空 active storage passes 并累计 `invalidated_storage_pass_count`,下一次 prepare report 暴露被清理的 storage-pass 数量；`ScreenSpaceUiTextBackend` 在同一 face invalidation 分支同步清理 source cache、retry queue 与 renderer storage passes。这避免 source/raster cache 已失效但 renderer 仍保留旧 face atlas draw state 的诊断盲区。

**native bitmap atlas nearest sampler(2026-07-05)**:AT-M2/AT-M3 针对最新 editor crop 中“等线已生效但小字号左右边缘仍不稳”的 GPU sampling 风险,将 runtime bitmap atlas sampler 对齐 glyphon nearest sampling。`atlas_renderer/resources.rs` 的 sampler 使用 nearest min/mag/mipmap 且 LOD clamp 为 0,避免线性过滤把相邻 atlas texel 混入紧凑文件名标签的左右边缘。2026-07-06 focused Cargo `glyph_atlas_bitmap_sampler_matches_glyphon_nearest_sampling_contract` 已通过 1/1,日志 `docs/tests/runtime/text/runtime_text_bitmap_atlas_nearest_sampler_focused_cargo_20260706.log` SHA256 `AD2A6E83F4D73F08C1A53404740E1148353D791F0F15AE9316B783FAE4BE5692`。该切片是采样层防线,不替代 retained-host live crop QA、full glyphon `TextAtlas` cutover 或 LCD/gamma/background policy。

**native bitmap atlas handoff owner(2026-07-05，2026-08-24 硬切换同步)**:AT-M3/PF-M1 将 native submission readiness 判定从 `scene_renderer/ui/text.rs` 根实现移入 `text/native_bitmap_atlas/handoff.rs`。`NativeBitmapAtlasHandoff` 和 `native_bitmap_atlas_handoff_for_report(...)` 与 `NativeBitmapAtlasPrepareReport` 同属 native bitmap atlas 子域,让 root text backend 只执行 single-storage submission、mixed-storage submission、idle 或透明降级,不再持有切换 policy。native bitmap atlas 没有 glyphon fallback 输入或成功路径。

**native bitmap atlas degradation telemetry(2026-07-05，2026-08-24 硬切换同步)**:AT-M3/PF-M1 继续把 handoff 诊断留在 native atlas 子 owner。`NativeBitmapAtlasPrepareReport.native_degradation_reason` 记录 native 栅格暂不可用的原因,并由 `text/native_bitmap_atlas/handoff.rs` 从 prepare report 状态推导。性能/缓存报告可区分无可见 glyph、unsupported format、source coverage 缺口、LCD background composite 输入缺失、atlas allocation failure 或 mixed storage split 未就绪,不再以旧 renderer 的存在作为诊断语义；`native_submission_ready=false` 只会导致本帧 native 透明降级。

**measure 桶化限制(2026-07-02 评审收口)**:wrap_width 桶化仅允许用于宽度不影响结果的场景(单行 / 无 wrap);有 wrap 时桶化会在断行临界点附近返回错行数/错高度,违反 index §6 #2"度量=绘制、禁止近似"。有 wrap 的 measure 键用精确宽度,或缓存"该断行结果成立的宽度有效域 `[min,max)`"并在区间内命中。

同帧去重不变量:同一 (text, style) 在一帧内 measure + layout 只 shape 一次(wrap 不同只重跑断行、命中同一 shaped run;`render_perf_text_measure_then_layout_shapes_once` 守卫,计数口径=shape 调用次数,与 wrap 宽度无关)。

### 并行隔离

- cosmic-text `FontSystem` 非 `Sync`:每 worker 持独立 `FontSystem`(共享只读 `FontDatabase` 的 `Arc<[u8]>` 字体数据),或主 `FontSystem` 加 `Mutex`(测量阶段争用低)。隔离决策在 `shape_pool.rs`,不外泄。
- 栅格(swash/fdsm)纯函数式输入输出,天然可并行;结果经通道回渲染线程。
- 确定性:并行只影响顺序不影响结果;`text_parallel_raster_deterministic` 守卫并行=串行。
- **代际与失效竞态(2026-07-17 owner 收口)**:CPU raster source 在 atlas page 分配前生成，只按字体 `face_epoch` 失效；它不得携带伪 page generation。allocation/staging/upload request 取得真实 page 后必须携带并校验 `page_generation`，代际不匹配或 face 已失效的上传**丢弃并重排队**，不得写入已易主的页。测试拆为 raster 边界 `text_raster_worker_pool_drain_accepts_atlas_independent_work_and_discards_old_faces` 与既有 upload page-generation mismatch guards。

### 性能预算(接 render/17)

- 帧新栅格 glyph 上限(默认按 atlas 上传带宽);超限下帧补,降级占位。
- scale 量化桶 `QUANT`(`04`):粗桶省重栅格但牺牲精度,细桶反之——默认 1px,可按场景调。
- 计数:`render_perf_text_*` 断言 shape/raster/upload 确定性计数;时间(shape 耗时/帧)只观测不断言(`render/17` 纪律)。
- **初始预算表(2026-07-02 评审收口;数值为初始标定,可按实测修订,但修订必须回写本表)**:

| 指标 | 初始预算 | 断言方式 |
|------|---------|---------|
| 帧新栅格 glyph 数 | ≤256 | 确定性计数(超限排队下帧) |
| 图集上传字节/帧 | ≤2 MiB | 确定性计数 |
| 稳定帧缓存命中率(shaped run) | ≥90% | 计数式(命中/总请求) |
| 滚动一屏 shape 增量 | =新进入可视区的段数 | 确定性计数 |
| shape 耗时 / 千字 | 只观测 | 记录不 gate |

  标定方法:以编辑器典型工作台(Asset Browser + Console 万行 + Inspector)与 `text_corpus` 混排语料跑 `render_perf_text_*`,取首轮实测值上浮 20% 定初值。

### 测试与验收清单

| 测试 | 断言 |
|------|------|
| `render_perf_text_measure_then_layout_shapes_once` | 同 (text,style) measure+layout shape 计数=1(wrap 不同不重 shape) |
| `text_cache_lru_trims_unreferenced_at_frame_end` | 容量超限时最久未引用 run 先被逐出;引用项保留(帧末不主动清空) |
| `render_perf_text_parallel_shape_count` | N 段并行 shape 次数=未命中段数,结果 = 串行 |
| `text_parallel_raster_deterministic` | 并行栅格逐字节=串行 |
| `render_perf_text_async_upload_merges_per_page` | 异步栅格,每页每帧≤1 次上传 |
| `text_first_frame_missing_glyph_degrades_not_blocks` | 缺 glyph 首帧降级占位,不阻塞,不 panic |
| `render_perf_text_scroll_list_reuses_cache` | 滚动列表 shape/layout 首段增量有界:滚动后只为新进入视口 row 增加 shaped miss/insert,重叠 row 命中 shaped cache；persistent-slot 回归锁定仅新行产生 atlas upload copy，UI prepare report 保留 source-cache、slot 与 copy-byte 计数 |
| `text_raster_worker_pool_drain_accepts_atlas_independent_work_and_discards_old_faces` | CPU raster source 不因 atlas page churn 失效；旧 `face_epoch` completion 被丢弃。真实 page generation 继续由 allocation/staging/upload mismatch guards 验收 |
| `render_perf_text_huge_log_shapes_visible_only` | 万行 log 首帧只 shape 可视区 ±N 段(2026-07-02 评审收口) |
| `text_paragraph_dirty_reshapes_edited_only` | 编辑单段只重 shape 该段(2026-07-02 评审收口) |

里程碑命令:`cargo test -p zircon_runtime render_perf_text --locked`、`text_cache --locked`、`text_parallel --locked`。

## 7. 风险与回退

- `FontSystem` 非 Sync 致并行受限:测量阶段争用低,可先 Mutex;shaping 重负载段再上 per-worker 实例。
- 异步上传与帧时序:缺字降级须确定性(占位规则固定),否则抓帧不稳;以 `text_first_frame_missing_glyph_degrades_not_blocks` 守恒。

## 8. 状态与产出记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

当前概述（2026-07-18）：Text MVP 的共享 FontDatabase 发布已从“每次调用均推进 generation”收敛为 render-input 语义变化才推进。等价 renderer 初始化保留 shaped-run、locale FontSystem 与 SDF bake resident cache；face 顺序、source、fallback、CompositeFont 或默认 UI family 变化仍执行一次完整 lineage invalidation。比较与替换只发生在低频 publish 写锁路径，普通 generation probe 与 shaping/raster 热路径不增加锁或字体字节扫描。系统字体 `Discover` 对 renderer clone 也已幂等，且 `TextRenderState` 用系统 locale + shared backend DB 直接构造 cosmic FontSystem，避免每次 renderer 构造的两次目录 I/O/backend face 追加；`sys-locale` 仅随 `text` feature 启用。test-only force publish 保留失效测试语义，test-only read guard 只隔离并行 SDF fixture。异步 raster worker 的 request sender 缺失路径也已从 production `expect` 改为 fail-closed `CoreError::ChannelSend`，错误发生在 work id 入队前，保持 `in_flight/queue_peak=0`。同一 worker boundary 已移除固定 `page_generation=0` 伪 target：completion 只按 face epoch 失效，真实 page generation 继续在 atlas allocation/staging/upload guard 中校验，避免 page churn 无效丢弃可复用 raster source。per-page upload merge 已由精确 1/1/1/1/1 基数 guard 覆盖。Text04 persistent slot MVP 又让精确 neutral raster key 跨帧复用 atlas rect，后续稳定帧不重复上传，同帧 mixed-storage 子提交仍保留首次 upload；页逐出/page-size 变化原子失效 slot，裁剪/近似/pixel-font/无稳定 identity 输入 fail-closed 回退旧策略，slot hit/miss/insert 已进入 submission report。旧 shared focused batch 2/2 已绿；新增 locale/default-family/idempotent-discovery guard、并行 SDF 20 项、raster worker 断链/face epoch hard-cut/per-page merge/persistent-slot 回归、default/UI、graphics-only 与真实产品帧仍待里程碑测试阶段，因此本轮新增切片状态为 `implemented / validation_pending`，PF/Text09 整体继续 `in_progress`。

2026-07-29 状态更新：Text09 的 cache O(1) index/LRU、same-frame pending shape fingerprint index、不可变 font-handle batch snapshot 与 canonical `ShapedGlyphRun` 直通已经完成本地实现；raster worker test constructor 的完成队列 byte-budget move-after-move 编译回归已修复，并新增 `0 -> 1` 归一化与实际背压回归。受管精确 cache 门 job `2f42664ec83b4d66a27a9f02671d5653` / run `02c936b32c8149e28eb633ed944e146c` 以 `exit 101` 在测试执行前被共享 Runtime 编译边界阻断；Text09 不将其计为测试通过。已将 Runtime15 UI/late-API structure guard 源路径漂移导入 `15/failure-2026-07-29-structure-guard-include-path-drift.md`，其余动态事件、动态场景 I/O、readback 与 native discovery 错误分别落入既有 Runtime10/Runtime11/Render16 handoff。当前状态为 `implemented / resolving_failure / managed_validation_pending`；真实 WGPU 文本产品帧、截图像素检查、里程碑产出记录与提交均未完成。

2026-07-30 状态更新：Runtime15 source-true F17 已在受管 job
`3d962990f2984ef2a288327ca0412bd0` / run
`3ef0c2c1b44645aaa5055db9d18555fa` 中完成，精确筛选执行 `1` 项并报告
`1 passed; 0 failed`。这只解除共享 lib-test 编译与结构守卫门禁，不替代
Text03 单次完整 shaping、Text09 batch-handle/cache 精确回归或真实 WGPU
framebuffer 产品截图。当前 CPU 槽由 Frameworks04 的受管 Cargo 树占用，已
续期的 Text03 预约仍待 FIFO 消费；在此期间补齐了 trailing-newline 的
measure/layout 高度一致性与 font-handle snapshot generation 二次确认。状态
保持 `implemented / resolving_failure / managed_validation_pending`，没有将
排队或跨计划运行中的任务标记为 `blocked`，也未声称截图、产出记录或提交完成。

2026-07-30 WGPU 验证状态：真实产品 framebuffer 命令已在 GPU job
`5c40beaeeed1466b9f169325a944d545` / run
`a38b36fd38504aafa06392083da6f2db` 中启动，但在渲染前以 `exit 101` 停止。
最低共享编译层为 Plugins01 `runtime_profile/availability_projection.rs:262`：
`RuntimePluginAvailabilitySummary::category_count` 是 `const fn`，却对
`RuntimePluginAvailabilityCategory` 调用了非 const 的派生 `PartialEq`。
随后 Frameworks04 的上游 lib-test job `e83f2aa0784d45cab6526effd572d7a2` /
run `45760a71db784350a710e5b33d138fb4` 以同一低层边界失败，并额外证明
`profile_availability_projection.rs:358` 与 `:362` 将
`RuntimePluginAvailabilityGeneration::entries(...)` 返回的 iterator 当作
slice 调用 `.first()`；它们应由该 API/测试 owner 一并收敛，而非由 Text09
添加上层绕过。
该 owner 已由 active `plugins01-availability-generation-r4-20260730` 和既有
[`runtime-profile-availability-rebuild`](../../zircon_plugins/01/failure-2026-07-17-runtime-profile-availability-rebuild.md)
handoff 覆盖，Text09 不跨租约绕过或复制修复。旧归档 PNG 在测试前的 SHA-256
为 `A96F6D283EBDC43ABBF5A078D319FEDA91B8C2801F0FDF60A1008A7C6EC40A01`；本次
没有通过 framebuffer readback、没有新 PNG、没有截图目检或产出记录。状态仍为
`implemented / resolving_failure / managed_validation_pending`。

2026-07-30 Text03 上游验证状态：availability owner 将 enum count 收敛为 const
`match` 并将 iterator assertions 改为 `.next()` 后，受管 Text03 job
`b2f400fa57644401825f314a28efa81e` / run
`dce6fe65543e4a0f9b5e0d2a7e74b21e` 已越过该层，但仍在测试二进制启动前以
`exit 101` 停止。新的最低共享边界是
`RuntimePluginCatalog` 的 `RefCell` project-plan cache/counter 破坏了
`OnceLock` 与 `RuntimeModuleLifecycleObserver: Sync` 契约，且 sibling consumers
直接读取 `CompiledProjectPluginPlan` 私有报告字段；它们由既有
[`runtime-plugin-catalog-derived-projection-rebuild`](../../zircon_plugins/01/failure-2026-07-17-runtime-plugin-catalog-derived-projection-rebuild.md)
handoff 及其 active catalog owner 处理。availability regression 另有 row identity
assertion 误用 `assert_eq!`（需要未承诺的 `PartialEq + Debug`）；保持 pointer identity
断言即可。Text03/Text09 不添加 trait、公开字段或上层同步绕过；其精确测试、WGPU
framebuffer、截图目检、里程碑记录与提交仍全部待完成。

2026-07-30 shared-support 更新：Text03 current-source job
`4959de0e7c1e4576af54e293fdd1d9f3` / run
`c033e99c0d194c7d8f111279d88bfb99` 已越过 catalog accessors，却在测试二进制启动前
停于 dynamic-session 对 frozen extension `Arc` 的可变应用。active Plugins01 consumer
随后在同一运行窗口发布只读 `WorldRuntimeExtensionPlan`，故该退出只作为 source-raced
上游诊断，不计 Text09 cache/font-handle regression，也不构成 WGPU framebuffer
readback 或 PNG 证据。Text03 fresh reservation
`e97f3d7bf6a4490ea685cde5cba94805` 已续期等待 FIFO；Text09 精确回归和真实 GPU 产品
frame 仍严格排在其 current-source 结果之后。

本子计划产出记录已超过 10 条，具体记录已迁入编号子目录。

- 迁入记录：[`09/2026-07-09-threading-caching-and-performance-output-records.md`](09/2026-07-09-threading-caching-and-performance-output-records.md)

2026-07-30 managed current-source progress: Text03 trailing-newline support regression
passed `1/1` in job `a0818d9b32b24998990447d1df80d4a1` / run
`0f98dc6c1b9043fc966d7aa108f9bc25`; it establishes the shared layout prerequisite
only. Text09 batch-handle/cache exact regressions and the fresh WGPU product framebuffer
are still pending FIFO execution, so no Text09 performance acceptance, screenshot, output
record, or commit is claimed and the recovery status remains `resolving_failure`.

2026-07-30 shared prerequisite update: Text03's single-shape grapheme projection
regression passed `1/1` in job `b4e6f332a13b44f6b66b906487a52c95` / run
`853e35937130424a9d276d69a9c20bd0`. Text09's own batch-handle/cache regressions and
fresh WGPU product framebuffer remain pending and are not claimed by this prerequisite.

2026-07-30 managed current-source progress: Text09's batch font-handle projection and
resolution deduplication regression passed `1/1` in job
`eb62a140fead4bfe9848c4d751f8a0d5` / run
`3b971c5816aa4721a32def40b9aab40d` (cold compile `28m47s`, test `8.10s`). The
source timestamps for `handle_registry.rs`, shared-font publication, and TextService
remained stable at launch. The cache O(1) regression and a fresh WGPU product
framebuffer remain required; no Text09 acceptance, screenshot, output record, or
commit is claimed by this focused pass.

2026-07-30 managed cache validation diagnostic: the exact cache O(1) job
`31fb549441c24066a43536fd7c6758e5` / run
`195c1cde803b4b80954e054539fbbd9c` reached current-source `zircon_runtime`
lib-test compilation but stopped before the test binary with upstream `E0432` in
`graphics/scene/scene_renderer/post_process/resources/construct/construct/construct.rs`:
the `terminal_resource_cache` parent path was absent. No Text09 cache assertion ran, so
this is a shared structure/import diagnostic rather than a cache regression failure.
The plan remains `implemented / resolving_failure / managed_validation_pending`; rerun
the same exact cache command after its owning source path is repaired, then continue to
the real WGPU product framebuffer.

2026-08-14 pre-Cargo static recovery: UI12 current-source diagnostics no longer read the
private SDF renderer through `ScreenSpaceUiTextPrepareReport`; the public crate-local
`sdf_generation` projection supplies the three base-stat counters instead. The production
font-manifest resolver now adapts the runtime asset-path function through a concrete
closure, closing the higher-ranked `FnOnce` inference failure. In the same non-algorithmic
Text09 infrastructure slice, `NativeBitmapAtlasPrepareReport` and the idle-report
constructor moved to `text/native_bitmap_atlas/report.rs`, while
`NativeBitmapAtlasTextArea` 的历史拆分已被 2026-08-24 的 `NativeBitmapAtlasGlyphRun` 硬切替代；
`text_area.rs` 已删除。frame state, source-image details, per-frame budget and frame driver now belong to
`text/native_bitmap_atlas/frame.rs`; `retry_frame` and `storage` import that sibling
owner directly. The atlas root is now declaration/re-export wiring only, and the input
fields remain visible only to its parent module. Scoped `rustfmt --check`, tracked
`git diff --check`, and source-owner assertions passed. The repository-wide structure
guard exceeded its 65-second window without a result, so it is not counted as passed.
UI12 has not released the Cargo lane: no Cargo, timing measurement, WGPU framebuffer,
PNG, milestone output record, commit, or WeCom notification is claimed. Text09 therefore remains
`implemented / resolving_failure / managed_validation_pending`.

PF-M5 current-source contract: `text/hard_line.rs` owns source separator identity and never
publishes an execution-budget boundary as a layout line. `TextShapingWorkBudget` classifies the
default 64 KiB inline threshold without authorizing text slicing. Retained session cache misses and
parallel unique pending jobs now report inline/oversized-synchronous counts, total input bytes, and
maximum request bytes; until a typed deferred outcome exists, horizontal, vertical, rich, and
prewarm callers retain one complete semantic request.
`HardLineIndexCache` separately keys a bounded retained offset index by
`TextDocumentKey(owner, revision)` and bypasses rather than retaining oversized or unkeyed
documents. For retained Plain/HorizontalTb/None/Clip input only,
`ui/text/layout_engine/viewport.rs` materializes the viewport plus explicit overscan while keeping
the full document height, and the persistent cache routes a strict partial window to same-frame
dedup instead of caching viewport-specific geometry. Runtime81 M0 regressions cover the
10,000-line visible-only case, a single edited paragraph yielding two shaped-cache hits and one
miss, one logical line across the work threshold, and a ligature whose cluster crosses that
threshold. This matches the relevant Unreal `FTextLayout` direction: stable source-line models
carry estimated geometry and dirty state, while lazy views are generated only around the
viewport. The wider PF-M5 deferred/cancelled work-unit scheduler, paragraph-height authority, and
scroll-anchor contracts remain open.

2026-08-14 priority-review synchronization: the stale Text locale review in
`engine-code-review-findings-2026-06.md` no longer describes per-run `locl` or variable axes as
an open cosmic-text capability. Current source itemizes once and routes each horizontal run through
the canonical RustyBuzz direct owner with its normalized language; cosmic remains whole-request
fallback. Exact Calibri Russian/Serbian `locl` and variable-width-axis regressions are present in
that owner. The related product proof harness is likewise current-source structural evidence only:
it requires framebuffer pixel changes across native/SDF/VerticalRl/rich cases, rejects both
workspace and configured Cargo `target` paths, and atomically writes the accepted PNG path. No
Cargo, measurement, WGPU execution, new image, acceptance, commit, or WeCom notification is
claimed by this synchronization.

2026-08-14 product-proof fixture isolation: `product_proof_work_path` already combines the
workspace-local label with process/time entropy, but both ignored WGPU product proofs previously
used the same `fixture` namespace. `product_fixture_asset_manager` now accepts an explicit,
validated label; the callers use `multilingual-fixture` and `dpi-fixture`, so even a degraded
clock cannot give the two cases the same fixture directory name. The path-owner regression checks
both prefixes and the shared workspace-local parent. Scoped `rustfmt --check`, `git diff --check`,
and a complete static call-site audit passed. UI12 still owns the validation lane, so no Cargo,
WGPU execution, PNG, timing data, acceptance, milestone record, commit, or WeCom notification is
claimed by this infrastructure completion.

2026-08-14 PF-M4 current-source reconciliation: the early scroll-list unit test deliberately
limits itself to UI measurement/layout cache behavior, but it is not the only performance guard.
The ignored real-WGPU `render_profiling/text_baseline/scroll_turnover.rs` uses a 1,000-row virtual
list with a 100-row window and 10-row turnover. After warmup it records every native raster and
atlas-upload counter for 300 measured frames, requires zero shaped/layout/source-cache/slot-cache
misses, zero worker pressure/placeholders, and zero native upload copies, bytes, requeues, and
failures while retaining positive native instances and draws. The metric plumbing is therefore
implemented; this is not a reason to create another counter path or change a hot algorithm before
the managed measurement run. UI12 still has not released the Cargo lane, so the WGPU baseline has
not been executed here and no timing, power, PNG, acceptance, commit, or WeCom result is claimed.

2026-08-24 P1-13 M2b artifact-publication continuation: both ordinary and secure-presentation
glyph artifact builders now return the canonical `TextLayoutOutcome<Option<ResolvedTextGlyphArtifact>>`.
`Ready(None)` remains limited to a valid DTO-only renderer path (for example a visual-only line
without a stable source projection); provider shaping failures and font-generation changes remain
`Failed` or `Deferred` through the artifact boundary. A 2026-08-26 current-source follow-up now
applies the same rule to source ownership: source/layout ranges that cannot share one owner, lines
outside the layout range, and UTF-8-splitting source slices return `Failed(LayoutFailed)` rather
than masquerading as a visual-only `Ready(None)`. The plain-layout publication owner delegates
the attachment decision to `ui/text/layout_engine/artifact.rs`, so a non-ready artifact result
returns from layout before `UiTextMeasureCache` can admit a `Ready(layout)`. Final command
preparation follows the same distinction for both plain and secure mask artifacts: only
`Ready(Some)` creates a handle, while non-ready outcomes record the typed error and clear the
handle. Artifact profiling no longer assumes an optional cache baseline exists, avoiding a
diagnostic-only production panic. Regressions lock deferred/failed attachment rejection and the
invalid-font artifact result. This is `non_validation_implementation_complete /
managed_validation_pending`: the isolated new-module and presentation formatting checks and
scoped whitespace check pass, while the shared layout-root formatting check still reports existing
workspace drift and must not be represented as a full formatting pass. No managed Cargo run,
performance/power measurement, WGPU framebuffer execution, or PNG was performed; the required
product evidence path remains `docs/tests/runtime/text`.

The source/range follow-up is `non_validation_implementation_complete /
managed_validation_pending`. Focused tests distinguish three invalid ownership cases from the
still-valid visual-only absence case. The implementation stays in the existing artifact lifecycle
and projection owners; `glyph_artifact.rs` remains 797 lines, while the new 116-line
`tests/invariant_failures.rs` child avoids growing the 969-line test root. Unreal Slate's shared
shaped-sequence/cache path rejects dirty values and requires a valid final subsequence; Fyrox keeps
source, line ranges, and glyph data in one formatted-text owner. Zircon keeps its stronger typed
failure instead of adding a renderer-local reconstruction path.

The 2026-08-26 Plain MVP renderer follow-up removes the next untyped branch for resolved glyph
artifacts. `logical_text_batches` now distinguishes `Missing`, `Stale`, and `Incomplete`; the
planner emits one command-level `Artifact`, `VisualOnly`, `SourceIsomorphicFallback`, or `Rejected`
receipt, keeps a rejection-only plan alive through prepare, and projects seven low-cardinality
`ui_text.resolved_glyph_artifact_route.*` counters. Only an exact source-isomorphic Plain layout may
use the established outer fallback; visual/BiDi output without its canonical artifact is rejected
instead of being reshaped in graphics. The contract is deliberately named for resolved glyph
artifacts, so it does not pretend that the separate compiled-rich route is already covered.

This follow-up is `non_validation_implementation_complete / managed_validation_pending`. Focused
tests cover artifact, valid visual-only, source-isomorphic missing-artifact fallback, rejected
visual/BiDi, stale/incomplete reason aggregation, prepare-report propagation, and profiler
projection. Scoped formatting, whitespace, call-path, file-budget, and production-exception scans
pass; `render.rs` is 782 lines, `resolved_layout.rs` 579, the decoration leaf 188, and the profiling
root/leaf 788/41. `RTS-P1-047/RTS-GATE-044` remain open for compiled-rich parity plus managed Cargo,
profiling/power evidence, current WGPU output, PNG inspection under `docs/tests/runtime/text`,
milestone commit, and WeCom synchronization.

2026-08-26 retained document-key/source-alias architecture review: `TextDocumentKey` is
crate-private and production creates it only from the surface node id plus
`UiLayoutCache::text_layout_revision`. Property mutation, rebuild, V2 style mutation, and pooled
node replacement advance that revision at the text-dirty owner. The same key is an O(1) lookup
receipt for long-document viewport work; recomputing a whole-source `DefaultHasher` on every
viewport request would restore `O(N)` scrolling cost and is explicitly rejected.

The previous key-only correctness gap is now narrowed again. `TextFrameDedup` and
`TextLayoutCache` still use the key only to find a bucket and compare exact stored text. The
retained Plain parsed-document owner is folder-backed under `ui/text/measure_cache` and rejects a
same-key/different-source hit before replacing the parsed artifact; its bounded report records
exact qualifications and stale aliases. `HardLineIndexCache` now retains the parsed document's
existing `Arc<str>`, accounts the retained source in its byte budget, uses `Arc::ptr_eq` for the
stable path, and performs exact comparison only when pointer identity differs. A stale alias removes
the old index and rebuilds the requested ranges. Focused regressions cover both changed-source
rebuilds and the hard-line pointer fast path.

RTS-P1-044 remains open at the upstream source owner. `UiNodeVisualData::resolve` and
`UiRenderCommand` still materialize a fresh `String` during render extraction, so the retained
parsed-document owner cannot receive a pointer-stable source/revision receipt and must exact-compare
the incoming Plain source on a same-key qualification. This is correct but does not yet prove the
intended `O(visible)` long-document dirty/scroll path. The structural follow-up is to retain one
immutable source snapshot beside the surface revision and reuse it through request, parsed artifact,
and hard-line index; a per-viewport whole-source `DefaultHasher` remains rejected. This slice is
`non_validation_implementation_complete / managed_validation_pending`: scoped Rustfmt and
whitespace checks pass, while managed tests, the 10,000-line visible-window gate, profiling/power,
WGPU framebuffer execution, and the required PNG under `docs/tests/runtime/text` remain pending.

2026-08-26 Plain post-wrap BiDi ownership review: `CanonicalPhysicalLineFragment` now retains the
one Text02 `BidiLineOrder` selected after Text03 wrapping and ordinary Plain UI layout consumes it.
This removes an independent UI order truth but is not recorded as a performance optimization: the
fragment owner still resolves the order explicitly, while rich/vertical/viewport fallback routes
may still analyze through the adapter. Two profiling scopes separate canonical receipt generation
from fallback analysis. The managed M0 trace must compare their call counts and p50/p95 across
1/100/1k/10k mixed-direction graphemes plus BidiTest/isolate cases before any attempt to reuse
shaping levels or change storage. Static checks pass; no timing, RSS, power, WGPU, or PNG evidence
exists yet.
## 2026-08-26 RTS-P1-046 Runtime UI operation-owner convergence

The Runtime UI path now follows one explicit text owner per operation. `UiSurface` retains one
`UiTextMeasureCache` and its `SharedTextLayoutSession` across measure, full/incremental layout,
render extraction, and glyph-artifact preparation. Standalone layout and extraction helpers create
one short-lived owner for the whole operation. Secure text fields and dialog action sizing receive
the extraction owner; Editor projection/materialization consumes the retained surface
`render_extract` instead of re-running extraction.

This is a structural and static-compliance result, not a performance result. Focused regressions
cover one-session standalone layout/extract construction, zero extra surface construction, secure
field owner routing, and dialog frame-measure deduplication. Editor retained paint, explicit
one-shot helper budgeting, managed Cargo validation, 31-sample p50/p95/p99/allocation/power
capture, and real WGPU framebuffer proof under
`docs/tests/runtime/text` remain open. No artifact is written under `target`.

The recursive layout measure boundary was hardened in the same slice: `measure_node`,
`measure_node_incremental`, fixed-width leaf measurement, and ordinary leaf measurement require the
operation cache instead of accepting an optional provider. This is an ownership invariant only;
the layout algorithm and its incremental traversal were not rewritten on static evidence. The
post-overlap render-command layout resolve phase now requires that same cache; only the explicitly
admitted parallel collection phase may defer owner text until its join.
Component text-field/dialog measurement and the extraction layout route also require the owner, so
the optional boundary is confined to overlap admission rather than nested rendering helpers.

The font-generation continuation closes the remaining renderer-owned artifact rebuild. `UiSurface`
now compares the shared font generation before `rebuild_dirty` can take its clean-frame return; a
mismatch marks the retained text owner dirty and performs one full layout/render-extract rebuild
through the existing surface session. Graphics rejects any artifact batch that became stale after
font loading, reports `ui_text.prepare.post_layout_stale_artifact_batch_rejections`, and emits no
glyphs from that retired artifact. `rebuild_resolved_text_glyph_artifact_line`, the secure
presentation rebuild, and `ScreenSpaceUiGlyphArtifactLine::refreshed_line` are deleted rather than
replaced by another renderer cache/session. Focused contracts cover retained-session recovery and
zero-session stale-batch rejection. Rustfmt, whitespace, and forbidden-call scans are current; the
Windows managed `font_generation` test batch did not start Cargo because `cargo.acquire` returned
`command_post_timeout` after submission acceptance. No retry or coordinator polling was performed.

The final static closure found and corrected three stale test contracts that would otherwise have
hidden the hard cut: the ellipsis/virtual-glyph regression now advances the font generation and has
the Text owner republish a complete immutable artifact, the render structure guard names the
source-isomorphic/native artifact routes and explicitly forbids the former renderer refresh helper,
and the prepare-report equality fixture expects zero post-layout stale-artifact rejections. A
repository-wide Rust-source scan now reports zero literal occurrences of the deleted rebuild API,
refresh helper, and `refreshed_line` field. This remains static evidence; managed compilation and
execution are still pending.

The adjacent Editor retained-paint review did not justify another implementation change without
the managed M0 trace. Its normal `paint_text` path already uses a bounded 2,048-entry cache keyed by
host font identity, runtime font generation, geometry, smoothing, and layout policy; direct
`layout_text` and per-line fallback shaping occur on a cache miss. The separate
`measure_text_size` call is confined to the final no-run/no-shaped Host fallback. Converting that
cache to a different owner or relative-coordinate representation remains an optimization gated by
the declared trace; no process-global replacement cache, renderer layout owner, or unmeasured key
rewrite was added in this slice.

## 2026-08-26 Arabic Tatweel candidate safety and profiling plan

Current-source review found a correctness defect before a proven performance defect: the Tatweel
fit loop previously accepted any finite candidate width below the target, even when shaping used
glyph 0, split the joining context across fallback faces, mixed generated/source clusters, or did
not expand the line. `text/layout/arabic_justification.rs` now validates the retained
`MeasuredTextLine` and publishes a success receipt only for an independent nonzero Tatweel cluster
with positive advance, RTL neighbors, and one face/instance. The validator is a monotonic
`O(glyph clusters + insertions)` scan; its insertion-range scratch is bounded by the existing 32
candidate limit. These are source-complexity facts, not measured latency or power results.

The 32-candidate and 5-probe policy is frozen until a matched profile is available. The managed
profiling run must aggregate once per justified physical line: requested/attempted/accepted
Tatweel count, candidate input bytes, safe receipt count, a low-cardinality rejection category,
shaped-cache hit/miss and backend shape-call count. It must use 31 cold/warm samples at
1/100/1k/10k graphemes for Arabic-only and mixed RTL text, with fixed face/fallback set, language,
features, DPI and frame width; record p50/p95/p99, allocations, RSS and process power. Per-glyph or
per-probe trace events are prohibited because their cardinality would contaminate the measured
algorithm.

The line-local instrumentation is now implemented without changing that algorithm.
`ui/text/layout_engine/line_box/profile.rs` retains counters only in profiling builds and emits one
`arabic_tatweel_candidate_fit` scope plus six aggregate samples for each physical line that reaches
candidate fitting: requested count, probe count, total candidate input bytes, safe-candidate count,
accepted Tatweel count, and last safety-rejection code. Probe iterations only update saturating local
integers; they do not publish events or touch a global/TLS registry. Rejection code 0 means no safety
rejection, codes 1 through 13 are the explicit stable mappings on
`ArabicTatweelCandidateRejection::profile_code`, and code 14 is the separate receipt-count mismatch
guard. A profiling-feature contract uses a real Arabic shaped/layout fixture rather than a synthetic
counter-only target. Static formatting, source, cardinality, and file-budget checks are complete;
managed execution of that contract is pending.

No probe-search, cache lifetime, or canonical artifact optimization is authorized until those
results identify a dominant owner and the same corpus preserves source anchors, cluster/face
identity, width bounds, and framebuffer pixels. Managed Cargo/profile, Unreal experience-value
comparison, power data, and real WGPU PNG evidence under `docs/tests/runtime/text` are pending;
there is no claim that the bottleneck disappeared or that the current algorithm is optimal.

Status: `arabic_tatweel_probe_instrumentation_implemented / algorithm_unchanged /
static_checks_complete / managed_profile_pending`.

## 2026-08-26 Plain soft-hyphen retained-artifact profiling contract

The typed discretionary-hyphen cut does not add a paragraph pass or a growing candidate reshape loop.
Only the final Plain line that contains generated display content enters the existing
`LogicalVirtualLineSequence`; its canonical fragment must serve layout metrics and renderer artifact
projection. A real `pre\u{00ad}fix` Word-wrap profiling fixture now requires exactly one logical-virtual
shape and one retained projection, with zero renderer projection-shape and fallback-shape requests.

This contract has passed formatting and source checks but has not run through managed Cargo. Rich text
is intentionally excluded because it still lacks a per-run canonical glyph artifact and uses
source-owned UAX#9. No latency, allocation, RSS, power, or cross-engine comparison is claimed. Status:
`plain_soft_hyphen_retained_artifact_profile_contract_implemented / rich_artifact_open /
managed_profile_pending`.

## 2026-08-26 Rich immutable artifact structural cut

The current-source review found a structural cost before changing any cache budget or micro
algorithm. One public rich handle could resolve either compiled metadata or a glyph artifact, never
both, while the renderer reshaped every visual paint run. RTL visual projection can split one
logical style run into grapheme-sized paint runs, so this route multiplied shaping requests and lost
joining context independently of cache lookup cost.

The implemented private composite payload now owns compiled metadata, one immutable glyph artifact,
the exact layout-line snapshot, and a run-to-glyph-slice directory. A physical line stores glyphs
once. Renderer runs borrow ranges from that line; a continuation run inside a cross-style ligature
gets an explicit empty range instead of falling back and drawing the glyph twice. Intentional
fallback lines retain a negative artifact receipt, so render extraction does not rebuild them every
frame. Font-pair registration is batched once per line.

Profiling-only counters are bounded to rich artifact shape requests/input bytes, mapped runs,
shaped-cache hits/misses, renderer artifact-run uses, and renderer fallback requests/input bytes.
No per-glyph or per-grapheme event was added. The static algorithm is
`O(line/style intersections + glyphs + clusters + paint runs)`; that is a source-complexity
statement, not measured performance.

Status:
`rich_composite_artifact_and_run_slice_route_implemented /
rich_horizontal_soft_hyphen_virtual_artifact_implemented /
rich_vertical_soft_hyphen_virtual_artifact_implemented /
virtual_receipt_linear_capture_implemented /
static_checks_complete / managed_profile_pending`. Managed Cargo, 31-sample cold/warm
1/100/1k/10k workloads, backend call attribution, allocations/RSS, p50/p95/p99, process power,
Unreal experience-value comparison, real WGPU framebuffer, and PNG evidence under
`docs/tests/runtime/text` remain pending.

2026-08-30 current-source/profile follow-up: the composite artifact already eliminates normal rich-run
renderer reshaping. `render/paint_projection.rs` now measures the remaining transient `UiTextPaint`
materialization boundary with a fixed scope and twelve low-cardinality command/run/text/style-byte
counters. Only rebuilt plan-cache segments count; exact cache hits publish zero new work. Static Runtime
Text coverage passes 52/52. Payload byte lengths are not allocator/RSS proof, so the 31-sample managed
baseline, power capture, DTO-owner decision, WGPU, and PNG gates remain open.

Horizontal rich soft hyphen now retains the same logical-display/BiDi sidecar class used by Plain,
but resolves each logical cluster back to its rich source style before shaping. Adjacent equal styles
coalesce; glyph projection and zero-width run ownership remain monotonic. A profiling-only
`rich_artifact_logical_sidecar_line_count` distinguishes virtual/external sidecar routes without
adding per-run or per-glyph events. Vertical U+2026 and typed discretionary hyphen both use the
canonical artifact; U+00AD remains in the replacement receipt rather than masquerading as the display
run's source bytes. This is static
infrastructure evidence only; it does not change the pending managed profile or power gates.

## 2026-08-26 Rich ellipsis current-run owner receipt

Horizontal text-only rich ellipsis now enters the same logical-display sidecar before UAX#9 instead
of being appended to an already visual line. A candidate-owned non-empty source range identifies the
style run for each generated cluster. The receipt drives marker measurement, coalesced artifact
shaping, the run-slice directory, and renderer font/color/decoration lookup; a zero-width selection
anchor is no longer treated as a style lookup key.

The owner preview and final retention are linear grapheme passes. Logical style coalescing and glyph
directory publication remain monotonic, so the static bound stays
`O(graphemes + style intersections + glyphs + paint runs)` rather than `O(styles * graphemes)`.
The final-line owner also records one optional replaced-source interval only when retained ranges leave
exactly one non-empty gap. Caret, hit-test, and selection queries consume that receipt in the existing
cluster/glyph walk, preserving `O(clusters + glyphs)` query work without a persistent paragraph-sized
map. Accessibility remains source-owned by template/component/widget state and never adopts visual
ellipsis text. This is a complexity review and ownership proof, not profile evidence. Status:
`rich_text_only_ellipsis_virtual_artifact_implemented /
virtual_source_and_style_receipt_implemented /
private_omitted_source_geometry_receipt_implemented /
accessibility_source_preservation_confirmed / static_checks_complete / managed_profile_pending`.
Managed Cargo, 31-sample scale workloads, allocations/RSS, p50/p95/p99, power, Unreal timing
comparison, real WGPU framebuffer, and PNG evidence under `docs/tests/runtime/text` remain pending.

## 2026-08-26 Rich inline external-block artifact review

Local Unreal `SlateImageRun`/`SlateWidgetRun` review confirms that a compiled inline image/widget owns
one source character range, measures and arranges an independent layout block, and paints the object;
it is not a font glyph. Runtime Text now captures exact compiled inline ranges as external clusters.
The visual-order owner retains those clusters for UAX#9 and the final advance array, while rich style
coalescing splits around them and glyph projection omits them. A literal U+FFFC has no compiled receipt
and remains normal text. Horizontal text+inline, inline+ellipsis, and inline-only lines share this path;
the latter is admitted as a zero-glyph artifact only when every cluster is explicitly external.

Capture, style coalescing, glyph projection, run-directory publication, and visual geometry use
monotonic cursors, preserving `O(clusters + virtual receipts + external ranges + style intersections +
glyphs + paint runs)`. No per-inline
font shaping, renderer-local object inference, placeholder glyph cache entry, or paragraph-sized
geometry map is introduced. A rejected visual-order sidecar is also rejected by geometry consumers.
This is static algorithm/ownership evidence, not measured performance evidence. Status:
`horizontal_inline_external_cluster_artifact_implemented /
inline_empty_glyph_slice_and_geometry_receipt_implemented /
vertical_rich_and_external_block_canonical_artifact_implemented /
vertical_rich_ellipsis_virtual_artifact_implemented /
vertical_rich_soft_hyphen_virtual_artifact_implemented /
typed_virtual_fragment_role_implemented /
virtual_receipt_linear_capture_implemented /
logical_virtual_glyph_projection_owner_split_implemented /
logical_virtual_fragment_validation_owner_split_implemented / static_checks_complete /
managed_profile_pending`. Managed Cargo, 31-sample timing/allocation/RSS, power, Unreal timing
comparison, real WGPU framebuffer, and PNG evidence remain pending.

## 2026-08-26 Rich renderer linear directory publication

Current-source review found that the so-called rich run directory was still resolved with two nested
linear searches: every paint run scanned layout lines, then scanned the artifact run Vec. The worst
case was `O(R^2)` for `R` styled runs before renderer shaping or painting. The resolved-layout owner now
consumes the already canonical flattened `layout.lines -> line.runs` order once and calls an exact
indexed runtime-artifact resolver. Text/source/visual identity is checked at each cursor position;
cardinality or order drift rejects the complete route batch. Static publication complexity is
`O(lines + runs)` with O(R) route output and O(1) directory access.

The route output distinguishes canonical `Artifact`, intentional compatibility `VisualOnly`, and
`Rejected(Missing|Stale|Incomplete)`. These low-cardinality counters are present in the existing render
route report and profiling aggregates. Normal rich profiling now requires zero
`shape_renderer_fallback` spans; visual-only compatibility remains measurable separately. The renderer
only reshapes a rejected text run when the resolved line proves exact source isomorphism;
non-isomorphic rejected runs produce no batch, preventing stale/missing artifact recovery from
silently changing BiDi, virtual-marker, or source-range semantics. The renderer
root was reduced from 816 to 710 lines by moving rich planning into its 473-line owner. This is source
complexity and ownership evidence only. Status:
`rich_renderer_typed_linear_run_directory_implemented / static_checks_complete /
rich_nonisomorphic_rejection_fail_closed_implemented /
managed_profile_pending`. No p50/p95/p99, allocation/RSS, power, Unreal timing, WGPU, or PNG result is
claimed.

2026-08-31 scope correction: the complexity statement above applies to typed glyph-artifact route
publication only. Current-source review found that `text_paint_runs_from_resolved_layout` still derives
each paint-run frame with repeated whole-line grapheme/advance work, while the inline renderer again
searches the line/run and reconstructs the same prefix. Profiling builds now publish seven fixed
inline-work and paint-frame-agreement counters; ordinary builds retain no counter fields. The complete
Interface production-helper benchmark is also source-present as a Windows release-only ignored test
with 1/100/1k/10k runs, three warm-ups, 31 raw timing/RSS samples, and p50/p95/p99 output. It has not run;
the renderer now has a separate untimed-counter/timed-planning harness for dense LTR/RTL/VerticalRl
1/100/1k/10k inline objects and 1/100/1k hard lines. Neither harness has run; the complete baseline and
any single-owner geometry cutover remain pending. Detailed plan:
[`09/2026-08-31-rich-paint-block-geometry-owner-and-profile-plan.md`](09/2026-08-31-rich-paint-block-geometry-owner-and-profile-plan.md).

## 2026-08-26 UI text cache byte-residency pre-cap instrumentation

The current-source review found that the retained UI text caches were bounded only by entry count:
4096 measure entries and 2048 layout entries. That is not a memory budget because one entry may retain
a long source plus variable-size line, run, advance, editable, and composition DTOs. This slice adds
measurement before policy: admission, LRU ordering, entry caps, and layout behavior are unchanged.

`TextMeasureCache` and `TextLayoutCache` now maintain current and frame-peak estimated bytes at their
lowest cache owner. Insert, same-key update, LRU eviction, end-of-frame trimming, and clear update the
receipt consistently. UI admission supplies the source/style key heap estimate; layout admission also
supplies the serializable resolved-layout DTO estimate. Profiling publishes four low-cardinality
measure/layout current/peak `cache_dto_source_*_bytes` counters without per-entry labels or source text.

This estimate is intentionally named and documented as a source/DTO-owned heap lower bound. It excludes
opaque/shared glyph artifacts, allocator RSS, hash-table capacity, and allocator overhead. Charging a
shared artifact to every layout entry would double count memory that may not be released on entry
eviction, so artifact residency remains open until its unique owner can publish a non-duplicating
receipt. `Arc::strong_count` is not an admission or accounting policy.

Focused source regressions cover insert/update/evict/clear/peak accounting. Rustfmt, targeted
whitespace, source-owner guards, file-budget checks, and scoped diff-check pass. Managed Cargo,
31-sample scale workloads, allocation/RSS, power, Unreal timing comparison, artifact-owner attribution,
byte-cap/admission tuning, real WGPU framebuffer, and PNG evidence remain pending. Status:
`cache_dto_source_residency_receipt_implemented /
cache_update_evict_clear_accounting_implemented / static_checks_complete /
artifact_residency_and_byte_cap_open / managed_profile_pending`; `RTS-P1-045` remains open.

## 2026-08-26 Oversized shaping work production receipt

The 64 KiB `TextShapingWorkBudget` previously had no production decision or observation site. The
retained session entered the canonical backend synchronously after a shaped-cache miss; prewarm
deduplicated pending jobs and joined their completion. Cutting text at this threshold would change
Arabic, Indic, emoji, and ligature context, while returning Deferred before a caller contract exists
would break synchronous layout. This slice therefore adds evidence without changing scheduling.

`TextShapingWorkReport` records within-threshold requests, oversized requests that still complete
synchronously, total synchronous input bytes, and maximum request bytes. The session records only
canonical cache misses. Parallel prewarm records only unique pending jobs, excluding cache hits,
batch duplicates, and invalid requests, then merges the receipt into the same frame owner. Batch and
UI profiling expose aggregate low-cardinality counters and never publish source text.

Local Unreal review anchors this boundary in `FTextLayout`: stable `FLineModel` dirty state and
estimated geometry survive while `FLineView` is materialized lazily. `FSlateFontCache` range shaping
retains text outside the selected range as context. Zircon therefore keeps source/context complete
and treats the byte threshold only as a future scheduling classification, not a line/run/cluster cut.

Tests cover classification/merge, one charge per session miss with no cache-hit recharge, unique
parallel pending attribution, and an artificially small threshold proving the complete string is
still shaped as oversized synchronous work. The session regression lives in a focused child so the
owner root remains below the 800-line warning. Rustfmt, scoped diff-check, call-site/report/private-
boundary guards, and file-budget checks pass. Managed Cargo, 1/100/1k/10k 31-sample profiling,
CPU/allocation/RSS/power, typed deferred/cancelled work units, WGPU, and PNG evidence remain pending.
Status: `shaping_work_budget_production_receipt_implemented /
cache_miss_and_unique_pending_attribution_implemented / source_semantics_preserved /
algorithm_unchanged / static_checks_complete / typed_defer_cancel_and_managed_profile_pending`.
`RTS-P0-001`, `RTS-P1-015`, and `RTS-P1-016` remain open.

## 2026-08-26 Final-line break-safety profiling receipt

The shaping `Safe/RequiresReshape/Unknown` receipt now survives through measured clusters into the
private grapheme advance index. Profiling builds aggregate the actual candidate boundaries selected
by boundary correction with one monotonic cluster cursor. The four low-cardinality counters are
`text.layout.boundary_candidate_ranges`, `text.layout.boundary_receipt_safe`,
`text.layout.boundary_receipt_requires_reshape`, and `text.layout.boundary_receipt_unknown`.

The observer costs `O(candidate boundaries + measured clusters)`, allocates no boundary vector,
emits no per-boundary event, and carries no source text. Non-profiling builds do not perform the
queries. UAX#14 opportunities, atomic-cluster coalescing, the fixed eight-grapheme correction window,
layout output, and shape-call counts are unchanged. Managed corpus/profile/power evidence is still
required before selecting an exact two-sided reshape or another context policy. Status:
`break_safety_measurement_retention_implemented /
monotonic_candidate_boundary_profile_implemented / algorithm_unchanged / static_checks_complete /
exact_final_line_reshape_and_managed_profile_pending`; `RTS-P1-017` and `RTS-P1-035` remain open.

## 2026-08-26 Vertical substitution comparison profiling receipt

The current vertical `TransformOrRotate` path performs a second RustyBuzz shape with `vert/vrt2`
disabled to prove whether each cluster actually received a vertical substitution. Itemization merges
adjacent graphemes with the same face, instance, BiDi level/direction, script, and vertical orientation,
so this work is `O(Tr logical segments)` with a worst-case `O(graphemes)` call bound.

The existing request-local backend-call TLS now lives in `cosmic/direct_profile.rs` and also aggregates
comparison calls, complete segment input bytes, disabled-output glyphs, and changed clusters. A
successful direct request publishes one fixed eight-counter set; a failed direct request discards its
local report. Segment loops emit no profiler events, source labels, timings, or global-lock updates.
Normal builds and inactive CPU captures create no report; Tracy retains its continuous-build behavior.

The ignored managed scale harness adds a `vertical_tr` workload at 1/100/1k/10k units and 31 samples.
Non-Tr workloads require zero comparison attribution; Tr input attribution must be complete and calls
must stay linear. No run data exists yet, so backend trace, feature-plan provenance, or comparison-cache
changes remain deferred. Status: `vertical_substitution_comparison_receipt_implemented /
request_local_capture_only_aggregation_implemented / algorithm_unchanged / static_checks_complete /
managed_profile_pending`; `RTS-P1-019` and `RTS-P1-020` remain open.

## 2026-08-26 Common/Inherited script-run current-source correction

The old `pending_common_start/end` performance/correctness finding no longer matches current source.
The paragraph owner performs one forward `ScriptExtension` bitset intersection; the locked
`unicode-script 0.5.8` dependency represents Common and Inherited as wildcard sets. Leading neutral
characters therefore converge on the first specific script, intermediate and trailing punctuation
remain with the previous compatible run, and all-Common text remains `Zyyy`.

Focused regressions now freeze those policies. Production itemization remains allocation-free apart
from its output and bracket stack and retains `O(codepoints + runs + bracket_depth)` complexity; no
delayed Common state or extra pass was added. Status:
`stale_pending_common_finding_corrected / script_extension_policy_regressions_added /
production_algorithm_unchanged / static_checks_complete / managed_text_test_pending`.

## 2026-08-26 Unified vertical cluster decision receipt

Vertical shaping now retains one compact decision basis on each cluster head: Unicode orientation,
the effective `vert/vrt2` set, substitution proof state, and a typed fallback reason. Complete runtime
and neutral decisions combine that basis with the glyph's existing rotation and selected face/instance,
so font identities are not duplicated and renderer consumers do not re-run Unicode or font queries.

The direct path derives the feature set from the same projected backend features and maps the existing
Tr output comparison to `Observed/NotObserved`. Compatibility shaping explicitly reports unavailable
backend provenance for Tr instead of guessing. The path adds no shape call, paragraph pass, font lookup,
or heap allocation; access is constant-time. Managed size/RSS/profile/power data is still required.
Status: `vertical_cluster_decision_basis_implemented /
direct_feature_set_and_substitution_provenance_retained / neutral_projection_preserved /
compatibility_unknown_explicit / static_checks_complete / managed_validation_pending`.

## 2026-08-26 Horizontal hybrid shaping composition receipt

Horizontal direct shaping now completes every reachable logical segment and retains sorted failure
holes instead of discarding successful glyphs at the first backend capability error. One complete
Cosmic candidate is qualified with monotonic direct/alternate cursors; identity, line topology,
source order, hole containment, and non-empty coverage failures retain that candidate unchanged.
Qualified output merges only hole-contained alternate glyphs and rebuilds positions and selected-face
line metrics. Qualification and merge are `O(lines + holes + glyphs)` with no per-glyph hole scan.

A hybrid `ShapedGlyphRun` allocates one boxed receipt containing absolute alternate ranges and the
first failure; ordinary runs allocate none. Receipt payload and range capacity participate in shaped
cache byte admission. Profiling detaches direct TLS before Cosmic and publishes one low-cardinality
request aggregate for input bytes, holes, retained direct glyphs, selected alternate glyphs, rejected
composition, and direct backend calls. There are no source labels, per-glyph events, or hot-path locks.

Rustfmt, scoped whitespace, complete run-literal initialization, unique classifier, cursor, and file
budget checks pass. Cargo, fault injection, scale timing, allocation/RSS, power, WGPU, and PNG remain
pending. A whole Cosmic candidate is still shaped, so no backend-work reduction or engine-parity claim
is made. Status: `direct_partial_attempt_implemented / source_ordered_hybrid_composition_implemented /
selected_face_metric_rebuild_implemented / hybrid_artifact_receipt_and_profile_implemented /
fail_closed_whole_candidate_retained / static_checks_complete / managed_validation_pending`.

## 2026-08-26 Source lifetime pre-optimization receipt

Current synchronous shaping allocates exact run source when no owner is supplied, while parallel hard
line splitting materializes one Arc owner per paragraph before that Arc is reused by the run. Local
Unreal Slate instead borrows external text and keeps only source range/index mapping in its shaped
sequence. This establishes the target ownership direction but does not prove source storage is the
measured Zircon bottleneck.

The shaped-artifact boundary now reports materialization, exact-owner reuse, allocation count, and
allocation bytes. Parallel batches separately report logical source leases, unique Arc owners, leased
bytes, and unique-owner bytes before cache/duplicate admission. These are fixed request aggregates;
they expose neither text nor pointer identities and add no glyph-loop observer. Shaping, cache
admission, paragraph splitting, source storage, and renderer behavior are unchanged.

The managed matrix is cold/warm synchronous/parallel 1/100/1k/10k hard lines, stable and one-line-edit
documents, duplicate paragraphs, mixed scripts, and controlled hybrid failure, each at 31 samples with
allocation/RSS/p50/p95/p99 and valid-sensor power. Only a material source allocation/residency share
authorizes immutable document snapshot + range lease and unique-owner cache accounting. Glyph SoA is a
separate measured migration. Status: `source_lifetime_architecture_research_complete /
unreal_external_text_owner_confirmed / source_materialization_and_batch_owner_instrumentation_implemented /
algorithm_unchanged / static_checks_complete / managed_profile_pending`.

## 2026-08-26 Ephemeral cache hash and stable artifact digest boundary

Runtime Text cache lookup identities now use `EphemeralCacheHash` instead of bare `u64` fields named
`content_hash`, `source_hash`, or `text_hash`. The type has no serialization or byte-export API. One
`EphemeralCacheHasher` owner contains `DefaultHasher`; shaped, pending-batch, rich, measure/layout, and
physical-line caches still compare complete keys and exact source before reuse. `TextDocumentKey`
continues to provide `O(1)` owner+revision identity, so viewport work does not hash the whole document.

Persisted SDF generation/offline identities use `StableContentDigest`. Existing BLAKE3 bytes, v1
header, field order, checksum, and artifact path are unchanged, and the public inspection tool still
returns `[u8; 32]`. Artifact/replay owners must pair a stable digest with their own format/domain
version; a cache hash cannot cross that boundary.

This is a zero-storage-overhead type migration and does not change cache algorithms, shape work, hash
complexity, or SDF output. Rustfmt, scoped diff-check, sole-DefaultHasher-owner, production bare-hash,
and digest-propagation scans pass. Managed Cargo, collision/golden tests, timing/RSS/power, WGPU, and
PNG remain pending. Status: `ephemeral_cache_hash_type_implemented /
stable_artifact_digest_type_implemented / default_hasher_isolated / sdf_v1_bytes_unchanged /
algorithm_unchanged / static_checks_complete / managed_validation_pending`.

## 2026-08-26 Paragraph and layout artifact lifetime audit

`SharedTextLayoutSession` is already a retained UI measure owner with separate shaped-run, hard-line,
and layout/measure cache owners. A shaped-cache hit does not rebuild paragraph Bidi/script analysis,
and keyed plain viewport requests reuse one hard-line source owner. The remaining hypothesis is
duplicate analysis on miss/fallback paths: direct and Cosmic each construct line-break/hard-line
views, while rich advance-index, physical-line fragments, and viewport projection have distinct line
consumers.

No retained paragraph artifact is authorized yet. The measurement matrix is plain/rich,
direct-success/partial-fallback/terminal, cold/warm, 1/100/1k/10k hard lines, stable scroll and
single-line edits, with 31 samples per cell. Record analysis construction count, hard-line and
line-break bytes, shaped-cache hit/miss, layout DTO current/peak, allocation/RSS, p50/p95/p99, and
valid-sensor power. Only a measured dominant duplicate-analysis share can authorize a
document-revision-owned artifact containing source snapshot, Bidi/script/line-break analysis,
hard-line index, and dirty-range dependencies. Glyph SoA, source leases, renderer artifacts, and
cache-policy changes remain separate decisions.

Static current-source owner scan and rustfmt are complete; no algorithm or shape-call behavior changed.
Managed Cargo, counters, timing/RSS/power, WGPU, and PNG remain pending. Status:
`paragraph_lifetime_architecture_review_complete / duplicate_analysis_instrumentation_deferred /
algorithm_unchanged / static_checks_complete / managed_profile_pending`.

## 2026-08-26 Stable text layout diagnostic catalog

`TextLayoutError` now has a stable diagnostic code and localization catalog key for every variant.
The process-facing code is `ZR-TEXT-LAYOUT-001` through `ZR-TEXT-LAYOUT-011`; the catalog key is
`text.layout.<variant>`. These values are constant data and add no per-request allocation or cache
state. `Display` is retained only as a human-readable projection. Status:
`diagnostic_code_catalog_implemented / backend_neutral_boundary_preserved /
focused_behavior_tests_complete / managed_validation_pending`.

## 2026-08-26 UI shaper wrapper removal

The one-member `UiTextShaperStack` added no scheduling, cache, backend ordering, or receipt state.
It is removed and entrypoints invoke `UiSharedTextShaper` directly. This deletes one misleading
abstraction without changing allocation, shaping calls, layout, cache, or render behavior. Status:
`empty_ui_shaper_stack_removed / sole_shared_adapter_preserved / source_guard_updated /
static_checks_complete / managed_validation_pending`.

## 2026-08-26 Serializable DTO and renderer batch residency receipt

Public `UiResolvedTextLayout`/`UiShapedText` values remain serde-capable boundary DTOs; they are not
silently changed to process-local `Arc` or leases. The layout cache already accounts owned line/run
text and advances. The final prepare report now separately sums native/SDF renderer batch count,
UTF-8 text bytes, and glyph-advance bytes after Auto routing, so every final batch is counted once.
The receipt is allocation-free apart from the existing report and records no raw text.

Intermediate `UiTextPaint` clones and actual serialization materialization remain open. Managed
plain/rich, artifact/visual/fallback, cold/warm, 1/100/1k/10k-line profiling must show duplicate text
storage dominates before an internal range/Arc/lease cutover. Status:
`layout_dto_and_renderer_batch_residency_receipts_implemented / intermediate_paint_copy_open /
algorithm_unchanged / static_checks_complete / managed_profile_pending`.

## 2026-08-27 Runtime Text budget snapshot and residency receipts

Budget review now separates correctness context, per-line work, cache residency, and asynchronous
completion instead of introducing a cross-domain mutable profile. The hard-line index report carries
its effective 16-entry/32 MiB snapshot plus resident/eviction/oversized state. SDF scheduler
diagnostics carry all effective in-flight/glyph/source/completion limits beside backlog state.
Bitmap page-shadow publishes resident pages/bytes, its 32 MiB ceiling, and cumulative admission
rejections through renderer prepare; one known rejected page is not retried by a later patch in the
same commit.

The read-only projection lives under `text.runtime_budget.*` in a named 57-line child profile owner;
the prepare-profile root remains 771 lines. Defaults and admission/routing algorithms are unchanged.
Use the managed 1/100/1k/10k matrix to correlate usage/limit/rejection before changing a value.
Status: `owner_local_budget_snapshots_implemented / runtime_budget_profile_projection_implemented /
page_shadow_residency_receipt_implemented / algorithm_defaults_unchanged / static_checks_complete /
managed_profile_pending`.

## 2026-08-27 Session diagnostics ownership and route receipts

The layout-fallback and shaping-failure reports no longer use process-global mutexes. The retained
`SharedTextLayoutSession` owns one frame-local receipt and resets it at `begin_frame`; standalone
operations keep the same operation-local session boundary. Successful cache hits do not increment
backend-work counters. Completed cache misses classify direct, whole-run alternate, or hybrid output
from the shaped artifact. Failed outcomes count terminal runs; generation-deferred outcomes have a
separate typed receipt and do not contaminate the terminal route.

Parallel prewarm aggregates the same fixed shaping value inside `TextParallelShapeBatchReport` and
merges it into the retained session after join. Workers do not retain or lock the session. The UI
profile projects 35 fixed session counter names and stores no raw text, pointer, document ID, or
dynamic backend label. Exact document drill-down remains open until all plain/rich/measure/hit-test
paths share a bounded document diagnostics owner; `TextDocumentKey` must not become a metric label.

Static source/rustfmt/diff checks pass, and the profile function emits 66 counters against its 128-entry
focused-test capacity. No managed Cargo, contention timing, 31-sample RSS/power, WGPU, or PNG evidence
was produced. Status: `session_owned_diagnostics_implemented /
process_global_report_mutexes_removed / parallel_prewarm_merge_implemented /
document_drilldown_owner_open / static_checks_complete / managed_validation_pending`.

## 2026-08-27 Shape-range API terminology boundary

The shaping-stage line value is now `ShapedHardLine`; it is produced before wrapping, ellipsis, and
final placement. Provider/session methods are now named `shape_horizontal_range(_with_kerning)` and
`shape_vertical_range(_with_kerning)` because their contract is a source slice plus an absolute
range and one request may produce multiple hard lines. Final visual lines remain layout-owned.

This is a source hard cut across 41 Rust files with no alias or wrapper. It changes no serde field,
request key, work-budget accounting, cache policy, backend call, or layout algorithm. Exact old Rust
symbols scan to zero and scoped static checks pass. Managed Cargo, serde golden, WGPU/PNG, and
performance/power evidence remain pending. Status: `shaped_hard_line_term_hard_cut_complete /
shape_range_api_hard_cut_complete / algorithm_unchanged / static_checks_complete /
managed_validation_pending`.

## 2026-08-27 Generation-deferred diagnostics contract

All generation instability paths now construct one `FontGenerationChanged` receipt with deferred
disposition: stable-generation retry exhaustion, stale retained-session cache/result, explicit worker
defer, and stale parallel worker completion. Missing primary face uses a separate terminal
`FontPrimaryUnavailable` receipt. Session and batch reports expose fixed aggregate deferred
failure/run counters; no source, document identity, pointer, or dynamic label is emitted. The two
additional deferred counters are retained beside the later request-resolution dimensions. The
layout-resolve profile now has 66 emissions below its 128-entry focused capacity; the broader
integration capture uses 160. No cache admission,
scheduling, retry budget, or backend algorithm changed; managed
contention/timing/RSS/power validation remains pending.

## 2026-08-27 Font-resolution request work accounting

`TextShapingRequestDiagnostics` is a fixed 152-byte value carried beside the shaped run until the
session/parallel completion owner consumes it. It records shaping attempts, generation restarts, and
17 resolution counters. It is absent from the public shaped artifact and cache resident-byte model;
therefore a cache hit adds no historical backend work. Ready, terminal, and deferred paths merge the
same envelope, while generation retry keeps discarded attempt costs.

Coverage counts come from the existing resolver/candidate loops and match actual
`face_covers_codepoint` probes, including candidate compiler filtering, complete short circuit,
partial ranking, and missing diagnostics. No probe or allocation was added. Fixed UI dimensions are
35 session names; total layout-resolve emission is 66 under focused capacity 128, and the broader
integration capture uses 160. Managed Cargo, concurrent
stress, 31-sample cold/warm corpus timing, RSS/power, and WGPU/PNG remain pending, so these counters do
not yet prove a bottleneck or authorize an algorithm change.

## 2026-08-27 Fallback cache lock profiling gate

The five fallback cache layers currently share one mutable lock, and every LRU hit writes recency
state. Cold composite compilation also occurs inside that lock. A separate mostly-primary workload can
repeat coverage work when the whole-text primary scan rejects late and itemization restarts by cluster.
These are measured as independent hypotheses rather than assumed causes.

All cache-state operations now cross one owner boundary. Only test/profiling builds measure acquire,
wait, and hold nanoseconds. Request-scoped TLS publishes three fixed completion counters, avoiding both
per-lock profiler calls and overlapping process-global snapshot deltas under parallel shaping. Normal
builds do not call `Instant`. Cache topology and resolver behavior remain unchanged until the required
31-sample cold/warm CPU/timing/RSS/power matrix identifies the dominant term. Status:
`request_local_cache_lock_profile_implemented / structural_optimization_profile_gated /
managed_profile_pending`.

## 2026-08-27 Shape-request analysis construction profile

The canonical shape owner now aggregates eleven profiling-only values: request count/bytes plus build
count, input bytes, and elapsed nanoseconds for Bidi, script/emoji, and line-break analysis. Direct
success should expose one of each analysis build; horizontal whole-alternate/hybrid currently exposes
two line-break builds. That topology is an observation target, not proof of a bottleneck.

The three constructors only call `Instant` while a managed capture has activated request TLS. They
update integers locally and publish once after Ready/error completion; normal builds keep no profile
module or timing call. No analysis object is retained and no cache/lifetime/algorithm changed. Use the
required 1/100/1k/10k, 31-sample route matrix before hoisting line-break state or designing a retained
paragraph artifact. Status: `shape_request_analysis_profile_implemented /
duplicate_construction_observable / retained_paragraph_artifact_open / managed_profile_pending`.

## 2026-08-27 Document revision foundation boundary

Current source already contains a crate-private piece-backed `TextDocument`, revision key, exact
old/new byte dirty spans, length delta, and a revision-bound hard-line/grapheme source index. This
corrects the older statement that no edit delta existed. It does not close document layout: no
surface, reducer, IME, accessibility, render, or layout-session path consumes this owner, and an
invalidated source index still flattens the complete document before rebuilding.

The internal replace gateway now requires the expected `TextDocumentKey`. Stale input and revision
exhaustion return typed errors before source, pieces, length, index, or key can change. The byte-delta
regression now correctly records a 13-byte replacement over six bytes as `Increased(7)` and a new
dirty end at byte 19. The document authority is no longer cloneable or value-comparable, preventing
two mutable branches from publishing different source under one owner/revision identity.

Unreal Slate's retained `FLineModel`, per-capability dirty flags, model change counter, and lazily
generated visual views support the next owner boundary: stable hard-line models first, visual reflow
second. Do not derive old/new paragraph dirtiness by taking two full snapshots per edit. Profile the
separator-aware retained index and edit/scroll matrix before selecting an incremental structure.
Status: `internal_revision_foundation_implemented / product_authority_unwired /
paragraph_dirty_stable_line_reflow_open / static_checks_complete / managed_validation_pending`.

## 2026-08-27 Typed text-input constraint receipt boundary

The product sanitizer now returns one structured result for filter removal, canonical single-line
hard-separator removal and max-grapheme truncation. It scans the replacement once for filter/line
admission, treats CRLF as one separator and truncates the accepted buffer in place. Keyboard text,
text events, IME and accessibility project the same low-cardinality receipt without raw content.
The catalog and input gateway now also agree that a zero max length is unbounded.

This is observability and correctness infrastructure, not the requested incremental validator. Each
edit still counts graphemes in the retained prefix and suffix, so the current cost is `O(current text
outside the replaced range + replacement)`. Do not micro-optimize the character loop or add another
cache at this call site. First wire the retained document/grapheme index as the single authority, then
profile the 1/100/1k/10k edit matrix and attribute index repair, allocation, RSS and input latency.
Constrained preedit now records only cursor/clause endpoints during that same scan, then maps UTF-8
byte ranges and clamps the cursor to grapheme boundaries. Auxiliary memory is `O(clause count)` rather
than `O(preedit bytes)`; adjusted and fully removed ranges publish low-cardinality counts. Single-line
Enter now uses a dedicated handled Submit path with zero property mutations; repeat does not resubmit.
Production platform clause generation remains open. Status:
`typed_constraint_receipt_implemented / replacement_single_pass_implemented /
requested_boundary_edit_mapping_implemented / single_line_enter_submit_implemented /
retained_incremental_validation_open /
platform_clause_producer_open / managed_profile_pending`.

## 2026-08-28 Retained document storage structural profile closure

The pre-change 17-scenario, 31-sample matrix measured immutable addition-chunk/piece growth and
full-hard-line separator-neutral preparation as the two dominant edit costs. The selected correction
uses one append-only addition source and a local stable-hard-line content update, retaining the
separator-aware reparse route for CR/LF structure changes. The identical post matrix reduces 10k
tail insert p50 from 1,710.706 to 4.508 milliseconds and counted allocation from 8.127 GB to
3.643 MB; one-million-character local edit streams no longer copy the complete line.

`TextDocumentStorageReport` remains a content-free lower bound for current source capacity,
piece/hard-line/grapheme-index capacities, and the current flattened snapshot. Allocator headers and
old externally leased snapshots remain excluded, so the report still cannot be treated as an
admission budget. The data closes these two measured hotspots only; it does not select a rope, gap
buffer, tree-backed piece table, or guessed compaction policy. WPR stack/power and matched Unreal
runtime evidence remain open. Status:
`append_only_addition_source_implemented / local_hard_line_edit_implemented /
baseline_and_post_matrix_complete / structural_hotspots_eliminated /
power_and_matched_unreal_pending`.

## 2026-08-27 Surface-session document admission preflight

Document mutation now has a prepare/commit split. Prepare computes the checked next revision/length,
local hard-line repair and exact piece topology without changing source, pieces, indexes, revision, or
snapshot state; commit rechecks the expected key. This permits admission to reject a prepared change
before immutable source or a new revision is published.

The crate-private session store has no default policy and no global registration. Callers must provide
explicit per/total document bytes, replacement work input, retained/addition source, piece, current
snapshot and active lease count/byte limits. Managed snapshot leases are non-cloneable and release
their active budget on `Drop`. Reports and errors are content-free.

This is containment, not calibrated performance policy. Product thresholds remain unfrozen after the
document/storage profile matrix. The store belongs beside one surface's input/edit session,
not in clone/serde `UiSurface` and not in a process-global manager. `BTreeMap` lookup is currently
`O(log D)`; only a measured lookup cost may authorize another index. Status:
`document_prepare_commit_boundary_focused_harness_passed /
exclusive_store_prepare_commit_focused_harness_passed /
explicit_limit_session_store_focused_harness_passed / append_only_addition_source_implemented /
surface_input_session_integration_implemented_unvalidated /
product_thresholds_and_managed_runtime_pending`.

## 2026-08-28 Exact committed edit intent preflight

The edit-state owner now retains the exact old/new byte range and typed edit kind while applying an
edit. Replacement bytes are borrowed from the final state, so product document integration does not
need a second String or an `O(N)` before/after diff per key. Identical selected replacement, empty
insert, caret/selection movement, composition preedit/cancel, and unchanged composition commit are
state-only and do not request a document revision.

The one-or-two action keyboard sequence admits at most one committed edit. A future mapping with two
committed actions returns a typed error and publishes no reduced state. The internal property
transaction validates the intent before mutation and returns it in its receipt. Public receipt
projection remains document-owned.

This began as an algorithmic prerequisite; the following integration section now implements the
required ordering. Surface first prepares the ten-property mutation, then prepares/admits the session
document edit, validates public projection, and only then enters an infallible dual commit. Committing
either side before the other's fallible preflight remains forbidden. Focused edit harness: `12/12`;
the current direct document suite is `54/54`. Status:
`whole_string_edit_diff_avoided / exact_intent_focused_harness_passed /
surface_property_exclusive_prepare_implemented / document_store_exclusive_prepare_implemented /
dual_commit_coordinator_and_product_binding_implemented_unvalidated /
document_harness_54_of_54_passed /
product_thresholds_model_rebase_and_managed_profile_power_wgpu_pending`.

## 2026-08-28 UI input document-session integration boundary

**Status:** `manager_document_session_gateway_implemented_unvalidated /
steady_edit_avoids_full_source_diff /
committed_source_epoch_implemented /
accessibility_text_actions_document_owned_unvalidated /
topology_gated_detached_owner_reclamation_implemented_unvalidated /
surface_instance_session_identity_implemented_unvalidated /
multi_document_store_accounting_profiled_and_optimized /
managed_runtime_power_wgpu_pending`.

The production `UiInputManager` path now owns one retained document session rather than placing
document authority in clone/serde `UiSurface` or a process-global service. The first edit for one
editable node opens the current committed source once; subsequent synchronized edits qualify reuse
with `(tree, node, committed-source epoch)` and use the exact range/replacement intent. They do not
hash, compare, or diff the complete source per key. Caret, selection, style/layout invalidation, and
IME preedit do not advance the committed-source epoch. A real input commit or accepted external body
replacement advances it once; a mismatch closes the old document and opens a new identity before the
next edit. Public publication is one content-free receipt after both document and Surface preflight.
Keyboard/text, IME commit/delete-surrounding, clipboard cut/paste, and Accessibility TextInput
`SetValue`/`ReplaceSelectedText` now share this boundary. Accessibility selection remains state-only.

Detached editable owners are reclaimed by the manager session. The product Runtime UI pair invokes
owner synchronization before rebuild and dispatch; the stable path compares retained topology
generation/node count, while a topology change scans only the bounded binding set and direct pending
removals close their exact binding. This avoids an unconditional per-frame/per-event `O(D)` scan.
Focus-loss composition is now a state-only cancellation boundary. `UiSurface::focus` restores the
preedit-before source without producing committed intent, advancing source epoch, or publishing a
synthetic document receipt; the manager consumes the exact focus-loss owner set to clear only that
binding's history.

The initial policy uses named, bounded MVP values for document count and current/retained bytes,
replacement input, addition sources, pieces, current snapshots, and active snapshot leases. These
limits are containment defaults, not product-load or Unreal-parity evidence. The completed profile found that
aggregate store reporting, rather than `BTreeMap` lookup or local piece repair, was the dominant
multi-document admission term; retained accounting now makes `report()` constant-time and keeps
totals at successful open, changed edit, first snapshot materialization, and close. The post matrix
is flat at 0.5-0.8 microseconds per 100 reports from one through 1024 documents. Stable 1024-owner
store-edit lanes improve 20.37-23.78 times without an allocation increase. A different UUID index
remains unauthorized because lookup was not the measured bottleneck.

Scoped parsing/formatting, whitespace checks, call-site scans, and a no-production-panic scan pass.
The direct current-source document harness passes 53/53 tests, including incremental residency
lifecycle and dropped-prepared-edit stability.
The single Windows managed validation request was accepted but returned `command_post_timeout`
without a terminal result; it was not polled or resubmitted. Therefore no managed Runtime pass,
power conclusion, WGPU screenshot, or optimality claim is recorded from this integration slice.

## 2026-08-28 Delta document history boundary

Undo/redo is now session-owned and delta-based rather than a retained sequence of complete
`UiEditableTextState` or flattened document snapshots. Each normal commit copies only its exact
removed range from revision-checked piece storage and its inserted range from the already-final
state. The 1 MiB history budget is checked from exact range/replacement lengths before reading the
removed range, so a large barrier edit does not perform a throwaway source copy. Stack work is
amortized constant-time and bounded by 100 entries; new edits drain at most that bounded redo set.

Secure edits retain no history delta. Source epoch rebind, owner detach, tree/Surface identity
change, and secure synchronization release history with its binding. Undo/redo still pass through
the same document and Surface preflight, and stacks move only after a changed dual commit. The
content-free receipt carries typed `Undo`/`Redo` without removed or inserted bytes.

The direct history harness passes `3/3`; the document harness passes `54/54` with range reads across
piece boundaries and stale-revision rejection. Product route tests are written but remain managed-
Runtime unvalidated. The 100-entry/1-MiB policy is containment, not measured product tuning; its
allocation/RSS/p50/p95/p99 matrix, power trace, and WGPU acceptance remain open. Status:
`delta_history_implemented_unvalidated / full_snapshot_history_rejected /
oversized_copy_avoided / focused_history_harness_3_of_3 /
document_harness_54_of_54 / managed_profile_power_wgpu_pending`.

## 2026-08-28 Focus-loss composition ownership boundary

Focus loss no longer converts transient preedit into a hidden committed document edit inside
`UiSurface`. Cancellation restores only the composition range through the existing property
transaction and supplies no `CommittedTextEditIntent`, so document UUID/revision authority remains
entirely in the manager session and the source epoch remains unchanged. This avoids an otherwise
mandatory full document rebind after every focus-loss composition.

Undo history invalidation consumes the already-emitted input-method Disable lifecycle instead of
scanning focus state or the UI tree. Pending programmatic lifecycle requests are inspected before
dispatch/tick, and requests already projected into a dispatch result are inspected afterward. Each
Disable performs one owner-key `BTreeMap::remove`, `O(log D)` in active document bindings; no frame
timer, whole-tree walk, source flatten, or steady-state history allocation is added.

An E-drive current-source executable passes the focused cancellation path `1/1`; scoped formatting
and whitespace checks pass. Product Runtime tests, platform IME behavior, allocation/power evidence,
and genuine WGPU acceptance remain open. Status:
`focus_loss_state_only_cancel_implemented_unvalidated /
hidden_document_rebind_removed_unvalidated /
lifecycle_keyed_history_invalidation_implemented_unvalidated /
focused_current_source_1_of_1 / managed_runtime_power_wgpu_pending`.

## 2026-08-28 Clipboard host bridge boundedness review

The clipboard path now crosses Runtime UI, the versioned dynamic ABI, App, and the exact originating
Surface as one typed transaction. Each Surface manager queue is capped at 256 rows and retains only
the newest undrained request per owner. Dynamic collection is `O(S + R)` for `S` declared surfaces
and `R` queued rows; host output retains its existing 256-row transactional paging. The 32 KiB UTF-8
body cap keeps worst-case JSON control-character expansion below the 256 KiB host/event envelope.
Windows platform work is `O(B)` in the UTF-16/UTF-8 body length and uses the real target HWND;
non-Windows backends return typed `Unsupported` rather than claiming success.

This is a bounded correctness bridge, not a measured optimization. It does not change shaping,
layout, raster, or WGPU output, so no screenshot is produced for this slice. Scoped formatting,
whitespace, source ownership and payload-envelope contracts are present; managed Cargo, real Windows
clipboard fault injection, latency/allocation/RSS/power sampling, macOS/Linux/Web qualification, and
product WGPU acceptance remain pending. Status:
`clipboard_runtime_app_roundtrip_implemented_unvalidated /
clipboard_queue_and_payload_bounded_unvalidated / algorithmic_cost_review_complete /
managed_system_clipboard_profile_power_wgpu_pending`.

## 2026-08-28 Dynamic UI generic host-reply queue budget

The dynamic Runtime now projects the seven non-IME/non-clipboard `UiDispatchHostRequestKind`
operations into typed `ZrRuntimeHostRequestV1::UiHost` rows instead of discarding them after reducing
the dispatch result to handled state. The queue is bounded at the lowest pending owner by the host
page's 256-row cap, 64 KiB per encoded row and 240 KiB aggregate with 16 KiB envelope reserve.
Admission serializes once for accounting; rejected rows do not enter the pending owner. IME and
clipboard retain their dedicated transaction queues, and internal dynamic reason text does not cross
the ABI. App fallback diagnostics are fixed-kind and logarithmically rate-limited. This is bounded
delivery infrastructure, not a measured optimization: managed allocation/CPU/RSS/power, platform
adapter behavior, Cargo and WGPU/PNG remain open. Status:
`generic_ui_host_reply_queue_bounded_unvalidated / binding_receipt_policy_open /
managed_profile_platform_validation_pending`.

## 2026-08-28 Focused bound model-update boundedness review

The model/edit split reuses the existing per-Surface `UiInputManager` document session. A focused
bound refresh performs one document-key lookup and one owner-key pending insertion/replacement. The
pending owner is a `BTreeMap`, so admission and focus-loss lookup are `O(log E)` for `E <= 256`
editable pending owners; it does not scan source text, layout lines, glyphs, or the UI tree on the
stable topology path. Blur performs a document UUID/revision comparison before any source mutation.
An exact match uses the existing full-replacement document/Surface transaction; a mismatch emits a
fixed-size content-free conflict and performs no replacement.

The containment budget is 256 combined pending/terminal rows, 4 MiB for one request value, and
16 MiB aggregate pending value bytes. One pending row exists per owner. Supersession accounts bytes
before insertion and publishes a terminal receipt for the older request; a latest unchanged refresh
also removes an older pending value. Secure pending text remains in the Surface secure store and is
not duplicated into manager state or receipt/debug output. The store now owns it as
`Zeroizing<String>`: replacement, discard, clear and teardown erase the pending allocation, while an
accepted transfer moves the allocation with `mem::take` rather than allocating another full value.
This does not zeroize the request or the accepted Surface/document state and is not an end-to-end
security or performance claim. These bounds prevent unbounded model
churn while a field remains focused, but they are initial MVP limits rather than measured product
tuning.

Sixteen fixed, content-free `ui_text.model_update.*` counters now record request bytes, bound versus
explicit origin, focused and secure classification, each receipt status, pending admission/release
bytes, and supersession. Counter names never include request/tree/node identity or source text. The
queue, transaction, and profile owners are 535/282/137 lines respectively; measurement did not add a
second queue, source snapshot, timer, or dynamic profile label.

No optimization claim is made for the accepted replacement itself: current Surface state/property
projection still owns full `String` values and remains `O(N)`, as recorded by the existing
`ui_text.edit` profile counters. Managed 1/100/1k/10k refresh/defer/blur/conflict allocation,
p50/p95/p99, RSS and power capture, matched Unreal behavior, Runtime tests, and WGPU acceptance remain
open. Status: `focused_model_update_queue_bounded_unvalidated /
steady_focus_refresh_avoids_source_scan_unvalidated /
secure_pending_text_not_duplicated_unvalidated /
secure_pending_drop_zeroization_implemented_unvalidated /
persistent_secure_document_zeroization_open /
surface_full_value_projection_profile_pending /
managed_runtime_power_wgpu_pending`.

## 2026-08-29 Retained grapheme authority product wiring

The max-grapheme constraint path now asks the retained `TextDocument` authority for the number of
graphemes outside the exact replacement range. The query is fenced by document UUID/revision in the
store and by tree/node/source epoch in the Surface session. The source index requires both range
ends to be exact Unicode grapheme boundaries and uses two binary searches over the revision-owned
boundary vector; a split cluster returns `InvalidGraphemeBoundary` instead of guessing an offset.

Keyboard text, text events, IME preedit/commit, and accessibility selected-text replacement pass the
document-owned count into the existing sanitizer. Accessibility whole-value replacement supplies the
mathematical retained count of zero; selected replacement with an active transient composition uses
the visible-source fallback because its accessibility range is not a committed-document coordinate.
Test and unmanaged call paths retain the same explicit fallback; it is no longer an implicit second
cache. If a session query cannot be admitted or matched, the fallback preserves current input
behavior and is separately observable.

Index-triggered flattening now uses the store's existing current-snapshot admission limit and updates
its residency receipt. A denied query does not flatten or mutate the document. Eight fixed
`text_document_grapheme_*` counters separate query time, binary searches, warm hits, rebuild count,
input bytes, boundary count, and rebuild time. Three fixed `text_input_grapheme_*` counters separate
document-index use from source fallback and report only the bytes actually scanned outside the
replacement range. `Instant` is called only while a profiling capture is active; no text, owner,
document, range, or dynamic label enters diagnostics.

This is the prerequisite product wiring requested by the earlier constraint review. At the time of
this record it was not an incremental-index optimization or a measured performance result. The
managed matrix described below remains the acceptance gate; the follow-up implementation now
preserves the index for a conservative ASCII/no-CRLF subset while Unicode-sensitive edits still
fall back to a full rebuild. The next authorized step is the managed
1/100/1k/10k tail/middle/selection/preedit matrix with allocation, RSS, input latency, index repair,
fallback rate, and valid power data. Only that evidence may select incremental boundary repair or a
different retained representation. Status:
`retained_grapheme_authority_product_wired_unvalidated /
revision_epoch_and_snapshot_admission_fenced /
fixed_profile_receipts_implemented /
ascii_incremental_splice_static_implemented /
unicode_sensitive_rebuild_preserved /
retained_incremental_validation_open / managed_profile_power_wgpu_pending`.

## 2026-08-30 Conservative grapheme index splice

The retained `TextDocumentSourceIndex` now has a checked incremental path for edits whose old
range, replacement, and one-grapheme context on either side are all ASCII and contain no CR/LF.
The prepared edit is tied to the current document revision and exact cached boundary indexes.
Commit splices the replacement boundaries and shifts the retained suffix with checked arithmetic;
any stale revision, non-boundary range, Unicode context, CR/LF context, or capacity/range failure
rejects the splice and preserves the existing full-rebuild fallback. Failed preconditions are not
reported as successful incremental updates.

This is a structural baseline implementation, not a performance claim. The splice work is
`O(old_range_bytes + replacement_bytes + suffix_boundary_count)`, while piece-backed local context
extraction can still scan the retained piece prefix. End-to-end edit latency, allocation, RSS,
power, and fallback rates therefore remain open until the managed matrix runs. Status:
`ascii_incremental_index_splice_static_implemented /
unicode_and_crlf_fallback_preserved /
fixed_incremental_profile_counters_implemented /
managed_profile_power_wgpu_pending`.

2026-08-30 preflight allocation correction: ASCII/no-CRLF admission now walks the existing piece
bytes directly instead of materializing a temporary context `String`. The helper validates checked
piece bounds and requires the complete requested window to be covered; a malformed piece map or
separator byte rejects admission and leaves the full-rebuild fallback intact. This removes one
local preflight allocation from the eligible edit path, but does not change the splice gate or add
an end-to-end performance claim. Status:
`ascii_incremental_preflight_allocation_free_static_implemented /
piece_coverage_fail_closed / managed_profile_power_wgpu_pending`.

The companion Python source contract `test_runtime_text_document_incremental_index_contract.py`
passes 4/4, covering no flattening in incremental admission, checked piece-byte coverage, fixed
profile names, and explicit Unicode/CRLF fallback tests.

## 2026-08-29 FontObject cache and generation identity

Font-asset owner mapping is now a shaping/layout render input. Attaching or removing a second logical
owner advances the shared font generation even when its physical face bytes are deduplicated and remain
alive through another owner. This invalidates stale shaped, fallback and line-metric results without
retiring the shared face. `ShapedRunCacheKey` stores `font_asset` and `font_family` separately; fallback
query identity includes the exact owner, so same-name typefaces from different FontObjects cannot alias.

Asset CompositeFont indexes are compiled with runtime/project indexes when the generation is published
and retained as `Arc`; requests perform O(1) owner lookup rather than hashing the authored sub-font table.
Owner candidate construction always prefers local same-family faces. A missing request typeface is local-only;
only CompositeFont, asset fallback, and base/platform declarations may search global faces when the owner does
not provide that family. No per-glyph allocation, URI parsing, filesystem access, new coverage pass,
or cache lock was added. This is a structural correctness correction, not a measured optimization. Rustfmt
and static scans pass; managed 1/100/1k/10k timing, cache-lock profile, RSS/power, Cargo and WGPU/PNG remain
pending. Status: `font_object_cache_identity_static_implemented /
generation_compiled_asset_composite / performance_claim_profile_gated`.

Unavailable explicit owners use a request-local `Cow<FontQuery>` constraint. The common registered-owner
and empty-family paths remain borrowed and allocation-free; only an unavailable owner with a non-empty
owner-local family clones the small query once and clears its family vector before shaping/metric/SDF
resolution. Different unavailable asset URIs may conservatively remain distinct shaped-cache keys, while
their fallback/metric output can share the default-chain cache identity because the constrained render input
is identical. No timing or allocation claim is accepted until the managed profile gate runs.

Family-source scope survives normalized deduplication in one O(n) HashMap-indexed pass. If a local query name
is repeated by an authored external fallback, the existing entry is upgraded instead of duplicated; otherwise
it remains owner-only. This replaces an implicit global retry without increasing the candidate count or adding
another coverage pass.

The FontObject owner state also retains its ordered faces as a generation-local `Arc<[FontFaceId]>`.
Primary, scoped fallback, and line-metric queries borrow this immutable slice instead of reconstructing a
temporary owner-face vector through source-key hash lookups. This removes a newly introduced intermediate
materialization from the request path; no p50/p95, allocation, RSS, or power claim is accepted before the
managed profile matrix runs.

The packaged last-resort face is a generation-owned render input. Changing it detaches face-match,
fallback-resolution, candidate, and line-metric caches; shared database equivalence includes the exact face.
Normal covered clusters add no candidate visit or source lookup. Only the already-terminal missing path performs
one O(1) option read before shaping through the engine-owned face. Timing, raster, RSS, and power claims remain
gated on managed evidence.

Shared font publication now retains `Arc<FontDatabase>` and canonical shaping acquires an O(1)
`FontCollectionSnapshot` lease instead of cloning the complete database. Owned clones remain only for legacy
mutable renderer consumers and have a separate `shared_owned_snapshot_clone` profiler span. Cosmic locale
FontSystem refresh consumes the supplied snapshot and uses generation, not Arc identity, as its invalidation
key; diagnostic-only equivalent publication therefore does not rebuild up to four locale systems. Arc reuse,
retired-snapshot lifetime, and explicit old-snapshot cosmic binding have source regressions. No clone-count,
p50/p95, RSS, power, or Unreal same-load claim is accepted before the managed matrix runs.

The handle registry, immutable resolver publication, and counters are now collection-owned rather than separate
process globals. Registration and resolution remain batched `O(U + N)` operations over unique pairs and projected
glyphs. A UI raster artifact acquires one database Arc and one registry Arc, then resolves old-generation handles
without a lock or current-generation reprobe while that in-flight lease lives. SDF bake and its font-asset subcache
probe only their renderer-owned collection. These are structural complexity claims with source regressions;
allocation counts, p50/p95, RSS, package power, and Unreal same-load comparison remain pending.

The 2026-08-29 owner-ready continuation removes the remaining split-generation hot path from retained UI
layout. `UiTextMeasureCache` owns one collection-bound `SharedTextLayoutSession`; size/layout keys, viewport
certification, physical and virtual fragment reuse, layout publication fences, render-command artifact refresh,
and `UiSurface` invalidation all read that owner. A foreign collection mutation is therefore O(1) irrelevant to
the surface, while an owned mutation invalidates the retained caches without constructing a second session.
Resolved artifacts capture their immutable database and resolver publications once after handle registration;
line acquisition is O(1) Arc cloning and does not lock or re-probe current global state. No per-glyph loop, extra
coverage pass, or source copy was added. These are source-level complexity claims only: Cargo, 31-sample cold/
warm timing, allocation/RSS, WPR power, same-load Unreal comparison and real Native/SDF WGPU screenshots remain
pending. Status: `collection_bound_retained_layout_and_artifact_lease_static_implemented /
performance_claim_profile_gated / managed_validation_pending`.

## 2026-08-29 Screen-space renderer collection ownership

The screen-space render path now accepts the same explicit `FontCollectionService` at
`ScreenSpaceUiRenderer`, `ScreenSpaceUiTextSystem`, and `TextRenderState`. The existing constructors remain
process-default adapters only at the renderer boundary. Process task-pool sizing is reused without selecting a
process font collection, so worker budget policy and font-resource ownership no longer imply each other.

Renderer plan and retained text-segment cache keys now store `FontCollectionRevision(collection_id,
generation)`. Artifact reconciliation compares the artifact's immutable font lease against that revision before
accepting glyph IDs. Equal generations from different collections therefore rebuild/reject instead of aliasing.
The plan key reads the collection's currently published atomic revision; raster/segment publication reads the
revision already adopted by `TextRenderState`, avoiding a new-publication key paired with an old database.

This adds one collection-id word to each generation-sensitive plan/segment identity and preserves O(1) cache-key
comparison. The existing batch admission scan remains O(text batches); no extra shape, coverage pass, database
clone, source copy, registry lock, or per-glyph work was introduced. Same-generation foreign-collection tests are
present for plan cache, segment cache, and artifact admission. Rustfmt, scoped diff-check, production global-probe
scan, conflict scan, and owner line budgets pass. Managed Cargo, 31-sample CPU/allocation/RSS, WPR power, matched
Unreal workload, and real Native/SDF WGPU PNG remain pending. Status:
`screen_space_collection_injection_static_implemented /
renderer_revision_cache_identity_static_implemented /
core_manager_window_pie_wiring_open / managed_validation_pending`.

2026-08-30 Core manager injection recheck：Graphics module descriptor 已声明对
`TextModule.Manager.FontServices` 的 manager dependency；生产 `create_render_framework_with_render_features`
从 `CoreHandle` 解析 Core-owned `FontCollectionService`，并沿
`WgpuRenderFramework -> SceneRenderer -> ScreenSpaceUiRenderer -> ScreenSpaceUiTextSystem -> TextRenderState`
的显式构造边界传递。新增静态回归检查锁定该生产链不读取 process-global collection；全局构造器仅作为
renderer/bootstrap compatibility adapters 保留。该切片不改变渲染算法或热路径复杂度，窗口/PIE owner 接线、
受管 Cargo、真实 WGPU/PNG、31-sample CPU/allocation/RSS、WPR power 与匹配 Unreal 负载仍待验证。状态：
`core_manager_font_collection_injection_static_implemented / window_pie_wiring_open /
managed_validation_pending`。

同日动态 Runtime UI 复核确认：`RuntimePreparedProject::load_runtime_ui_surfaces(&CoreHandle)` 将
Core-owned collection 传入 `RuntimeUiSurfaceSet::load`，再由 `UiV2SurfaceBuilder` 传到
`UiSurface::new_with_font_collection`；`UiSurface::new` 和旧模板 builder 的 process-default 访问未被该
动态 Runtime 生产链调用。`UiSurface::new` 仍服务于单一进程 owner 的 Editor host 与 standalone 调用，
不能据此让 Runtime Core/session 回退到进程集合。

## 2026-08-29 Runtime font asset claim/release lifecycle

Text Core now owns the runtime font asset residency contract. `RuntimeFontAssetClaimScope` is a non-Clone RAII
scope backed by `FontCollectionService`; dynamic Runtime UI claims its deduplicated default/explicit dependencies
before first layout, while the screen-space renderer reconciles its current dependency set before refreshing
`TextRenderState`. Aggregate claim counts allow HUD, menus, and renderer consumers to share one owner without
premature removal. Only the last release retires newly unclaimed owners, and all such removals are committed and
published as one database mutation; the packaged default family/composite is restored in the same transaction.
Renderer local ready/missing/error records are pruned when a dependency is released, so a later project/session
claim retries admission instead of reusing a negative record for a retired owner.

The steady renderer path is source-constrained to one dependency-length check and `HashSet` membership probes:
it does not acquire the claim mutex, allocate `String`/`Arc`, clone the database, publish a generation, or perform
glyph work when dependencies are unchanged. Changed dependencies are `O(D)` Arc/hash reconciliation; a final
release performs one collection publication plus `O(R)` removal for released owners; owner-local registration
staging clones remain and must be measured separately. These are complexity contracts, not measured performance
results. The mandatory profile matrix remains: dependency counts 0/1/8/64,
600 stable frames, two scopes sharing one owner, project switch, hot reload, and release/re-admit. Record claim
lookup, allocations, mutex/DB clone counts, generation publications, p50/p95, RSS, and package power. Expected
stable counters are `added=0`, `released=0`, `unclaimed=0`, and `font_inputs_changed=0`; no dynamic profile has
run. Release plus all changed/new admissions now share one collection mutation/publication; the remaining work is
managed residency and performance evidence. Status:
`collection_owned_runtime_font_claim_ledger_static_implemented /
stable_path_allocation_and_lock_free_by_source / release_plus_admission_single_publication_static_implemented /
managed_profile_and_product_validation_pending`.

## 2026-08-29 Font publication clone-boundary profile plan

The current publication path has three distinct copy boundaries that must not be conflated:

1. `FontCollectionService::mutate` clones the published `FontDatabase` to create the next mutable
   generation. This copy is required while immutable snapshots may still be leased.
2. `FontDatabase::replace_asset_registrations` creates an owner-local staging clone before applying
   registration changes. That clone preserves the direct `FontDatabase` error-atomicity contract,
   but is nested inside the outer collection transaction for runtime asset admission and may be
   structurally redundant when the outer transaction already owns the next database.
3. Legacy mutable renderer consumers request an owned `FontDatabase` after publication. They may
   need that ownership, while claim/admission and projection-only callers do not.

Before changing the registration API, the managed Windows profile must capture each boundary separately:
`text.font_database.shared_mutation`, `text.font_database.owner_registration_staging_clone`,
`text.font_database.shared_owned_snapshot_clone`, plus fixed counters for before/after face count and
clone face count. Run dependency counts 0/1/8/64, first admission, repeated equivalent admission,
replacement, last-owner release, and release plus changed/new admission in one transaction. Use 31
sample cold/warm runs, p50/p95/p99 CPU time, allocation count/bytes, peak RSS, generation publications,
and package power. The stable renderer frame must show zero staging/owned clone spans; the changed path
must show one outer publication clone, and an owner registration must report whether the nested staging
clone is present. Compare the same workload against the Unreal cache flush/update path before selecting
an API cutover. No performance or power claim is accepted from source inspection alone.

The non-validation implementation may expose a collection-owned published `Arc<FontDatabase>` result
for callers that only need the mutation receipt. This removes the post-publication owned clone without
changing the snapshot lifetime contract. The owner-local staging clone remains behind its own profile
boundary until the managed evidence proves that an in-place transactional registration API preserves
error atomicity and cache invalidation semantics.

Status: `clone_boundary_profile_plan_written / published_arc_receipt_api_static_implemented /
owner_staging_api_profile_gated / managed_profile_and_power_pending`.

The former single-asset renderer load/resolve path and standalone runtime admit/retire wrappers are
deleted. The remaining test-only ensure helper invokes the production batch owner
(`refresh_font_asset_records`) with an isolated collection and a real claim scope, so tests cannot
silently preserve a second per-asset mutation/publication contract.

An equivalent render-input mutation still publishes its derived-state candidate even when generation is
preserved. `FontDatabase` contains generation-excluded instance/cache/diagnostic state; blindly reusing the
old Arc would discard a generic mutation to that state and could orphan a returned instance identity. A
future no-op publication fast path therefore requires a narrower typed mutation contract, not another
`has_same_render_inputs` branch in the generic publisher.

## 2026-08-29 SDF font admission ownership review

The current-source call graph showed that production SDF preparation is reached through the same
screen-space `TextRenderState` after renderer dependency admission and collection refresh. The SDF
face cache nevertheless reparsed an asset manifest and mutated only the render state's owned database
when an asset mapping was absent. That created a second font loader after shaping: raster could observe
a face that was never published by the collection and was unavailable to shaping/layout or another
consumer of the same generation.

The runtime SDF face cache is now lookup-only. It resolves an already admitted asset primary face, then
the packaged runtime default face, and otherwise records `NoRegisteredFaces`; it performs no manifest
I/O, source decode, database registration, owner removal, or collection publication. Source/decode/budget
diagnostics remain owned by the upstream batch admission cache. Offline `.zsdf` manifest/artifact reads
remain in `offline_source.rs`; the direct source registration helper used to prepare offline fixtures is
compiled only for tests. This removes one unbounded source-I/O/registration branch from raster work and
prevents shaping/raster database divergence. It is an architectural complexity correction, not measured
latency or power evidence. Seven SDF-local source/decode/budget/registration counters that became
permanently zero were removed from the bake/prepare/profile DTO chain; SDF retains only total unresolved
asset count and `NoRegisteredFaces`, while detailed load failures remain at the admission owner.

Static ownership contracts pass 9/9, rustfmt passes, and scoped diff-check is clean. Managed Windows
validation returned `cargo_reuse_pool_busy` before Cargo started, so no compile/test, WGPU screenshot,
allocation/RSS, 31-sample latency, Unreal matched-load, or WPR power result is claimed. Status:
`sdf_face_cache_lookup_only_static_implemented / duplicate_raster_font_loader_deleted /
managed_validation_profile_power_and_product_png_pending`.

## 2026-08-29 Session-bound shaping entrypoints

Production paragraph prewarm now enters only through
`shape_paragraphs_with_cache_in_font_collection(...)`, carrying the layout session's explicit
`FontCollectionService` through cache lookup, worker shaping, and Ready admission. The process-default
wrapper is compiled only for focused tests, and the unused process-default finish wrapper was deleted.
Together with the caller-snapshot Cosmic cache initialization and the published-snapshot receipt used
for renderer system-font opt-in, the production parallel path no longer
contains an implicit process font-collection lookup. Static ownership contracts pass 12/12; Cargo,
multi-session corpus shaping, WGPU/PNG, and profile/RSS/power validation remain pending.

## 2026-08-30 Dynamic fallback UI collection identity

The no-project HUD/menu fallback extract path now shares the dynamic session's Core-owned font
collection. Session construction resolves that service once after module activation and gives the
same `Arc` to project surfaces and `RuntimeUiExtractCache::new_with_font_collection`. The cache key
reads the generation from its retained `UiTextMeasureCache` layout session; production no longer has
`RuntimeUiExtractCache::default` or calls `current_resolved_text_font_generation()`.

This makes Core font publication invalidate its own fallback extract and prevents an unrelated
process-default mutation from causing a false rebuild. It changes ownership and invalidation
correctness, not shaping/layout complexity. Static ownership suite 18/18, rustfmt and scoped diff
checks pass. An independent-collection generation/rebuild Rust regression is written but has not run.
Multi-Core isolation, font-publication rebuild behavior, Cargo, project/no-project
WGPU/PNG, 31-sample latency/allocation/RSS and package power remain pending. Status:
`fallback_ui_extract_core_collection_bound_static_implemented /
cross_collection_invalidation_removed / managed_validation_pending`.

## 2026-08-30 Surface input geometry collection identity

The retained Surface already owned a Core-selected layout session, but missing-artifact caret,
selection, IME-rectangle, and pointer hit-test recovery constructed the process-default direct shape
provider. That allowed one input query to combine a layout from collection A with source metrics from
collection B. `FontCollectionTextShapeRunProvider` now accepts an immutable
`FontCollectionSnapshot`; Surface input captures its measure-cache snapshot once per IME context
refresh or pointer hit and carries it through the geometry boundary. Process-owner public compatibility
helpers take their own process snapshot explicitly. Because the neutral layout DTO has no collection
revision, a generation mismatch rejects source reshaping and consumes the published artifact/glyph
advances until normal layout recomputation.

This follows Unreal's `FSlateFontMeasure` ownership of the exact `FSlateFontCache` used by measure and
character-offset queries. It is not a shaping algorithm optimization: no cache size, line-break,
cluster, or glyph loop changed. The added hot-path work is one Arc-backed snapshot lease per input
query, not per glyph; the existing direct shape already acquired a shared snapshot. The Rust regression
requires every shaped face handle to retain the injected collection id and reports the injected
revision, but it has not run because managed Cargo remains unavailable. Static ownership suite 19/19,
rustfmt, and scoped diff-check pass. WGPU/PNG, IME/pointer product input, 31-sample latency/allocation,
RSS, and package power remain pending. Status:
`surface_input_geometry_collection_bound_static_implemented /
cross_collection_metric_recovery_removed / managed_validation_pending`.

## 2026-08-30 One-shot text provider snapshot ownership

兼容性 `DirectTextShapeRunProvider` 以前是零状态对象，每次请求都重新读取进程字体集合；在
多行 measure/断行期间如果发生字体 publication，同一操作可能前后使用不同 generation。现在
provider 创建时固定一个不可变 `FontCollectionSnapshot`，revision 查询与 horizontal/vertical
shape 都复用该 snapshot，并继续走 canonical diagnostics entrypoint。Runtime retained 路径仍以
`SharedTextLayoutSession`/显式 collection provider 为 owner，未新增第二套 shaper。

该切片对齐 Unreal `FSlateFontMeasure` 保留具体 `FSlateFontCache` 的 owner 语义，是正确性边界
修复而非性能优化。20/20 静态 suite、Python compile、rustfmt 和 scoped diff-check 通过；
publication 中途变化的 Rust face-identity 回归已写但未运行。Cargo、真实 WGPU/PNG、IME/pointer、
31 样本 profile/RSS/power 与 Unreal 对拍仍待 managed validation。状态：
`one_shot_provider_snapshot_bound_static_implemented / cross_generation_mix_removed /
managed_validation_pending`。

## 2026-08-30 Compiled rich cluster ownership profile

`CompiledRichText` previously constructed and retained one `(u32, u32)` range for every grapheme in
the complete visible document even though no production consumer read it. A 31-sample E-drive release
microprofile of that exact expression measured 1/8/32 MiB ASCII payloads at 8/64/256 MiB and p50
65,236/736,093/3,074,179 us. Unreal's rich parser contract publishes stripped output plus line/run
ranges and metadata; character breaking remains in shaping/layout. Zircon has therefore hard-cut the
field, full-document segmentation pass, equality/accounting entries, and accessor rather than hiding
the duplicate owner behind another quota or lazy cache.

The removed owner now has exact payload zero and no replacement stage. Runtime Text static contracts
pass 34/34, targeted rustfmt/source/diff guards pass, and no `cluster_ranges` reference remains. This
does not establish end-to-end parser/layout/frame or package-power performance. Managed Cargo, real
WGPU/PNG, allocation/RSS/power, and matched Unreal-load validation remain pending. Status:
`rich_compiled_duplicate_cluster_owner_removed_static / isolated_profile_recorded /
managed_product_validation_pending`.

## 2026-08-30 Rich table projection structural profile

Compiled rich-table construction previously rescanned every run, paragraph, and table for every
cell. The E-drive 31-sample release baseline at 4,096 objects performed 50,331,648 comparisons for
8,192 emitted indices and measured p50/p95/p99 60,544/85,779/123,556 us. The owner now builds three
request-local balanced interval trees with subtree `max_end`; canonical parser output uses a linear
order check before tree construction, defensive out-of-order constructor input falls back to sorting,
and the trees are dropped before artifact publication. UI consumes the checked source-order output
without another sort/dedup pass.

The same isolated final-path lane measured 215,046 entered interval nodes and p50/p95/p99
3,337/4,467/5,611 us at 4,096 objects, improving p50/p95 by 18.14x/19.20x. Across 256 to 4,096
objects, p50 growth reduced from 260.97x to 22.70x. This confirms removal of the isolated quadratic
rescan, but the first-sample working-set delta increased from 208,896 to 360,448 bytes because the
temporary trees exchange memory for time. Run/paragraph/table/cell/projection and BBCode block/table
depth budgets now fail typed before owner growth. The current reproducible Runtime Text static suite
passes 34/34; managed
Cargo, real table layout, allocation/RSS, package power, WGPU/PNG, and matched Unreal-load evidence
remain pending. Status: `rich_table_projection_interval_owner_static_implemented /
isolated_quadratic_bottleneck_removed_profiled / managed_product_validation_pending`.

## 2026-08-30 Rich decorator exact-tag dispatch profile

Zircon decorators uniquely claim one normalized tag at registration, but the old parser-local registry
performed a full vector scan for every candidate open token. The E-drive release baseline used 4,096
final-tag hits per sample and 31 samples: 16/256/4,096 decorators measured p50
517/7,381/116,314 us. Decorator count grew 256x and p50 grew 224.98x. Local Unreal's marshaller scans
because `ITextDecorator::Supports` is an arbitrary predicate; Zircon's exact-tag contract has no such
ordering semantics.

`DecoratorRegistry` now has one keyed owner and uses one `Entry` operation for registration
admission/insertion plus borrowed lookup for dispatch. Existing parser-local generation and compiled
cache identity remain unchanged. The same indexed lanes measured p50 140/142/139 us; at 4,096
decorators p50/p95 improve 836.79x/1,040.07x and the decorator-count-dependent dispatch slope is gone.
The timed working-set delta excludes registry construction and is not retained-memory evidence.
The current reproducible Runtime Text static suite passes 34/34. A subsequent static
provider-admission slice catches callback
unwind as a tagged typed failure and adds default 64 KiB per-call decorator metadata plus 32 MiB
request-retained run metadata caps before publication. Its Rust behavior tests are written but unrun.
Decorator callback deadline/cancel/private allocator quota, provider leases, registration count,
managed Cargo, registry allocation/RSS, package power, WGPU/PNG and matched Unreal load remain pending.
Status: `rich_decorator_exact_tag_hash_dispatch_static_implemented /
isolated_linear_dispatch_bottleneck_removed_profiled /
decorator_panic_and_metadata_admission_static_implemented / managed_product_validation_pending`.

## 2026-08-30 Rich owned parse clone hard cut

Production consumer tracing found that Runtime UI/layout already retains `Arc<CompiledRichText>` and
that the public `RichTextParser::parse()` path had no production caller. The path nevertheless cloned
the complete parsed payload after every canonical compile/cache lookup, duplicating runs, paragraphs,
tables, and dynamic metadata into a detached owner whose identity no longer included the source and
parser generation held by its parent artifact. Unreal's rich-text marshaller consumes parser output
directly while constructing layout runs; it does not publish a second partial deep-copy artifact API
after a canonical owner already exists.

Before the cutover, an E-drive 31-sample release clone profile measured 12,355/98,819/395,267
allocations and 1,014,784/8,118,272/32,473,088 requested bytes at
4,096/32,768/131,072 runs. The corresponding p50 values were 2,454/22,059/111,366 us; the largest
lane measured p95/p99 232,754/331,802 us and a first-sample working-set delta of 40,169,472 bytes.

Production now exposes only `compile() -> Arc<CompiledRichText>`. Parsed data is borrowed through the
retained parent artifact; the owned parse helper and crate bridge exist only under `cfg(test)` for the
parser corpus. There is no compatibility alias, second cache, or detached snapshot. The removed
production post-compile stage has exact allocation and byte cost zero because it no longer exists.
The current reproducible Runtime Text static suite passes 34/34, with rustfmt and scoped source/diff
guards passing. Managed
Cargo, external downstream migration, end-to-end layout/frame profiling, RSS/power, real WGPU/PNG,
and matched Unreal-load evidence remain pending. Status:
`rich_owned_parse_clone_hard_cut_static_implemented /
immutable_compiled_artifact_public_owner_converged / isolated_clone_profile_recorded /
managed_product_validation_pending`.

## 2026-08-30 Rich parser generation exhaustion boundary

The compiled-rich cache key previously reused identities after representable exhaustion:
parser identity used `fetch_add().max(1)`, while decorator and emoji generations wrapped zero back to
one. Registration also mutated provider state before advancing its generation, so checking only after
the mutation could publish new behavior under an old cache key.

Parser identity now uses a nonzero optional owner state and atomic `fetch_update + checked_add`.
Exhaustion is terminal and typed before source/cache work. Decorator/emoji next generations are
checked before registry mutation; `u64::MAX - 1` may advance to `u64::MAX`, and the following mutation
returns typed `GenerationExhausted` without changing owner state. This is a cache correctness repair,
not a performance optimization, so no latency/power claim is made. The current reproducible Runtime
Text static suite passes 35/35 and source guards confirm no `fetch_add`/`wrapping_add` remains in the
owner. Owner-local Rust boundary tests are written but unrun. RuntimeRichTextService ownership,
provider revoke/leases, targeted generation retirement, managed Cargo, WGPU/PNG, RSS, and package
power remain pending. Status: `rich_parser_non_reusing_generation_static_implemented /
cache_identity_wrap_alias_removed / managed_product_validation_pending`.

## 2026-08-30 Surface-session compiled-rich cache ownership

Current-source tracing showed that every production rich compile consumer already sits below one
`SharedTextLayoutSession`, while a separate process-static parser and compiled cache mixed unrelated
Surface lifetimes, LRU pressure, counters, and clear behavior. Local Unreal keeps decorator/parser
state with the widget/marshaller owner; it does not publish one independent process-wide compiled
rich cache. The repository interface roadmap also does not justify adding a new application singleton
for a concrete Runtime UI object.

The production static cache/free compile/lookup/report boundary is now hard-cut. Each
`RichTextParser` owns one `CompiledRichTextCacheOwner`; the retained Surface session owns the parser
and is passed through layout, measurement, prewarm, retained-document, and render preparation.
Profiling samples that exact session report. A cfg-gated static parser remains only for the test
corpus. Independent session construction produces independent artifacts, and clearing one owner does
not remove another owner's residency.

The combined static suite passes 36/36; the Rust isolation test is written but unrun while managed
Cargo is bypassed. Targeted formatting of the extracted test module and scoped diff checks pass.
This is lifecycle/correctness work, not a measured optimization: multi-Surface allocation/RSS,
contention, latency, package power, WGPU/PNG, and matched Unreal-load evidence remain pending. Status:
`compiled_rich_surface_session_owner_static_complete / process_global_cache_removed /
managed_product_validation_pending`.

## 2026-08-30 One-shot layout-session current-source audit

P1-14 was re-audited before optimization. Runtime `UiSurface` and dynamic HUD/menu product paths now
retain their text cache/session and Core font collection; the apparent HUD/menu default-cache
constructions are cfg-test helpers. The remaining one-shot constructors belong to explicit public
compatibility/standalone operations, tests, or native framebuffer validation. Repository tracing
finds zero retained product-frame callers rebuilding them per owner/frame.

Unreal likewise retains `FSlateFontMeasure` under `FSlateFontServices` with per-font caches. Zircon's
product owner now has the same relevant lifetime property, so no TLS/global cache or unmeasured API
rewrite is authorized. A future observed product caller must first publish construction count,
cold/warm latency, allocation/RSS, cache/backend counters, and power evidence, then migrate to an
explicit owner. Status: `P1-14_current_source_hot_path_not_reproduced /
structural_optimization_profile_gated`. See
[`09/2026-08-30-one-shot-layout-session-current-source-audit.md`](09/2026-08-30-one-shot-layout-session-current-source-audit.md).

## 2026-08-30 Surface render-extract text-prewarm owner split

`ui/surface/render/extract.rs` previously mixed render-command orchestration with popup anchor
resolution/tests and owner-text prewarm request/suppression policy at 965 lines. Popup ownership moved
unchanged to the 208-line `extract/popup_anchor.rs`; owner-text collection moved unchanged to the
158-line `extract/owner_text_prewarm.rs`; the orchestration root is now 632 lines.

The move preserves popup frame validation, component suppression, partial-viewport rejection, overlap
admission, request construction and render-command order. Six moved function/test names remain unique;
the failing-first owner contract, Rust 2024 rustfmt and complete Runtime Text infrastructure static suite
pass 47/47 in 1.744 s. Managed Cargo, runtime prewarm counters, allocation/RSS/power and WGPU/PNG remain
pending. Status: `surface_render_extract_owner_split_static_complete / behavior_unchanged /
managed_validation_pending`.

## 2026-08-31 Text batch owner split

The renderer root's text route selection and `ScreenSpaceUiTextBatch` materialization were moved to
`render/text_batches.rs` after the root approached the repository's structural warning line. The new
owner is 331 lines and the orchestration root is 507 lines; the 711-line `resolved_layout.rs` and the 572-line
`rich_text.rs` share its batch constructor. The owner also holds the command-wide `TextPlanOutcome`,
so plain and rich artifact rejection share one fail-closed decoration policy without a compatibility alias.
This is a responsibility-boundary and correctness change with no algorithm, cache-key,
allocation, latency, RSS, power, or GPU claim. The owner contract is 3/3 and scoped formatting/diff
checks pass. Status: `text_batch_owner_split_static_implemented / non_finite_layout_geometry_rejected /
managed_validation_pending`.
