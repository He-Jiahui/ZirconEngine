---
related_code:
  - zircon_runtime/src/text/cache/mod.rs
  - zircon_runtime/src/text/cache/frame_dedup.rs
  - zircon_runtime/src/text/cache/layout_cache.rs
  - zircon_runtime/src/text/cache/measure_cache.rs
  - zircon_runtime/src/text/cache/shaped_cache.rs
  - zircon_runtime/src/text/cache/tests.rs
  - zircon_runtime/src/text/font/shared.rs
  - zircon_runtime/src/text/font/shared/tests.rs
  - zircon_runtime/src/text/font/database/equivalence.rs
  - zircon_runtime/src/text/sdf/font_bake/tests/cache_generation.rs
  - zircon_runtime/src/text/parallel/mod.rs
  - zircon_runtime/src/text/parallel/shape_pool.rs
  - zircon_runtime/src/text/parallel/raster_pool.rs
  - zircon_runtime/src/text/parallel/tests.rs
  - zircon_runtime/src/ui/text/measure_cache.rs
  - zircon_runtime/src/ui/surface/render/extract.rs
  - zircon_runtime/src/ui/tests/text_pipeline
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_renderer/renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_renderer/tests.rs
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
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

当前完成口径(2026-07-08):`UiTextMeasureCache` 已把 generic `TextMeasureCache`、`TextLayoutCache`、`TextFrameDedup` 与 `ShapedRunCache` 接到生产 UI measurement / full layout 请求路径；同一帧内 exact key + exact text 的重复 natural-size/full-layout 请求先命中 frame dedup,measurement miss 与 full-layout miss 则通过 `TextShapeRunProvider` 共享同一个 `ShapedRunCache`。`text/layout` 的 line metrics、line break、ellipsis、grapheme advances 与 source-range measure 已有 provider path,且普通非 Tab 文本不再为了 tab alignment 额外 shape `" "`。`render_perf_text_measure_then_layout_shapes_once` 在预热稳定 `"Hg"` line metrics 后断言 `editor base.zui` 的 measure+layout 只插入/ miss 一个真实 source shaped run,避免为解决小字号左右落点问题而改用真实字符串行高；`render_perf_text_scroll_list_reuses_cache` 已覆盖滚动列表首段:首屏 5 行各 shape 一次,滚动 3 行后只为 3 个新进入视口的 row 增加 shaped-run miss/insert,重叠 row 必须命中 shaped cache。2026-07-08 `text/parallel/shape_pool.rs` 新增 owned paragraph batch + `TaskPool` 并行 shape 数据面,先查 `ShapedRunCache`,再只把未命中的唯一 paragraph 交给 worker chunk,插入后按原请求顺序返回 `Arc<ShapedGlyphRun>`；同日 follow-up 用显式 `Vec<PendingShapeJob>` pending queue 类型解除验证时暴露的类型推断编译阻塞,`render_perf_text_parallel_shape_count` focused Cargo 已通过 1/1。随后 `UiTextShapePrewarmRequest` 与 `UiTextMeasureCache::prewarm_horizontal_paragraphs(...)` 把这一数据面接到 UI cache owner:可见 editor row 可在布局前按 batch 预热到同一个 `ShapedRunCache`,后续 full layout 不再为这些行重复 shape,且 absolute layout 仍按 frame miss。surface render owner-text 自动收集/调度已接入 `ui/surface/render/text_prewarm.rs`;组件 painter 生成的 `Text` command 现在也在 command generation 后统一 prewarm,并在返回 `UiRenderExtract` 前补齐 `text_layout`,避免 retained-host 继续走裸文本 fallback。rich/vertical 文本预热已通过 `from_layout_source(...)` 关闭。2026-07-10 追加 retained framebuffer proof PNG 像素指标复查,确认 full/narrow label 的 ink coverage 与内部空列稳定,但实时 editor-window typography QA 因当前 active cargo/rustc/link 队列 12-15 暂未启动；同日 `ScreenSpaceUiTextPrepareReport.raster_upload` 已接入 native raster/upload prepare-report 计数 surface。真实 scroll raster/upload 增量断言、live editor-window typography QA、per-page upload merge 和 full glyphon `TextAtlas` cutover 仍 open。

2026-07-10 补记:`scene_renderer/ui/text.rs` 新增 `ScreenSpaceUiTextRasterUploadReport`,由 `text_prepare_report(...)` 汇总 `NativeBitmapAtlasPrepareReport` 与 `GlyphAtlasBitmapRendererPrepareReport` 的 source/cache/worker/upload/requeue/failure 计数。该 report 是 PF-M4 滚动性能断言的中间数据面,不改变 renderer 行为,也不等同于已完成 `render_perf_text_scroll_list_reuses_cache` 的 raster/upload bytes 断言。Focused Cargo 本轮被现有 `zircon_runtime/Cargo.toml` / `Cargo.lock` 不一致阻塞,记录在 `docs/tests/runtime/text/runtime_text_raster_upload_report_cargo_blocker_manifest_lock_20260710.log`。

2026-07-08 补记:`UiTextStyleKey` 已把 `UiTextWritingMode` 纳入 full-layout cache 与 same-frame dedup key。此前 HorizontalTb 与 VerticalRl 在相同 text/frame/style 其它字段下可能共享 `UiTextMeasureKey`,导致竖排 layout 误用水平行缓存。`resolved_layout.rs::style_key_encodes_text_writing_mode` 与 `ui/tests/text_pipeline/measure_cache.rs::text_measure_cache_separates_layouts_by_writing_mode` 覆盖 persistent layout miss 与 same-frame dedup miss。该切片只修 key 维度,不增加 UI/root text facade、字体 token、letter-spacing 或 renderer shortcut。

2026-07-07 补记:`GlyphAtlasBitmapTextureUploadRequestPlan` 已把 page-generation stale/missing 与 face invalidation 从"跳过上传"提升为显式 `GlyphAtlasBitmapRequeuedUpload` 报告。`glyph_atlas_bitmap_texture_upload_request_plan_with_atlas_and_face_validity(...)` 只在 live atlas generation 匹配且 face 仍有效时输出 texture upload request；否则记录 `PageGenerationMismatch`、`MissingPage` 或 `FaceInvalidated`,为 PF-M3 async raster/upload 的 miss/requeue 路径补上可观测数据面。真实 worker、per-face artifact validity source、global glyph slot invalidation 和完整 `TextAtlas` cutover 仍 open。

2026-07-07 补记:`scene_renderer/ui/atlas_texture_upload/frame.rs` 已消费上面的低层 requeue plan,把 `requeued_upload_count`、`missing_page_requeue_count`、`page_generation_mismatch_requeue_count` 与 `face_invalidated_count` 投影进 `GlyphAtlasBitmapTextureUploadFrameReport`。新增 `glyph_atlas_bitmap_texture_upload_frame_plan_for_atlas_and_face_validity(...)`,当 missing page、page generation mismatch 或 face invalidated 出现时帧计划保持 fail-closed,不进入 WGPU texture write。真实 worker、per-face artifact validity source、global glyph slot invalidation、focused Cargo green、live editor-window typography QA 和完整 `TextAtlas` cutover 仍 open。

2026-07-07 补记:`scene_renderer/ui/atlas_renderer/renderer.rs` 已把 renderer prepare telemetry 接上 requeue frame report。生产 bitmap atlas renderer 现在从 `GlyphAtlasBitmapRenderSubmissionPlan.run.atlas` 读取 live atlas generation,调用 `glyph_atlas_bitmap_texture_upload_frame_plan_for_atlas(...)` 后再写 texture；`GlyphAtlasBitmapRendererPrepareReport` 汇总 requeued/missing-page/page-generation-mismatch/face-invalidated upload counters,`upload_failure_count` 也把 requeued uploads 计入失败口径。per-face artifact validity source、真实 async worker、global glyph slot invalidation、focused Cargo green、live editor-window typography QA 和完整 `TextAtlas` cutover 仍 open。

2026-07-07 补记:`GlyphAtlasBitmapRunPlan` 已新增 slot invalidation report,且 bitmap run/render submission/retry driver 可以消费上一帧 `GlyphAtlasSet`。生产 `ScreenSpaceUiTextBackend` 现在跨非空 native bitmap frame 保留主 submission atlas,并在 face invalidation 或 idle native frame 清空；未引用 page 被重建时记录 `GlyphAtlasBitmapSlotInvalidation`、递增 page generation 并整页标脏。该切片关闭主 native bitmap atlas 路径的 slot invalidation state 首段,让 stale upload requeue guard 有真实跨帧 atlas state 可比较；mixed-storage persistent atlas 后续状态见下一条补记,真实 async worker、完整 glyph slot owner、focused Cargo green、live editor-window typography QA 和完整 `TextAtlas` cutover 仍 open。

2026-07-07 补记:`text/native_bitmap_atlas/storage.rs` 已接管 mixed-storage partition/submission owner,且 `NativeBitmapAtlasFrame::storage_submissions()` 将主 frame `self.submission.run.atlas.clone()` 传入每个 storage submission。这样 mixed R8/RGBA frame 的 per-storage render submission 不再从 default `GlyphAtlasSet` 重建,而是继承 persistent frame atlas 与 page generation,关闭 mixed-storage persistent atlas 的 default-atlas reset 缺口。true async raster worker、完整 glyph slot owner/reuse、focused Cargo green、live editor-window typography QA 和完整 `TextAtlas` cutover 仍 open。

2026-07-07 补记:`text/native_bitmap_atlas.rs` 现在在 `source_cache.image(...)` 返回 `None` 时累计 `missing_raster_image_count`,并把该计数写入 `NativeBitmapAtlasPrepareReport`。`native_bitmap_atlas/handoff.rs` 新增 `MissingRasterImage` fallback reason,且只要该计数非 0,native bitmap atlas 即使 source/visible 计数看起来匹配也不能替代 glyphon。该切片关闭 PF-M3 首帧缺失 raster 图像时静默跳过但仍接管 glyphon 的风险,属于 fail-closed 降级前置项；true async raster worker、占位/近似桶首帧降级、完整 glyph slot owner/reuse、live editor-window typography QA 和完整 `TextAtlas` cutover 仍 open。

2026-07-07 补记:`text/parallel/raster_pool.rs` 现在落地真实 swash CPU glyph raster worker queue。`TextRasterWorkerPool` 按显式 worker count 或 `TaskPoolOptions` 的 async-compute budget 创建 `zircon-text-raster-*` worker；每个 worker 持有独立 `SwashRasterizer`,只消费 `Arc<[u8]>` 字体数据与 `SwashRasterRequest`,输出 `GlyphBitmap` completion。提交端维护 in-flight work id 去重、可选有界队列 backpressure 与诊断计数。2026-07-17 owner hard cut 把 completion 失效条件收敛为 `face_epoch`：raster source 在 page allocation 前生成，atlas page churn 不得使可复用 bitmap 失效；page generation 仍在后续 staging/upload boundary fail-closed。per-page upload merge、scroll raster/upload perf counters、live editor-window typography QA 和完整 `TextAtlas` cutover 仍 open。

2026-07-07 补记:`TextRasterCompletionDrain` 携带 face-invalidated work id,让主线程 owner 能清理被拒收 completion 的 pending 映射。`NativeBitmapAtlasSourceCache` 新增 `register_worker_request(...)` 与 `apply_worker_completion_drain(...)`,accepted `GlyphBitmap` 会转换为 source cache 可复用的 `SwashContent`/bearing/size/bytes；failed、unknown、invalid bitmap、face-invalidated 与 pending worker 数进入 source-cache frame report。2026-07-17 删除 stale-page work id/count，因为 raster source 不拥有 atlas page；历史 direct lib-test 只证明旧边界，新的 owner hard cut 仍待 current-source focused Cargo。本切片为非视觉数据面,不生成 PNG。

2026-07-07 补记:`scene_renderer/ui/text.rs`、`text/native_bitmap_atlas.rs` 与 `native_bitmap_atlas/source_cache.rs` 已把 production native bitmap atlas miss 接到 `TextRasterWorkerPool` 请求面。`ScreenSpaceUiTextBackend` 持有 optional raster worker pool,每帧先按当前 face epoch drain completion 到 source cache；glyph source cache miss 时不再同步调用 glyphon `SwashCache`,而是用当前 `fontdb` face index、font bytes 与 glyphon `CacheKey` 构造 `SwashRasterRequest::glyphon_cache_key(...)`,提交 `TextRasterWorkItem`,并以 `CacheKey` pending map 防止同 glyph 重复入队。2026-07-17 已删除固定 `page_generation=0` target；per-page upload merge 仍是独立后续优化。scroll raster/upload perf counters、live editor-window typography QA 与完整 `TextAtlas` cutover 仍 open。新的 hard cut 仍待 focused Cargo；本切片为非视觉数据面,不生成 PNG。

2026-07-08 补记:`text/native_bitmap_atlas/source_cache.rs` 已关闭横向亚像素 bucket 的 cache-key 抖动。`native_bitmap_atlas_stable_raster_cache_key(...)` 把 glyphon `CacheKey.x_bin` 归一为 `SubpixelBin::Zero`,并在 worker registration、cache lookup、approximate lookup、worker request、pending check 与 insert 入口统一使用该 stable key；`y_bin` 保留给纵向近似桶替代。scoped rustfmt 通过,`cargo check -p zircon_runtime --lib --tests --no-default-features --locked --jobs 1` exit 0,日志 `docs/tests/runtime/text/runtime_text_native_bitmap_stable_phase_check_tests_20260708.log` SHA256 `4C1B97B79C5783176B6C03256EDDF4D2B696FABA54D4E01CD730B6E169E4EE66`;完整 `native_bitmap_atlas` Cargo test 超时停止,不声明 full green。本切片为非视觉数据面,不生成 PNG；per-page upload merge、scroll raster/upload counters、live editor-window typography QA 与完整 `TextAtlas` cutover 仍 open。

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

2026-07-08 补记:`text/parallel/shape_pool.rs` 已落地 PF-M2 的 paragraph shaping 数据面。`TextShapeParagraph` 拥有文本、resolved style、range、direction/orientation 与 kerning 输入;`shape_paragraphs_with_cache(...)` 先按 `ShapedRunCacheKey + exact text` 查 shared `ShapedRunCache`,再把未命中的唯一 paragraph 交给 `TaskPool`/`parallel_for` chunk 执行,最后按原请求顺序返回 `Arc<ShapedGlyphRun>`。同批重复 miss 只 shape 一次,第二批相同请求命中 shaped cache。后续 editor property/axis 截图验证暴露 `pending` 队列类型推断失败后,本 owner 仅补 `Vec<PendingShapeJob>` 显式类型并复跑 `render_perf_text_parallel_shape_count` 1/1 passed。随后 `UiTextMeasureCache::prewarm_horizontal_paragraphs(...)` 用 `UiTextShapePrewarmRequest` 将可见 UI 段落批量预热到同一个 shaped-run cache,预热请求使用与 measure/layout 一致的 `UiTextDirection::Auto` 和完整 source range,避免预热 key 与后续布局 key 分叉。该 UI cache 入口已通过 focused Cargo,后续 surface render owner-text 自动 collection 已由下一条关闭；live editor-window typography QA、scroll raster/upload counters、per-page upload merge 与 full glyphon `TextAtlas` cutover 仍 open;本切片不生成视觉 PNG。

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
3. **单 run 上限与保护**:单 run 最大字节数(默认 64 KiB)与超限切分规则(在最近段落/强制断点处切);恶意超长单行(无断点)按上限硬切并记诊断,防 shaping O(n²) 路径拖死帧。

测试:`render_perf_text_huge_log_shapes_visible_only`(万行 log 首帧只 shape 可视区)、`text_paragraph_dirty_reshapes_edited_only`(编辑单段只重 shape 该段)、`text_oversized_run_splits_at_cap`。

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

**native bitmap atlas 空帧 flush(2026-07-05)**:PF-M1/AT-M3 的 swash source-image cache 采用持久跨帧复用 + 容量 LRU,但 native text 输入为空代表当前 renderer 已无 native glyph 需求。本轮把该状态定义为显式 flush:`ScreenSpaceUiTextBackend` 在 `texts.is_empty()` 分支 trim glyphon `TextAtlas` 的同时调用 `native_bitmap_atlas_idle_prepare_report(...)`,清空 `NativeBitmapAtlasSourceCache` 并把 `evicted_count/entry_count` 写入 `NativeBitmapAtlasPrepareReport.source_cache`。这不改变普通非空帧的 LRU 语义,也不引入每帧清空未引用项。

**native bitmap atlas 帧戳输入修正(2026-07-05)**:PF-M1/AT-M3 的 atlas retry、page residency 与后续 async raster 产物失效都依赖真实 frame index。旧生产 native path 用固定 `BITMAP_ATLAS_FRAME_INDEX = 1`,会让 blocked retry 永远报告 next frame 2,也让后续 frame-loop telemetry 无法区分连续帧。现在 `ScreenSpaceUiTextBackend` 对非空 native atlas frame 递增 `bitmap_atlas_frame_index`,并把该值传入 `native_bitmap_atlas_frame(...)`、prepare report 与 per-storage submission。该修正不等同于真实 retry-frame state execution 或全局 glyph cache slot invalidation,但关闭了它们接线前的固定帧号阻塞。

**native bitmap atlas retry face-invalidation report(2026-07-05)**:PF-M1/AT-M3 的 retry-frame state 现在把 face invalidation 导致的 blocked retry glyph 清理作为显式 telemetry 暴露。`GlyphAtlasBitmapRetryFrameState::discard_all_for_face_invalidation()` 清空 blocked queue 并累计 `pending_invalidated_blocked_glyph_count`;`apply_submission_plan(...)` 与 `native_bitmap_atlas_idle_prepare_report(...)` 通过 `take_report()` 在当前 visible frame 或 idle frame 写出 `NativeBitmapAtlasPrepareReport.retry_state.invalidated_blocked_glyph_count` 后清零。这样字体 face 变化不会把待重试 glyph 静默丢弃,也不会让空 native text frame 把 invalidation 计数滞留到下一次非空帧。该切片仍不等同于异步产物 face-validity requeue 或全局 glyph cache slot invalidation。

**native bitmap atlas renderer face-invalidation storage-pass telemetry(2026-07-05)**:PF-M1/AT-M3 继续把 face invalidation 推到 renderer-local storage pass 状态。`GlyphAtlasBitmapRenderer::discard_all_for_face_invalidation()` 清空 active storage passes 并累计 `invalidated_storage_pass_count`,下一次 prepare report 暴露被清理的 storage-pass 数量；`ScreenSpaceUiTextBackend` 在同一 face invalidation 分支同步清理 source cache、retry queue 与 renderer storage passes。这避免 source/raster cache 已失效但 renderer 仍保留旧 face atlas draw state 的诊断盲区。

**native bitmap atlas nearest sampler(2026-07-05)**:AT-M2/AT-M3 针对最新 editor crop 中“等线已生效但小字号左右边缘仍不稳”的 GPU sampling 风险,将 runtime bitmap atlas sampler 对齐 glyphon nearest sampling。`atlas_renderer/resources.rs` 的 sampler 使用 nearest min/mag/mipmap 且 LOD clamp 为 0,避免线性过滤把相邻 atlas texel 混入紧凑文件名标签的左右边缘。2026-07-06 focused Cargo `glyph_atlas_bitmap_sampler_matches_glyphon_nearest_sampling_contract` 已通过 1/1,日志 `docs/tests/runtime/text/runtime_text_bitmap_atlas_nearest_sampler_focused_cargo_20260706.log` SHA256 `AD2A6E83F4D73F08C1A53404740E1148353D791F0F15AE9316B783FAE4BE5692`。该切片是采样层防线,不替代 retained-host live crop QA、full glyphon `TextAtlas` cutover 或 LCD/gamma/background policy。

**native bitmap atlas handoff owner(2026-07-05)**:AT-M3/PF-M1 将 glyphon/native replacement 判定从 `scene_renderer/ui/text.rs` 根实现移入 `text/native_bitmap_atlas/handoff.rs`。`NativeBitmapAtlasHandoff` 和 `native_bitmap_atlas_handoff_for_report(...)` 现在与 `NativeBitmapAtlasPrepareReport` 同属 native bitmap atlas 子域,让 root text backend 只执行 single-storage replacement、mixed-storage replacement 或 glyphon fallback,不再持有 cutover policy。该切片不改变行为,但为后续完整 `TextAtlas` cutover、fallback reason telemetry 和性能缓存切换提供更清晰的 owner。

**native bitmap atlas fallback reason telemetry(2026-07-05)**:AT-M3/PF-M1 继续把 handoff 诊断留在 native atlas 子 owner。`NativeBitmapAtlasPrepareReport.glyphon_fallback_reason` 现在以 enum 记录 glyphon fallback 原因,并由 `text/native_bitmap_atlas/handoff.rs` 从 prepare report 状态推导。这样 full `TextAtlas` cutover 前,性能/缓存报告可以区分无可见 glyph、unsupported format、source coverage 缺口、LCD background composite 输入缺失、atlas allocation failure 或 mixed storage split 未就绪,不再只依赖 `replaces_glyphon=false`。

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
| `render_perf_text_scroll_list_reuses_cache` | 滚动列表 shape/layout 首段增量有界:滚动后只为新进入视口 row 增加 shaped miss/insert,重叠 row 命中 shaped cache；raster/upload prepare-report counter surface 已接入,真实滚动增量断言仍待补齐 |
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
