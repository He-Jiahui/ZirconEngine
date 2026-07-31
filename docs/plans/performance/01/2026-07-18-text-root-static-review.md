---
related_code:
  - zircon_runtime/src/text/language.rs
  - zircon_runtime/src/text/layout_session.rs
  - zircon_runtime/src/text/mod.rs
  - zircon_runtime/src/text/native_bitmap_atlas.rs
  - zircon_runtime/src/text/native_buffer.rs
  - zircon_runtime/src/text/render_state.rs
  - zircon_runtime/src/text/service.rs
  - zircon_runtime/src/text/atlas/bitmap_run.rs
  - zircon_runtime/src/text/native_bitmap_atlas/source_cache.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/text/04-glyph-atlas-and-rasterization.md
  - docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/ShapedTextCache.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/SlateSdfGenerator.cpp
  - dev/bevy/crates/bevy_text/src/font_atlas.rs
tests:
  - layout_session line-text clone source guard passed
  - current-source Windows zircon_runtime text tests pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime text根模块逐文件性能静态审查（2026-07-18）

## 范围与覆盖

`zircon_runtime/src/text`根目录当前源7/7个Rust文件、1,994行已逐文件阅读：language、layout session、module wiring、native bitmap atlas、native buffer、render state与neutral layout service。为验证atlas稳定帧语义，继续回查`native_bitmap_atlas/source_cache.rs`、`native_bitmap_atlas/{retry_frame,storage}.rs`、`atlas/bitmap_run.rs`与page owner。`mod.rs`、native atlas、render state含Text01活跃owner改动，本审查只读取；直接修改仅限已独占的`layout_session.rs`。

## PERF-MVP-231：atlas没有persistent glyph slot

`NativeBitmapAtlasSourceCache::cached_image`与approximate hit返回owned image clone，其中`bytes: Vec<u8>`按每个可见glyph occurrence完整复制。该bytes随后进入per-occurrence `NativeBitmapAtlasSourceImage`；`GlyphAtlasBitmapSource`只含format/size/screen rect/color/byte length，没有CacheKey或GlyphRasterKey identity。

`glyph_atlas_bitmap_run_plan_with_atlas_and_padding`虽然接收上一帧page set，却每帧创建空shelf allocators与active-page map，对全部source重新allocate、mark dirty、生成upload copy/command。因此source cache命中只避免CPU raster，不避免稳定帧slot allocation、bitmap clone与GPU upload。render state还从frame clone atlas回自身；retry用nested find/contains匹配source并多轮clone image bytes，mixed storage的prepare report与真实提交会分别重建storage submissions并clone images/atlas。

Text04必须把glyph identity、persistent slot、page generation与face epoch收在同一owner：draw occurrence只引用slot，bytes只在新glyph或失效时上传，storage/retry借用shared source并用key index匹配。稳定帧的slot alloc/dirty/upload/bytes clone必须为0。

## PERF-MVP-232：shape双DTO往返与无界restart

cache miss路径为`SharedTextLayoutSession::shape_canonical -> dyn TextLayoutService::shape -> shape_text(internal run) -> project_shape_result(neutral runs/glyphs) -> detailed_run(internal run)`。它为同一结果建立两轮line/glyph Vec并复制source/line String；service还为font/family/language构造owned `TextStyle`。neutral contract有真实外部消费者，问题是internal session也被迫走完整投影，而不是service实现和session共享canonical owned result。

`SharedTextLayoutService::shape`用无上限`loop`包住整次shape，只要shared font database generation在前后变化就重做。在font hot reload/publish churn下，caller-thread耗时没有上界，也没有restart telemetry。

本轮TDD源码门禁先证明`text: line_text.clone()`存在，再以`line_text_len`保存长度并把owned String直接move进`ShapedTextLine`，删除每line一次冗余clone；源码门禁、`rustfmt --check`与diff check通过。这是局部止损，不代表双DTO问题关闭。

## 其余文件结论

`native_buffer.rs`每次创建glyphon Buffer、fallback spans并Advanced shape，回链PERF-MVP-227的generation-owned text preparation；language normalization的owned key分配回链PERF-MVP-228；render-state worker构造/drain回链PERF-MVP-229。module wiring与typed report本身没有新增独立热点。

## 责任计划与验收

Text04收到`failure-2026-07-18-native-bitmap-atlas-no-persistent-glyph-slots.md`，Text09收到`failure-2026-07-18-text-layout-roundtrip-and-generation-retry.md`。验收需覆盖stable 300-frame bitmap slot/clone/upload=0、重复glyph occurrence只引用单slot、mixed storage/retry近线性；以及1/100/10k glyph的canonical shape一次、internal DTO bytes=0、font-generation restart有界。current-source Cargo、workbench/Console产品trace、WGPU/Softbuffer像素与RenderDoc resource upload证据完成前，本模块保持pending。
