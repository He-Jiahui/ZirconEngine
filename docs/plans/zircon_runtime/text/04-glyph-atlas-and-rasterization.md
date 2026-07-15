---
related_code:
  - zircon_runtime/src/text/atlas/mod.rs
  - zircon_runtime/src/text/atlas/bitmap_run.rs
  - zircon_runtime/src/text/atlas/bitmap_run/allocation.rs
  - zircon_runtime/src/text/atlas/bitmap_run/failure.rs
  - zircon_runtime/src/text/atlas/bitmap_run/placeholder.rs
  - zircon_runtime/src/text/atlas/bitmap_run/retry.rs
  - zircon_runtime/src/text/atlas/bitmap_run/staged_upload.rs
  - zircon_runtime/src/text/atlas/bitmap_run/staging.rs
  - zircon_runtime/src/text/atlas/bitmap_run/tests.rs
  - zircon_runtime/src/text/atlas/bitmap_run/types.rs
  - zircon_runtime/src/text/atlas/bitmap_run/upload.rs
  - zircon_runtime/src/text/atlas/bitmap_run/validation.rs
  - zircon_runtime/src/text/atlas/page.rs
  - zircon_runtime/src/text/atlas/page_residency.rs
  - zircon_runtime/src/text/atlas/page_residency/tests.rs
  - zircon_runtime/src/text/atlas/render_contract.rs
  - zircon_runtime/src/text/atlas/render_contract/tests.rs
  - zircon_runtime/src/text/atlas/render_plan.rs
  - zircon_runtime/src/text/atlas/render_plan/tests.rs
  - zircon_runtime/src/text/atlas/render_batch.rs
  - zircon_runtime/src/text/atlas/render_batch/tests.rs
  - zircon_runtime/src/text/atlas/render_gpu_plan.rs
  - zircon_runtime/src/text/atlas/render_gpu_plan/tests.rs
  - zircon_runtime/src/text/atlas/render_gpu_plan/bind_group.rs
  - zircon_runtime/src/text/atlas/render_gpu_plan/draw_command.rs
  - zircon_runtime/src/text/atlas/render_gpu_plan/pipeline.rs
  - zircon_runtime/src/text/atlas/render_gpu_plan/vertex.rs
  - zircon_runtime/src/text/atlas/render_gpu_plan/viewport.rs
  - zircon_runtime/src/text/atlas/render_submission.rs
  - zircon_runtime/src/text/atlas/render_submission/placeholder.rs
  - zircon_runtime/src/text/atlas/render_submission/frame_driver.rs
  - zircon_runtime/src/text/atlas/render_submission/frame_state.rs
  - zircon_runtime/src/text/atlas/render_submission/retry.rs
  - zircon_runtime/src/text/atlas/render_submission/tests.rs
  - zircon_runtime/src/text/atlas/shaders/glyph_atlas_sampling.wgsl
  - zircon_runtime/src/text/atlas/shaders/glyph_atlas_pipeline.wgsl
  - zircon_runtime/src/text/atlas/shelf_allocator.rs
  - zircon_runtime/src/text/atlas/dirty.rs
  - zircon_runtime/src/text/atlas/dirty/tests.rs
  - zircon_runtime/src/text/atlas/upload.rs
  - zircon_runtime/src/text/atlas/upload/tests.rs
  - zircon_runtime/src/text/atlas/raster_key/mod.rs
  - zircon_runtime/src/text/atlas/raster_key/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/native_bitmap_atlas.rs
  - zircon_runtime/src/text/native_bitmap_atlas/source_cache.rs
  - zircon_runtime/src/text/native_bitmap_atlas/storage.rs
  - zircon_runtime/src/text/native_bitmap_atlas/retry_frame.rs
  - zircon_runtime/src/text/native_bitmap_atlas/tests.rs
  - zircon_runtime/src/text/native_bitmap_atlas/tests/frame.rs
  - zircon_runtime/src/text/native_bitmap_atlas/tests/handoff.rs
  - zircon_runtime/src/text/native_bitmap_atlas/tests/retry_frame.rs
  - zircon_runtime/src/text/native_bitmap_atlas/tests/source.rs
  - zircon_runtime/src/text/native_bitmap_atlas/tests/source_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/font_id_report.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/sdf_fallback.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_texture_upload.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_texture_upload/write.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_texture_upload/binding.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_texture_upload/frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_texture_upload/resource.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_texture_upload/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_renderer/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_renderer/pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_renderer/resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_renderer/vertex.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_renderer/renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_renderer/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas/tests
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_upload.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render.rs
  - zircon_runtime/src/text/raster/mod.rs
  - zircon_runtime/src/text/raster/policy.rs
  - zircon_runtime/src/text/raster/swash/mod.rs
  - zircon_runtime/src/text/raster/swash/bitmap.rs
  - zircon_runtime/src/text/raster/swash/color_strike.rs
  - zircon_runtime/src/text/raster/swash/error.rs
  - zircon_runtime/src/text/raster/swash/request.rs
  - zircon_runtime/src/text/raster/swash/rasterizer.rs
  - zircon_runtime/src/text/raster/swash/tests.rs
  - zircon_runtime/Cargo.toml
  - zircon_runtime/src/ui/text/measure_cache.rs
  - zircon_runtime_interface/src/ui/surface/render/resolved_style.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/blend.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/blend/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/sync.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/sync/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/layout.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/layout/metrics.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/layout/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/placement.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/placement/metrics.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/placement/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/glyphs.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/glyphs/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/glyphs/row.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/glyphs/row/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/raster.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/raster/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/font.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/font/tests.rs
design_references:
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Fonts/FontTypes.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/SlateFontRenderer.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Fonts/FontRasterizationMode.h
  - dev/bevy/crates/bevy_text/src/font_atlas.rs
  - dev/bevy/crates/bevy_text/src/font_atlas_set.rs
  - dev/Fyrox/fyrox-ui/src/font/mod.rs
  - dev/slint/internal/core/textlayout/sharedparley.rs
plan_sources:
  - docs/plans/zircon_runtime/text/index.md
  - docs/plans/zircon_runtime/text/02-shaping-unicode-and-bidi.md
  - docs/plans/zircon_editor/editor_layout/17-text-rendering-and-typography.md
  - docs/plans/zircon_runtime/render/14-2d-stack.md
status: in_progress
---

# 04 字形栅格化 / 字形图集 / 分辨率精度

> 本计划把 `02` 的 `ShapedGlyph.glyph_id` 栅格成像素并装进 GPU 图集。它是 `editor_layout/17 G2`(字形随 DPI 重栅格,根治像素化)的实现,统一现有 glyphon bitmap atlas 与 SDF atlas 的分配/上传策略。

## 1. 目标

1. **栅格器选型与统一**:bitmap 路径用 swash(彩色 emoji + outline alpha + subpixel);SDF/MSDF 路径见 `05`。栅格输入按物理像素。
2. **图集化生成**:shelf(货架行)分配器、多页管理、脏矩形增量上传、页级 LRU 逐出;`R8Unorm`(alpha/SDF)与 `Rgba8Unorm`(彩色/MSDF)分组分页。
3. **分辨率精度**:`physical_px = logical_px × scale_factor`;scale 变即重栅格;subpixel 定位(水平 1/3 量化或整像素吸附);hinting 策略;atlas key 含 scale 量化桶。
4. **统一图集服务**:UI 与场景 2D 共用 `GlyphAtlasSet`(`render/14` 已起名),替换现有各自为政的 glyphon `TextAtlas` 与 `sdf_atlas`。

## 2. 现状与差距

- `graphics/.../ui/text.rs`:glyphon 自管 `TextAtlas` + `SwashCache`,栅格/装箱/上传都在 glyphon 内部,ZirconEngine 不可控、不能与 SDF 共享。2026-07-03 native glyphon `TextArea` 入口已新增 `native_text_area_placement(...)`,在进入 glyphon 前把 frame `left/top` 归一到设备像素并保留整数 clip bounds,先关闭用户编辑器截图中由小数 origin 造成的字形左右漂移/采样相位不稳首段；完整 glyphon atlas cutover 仍未完成。
- `ui/sdf_atlas.rs`:自有 SDF cache 已从固定单页扩展到统一 `GlyphAtlasSet` 的 SDF page identity + shared shelf rect + page residency/LRU 数据面。2026-07-02 已让 `SdfAtlasPlan` 持有 `graphics/text/atlas::GlyphAtlasSet` 的 `Sdf` page identity,并用共享 shelf allocator、dirty-rect owner 与 page residency owner 生成 slot rect/cache report/upload report 数据面；`SdfAtlasSlot.page_key`、`SdfAtlasCacheReport.dirty_pages` 与 `SdfAtlasUploadReport.dirty_pages` 已补齐 page-keyed dirty/upload 数据面,同时保留 page[0] `dirty_rect` 兼容字段；`sdf_upload.rs`/`sdf_render.rs` 已完成 SDF dirty-rect `Queue::write_texture` partial upload,并让 renderer/texture owner 消费所有 page-keyed upload commands 到 `texture_2d_array` layer；`graphics/text/atlas/upload.rs` 首段已接管通用 upload command math；`graphics/text/atlas/page_residency.rs` 首段已接管每格式页上限、缺页分配、最旧未引用页逐出与全页受保护阻塞的 LRU 决策数据面；shelf overflow 现在不再放大单页 atlas,而是在固定 page size 内分配,溢出时通过 `GlyphAtlasSet::reserve_page_for_format(...)` 申请 page[1+] 并把 `SdfAtlasSlot.page_key` 指向真实页；evicted/rebuilt SDF page 现在通过 `SdfAtlasPlan.rebuilt_pages` 在 cache transition 中整页标脏；over-cap/oversized allocation failure 现在记录到 `SdfAtlasPlan.allocation_failures` 并汇总到 SDF prepare report,且 `SdfAtlasRun.glyph_failure_reasons` 按字符位置记录 page-limit/oversized 原因；fallback policy 已拆到 `ui/text/sdf_fallback.rs`,并能把连续同原因失败字形归并为 fallback spans/report span counts；Horizontal LTR/explicit RTL/no-wrap/non-justify 失败 span 已生成局部 native overlay,不支持的混合情形继续 whole-batch native fallback 且会记录 unsupported mixed overlay reason diagnostics。真实 alpha bitmap atlas 替换、glyphon atlas 迁移、持久化 glyph cache/residency 驱动的完整淘汰闭环、broader glyph-level mixed fallback(Vertical/Auto-Mixed/justify/wrapped)、independent oversized fallback、DPI/subpixel/hinting 仍未完成。
- `graphics/text/raster/policy.rs`:已承接 `raster_path_for`/`GlyphRasterPolicy` 的 bitmap/SDF/MSDF/Color 选路数据面,并开始按请求格式与 outline/shadow/glow 效果强制距离场路径。
- `graphics/text/raster/swash/`:已建立 swash 隔离层首段数据契约并按结构规范拆成 folder-backed owners:`bitmap.rs` 记录 `GlyphBitmap` size/bearing/px_size/data/channels/content 与 fallible validation,alpha/color/subpixel 位图可映射到 `GlyphAtlasFormat::{AlphaMask,Color,SubpixelMask}` 与 R8/RGBA storage；`atlas_source.rs` 将已验证 `GlyphBitmap` 投影为 `GlyphAtlasBitmapSource`,保留 atlas format、content size、screen rect、foreground/background color 与真实 `data.len()` source byte length,使 swash 输出可直接喂给 bitmap atlas run validation/allocation；`request.rs` 持有 `SwashRasterRequest`/`SwashRasterSource`/`SwashBitmapStrike` 与 swash source/render-format 选择；`rasterizer.rs` 持有真实 swash `ScaleContext`/`Scaler`/`Render` adapter,并把 swash `Image` 归一化为 `GlyphBitmap`；`color_strike.rs` 持有 COLR/CPAL 优先与 CBDT/sbix strike selection,选择 ≥目标尺寸最近 strike 下采样,否则最大较小 strike 作为显式 upscale fallback,并按比例换算 size/bearing/advance；`error.rs` 持有 `SwashRasterError`;`tests.rs` 保留 FiraSans 真实字体 alpha/subpixel outline、bitmap validation、atlas source bridge 与 emoji strike owner tests。Focused Cargo `text_raster_swash` 旧 11/11 证据仍适用旧单文件实现;最新 bridge 切片因外部 cargo/rustc lanes 活跃只声明 scoped rustfmt、diff check 与视觉证明。emoji RGBA fixture 实像素测试、生产 alpha bitmap atlas renderer 与 glyphon `TextAtlas` 切换仍未完成。
- `graphics/text/raster/swash/request.rs` + `rasterizer.rs`:2026-07-07 补齐 glyphon `CacheKey` 到 swash raster request 的语义 parity。`SwashRasterRequest::glyphon_cache_key(...)` 现在保留 glyph id、px size、x/y subpixel offset、`DISABLE_HINTING`、`PIXEL_FONT` offset 规则、`FAKE_ITALIC` transform、font weight variation 与 glyphon source fallback 顺序 `[ColorOutline(0), ColorBitmap(BestFit), AlphaOutline]`;`SwashRasterizer` 将 offset、render format、fake-italic skew 与 `wght` variation 转发给 swash。该切片只关闭 request/rasterizer 数据面,不声明 production native bitmap atlas miss scheduling、per-page upload merge、live editor-window typography QA 或完整 glyphon `TextAtlas` cutover 完成。
- `graphics/text/atlas/bitmap_run/staged_upload.rs`:2026-07-07 在 page-generation upload guard 之后补上 stale upload requeue report。`GlyphAtlasBitmapTextureUploadRequestPlan` 现在显式携带 `requeued_uploads`、`stale_page_generation_count` 与 `face_invalidated_count`;带 live atlas/face-validity 输入的 request plan 遇到 missing page、page generation mismatch 或 face invalidated 时不产出 texture upload request,而是记录 `GlyphAtlasBitmapRequeuedUpload`。这关闭了 stale artifact / face invalidated artifact 被静默跳过的首段数据面;真实 async worker、global glyph slot invalidation 与完整 glyphon `TextAtlas` cutover 仍未完成。
- `graphics/scene/scene_renderer/ui/atlas_texture_upload/frame.rs`:2026-07-07 继续把 low-level requeue report 接到 renderer-local texture upload frame。`GlyphAtlasBitmapTextureUploadFrameReport` 现在按帧汇总 missing-page、page-generation mismatch、face invalidated 与总 requeued upload 计数；`glyph_atlas_bitmap_texture_upload_frame_plan_for_atlas_and_face_validity(...)` 在任何 requeue 存在时保持 `ready_to_write_texture=false`,不向 WGPU writer 交出可写 plan。该切片只收束 frame report handoff,真实 async worker、global glyph slot invalidation、完整 glyphon `TextAtlas` cutover 与 live editor-window typography QA 仍未完成。
- `graphics/scene/scene_renderer/ui/atlas_renderer/renderer.rs`:2026-07-07 继续把 requeue frame report 推到 renderer prepare telemetry。生产 `prepare_submission(...)` / `prepare_storage_submissions(...)` 现在用 submission 自带的 live `GlyphAtlasSet` 调用 `glyph_atlas_bitmap_texture_upload_frame_plan_for_atlas(...)`,不再绕过 page-generation/missing-page guard；`GlyphAtlasBitmapRendererPrepareReport` 汇总 `upload_requeued_count`、`upload_missing_page_requeue_count`、`upload_page_generation_mismatch_requeue_count` 与 `upload_face_invalidated_count`,且 `upload_failure_count` 将 requeued uploads 计入失败口径。该切片关闭 renderer telemetry handoff 缺口,但 per-face artifact validity source、真实 async worker、global glyph slot invalidation、完整 glyphon `TextAtlas` cutover 与 live editor-window typography QA 仍未完成。
- `graphics/text/atlas/bitmap_run.rs` + `render_submission/retry.rs` + `scene_renderer/ui/text/native_bitmap_atlas.rs`:2026-07-07 补上主 native bitmap atlas 路径的持久 atlas state 与 slot invalidation 数据面首段。bitmap run/render submission/retry driver 现在可接收上一帧 `GlyphAtlasSet`,在帧开始清除 page reference,需要重建未引用页时记录 `GlyphAtlasBitmapSlotInvalidation { page_key, page_generation }` 并把整页标脏；`ScreenSpaceUiTextBackend` 在非空 native bitmap frame 间保留主 submission atlas,字体 face invalidation 与空 native text frame 则清空该 atlas。该切片让 page-generation guard 有真实跨帧 page state 可比较,避免 atlas page 重建后旧 slot 继续静默可写；同日 follow-up 又把 storage partition/submission 逻辑拆到 `scene_renderer/ui/text/native_bitmap_atlas/storage.rs`,并让 per-storage submission 通过 `glyph_atlas_bitmap_render_submission_plan_with_atlas(...)` 继承主 frame 的 `self.submission.run.atlas.clone()`,关闭 mixed R8/RGBA storage split 中 per-storage default-atlas reset。真实 async worker、完整 glyph slot owner、focused Cargo green、完整 glyphon `TextAtlas` cutover 与 live editor-window typography QA 仍未完成。
- `scene_renderer/ui/text/native_bitmap_atlas.rs` + `native_bitmap_atlas/handoff.rs`:2026-07-07 继续把 native bitmap atlas 的缺失 raster 图像从静默跳过改为可诊断 fail-closed。`source_cache.image(...)` 返回 `None` 时累计 `missing_raster_image_count`,prepare report 暴露该计数；handoff owner 新增 `MissingRasterImage` fallback reason,并确保只要缺图计数非 0,native atlas 不能替代 glyphon,即使 source image count 与 visible glyph count 看起来相等。该切片不完成真实 async raster worker 或首帧占位渲染,但关闭 atlas 输入不完整时仍接管 glyphon 的首帧降级风险。
- `scene_renderer/ui/text/native_bitmap_atlas/source_cache.rs` + `graphics/text/parallel/raster_pool.rs`:2026-07-07 继续补真实 async raster worker 的 atlas 回流端。`TextRasterCompletionDrain` 现在保留 stale-page 与 face-invalidated work id,`NativeBitmapAtlasSourceCache` 记录 work id → glyphon `CacheKey` pending 映射并消费 completion drain；accepted `GlyphBitmap` 被转成 native source cache 的 `SwashContent`、bearing、尺寸与 bytes,failed/unknown/invalid/stale/face-invalidated/pending worker 计数进入 `NativeBitmapAtlasSourceCacheFrameReport`。idle frame 与 face invalidation 会同步清空 pending worker key。直接 lib-test binary 通过 source-cache 5/5 与 worker-drain 1/1；该切片仍不声明 production native bitmap atlas miss scheduling、把已闭合的 CacheKey request parity 接入实际 worker request、per-page upload merge 或完整 glyphon `TextAtlas` cutover 完成。
- `scene_renderer/ui/text.rs` + `scene_renderer/ui/text/native_bitmap_atlas.rs` + `native_bitmap_atlas/source_cache.rs`:2026-07-07 将 production native bitmap atlas miss scheduling 接到真实 worker request。`ScreenSpaceUiTextBackend` 持有 optional `TextRasterWorkerPool`;native frame 先 drain completion,再让 miss 调用 `request_worker_image(...)`。source cache 用真实 face index/font bytes 和 glyphon `CacheKey` 生成 `SwashRasterRequest::glyphon_cache_key(...)`,提交 `TextRasterWorkItem`,并用 `CacheKey` pending map 去重；source image 只从已完成 cache 读取,不再在 native atlas miss 路径同步调用 glyphon `SwashCache`。该切片关闭 production miss -> worker request 首段,但 `page_generation=0` 仍待 per-page upload merge 替换,live editor-window typography QA 与完整 glyphon `TextAtlas` cutover 仍未完成。
- `scene_renderer/ui/text/native_bitmap_atlas/source_cache.rs` + `native_bitmap_atlas.rs` + `native_bitmap_atlas/handoff.rs`:2026-07-07 关闭 PF-M3 “已有近似桶”首帧替代切片。source cache 只在 font/glyph/size/weight/flags 完全相同且仅 subpixel bin 不同时返回近似图像,并记录 `approximate_hit_count`;native frame 仍为 exact key 排队 worker request,但当前帧可用近似 source image 继续 native bitmap atlas submission,不走透明占位;prepare report 记录 `approximate_raster_image_count`,first-frame degradation 记录 `ApproximateBucketReplacement`。该切片不替代 per-page upload merge、persistent glyph slot owner 或 full glyphon `TextAtlas` cutover。
- `scene_renderer/ui/text/native_bitmap_atlas/source_cache.rs`:2026-07-08 继续收窄 native bitmap atlas 在用户小字号 editor label 上的横向相位抖动风险。source cache 现在通过 `native_bitmap_atlas_stable_raster_cache_key(...)` 把 glyphon `CacheKey.x_bin` 归一为 `SubpixelBin::Zero`,并在 worker registration、cache lookup、approximate lookup、worker request、pending check 与 insert 入口统一使用该 key。该切片关闭 horizontal subpixel bucket 反复生成 source image / pending worker request 的数据面；`y_bin` 仍保留给纵向近似桶 fallback,不改变 per-page upload merge 或完整 glyphon `TextAtlas` cutover 口径。
- `scene_renderer/ui/text.rs` + `scene_renderer/ui/text/native_bitmap_atlas/source_cache.rs`:2026-07-10 追加 native raster/upload prepare-report 聚合层。`ScreenSpaceUiTextPrepareReport.raster_upload` 从 native bitmap atlas prepare report 读取 visible/source/missing/approx glyph、source-cache hit/miss/worker-request 与 submission upload bytes,再合并 bitmap renderer upload/requeue/failure/ready 状态,为 AT-M3/PF-M4 的 scroll raster/upload 计数断言提供单一入口。该切片只接入可观测 surface,不声明 per-page upload merge、真实 scroll increment assertion、live editor-window typography QA 或完整 glyphon `TextAtlas` cutover 完成。
- 2026-07-10 proof 复查:`docs/tests/runtime/text/runtime_text_editor_grayscale_line_snap_subpixel_glyph_phase_20260708*.png` 追加像素指标 JSON `runtime_text_editor_grayscale_line_snap_subpixel_glyph_phase_image_metrics_20260710.json`,记录 full label painted=210/max gap=6/center=52.843 与 narrow label painted=258/max gap=3/center=70.740。该复查只验证 retained framebuffer screenshot proof,不关闭 live editor-window typography QA;实时窗口 capture 因当前 cargo/rustc/link 队列拥塞暂缓,process gate 记录在 `runtime_text_live_editor_capture_process_gate_20260710.log`。
- `zircon_editor` retained-host consumer:2026-07-03 已把 `EditorTypographyTokens.font_smoothing` 投影为 `HostTextPreferences.smoothing`,并让 smoothing 进入 `paint_text/raster.rs` glyph cache key。默认 `grayscale` 请求 swash `Format::Alpha`,避免 retained 软件 framebuffer 在没有 dedicated LCD/background blend policy 时出现彩边;可配置 `subpixel` 请求 `Format::Subpixel`,保留 swash `Content::SubpixelMask` 的 RGB coverage 为 `CachedGlyphRasterFormat::SubpixelMask`,继续由 `draw/glyphs/row.rs` / `blend_pixel_channel_coverage(...)` 消费。2026-07-03 最新 editor tab crop 又暴露了 retained CPU path 的局部 spacing/placement 问题：`draw/layout.rs` 已改为在 host run width 或 host per-grapheme natural advances 不可得时 fail-closed；2026-07-04 follow-up 又把局部 advance 容差固定为 `0.0625px`,超过 1/16px 的 runtime/shaped-origin 借位直接回退 host natural spacing。`draw/glyphs.rs` 已将 subpixel bin 从向左取整改为最近 bin,并继续提升到 8x/8-bin pen-origin phase,避免 glyph 栅格位置系统性左偏或 0.125px 量化漂移。后续 pen-origin phase 修复继续把 `RuntimeTextGlyph.origin_x` 贯穿 host/runtime projection,让 retained subpixel bin 以 pen origin 而不是 glyph bitmap-left/left bearing 选择,同时把 swash/fontdue fallback 的 `CachedGlyphMetrics.x_offset` 规范为相对 pen origin。该消费者对齐本计划 `font_smoothing`/`SubpixelMask` 采样语义,但不替代 runtime 统一 glyph atlas cutover,也不在 editor 控件层写死具体字体族;截图 crop 只作为当前 retained-host 视觉证据。
- `zircon_editor` retained-host no-rollover phase clamp:2026-07-06 用户最新局部截图显示等线已生效后,nearest high-phase rollover 仍会让接近 `.95px` 的 glyph origin 滚到下一整数像素,造成 compact label 单字左右落点不舒服。该历史切片把 rounded 8-bin 结果 clamp 到最高 in-pixel bin,让 `20.95px`/`44.95px` 保留在当前 pixel cell 的 `0.875` phase。2026-07-07 复核后,该策略已由 nearest-phase quantization supersede,因为 high-phase clamp 会留下系统性左右偏置。该切片只修 retained-host placement owner,不改 ZUI 字体 token、runtime FontDatabase、glyph atlas identity、root painter 或组件局部字距。
- `zircon_editor` retained-host nearest-phase quantization refresh:2026-07-07 用户最新截图再次确认 high-phase glyph 不能被 clamp 到当前 pixel 的最高 bin。前一轮 in-pixel clamp 结论已作为历史状态处理；当前 `placement/metrics.rs` 对完整 screen x 做 1/8px 最近点量化,再拆分 pixel/bin,让 `20.95px` 进入下一 pixel 的 `0/8` phase。验证证据写入 `docs/tests/runtime/text/runtime_text_editor_retained_phase_quantization_*`;本轮 recheck wrapper harness 通过 6/6,proof PNG 已人工复核,同名 target/cargo-target PNG 扫描为 0；focused Cargo screenshot rerun 904s 编译超时且没有新 recheck PNG。该消费者修复不改变 glyph atlas identity、native bitmap atlas cutover、字体族、ZUI 资产、root painter 或组件局部 letter-spacing。
- `zircon_editor` retained-host grayscale line-snap subpixel glyph phase:2026-07-08 用户最新 editor crop 证明上一条 default grayscale per-glyph device-pixel placement 会把自然 fractional advances 变成不均匀整数步进。当前有效 raster/placement policy 是默认 Grayscale 只在 line origin 贴近 nearest device pixel,单字 glyph 继续按完整 screen x 的 retained 1/8px phase 采样；explicit Subpixel 保留 fractional line origin + 1/8px glyph phase。该修复仍在 retained-host CPU placement/raster owner 内,不改 glyph atlas identity、native bitmap atlas cutover、字体族、ZUI 资产、root painter 或组件局部 letter-spacing。
- `zircon_editor` retained-host raster-bearing alignment reclose:2026-07-05 用户最新 crop 证明等线已生效后,有限 layout bitmap-left 仍可能把 host-layout bearing 与当前 raster backend bearing 的差异显示成单字左右偏移。`paint_text/draw/glyphs.rs::retained_glyph_bitmap_pixel_x(...)` 当前规则是:只要 `RuntimeTextGlyph.origin_x` finite,最终 bitmap-left 使用 `origin_pixel_x + raster.metrics.x_offset`;`RuntimeTextGlyph.x` 仅在 origin 不可用时作为 layout fallback。`draw/glyphs/tests.rs` 同时覆盖正常 raster-bearing authority、stale layout-left 不可覆盖有效 origin、invalid-origin fallback。验证图/日志:`docs/tests/runtime/text/runtime_text_editor_retained_raster_bearing_alignment_preview_20260705.png` SHA256 `7B66F057AA95412E65BDE04CF9908C42D47E3CD3598EE041622E23F5AB4A663B` / `runtime_text_editor_retained_raster_bearing_alignment_validation_20260705.log` SHA256 `255DCCF5EA9B559595EB0A50BADF2A36C04BB1515894DF247977D38857A763C2`;target/cargo-target 同名扫描 0,focused Cargo 因外部 cargo/rustc lanes 活跃 deferred。
- `zircon_editor` retained-host shaped-position bridge:2026-07-03 针对用户最新 editor tab crop 中“字体已是等线但字符左右间距/落点仍怪”的问题,`paint_text/draw/layout.rs` 继续把 runtime `ShapedGlyphRun` 的 `ShapedGlyph` 列表随 single-line layout 带到 host projection;当 shaped glyph id、host glyph index、byte offset visual range 与总宽度均匹配时,retained host 直接使用 `shaped.x + shaped.offset_x` 作为 pen origin,再加 bitmap-left offset 得到 draw x。最新 follow-up 又要求 shaped-origin 推导出的局部相邻 advance 必须匹配当前 host face 的自然 advance 容差,否则即使总宽匹配也回退到 host natural spacing,避免 `folder-open.svg` 这类 compact tab label 被局部 0.25px 借位拉出左右漂移。glyph id/range 不匹配、RTL 或 virtual glyph 会回退到既有 host natural spacing / runtime advance guard,避免把不确定 shaping 结果硬投影进编辑器。该切片只修 retained-host 低层布局桥,不改控件字体族、ZUI 资产、root painter 或 runtime atlas owner。
- `zircon_editor` retained-host shaped-origin spacing correction:2026-07-04 follow-up 是历史状态。该切片当时确认上面的 shaped-position bridge 不应再被 retained raster-bin phase 反向否决,并移除 shaped-position 专用的 `shaped_positions_preserve_retained_raster_bins(...)` gate。2026-07-05 shaped-origin phase fallback 已 supersede 该行为:当前 same-phase runtime shaped pen origin 仍跟随 shaping authority,但跨 retained 1/8px phase 的 matched shaped origin 会回退 host natural spacing。保守的 retained raster-bin fail-closed 也继续保留在 runtime grapheme advance projection 路径,防止没有完整 shaped glyph 列表时累积 advance drift。该切片不改 raster cache key、8-bin placement owner、字体族、ZUI 资产、runtime FontDatabase 或 atlas routing。
- `zircon_editor` retained-host fallback raster phase:2026-07-04 针对用户最新小字号 editor tab/file label crop,确认上层字体族已切等线、layout/shaped bridge 也已 fail-closed 后,低层 fontdue fallback alpha mask 仍忽略 retained pen-origin phase。`draw/glyphs.rs` 将 retained fallback supersample/bin 改为 4x/4-bin,`raster.rs` 让 `FontdueFallback` 携带 `CachedGlyphRaster.sample_offset_x`,`draw/glyphs/row.rs` 在 alpha-mask downsampling 前按 phase 修正采样窗口。2026-07-05 follow-up 又补齐 supersampled SubpixelMask/RGB row 的同一 `sample_offset_x` 消费；native scale=1 的 swash SubpixelMask 仍由 `Render::offset(...)` 烘入 phase。该路径只修 CPU retained-host row sampling,不改 ZUI 控件字体族、runtime atlas identity、shader/blend contract 或 editor root painter。
- `zircon_editor` retained-host cumulative runtime-advance phase guard:2026-07-04 针对最新 editor label crop 中“等线已生效但字符仍像左右偏移”的问题,确认单 grapheme `0.0625px` 容差仍可能让多个 `0.05px` 微偏差累积跨越 retained 1/8px raster bin。`paint_text/draw/layout.rs` 现在在接受 runtime advance projection 前运行 `runtime_advances_preserve_retained_raster_bins(...)`,逐 glyph 比较 host natural origin 与 runtime projected origin 的 retained placement bin；一旦累积相位跨 bin,整段回退 host natural spacing。`draw/layout/tests.rs` 用 `editor base.zui` 锁定该 fail-closed 行为。该切片不改字体族、ZUI 资产、root painter、runtime FontDatabase 或 atlas routing。
- `zircon_editor` retained-host resolved font-family projection:2026-07-04 在上面的 phase/left-bearing/raster guard 之外补齐 family identity 一致性。`paint_text/font.rs::runtime_text_style_for_face(...)` 不再把 requested `system-ui`/generic family 直接传给 runtime layout,而是复用 retained-host cache 里的 `runtime_family`。这让 retained layout/shape 与最终 swash/fontdue raster 均选中 DengXian/等线这类已解析实际 face,避免同一可见字体下 advance 来源仍不一致。该切片不改 glyph raster cache key、ZUI 资产、root painter、runtime FontDatabase 或 atlas routing。
- `zircon_editor` retained-host cache poison recovery:2026-07-04 在同一 retained CPU text owner 内收束缓存可靠性债,新增 `paint_text/sync.rs` 作为 poison-recovering mutex helper owner。字体 cache、glyph raster cache 与 Swash `ScaleContext` lock 失败后不再走生产 `expect` 崩溃路径,而是恢复 guard 后继续绘制；该切片不改变 cache key、字形采样、字体族选择、ZUI 资产或 runtime atlas owner,只关闭 cache/context lock poisoning 对编辑器文本绘制的直接崩溃面。
- `zircon_editor` retained-host sync tests owner split:2026-07-04 继续按结构规范收束同一 cache reliability owner。`paint_text/sync.rs` 只保留 `lock_recovering_poison<T>` 与 test module hook,poison regression 移到 `paint_text/sync/tests.rs`,让测试用 `catch_unwind`/`panic`/`expect` 不再出现在生产 owner 文件中。该切片不改变 mutex recovery 行为。
- `zircon_editor` retained-host unavailable font fallback:2026-07-04 继续关闭 retained CPU text owner 的生产崩溃面。`font.rs` 不再把 embedded static font 当作必然可解析并 `expect`,而是让 `HostTextFont` 携带 `Option<Font>`;系统字体、请求 embedded face 与 embedded mono 均失败时进入 unavailable font 状态。`draw/layout.rs` 在字体不可用时返回空 glyph run,`raster.rs` 返回空 alpha-mask raster,让编辑器绘制继续而不是崩溃。该切片不改字体偏好、ZUI 资产、runtime FontDatabase 策略或 atlas/raster key 语义。
- `zircon_editor` retained-host blend contract regressions:2026-07-04 继续在 retained CPU text 最终像素合成 owner `paint_text/blend.rs` 挂载 child tests,由 `paint_text/blend/tests.rs` 补回归,锁定 alpha mask 半透明合成、全透明 no-op、SubpixelMask RGB coverage 独立合成与 source alpha 进入 per-channel coverage 的合同。该切片不改当前合成算法,不替代 GPU atlas shader/blend contract 或最终 LCD/gamma/background policy,只防止后续 row sampling/背景合成改动把 retained framebuffer 的不透明输出语义打破。
- `zircon_editor` retained-host framebuffer ink spacing guard:2026-07-04 继续把用户截图中的小字号 label 左右不适问题锁到 retained framebuffer 像素层。`paint_text_tests.rs::retained_text_editor_crop_labels_keep_stable_ink_spacing` 通过真实 `HostRgbaFrame` 扫描 ink left edge、ink center、painted pixel count 与 internal empty columns,并比较 8.875px/8.925px 近起点,防止等线字体、layout bridge、8-bin placement 和 fallback phase 修复后仍出现整像素级左右跳或异常空列。该切片不改 glyph atlas identity、runtime FontDatabase、ZUI 资产、root painter、GPU draw-list 或控件局部字体策略。
- `zircon_editor` retained-host grayscale alpha phase:2026-07-05 用户最新小字号 editor label crop 继续显示“字体已是等线但字符左右间距和渲染落点仍偏左/偏右”。复核上一轮 grayscale pixel snap 后确认逐字 nearest device-pixel placement 会把自然 fractional advance 改成不均匀整像素步进。当前策略将 `HostTextSmoothing::Grayscale` 收窄为 swash `Format::Alpha` 覆盖格式,不再表示逐字整像素吸附；`retained_glyph_placement_for_smoothing(...)` 对 grayscale 与 explicit subpixel 都使用 `RetainedGlyphPlacement::from_screen_x(...)` 的 8-bin alpha phase。2026-07-06 已将默认 Grayscale 的 line origin 重新收束为 nearest device pixel,显式 Subpixel 仍保留 finite fractional line origin；`runtime_advances_preserve_retained_raster_bins(...)` 与 invalid-origin fallback 均复用同一 placement policy。该切片只修 retained CPU glyph placement policy,不改 ZUI 字体族、root painter、runtime FontDatabase、glyph atlas identity 或 native bitmap atlas cutover。
- `zircon_editor` retained-host subpixel line-origin preservation:2026-07-05 继续保留显式 Subpixel/LCD background composite 需要的 fractional origin 数据。后续 fractional-origin follow-up 曾短暂把默认 grayscale 也切到 finite fractional line origin,但 2026-07-06 已被 `runtime_text_editor_grayscale_origin_snap_direct_binary_visual_passed` supersede:当前 `draw/placement.rs::retained_text_origin_for_smoothing(...)` 对 Grayscale 吸附到 nearest device pixel,对 Subpixel 保留 finite fractional line origin,非 finite 值仍归零。该切片不改控件字体族、ZUI 资产、root painter、runtime FontDatabase、glyph atlas identity 或 native bitmap atlas cutover。
- `zircon_editor` retained-host shaped-origin phase fallback:2026-07-05 针对用户最新 editor tab/file label crop 中“等线已生效但字符左右间距/渲染位置仍偏左或偏右”的剩余问题,确认 matched shaped positions 仍可能把个别 glyph pen origin 推过 retained 1/8px raster phase 边界。`paint_text/draw/layout.rs` 恢复 shaped-position 接收路径的 phase guard:只有 glyph id/range/advance 匹配且 shaped origin 与 host natural origin 同 phase 时才使用 shaped origin,跨 phase 则回退 host natural spacing；同 phase 的 runtime shaped pen origin 仍可用。该切片只修 retained-host 低层 layout bridge,不改控件字体族、ZUI 资产、root painter、runtime FontDatabase、glyph atlas identity 或 native bitmap atlas cutover。
- `zircon_editor` retained-host same-phase origin drift guard:2026-07-07 进一步收窄上面的同相位规则。最新局部截图显示 same retained 1/8px phase 内的 `0.04px~0.05px` origin drift 仍会在小字号 DengXian label 上产生可见左右不适；`paint_text/draw/layout/metrics.rs::glyph_origin_matches_without_visible_drift(...)` 因此把可接受漂移限制为 `0.03125px`。`paint_text/draw/layout.rs` 的 shaped-position gate 与 runtime-advance projection gate 现在都要求 finite、无可见漂移、并仍共享 retained placement bin；`draw/layout/tests.rs` 保留 `0.02px` 合法 same-phase offset,并覆盖 same-phase visible drift fail-closed。direct editor test binary 已运行 proof 通过 1/1,并把真实 retained framebuffer/full-label/narrow-label PNG 写到 `docs/tests/runtime/text`;target/cargo-target 同名截图扫描为 0。该切片仍不改字体族、ZUI 资产、root painter、runtime FontDatabase、glyph atlas identity、native bitmap atlas cutover 或组件局部 letter-spacing；Cargo wrapper proof 仍无 `test result`,不声明 Cargo green。
- `zircon_editor` retained-host proof stem hook:2026-07-07 为上面的 same-phase origin drift guard 准备独立 framebuffer proof 归档。`paint_text_tests.rs::export_editor_crop_framebuffer_if_requested()` 现在在 `ZR_TEXT_EDITOR_CROP_PROOF_DIR` 外再接受 `ZR_TEXT_EDITOR_CROP_PROOF_STEM`,允许同一真实 `HostRgbaFrame` 导出写成本切片专属 PNG/log 名称,避免覆盖 20260705 crop evidence。focused Cargo proof 三次尝试均未产出 `test result`;随后 direct editor test binary 使用 same stem 通过 1/1,写出 framebuffer、full-label crop、narrow-label crop 与 metrics log 到 `docs/tests/runtime/text`,PNG SHA256 `8C81D6D27699ED503196F146636A3CF7EB51D202FF4E933AC96E6D1F17BD4E83` / `1C33579842EE9D0A912695219CDDA508BF247703151729C60A8EC93AD5365128` / `83B6CFDE5EAC92A9D2E349C605630484BC1FB5C3DA059F99D508D20B0E443339`。该切片只改测试证据落盘命名,不改变 glyph atlas identity、native bitmap atlas cutover、布局策略、字体族、ZUI 资产或 root painter。
- `zircon_editor` retained-host SubpixelMask sample phase:2026-07-05 继续关闭 retained CPU raster row 的相位分叉。`paint_text/draw/glyphs/row.rs::sampled_subpixel_coverage(...)` 之前在 `raster_scale > 1` 时没有消费 `sample_offset_x`,与 AlphaMask downsampling 不同；现在 RGB/SubpixelMask 也先执行 `normalized_sample_offset(...)`,再计算 supersampled x0/x1 窗口。`draw/glyphs/row/tests.rs::sampled_subpixel_coverage_applies_fallback_phase` 锁定 offset `0.0 -> [128,0,128]` 与 `0.5 -> [255,0,0]`,避免 fallback/放大 SubpixelMask 又回到未偏移采样窗口。该切片不改默认 grayscale Alpha coverage、ZUI 字体族、runtime FontDatabase、glyph atlas identity 或 native bitmap atlas cutover。
- `scene_renderer/ui/render/background.rs`:2026-07-05 继续收窄 SubpixelMask background composite 输入。`ScreenSpaceUiBackgroundTracker` 只记录前序不透明纯色 UI quad 的 visible frame/color candidate,并用后续透明背景、图片、文字或边框命令作为 blocker；`text_batch_background_color(...)` 保证文本自身透明/无效 `background_color` 仍保持 unknown,不会借用前序背景。父 `render.rs` 回到 767 行编排 owner,背景推断拆为 157 行 child owner；后续同日补齐 known framebuffer background 输入首段:空场景且 load-store 的 UI pass 可从不透明 `preview.clear_color` 继承背景,clear attachment 只接受不透明 clear color,一旦有 skybox/mesh/sprite/particle/visible overlay 或透明 UI blocker 仍保持 unknown。2026-07-06 follow-up 又把粒子否决从 CPU particle sprites 扩大到 emitters、previous sprites、bounds 与 GPU particle frame alive/spawned counters,避免 GPU 粒子仍在 framebuffer 中时误继承 clear color。该切片关闭 render-command background inference 与空场景 clear-background acquisition 首段,不伪造 framebuffer readback acquisition。
- 统一 page identity、page residency/LRU 决策、shelf 分配器、per-page dirty-rect 合并数据面、SDF page-keyed cache/upload report、SDF render partial upload、texture-array layer consumption、renderer-local WGPU texture write mapping owner、bitmap request+staging bytes upload binding owner、shelf overflow 多页 slot allocation、rebuilt-page full-dirty invalidation、over-cap/oversized allocation failure reporting、per-glyph failure reason mapping、glyph-level fallback span planning、Horizontal LTR/explicit RTL/no-wrap/non-justify mixed native overlay、unsupported mixed overlay diagnostics、whole-batch native fallback、通用 alpha/SDF atlas upload command owner、swash `GlyphBitmap` -> bitmap atlas source bridge 与 renderer-local bitmap atlas WGPU resource owner 已有首段；2026-07-05 已将 alpha-mask native bitmap glyph source 从 glyphon `TextArea`/`SwashCache` 喂入 `GlyphAtlasBitmapRenderer`,并执行 submission source bytes texture upload；同日 follow-up 已让 `TextArea.bounds` partial clip 在 source-feed 入口裁剪 alpha screen rect/content size/source bytes,已裁剪 alpha source 不再整批回退 glyphon；prepare-report follow-up 又把 `NativeBitmapAtlasPrepareReport` 接入 `ScreenSpaceUiTextPrepareReport.native_bitmap_atlas`,记录 visible/source/unsupported/clipped/submission 计数、单一 atlas storage format 与 mixed-storage fallback,并让 `replaces_glyphon()` 必须确认单 texture-array storage 不混用 R8/RGBA；RGBA source follow-up 已让 `SwashContent::Color` 走 `GlyphAtlasFormat::Color`/`Rgba8Unorm` source bytes 且前景乘子固定为 white,避免颜色字形被文本色二次染色；mixed-storage renderer cutover follow-up 已让 contiguous R8/RGBA frame 通过 per-storage renderer pass 关闭 glyphon；SubpixelMask background input follow-up 已将 UI command 自身不透明背景色传入 native bitmap atlas source/report,并记录 background-composite glyph 覆盖与缺失计数；inherited opaque UI background follow-up 又允许没有后续 blocker 的前序不透明纯色 UI quad 作为同一背景输入；latest replacement follow-up 已让已知不透明背景的 `SwashContent::SubpixelMask` 走 shader-composited RGB + WGPU REPLACE blend 并关闭 glyphon fallback,缺失/透明背景仍保留 glyphon native path。动态 framebuffer background acquisition、完整 glyphon `TextAtlas` 切换、持久化 glyph cache eviction 全链路、broader glyph-level mixed fallback(Vertical/Auto-Mixed/justify/wrapped)、independent oversized fallback、DPI 重栅格契约(scale 变不重栅格 → 放大像素化,`editor_layout/17 G2`)、subpixel 与 hinting 策略书面化仍未完成。

- `scene_renderer/ui/text/native_bitmap_atlas/source_cache.rs`:2026-07-05 新增原生 bitmap atlas source-image cache child owner。`NativeBitmapAtlasSourceCache` 按 glyphon/cosmic `CacheKey` 持久化 swash image content、bearing、尺寸与 source bytes,`ScreenSpaceUiTextBackend` 跨帧持有该 cache；`native_bitmap_atlas_frame(...)` 只有 miss 才调用 `SwashCache::get_image_uncached(...)`,并把 hit/miss/insert/evict/entry counters 写入 `NativeBitmapAtlasPrepareReport.source_cache`。同日 follow-up 已为字体资产集合变化增加 `discard_all_for_face_invalidation()` 与 `invalidated_count`,让 source-cache 旧 face source images 在下一帧前 fail-closed 清理并进入 report。这关闭了当前生产 native bitmap path 的 source 级重复栅格/重复 metadata 获取与 face-invalidation source-cache flush 首段,但真正 face validity requeue、全局 atlas slot invalidation、async worker 和 full glyphon `TextAtlas` cutover 仍未完成。

- `scene_renderer/ui/text.rs` + `text/native_bitmap_atlas.rs`:2026-07-05 继续修正生产 native bitmap atlas frame-loop 的帧戳输入。旧路径用固定 `BITMAP_ATLAS_FRAME_INDEX = 1` 调用 `glyph_atlas_bitmap_render_submission_plan(...)`,导致 blocked retry 与 page residency telemetry 始终只能表达 frame 1 -> retry 2。现在 `ScreenSpaceUiTextBackend` 持有递增 `bitmap_atlas_frame_index`,非空 native bitmap atlas frame、`NativeBitmapAtlasPrepareReport.frame_index` 与 per-storage submission 都使用同一个 live frame index。该切片只关闭生产提交帧号常量,仍不声明真实 retry-frame state execution、全局 atlas slot invalidation、async worker 或 glyphon `TextAtlas` cutover 完成。

- `scene_renderer/ui/text/native_bitmap_atlas/retry_frame.rs`:2026-07-05 将已有 `GlyphAtlasBitmapRetryFrameState` 接入生产 native bitmap atlas frame-loop。`ScreenSpaceUiTextBackend` 跨帧持有 retry state,字体面失效和空文本帧会清空 queue；native atlas 子 owner 只重试当前帧仍可见的 blocked source,把 source bytes 按 retry-aware submission input 顺序重映射,并丢弃已经不可见的 stale blocked source,避免旧 glyph 在后续帧被重新绘制。该切片关闭真实 renderer retry-frame state execution 的首段;全局 atlas slot invalidation、async worker、完整 glyphon `TextAtlas` cutover 与 live editor-window typography QA 仍 open。

- `scene_renderer/ui/text/native_bitmap_atlas/retry_frame.rs`:2026-07-06 stale retry selection telemetry follow-up 将旧 blocked source 的“本帧不可见而被丢弃”从隐式行为提升到 `NativeBitmapAtlasPrepareReport.discarded_stale_retry_glyph_count`。同一 queued blocked source 现在只会匹配一个当前 visible source,避免重复相同 source image 时把同一 retry 项消费两次；重复 visible source 中剩余项继续作为 new source 进入本帧提交。新增 `native_bitmap_atlas_retry_frame_does_not_reuse_one_blocked_source_twice`,并让 stale discard 回归断言 discarded count；`text/tests.rs` 的 aggregate prepare-report expectation 同步新字段默认值。验证日志 `docs/tests/runtime/text/runtime_text_native_bitmap_atlas_retry_stale_selection_validation_20260706.log` SHA256 `9961E678C812FBB79998B09EFBF0F430FB651EEEFABA46F43839BBD728254D01`；focused Cargo `native_bitmap_atlas_retry` 通过 5/5,日志 `docs/tests/runtime/text/runtime_text_native_bitmap_atlas_retry_focused_cargo_20260706.log` SHA256 `9ECCE49DCCAD9B5CE533EE1E2111A36D6F0F0F8A9AC066A226B0120F85368F3F`。该切片关闭 stale retry 可观测性和 duplicate retry consumption 风险,并补上 retry-frame focused Cargo 证据；不声明 async raster face-validity requeue、global slot invalidation、完整 glyphon `TextAtlas` cutover 或 live editor-window typography QA。

- `scene_renderer/ui/text/native_bitmap_atlas/retry_frame.rs` + `graphics/text/atlas/render_submission/frame_state.rs`:2026-07-05 retry face-invalidation report follow-up 让 `GlyphAtlasBitmapRetryFrameState` 不再把 face invalidation 清空 blocked retry queue 作为无声 drop。`discard_all_for_face_invalidation()` 累计 pending invalidated glyph count,visible frame 的 retry driver 和 `native_bitmap_atlas_idle_prepare_report(...)` 都用 `take_report()` 把该计数写入 `NativeBitmapAtlasPrepareReport.retry_state.invalidated_blocked_glyph_count` 后清零。新增 visible-frame 与 idle-frame 回归,验证图 `docs/tests/runtime/text/runtime_text_native_bitmap_atlas_retry_face_invalidation_preview_20260705.png` SHA256 `A02F00772E0908C0FCFB69F51DFE70BD353A23B7855A6D1583CF052E15EC505A`,验证日志 `docs/tests/runtime/text/runtime_text_native_bitmap_atlas_retry_face_invalidation_validation_20260705.log` SHA256 `C6C3B2CE2460E450C2163407A4995E8915E260A28F6D777A7C17544E1B2FD47D`。该切片只关闭 retry-state telemetry,不声明 async raster face-validity requeue、stale artifact requeue、full glyphon cutover 或 live editor-window typography QA。

- `graphics/scene/scene_renderer/ui/atlas_renderer/{renderer.rs,text.rs,tests.rs}`:2026-07-05 renderer face-invalidation follow-up 让生产 `GlyphAtlasBitmapRenderer` 在 font face/asset 变化时同步清空 active storage-pass draw/upload state,并在下一次 prepare report 中暴露 `invalidated_storage_pass_count`。`ScreenSpaceUiTextBackend` 在同一 face invalidation 分支同时清理 source cache、retry queue 与 renderer storage passes,避免旧 face 的 atlas draw command 在 source/raster cache 已失效后继续可见。验证图 `docs/tests/runtime/text/runtime_text_bitmap_renderer_face_invalidation_preview_20260705.png` SHA256 `5D35B7421413F7C8B1C47E4AAC5B25794D8BC2DFB27CC66730732091EF981CB1`,验证日志 `docs/tests/runtime/text/runtime_text_bitmap_renderer_face_invalidation_validation_20260705.log` SHA256 `15B149B884E73979E2B16CBE3B6A2D94735362B3F7E086E8FBD5CE12CD9E0FF1`。该切片关闭 renderer-local stale storage-pass telemetry,不声明 async raster face-validity requeue、global glyph slot eviction 或 live editor-window typography QA。

- `graphics/scene/scene_renderer/ui/atlas_renderer/resources.rs`:2026-07-05 针对最新 editor 截图中“等线已生效但字符左右间距/落点仍不舒服”的 atlas sampling 风险,bitmap atlas sampler 从 Linear min/mag 改为 Nearest min/mag/mipmap 并固定 LOD 0,与 glyphon bitmap cache 的 nearest sampling contract 对齐。`tests.rs` 新增 `glyph_atlas_bitmap_sampler_matches_glyphon_nearest_sampling_contract`,防止小字号 alpha/RGBA atlas texel 被线性过滤混入邻近字形左右边缘。验证图 `docs/tests/runtime/text/runtime_text_bitmap_atlas_nearest_sampler_preview_20260705.png` SHA256 `A8C071C64D89F6380CAC2D11B64970CD078051B9DD030AD3D1515395EF5C9A0B`,验证日志 `docs/tests/runtime/text/runtime_text_bitmap_atlas_nearest_sampler_validation_20260705.log` SHA256 `8734A1008CB635E072FFE89546368A579BAD4075DBEBABB6969031E747394DD0`。2026-07-06 focused Cargo `glyph_atlas_bitmap_sampler_matches_glyphon_nearest_sampling_contract` 通过 1/1,日志 `docs/tests/runtime/text/runtime_text_bitmap_atlas_nearest_sampler_focused_cargo_20260706.log` SHA256 `AD2A6E83F4D73F08C1A53404740E1148353D791F0F15AE9316B783FAE4BE5692`,exit `docs/tests/runtime/text/runtime_text_bitmap_atlas_nearest_sampler_focused_cargo_20260706.exit.txt` SHA256 `A9F58776A09B5DAC438049683F24BF85764E0FF8E7455952456165C68C158627`。该切片只关闭 runtime bitmap atlas GPU sampling phase/edge bleed 风险,不替代 retained-host crop live window QA、full glyphon `TextAtlas` cutover 或 broader LCD/gamma/background policy。

- `scene_renderer/ui/text/native_bitmap_atlas/handoff.rs`:2026-07-05 将 native bitmap atlas 的 glyphon/native handoff 判定从 `scene_renderer/ui/text.rs` 根实现移到 native bitmap atlas 子 owner。`NativeBitmapAtlasHandoff` 与 `native_bitmap_atlas_handoff_for_report(...)` 现在跟 `NativeBitmapAtlasPrepareReport` 同域维护,`text.rs` 只消费 single-storage replacement、mixed-storage replacement 与 glyphon fallback 决策,不再同时拥有 frame orchestration 和 cutover policy。原 handoff 回归同步迁入 `text/native_bitmap_atlas/tests.rs`,验证图 `docs/tests/runtime/text/runtime_text_native_bitmap_atlas_handoff_owner_preview_20260705.png` SHA256 `B97D06F24B38594DCECF485FEC38D27E825565D6AD9F48699476C22081901BDF`,验证日志 `docs/tests/runtime/text/runtime_text_native_bitmap_atlas_handoff_owner_validation_20260705.log` SHA256 `B64F35B3D2B3785602012ED91FF60550CA05FD0B85E0FC2A91D82B5B3AA223D9`。该切片是结构收束和 TextAtlas cutover 判定面整理,不声明行为变化、完整 glyphon `TextAtlas` cutover 或 live editor-window typography QA 完成。

- `scene_renderer/ui/text/native_bitmap_atlas/handoff.rs`:2026-07-05 fallback reason telemetry follow-up 在 `NativeBitmapAtlasPrepareReport` 上新增 `glyphon_fallback_reason`,由 native atlas handoff owner 按固定优先级写出 glyphon fallback 原因:`NoVisibleRasterGlyphs`、`UnsupportedGlyphFormat`、`IncompleteSourceCoverage`、`MissingBackgroundCompositeInput`、`AtlasAllocationFailure`、`MixedStorageSplitNotReady` 等。`text.rs` 的 handoff 分支不变,但 prepare report 不再只暴露 bool/count,可以区分 LCD 背景缺失、source 覆盖不完整、atlas 分配失败或 mixed storage split 未就绪。验证图 `docs/tests/runtime/text/runtime_text_native_bitmap_atlas_fallback_reason_preview_20260705.png` SHA256 `3B6A5965753EF9769E5CBCDAA1827F3EBF0A6A04C8D389260CF1B00CB65BB153`,验证日志 `docs/tests/runtime/text/runtime_text_native_bitmap_atlas_fallback_reason_validation_20260705.log` SHA256 `B541E92E405B52A5A8D66E79EB3BCB5E3159422EF3EDD88FFD45AA09D228C09C`。该切片关闭 fallback reason telemetry 首段,不声明完整 glyphon `TextAtlas` cutover、真实 framebuffer background acquisition 或 live editor-window typography QA 完成。

- `graphics/text/atlas/{page.rs,upload.rs,render_contract.rs,render_plan.rs,render_batch.rs}`:2026-07-03 已补 `GlyphAtlasSamplingSemantics`,让 `GlyphAtlasFormat::SubpixelMask` 与 `Color` 即使同用 RGBA8 atlas storage,也分别携带 `SubpixelCoverage` 与 `ColorRgba` 采样/混合语义；`GlyphAtlasPageSpec` 与 `GlyphAtlasUploadCommand` 均保留该语义,focused `render_text_atlas` 通过 14/14。随后 `render_contract.rs` 与 `atlas/shaders/glyph_atlas_sampling.wgsl` 接上 shader/blend contract owner:`SubpixelCoverage` 选择 `SubpixelRgbCoverage + SubpixelBackgroundComposite`,`ColorRgba` 选择 `ColorRgba + SourceRgba`,focused `render_text_atlas` 通过 18/18。最新 `render_contract.rs` 继续暴露 `GLYPH_ATLAS_TEXT_SHADER` 与 `GlyphAtlasShaderEntryPoints`,并通过 `atlas/shaders/glyph_atlas_pipeline.wgsl` 固定 `vs_main` 和 alpha/subpixel/SDF/MSDF/color fragment entry points。`render_plan.rs` 现在把 shared `GlyphRasterPlacement` 的 snapped x、clip 后屏幕矩形、按 glyph content size clamp 的 atlas UV、page layer、foreground/background color 与 `GlyphAtlasRenderContract` 绑定到同一 draw quad 数据面,focused draw-plan tests 通过 4/4；后续又把 `SubpixelBackgroundComposite` 的顶点背景输入规范为 finite/clamped RGB + opaque alpha,避免无效/半透明背景色放大 LCD 边缘偏色。`render_batch.rs` 继续把可见 draw quads 按 `(GlyphAtlasPageKey, GlyphAtlasRenderContract)` 分批,统计 visible/skipped glyph 与 vertex count,并阻止同为 RGBA8 storage 的 `SubpixelMask` 与 `Color` 混批。完整生产 bitmap atlas renderer、真实 GPU upload/draw、真实 framebuffer 背景获取/合成与 glyphon `TextAtlas` cutover 仍未接线。
- `graphics/text/atlas/bitmap_run.rs`:2026-07-04 新增 bitmap atlas run data-plane owner,把 AlphaMask/SubpixelMask/Color bitmap glyph source 分配到统一 `GlyphAtlasSet` pages,生成 dirty pages、draw glyphs 与 typed failures；随后 `upload_commands` 由同一 run owner 通过共享 `glyph_atlas_upload_command(...)` 生成,避免 renderer root 重复推导 partial/full upload。最新 upload-copy follow-up 让 `GlyphAtlasBitmapRunPlan.upload_copies` 持有 `GlyphAtlasBitmapUploadCopy`,按 source index 记录 page key、atlas rect、content size、source bytes/row、source byte len、atlas bytes/row 与 atlas byte offset,让未来 renderer staging buffer 可以按 run plan 拷贝 glyph bytes。follow-up 又在 `PageReservationBlocked` 时记录 `GlyphAtlasBitmapQueuedGlyph`,保留 source payload/source index 与最早 `retry_frame_index`,让后续占位渲染和下帧重试不再只依赖失败总数；placeholder follow-up 同步生成 `GlyphAtlasBitmapPlaceholderGlyph { mode: TransparentQuad }`,保留 blocked source 的 screen rect 与 retry frame,让本帧占位渲染有独立数据面。retry follow-up 新增 `bitmap_run/retry.rs`,用 `GlyphAtlasBitmapRetryPlan` 按 frame index 拆分 due/deferred blocked glyphs,并通过 `retry_sources()` 暴露可重新喂给下一次 atlas run 的源数据。结构 follow-up 将混合实现拆成 folder-backed owners:`bitmap_run/types.rs` 承接 source/glyph/run plan 声明,`failure.rs` 承接 typed failure 与 blocked retry queue,`placeholder.rs` 承接占位数据合同,`retry.rs` 承接下帧 retry queue 消费,`validation.rs` 承接源数据校验,`allocation.rs` 承接 page reservation/shelf allocation/dirty marking,`upload.rs` 承接 staging copy 与 dirty-page upload command projection,root `bitmap_run.rs` 降为 run orchestration + exports。`bitmap_run/tests.rs` 锁定格式分流、shelf overflow、失败原因、staging upload copy、dirty-page upload、full-page upload promotion、blocked retry queue、placeholder data-plane、retry queue consumer 和 draw-batch bridge。该切片仍是数据面,不替代生产 bitmap atlas renderer、真实 GPU upload/draw 或 glyphon `TextAtlas` cutover。
- `graphics/text/atlas/bitmap_run/staging.rs` 与 `bitmap_run/staged_upload.rs`:2026-07-04 继续关闭 bitmap atlas upload handoff。`staging.rs` 先把 `GlyphAtlasBitmapUploadCopy` + `GlyphAtlasBitmapUploadSourceBytes` 拷入 page-keyed full-page staging buffers,保留 source stride 与 atlas page stride；`staged_upload.rs` 再把这些 staging pages 与 `GlyphAtlasUploadCommand` 绑定为 `GlyphAtlasBitmapStagedUploadPlan`,显式给出 `staging_page_index`、command 和 staging page byte len,并 typed 报告 missing staging page / source range out-of-bounds。prepared follow-up 新增 `GlyphAtlasBitmapPreparedUploadPlan` 与 `glyph_atlas_bitmap_prepared_upload_plan(...)`,让 future renderer 以一个入口从 run plan + source bytes 得到 staging pages 与 staged uploads；若 source-byte staging 已失败,prepared plan 不再输出 staged uploads,避免把不完整 atlas page 交给 `Queue::write_texture`。最新 texture-request follow-up 新增 `GlyphAtlasBitmapTextureUploadRequestPlan`,把 staged uploads 投影为 WGPU-neutral texture write 字段:origin xy/layer、extent、source offset、bytes/row、rows/image 和 byte lengths。该 pair 是未来 renderer `Queue::write_texture` 消费的纯数据合同,避免在 renderer root、glyphon cutover 或 editor 消费路径重新计算 page/rect/stride/offset。
- `graphics/scene/scene_renderer/ui/atlas_texture_upload.rs`:2026-07-04 继续把 bitmap atlas upload handoff 留在 renderer-local leaf owner。`GlyphAtlasBitmapTextureUploadFramePlan` 现在从 prepared upload 出发,组合 texture upload request projection、staging-byte binding 与 `GlyphAtlasBitmapTextureUploadFrameReport`;只在 request/binding 存在且 staging、staged-upload、binding failure 全为 0 时标记 `ready_to_write_texture`。`write_glyph_atlas_bitmap_texture_upload_frame_plan(...)` 复用既有 binding writer,但 fail-closed 阻止不完整 staging page 进入 WGPU writer。该切片仍不生成 frame-loop glyph sources、不创建 bitmap texture-array 资源,也不替代 glyphon `TextAtlas` cutover。
- `graphics/text/atlas/{page.rs,page_residency.rs,upload.rs}` + bitmap upload handoff:2026-07-05 为 PF-M1 代际竞态补首段数据面。`GlyphAtlasPageSpec.generation` 标记页内容代际,`page_residency.rs` 在 LRU eviction/rebuild 复用同一 `GlyphAtlasPageKey` 时递增 generation；`GlyphAtlasUploadCommand`、`GlyphAtlasBitmapPageUploadStaging` 与 `GlyphAtlasBitmapTextureUploadRequest` 均携 `page_generation`。`glyph_atlas_bitmap_texture_upload_request_plan_with_atlas(...)` 按当前 `GlyphAtlasSet` 跳过 stale request,`atlas_texture_upload/binding.rs` 拒绝 staging/request generation mismatch,避免旧异步栅格产物写入已经易主的 texture-array layer。native source-cache face invalidation flush/report 已由 `scene_renderer/ui/text/native_bitmap_atlas/source_cache.rs` 首段关闭；face validity requeue、全局 atlas slot invalidation 和真实 async raster worker 仍在 09 后续。
- `graphics/scene/scene_renderer/ui/atlas_texture_upload/{write,binding,frame,resource}.rs`:2026-07-04 将 renderer-local upload handoff 从单文件混合 owner 拆成 folder-backed leaves。`write.rs` 只承接 `GlyphAtlasUploadCommand` 到 WGPU write fields 的字段投影和 `Queue::write_texture` 调用；`binding.rs` 只承接 bitmap texture upload request 与 staging page bytes 的 typed binding/failure；`frame.rs` 只承接 prepared upload frame readiness/report 与 fail-closed writer orchestration。最新 `resource.rs` 新增 `GlyphAtlasTextureArraySpec` 与 `create_glyph_atlas_texture_array_resources(...)`,把 R8/RGBA storage format 到 WGPU `TextureFormat`、D2Array view、extent clamp 与 COPY_DST/TEXTURE_BINDING usage 规则从 `sdf_render.rs` 私有 helper 抽出；SDF renderer 继续使用 R8Unorm 语义,但 texture-array 资源创建已走共享 owner。root `atlas_texture_upload.rs` 降为结构入口和窄导出,不再继续累积 request、binding、frame report、resource descriptor 与 WGPU write 细节。
- `graphics/scene/scene_renderer/ui/atlas_renderer/`:2026-07-04 新增 renderer-local bitmap atlas WGPU resource owner。`vertex.rs` 把 `GlyphAtlasGpuVertexBufferLayout` 映射为 wgpu `VertexBufferLayout`,并让 `GlyphAtlasGpuVertex` 通过 `Pod/Zeroable` 成为可安全上传的 vertex bytes；`resources.rs` 创建 texture-array/sampler bind-group layout、sampler、placeholder atlas texture/view/bind group；`pipeline.rs` 从 `GLYPH_ATLAS_TEXT_SHADER`、shader entry points、blend contract 与 primitive topology 创建 real wgpu pipeline；`renderer.rs` 缓存 pipeline resources、创建 vertex buffer、保存 draw commands 并在 render pass 中按 draw command 提交。`ScreenSpaceUiTextSystem` 已挂载 `GlyphAtlasBitmapRenderer`,先以 empty draw plan 进入 prepare/report/render no-op 路径；2026-07-05 起 alpha-mask native glyph sources/submission source bytes 已可进入 guarded upload/draw submission,且 replacement report 暴露是否实际替换 glyphon。该 owner 仍未完成 Color/SubpixelMask source、mixed storage texture-array cutover、persistent frame-loop glyph cache/eviction 或完整 glyphon `TextAtlas` 替换。
- `graphics/scene/scene_renderer/ui/atlas_renderer/{renderer.rs,resources.rs}`:2026-07-04 follow-up 修复 editor screenshot 编译链路发现的两个底层阻塞:bitmap atlas renderer 类型/方法提升到 UI 子系统可见范围,并在 atlas resources 内使用 `glyph_atlas_gpu_bind_group_layout()` 填充 texture/sampler binding,避免把 wgpu bind group layout 参数误当 glyph atlas layout。
- `graphics/text/atlas/render_submission.rs`:2026-07-04 新增 bitmap atlas render submission data-plane owner,把 bitmap run、upload commands、clipped draw batches 与 `GlyphAtlasGpuDrawPlan` 聚合成 `GlyphAtlasBitmapRenderSubmissionPlan`。随后补 `GlyphAtlasBitmapRenderSubmissionReport`,统一汇总 source/allocation/failure、dirty/rebuilt page、full/partial upload、upload byte、draw batch、pipeline、GPU batch、draw command、vertex 与 upload/GPU/failure readiness flags,让 renderer/glyphon cutover 前可先检查提交质量。follow-up 又把 failure diagnostics 从单一总数拆成 UnsupportedFormat/EmptyContent/DataLengthMismatch/PageReservationBlocked/OversizedGlyph 五类 counter,并提供 source-validation 与 atlas-capacity 两个汇总 helper,同时暴露 `blocked_retry_count` 与 `next_retry_frame_index`,让后续 renderer/telemetry 能区分源数据错误、atlas 容量压力和下帧重试压力。最新 placeholder draw-plan follow-up 新增 `render_submission/placeholder.rs`,将 blocked placeholder glyphs 按同一 `GlyphAtlasScreenRect::clipped_to(...)` 生成 renderer-facing visible/skipped placeholder draw plan,并把 `visible_placeholder_count`/`skipped_placeholder_count` 汇入 report。retry backpressure follow-up 又让 `bitmap_run/retry.rs` 通过 `GlyphAtlasBitmapRetryBackpressurePolicy` 限制每帧 due retry source 数,并让 `render_submission/retry.rs` 报告 `backpressured_retry_count`,避免未来 renderer/frame-loop 在 root 层重复节流 blocked queue。`render_submission/tests.rs` 锁定 visible glyph → GPU draw data、clipped glyph → upload-only、full-page upload 计数、invalid bitmap source → diagnostics-only、same-frame page eviction blocked、blocked retry report、mixed failure breakdown、clipped placeholder draw-plan 与 backpressured retry report 等 renderer handoff 语义。该 owner 仍不创建 wgpu texture/pipeline/bind group,不替代真实 GPU upload/draw 或 glyphon `TextAtlas` cutover。
- `graphics/text/atlas/render_submission/frame_state.rs`:2026-07-04 新增跨帧 retry state owner。`GlyphAtlasBitmapRetryFrameState` 持有 renderer root 之下的 blocked glyph queue,用 queued glyphs + 本帧 new sources 生成 retry-aware submission plan,再通过 `apply_submission_plan(...)` 把 `GlyphAtlasBitmapRetryFrameSubmissionPlan.frame_outcome.next_blocked_glyphs` 写回下一帧状态。该 owner 只承担 frame-loop handoff 数据面,避免未来 root 层复制 deferred/backpressured commit、source-index remap 或 earliest retry frame 统计；真实 renderer frame-loop execution、wgpu upload/draw 与 glyphon `TextAtlas` cutover 仍未接线。
- `graphics/text/atlas/render_submission/frame_driver.rs`:2026-07-04 新增 retry frame driver owner。`GlyphAtlasBitmapRetryFrameDriverConfig` 固定一帧的 atlas page/viewport/clip/backpressure 参数,`glyph_atlas_bitmap_retry_frame_driver_submit_with_config(...)` 让 future renderer/frame-loop 用一个入口完成 state → retry submission plan → outcome commit,并返回 `GlyphAtlasBitmapRetryFrameDriverOutput` 给 telemetry/render handoff。该 owner 仍是 renderer handoff 数据面,不创建 wgpu texture/pipeline/bind group,不替代真实 GPU upload/draw、生产 bitmap atlas renderer 或 glyphon `TextAtlas` cutover。
- `graphics/text/atlas/render_gpu_plan.rs`:2026-07-03 已新增 bitmap atlas GPU vertex layout contract、viewport transform contract、draw-command contract、pipeline/bind-group contract 与 shader-entry contract 数据面。`GlyphAtlasGpuVertexBufferLayout` 固定 stride=52 bytes,并记录 shader location 0..4 对应 `position_ndc@0`、`uv@8`、`foreground_color@16`、`background_color@32`、`page_index@48`;`GlyphAtlasGpuViewportTransform` 记录 viewport size 与 `PixelEdges` 坐标约定,并显式把像素边界坐标映射到 NDC:左上 `[0,0] -> [-1,1]`,中心 `[w/2,h/2] -> [0,0]`,右下 `[w,h] -> [1,-1]`,空 viewport 以 1px extent 兜底。`GlyphAtlasGpuDrawPlan` 现在随 vertices/batches 一起携带 layout、viewport transform、`GlyphAtlasGpuDrawCommand` 列表、固定 texture-array/sampler bind-group layout 与 unique `GlyphAtlasGpuPipelineContract` 列表；每条 draw command 显式携带 render contract、`TriangleList` topology、vertex range、quad/triangle count 语义、atlas layer 与 pipeline key。`GlyphAtlasGpuPipelineContract` 同步携带 `shader_entry_points`,后续 renderer/shader 接线不再需要从 Rust struct 字段顺序、私有 helper、batch 顺序、shader include 或临时常量重新推导属性布局、半像素约定、draw range/layer、texture binding、fragment entry point 或 pipeline state。本轮按结构规范把该 mixed GPU contract umbrella 拆为 folder-backed child owners:`render_gpu_plan/vertex.rs`、`bind_group.rs`、`viewport.rs`、`draw_command.rs` 与 `pipeline.rs`,随后继续把既有 GPU draw-plan tests 搬到 `render_gpu_plan/tests.rs`,root `render_gpu_plan.rs` 降为 draw-plan assembly + module exports。该切片回应编辑器 tab 截图中小字号文字局部左右落点不稳的 GPU 输入侧风险,但不替代真实 glyphon atlas cutover 或窗口级 editor typography QA。

## 3. 参考代码

| 文件 | 应重点阅读 |
|------|-----------|
| `dev/bevy/crates/bevy_text/src/font_atlas.rs` | `FontAtlas { dynamic_texture_atlas_builder, glyph_to_atlas_index, texture }`;`add_glyph_to_atlas`(栅格→装箱→页满建新页);`get_outlined_glyph_texture`;padding=2。**Rust 图集落地主样板** |
| `dev/bevy/crates/bevy_text/src/font_atlas_set.rs` | `FontAtlasKey { font_size_bits, variations_hash, hinting, font_smoothing }`——**图集缓存键含 scale/hinting 的权威**;按 (font, size) 分图集 |
| `dev/Fyrox/fyrox-ui/src/font/mod.rs` | `Page` + `RectPacker` + `FontGlyph { bitmap_top/left, tex_coords, page_index }`,多页扩展、字形过大处理(`GlyphTooLarge`) |
| `dev/UnrealEngine/.../Fonts/FontTypes.h` | `FSlateFontAtlas`/`FSlateTextureAtlas`:动态装箱、按内容类型(Alpha/ColorBgra/MCDF)分纹理、`FAtlasedTextureSlot` 链表分配 |
| `dev/UnrealEngine/.../Fonts/SlateFontRenderer.cpp` | FreeType 栅格、subpixel、`FCharacterRenderData`(像素 + bearing);hinting/LCD 过滤 |
| `dev/UnrealEngine/.../Fonts/FontRasterizationMode.h` | `EFontRasterizationMode::{Bitmap,Msdf,Sdf}`——栅格模式枚举对照 |
| `dev/slint/.../textlayout/sharedparley.rs` | parley + swash 栅格缓存的轻量组织 |

**Rust/wgpu 落地**:swash `ScaleContext`/`Scaler`/`Render::new(&[Source::ColorOutline, Source::ColorBitmap, Source::Outline]).format(Format::Alpha|SubpixelMask)`(bevy `font_atlas.rs` 同款);`etagere`(shelf/guillotine 装箱,可选)。`render/14` §目标架构已定 shelf 分配器 + 1024×1024 R8 页 + padding 2 + 页级 LRU。

## 4. 目标架构

```
ShapedGlyph(glyph_id, font_id, style{size, scale, format}) →
  GlyphRasterKey { face, glyph_id, px_size_bucket, subpixel_bin, format, hinting } →
    [miss] swash rasterize(物理像素) → GlyphBitmap(R8 / RGBA8) →
      shelf alloc(按行高分桶) → page(脏矩形累积) → GPU upload(每页≤1次/帧) →
        GlyphAtlasRef { page, format, uv_min/max, bearing, px_size }
```

`GlyphAtlasSet` 持两组页(alpha/color)× 两格式(bitmap/SDF——SDF/MSDF 烘焙见 `05`,装箱共用本服务)。

## 5. 里程碑

### AT-M1 swash 栅格 + shelf 图集(替换 glyphon 自管)

实施切片:
1. `graphics/text/atlas/`:shelf 分配器、页(1024×1024,R8/RGBA8)、脏矩形上传(graph 资源节点声明 IO)、页级 LRU。
2. `graphics/text/raster/swash/`:swash 栅格隔离层(alpha + 彩色 emoji);bearing/px_size 提取。
3. UI 文本绘制改消费 `GlyphAtlasRef`(从 glyphon 自管 atlas 切到统一 atlas);glyphon 退为"按 atlas 坐标画 quad"或整体由 `render/14` 的 sprite 批接管。

测试:`render_text_atlas_shelf_allocates_same_height_into_one_row`、`render_text_atlas_evicts_lru_page`、`text_raster_swash_emoji_rgba_glyph`。

### AT-M2 DPI 重栅格 + subpixel + hinting

实施切片:
1. atlas key 含 `px_size_bucket`(`logical_px × scale_factor` 量化)与 `subpixel_bin`(水平 1/3 或整像素吸附);scale 变换触发重栅格(接 `editor_layout/17 §3.2`(2026-07-02 评审收口:原引 §3.4 为指错节勘误))。
2. hinting 策略:`HintingMode::{None,Vertical,Full}`(默认 Vertical,对小字号清晰);font_smoothing 开关。(2026-07-02 评审收口)`font_smoothing`(灰度 AA vs subpixel AA vs 无平滑)必须进栅格键:要么在 `GlyphRasterKey` 上独立加 `font_smoothing` 字段(bevy `FontAtlasKey` 同款),要么并入 `HintingMode` 枚举维度并在文档注明;SDF/MSDF 路径 hinting 恒为 `None`(距离场栅格不做 grid-fitting,键侧固定,避免同 glyph 因 hinting 维度产生无意义的 SDF 重复烘焙)。
3. 整像素吸附:文本/1px 边框整像素吸附(`render/14`/`editor_layout/21 §3.5`),自由内容不吸附。

测试:`text_atlas_key_rebuckets_on_scale_change`、`text_raster_subpixel_bins_distinct`、`render_text_dpi_rerasterize_at_2x_sharp`。

### AT-M3 脏矩形增量上传定稿

实施切片:
1. 启用脏矩形/脏槽增量上传(现有 `sdf_upload.rs` DirtySlots 设计落地);每页本帧新增 glyph 合并为最小覆盖矩形,单次 `write_texture`。
2. 过大字形(超页)降级:大字号走 SDF(`05`)或独立纹理。

测试:`render_text_atlas_partial_upload_merges_dirty_rects`、`text_atlas_oversized_glyph_falls_back_to_sdf`。

## 6. 工程落地细化(实施权威)

### 模块与文件落点

实现层 `zircon_runtime/src/graphics/text/`:

| 文件 | 内容 |
|------|------|
| `atlas/mod.rs` | atlas 子 owner 导航与 crate-private 导出 |
| `atlas/page.rs` | `GlyphAtlasFormat`、`GlyphAtlasPageKey`、`GlyphAtlasPageSpec`、`GlyphAtlasRect`、`GlyphAtlasSet` |
| `atlas/page_residency.rs` | 页驻留/LRU 数据面:每格式页上限、缺页分配、最旧未引用页逐出、本帧引用页保护；SDF page registration 已首段消费 |
| `atlas/shelf_allocator.rs` | shelf 行分配(行内 x 递增,padding=2;首段已由 SDF page[0] 消费,并在 SDF shelf overflow 时通过 `reserve_page_for_format` 分配 page[1+]) |
| `atlas/dirty.rs` | per-page 脏矩形合并数据面；SDF render path 已首段消费 dirty rect 做局部 `write_texture` |
| `atlas/upload.rs` | 首段持有通用 alpha/SDF/MSDF upload command math:None/FullPage/PartialRect、storage-format byte stride、source range 校验；后续接 graph/resource IO,每页≤1 次/帧 |
| `raster/mod.rs` | 栅格调度:format/policy 选 swash vs SDF(05) |
| `raster/swash/` | **swash 唯一隔离层** —— `bitmap.rs`/`request.rs`/`rasterizer.rs`/`color_strike.rs`/`error.rs`;alpha + 彩色;出口 `GlyphBitmap`。(2026-07-02 评审收口)彩色字形源优先级:COLR/CPAL **优先走矢量栅格**(分层 outline 着色,任意尺寸清晰);无 COLR 时回退位图表 CBDT/sbix——strike 选择取 **≥目标物理尺寸的最近 strike** 下采样(避免放大模糊),bearing/advance 按 strike ppem 与目标尺寸的比例换算;测试 `text_raster_emoji_strike_selection` |
| `raster/policy.rs` | `raster_path_for`(承接 `ui/text/raster.rs`):按字号/格式/face 选路径 |

### 核心类型与键

```rust
pub struct GlyphRasterKey {
    pub face: InstancedFaceId,   // 含变量轴(01)
    pub glyph_id: u16,
    pub px_size_bucket: u32,     // round(logical_px × scale_factor / QUANT) × QUANT
    pub subpixel_bin: u8,        // 0..3 水平 1/3 量化(整像素吸附时恒 0)
    pub format: GlyphAtlasFormat,// AlphaMask | Sdf | Msdf | Color
    pub hinting: HintingMode,
    pub synthetic: SyntheticFlags, // (2026-07-02 评审收口)合成样式:bold=swash embolden,oblique=quad shear;影响像素,不进键即污染缓存
}
pub enum HintingMode { None, Vertical, Full }
// (2026-07-02 评审收口)合成 bold/italic 标志(06 fallback 无真实 bold/italic face 时启用):
bitflags! { pub struct SyntheticFlags: u8 { const BOLD = 1; const OBLIQUE = 2; } }
pub struct GlyphBitmap { pub size: UVec2, pub bearing: Vec2,
    pub data: Vec<u8>, pub channels: u8 /*1=R8,4=RGBA8*/ }
// GlyphAtlasRef 见 render/14(page/format/uv_min/max/bearing/px_size)
```

bevy 对照:`FontAtlasKey { font_size_bits, variations_hash, hinting, font_smoothing }` → 本仓 `GlyphRasterKey`(加 subpixel_bin + format)。

### 分辨率精度规则(接 `editor_layout/17 G2`)

1. **物理像素栅格**:栅格尺寸 = `logical_px × scale_factor`,量化到 `QUANT`(默认 1px;高频缩放场景可设更粗桶避免抖动)。
2. **scale 变即重栅格**:`scale_factor` 改 → `px_size_bucket` 变 → key miss → 重栅格;旧桶页随 LRU 自然逐出。
3. **subpixel**:水平方向 3 个 bin(0/⅓/⅔);竖排或整像素吸附时关闭(`subpixel_bin=0`)。文本基线整像素吸附避免抖动。(2026-07-02 评审收口)量化时机与归属:`subpixel_bin` 在 **render extract / glyph quad 生成阶段**由每 glyph 最终屏幕 x 的小数部分量化得到(布局阶段不定 bin——同一 `LaidOutText` 平移后 bin 会变);量化后 glyph quad 位置**吸附到 bin 起点**(x = floor(x) + bin/3),保证栅格位图与落点严格一致。逻辑落点:`raster/policy.rs`(bin 量化函数)或 `render/14` glyph_quads 生成处。
4. **bitmap vs SDF 边界**(`raster/policy.rs`):小字号(≤ ~32px 物理)走 bitmap(更锐利);大字号/可缩放/3D 空间文本走 SDF/MSDF(`05`,分辨率无关、省重栅格)。彩色 emoji 恒 bitmap RGBA。

### 图集分配(shelf,对照 bevy + `render/14`)

- 行高桶:glyph 高度向上取整到 8px;同桶进同 shelf 行;行内 x 递增,glyph 间 padding=2(防双线性渗色)。
- 页:1024×1024;每格式每色组上限 8 页。
- 逐出:页级 LRU——glyph 命中刷新页帧戳;页满且需新页时,逐出最旧**未被本帧引用**页,整页清空重建(glyph 映射一并失效,UE flush 风格,不逐字搬迁)。
- (2026-07-02 评审收口)**blocked 策略**:页满且**全部页本帧被引用**(residency 决策返回 blocked)时,不强制逐出——新增 glyph 进入排队队列延到下帧分配,本帧以占位渲染(.notdef 框或透明 quad,不 panic);blocked 次数与排队 glyph 数计入 `render_perf_text_*` 计数器,持续 blocked 说明页上限/页面尺寸需扩容。配套约束(D3):`GlyphAtlasRef` 只允许**帧内短生命周期**持有——atlas 槽位在 render extract/quad 生成阶段按 `GlyphRasterKey` 现查,跨帧必须重查,禁止把 `GlyphAtlasRef` 缓存进 `ShapedGlyph`/布局结果等长生命周期结构(否则页重建后成悬垂引用)。
- 过大字形:超页尺寸 → 降级 SDF 或独立纹理(`GlyphTooLarge` 对照 Fyrox)。

### 与既有路径硬切换

| 现有 | 切换 |
|------|------|
| glyphon `TextAtlas`/`SwashCache` 自管栅格装箱 | 切 `GlyphAtlasSet` + `raster/swash/`;glyphon 退为坐标画 quad,或由 `render/14` sprite 批接管 glyph quad |
| `ui/sdf_atlas.rs` 固定 64×64/256 槽 | 统一进 `atlas/`(SDF 页与 alpha 页同分配器,见 05);保留语义,改 shelf |
| `ui/text/raster.rs` 策略 | 迁 `raster/policy.rs`;签名保留 |

### 测试与验收清单

| 测试 | 断言 |
|------|------|
| `render_text_atlas_shelf_allocates_same_height_into_one_row` | 同高桶 glyph 同 shelf 行,x 递增,padding=2 |
| `render_text_atlas_evicts_lru_page` | 页满逐出最旧未引用页;本帧引用页不可逐出 |
| `render_text_atlas_partial_upload_merges_dirty_rects` | 本帧新增 glyph 合并为最小矩形,每页 1 次上传 |
| `text_raster_swash_emoji_rgba_glyph` | 彩色 emoji 栅格为 RGBA8,落 color 页 |
| `text_atlas_key_rebuckets_on_scale_change` | scale 1.0→2.0 致 px_size_bucket 变、key miss、重栅格 |
| `text_raster_subpixel_bins_distinct` | 3 个 subpixel bin 产不同位图;吸附模式恒 bin0 |
| `render_text_dpi_rerasterize_at_2x_sharp` | 2x 下字形物理像素=2×逻辑,非放大模糊(抓帧) |
| `text_atlas_oversized_glyph_falls_back_to_sdf` | 超页字形降级 SDF,不 panic |
| `text_raster_emoji_strike_selection` | (2026-07-02 评审收口)CBDT/sbix 选 ≥目标尺寸最近 strike 下采样,bearing 按比例换算;COLR 存在时优先矢量 |
| `render_text_atlas_blocked_queues_glyph_to_next_frame` | (2026-07-02 评审收口)全页本帧引用时新增 glyph 排队下帧+占位渲染,blocked 计数进 `render_perf_text_*`,不 panic |

里程碑命令:`cargo test -p zircon_runtime render_text_atlas --locked`、`text_raster --locked`。

## 7. 风险与回退

- glyphon 深度耦合:若一步切走 glyphon 风险大,AT-M1 可先让 glyphon 与 `GlyphAtlasSet` 并存(glyphon 仅 latin 快路径),AT-M3 后全切;但不留双布局路径(布局恒走 02/03)。
- subpixel 与 wgpu 混合:subpixel AA 需特殊 blend,V1 用整像素 + 灰度 AA(覆盖多数场景),subpixel 为 feature。

## 8. 状态与产出记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

当前概述（2026-07-11）：真实 runtime 产品帧缓冲继续通过 native glyphon/cosmic bitmap/color atlas 绘制中文、RTL 与彩色 Emoji，并直接锁定 `Segoe UI Emoji`、`SwashContent::Color` 与 RGBA 字节合同。本轮又让 SDF atlas key 保留 shaping backend 的实际 glyph id + face id，`sdf_atlas/text_keys.rs` 独立拥有 key 收集，避免 renderer root 或 `sdf_atlas.rs` 堆入第二套字形身份推导；atlas owner 精确测试 23/23、font bake 10/10。native bitmap atlas 的 mixed-storage handoff 现在按原始绘制顺序拆分连续 storage runs，`R8 -> RGBA -> R8` 可生成三个有序 renderer passes，不再因同一格式重复出现而退回 glyphon；当前源 native atlas 44/44、atlas renderer 13/13。动态 framebuffer 背景获取、完整 glyphon atlas 硬切、DPI 重栅格、persistent native glyph-slot 全闭环与 live editor-window typography QA 仍未关闭。

本子计划产出记录已超过 10 条，具体记录已迁入编号子目录。

- 迁入记录：[`04/2026-07-09-glyph-atlas-and-rasterization-output-records.md`](04/2026-07-09-glyph-atlas-and-rasterization-output-records.md)
