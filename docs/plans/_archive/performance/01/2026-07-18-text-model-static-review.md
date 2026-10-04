---
related_code:
  - zircon_runtime/src/text/model
  - zircon_runtime/src/text/cache/shaped_cache.rs
  - zircon_runtime/src/text/layout_session.rs
  - zircon_runtime/src/text/shaping/script_segment.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/text/02-shaping-unicode-and-bidi.md
  - docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Fonts/FontCache.h
  - dev/bevy/crates/bevy_text/src/pipeline.rs
tests:
  - current-source Windows zircon_runtime text model/shaping tests pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime text model逐文件性能静态审查（2026-07-18）

## 范围与覆盖

`zircon_runtime/src/text/model/**`当前源10/10个Rust文件、819行已逐文件阅读，覆盖font composite/database/face/family、geometry、rich model、shaped run与style。调用图回查script segmentation/projection、layout measure、shaped cache与layout session；这些模型直接决定每次shape和resident cache的内存布局。

## PERF-MVP-233：per-glyph String与重复source ownership

`ShapedGlyphScript`仅表达ISO15924四字节tag，却存为`String`。default为每个glyph执行`"Zyyy".to_string()`，script segmentation也为实际tag逐次`to_string()`；`detailed_glyph`当前给每个neutral glyph构造default script。因此glyph数G带来O(G)小堆分配，且Clone/serde继续放大。

同一文本在shaped cache中至少有三份owned内容：entry的exact-collision `Arc<str>`、run的`source_text: String`、每条line的`text: String`。single-line是完整原文三份；多行则run全文加所有line片段再加cache key。生产搜索显示`line.text`除内存估算外无消费者，layout measure从`run.source_text + range`取片段，说明line String不是必要数据。

`normalized_open_type_features`还在cache key hash与cosmic attrs路径分别`to_vec + sort + dedup`；默认空features成本小，但非空style在同一次miss会重复规范化，归入PERF-MVP-228的key/style generation收敛。

## 参考引擎结论

Unreal `FShapedGlyphEntry`以固定宽度glyph/source/direction/flags字段保存每字形数据，sequence集中拥有glyph array；script用于shape run分段，不以String附着到每个glyph。Bevy文本管线同样把script/shaping状态留在run/布局阶段，缓存与atlas identity使用紧凑typed key。Zircon需要保留更强的可序列化诊断合同，但不应为此让每glyph拥有堆字符串。

## 目标模型与验收

Text02收到`failure-2026-07-18-shaped-run-per-glyph-string-and-text-duplication.md`。ISO tag应变为`[u8; 4]`/packed u32的新type并实现文本serde兼容；run只持一份shared source，line只持source/visual range并提供borrowed slice accessor；Text09 exact collision compare复用该source Arc，不另存原文。

验收覆盖1/100/10k glyph：script heap allocations=0、4 bytes/glyph；single/multiline source owned bytes≤一份原文+结构开销；run/cache clone只增加shared ref。必须保留Latn/Cyrl/Arab/Zsye/Zyyy等tags、source/visual range、serde roundtrip、BIDI/vertical、collision compare与cache size report准确性。动态与规模证据完成前本目录继续pending。
