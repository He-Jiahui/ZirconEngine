---
related_code:
  - zircon_runtime/src/core/framework/text
  - zircon_runtime/src/text/service.rs
  - zircon_runtime/src/text/layout_session.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md
reference_sources:
  - dev/bevy/crates/bevy_text/src/pipeline.rs
  - dev/slint/internal/core/textlayout/shaping.rs
tests:
  - sixteen of sixteen framework text Rust files reviewed
  - existing PERF-MVP-232 source guard remains the direct local mitigation
  - current-source Cargo, allocation counters and F2/F4 product traces pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime text framework逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`zircon_runtime/src/core/framework/text/**`当前Rust文件16/16（285行），覆盖direction/render/writing enums、borrowed font/shape requests、copy generation handles、owned glyph/run/result DTO、layout service trait与tests；并回查`text/service.rs`、`text/layout_session.rs`的生产实现与graphics/UI消费者。

## 结论：复用PERF-MVP-232，不建立重复根因

请求侧没有新增独立热点：`TextFontRequest`与`TextShapeRequest`借用families、asset、language和text，face handle为Copy。结果侧`TextShapeResult -> Vec<TextShapeRun> -> Vec<TextGlyph>`必须拥有全部glyph结构；这对真正跨framework边界的消费者合理，但当前internal session也调用neutral trait，service先把internal `ShapedGlyphRun`逐line/glyph投影成neutral Vec，session随后又重建internal Vec/String。该完整往返、font generation无界restart及局部line String clone已由PERF-MVP-232和Text09的`failure-2026-07-18-text-layout-roundtrip-and-generation-retry.md`负责；本轮不以同一DTO现象增加新编号。

修复边界仍是single canonical owned run/Arc：runtime internal session/cache直接消费canonical artifact，neutral DTO只在真实外部trait边界按需投影并计数；不能通过再加一套shape backend或只复用Vec capacity掩盖逐元素转换。Bevy/Slint参考路径同样先持有可复用的shaped/layout artifact，再由布局/输出阶段消费，而非在内部层间往返等价owned DTO。

## 验收要求

按1/100/10k glyph、1/100/1k runs、text 0/16 B/1 KiB/1 MiB记录canonical shape count、neutral projection glyph/bytes、alloc/realloc、source String bytes、font-generation restarts/deferred与CPU p50/p95/p99：internal UI measure/layout每唯一generation shape≤1且neutral projection bytes=0；真实external trait projection只发生一次并可观测；restart有明确上限，超限typed defer。source/visual ranges、font handle generation、BIDI/vertical、metrics、fallback/cache与像素等价，current-source Cargo及F2/F4产品trace完成前，本目录留在`pending.md`。
