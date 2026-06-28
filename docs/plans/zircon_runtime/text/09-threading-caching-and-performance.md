---
related_code:
  - zircon_runtime/src/ui/text/measure_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas.rs
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
design_references:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/ShapedTextCache.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/SlateSdfGenerator.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Fonts/FontCache.h
  - dev/bevy/crates/bevy_text/src/pipeline.rs
  - dev/bevy/crates/bevy_text/src/font_atlas_set.rs
plan_sources:
  - docs/plans/zircon_runtime/text/index.md
  - docs/plans/zircon_runtime/text/02-shaping-unicode-and-bidi.md
  - docs/plans/zircon_runtime/text/04-glyph-atlas-and-rasterization.md
  - docs/plans/zircon_runtime/render/14-2d-stack.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
status: planned
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

缓存键全部不持引用(`render/14` 已定 `ShapedTextCacheKey`)。同帧去重:measure 阶段与 layout 阶段共享 shaped cache,同一 (text,style,wrap) 只 shape 一次。

## 5. 里程碑

### PF-M1 三级缓存契约定稿 + 同帧去重

实施切片:
1. `graphics/text/cache/`:`ShapedRunCache`(`02`)、`MeasureCache`(`03`,承接 `measure_cache.rs`)、atlas cache(`04` 内)——统一键/容量/帧戳/LRU。
2. 同帧去重:measure 闭包与 full layout 走同一 `ShapedRunCache`;命中计数。

测试:`render_perf_text_measure_then_layout_shapes_once`、`text_cache_lru_trims_unreferenced_at_frame_end`。

### PF-M2 并行 shaping + 并行栅格

实施切片:
1. per-paragraph 并行 shape(rayon;cosmic-text `FontSystem` per-worker 或 Mutex 隔离);Arc 结果入缓存。
2. 并行 glyph 栅格(SDF/MSDF 生成下 worker);渲染线程收集 + 上传。

测试:`render_perf_text_parallel_shape_count`(N 段并行,shape 次数=未命中段数)、`text_parallel_raster_deterministic`(并行结果与串行一致)。

### PF-M3 异步上传 + 首帧降级

实施切片:
1. glyph 栅格异步,GPU 上传渲染线程合并;栅格未完成的 glyph 首帧降级(占位/已有近似桶)避免阻塞。
2. 帧预算:超预算时限制本帧新栅格 glyph 数,余下下帧补(滚动大列表不卡)。

测试:`render_perf_text_async_upload_merges_per_page`、`text_first_frame_missing_glyph_degrades_not_blocks`。

### PF-M4 性能计数进测试(接 render/17)

实施切片:
1. `render_perf_text_*`:shape 次数、raster 次数、上传字节、缓存命中率确定性断言(时间类只观测);接 `render/17` PF 观测底座。

测试:`render_perf_text_scroll_list_reuses_cache`(滚动复用,shape/raster 增量有界)。

## 6. 工程落地细化(实施权威)

### 模块与文件落点

实现层 `zircon_runtime/src/graphics/text/cache/`:

| 文件 | 内容 |
|------|------|
| `mod.rs` | 缓存装配(薄) |
| `shaped_cache.rs` | `ShapedRunCache`(`ShapedTextCacheKey → Arc<ShapedGlyphRun>`,LRU + 帧戳;`render/14` 键) |
| `measure_cache.rs` | measure 结果缓存(承接 `ui/text/measure_cache.rs`;键含 wrap_width 桶) |
| `frame_dedup.rs` | 同帧去重(measure↔layout 共享);命中计数 |

并行:`graphics/text/parallel/`:

| 文件 | 内容 |
|------|------|
| `shape_pool.rs` | per-paragraph 并行 shape(rayon;`FontSystem` per-worker 隔离) |
| `raster_pool.rs` | 并行 glyph 栅格 → 上传队列(crossbeam 通道) |

### 缓存契约(统一)

| 缓存 | 键 | 容量/逐出 | 帧戳 |
|------|----|-----------|------|
| ShapedRunCache | `font_id + size_bits + features_hash + wrap + text_hash`(不持引用) | 1024 run / 8 MiB LRU | 本帧引用打戳,帧末 trim 未引用 |
| MeasureCache | shaped key + wrap_width 桶 | 4096 项 LRU | 同上 |
| GlyphAtlas(04) | `GlyphRasterKey` | 页级 LRU,8 页/格式 | 本帧引用页不可逐出 |

同帧去重不变量:同一 (text, style, wrap) 在一帧内 measure + layout 只 shape 一次(`render_perf_text_measure_then_layout_shapes_once` 守卫)。

### 并行隔离

- cosmic-text `FontSystem` 非 `Sync`:每 worker 持独立 `FontSystem`(共享只读 `FontDatabase` 的 `Arc<[u8]>` 字体数据),或主 `FontSystem` 加 `Mutex`(测量阶段争用低)。隔离决策在 `shape_pool.rs`,不外泄。
- 栅格(swash/fdsm)纯函数式输入输出,天然可并行;结果经通道回渲染线程。
- 确定性:并行只影响顺序不影响结果;`text_parallel_raster_deterministic` 守卫并行=串行。

### 性能预算(接 render/17)

- 帧新栅格 glyph 上限(默认按 atlas 上传带宽);超限下帧补,降级占位。
- scale 量化桶 `QUANT`(`04`):粗桶省重栅格但牺牲精度,细桶反之——默认 1px,可按场景调。
- 计数:`render_perf_text_*` 断言 shape/raster/upload 确定性计数;时间(shape 耗时/帧)只观测不断言(`render/17` 纪律)。

### 测试与验收清单

| 测试 | 断言 |
|------|------|
| `render_perf_text_measure_then_layout_shapes_once` | 同 (text,style,wrap) measure+layout shape 计数=1 |
| `text_cache_lru_trims_unreferenced_at_frame_end` | 帧末未引用 run 被 trim;引用项保留 |
| `render_perf_text_parallel_shape_count` | N 段并行 shape 次数=未命中段数,结果 = 串行 |
| `text_parallel_raster_deterministic` | 并行栅格逐字节=串行 |
| `render_perf_text_async_upload_merges_per_page` | 异步栅格,每页每帧≤1 次上传 |
| `text_first_frame_missing_glyph_degrades_not_blocks` | 缺 glyph 首帧降级占位,不阻塞,不 panic |
| `render_perf_text_scroll_list_reuses_cache` | 滚动列表 shape/raster 增量有界,缓存命中率高 |

里程碑命令:`cargo test -p zircon_runtime render_perf_text --locked`、`text_cache --locked`、`text_parallel --locked`。

## 7. 风险与回退

- `FontSystem` 非 Sync 致并行受限:测量阶段争用低,可先 Mutex;shaping 重负载段再上 per-worker 实例。
- 异步上传与帧时序:缺字降级须确定性(占位规则固定),否则抓帧不稳;以 `text_first_frame_missing_glyph_degrades_not_blocks` 守恒。

## 8. 状态与产出记录

| 日期 | 里程碑/切片 | 状态 | 产出 | 验证 | 后续 |
|------|-------------|------|------|------|------|
| 2026-06-27 | 计划建立 | planned | 三级缓存契约 + 并行 shape/raster + 异步上传 + 性能计数路线;接 render/17 | 文档 | PF-M1 缓存契约 + 同帧去重;依赖 02/03/04 |
