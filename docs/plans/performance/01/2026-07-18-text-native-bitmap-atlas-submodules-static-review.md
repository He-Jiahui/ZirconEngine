---
related_code:
  - zircon_runtime/src/text/native_bitmap_atlas
  - zircon_runtime/src/text/native_bitmap_atlas.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/text/04-glyph-atlas-and-rasterization.md
  - docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md
reference_sources:
  - dev/bevy/crates/bevy_text/src/font_atlas.rs
  - dev/bevy/crates/bevy_text/src/font_atlas_set.rs
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/FontCache.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Fonts/FontCache.h
tests:
  - current-source Windows zircon_runtime native_bitmap_atlas tests pending
  - stable/empty/reappearing text residency and worker counters pending
  - Softbuffer/WGPU/RenderDoc upload and pixel parity pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime text native bitmap atlas子模块逐文件性能静态审查（2026-07-18）

## 范围与覆盖

`zircon_runtime/src/text/native_bitmap_atlas/**`当前源10/10个Rust文件、2,677行已逐文件阅读，覆盖handoff、retry frame、source cache、storage partition及全部测试。调用图继续追到`NativeBitmapAtlasFrame::prepare_report`和screen-space UI renderer；根文件`native_bitmap_atlas.rs`已在text root 7/7切片读完，本记录只补齐其子目录并核对跨文件实际执行次数。

## 回链PERF-MVP-231：稳定帧仍复制bytes并重复构建submission

每个可见glyph先从source cache取得owned `NativeBitmapAtlasCachedGlyphImage`；exact/approximate hit都会clone其中的`Vec<u8>`。retry source选择对visible sources逐个在queued glyphs中`find`，之后用`Vec::contains`排除新source，并在new/final submission阶段再次clone完整image bytes。mixed-storage partition又把每张image clone进连续format run，并为每run clone atlas、重新执行render submission规划。

调用图证明重复不只发生一次：`prepare_report()`已经调用`storage_submissions()`构造全部run、atlas与draw plan来判断replacement readiness；renderer在MixedStorage或mixed placeholder分支随后再次调用同一函数。测试明确锁定Alpha/Color/Alpha必须保持三个ordered storage submissions，因此不能通过简单按format排序合并而破坏painter order。上述问题均已由PERF-MVP-231的persistent glyph slot、shared source bytes、单一atlas owner和预计算report统计覆盖，不重复新建根因。

## PERF-MVP-242：空帧清空与线性维护制造冷启动抖动

`discard_all_for_idle_frame()`在任意无native text帧清空全部cache entries和两个pending worker map。已经提交到worker的请求并未取消；迟到completion失去work-id映射后计入unknown，CPU raster结果被丢弃。文本下一次出现时又从cold miss开始，重新产生placeholder/approximate fallback、raster和GPU upload。编辑器切tab、临时隐藏overlay或场景切换都可能把一次空帧放大成重复冷启动。

exact miss后，每个glyph调用`approximate_cached_image()`；它扫描整个最多2048项HashMap并计算最小距离。实际cache key已把`x_bin`归零，同font/glyph/size/weight/flags只剩另外最多3个`y_bin`候选，可以直接做有界key probe。满缓存插入时`evict_least_recently_used()`也全表找最小tick；连续U个新glyph形成O(U×capacity)维护成本。exact hit深clone与atlas重传仍回链PERF231，PERF242只负责residency、pending生命周期和cache索引复杂度。

## 参考引擎结论

Bevy `FontAtlas`持久保存`GlyphCacheKey -> GlyphAtlasLocation`，atlas texture保留CPU侧数据以便后续增量添加；`FontAtlasSet`再按face/size/variations/hinting/smoothing持久分组，不以空文本帧清空。UE Slate把atlas data弱引用直接缓存到shaped glyph上，miss才查`ShapedGlyphToAtlasData`共享map；atlas页按显式page阈值/flush请求管理，而不是按“本帧没有文本”刷新。两者都把空闲、预算逐出、字体失效和每次绘制无文本区分为不同事件。

## 责任计划与验收

Text04收到`failure-2026-07-18-native-bitmap-source-cache-idle-flush-and-linear-maintenance.md`，并要求与PERF231 persistent slot以及Text09 worker预算联动。最低验收是stable text后插入1/300个空帧再恢复：预算内glyph raster、placeholder、upload与unknown completion均为0；face generation变化仍精确失效。2048 resident下近似查找每miss最多3次key probe，LRU逐出amortized O(1)，1/100/1k新glyph记录probe/touched slots/CPU/alloc。current-source Cargo、真实编辑器切tab/overlay trace与Softbuffer/WGPU/RenderDoc资源上传/像素证据完成前，本目录保持pending。
