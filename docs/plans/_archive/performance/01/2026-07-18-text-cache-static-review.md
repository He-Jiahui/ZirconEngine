---
related_code:
  - zircon_runtime/src/text/cache
  - zircon_runtime/src/ui/text/measure_cache.rs
  - zircon_runtime/src/ui/surface/surface.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/ShapedTextCache.h
  - dev/bevy/crates/bevy_text/src/font_atlas.rs
  - dev/bevy/crates/bevy_text/src/pipeline.rs
  - dev/slint/internal/core/textlayout/sharedparley.rs
tests:
  - current-source Windows zircon_runtime text cache tests pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime text cache逐文件性能静态审查（2026-07-18）

## 范围与覆盖

`zircon_runtime/src/text/cache/**`当前源6/6个Rust文件、1,900行已逐文件阅读：`frame_dedup.rs`、`measure_cache.rs`、`layout_cache.rs`、`shaped_cache.rs`、`mod.rs`与`tests.rs`。调用图确认这些不是孤立工具：`UiTextMeasureCache`同时持有四类缓存，`UiSurface`常驻该owner；`ShapedRunCache`还被layout session与parallel shape pool消费。因此本结论直接覆盖基本编辑器的measure、layout、prewarm与draw-text路径。

## 发现：PERF-MVP-228

四类缓存都把条目存入`Vec`。`TextMeasureCache`默认4096项、`TextLayoutCache`默认2048项、`ShapedRunCache`默认1024项/8 MiB；同帧dedup没有独立容量。命中和update均从头线性查找，persistent cache逐出先全表`min_by_key`，随后`Vec::remove(index)`移动尾部条目。稳定帧越依赖cache复用，命中本身越按resident entry count放大；容量边缘的insert/trim还叠加扫描与搬移。

`ShapedRunCacheKey::from_request`每次热查询还会为normalized font family与language生成owned `String`，然后才进入线性查找。其Auto/Mixed到resolved direction复用、exact text碰撞校验和layout width validity range说明不能用裸hash直接替换；但这要求bucket内精确比较，不要求扫描全部resident entries。

现有tests只用容量2/4验证collision、direction alias、LRU与report语义，没有16/256/1024/4096项的probe、allocation或eviction搬移门禁。当前资产模块编译错误会在`zircon_runtime`测试执行前终止，因此本目录不能进入`review.md`。

## 参考引擎结论

- Unreal `FShapedTextCache`用`TMap<FCachedShapedTextKey, FShapedGlyphSequencePtr>`索引整形结果，避免为正常命中遍历整个缓存。
- Bevy的`FontAtlas`以`HashMap<GlyphCacheKey, GlyphAtlasLocation>`查询字形，并按generation裁剪font source cache；热查询与生命周期治理是两个独立责任。
- Slint按`ItemRc`持有段落cache，显式处理scale-factor/component生命周期失效，并暴露cache miss counter；layout阶段临时取出paragraph Vec后以RAII归还，避免重复shape。

Zircon应采用相同原则，但保留自身exact-text碰撞防线、方向alias、双容量上限与确定性report，不复制参考引擎的对象模型。

## 责任计划与验收

Text09 PF-M1/PF-M4收到`failure-2026-07-18-text-cache-linear-lookup-and-eviction.md`。目标结构为hash bucket定位候选、stable slot保存entry、O(1) touch或generation queue实现amortized O(1)逐出；frame dedup可按帧整体clear索引。key规范化字符串应由shared/interned identity或style generation预计算，热命中owned bytes为0。

验收矩阵覆盖16/256/1024/2048/4096 resident entries的hit/miss/update/evict，记录probe/visited/moved entries、key allocation bytes与wall-clock分布；语义测试必须继续覆盖forced hash collision、Auto/Mixed direction reuse、width validity interval、同帧dedup、双上限LRU与report计数。随后运行`render_perf_text_scroll_list_reuses_cache`、典型workbench/Console产品trace，确认稳定帧cache lookup不成为新的主线程热点。

## 动态状态

本轮没有宣称Cargo通过。受管scene job `aefa636dfd58408bb195716eacb771ba`已证明当前`zircon_runtime`在执行测试前被资产模块两个编译错误挡住；资产owner恢复编译后，再通过协调器运行Text09计划规定的`render_perf_text`、`text_cache`、`text_parallel`测试与上述规模门禁。完成前本模块保持`static_complete_dynamic_pending`。
