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

缓存键全部不持引用(`render/14` 已定 `ShapedTextCacheKey`)。同帧去重:measure 阶段与 layout 阶段共享 shaped cache,同一 (text,style) 只 shape 一次。

**缓存两级裁决(2026-07-02 评审收口,D6)**:shaping 结果与 wrap 无关(`02` 交付无宽度约束整形),`ShapedRunCache` 键**不含 wrap**——measure 以 ∞ 宽试测、layout 用实宽时命中同一 shaped run,这是"只 shape 一次"不变量成立的前提。断行/对齐结果另立 `LayoutCache`(键=shaped key + wrap + align/overflow/max_lines)缓存,见下方缓存契约表。命中后必须对缓存值内存文本副本(`Arc<str>`)做等值比较防 hash 碰撞,禁止裸 hash 直取。

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

### PF-M5 超长文本分段与虚拟化(2026-07-02 评审收口新增)

并行与缓存只解决"多而小"的文本;百 KB 级 Console log / 长文档是"单个巨大"文本,没有增量策略必掉帧。实施切片:

1. **段落级脏跟踪**:文本按段落(hard break)切分为独立 shaping 单元;编辑/追加只重 shape 受影响段,其余段命中缓存。
2. **可视区惰性布局**:仅 shape/layout 可视区 ±N 段(N 默认 2 屏);视口外行高用估算值(已 shape 段用真实值),滚动进入时精化并修正滚动偏移。
3. **单 run 上限与保护**:单 run 最大字节数(默认 64 KiB)与超限切分规则(在最近段落/强制断点处切);恶意超长单行(无断点)按上限硬切并记诊断,防 shaping O(n²) 路径拖死帧。

测试:`render_perf_text_huge_log_shapes_visible_only`(万行 log 首帧只 shape 可视区)、`text_paragraph_dirty_reshapes_edited_only`(编辑单段只重 shape 该段)、`text_oversized_run_splits_at_cap`。

## 6. 工程落地细化(实施权威)

### 模块与文件落点

实现层 `zircon_runtime/src/graphics/text/cache/`:

| 文件 | 内容 |
|------|------|
| `mod.rs` | 缓存装配(薄) |
| `shaped_cache.rs` | `ShapedRunCache`(`ShapedTextCacheKey → Arc<ShapedGlyphRun>`,LRU + 帧戳;`render/14` 键,**无 wrap**,见缓存契约表) |
| `layout_cache.rs` | `LayoutCache`(断行/对齐结果缓存,键含 wrap;2026-07-02 评审收口新增) |
| `measure_cache.rs` | measure 结果缓存(承接 `ui/text/measure_cache.rs`;桶化限制见缓存契约) |
| `frame_dedup.rs` | 同帧去重(measure↔layout 共享);命中计数 |

并行:`graphics/text/parallel/`:

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

**measure 桶化限制(2026-07-02 评审收口)**:wrap_width 桶化仅允许用于宽度不影响结果的场景(单行 / 无 wrap);有 wrap 时桶化会在断行临界点附近返回错行数/错高度,违反 index §6 #2"度量=绘制、禁止近似"。有 wrap 的 measure 键用精确宽度,或缓存"该断行结果成立的宽度有效域 `[min,max)`"并在区间内命中。

同帧去重不变量:同一 (text, style) 在一帧内 measure + layout 只 shape 一次(wrap 不同只重跑断行、命中同一 shaped run;`render_perf_text_measure_then_layout_shapes_once` 守卫,计数口径=shape 调用次数,与 wrap 宽度无关)。

### 并行隔离

- cosmic-text `FontSystem` 非 `Sync`:每 worker 持独立 `FontSystem`(共享只读 `FontDatabase` 的 `Arc<[u8]>` 字体数据),或主 `FontSystem` 加 `Mutex`(测量阶段争用低)。隔离决策在 `shape_pool.rs`,不外泄。
- 栅格(swash/fdsm)纯函数式输入输出,天然可并行;结果经通道回渲染线程。
- 确定性:并行只影响顺序不影响结果;`text_parallel_raster_deterministic` 守卫并行=串行。
- **代际与失效竞态(2026-07-02 评审收口)**:异步栅格产物回渲染线程时,目标页可能已被 `04` 页级 LRU 整页清空重建、或字体已被 `01` 卸载。上传队列条目必须携带 `page_generation` 并在应用前校验 face 有效性——代际不匹配或 face 已失效的产物**丢弃并重排队**(重排队时重新走 miss 路径),不得写入已易主的页。测试:`text_async_raster_discards_stale_page_generation`。

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
| `render_perf_text_scroll_list_reuses_cache` | 滚动列表 shape/raster 增量有界,缓存命中率高 |
| `text_async_raster_discards_stale_page_generation` | 页代际不匹配/face 已失效的异步产物被丢弃重排队,不写入易主页(2026-07-02 评审收口) |
| `render_perf_text_huge_log_shapes_visible_only` | 万行 log 首帧只 shape 可视区 ±N 段(2026-07-02 评审收口) |
| `text_paragraph_dirty_reshapes_edited_only` | 编辑单段只重 shape 该段(2026-07-02 评审收口) |

里程碑命令:`cargo test -p zircon_runtime render_perf_text --locked`、`text_cache --locked`、`text_parallel --locked`。

## 7. 风险与回退

- `FontSystem` 非 Sync 致并行受限:测量阶段争用低,可先 Mutex;shaping 重负载段再上 per-worker 实例。
- 异步上传与帧时序:缺字降级须确定性(占位规则固定),否则抓帧不稳;以 `text_first_frame_missing_glyph_degrades_not_blocks` 守恒。

## 8. 状态与产出记录

| 日期 | 里程碑/切片 | 状态 | 产出 | 验证 | 后续 |
|------|-------------|------|------|------|------|
| 2026-06-30 | PF-M1 overflow cache key interface guard | runtime_text_overflow_cache_key_interface_check_passed_focused_test_blocked | `UiTextStyleKey` 不再直接存放含 f32 的 public `UiTextOverflow`。`ui/text/resolved_layout.rs` 新增私有 `UiTextOverflowKey`,将 overflow policy 投影成 `Eq`/cache-key-safe 值,其中 clamp 字号边界以 `to_bits()` 进入 key,为后续 measure/full-layout cache 共享同一 style key 做准备。 | scoped rustfmt 通过；runtime production check 通过；editor-host build 通过；focused unit test 已加入但 runtime `--lib` tests 受既有 test-only include/E0282 与 no-default fresh target timeout 阻断,未计通过。 | 只关闭 cache key 基础接口风险;PF-M1 仍需 shaped/measure/glyph atlas 三级缓存 owner、容量/LRU/帧戳和同帧去重性能计数。 |
| 2026-06-27 | 计划建立 | planned | 三级缓存契约 + 并行 shape/raster + 异步上传 + 性能计数路线;接 render/17 | 文档 | PF-M1 缓存契约 + 同帧去重;依赖 02/03/04 |
