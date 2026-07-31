---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: shaped-run-per-glyph-string-and-text-duplication
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/text/02-shaping-unicode-and-bidi.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/text/02
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/text/model/shaped_run.rs
  - zircon_runtime/src/text/shaping/script_segment.rs
  - zircon_runtime/src/text/shaping/horizontal/projection.rs
  - zircon_runtime/src/text/layout_session.rs
  - zircon_runtime/src/text/cache/shaped_cache.rs
---

# Shaped run逐glyph String与source text重复所有权

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_runtime/src/text/model/**`当前源10/10 Rust文件
- 修复责任计划：`docs/plans/zircon_runtime/text/02-shaping-unicode-and-bidi.md`
- 联动责任：cache ownership联动Text09 PF-M1。
- 交接原因：script tag、ShapedGlyph/Run/Line公共模型与serde归Text02；Text09只消费该模型，不能在cache内另造不兼容压缩副本。

## 失败现象与复现证据

每个`ShapedGlyph`持有`ShapedGlyphScript { iso15924: String }`。default和script segmentation都分配四字符String；10k glyph至少10k次小堆分配。shaped cache又同时保存exact text Arc、run source String和line text String；single-line完整原文三份，multiline为key全文+run全文+各line片段。`line.text`生产调用图只命中cache size估算。

`normalized_open_type_features`在key hash与cosmic shape各复制排序一次非空feature list，说明style/key也缺少canonical normalized identity；该部分联动PERF-MVP-228。静态证据见`docs/plans/performance/01/2026-07-18-text-model-static-review.md`。

## 最低共享层根因

模型把run级script分类和source view物化成glyph/line级owned payload，以便直接derive Clone/serde；cache为碰撞防护又单独复制source ownership。缺少packed ISO tag、single source owner与range-based view，使算法正确性数据和诊断序列化成本永久进入热路径。

## 架构修复验收

- 新建Copy ISO15924 tag type，内部为`[u8; 4]`或packed u32；自定义serde继续输出/接受四字符tag，拒绝非法长度/字节。
- `ShapedGlyph`不再包含heap-owned script；Latn/Cyrl/Arab/Zsye/Zyyy与horizontal projection API等价。
- `ShapedGlyphRun`只拥有一份shared source；`ShapedTextLine`保存range/offset并通过run accessor借用slice，删除line String。
- Text09 `ShapedRunCacheEntry`复用run source Arc做exact collision compare，不另存第三份文本；estimated bytes按实际unique ownership计算。
- normalized OpenType feature identity在style/key generation一次完成，key hash和shape attrs复用；非空features每request normalize≤1。
- 1/100/10k glyph记录script allocations/bytes、source owned copies/bytes、run clone bytes与cache estimated-vs-actual；目标script alloc=0、source owned copy=1、clone只增shared ref。
- serde roundtrip、source/visual ranges、line measure、BIDI/vertical/script/emoji tests、cache collision与产品文本像素全部通过。

## 禁止临时方案

- 不得把四字符String换成`Arc<str>`后仍执行每glyph原子引用/clone；script必须inline Copy。
- 不得删除exact text比较，仅凭hash消除cache source副本。
- 不得保留line String再增加borrowed accessor；旧重复所有权应hard cut。
- 不得用错误的cache memory estimate掩盖实际重复bytes；报告必须按unique allocation校准。

## 修复结果与回传

Open state: `PERF-MVP-233已完成model 10/10静态审查；等待Text02联动Text09回传packed script、single shared source、range-only line、serde兼容、规模内存/分配、current-source Cargo与产品trace`。
