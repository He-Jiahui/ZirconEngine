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
  - zircon_runtime/src/text/atlas/bitmap_run/tests/persistent_slots.rs
  - zircon_runtime/src/text/atlas/bitmap_run/types.rs
  - zircon_runtime/src/text/atlas/bitmap_run/upload.rs
  - zircon_runtime/src/text/atlas/bitmap_run/validation.rs
  - zircon_runtime/src/text/atlas/page.rs
  - zircon_runtime/src/text/atlas/page_shadow/mod.rs
  - zircon_runtime/src/text/atlas/page_shadow/commit.rs
  - zircon_runtime/src/text/atlas/page_shadow/patch.rs
  - zircon_runtime/src/text/atlas/page_shadow/shadow.rs
  - zircon_runtime/src/text/atlas/page_shadow/store.rs
  - zircon_runtime/src/text/atlas/page_residency.rs
  - zircon_runtime/src/text/atlas/page_residency/tests.rs
  - zircon_runtime/src/text/atlas/render_contract.rs
  - zircon_runtime/src/text/atlas/render_contract/tests.rs
  - zircon_runtime/src/text/atlas/render_plan.rs
  - zircon_runtime/src/text/atlas/render_plan/tests.rs
  - zircon_runtime/src/text/atlas/render_batch.rs
  - zircon_runtime/src/text/atlas/render_batch/tests.rs
  - zircon_runtime/src/text/atlas/render_gpu_plan.rs
  - zircon_runtime/src/text/atlas/render_gpu_plan/instance.rs
  - zircon_runtime/src/text/atlas/render_gpu_plan/viewport.rs
  - zircon_runtime/src/text/atlas/render_gpu_plan/tests.rs
  - zircon_runtime/src/text/atlas/render_gpu_plan/bind_group.rs
  - zircon_runtime/src/text/atlas/render_gpu_plan/draw_command.rs
  - zircon_runtime/src/text/atlas/render_gpu_plan/pipeline.rs
  - zircon_runtime/src/text/atlas/render_submission.rs
  - zircon_runtime/src/text/atlas/render_submission/placeholder.rs
  - zircon_runtime/src/text/atlas/render_submission/frame_driver.rs
  - zircon_runtime/src/text/atlas/render_submission/frame_state.rs
  - zircon_runtime/src/text/atlas/render_submission/retry.rs
  - zircon_runtime/src/text/atlas/render_submission/tests.rs
  - zircon_runtime/src/text/atlas/shaders/glyph_atlas_sampling.wgsl
  - zircon_runtime/src/text/atlas/shaders/glyph_atlas_pipeline.wgsl
  - zircon_runtime/src/text/atlas/shelf_allocator.rs
  - zircon_runtime/src/text/atlas/slot_cache.rs
  - zircon_runtime/src/text/atlas/slot_cache/tests.rs
  - zircon_runtime/src/text/atlas/dirty.rs
  - zircon_runtime/src/text/atlas/dirty/tests.rs
  - zircon_runtime/src/text/atlas/upload.rs
  - zircon_runtime/src/text/atlas/upload/tests.rs
  - zircon_runtime/src/text/atlas/raster_key/mod.rs
  - zircon_runtime/src/text/atlas/raster_key/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/resolved_batches.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/resolved_batches/auto_route.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/prepare_report.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/tests/rendering.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/tests.rs
  - zircon_runtime/src/text/native_bitmap_atlas.rs
  - zircon_runtime/src/text/native_bitmap_atlas/source_cache.rs
  - zircon_runtime/src/text/native_bitmap_atlas/source_cache/lru.rs
  - zircon_runtime/src/text/native_bitmap_atlas/storage.rs
  - zircon_runtime/src/text/native_bitmap_atlas/retry_frame.rs
  - zircon_runtime/src/text/native_bitmap_atlas/tests.rs
  - zircon_runtime/src/text/native_bitmap_atlas/tests/frame.rs
  - zircon_runtime/src/text/native_bitmap_atlas/tests/handoff.rs
  - zircon_runtime/src/text/native_bitmap_atlas/tests/retry_frame.rs
  - zircon_runtime/src/text/native_bitmap_atlas/tests/source.rs
  - zircon_runtime/src/text/native_bitmap_atlas/tests/source_cache.rs
  - zircon_runtime/src/text/native_bitmap_atlas/tests/source_cache/residency.rs
  - zircon_runtime/src/text/native_bitmap_atlas/tests/storage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/font_id_report.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/sdf_fallback.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_texture_upload.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_texture_upload/write.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_texture_upload/binding.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_texture_upload/frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_texture_upload/resource.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_texture_upload/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_renderer/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_renderer/instance.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_renderer/instance_buffer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_renderer/pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_renderer/resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_renderer/state.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_renderer/renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_renderer/product_framebuffer.rs
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
  - zircon_runtime/src/text/raster/service/mod.rs
  - zircon_runtime/src/text/raster/service/face.rs
  - zircon_runtime/src/text/raster/service/glyph_raster_service.rs
  - zircon_runtime/src/core/framework/text/glyph_raster/mod.rs
  - zircon_runtime/src/core/framework/text/glyph_raster/request.rs
  - zircon_runtime/src/core/framework/text/glyph_raster/receipt.rs
  - zircon_runtime/src/ui/surface/text_artifact.rs
  - zircon_runtime/Cargo.toml
  - zircon_runtime_interface/src/ui/surface/render/command.rs
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
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/glyphs.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/glyphs/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/glyphs/row.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/glyphs/row/tests.rs
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
status: runtime_context_owned_policy_exact_cache_vertical_capability_and_geometry_guards_source_implemented_managed_validation_pending
---

# 04 字形栅格化 / 字形图集 / 分辨率精度

> 本计划把 `02` 的 `ShapedGlyph.glyph_id` 栅格成像素并装进 GPU 图集。它是 `editor_layout/17 G2`(字形随 DPI 重栅格,根治像素化)的实现,统一现有 glyphon bitmap atlas 与 SDF atlas 的分配/上传策略。

## 1. 目标

1. **栅格器选型与统一**:bitmap 路径用 swash(彩色 emoji + outline alpha + subpixel);SDF/MSDF 路径见 `05`。栅格输入按物理像素。
2. **图集化生成**:shelf(货架行)分配器、多页管理、脏矩形增量上传、页级 LRU 逐出;`R8Unorm`(alpha/SDF)与 `Rgba8Unorm`(彩色/MSDF)分组分页。
3. **分辨率精度**:`physical_px = logical_px × scale_factor`;scale 变即重栅格;subpixel 定位(水平 1/3 量化或整像素吸附);hinting 策略;atlas key 含 scale 量化桶。
4. **统一图集服务**:UI 与场景 2D 共用 `GlyphAtlasSet`(`render/14` 已起名),替换现有各自为政的 glyphon `TextAtlas` 与 `sdf_atlas`。

## 2. 现状与差距

### 2026-08-24 当前实现状态: native 输入硬切换, 验证进行中

`ScreenSpaceUiTextSystem` 的 bitmap 路径现在只接收由 canonical
`TextLayoutService` 投影的 `TextGlyph` 数据: glyph id、精确
`InstancedFaceId`、advance、offset、baseline 与已裁剪的屏幕范围。它不再接收
原始字符串、glyphon `Buffer` 或 `TextArea`, 因而不可能在 renderer 边界重新
shape 文本。`native_bitmap_atlas/glyph_run.rs` 是该 renderer-facing 输入 DTO;
`native_glyph_run.rs` 只做屏幕投影和精确 `GlyphRasterKey` 构造，并将同一批已解析
font handles 同时投影为 fallback font-id 诊断，避免第二次 O(G) registry 查询;
`source_cache.rs` 从 `FontDatabase` 的同一实例取得 face bytes、face index 与
variation coordinates 后直接创建 Swash 请求。

旧 `native_buffer.rs`、`native_bitmap_atlas/text_area.rs` 和本地 glyphon cache-key
适配层已从模块树移除。首帧缺少位图时的实际 UI 行为是透明占位并在下一帧重试,
不是 glyphon renderer 回退。旧报告字段的命名迁移不构成一个第二渲染路径。

当前验证状态为 **进行中，但受协调器产物治理阻塞**: 首次受管 Windows Cargo lane 已结束但
终端代理未保留退出回执；后续 lane 被协调器以共享
`F:\cargo-targets\zircon-engine\ephemeral` 未登记历史产物拒绝。该共享根目录不属于
本计划可安全清理的目标，已写入会话协调警告。完成前不得声称性能、功耗或 WGPU 视觉验收通过。随后必须
运行真实 WGPU 产品夹具, 其 PNG 仅可输出到
`docs/tests/runtime/text/runtime_text_native_bitmap_layout_product_framebuffer_20260802.png`。

- `graphics/.../ui/text.rs`:历史上该路径自管 glyphon `TextAtlas` 与 `SwashCache`,造成 renderer 侧二次 shape 和不可控的 atlas ownership。当前已硬切到 `NativeBitmapAtlasGlyphRun` 输入与 Zircon `GlyphAtlasSet`; `text_area`/native buffer 入口已删除。余下差距仅限受管 Cargo、WGPU 截图和性能采样验收,不再是生产 glyphon atlas cutover。
- `ui/sdf_atlas.rs`:自有 SDF cache 已从固定单页扩展到统一 `GlyphAtlasSet` 的 SDF page identity + shared shelf rect + page residency/LRU 数据面。2026-07-02 已让 `SdfAtlasPlan` 持有 `text/atlas::GlyphAtlasSet` 的 `Sdf` page identity,并用共享 shelf allocator、dirty-rect owner 与 page residency owner 生成 slot rect/cache report/upload report 数据面；`SdfAtlasSlot.page_key`、`SdfAtlasCacheReport.dirty_pages` 与 `SdfAtlasUploadReport.dirty_pages` 已补齐 page-keyed dirty/upload 数据面,同时保留 page[0] `dirty_rect` 兼容字段；`sdf_upload.rs`/`sdf_render.rs` 已完成 SDF dirty-rect `Queue::write_texture` partial upload,并让 renderer/texture owner 消费所有 page-keyed upload commands 到 `texture_2d_array` layer；`text/atlas/upload.rs` 首段已接管通用 upload command math；`text/atlas/page_residency.rs` 首段已接管每格式页上限、缺页分配、最旧未引用页逐出与全页受保护阻塞的 LRU 决策数据面；shelf overflow 现在不再放大单页 atlas,而是在固定 page size 内分配,溢出时通过 `GlyphAtlasSet::reserve_page_for_format(...)` 申请 page[1+] 并把 `SdfAtlasSlot.page_key` 指向真实页；evicted/rebuilt SDF page 现在通过 `SdfAtlasPlan.rebuilt_pages` 在 cache transition 中整页标脏；over-cap/oversized allocation failure 现在记录到 `SdfAtlasPlan.allocation_failures` 并汇总到 SDF prepare report,且 `SdfAtlasRun.glyph_failure_reasons` 按字符位置记录 page-limit/oversized 原因；fallback policy 已拆到 `ui/text/sdf_fallback.rs`,并能把连续同原因失败字形归并为 fallback spans/report span counts；Horizontal LTR/explicit RTL/no-wrap/non-justify 失败 span 已生成局部 native overlay,不支持的混合情形继续 whole-batch native fallback 且会记录 unsupported mixed overlay reason diagnostics。真实 alpha bitmap atlas 替换、glyphon atlas 迁移、持久化 glyph cache/residency 驱动的完整淘汰闭环、broader glyph-level mixed fallback(Vertical/Auto-Mixed/justify/wrapped)、independent oversized fallback、DPI/subpixel/hinting 仍未完成。
- `text/raster/policy.rs`:已承接 `raster_path_for`/`GlyphRasterPolicy` 的 bitmap/SDF/MSDF/Color 选路数据面,并开始按请求格式与 outline/shadow/glow 效果强制距离场路径。
- `text/raster/swash/`:已建立 swash 隔离层首段数据契约并按结构规范拆成 folder-backed owners:`bitmap.rs` 记录 `GlyphBitmap` size/bearing/px_size/data/channels/content 与 fallible validation,alpha/color/subpixel 位图可映射到 `GlyphAtlasFormat::{AlphaMask,Color,SubpixelMask}` 与 R8/RGBA storage；`atlas_source.rs` 将已验证 `GlyphBitmap` 投影为 `GlyphAtlasBitmapSource`,保留 atlas format、content size、screen rect、foreground/background color 与真实 `data.len()` source byte length,使 swash 输出可直接喂给 bitmap atlas run validation/allocation；`request.rs` 持有 `SwashRasterRequest`/`SwashRasterSource`/`SwashBitmapStrike` 与 swash source/render-format 选择；`rasterizer.rs` 持有真实 swash `ScaleContext`/`Scaler`/`Render` adapter,并把 swash `Image` 归一化为 `GlyphBitmap`；`color_strike.rs` 持有 COLR/CPAL 优先与 CBDT/sbix strike selection,选择 ≥目标尺寸最近 strike 下采样,否则最大较小 strike 作为显式 upscale fallback,并按比例换算 size/bearing/advance；`error.rs` 持有 `SwashRasterError`;`tests.rs` 保留 FiraSans 真实字体 alpha/subpixel outline、bitmap validation、atlas source bridge 与 emoji strike owner tests。Focused Cargo `text_raster_swash` 旧 11/11 证据仍适用旧单文件实现;最新 bridge 切片因外部 cargo/rustc lanes 活跃只声明 scoped rustfmt、diff check 与视觉证明。emoji RGBA fixture 实像素测试、生产 alpha bitmap atlas renderer 与 glyphon `TextAtlas` 切换仍未完成。
- `text/raster/swash/request.rs` + `rasterizer.rs`:2026-07-07 的 glyphon `CacheKey` parity 说明为**历史基线**。当前同步 Runtime service 与异步 native worker 都先生成同一后端无关 `TextGlyphRasterRequest`，再通过 `SwashRasterRequest::from_text_glyph_request(...)` 适配 glyph id、物理 ppem、3x4 x/y phase、hinting、smoothing、mode 与 synthetic italic；color source fallback 顺序仍为 `[ColorOutline(0), ColorBitmap(BestFit), AlphaOutline]`。renderer 不再持有或转换 glyphon cache key。
- `text/atlas/bitmap_run/staged_upload.rs`:2026-07-07 在 page-generation upload guard 之后补上 stale upload requeue report。`GlyphAtlasBitmapTextureUploadRequestPlan` 现在显式携带 `requeued_uploads`、`stale_page_generation_count` 与 `face_invalidated_count`;带 live atlas/face-validity 输入的 request plan 遇到 missing page、page generation mismatch 或 face invalidated 时不产出 texture upload request,而是记录 `GlyphAtlasBitmapRequeuedUpload`。这关闭了 stale artifact / face invalidated artifact 被静默跳过的首段数据面;真实 async worker、global glyph slot invalidation 与完整 glyphon `TextAtlas` cutover 仍未完成。
- `graphics/scene/scene_renderer/ui/atlas_texture_upload/frame.rs`:2026-07-07 继续把 low-level requeue report 接到 renderer-local texture upload frame。`GlyphAtlasBitmapTextureUploadFrameReport` 现在按帧汇总 missing-page、page-generation mismatch、face invalidated 与总 requeued upload 计数；`glyph_atlas_bitmap_texture_upload_frame_plan_for_atlas_and_face_validity(...)` 在任何 requeue 存在时保持 `ready_to_write_texture=false`,不向 WGPU writer 交出可写 plan。该切片只收束 frame report handoff,真实 async worker、global glyph slot invalidation、完整 glyphon `TextAtlas` cutover 与 live editor-window typography QA 仍未完成。
- `graphics/scene/scene_renderer/ui/atlas_renderer/renderer.rs`:2026-07-07 继续把 requeue frame report 推到 renderer prepare telemetry。生产 `prepare_submission(...)` / `prepare_storage_submissions(...)` 现在用 submission 自带的 live `GlyphAtlasSet` 调用 `glyph_atlas_bitmap_texture_upload_frame_plan_for_atlas(...)`,不再绕过 page-generation/missing-page guard；`GlyphAtlasBitmapRendererPrepareReport` 汇总 `upload_requeued_count`、`upload_missing_page_requeue_count`、`upload_page_generation_mismatch_requeue_count` 与 `upload_face_invalidated_count`,且 `upload_failure_count` 将 requeued uploads 计入失败口径。该切片关闭 renderer telemetry handoff 缺口,但 per-face artifact validity source、真实 async worker、global glyph slot invalidation、完整 glyphon `TextAtlas` cutover 与 live editor-window typography QA 仍未完成。
- `text/atlas/bitmap_run.rs` + `render_submission/retry.rs` + `text/native_bitmap_atlas.rs`:2026-07-07 补上主 native bitmap atlas 路径的持久 atlas state 与 slot invalidation 数据面首段。bitmap run/render submission/retry driver 现在可接收上一帧 `GlyphAtlasSet`,在帧开始清除 page reference,需要重建未引用页时记录 `GlyphAtlasBitmapSlotInvalidation { page_key, page_generation }` 并把整页标脏；`ScreenSpaceUiTextBackend` 在非空 native bitmap frame 间保留主 submission atlas,字体 face invalidation 与空 native text frame 则清空该 atlas。该切片让 page-generation guard 有真实跨帧 page state 可比较,避免 atlas page 重建后旧 slot 继续静默可写；同日 follow-up 又把 storage partition/submission 逻辑拆到 `text/native_bitmap_atlas/storage.rs`,并让 per-storage submission 通过 `glyph_atlas_bitmap_render_submission_plan_with_atlas(...)` 继承主 frame 的 `self.submission.run.atlas.clone()`,关闭 mixed R8/RGBA storage split 中 per-storage default-atlas reset。真实 async worker、完整 glyph slot owner、focused Cargo green、完整 glyphon `TextAtlas` cutover 与 live editor-window typography QA 仍未完成。
- `text/native_bitmap_atlas.rs` + `native_bitmap_atlas/handoff.rs`:2026-07-07 继续把 native bitmap atlas 的缺失 raster 图像从静默跳过改为可诊断 fail-closed。`source_cache.image(...)` 返回 `None` 时累计 `missing_raster_image_count`,prepare report 暴露该计数；handoff owner 新增 `MissingRasterImage` fallback reason,并确保只要缺图计数非 0,native atlas 不能替代 glyphon,即使 source image count 与 visible glyph count 看起来相等。该切片不完成真实 async raster worker 或首帧占位渲染,但关闭 atlas 输入不完整时仍接管 glyphon 的首帧降级风险。
- `text/native_bitmap_atlas/source_cache.rs` + `text/parallel/raster_pool.rs`:2026-07-07 的 work id → glyphon `CacheKey` pending 映射为**历史基线**。当前 pending/cache/worker completion 均以 `GlyphRasterKey` 为 key；accepted `GlyphBitmap` 继续转为 `SwashContent`、bearing、尺寸与共享 bytes，failed/unknown/invalid/face-invalidated/pending worker 计数仍进入 `NativeBitmapAtlasSourceCacheFrameReport`。2026-07-17 owner hard cut 删除 worker 层伪 `page_generation=0` target；raster bitmap 在 atlas page 分配之前产生，只能由 face epoch 失效，真实 page generation 继续由 allocation/staging/upload request guard 校验。idle frame 与 face invalidation 会同步清空 pending worker key。
- `scene_renderer/ui/text.rs` + `text/native_bitmap_atlas.rs` + `native_bitmap_atlas/source_cache.rs`:2026-07-07 的 miss scheduling 已在本轮收敛到 canonical glyph input。`ScreenSpaceUiTextBackend` 只向 native frame 传递 `NativeBitmapAtlasGlyphRun`; native frame 先按当前 face epoch drain completion，再让 miss 调用 `request_worker_image(...)`。source cache 用真实 face index/font bytes 将 `GlyphRasterKey` 投影为共享 `TextGlyphRasterRequest`，再交给唯一 Swash adapter，提交携 face epoch 的 `TextRasterWorkItem`，并用 `GlyphRasterKey` pending map 去重；source image 只从已完成 cache 读取，不在 native atlas miss 路径同步调用 glyphon `SwashCache`。
- `text/native_bitmap_atlas/source_cache.rs` + `native_bitmap_atlas.rs` + `native_bitmap_atlas/handoff.rs`:2026-07-07 关闭 PF-M3 “已有近似桶”首帧替代切片。source cache 只在 font/glyph/size/weight/flags 完全相同且仅 subpixel bin 不同时返回近似图像,并记录 `approximate_hit_count`;native frame 仍为 exact key 排队 worker request,但当前帧可用近似 source image 继续 native bitmap atlas submission,不走透明占位;prepare report 记录 `approximate_raster_image_count`,first-frame degradation 记录 `ApproximateBucketReplacement`。该切片不替代 per-page upload merge、persistent glyph slot owner 或 full glyphon `TextAtlas` cutover。
- `text/native_bitmap_atlas/source_cache.rs`:2026-07-08 的 glyphon `CacheKey.x_bin` 归一化为**历史基线**。当前相位完全由 `GlyphRasterKey` 的 `subpixel_bin` 与 `vertical_subpixel_bin` 表达；cache lookup、approximate lookup、worker request、pending check 与 insert 均使用这一文本所有 identity。纵向近似仍最多探测三个候选 bin，不进行全表扫描。
- `scene_renderer/ui/text.rs` + `text/native_bitmap_atlas/source_cache.rs`:2026-07-10 追加 native raster/upload prepare-report 聚合层。`ScreenSpaceUiTextPrepareReport.raster_upload` 从 native bitmap atlas prepare report 读取 visible/source/missing/approx glyph、source-cache hit/miss/worker-request 与 submission upload bytes,再合并 bitmap renderer upload/requeue/failure/ready 状态,为 AT-M3/PF-M4 的 scroll raster/upload 计数断言提供单一入口。该切片只接入可观测 surface,不声明 per-page upload merge、真实 scroll increment assertion、live editor-window typography QA 或完整 glyphon `TextAtlas` cutover 完成。
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

- `text/native_bitmap_atlas/source_cache.rs`:2026-07-05 新增原生 bitmap atlas source-image cache child owner。`NativeBitmapAtlasSourceCache` 按 glyphon/cosmic `CacheKey` 持久化 swash image content、bearing、尺寸与 source bytes,`ScreenSpaceUiTextBackend` 跨帧持有该 cache；`native_bitmap_atlas_frame(...)` 只有 miss 才调用 `SwashCache::get_image_uncached(...)`,并把 hit/miss/insert/evict/entry counters 写入 `NativeBitmapAtlasPrepareReport.source_cache`。同日 follow-up 已为字体资产集合变化增加 `discard_all_for_face_invalidation()` 与 `invalidated_count`,让 source-cache 旧 face source images 在下一帧前 fail-closed 清理并进入 report。这关闭了当前生产 native bitmap path 的 source 级重复栅格/重复 metadata 获取与 face-invalidation source-cache flush 首段,但真正 face validity requeue、全局 atlas slot invalidation、async worker 和 full glyphon `TextAtlas` cutover 仍未完成。

- `scene_renderer/ui/text.rs` + `text/native_bitmap_atlas.rs`:2026-07-05 继续修正生产 native bitmap atlas frame-loop 的帧戳输入。旧路径用固定 `BITMAP_ATLAS_FRAME_INDEX = 1` 调用 `glyph_atlas_bitmap_render_submission_plan(...)`,导致 blocked retry 与 page residency telemetry 始终只能表达 frame 1 -> retry 2。现在 `ScreenSpaceUiTextBackend` 持有递增 `bitmap_atlas_frame_index`,非空 native bitmap atlas frame、`NativeBitmapAtlasPrepareReport.frame_index` 与 per-storage submission 都使用同一个 live frame index。该切片只关闭生产提交帧号常量,仍不声明真实 retry-frame state execution、全局 atlas slot invalidation、async worker 或 glyphon `TextAtlas` cutover 完成。

- `text/native_bitmap_atlas/retry_frame.rs`:2026-07-05 将已有 `GlyphAtlasBitmapRetryFrameState` 接入生产 native bitmap atlas frame-loop。`ScreenSpaceUiTextBackend` 跨帧持有 retry state,字体面失效和空文本帧会清空 queue；native atlas 子 owner 只重试当前帧仍可见的 blocked source,把 source bytes 按 retry-aware submission input 顺序重映射,并丢弃已经不可见的 stale blocked source,避免旧 glyph 在后续帧被重新绘制。该切片关闭真实 renderer retry-frame state execution 的首段;全局 atlas slot invalidation、async worker、完整 glyphon `TextAtlas` cutover 与 live editor-window typography QA 仍 open。

- `text/native_bitmap_atlas/retry_frame.rs`:2026-07-06 stale retry selection telemetry follow-up 将旧 blocked source 的“本帧不可见而被丢弃”从隐式行为提升到 `NativeBitmapAtlasPrepareReport.discarded_stale_retry_glyph_count`。同一 queued blocked source 现在只会匹配一个当前 visible source,避免重复相同 source image 时把同一 retry 项消费两次；重复 visible source 中剩余项继续作为 new source 进入本帧提交。新增 `native_bitmap_atlas_retry_frame_does_not_reuse_one_blocked_source_twice`,并让 stale discard 回归断言 discarded count；`text/tests.rs` 的 aggregate prepare-report expectation 同步新字段默认值。验证日志 `docs/tests/runtime/text/runtime_text_native_bitmap_atlas_retry_stale_selection_validation_20260706.log` SHA256 `9961E678C812FBB79998B09EFBF0F430FB651EEEFABA46F43839BBD728254D01`；focused Cargo `native_bitmap_atlas_retry` 通过 5/5,日志 `docs/tests/runtime/text/runtime_text_native_bitmap_atlas_retry_focused_cargo_20260706.log` SHA256 `9ECCE49DCCAD9B5CE533EE1E2111A36D6F0F0F8A9AC066A226B0120F85368F3F`。该切片关闭 stale retry 可观测性和 duplicate retry consumption 风险,并补上 retry-frame focused Cargo 证据；不声明 async raster face-validity requeue、global slot invalidation、完整 glyphon `TextAtlas` cutover 或 live editor-window typography QA。

- `text/native_bitmap_atlas/retry_frame.rs` + `text/atlas/render_submission/frame_state.rs`:2026-07-05 retry face-invalidation report follow-up 让 `GlyphAtlasBitmapRetryFrameState` 不再把 face invalidation 清空 blocked retry queue 作为无声 drop。`discard_all_for_face_invalidation()` 累计 pending invalidated glyph count,visible frame 的 retry driver 和 `native_bitmap_atlas_idle_prepare_report(...)` 都用 `take_report()` 把该计数写入 `NativeBitmapAtlasPrepareReport.retry_state.invalidated_blocked_glyph_count` 后清零。新增 visible-frame 与 idle-frame 回归,验证图 `docs/tests/runtime/text/runtime_text_native_bitmap_atlas_retry_face_invalidation_preview_20260705.png` SHA256 `A02F00772E0908C0FCFB69F51DFE70BD353A23B7855A6D1583CF052E15EC505A`,验证日志 `docs/tests/runtime/text/runtime_text_native_bitmap_atlas_retry_face_invalidation_validation_20260705.log` SHA256 `C6C3B2CE2460E450C2163407A4995E8915E260A28F6D777A7C17544E1B2FD47D`。该切片只关闭 retry-state telemetry,不声明 async raster face-validity requeue、stale artifact requeue、full glyphon cutover 或 live editor-window typography QA。

- `graphics/scene/scene_renderer/ui/atlas_renderer/{renderer.rs,text.rs,tests.rs}`:2026-07-05 renderer face-invalidation follow-up 让生产 `GlyphAtlasBitmapRenderer` 在 font face/asset 变化时同步清空 active storage-pass draw/upload state,并在下一次 prepare report 中暴露 `invalidated_storage_pass_count`。`ScreenSpaceUiTextBackend` 在同一 face invalidation 分支同时清理 source cache、retry queue 与 renderer storage passes,避免旧 face 的 atlas draw command 在 source/raster cache 已失效后继续可见。验证图 `docs/tests/runtime/text/runtime_text_bitmap_renderer_face_invalidation_preview_20260705.png` SHA256 `5D35B7421413F7C8B1C47E4AAC5B25794D8BC2DFB27CC66730732091EF981CB1`,验证日志 `docs/tests/runtime/text/runtime_text_bitmap_renderer_face_invalidation_validation_20260705.log` SHA256 `15B149B884E73979E2B16CBE3B6A2D94735362B3F7E086E8FBD5CE12CD9E0FF1`。该切片关闭 renderer-local stale storage-pass telemetry,不声明 async raster face-validity requeue、global glyph slot eviction 或 live editor-window typography QA。

- `graphics/scene/scene_renderer/ui/atlas_renderer/resources.rs`:2026-07-05 针对最新 editor 截图中“等线已生效但字符左右间距/落点仍不舒服”的 atlas sampling 风险,bitmap atlas sampler 从 Linear min/mag 改为 Nearest min/mag/mipmap 并固定 LOD 0,与 glyphon bitmap cache 的 nearest sampling contract 对齐。`tests.rs` 新增 `glyph_atlas_bitmap_sampler_matches_glyphon_nearest_sampling_contract`,防止小字号 alpha/RGBA atlas texel 被线性过滤混入邻近字形左右边缘。验证图 `docs/tests/runtime/text/runtime_text_bitmap_atlas_nearest_sampler_preview_20260705.png` SHA256 `A8C071C64D89F6380CAC2D11B64970CD078051B9DD030AD3D1515395EF5C9A0B`,验证日志 `docs/tests/runtime/text/runtime_text_bitmap_atlas_nearest_sampler_validation_20260705.log` SHA256 `8734A1008CB635E072FFE89546368A579BAD4075DBEBABB6969031E747394DD0`。2026-07-06 focused Cargo `glyph_atlas_bitmap_sampler_matches_glyphon_nearest_sampling_contract` 通过 1/1,日志 `docs/tests/runtime/text/runtime_text_bitmap_atlas_nearest_sampler_focused_cargo_20260706.log` SHA256 `AD2A6E83F4D73F08C1A53404740E1148353D791F0F15AE9316B783FAE4BE5692`,exit `docs/tests/runtime/text/runtime_text_bitmap_atlas_nearest_sampler_focused_cargo_20260706.exit.txt` SHA256 `A9F58776A09B5DAC438049683F24BF85764E0FF8E7455952456165C68C158627`。该切片只关闭 runtime bitmap atlas GPU sampling phase/edge bleed 风险,不替代 retained-host crop live window QA、full glyphon `TextAtlas` cutover 或 broader LCD/gamma/background policy。

- `text/native_bitmap_atlas/handoff.rs`:2026-07-05 将 native bitmap atlas 的 glyphon/native handoff 判定从 `scene_renderer/ui/text.rs` 根实现移到 native bitmap atlas 子 owner。`NativeBitmapAtlasHandoff` 与 `native_bitmap_atlas_handoff_for_report(...)` 现在跟 `NativeBitmapAtlasPrepareReport` 同域维护,`text.rs` 只消费 single-storage replacement、mixed-storage replacement 与 glyphon fallback 决策,不再同时拥有 frame orchestration 和 cutover policy。原 handoff 回归同步迁入 `text/native_bitmap_atlas/tests.rs`,验证图 `docs/tests/runtime/text/runtime_text_native_bitmap_atlas_handoff_owner_preview_20260705.png` SHA256 `B97D06F24B38594DCECF485FEC38D27E825565D6AD9F48699476C22081901BDF`,验证日志 `docs/tests/runtime/text/runtime_text_native_bitmap_atlas_handoff_owner_validation_20260705.log` SHA256 `B64F35B3D2B3785602012ED91FF60550CA05FD0B85E0FC2A91D82B5B3AA223D9`。该切片是结构收束和 TextAtlas cutover 判定面整理,不声明行为变化、完整 glyphon `TextAtlas` cutover 或 live editor-window typography QA 完成。

- `text/native_bitmap_atlas/handoff.rs`:2026-07-05 fallback reason telemetry follow-up 在 `NativeBitmapAtlasPrepareReport` 上新增 `glyphon_fallback_reason`,由 native atlas handoff owner 按固定优先级写出 glyphon fallback 原因:`NoVisibleRasterGlyphs`、`UnsupportedGlyphFormat`、`IncompleteSourceCoverage`、`MissingBackgroundCompositeInput`、`AtlasAllocationFailure`、`MixedStorageSplitNotReady` 等。`text.rs` 的 handoff 分支不变,但 prepare report 不再只暴露 bool/count,可以区分 LCD 背景缺失、source 覆盖不完整、atlas 分配失败或 mixed storage split 未就绪。验证图 `docs/tests/runtime/text/runtime_text_native_bitmap_atlas_fallback_reason_preview_20260705.png` SHA256 `3B6A5965753EF9769E5CBCDAA1827F3EBF0A6A04C8D389260CF1B00CB65BB153`,验证日志 `docs/tests/runtime/text/runtime_text_native_bitmap_atlas_fallback_reason_validation_20260705.log` SHA256 `B541E92E405B52A5A8D66E79EB3BCB5E3159422EF3EDD88FFD45AA09D228C09C`。该切片关闭 fallback reason telemetry 首段,不声明完整 glyphon `TextAtlas` cutover、真实 framebuffer background acquisition 或 live editor-window typography QA 完成。

- `text/atlas/{page.rs,upload.rs,render_contract.rs,render_plan.rs,render_batch.rs}`:2026-07-03 已补 `GlyphAtlasSamplingSemantics`,让 `GlyphAtlasFormat::SubpixelMask` 与 `Color` 即使同用 RGBA8 atlas storage,也分别携带 `SubpixelCoverage` 与 `ColorRgba` 采样/混合语义；`GlyphAtlasPageSpec` 与 `GlyphAtlasUploadCommand` 均保留该语义,focused `render_text_atlas` 通过 14/14。随后 `render_contract.rs` 与 `atlas/shaders/glyph_atlas_sampling.wgsl` 接上 shader/blend contract owner:`SubpixelCoverage` 选择 `SubpixelRgbCoverage + SubpixelBackgroundComposite`,`ColorRgba` 选择 `ColorRgba + SourceRgba`,focused `render_text_atlas` 通过 18/18。最新 `render_contract.rs` 继续暴露 `GLYPH_ATLAS_TEXT_SHADER` 与 `GlyphAtlasShaderEntryPoints`,并通过 `atlas/shaders/glyph_atlas_pipeline.wgsl` 固定 `vs_main` 和 alpha/subpixel/SDF/MSDF/color fragment entry points。`render_plan.rs` 现在把 shared `GlyphRasterPlacement` 的 snapped x、clip 后屏幕矩形、按 glyph content size clamp 的 atlas UV、page layer、foreground/background color 与 `GlyphAtlasRenderContract` 绑定到同一 draw quad 数据面,focused draw-plan tests 通过 4/4；后续又把 `SubpixelBackgroundComposite` 的顶点背景输入规范为 finite/clamped RGB + opaque alpha,避免无效/半透明背景色放大 LCD 边缘偏色。`render_batch.rs` 继续把可见 draw quads 按 `(GlyphAtlasPageKey, GlyphAtlasRenderContract)` 分批,统计 visible/skipped glyph 与 vertex count,并阻止同为 RGBA8 storage 的 `SubpixelMask` 与 `Color` 混批。完整生产 bitmap atlas renderer、真实 GPU upload/draw、真实 framebuffer 背景获取/合成与 glyphon `TextAtlas` cutover 仍未接线。
- `text/atlas/bitmap_run.rs`:2026-07-04 新增 bitmap atlas run data-plane owner,把 AlphaMask/SubpixelMask/Color bitmap glyph source 分配到统一 `GlyphAtlasSet` pages,生成 dirty pages、draw glyphs 与 typed failures；随后 `upload_commands` 由同一 run owner 通过共享 `glyph_atlas_upload_command(...)` 生成,避免 renderer root 重复推导 partial/full upload。最新 upload-copy follow-up 让 `GlyphAtlasBitmapRunPlan.upload_copies` 持有 `GlyphAtlasBitmapUploadCopy`,按 source index 记录 page key、atlas rect、content size、source bytes/row、source byte len、atlas bytes/row 与 atlas byte offset,让未来 renderer staging buffer 可以按 run plan 拷贝 glyph bytes。follow-up 又在 `PageReservationBlocked` 时记录 `GlyphAtlasBitmapQueuedGlyph`,保留 source payload/source index 与最早 `retry_frame_index`,让后续占位渲染和下帧重试不再只依赖失败总数；placeholder follow-up 同步生成 `GlyphAtlasBitmapPlaceholderGlyph { mode: TransparentQuad }`,保留 blocked source 的 screen rect 与 retry frame,让本帧占位渲染有独立数据面。retry follow-up 新增 `bitmap_run/retry.rs`,用 `GlyphAtlasBitmapRetryPlan` 按 frame index 拆分 due/deferred blocked glyphs,并通过 `retry_sources()` 暴露可重新喂给下一次 atlas run 的源数据。结构 follow-up 将混合实现拆成 folder-backed owners:`bitmap_run/types.rs` 承接 source/glyph/run plan 声明,`failure.rs` 承接 typed failure 与 blocked retry queue,`placeholder.rs` 承接占位数据合同,`retry.rs` 承接下帧 retry queue 消费,`validation.rs` 承接源数据校验,`allocation.rs` 承接 page reservation/shelf allocation/dirty marking,`upload.rs` 承接 staging copy 与 dirty-page upload command projection,root `bitmap_run.rs` 降为 run orchestration + exports。`bitmap_run/tests.rs` 锁定格式分流、shelf overflow、失败原因、staging upload copy、dirty-page upload、full-page upload promotion、blocked retry queue、placeholder data-plane、retry queue consumer 和 draw-batch bridge。该切片仍是数据面,不替代生产 bitmap atlas renderer、真实 GPU upload/draw 或 glyphon `TextAtlas` cutover。
- `text/atlas/bitmap_run/staging.rs` 与 `bitmap_run/staged_upload.rs`:2026-07-04 继续关闭 bitmap atlas upload handoff。`staging.rs` 先把 `GlyphAtlasBitmapUploadCopy` + `GlyphAtlasBitmapUploadSourceBytes` 拷入 page-keyed full-page staging buffers,保留 source stride 与 atlas page stride；`staged_upload.rs` 再把这些 staging pages 与 `GlyphAtlasUploadCommand` 绑定为 `GlyphAtlasBitmapStagedUploadPlan`,显式给出 `staging_page_index`、command 和 staging page byte len,并 typed 报告 missing staging page / source range out-of-bounds。prepared follow-up 新增 `GlyphAtlasBitmapPreparedUploadPlan` 与 `glyph_atlas_bitmap_prepared_upload_plan(...)`,让 future renderer 以一个入口从 run plan + source bytes 得到 staging pages 与 staged uploads；若 source-byte staging 已失败,prepared plan 不再输出 staged uploads,避免把不完整 atlas page 交给 `Queue::write_texture`。最新 texture-request follow-up 新增 `GlyphAtlasBitmapTextureUploadRequestPlan`,把 staged uploads 投影为 WGPU-neutral texture write 字段:origin xy/layer、extent、source offset、bytes/row、rows/image 和 byte lengths。该 pair 是未来 renderer `Queue::write_texture` 消费的纯数据合同,避免在 renderer root、glyphon cutover 或 editor 消费路径重新计算 page/rect/stride/offset。
- `graphics/scene/scene_renderer/ui/atlas_texture_upload.rs`:2026-07-04 继续把 bitmap atlas upload handoff 留在 renderer-local leaf owner。`GlyphAtlasBitmapTextureUploadFramePlan` 现在从 prepared upload 出发,组合 texture upload request projection、staging-byte binding 与 `GlyphAtlasBitmapTextureUploadFrameReport`;只在 request/binding 存在且 staging、staged-upload、binding failure 全为 0 时标记 `ready_to_write_texture`。`write_glyph_atlas_bitmap_texture_upload_frame_plan(...)` 复用既有 binding writer,但 fail-closed 阻止不完整 staging page 进入 WGPU writer。该切片仍不生成 frame-loop glyph sources、不创建 bitmap texture-array 资源,也不替代 glyphon `TextAtlas` cutover。
- `text/atlas/{page.rs,page_residency.rs,upload.rs}` + bitmap upload handoff:2026-07-05 为 PF-M1 代际竞态补首段数据面。`GlyphAtlasPageSpec.generation` 标记页内容代际,`page_residency.rs` 在 LRU eviction/rebuild 复用同一 `GlyphAtlasPageKey` 时递增 generation；`GlyphAtlasUploadCommand`、`GlyphAtlasBitmapPageUploadStaging` 与 `GlyphAtlasBitmapTextureUploadRequest` 均携 `page_generation`。`glyph_atlas_bitmap_texture_upload_request_plan_with_atlas(...)` 按当前 `GlyphAtlasSet` 跳过 stale request,`atlas_texture_upload/binding.rs` 拒绝 staging/request generation mismatch,避免旧异步栅格产物写入已经易主的 texture-array layer。native source-cache face invalidation flush/report 已由 `text/native_bitmap_atlas/source_cache.rs` 首段关闭；face validity requeue、全局 atlas slot invalidation 和真实 async raster worker 仍在 09 后续。
- `graphics/scene/scene_renderer/ui/atlas_texture_upload/{write,binding,frame,resource}.rs`:2026-07-04 将 renderer-local upload handoff 从单文件混合 owner 拆成 folder-backed leaves。`write.rs` 只承接 `GlyphAtlasUploadCommand` 到 WGPU write fields 的字段投影和 `Queue::write_texture` 调用；`binding.rs` 只承接 bitmap texture upload request 与 staging page bytes 的 typed binding/failure；`frame.rs` 只承接 prepared upload frame readiness/report 与 fail-closed writer orchestration。最新 `resource.rs` 新增 `GlyphAtlasTextureArraySpec` 与 `create_glyph_atlas_texture_array_resources(...)`,把 R8/RGBA storage format 到 WGPU `TextureFormat`、D2Array view、extent clamp 与 COPY_DST/TEXTURE_BINDING usage 规则从 `sdf_render.rs` 私有 helper 抽出；SDF renderer 继续使用 R8Unorm 语义,但 texture-array 资源创建已走共享 owner。root `atlas_texture_upload.rs` 降为结构入口和窄导出,不再继续累积 request、binding、frame report、resource descriptor 与 WGPU write 细节。
- `graphics/scene/scene_renderer/ui/atlas_renderer/`:2026-07-04 新增 renderer-local bitmap atlas WGPU resource owner。`vertex.rs` 把 `GlyphAtlasGpuVertexBufferLayout` 映射为 wgpu `VertexBufferLayout`,并让 `GlyphAtlasGpuVertex` 通过 `Pod/Zeroable` 成为可安全上传的 vertex bytes；`resources.rs` 创建 texture-array/sampler bind-group layout、sampler、placeholder atlas texture/view/bind group；`pipeline.rs` 从 `GLYPH_ATLAS_TEXT_SHADER`、shader entry points、blend contract 与 primitive topology 创建 real wgpu pipeline；`renderer.rs` 缓存 pipeline resources、创建 vertex buffer、保存 draw commands 并在 render pass 中按 draw command 提交。`ScreenSpaceUiTextSystem` 已挂载 `GlyphAtlasBitmapRenderer`,先以 empty draw plan 进入 prepare/report/render no-op 路径；2026-07-05 起 alpha-mask native glyph sources/submission source bytes 已可进入 guarded upload/draw submission,且 replacement report 暴露是否实际替换 glyphon。该 owner 仍未完成 Color/SubpixelMask source、mixed storage texture-array cutover、persistent frame-loop glyph cache/eviction 或完整 glyphon `TextAtlas` 替换。
- `graphics/scene/scene_renderer/ui/atlas_renderer/{renderer.rs,resources.rs}`:2026-07-04 follow-up 修复 editor screenshot 编译链路发现的两个底层阻塞:bitmap atlas renderer 类型/方法提升到 UI 子系统可见范围,并在 atlas resources 内使用 `glyph_atlas_gpu_bind_group_layout()` 填充 texture/sampler binding,避免把 wgpu bind group layout 参数误当 glyph atlas layout。
- `text/atlas/render_submission.rs`:2026-07-04 新增 bitmap atlas render submission data-plane owner,把 bitmap run、upload commands、clipped draw batches 与 `GlyphAtlasGpuDrawPlan` 聚合成 `GlyphAtlasBitmapRenderSubmissionPlan`。随后补 `GlyphAtlasBitmapRenderSubmissionReport`,统一汇总 source/allocation/failure、dirty/rebuilt page、full/partial upload、upload byte、draw batch、pipeline、GPU batch、draw command、vertex 与 upload/GPU/failure readiness flags,让 renderer/glyphon cutover 前可先检查提交质量。follow-up 又把 failure diagnostics 从单一总数拆成 UnsupportedFormat/EmptyContent/DataLengthMismatch/PageReservationBlocked/OversizedGlyph 五类 counter,并提供 source-validation 与 atlas-capacity 两个汇总 helper,同时暴露 `blocked_retry_count` 与 `next_retry_frame_index`,让后续 renderer/telemetry 能区分源数据错误、atlas 容量压力和下帧重试压力。最新 placeholder draw-plan follow-up 新增 `render_submission/placeholder.rs`,将 blocked placeholder glyphs 按同一 `GlyphAtlasScreenRect::clipped_to(...)` 生成 renderer-facing visible/skipped placeholder draw plan,并把 `visible_placeholder_count`/`skipped_placeholder_count` 汇入 report。retry backpressure follow-up 又让 `bitmap_run/retry.rs` 通过 `GlyphAtlasBitmapRetryBackpressurePolicy` 限制每帧 due retry source 数,并让 `render_submission/retry.rs` 报告 `backpressured_retry_count`,避免未来 renderer/frame-loop 在 root 层重复节流 blocked queue。`render_submission/tests.rs` 锁定 visible glyph → GPU draw data、clipped glyph → upload-only、full-page upload 计数、invalid bitmap source → diagnostics-only、same-frame page eviction blocked、blocked retry report、mixed failure breakdown、clipped placeholder draw-plan 与 backpressured retry report 等 renderer handoff 语义。该 owner 仍不创建 wgpu texture/pipeline/bind group,不替代真实 GPU upload/draw 或 glyphon `TextAtlas` cutover。
- `text/atlas/render_submission/frame_state.rs`:2026-07-04 新增跨帧 retry state owner。`GlyphAtlasBitmapRetryFrameState` 持有 renderer root 之下的 blocked glyph queue,用 queued glyphs + 本帧 new sources 生成 retry-aware submission plan,再通过 `apply_submission_plan(...)` 把 `GlyphAtlasBitmapRetryFrameSubmissionPlan.frame_outcome.next_blocked_glyphs` 写回下一帧状态。该 owner 只承担 frame-loop handoff 数据面,避免未来 root 层复制 deferred/backpressured commit、source-index remap 或 earliest retry frame 统计；真实 renderer frame-loop execution、wgpu upload/draw 与 glyphon `TextAtlas` cutover 仍未接线。
- `text/atlas/render_submission/frame_driver.rs`:2026-07-04 新增 retry frame driver owner。`GlyphAtlasBitmapRetryFrameDriverConfig` 固定一帧的 atlas page/viewport/clip/backpressure 参数,`glyph_atlas_bitmap_retry_frame_driver_submit_with_config(...)` 让 future renderer/frame-loop 用一个入口完成 state → retry submission plan → outcome commit,并返回 `GlyphAtlasBitmapRetryFrameDriverOutput` 给 telemetry/render handoff。该 owner 仍是 renderer handoff 数据面,不创建 wgpu texture/pipeline/bind group,不替代真实 GPU upload/draw、生产 bitmap atlas renderer 或 glyphon `TextAtlas` cutover。
- `text/atlas/render_gpu_plan.rs`:2026-07-03 已新增 bitmap atlas GPU vertex layout contract、viewport transform contract、draw-command contract、pipeline/bind-group contract 与 shader-entry contract 数据面。`GlyphAtlasGpuVertexBufferLayout` 固定 stride=52 bytes,并记录 shader location 0..4 对应 `position_ndc@0`、`uv@8`、`foreground_color@16`、`background_color@32`、`page_index@48`;`GlyphAtlasGpuViewportTransform` 记录 viewport size 与 `PixelEdges` 坐标约定,并显式把像素边界坐标映射到 NDC:左上 `[0,0] -> [-1,1]`,中心 `[w/2,h/2] -> [0,0]`,右下 `[w,h] -> [1,-1]`,空 viewport 以 1px extent 兜底。`GlyphAtlasGpuDrawPlan` 现在随 vertices/batches 一起携带 layout、viewport transform、`GlyphAtlasGpuDrawCommand` 列表、固定 texture-array/sampler bind-group layout 与 unique `GlyphAtlasGpuPipelineContract` 列表；每条 draw command 显式携带 render contract、`TriangleList` topology、vertex range、quad/triangle count 语义、atlas layer 与 pipeline key。`GlyphAtlasGpuPipelineContract` 同步携带 `shader_entry_points`,后续 renderer/shader 接线不再需要从 Rust struct 字段顺序、私有 helper、batch 顺序、shader include 或临时常量重新推导属性布局、半像素约定、draw range/layer、texture binding、fragment entry point 或 pipeline state。本轮按结构规范把该 mixed GPU contract umbrella 拆为 folder-backed child owners:`render_gpu_plan/vertex.rs`、`bind_group.rs`、`viewport.rs`、`draw_command.rs` 与 `pipeline.rs`,随后继续把既有 GPU draw-plan tests 搬到 `render_gpu_plan/tests.rs`,root `render_gpu_plan.rs` 降为 draw-plan assembly + module exports。该切片回应编辑器 tab 截图中小字号文字局部左右落点不稳的 GPU 输入侧风险,但不替代真实 glyphon atlas cutover 或窗口级 editor typography QA。

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
1. `text/atlas/`:shelf 分配器、页(1024×1024,R8/RGBA8)、脏矩形上传(graph 资源节点声明 IO)、页级 LRU。
2. `text/raster/swash/`:swash 栅格隔离层(alpha + 彩色 emoji);bearing/px_size 提取。
3. UI 文本绘制改消费 `GlyphAtlasRef`(从 glyphon 自管 atlas 切到统一 atlas);glyphon 退为"按 atlas 坐标画 quad"或整体由 `render/14` 的 sprite 批接管。

测试:`render_text_atlas_shelf_allocates_same_height_into_one_row`、`render_text_atlas_evicts_lru_page`、`text_raster_swash_emoji_rgba_glyph`。

### AT-M2 DPI 重栅格 + subpixel + hinting

实施切片:
1. atlas key 含 `px_size_bucket`(`logical_px × scale_factor` 量化)与 `subpixel_bin`(水平 1/3 或整像素吸附);scale 变换触发重栅格(接 `editor_layout/17 §3.2`(2026-07-02 评审收口:原引 §3.4 为指错节勘误))。
2. hinting 策略:`HintingMode::{None,Vertical,Full}`(默认 Vertical,对小字号清晰);font_smoothing 开关。(2026-07-02 评审收口)`font_smoothing`(灰度 AA vs subpixel AA vs 无平滑)必须进栅格键:要么在 `GlyphRasterKey` 上独立加 `font_smoothing` 字段(bevy `FontAtlasKey` 同款),要么并入 `HintingMode` 枚举维度并在文档注明;SDF/MSDF 路径 hinting 恒为 `None`(距离场栅格不做 grid-fitting,键侧固定,避免同 glyph 因 hinting 维度产生无意义的 SDF 重复烘焙)。
3. 整像素吸附:文本/1px 边框整像素吸附(`render/14`/`editor_layout/21 §3.5`),自由内容不吸附。

测试:`text_atlas_key_rebuckets_on_scale_change`、`text_raster_subpixel_bins_distinct`、`render_text_dpi_rerasterizes_with_distinct_native_cache_entries`。

### AT-M3 脏矩形增量上传定稿

实施切片:
1. 启用脏矩形/脏槽增量上传(现有 `sdf_upload.rs` DirtySlots 设计落地);每页本帧新增 glyph 合并为最小覆盖矩形,单次 `write_texture`。
2. 过大字形(超页)降级:大字号走 SDF(`05`)或独立纹理。

测试:`render_text_atlas_partial_upload_merges_dirty_rects`、`text_atlas_oversized_glyph_falls_back_to_sdf`。

## 6. 工程落地细化(实施权威)

### 模块与文件落点

实现层 `zircon_runtime/src/text/`:

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
| `render_text_dpi_rerasterizes_with_distinct_native_cache_entries` | 同一 WGPU renderer 的 1x→2x 切换产生 exact source-cache miss，且同 device-space frame 的字形 bounds/coverage 增长（抓帧） |
| `text_atlas_oversized_glyph_falls_back_to_sdf` | 超页字形降级 SDF,不 panic |
| `text_raster_emoji_strike_selection` | (2026-07-02 评审收口)CBDT/sbix 选 ≥目标尺寸最近 strike 下采样,bearing 按比例换算;COLR 存在时优先矢量 |
| `render_text_atlas_blocked_queues_glyph_to_next_frame` | (2026-07-02 评审收口)全页本帧引用时新增 glyph 排队下帧+占位渲染,blocked 计数进 `render_perf_text_*`,不 panic |
| `render_perf_text_atlas_reuses_persistent_slot_without_upload` | 相同 neutral raster key 跨帧复用同一 rect,只更新 screen rect,稳定帧不产生 dirty/upload |
| `render_perf_text_atlas_persistent_slot_allocates_only_new_glyph` | 已缓存 glyph 命中,仅新 glyph 分配 slot 并上传 |
| `render_text_atlas_same_frame_slot_projection_preserves_upload` | mixed-storage 同帧子提交复用 rect 但仍保留 dirty/upload；仅后续帧命中跳过上传 |
| `render_perf_text_atlas_submission_reports_persistent_slot_counters` | submission report 准确投影 slot hit/miss/insert 与稳定帧零 upload |
| `render_text_atlas_persistent_slot_eviction_invalidates_page_identity` | 页逐出同步清 allocator/slot,key 再出现必须 miss,且 page generation 单调推进 |
| `render_text_atlas_persistent_slot_rebuilds_when_page_size_changes` | page size 改变不得复用旧 rect,必须重建并上传 |

里程碑命令:`cargo test -p zircon_runtime render_text_atlas --locked`、`text_raster --locked`。

## 7. 风险与回退

- glyphon 深度耦合:若一步切走 glyphon 风险大,AT-M1 可先让 glyphon 与 `GlyphAtlasSet` 并存(glyphon 仅 latin 快路径),AT-M3 后全切;但不留双布局路径(布局恒走 02/03)。
- subpixel 与 wgpu 混合:subpixel AA 需特殊 blend,V1 用整像素 + 灰度 AA(覆盖多数场景),subpixel 为 feature。

## 8. 状态与产出记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

当前概述（2026-07-11）：真实 runtime 产品帧缓冲继续通过 native glyphon/cosmic bitmap/color atlas 绘制中文、RTL 与彩色 Emoji，并直接锁定 `Segoe UI Emoji`、`SwashContent::Color` 与 RGBA 字节合同。本轮又让 SDF atlas key 保留 shaping backend 的实际 glyph id + face id，`sdf_atlas/text_keys.rs` 独立拥有 key 收集，避免 renderer root 或 `sdf_atlas.rs` 堆入第二套字形身份推导；atlas owner 精确测试 23/23、font bake 10/10。native bitmap atlas 的 mixed-storage handoff 现在按原始绘制顺序拆分连续 storage runs，`R8 -> RGBA -> R8` 可生成三个有序 renderer passes，不再因同一格式重复出现而退回 glyphon；当前源 native atlas 44/44、atlas renderer 13/13。动态 framebuffer 背景获取、完整 glyphon atlas 硬切、DPI 重栅格、persistent native glyph-slot 全闭环与 live editor-window typography QA 仍未关闭。

2026-07-17 MVP 边界更新：异步 swash raster source 已硬切为 face-epoch owner，删除固定 `page_generation=0` 与 worker stale-page 分支；真实 page generation 继续由 bitmap allocation、staging、texture-upload request 与 renderer binding 校验。这样 atlas page LRU churn 不再无效丢弃可复用 CPU bitmap，同时旧页写入仍 fail-closed。`render_perf_text_async_upload_merges_per_page` 又锁住同页多 glyph 只生成一个 dirty rect、upload command、staging page 与 texture request。实现与 focused test code 已完成，scoped rustfmt 与旧符号扫描通过；focused Cargo 与产品帧验证仍为 `validation_pending`，完整 glyphon atlas 硬切和 live editor-window typography QA 继续 open。

2026-07-18 persistent slot MVP 更新：`GlyphAtlasSet` 已由“只持久 page residency”推进到同时持有 neutral `GlyphRasterKey -> page/generation/inserted-frame/rect/content-size` 槽位与每页 shelf allocator。精确 native bitmap source 在 face/instance、glyph、物理 px 桶、水平/垂直 subpixel phase、format、hinting/smoothing 与 synthetic oblique 完全一致时可跨帧命中，后续帧命中只重建 draw placement，不再重复 dirty/upload；同帧 mixed-storage 子提交复用 rect 但仍保留首次 GPU upload，新 key 只增量分配和上传。页逐出会原子清理该页 allocator 与所有 slot，page size 改变会拒绝旧 rect；裁剪 glyph、近似桶、pixel-font、无稳定 face identity 或 mixed legacy source 保持旧帧内策略，避免错误缓存。slot hit/miss/insert 已进入 submission report。实现、focused test code、scoped rustfmt 与 diff check 已完成，状态为 `implemented / validation_pending`；受管 focused/default/graphics-only、真实 WGPU 产品帧截图、完整 glyphon `TextAtlas` 硬切与 live editor typography QA 仍 open。

2026-07-19 AT-M2 native DPI key 传播更新：生产 native path 的 `LayoutGlyph::physical((left, top), text_area.scale)` 已在 cosmic owner 内把 logical font size 乘 surface scale，并将物理字号写入 `CacheKey.font_size_bits`。`native_bitmap_atlas_raster_key_from_physical_cache(...)` 现在显式拥有这条已物理化输入到 neutral `GlyphRasterKey` 的投影，并固定 generic request `scale_factor=1.0`，避免把同一 DPI scale 乘两次；`native_bitmap_atlas_physical_cache_key_rebuckets_at_2x_scale` 锁定 12px/24px physical cache key 分别产生 12/24 bucket 与不同 atlas identity。该切片关闭 native persistent-slot key 的 DPI propagation contract，不宣称真实 2x framebuffer 清晰度已经验收；scoped rustfmt/diff-check 与 fresh managed focused Cargo 仍待本里程碑 testing stage，`render_text_dpi_rerasterizes_with_distinct_native_cache_entries` WGPU 产品门继续 open。

2026-07-30 Text04 native raster settle 第一段：native source-cache 已将每个 backend face 在当前 face epoch 内映射为稳定的 Swash identity，并把 `[face_epoch, face_id]` 经 `SwashRasterRequest` 传给 per-worker `ScaleContext::builder_with_id(...)`。这消除了 fresh identity 造成的 proxy/hint cache miss，同时 face invalidation 会同步清空 identity 与共享 font bytes。worker request 还直接持有 `FontDatabase` 已缓存的 `Arc<VariationCoords>`，避免同 face/weight 的每 glyph variations Vec clone；回归覆盖同 face request identity 复用、face invalidation 后 identity 变化及 shared variations handle。

2026-07-30 Text04 native raster settle 第二段：真实 WGPU 产品 framebuffer job `a0053c3a4254472cab826db7e32b3216` 已重编译第一段源码，但 120 帧后仍有 115 个 native-atlas raster placeholder pending（failed=0），因此没有 capture 帧或新 PNG。r5 将剩余根因收敛为同 face/size/instance 的逐 glyph scaler build：worker 在既有有界队列内至多收集 32 项，按 shared font bytes、face identity、物理字号、hint 和变体分组；同组只 parse 一次 `FontRef`、build 一次 scaler，并逐项立即发布到既有 cancellation、completion-byte-budget 与 diagnostics 边界。`text_raster_swash_rasterizer_batches_compatible_requests_into_one_scaler` 使用真实 FiraSans 字形锁定这一合同，worker 回归锁定批内取消不阻断其余兼容 glyph。为遵守代码结构规范，执行状态机已拆为 `text/parallel/raster_pool/worker.rs`（251 行），`raster_pool.rs` 回落至 806 行。scoped rustfmt/diff check 已通过，定向 Cargo 预约 `cf4d662d664d4845988bf418890a6803` 等待 FIFO；状态为 `implemented / resolving_failure / managed_validation_pending`，不声明截图、产出记录或提交完成。

本子计划产出记录已超过 10 条，具体记录已迁入编号子目录。

- 迁入记录：[`04/2026-07-09-glyph-atlas-and-rasterization-output-records.md`](04/2026-07-09-glyph-atlas-and-rasterization-output-records.md)

2026-07-31 当前源编译门：受管 job `4d6d648211034a6492c2bd0b0443a757` 在运行 Text04 新增的 Swash scaler 回归前以 exit `101` 停在 `zircon_runtime` lib-test 编译。诊断来自授权范围外的 `scene/level_system.rs:181`（不稳定 `cfg` 表达式属性）和 `core/runtime/tests/tasks.rs:1`（私有 `environment` 模块导入）。因此 Text04 维持 `implemented / resolving_failure / managed_validation_pending`：两个定向回归、真实 WGPU framebuffer 和新 PNG 都尚未通过或产生，不声明截图、产出记录或提交完成。

2026-07-31 审查收敛：`SwashRasterizer::rasterize(...)` 不再以生产 `expect` 假定批处理回调必定写入结果；异常路径现在返回现有 `MissingGlyphImage` 错误。新增 `text_raster_swash_rasterizer_separates_different_physical_sizes`，锁定 18px 与 19px 请求各自 build scaler，防止 DPI 栅格结果被错误合批。该回归及既有兼容批次回归仍等待默认 feature 的受管编译门恢复后执行。

2026-07-31 当前生产编译证据：受管 check job `eeee3c03fa29475ca1191c3b3031b5aa` / run `16170cacbdc44455bfd6856788ae8fca` 执行 `cargo +1.94.1 check -p zircon_runtime --lib --features animation --locked --jobs 1 --color never`，以 exit `0` 完成（27m12s）。它直接编译本切片的 Swash rasterizer、raster worker、native atlas source-cache 路径；546 个既有 library warning 未升级为 error。该 partial feature check 只证明当前生产代码可编译，不替代默认 feature 的两个定向回归、真实 WGPU framebuffer settle 门或新 PNG 证据。

2026-07-31 Text04 retained bitmap-page replay 实现完成 / 受管验证待执行：为修复 persistent slot 的局部 upload 合并在 staging 中零填充仍在用的 atlas texel（P1）这一 failure，`GlyphAtlasSet` 现持有受 32 MiB 上限约束、按 page generation 校验的 CPU page-shadow；新 glyph 的 compact staging 先从该 shadow 回放，再写入本帧栅格字节。dirty page 在可回放时被严格压到每页至多 8 次 GPU write，逐出、重建、generation 不符或提交未 ready 时会清除受影响页的 slot/shadow，让下一帧前向重传，绝不保留可能对应半提交 GPU 内容的缓存。renderer 的真实 `Queue::write_texture` handoff 已仅在 frame ready 后提交 shadow patch；mixed storage pass 聚合相同的提交令牌，避免稳态逐帧深拷贝 atlas/shadow。实现遵守结构规范：shadow data-plane 拆为 `page_shadow/{commit,patch,shadow,store}.rs`，根 `page.rs` 保持 482 行，未引入生产 panic、compatibility shim 或无界 per-frame clone。新增 ignored 的 `render_text_native_bitmap_atlas_product_framebuffer` 真实 WGPU product test：它从 native alpha bitmap atlas upload、draw submission 到 offscreen target/readback，验证四个规划屏幕位置均有 GPU 像素，并且只在成功时导出 `docs/tests/runtime/text/runtime_text_native_bitmap_atlas_product_framebuffer_20260731.png`，同时拒绝同名文件落在任意 target。

2026-07-31 PERF-MVP-244 前向修复实现完成 / 受管验证待执行：bitmap atlas draw 现只保留一次 clipped occurrence，并将每 glyph 打包为 68 B `GlyphAtlasGpuInstance`（screen rect、UV rect、前景/背景色、atlas layer）；`vertex_index` 在 `glyph_atlas_pipeline.wgsl` 展开固定六个 triangle-list corner，viewport pixel→NDC 已硬切到 16 B vertex-stage uniform，CPU 不再为每 glyph 物化或投影六个 52 B 顶点。renderer 的 WGPU instance buffer 使用 `VertexStepMode::Instance` 和 `draw(0..6, instance_range)`；batch 只合并相邻的相同 page/contract，保留 Alpha/Color/Subpixel painter order。稳定/缩小 instance 帧复用同一 `VERTEX | COPY_DST` buffer，只通过 `Queue::write_buffer` 写入有效字节；仅首次或容量不足时才按至少 4 KiB 的二次幂重新分配，prepare report 记录实际 capacity/reallocation count；显式 idle 则释放所有历史 pass buffer，仅保留一个空 pass，避免 mixed-storage 峰值常驻。旧 `render_gpu_plan/vertex.rs` 与 `atlas_renderer/vertex.rs` 均删除，分别由 `instance.rs` leaf owner 接替；为遵守结构规范，renderer state/prepare DTO 已拆到 `state.rs`，instance-buffer 生命周期拆到 `instance_buffer.rs`，当前 renderer root 为 725 行、state leaf 为 137 行、instance-buffer leaf 为 68 行。atlas resource 异常缺失时不再以生产 `expect` 中断，而是将 upload 降级为 binding failure、禁用 shadow commit，使下一帧前向重传。新/更新的 leaf tests 固定 instance stride、instance range、uniform payload、角点几何、非相邻格式次序、容量增长、idle resource release 与该 forward-retry 合同；新增整页阈值提升回归，将已 shadow-commit 的 8x8 持久槽位与同页 56x64 新槽位组合，要求 full-page staging 回放旧槽位四角像素。screenshot-guard P2 在目录创建前将 output/target 解析为“最深存在祖先”的 canonical identity 并重附缺失尾部，避免 Windows `\\?\` 与普通 DOS 前缀混用；不可解析时 Windows 按组件忽略 ASCII 大小写比较，覆盖 `..` 和 `DOCS` target 别名，独立静态复审已通过。scoped rustfmt 与无索引 diff check 已通过。受管 Cargo/WGPU product framebuffer 尚未收到成功回执，因此全部关联 failure 继续为 open，未生成截图、未声明验证或提交完成。状态维持 `implemented / resolving_failure / managed_validation_pending`。

2026-08-01 PERF-MVP-244 规模证据补齐：新增 1/100/1k/10k glyph 精确门禁，固定 `N` occurrences、`N` 68 B instances、`0` CPU quad vertices、同合同连续场景 `1` draw、`68N` upload bytes，并验证 resizable WGPU instance buffer 在同容量稳态不重新分配；ignored 的 31-sample exporter输出 p50/p95，不把机器时延设为验收阈值。二次静态审查修正 p95 nearest-rank 计算和误导性的三 batch helper 名称，未发现 actionable P0/P1/P2。failure 保持 `open / implementation_complete / resolving_failure / managed_validation_pending`，等待 managed focused/upward/ignored WGPU；未产生新 PNG。

2026-08-01 PERF-MVP-244 current-source 结构计数：`render_gpu_plan.rs` 134 行、plan `instance.rs` 100、`draw_command.rs` 66、atlas renderer root 769、renderer `state.rs` 148、WGPU `instance.rs` 63、`instance_buffer.rs` 75；全部 production owner 低于 800 行 warning。7 月 31 日段落保留的是当时计数，本行是当前权威计数。

2026-07-31 Text04 mixed-storage page-shadow P1 前向修复：连续 `AlphaMask → Color → AlphaMask` split 在同一帧各自持有 atlas clone，因而 pending `zero_initialize_shadow_pages` 不能被后一个 Alpha split 当作已可回放 shadow；否则其 75% dirty threshold full-page upload 会清除前一 split 的同页像素。storage owner 已只以 generation-matched committed shadow 授权 replay；缺少 committed shadow 时，包含其他 split slot 的页维持 partial upload。新回归以 8x8 Alpha、Color、56x64 Alpha 构造共享 Alpha page，并锁定后一个 Alpha command/staging 为 compact partial rect。该 P1 的独立静态复审与受管 Cargo/WGPU 仍待执行；failure 保持 open，未生成 PNG 或声明验证完成。

2026-07-31 Text04 mixed-storage shadow replay follow-up：上一帧 committed shadow 也不能回放同一帧 sibling split 新写入的 slot。storage split 现在若发现同页有当前 range 之外的 upload copy，禁止该 split 将 dirty threshold 升级到 full page；它仍使用 bounded region merge/partial staging，避免退化成每 glyph write。跨帧回归先提交 cached Alpha shadow，再用 `cached Alpha → new Alpha → Color → later Alpha(64x48)` 固定 shared page 与 75% threshold，断言后一个 Alpha 只能 partial upload。独立静态复审已确认 fresh 与 committed-shadow 两条 split 覆盖路径均无 P0/P1/P2；受管 Cargo/WGPU 仍待执行，failure 保持 open，未生成 PNG 或声明验证完成。

2026-07-31 PERF-MVP-242 source-cache 非验收收敛：当前 native bitmap source cache 的空文本帧只刷新 report，不再清除 resident image 或 pending worker；CPU residency 同时受 2048 entry 与 8 MiB hard cap 约束，cached pixels 使用共享 `Arc<[u8]>`，approximate lookup 直接构造最多 3 个 vertical-bin key。LRU 的 head/tail 与双向链接已从 601 行 cache owner 拆到 239 行 `source_cache/lru.rs` leaf，正常 hit/insert/evict 只做常数次 HashMap 操作；链接不一致时不再通过生产 `expect` panic，而是执行一次异常 O(n) 索引重建并增加 `lru_repair_count`。回归人为注入 dangling tail，锁定修复后最近使用顺序、逐出结果和 telemetry；二次静态审查未发现 P0/P1/P2，scoped Cargo 尚未执行。跨 source/slot/GPU page 的统一 budget-pressure eviction、300 empty-frame 规模证据及 WGPU/RenderDoc 像素验收仍 open，状态为 `implemented / resolving_failure / managed_validation_pending`。

2026-08-01 PERF-MVP-242 统一 residency 前向收敛：source cache 以 O(1) reverse index 将 exact `CacheKey` 绑定到 neutral `GlyphRasterKey`；source entry/byte pressure 发出的 key 由 `GlyphAtlasSet` page owner 一次提升 generation 并清 page allocator/slots/shadow，同页 raster keys 再定点反向失效 source。atlas LRU eviction 通过显式 run-plan key 列表回传，renderer failed upload 对 page keys 去重后只推进一次 generation，并在下一帧报告失效。产品 telemetry 现包含 CPU resident/max bytes、LRU touch、linked eviction/invalidation 与 logical GPU page bytes。300 empty frames、2048 resident + 1/100/1k misses 和 ignored 31-sample p50/p95 证据代码均已写入；test owner 拆分后 parent 641 行、residency child 212，production source/LRU/page/render-state 为 731/243/527/396。二次静态审查同时修复 instance hard-cut 后两个 stale `.vertices` 消费者、hidden invalidation handoff、duplicate generation bump、stale exact report 与失真 static guard，未留下 actionable P0/P1/P2。状态为 `implementation_complete / resolving_failure / managed_validation_pending`；current-source managed Cargo/WGPU/RenderDoc 仍未验收，不产生新 PNG。

Coordinator handoff（2026-08-01）：Text04 完整 Plan/scope registration 在 coordinator health 阶段超时且没有 receipt；按规则不重试、不查询数据库、不等待 maintenance/validation queue。coordinator wakeup 后提交 source residency focused/scale/ignored p50-p95、instance draw focused/upward 与 exact ignored WGPU product framebuffer；当前没有 queued/running ticket。

2026-08-03 Text04 Native contract capture forward fix: the generic runtime UI text contract
helper no longer captures its first submitted frame. Text-bearing extracts retain one WGPU
backend and viewport across at most 120 uniquely identified submissions, then require two
consecutive frames with zero raster pending/failure/missing-image/placeholder/requeued-upload/
upload-failure telemetry before framebuffer capture. The new pure settle-contract regression
locks every nonzero telemetry input and the consecutive-frame requirement. This only repairs
test-driving infrastructure; it does not assert a WGPU pass, create a PNG, or change this
failure from `open / implementation_complete / resolving_failure / managed_validation_pending`.

2026-07-31 PERF-MVP-245 retry 产品预算 current-source 复核：native bitmap 产品入口不再使用 unlimited policy；Text09 权威的 256 glyph / 2 MiB 确定性帧预算被 old retry 与 new visible work 各分 128 glyph / 1 MiB，retained blocked queue 另有 256 entry / 2 MiB hard cap，超限以独立 overflow/rejected counter fail-closed 到 Glyphon。retry/new 只对 persistent-slot miss 计预算，同 `GlyphRasterKey`/generation 在帧内去重；队列保留旧项顺序并轮转 backpressured 项，300 帧回归固定三项各尝试 100 次。Text09 对 CPU time 的契约仍是只观测不 gate，因此本阶段不新增无标定毫秒/像素常数；current-source Cargo、规模 telemetry 与产品像素尚未执行，failure 保持 open。

2026-07-31 Text04 raster completion API current-source 复核：native atlas/source cache/worker pool 已硬切到唯一 `face_epoch` identity 与 `drain_completed_for_face_epoch(..., TextRasterCompletionDrainBudget)`；旧 `TextRasterWorkTarget`、`drain_completed_for_target` 及 completion-side page-generation 字段在 Text owner 扫描为 0。真实 page generation 仍只由 allocation/staging/upload owner 校验，避免 CPU raster completion 因 page churn 被错误丢弃。7 月 31 日 partial-feature `zircon_runtime --lib` 生产 check 已覆盖这些 owner，但 focused lib-test、Editor09 upward gate、独立复审与受管 commit SHA 仍待完成，接口 failure 不转 fixed。

2026-07-31 Auto raster route current-source 复核：screen-space UI 的 Auto batch 已由 `resolved_auto_text_render_mode(...)` 统一进入 `GlyphRasterPolicyRequest`；无显式 font default 时，小字号 alpha 走 Native，大字号/outline/shadow 走 SDF，true-distance glow 走 MTSDF，font asset 明确指定 Native/Sdf/Msdf/Mtsdf 时保持作者语义。产品与 policy tests 已覆盖这些分支，旧“policy 仅测试消费”描述不再成立。当前 `ScreenSpaceUiTextBatch` 尚无稳定 command/layout identity，因此不能以数组下标或 text hash 错建跨帧 hysteresis；稳定 identity、physical scale 输入、residency-aware hysteresis 与产品 WGPU 仍 open。

2026-08-01 PERF-MVP-241 Auto route identity/hysteresis 实现完成：`ScreenSpaceUiTextBatch` 现在携 `UiTreeId(Arc<str>) + UiNodeId + source_range` 的稳定 fragment identity 与既有 command/layout generation；普通 line、rich run、inline icon 共用一次 command-generation 读取。`AutoTextRasterRouter` 按 generation 只评估一次 policy，并以 warm route 作为 residency hint；`GlyphRasterPolicy` 单源执行 24px threshold 与 22/26px 双向 hysteresis，显式 font mode 和 effects 不被改写。router 以 2048 entry/300 idle frames/tokenized recency 有界维护，产品 report 暴露 hit/evaluation/retain/switch/eviction，1/100/1k 与 ignored 31-sample p50/p95 证据代码已写入。二次审查修复 tree identity、Arc move、visibility、projection duplication 与 eager allocation 后 P0/P1/P2=0；状态为 `implementation_complete / resolving_failure / managed_validation_pending`，managed Cargo/WGPU/RenderDoc 与新 PNG 尚未验收。

2026-08-08 AT-M2 DPI render-extract 输入链路实现完成：`UiRenderExtract` 现在显式持有可序列化的 `raster_scale`，缺失、非有限或非正值统一规范到 `1.0`；`UiSurface` 从 retained `UiWindowMetrics.scale_factor` 投影该值，完整 rebuild、局部 geometry patch 与 render-cache 重建均保留它。screen-space 文本计划把同一值传给普通行、rich run 与 inline icon batch，native `TextArea.scale` 消费该规范化比例，因此 cosmic/glyphon 的 `LayoutGlyph::physical(...)` 会令物理字号进入既有 `CacheKey.font_size_bits`，继而触发 native bitmap atlas 的新物理桶；逻辑布局、shaping、Auto 路由与 SDF/MSDF atlas 不因 DPI 改变而重新构建。新增窗口 retained-extract、render batch 和非法 DTO 比例回归；全部 `UiRenderExtract` 构造点已显式指定默认 `1.0`。二次审查曾发现 native fallback overlay 缺失该必填字段，已前向修复并以非默认 `1.5` 比例回归覆盖；复审后无 P0/P1/P2。`render_text_dpi_rerasterizes_with_distinct_native_cache_entries` 现作为独立 ignored WGPU product gate：它在同一 WGPU renderer、同一 device-space frame 与同一文本输入下连续提交 1x/2x，要求 2x 阶段出现 exact source-cache miss，并比较 glyph bounds、coverage 和稳定 raster/upload 状态；该 gate 不写出 PNG，尚未实际执行，不能替代清晰度视觉验收。当前为 `implementation_complete / secondary_review_complete / managed_validation_pending`：已完成 scoped rustfmt（不含已有未格式化 render 子模块）、diff check、构造点完整扫描与静态链路检查；不绕过已归档受管 Cargo 会话，不声明 Cargo/WGPU 通过，不生成 PNG 或 accepted 输出记录。

2026-08-08 结构规范收敛：DPI 输入进入 `UiSurface` 后不再继续堆叠在超预算的 rebuild root。`surface/rebuild.rs` 保留 rebuild 编排并降至 715 行；独立的 pipeline-stage report、dirty-reason 与 cache-stat DTO 移入 `surface/rebuild/report.rs`（351 行），`UiSurfaceRebuildReport` 仍由原路径公开导出。该调整不改变布局、缓存或渲染行为，状态继续为 `implementation_complete / secondary_review_complete / managed_validation_pending`。

2026-08-08 Text04 前向编译修复与回归补强：SDF atlas fallback 在检测 shaped glyph/artifact 几何时曾把 `ScreenSpaceUiTextBatch` 按值传给仅接收引用的 helper，阻断该回退模块编译；现将该检测一次显式借用并缓存，失败 span 决策与 whole-batch native fallback 共用同一结果。overlay 回归现明确锁定显式 native `prefix`/`suffix` 与动态 `bc` overlay 的顺序，并继续验证下一稳定帧只保留显式 native decoration metrics；工厂回归已使用非空 shaped/artifact metadata 并断言 clip、style、effects、decorations、clip transform 和 non-default raster scale 的保留。完成 scoped rustfmt、tracked diff check 和旧值传递调用静态扫描；未执行 Cargo/WGPU，未生成 PNG，关联 failure 仍为 `open / implementation_complete / resolving_failure / managed_validation_pending`，计划整体保持 `in_progress`。

2026-08-24 Text04 帧级提交重审：当前 native bitmap mixed-storage 路径虽能保留 `Alpha -> Color -> Alpha` 的 painter 顺序，但将每一个连续格式段投影成一份新的 `GlyphAtlasBitmapRenderSubmissionPlan`。`storage.rs` 为每段 clone atlas、重扫 glyph/failure/placeholder/draw/upload 数组、重建 dirty/upload/batch/GPU plan；`frame.rs`、UI text 和 WGPU renderer 又逐段分配 source-byte vector、instance buffer、upload plan 和 shadow commit。交替格式时连续段数可随 glyph 数增长，因而形成 `O(R * G)` 的 CPU 重建和深复制风险，其中 `R` 是有序格式段数、`G` 是全帧 glyph 数。它是 PERF-MVP-231 已记录 mixed-storage 链的未闭合结构性 P0，不是允许全局 regroup 的理由。

本轮已完成架构和参考引擎复核，测量设计见 [`04/2026-08-24-mixed-storage-frame-plan-and-profiling.md`](04/2026-08-24-mixed-storage-frame-plan-and-profiling.md)。后续硬切必须保留唯一的帧级 atlas/submission/source-image/shadow-commit owner，以 source order 的 draw commands 在渲染阶段切换 format resource；上传按唯一 resource/page 分组，而不是按连续绘制段复制整帧。新统计将分开 `storage_resource_count` 与允许增长的 `ordered_draw_segment_count`，不能继续把连续段数当成 storage pass/resource 数。现有测试中“三个 storage submission”是旧实现细节，必须改为 painter-order、单帧计划、一次 shadow commit 和资源分组的行为契约。

状态：`architecture_review_complete / measurement_plan_complete / implementation_pending / managed_validation_pending`。尚无这项结构改造的 Cargo、GPU timestamp、RenderDoc、功耗、WGPU framebuffer 或 PNG 数据；当前计划继续为 `in_progress`，不得因本次静态复核标记实现或验收完成。

2026-08-24 Text04 帧级提交硬切实现完成：native bitmap path 已由唯一 canonical frame submission owner 持有 atlas、source images、upload plan 与一次 shadow commit；renderer 对同一 frame 仅借用该资源，而不再按连续格式段 clone atlas 或重建 plan。上传 dispatch 先构造固定 `GlyphAtlasFormat` resource table 并 preflight 全部 binding，再直接按 format replay，消除按 resource 过滤/临时 vector 的 `O(F * B)` 退化，稳定路径为 `O(F + B)` 且不新增堆分配。missing/duplicate format 的 focused source regression 已覆盖，`Alpha -> Color -> Alpha` 仍作为三个有序 draw segment 保留 painter order，但不再表示三个 storage plan 或多个 shadow commit。已完成 scoped `rustfmt --check`、scoped `git diff --check` 与 retired-dispatch 静态扫描；受管 Cargo、1/100/1k/10k 计时/GPU timestamp、RenderDoc、功耗、真实 WGPU framebuffer 和 PNG 均仍待执行。本项状态更新为 `architecture_review_complete / measurement_plan_complete / implementation_complete / managed_validation_pending`，不表示 Text04 已验收或可提交。

2026-08-28 Text04 结构规范收口：843 行的 native bitmap source-cache root 已将 raster worker
请求、完成、取消与 font identity 状态机迁入 253 行的
`text/native_bitmap_atlas/source_cache/worker.rs`；597 行 root 继续唯一持有 residency、LRU、readiness、
invalidation 与同一 cache state。原有方法可见性与调用路径保留，worker child 直接修改原 cache，不建立第二份
queue、font bytes store、raster identity 或 cache。Rust 2024 rustfmt、scoped diff-check、冲突扫描、移动方法
唯一性和文件预算静态通过；Cargo、raster profiling/功耗、真实 WGPU framebuffer 与 PNG 未执行。状态为
`source_cache_worker_owner_split_static_passed / raster_and_cache_algorithm_unchanged /
managed_validation_pending`，Text04 继续为 `in_progress`。

2026-08-28 Text04 SDF atlas iterator hard-cut compile correction：retained frame/segment prepare 已将 glyph-key
收集权威迁到 `collect_sdf_atlas_text_keys_iter`，但 cache discard 与 standalone plan 两个 production slice
入口仍调用只在 `cfg(test)` 编译的旧 slice helper，导致本轮受管 default Runtime build 报 missing symbol。
两个入口现直接传入 `texts.iter()` 消费同一 iterator owner；slice helper 继续仅供测试，未恢复 production
facade、第二份 key collector 或另一条 atlas plan。结构守卫锁定 retained iterator、两个 slice iterator 入口
和 test-only helper 边界；Rustfmt、scoped diff 与 source anchor 静态通过。该 correction 尚未获得新的 managed
Runtime/WGPU green，也没有生成 PNG。状态为
`sdf_atlas_production_iterator_owner_converged_static /
missing_test_only_slice_helper_compile_path_removed /
managed_runtime_wgpu_pending`，Text04 继续为 `in_progress`。

2026-09-01 Text04 Runtime/Editor raster authority convergence：本轮按 Unreal Slate
`FSlateFontCache` 的单一字体缓存/栅格 authority 重新审查 retained Editor、Runtime artifact、native
bitmap atlas、Swash worker 与字体代际生命周期。Runtime 已建立后端无关
`TextGlyphRasterRequest`/`TextGlyphRasterReceipt`/类型化错误和 alpha/subpixel/color 格式合同；公共
request 唯一拥有物理 ppem、3x4 phase、hinting、smoothing、mode 与 synthetic oblique。每个
`FontCollectionService` 持有一个私有同步 `GlyphRasterService`，artifact face 在栅格前后校验精确
collection/face/instance/generation/source/variation；真实 Fira Sans 服务回归覆盖 receipt identity、
bearing、phase 与 bitmap 长度。native atlas 的 `GlyphRasterKey` 也已硬切为先投影同一公共 request，
同步服务和异步 worker 共用唯一 `SwashRasterRequest::from_text_glyph_request(...)` 校验/适配器；既有
worker batching、pending 去重、背压、completion byte budget、face epoch、source LRU 与 GPU atlas
residency 算法保持不变。SDF/MSDF 与尚不支持的 synthetic bold 在此 bitmap bridge 明确 fail closed，
不再静默丢失样式。

retained Editor 已删除私有 system fontdb、Fontdue layout/metrics、Swash `ScaleContext`、无界 glyph
`HashMap`、8-bin phase、font/artifact snapshot cache 和 direct `fontdb`/`fontdue`/`swash` 依赖；布局与
绘制只消费完整 Runtime artifact/face/receipt，缺失时 fail closed，不保留第二成功路径。本轮源码范围
量化为 1,566 insertions / 5,490 deletions，相关核心生产 owner 均低于 800 行。Editor 硬切静态合同
7/7 通过；Runtime text 静态合同 114/116，通过项未回退，两项失败仍属于未修改的 UI texture owner：
旧返回类型源码字符串与 829 行 `scene_renderer/ui/image.rs` 结构门。当前终态 Runtime 定向测试的托管
D 盘验证副本 `d4da4a0a64424fbfa97845713680b0cf` 在 overlay ownership 阶段失败，Cargo
未启动；刷新逐文件 claim 后，替代副本 `1e51cc39937545b2a729d7ff8bcbd37a` / request
`8c9c4e7e59774579a4db9544fb067aee` 已受理但尚无 Cargo 结果。缓存算法优化、31-sample 性能/功耗、
100/125/150/200% 当前源码 WGPU 产品帧、PNG 检查、milestone commit 与企微通知继续 pending；计划
保持 `in_progress / source_converged / managed_validation_pending`，不得声明 Cargo、视觉、性能或功耗
验收完成。

同日结构复核又定位到 retained 稳定帧的算法缺口：layout cache 命中后，Editor 仍按 visible glyph
逐个进入 collection-owned 同步 Swash mutex 并重新发布 bitmap，当前复杂度是
`O(visible glyphs * raster)`，尚未达到 Unreal `FSlateFontCache` 风格的
`O(distinct raster misses)`。本轮只新增默认 ignored 的 managed Windows release 基线
`text_runtime_raster_authority_sync_service_repeated_request_profile`：31 组交错样本、每组 1,024 次，
输出 cold、raster 与 shared-receipt clone 的 p50/p95/p99、p95 ratio 和 bitmap bytes。动态数据和
sampled stack 回来前，不实施新的 LRU、single-flight、锁分片或异步策略，也不把既有 native source
cache 的容量直接照搬到 retained service。状态仍为
`measurement_harness_source_implemented / managed_release_profile_pending`。

同步 service 的 Swash scaler id 同时修正为原样保留 font owner 已计算的 128-bit
`BLAKE3(domain || TTC face_index || full bytes)` source identity；collection generation 继续只作为 artifact/
receipt stale fence，不再 XOR 进 source identity 高 64 位。旧折叠可让不同 `(source,generation)` 形成同一
backend id，并让相同 bytes 的安全 scaler cache 无谓失效；新回归锁定两个 64-bit lane 的逐位映射与不同
source 不别名。该项是 identity correctness correction，不是基于猜测改变 residency 容量或并发算法。

retained layout cache 同步删除只保存 `DefaultHasher<u64>` 的字体请求身份，改为精确比较
`collection generation + HostTextFontRequest(face/family/weight)`；borrowed text/request lookup 保留，
命中时不构造第二份 cache key。该修复阻止理论 hash collision 复用错误 generation/family/weight artifact，
并遵守总规范的 exact identity 要求；cache 容量与逐出算法未改变。

同日 Runtime201 M0 authority 第一段继续前向收敛：`TextModule.Manager.FontServices` 保留稳定 registry
拼写以维持 Graphics 已声明依赖，但实际 manager 从只持 font collection 的 `TextRuntimeServices` 硬切为
per-Core `TextRuntimeContext`。context 具有非零精确 identity，固定同一 `FontCollectionService` 与 compiled
Unicode provider snapshot，并只从该 collection 创建 layout session；模块 cleanup 令 context
`active -> draining -> closed`，即使外部仍保留旧 `Arc` 也不能再创建 session，重新激活会生成不同 context/
collection。源码测试覆盖同 Core 稳定、跨 Core 隔离、font/Unicode 一致、关闭拒绝和重新激活替换。该项只
建立后续 surface-owned parser/cache lifecycle、scheduler/residency/health 迁移所需的生命周期根，不宣称
Runtime201 M0 已完成；process-global convenience、完整 session/request/artifact/frame identity 和
in-flight drain 继续 open。

当前精确 production/test/Cargo 源码范围为 67 文件、2,496 insertions / 6,069 deletions（含 17 个新文件，
不含两份计划记录），生产 owner 仍全部低于 800 行。重复稳定 glyph 的 release profile 首次请求
`452fd197875f43499206952d5dc46516` 因 validation overlay 未拥有新 performance source 而在 Cargo 前拒绝；
逐文件 claim/attribution 后的替代调用在返回 receipt 前超时，按规则没有重试或轮询，因此仍没有可引用的
性能、sampled stack、内存或功耗数据。retained layout cache 审查还登记了两个未优化热点：2,048-entry
容器以 `swap_remove_index(0)` 做 FIFO-like 驱逐且 hit 不刷新 recency，key 又包含 absolute rect x/y，布局
与屏幕 placement 尚未拆开；需先测 moved-text hit/miss/eviction，再决定相对布局 key 与 LRU 修正。当前保持
`in_progress / runtime_editor_hard_cut_and_core_context_source_implemented /
managed_validation_and_release_profile_pending`，failure、WGPU 产品帧、PNG、milestone commit 与企微通知均
不得提前关闭。

同日 Runtime201 M0 产品入口继续硬切：Dynamic Session 在模块激活后只解析一次
`TextRuntimeContext`，project UI surface builder 与 fallback menu/HUD extract cache 均通过该 context 的
admission gate 创建 `SharedTextLayoutSession`，并在 `UiTextMeasureCache` 保留精确
`TextRuntimeContextId`。字体 asset claim 仍从同一 context 投影 collection，因而加载、布局、artifact 与
栅格代际属于同一 lineage；Dynamic 产品构造链不再把 `font_collection_service_for_core` 当作顶层文本能力。
`UiSurface` 不持有或序列化 process-local context，只接收 owner-aware builder 注入的 measure cache；Editor/
standalone 公共构造仍暂时保留 process-owner compatibility。context 关闭后的新 cache/session 构造以类型化
错误拒绝，相关 identity 与关闭态源码回归已补齐。59 项 focused 静态合同通过；托管 Cargo、WGPU、性能、
功耗和当前源码 PNG 尚未形成 accepted 证据，因此状态为
`runtime_ui_context_binding_source_implemented / managed_validation_pending`，Runtime201 M0 仍未完成。

该 32-path context/product 源码快照已由托管验证副本
`f83b46a6e8ac4be480abad954a32784a` / request
`84dde68097f9408eb4df88201e369c53` 在 `D:/cargo-targets/verify` 受理，focused 命令为
`zircon_runtime --lib runtime_text_context`。receipt 仍只是 `materializing`，不得视为 Cargo pass，也不因其
pending 停止其它非验收工作。

context 健康面第一段同时 source-implemented：不可变 `TextRuntimeContextHealthSnapshot` 关联 context、
lifecycle、font collection/generation、Unicode snapshot、累计成功 layout-session admission 与 active family。
每次 context admission 现在分配 per-context checked/nonzero
`TextSessionId { context, sequence }`；显式 session/cache clone 共享 family ID 与 lease，但保持独立 parser/cache
owner，standalone compatibility session 明确为 `None`。sequence/active/admission 耗尽均返回 typed error，不复用
或 saturation。该健康快照仍不是完整 `TextHealthSnapshot`：surface、request/artifact/frame identity、cursor 和
terminal receipt 尚未接入，因此不关闭 Runtime201 RT-TXT-P1-004/005。

随后 clone 调用图复审发现一处实际越界：`UiSurfaceSessionIdentity::clone()` 已明确产生新 surface owner，
但派生的 `SharedTextLayoutSession::clone()` 仍共享同一 `Arc<RichTextParser>`，使任一 clone 的 `clear()` 与
resettable telemetry 能跨 surface 生效。现已改为显式 clone：font authority、shaped/hard-line cache snapshot、
budget/report 与 context lease family 保持，parser/cache owner 重新创建。这样 context active-family 仍只计一项，
两个 surface 的 parser clear/telemetry 则完全隔离；新增回归验证相同 markup 在两个 owner 独立编译、清理一个
不影响另一个。focused 59 项非 Cargo 合同继续通过；全仓 fmt check 仅被外部 dirty Rust edition/格式差异
阻断，本目标两项源码的 `git diff --check` 通过。

健康快照加入后，变更后的 33-path 源码另以
`92ea424e58844b4a8e5714b46c0a0545` / request
`719cec96ac764c8f99e2d2443362c4ab` 受理到 D 盘托管验证副本；同样只处于 `materializing`，不轮询、不声明
terminal Cargo 证据。

builtin Graphics module host 也已改为先解析 `TextRuntimeContext`，再向 renderer 投影所需 collection；全仓
扫描确认无产品调用后，旧 `font_collection_service_for_core` accessor 与 re-export 已删除。Dynamic UI 与
Graphics 现在从同一 capability root 进入文本 authority，稳定 manager registry 拼写继续保留以兼容模块依赖
声明。该项不把 context 反向塞进 Graphics，也不宣称 renderer transport 已拥有完整 context/frame lineage。

Graphics hard-cut 后的当前 authoritative 34-path 源码已以
`11fa5758d05b44a1a02d7da542fcf1b2` / request
`d39a51d99f3d462ab8cd234e35c10c26` 受理；该 receipt 仍只是 `materializing`，之前两个 copy 只对应更早源码
快照。后续 acceptance 必须使用当前 manifest 的 terminal ticket，本轮不查询状态。

Context 边界复审同时固定为“能力共享、解析缓存不共享”：`TextRuntimeContext` 只提供 immutable
font collection、Unicode snapshot 与 lifecycle lease；`RichTextParser` 的 decorator/emoji registry、
compiled markup cache、single-flight 与 resettable telemetry 继续由每个
`SharedTextLayoutSession` 持有。`UiTextMeasureCache::clear()` 必须只清理本 surface 的 parser，
否则跨 surface 的失效与计数会互相污染。这与 Unreal 的 `FTextLayout`/marshaller 会话归属、
font cache 作为共享 authority 的分层一致。后续优化不得把该 parser 提升到 Core context；只有在
显式 snapshot/共享 owner 契约落地后，才可重新评估 session correlation，而不是借助全局缓存规避成本。

结构规范复审发现本轮已触碰的 Dynamic Runtime UI root 为 1,352 行；其 401 行既有行为测试已原样迁到
`dynamic_api/session/runtime_ui/tests.rs`，root 降至 947 行并继续只负责编排与运行时状态。该拆分不改变测试
逻辑、输入路由或 surface 行为，仅闭合 `engine-code-structure-convention` 的 touched-owner 预算。

同日 current-source 结构继续收敛：首次测试拆分后的 Runtime UI root 实际仍为 990 行，输入分发、focus/
navigation owner、pointer capture、publication refresh 与 dispatch-output recording 已整体迁入
`dynamic_api/session/runtime_ui/input_routing.rs`。root 现为 428 行，input-routing owner 为 574 行，
11 项行为测试留在 439 行的 `runtime_ui/tests.rs`；只移动既有实现，不改变输入算法或 public contract。
focused 59 项非 Cargo 静态合同与 scoped Rustfmt/diff-check 均通过。完整 Session 自有
production/test/Cargo 源码统计纠正为 71 文件、3,558 insertions / 6,648 deletions、18 个新文件；旧 67
文件统计漏记 `core/framework/text/mod.rs` 与两项 surface-artifact owner，不再作为当前权威数字。

包含新 test/input-routing owner 的当前 36-path Runtime 源码已以 claim
`021962ded48442d9b9240d476fddf209`、attribution
`64fb653e8b9547978f7df80342a8b842` 封存，并由 D 盘托管副本
`57b1960cb4c24b1f83e1ede711503bc0` / request
`86a648d825f14f9bad4d270a418df493` 受理。receipt 仅为 `materializing`，不轮询、不声明 Cargo pass；
前一 34-path copy 已被当前 manifest supersede。WGPU 产品帧、PNG、性能/功耗、milestone commit 与企微
通知仍 pending，Text04 保持 `in_progress / source_converged / managed_validation_pending`。

同日 context lifecycle 补齐第一段 active-session draining：每次 context admission 建立一个公开可读、不可伪造的
context-qualified `TextSessionId` 与 logical session-family lease，`SharedTextLayoutSession`/measure-cache clone
共享该 ID/lease，只有最后一个 clone Drop 才将 active-family 计数减一。context 进入 Draining 后拒绝新
admission；若仍有 family，`close()` 保持 Draining，最后一个 family 释放时原子转 Closed。health snapshot 新增
active-family count，state 与 active count 现打包进同一 atomic word，admit/close/release 在一个 CAS 上
线性化，不再可能发布 `Closed + active>0` 的撕裂 health；release 下溢转 Faulted，不回绕、不 panic。
ownership lease/ID 明确不参与 session 的语义 `PartialEq`。下一步继续接 surface/request/artifact/frame
correlation，而不是把 family ID 误当作 parser 或 surface owner。

纳入 `text/layout_session.rs` 与 parser-owner clone 回归时，自有源码 snapshot 为 72 文件、3,821
insertions / 6,649 deletions、18 个新文件；随后六路径 identity/lifecycle slice 当前相对 HEAD 为 986
insertions / 2 deletions（含三个 new owner），完整 manifest 总量在下一次 sealed snapshot 统一重算。前一
37-path Runtime snapshot 的 claim
`ea81d3e888724051b7cfbb73cec4f918` / attribution `5ce0b8902afa4019a93772faa17e1da8` 只覆盖
clone 修复前的内容，不能作为当前验证 manifest；本次仅刷新变更的
`text/layout_session.rs` 与 `text/layout_session/tests.rs`，lease claim 为
`657a615de131431e8cb83f441fa9a572`、baseline attribution 为
`f75867498d1a4b6680439166001f2497`。此前 D 盘 managed copy
`71435eb9a80b4cb2a7b14c3e7466f08e` / request `f546baad8fc3473eaf0a8843e5f5df6b` 仍只对应旧
37-path snapshot，receipt 仅为 `materializing`；focused 59 项非 Cargo 静态合同通过，不轮询、不声明
Cargo/视觉/性能完成。当前六路径 source claim 为 `ebc502589c9142a69e1958d9c8f2b07e`，attribution 为
`335996f7aef741f1b2ea900570eb85fe`；下一次 managed validation 必须提交完整当前 manifest。

同日 Phase A 产品 constructor 边界新增静态架构合同：Dynamic session construction、fallback Runtime UI
extract cache 与 builtin Graphics module host 必须从 Core 解析同一 `TextRuntimeContext`，正式构造段禁止重新调用
`shared_font_collection_service()`、`UiTextMeasureCache::default()` 或
`SharedTextLayoutSession::new()`。该 guard 只约束已完成 context 接线的产品根，不误扫 `#[cfg(test)]` 与已明确标注的
Editor/standalone compatibility constructor。现有 WGPU/SceneRenderer 全局入口同时标记为统一
`Runtime201 standalone compatibility`，静态合同只允许三个 owner 中四处相邻标记的 global call；新增或未标记
调用会直接失败。因此 Runtime201 `RT-TXT-P2-010` 仅推进到 product-root guard + compatibility inventory
source-implemented，不能据此关闭 `TXT-G02`。完整 feature gate、Editor preview/tool 迁移以及受管 Cargo 仍开放。
focused 非 Cargo 合同集合以 `60/60`、`1.616s` 通过；最终 guard/WGPU/SceneRenderer/icon-startup SHA-256
依次为 `6F9645DF3DD7C8EA1D9992DB9BAE814409256E133A656AB9040A80286D0489B4`、
`8BAB2656938E7029B118954AB68429D8E21B7C39C03B94EB053F9D517581FD6D`、
`8444731760941EFDD0BAD9AF1C507507662C187EBCAB17725DDB2B9B39D14258`、
`51DD683BF5D3935CA464674EBECD97423CBD232B131AF4CA7417BDF78C72A6CA`，current attribution 依次为
`d878544a9d4f44d1b5ed32059e7ed52f`、`cad18c060a3f448889465b9a4cd6cd69`、
`35bc9b52016348979a9f404ec086cfd7`、`b62957cfc4a14316a7df14d7d7475179`。三个 Graphics owner 的
whole-file Rustfmt check 暴露既有 import/test 格式差异，本切片未制造无关 formatter churn；精确 diff-check
通过。该证据仅为源码合同，不代表 managed Cargo 或产品帧完成。

同日系统字体事实源 ownership 继续收敛。模块依赖复核确认 builtin `GraphicsModule` 同时声明对
`TextModule` 与 `TextModule.Manager.FontServices` 的依赖；Core 在 Text 停用前会拒绝仍有 running Graphics
dependent 的 unload，正常关闭顺序已保证 renderer 先退出，因此没有为同一事实重复铺设贯穿 WGPU 的长期
render lease。外部越过模块生命周期继续持有 manager `Arc` 属于通用 manager API 合同，不以裸
`FontCollectionService` 投影为由扩张本切片。

对照 Unreal `SlateRenderer.cpp` / `RenderingPolicy.cpp` 的 `FSlateFontServices` 注入，以及
StandaloneRenderer D3D/OpenGL 在 renderer 创建前构造 font services 的 owner 关系，Zircon 已删除
`ScreenSpaceUiTextSystem::new_with_font_collection` 中无条件
`SystemFontPolicy::Discover` 的运行期 mutation。新增 public `TextSystemFontPolicy` 与 target-aware
`TextModule::for_target(...)`：builtin ClientRuntime/EditorHost 显式选择 `DiscoverPlatform`，Server 与默认
standalone context 为 `PackagedOnly`；策略在 `TextRuntimeContext` 创建 collection 的初始数据库时一次应用，
初始 generation 保持 1，renderer 只捕获已确定的 snapshot。health snapshot 同时发布所选 policy 与实际
discovered face count，防止策略决定与运行结果失联。该项把 Runtime201 `RT-TXT-P1-006` 推进到
`context_owned_target_policy_source_implemented`，但 versioned project profile、allowlist、locale pack、
shipping determinism/fingerprint/receipt 仍属于 `RT-TXT-P1-007` 开放项，不能标记完成。

新增源码合同锁定 renderer 生产 owner 中 `apply_system_font_policy`/`SystemFontPolicy` 为 0，并要求 Context、
FontCollectionService、TextModule、builtin target selection 与 health 字段同时存在。focused 非 Cargo 合同集
以 `61/61`、`2.276s` 通过；六个非 renderer Rust owner 的 Rust 2024 `rustfmt --check` 与八路径
`git diff --check` 通过。当前八路径 lease claim 为 `d53e6b049f844a9ca0b2c9d038805ee0`，attribution 为
`7f29530b2ad941d5bac8686ae0e90af6`；Context/health/font collection/module/mod/renderer text/builtin modules/
source guard SHA-256 依次为
`59F24E1CA5694717E637D8E7B6D03FDD2A7CDFE23CD4F4534CEE61B2001CEE36`、
`85CA1B303241A2BF8304CADC30E1E5DA2FD94753362CD7BC300515F03703DD7D`、
`6A6158F7BEFF6749FC3BB8276E62A71EB377FA01FDEC28E8487F6C8ADA8B3FC7`、
`B8AF88C7C284A1D5522E7310DBED79314B15C5AFADE792AA34D4FC12DE79D4E7`、
`7F5DB47184F645423C3DC66CE61E39030D795770B3666B721BBFDDF6358EE113`、
`A27E6DEA88EDE868991BD2157884D493869572C40C18B089631341F03A087B96`、
`075E9560C354F81B234C597117B6A8134A81C2CA4E9EAA37199066986DBE9595`、
`52FCAAE4E1E41F7D03C4F18E93DF54D60D01FD84A59C1B36A6F8C295934B256C`。本轮没有 Cargo、
WGPU、PNG、性能或功耗证据，Text04 继续 `in_progress / managed_validation_pending`。

同日 correctness 复核关闭 Runtime201 `RT-TXT-P2-002` 的一个具体缺口：shaped-cache 的
direction-alias bucket fingerprint 原本包含 collection 与 Unicode snapshot，但在 fingerprint
碰撞后的 exact candidate compare 中漏比较这两个 authority。现已补齐
`font_collection` 与 `unicode_data_snapshot` 精确比较，并加入跨 collection/跨 Unicode generation
回归；不会把不同 Context 或 Unicode 代际的 auto-direction run 当作同一结果。该修复未改变 cache
容量、bucket 索引或 eviction 算法。新增合同后的 focused 非 Cargo 集合以 `62/62`、`2.324s` 通过，
`shaped_cache.rs` 与 tests 的 Rust 2024 `rustfmt --check`、`git diff --check` 均通过。当前三路径
claim/attribution request 为 `a6b15577c79840c0bbc02376f5eeb910`；最新 SHA-256 为
`5DA7EAF1F3652D4A41D2B110F780C60CD29645B0DD8BA70D1CA8D1E7B5E61763`、
`42A9E9D5BBD21E6D9FA750842F38DDEED892AF221C2F6CF91C5CBB4741ACE4E4`、
`46D619421ABDD6DE73DC2661D7CC7E5DAC2E46B8D30E464B80A5E2A53D4ADE1B`。该 correctness slice 仍待
managed Cargo 终态；不产生性能、功耗、WGPU 或 PNG 验收结论。

同日继续关闭 Runtime201 `RT-TXT-P2-003` 的 capability-contract 缺口：`TextShapeRunProvider` 的默认
vertical 方法不再静默委托 horizontal shaping，而是 fail-closed 返回
`TextLayoutError::UnsupportedWritingMode`。Runtime 的 Direct、FontCollection 与 SharedSession provider
均已有显式 vertical 实现；依赖旧默认行为的 rich-vertical 计数测试 provider 已改为显式转发 vertical
请求。新增回归证明仅实现 horizontal 的 provider 不会伪造 vertical 成功，并加入源码合同防止默认实现
再次调用 horizontal。focused 非 Cargo 集合以 `63/63`、`2.144s` 通过，三份 Rust owner 的 Rust 2024
`rustfmt --check` 与四路径 `git diff --check` 通过。当前四路径 claim 为
`fcaba3b5c1b7437ca4fdf71dfa410b6b`，attribution request 为
`3aaa973cabca42df8d08ad6dd38223dd`；`shaping/mod.rs`、`shaping/tests.rs`、
`layout/rich_vertical/tests.rs` 与最新 source guard SHA-256 依次为
`BE51D717B05F90A18161AC2E2581165902DDDE6F73916491AA9BDF35689E439B`、
`38A5F873F06AADBBBF68D4640880F3B6764E3417E3B75C1F5D6E5933E91432F8`、
`5805033143010061D5B79A8DB4C1A64E853B2AEFED38EAFA0EBAE5962C611E74`、
`0E5016EDBA405354F0EB9673386675F5E58E3935B277154ABDBC1D32B21D6015`（该 guard 随后又加入几何合同）。这只修复 capability
边界与错误语义，不宣称 vertical shaping、性能或视觉验收完成；managed Cargo 终态仍待受管验证。

随后针对 `VerticalRl` 列布局完成一项 MVP 几何安全修复：`layout_vertical_rl_columns` 的 frame-right、列 x
坐标和测量宽度保留正常路径的原有 `f32` 舍入，仅在候选结果非有限时用未截断的 `f64` 精确中间值并收敛到有限范围，
避免有限的 `f32` 输入因中间加法、乘法或列数累积产生 `inf` 几何，再泄漏到布局/绘制阶段。正常值的从右到左放置与单次 frame 遍历没有改变；新增边界回归覆盖
`f32::MAX` frame、advance、column height 组合，并加入源码合同确保中间计算继续使用有限收敛。受限文本基础设施
契约模块本次为 `60/60`、`1.889s`；`vertical_layout.rs`、`single_pass_frame_tests.rs`、
`shaping/vertical.rs`、`shaping/vertical/direct.rs`、`shaping/vertical/tests.rs`、
`shaping/vertical/orientation.rs` 的 Rust 2024 `rustfmt --check` 与所有本轮 owned-path `git diff --check` 均通过。
当前几何 owner SHA-256 为
`9453DCD7D1AFFF9CF96E1A4B26041D2EF000C6ED9C3EFE82ACCD0CA8757135BD`、
`A07D750CA76AA9181C6AF96263BE3DC497AB552F3C66A8DC40920B6DAADD500B`、
`1FEBD79D0B3D17D7E54DF39D24DCBD1E82CC90FFA9599225D2CBE6EA3FC59827`。该修复是几何正确性保护，未声明
性能优化、功耗改善、Cargo/WGPU 构建或真实渲染截图验收；受管验证仍保持 pending。

同一边界也在垂直 shaping owner 上收敛：`apply_vertical_layout` 对 cluster advance 与 cursor 使用并行
`f32`/`f64` 累计，普通结果保持原有舍入，溢出候选改用有限 `f64` 几何；run measured width 同样受保护。
极端 glyph advance 回归覆盖 glyph x/y/advance/offset、line width 与 run extents 全部 finite，并加入源码合同
锁定该策略。`shaping/vertical.rs`、`shaping/vertical/direct.rs` 与 `shaping/vertical/tests.rs` 最新 SHA-256 为
`B81546E27CF76E54923589106FD7E1B966ECC6C9EE7847864E1314AA557588A4`、
`6CCB6BEE1CCBF7F0851E1A16C684F7F06823D17B0E54D3B2F6291B6BEB6AF295`、
`79CA9E22273C2B18169955EAF542BC3B263FBC9A56C92AD0D7D3F00D3C5089A8`；上述 `60/60` 已包含该合同。
Direct vertical positioning 现在复用公共 finite-geometry guard，覆盖 cluster advance、cursor、upright offset 和多列 measured width，
避免 Direct 与 Cosmic 两条 provider 路径在极端有限输入上产生不同的非 finite 行为。
这仍是数值正确性保护，不声明 shaping 性能、功耗、Cargo/WGPU 或 PNG 结果。

进一步的 upright vertical glyph 复核发现 `(advance - horizontal_advance) * 0.5` 对不变量外的极端输入
仍可能溢出；`vertical/orientation.rs` 现在保留普通 `f32` 结果，仅对非 finite offset 使用 `f64`
精确差值并收敛，回归锁定极端水平 advance 仍得到有限 `-f32::MAX * 0.5`。该防御性 fallback 只在候选 offset
真正非 finite 时触发；该 owner SHA-256 为
`FAFF0CA26B648F66F8B6E8771EA1414AAC64017FC1A6C15EBE04A2F2C06343FE`；这是数值正确性保护，不声明
性能、功耗或视觉验收。

布局测量 owner 随后补齐同一有限几何语义：`measure_text_size_with_provider` 对多行高度保留普通
`f32` 累加，并以并行 `f64` 总量仅在候选溢出时收敛到有限范围；两行 `f32::MAX` 行高回归确保
`TextSize.height` 不发布 `inf`。该回归与 Direct/公共垂直 guard 一起纳入 `60/60` 源码合同；
`layout/measure.rs`、`layout/measure/measured_line_contract_tests.rs` 与合同测试最新 SHA-256 为
`5A600A1170CD6DA9D979888BEDF4A5B4AE1E294F37B4E7B400E633BEBC72190B`、
`9FE53A5D0CB54791BD8FE5130861BB896D74AEBAC193871ADF87FEF0FCB41ACB`、
`1FEBD79D0B3D17D7E54DF39D24DCBD1E82CC90FFA9599225D2CBE6EA3FC59827`。
这是基础测量正确性保护，不声明性能、功耗、Cargo/WGPU 或真实渲染截图验收。

随后横排 shaping owner 补齐同一有限几何语义：Direct、Cosmic hard-line 归一化与 partial
composition 共用 `shaping/horizontal/mod.rs::position_glyphs`，对 glyph cursor 保留普通
`f32` 累计，并仅在候选溢出时用 `f64` 精确累计收敛；run 的多行 `measured_height` 同样经
`finite_sum` 保护。两个 `f32::MAX` advance 的回归同时锁定 glyph x、line width 与 height
均为 finite；受限源码合同现为 `60/60`，`horizontal/mod.rs`、`horizontal/direct.rs`、
`horizontal/composition.rs`、`cosmic.rs`、`cosmic/hard_lines.rs` 与回归测试最新 SHA-256 为
`FC639ECD3D25F293A013BC49C649471514862F4C1B76E450D4EBBE78E86E8A69`、
`82FF15212B3AD66035AB5228E6461DFAC176543E8C096AA46F5F14454773797C`、
`56EAE5D246D8A74F363A17E8E6E11899C6D9B5EB35B46FE4D8BBF5DAD19E3DAC`、
`C06FDBEF28A03816961192883183492EA680CDB757C751A3569132E78EFD4BB1`、
`336047454307AA2F8929EEFE4FAC92AAC27467FD1E75B031C4942A07C2D4D992`、
`71E8A97B6C44FC71746121FE92BBF25E8AE57589A11D3379FD0F2804D83735A5`；合同测试模块为
`1FEBD79D0B3D17D7E54DF39D24DCBD1E82CC90FFA9599225D2CBE6EA3FC59827`。这是横排数值正确性
保护，不声明 shaping 性能、功耗、Cargo/WGPU 或真实渲染截图验收；受管验证仍保持 pending。

有限几何发布规则已完成单 owner 收敛：`text/layout_geometry.rs` 现在统一拥有 `finite_geometry`、
`finite_f32_or_geometry` 与 `finite_sum`，测量、VerticalRl、水平 shaping、竖排 shaping 和 orientation
只消费该 owner，源码合同逐文件禁止同名私有实现回流。中央 owner SHA-256 为
`7E55F5AD5E35AB9C966393440A3058394B7CE62D6465A42E945AF4BE2E752180`。普通 `f32` 候选值仍原样发布，
只有候选成为非有限值时才使用 `f64` 精确旁路并饱和到有限范围；共享 cluster 几何入口同样遵循该规则，
其当前 SHA-256 为 `4253499C33B1670861E76246F652766536E540AB4C94512101C1F550B2903B7D`。tab 测量入口也已硬切到
同一 owner：tab interval、cursor、无 tab 宽度 fallback 均保持有限，回归 owner SHA-256 为
`1A7CA9018B82468245D96BBBADD921F418F32DF0AB79B6FACEFBCFAA213036C4`。这是基础几何一致性修复，不是未经 profile
证明的性能优化。Cargo/WGPU、真实渲染 PNG、功耗和性能基线仍等待受管验证。

本轮宽口径 `test_runtime_text*.py` 静态回归共运行 129 项，127 项通过；剩余两项均定位到未授权的
foreign `zircon_runtime/src/graphics/scene/scene_renderer/ui/image.rs`（既有 `UiTextureDependencies`
签名合同与 800 行 owner 预算），不属于本 Runtime 文本所有权范围，未修改、未纳入本轮修复。

优化前置门已同步到 `04/2026-08-31-retained-raster-service-convergence-review.md`：在修改系统字体发现
顺序、collection 指纹或 cache/index 算法前，必须先完成 managed Windows release 的 packaged/discovery、
1/100/1k/10k-face、冷暖 shaping、LTR/RTL/VerticalRl 与 retained-cache cohort，记录 p50/p95/p99、
allocation、RSS、fallback 顺序、身份、命中率和可用的 power/energy 数据，并与 Unreal Slate/FontServices
生命周期对照。当前只完成结构调研与测量计划，不作性能或功耗结论，也不提前改变这些算法。

## 2026-09-01 有限几何累加状态所有者修复

状态：`current-source implementation complete / managed validation pending`。

本轮在优化前重审中确认了一个基础算法错误：旧 `finite_sum` 虽在首次 `f32` 溢出时用 `f64`
总量饱和发布，但下一次加法又以已饱和的 `f32` 值作为候选；因此
`f32::MAX + f32::MAX - f32::MAX` 会错误发布 `0`，丢失溢出前的精确历史。该问题不是热点微调，
而是布局几何状态所有权不完整。

已完成项目：

- `text/layout_geometry.rs` 新增唯一 `FiniteGeometryAccumulator` owner。普通路径继续发布原有
  `f32` 顺序累加结果；一旦候选非有限，后续每次发布都从保留的 `f64` 历史恢复，直到抵消回到可表示范围。
- `finite_sum`、共享 glyph cluster、水平 glyph cursor、Direct/Cosmic 竖排 cluster/cursor、
  多行测量高度与 tab cursor 全部硬切到该 owner；源码扫描确认 `text` 模块内不再存在其它手写
  `f32/f64` 累加对。
- 新增回归锁定 `MAX + MAX - MAX == MAX`、继续抵消后回到 `0`，并更新源码合同锁定中央 owner
  及所有消费者。聚焦静态合同 `60/60`，最终复跑耗时 `2.491s`；宽口径
  `test_runtime_text*.py` 为 `129` 项、`127` 项通过，剩余两项仍仅属于 foreign
  `graphics/scene/scene_renderer/ui/image.rs` 的签名与 829 行预算，未越权修改。
- source-range glyph 宽度聚合也已从裸 `.sum()` 硬切到 `finite_sum`；两个 `f32::MAX` glyph 的
  Rust 回归已写入，动态执行仍等待受管 Cargo。
- 本轮 8 个 Rust owner 的 Rust 2024 `rustfmt --check` 与 11 路径 `git diff --check` 通过。

终端 SHA-256：`layout_geometry.rs`
`DAA0E5E9E8CCD949ECD89F1E01C8B985115F6EDDE7D00DA77EBB7526EFE79D8D`，
`cluster_geometry.rs` `9F3F51BDF1347A280EBED32D5E1FAFF1F14A66D55C3D91B6A83AF7413D0EF7C1`，
`layout/measure.rs` `5B63E8FC76B014C91C7839062BC566B3EC500A0BD613717B6CE613005CDC6F79`，
`layout/measure/measured_line_contract_tests.rs`
`B93534525855DACB65A92CC288B8D780F535C463E91BEDCAB3AE51DF9E102ED6`，
`shaping/horizontal/mod.rs` `A83381B22E2C3C1C239D815FA624CD6D825B4BDE7A7CE736A21A9E4218ED1A11`，
`shaping/vertical.rs` `531A4E995A359DEF78AFA5774F442CA74822A65AABA046D7F4A10033940E1BD2`，
`shaping/vertical/direct.rs` `5E862D4BAF43380F184FBB68A4C94ABBE053342B74CA12A98F571F538EA88550`，
`layout/tab.rs` `EB4F327B4F8A72EE67F168163A84ABD7925DB1EE68F4E28ECB662862A716256F`，
源码合同 `1825B500D16EB15AFAE95E1497B3649DD6472DAB359820AD7873B553692425A3`。

未完成边界：本项是 MVP 数值正确性和单 owner 收敛，不是性能优化结论。Cargo/WGPU、真实渲染 PNG、
功耗、release profile 与对 Unreal 经验值的量化比较仍必须通过受管 Windows 验证；在这些证据到位前
不提交里程碑、不发送企微完成通知，也不生成纯文本策略截图。
