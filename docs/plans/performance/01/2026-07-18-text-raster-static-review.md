---
related_code:
  - zircon_runtime/src/text/raster
  - zircon_runtime/src/text/parallel/raster_pool.rs
  - zircon_runtime/src/text/native_bitmap_atlas/source_cache.rs
  - zircon_runtime/src/text/font/database.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/resolved_batches.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
  - docs/plans/zircon_runtime/text/04-glyph-atlas-and-rasterization.md
  - docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/FontCacheCompositeFont.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/FontCache.cpp
  - dev/bevy/crates/bevy_text/src/pipeline.rs
  - Cargo.lock (swash 0.2.9)
  - swash 0.2.9 dependency source src/{font.rs,cache.rs,scale/mod.rs}
tests:
  - current-source Windows zircon_runtime raster tests pending
  - per-face proxy/hint/variation/raster counters pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime text raster逐文件性能静态审查（2026-07-18）

## 范围与覆盖

`zircon_runtime/src/text/raster/**`当前源10/10个Rust文件、1,461行已逐文件阅读，覆盖bitmap/SDF/MSDF/MTSDF policy、Swash request/source/bitmap validation、atlas source bridge、color strike、rasterizer/error与536行测试。调用图追到native bitmap source cache、raster worker pool和screen-space UI route，确认worker每线程复用一个`SwashRasterizer/ScaleContext`，但face identity与请求准备仍按glyph发生。

## PERF-MVP-240：fresh Swash identity让worker cache持续miss

`SwashRasterizer::rasterize`每个glyph执行`FontRef::from_index(font_data, face_index)`。Swash 0.2.9源码明确该入口经`FontDataRef::new(...).get(index)`执行基本验证并创建fresh `CacheKey::new()`；`ScaleContext::builder`的`FontCache`默认以该key作为identity。因此同一worker即使长期复用8-entry `ScaleContext`，连续同字体glyph也会被当作不同font proxy，proxy/hint cache无法按face稳定命中；nonzero TTC offset时builder还再次枚举font index，源码自带“make this faster”注释。

请求提交前，`NativeBitmapAtlasSourceCache`对每glyph调用`FontDatabase::effective_variations`；该路径重新parse `ttf_parser::Face`、collect axis Vec、BTreeMap规范化/量化。`glyphon_cache_key`先为wght创建variation Vec，成功解析后又通过`request.clone().with_variations`clone并替换。PERF-MVP-229另记录同一font bytes按glyph复制；三者共同让worker异步只搬走最终raster，昂贵face/variation准备仍在caller且worker cache失效。

Text04应以`FontFaceId/InstancedFaceId + generation`传稳定Swash `builder_with_id`或owning font identity，并按face/size/instance batch glyph；Text01发布shared bytes、face offset/key、parsed axes与normalized variations，Text09约束每worker resident face/scaler数量和内存。不能在Rasterizer里缓存借用临时Arc的`FontRef`。

## PERF-MVP-241：Auto raster policy未接产品路由

`GlyphRasterPolicy`包含24px阈值、scalable、format与effects选路，测试覆盖Bitmap/SDF/MSDF/MTSDF；但产品调用图没有调用`raster_path_for`或`raster_path_for_request`来决定native-vs-distance-field。renderer只在创建batch时用`distance_field_mode_for_request`选择距离场内部模式；`ResolvedScreenSpaceUiTextBatches`的Auto路由由font asset default决定，无asset时固定Native。

因此普通Auto大字号/缩放文本无法按policy转SDF，可能为连续物理字号建立bitmap variants；反之直接接入固定24px还会在动画跨阈值时抖动。Text04需用实测route cost/residency和hysteresis形成每command generation一次的统一决策，显式render mode保持硬语义。

## 参考引擎结论

UE composite font cache以`FFontData -> shared FFreeTypeFace` map持久复用face/memory，glyph/advance/kerning cache都以face identity和size为键。Bevy/Parley在一个positioned glyph run内只构建一次scaler并循环全部glyph，再按FontAtlasKey/GlyphCacheKey复用atlas；Zircon当前是每glyph构建scaler。Swash本身提供`builder_with_id`与最多64-entry `ScaleContext`，其owning-font文档要求保存首次解析得到的offset/key，正是当前逐glyph`from_index`没有做到的契约。

## 责任计划与验收

Text04收到`failure-2026-07-18-swash-face-cache-identity-and-auto-routing.md`，face/variation与worker/cache分别联动Text01/Text09，font bytes回链PERF-MVP-229。验收覆盖同face/instance 1/100/1k glyph、1/8/64 faces、12/24/48/96px及zoom动画：face/axis parse、proxy/hint hit/miss、variation alloc、raster/upload keys/bytes、CPU p50/p95；stable face proxy miss≤1/worker、per-generation parse≤1、Auto route不抖动。current-source Cargo与Softbuffer/WGPU/RenderDoc像素完成前，本目录保持pending。
