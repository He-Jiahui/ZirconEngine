---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: text-layout-roundtrip-and-generation-retry
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/text/09
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/text/layout_session.rs
  - zircon_runtime/src/text/service.rs
  - zircon_runtime/src/text/cache/shaped_cache.rs
  - zircon_runtime/src/core/framework/text
---

# Text layout双DTO往返与generation无界重试

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_runtime/src/text`根7/7 Rust文件
- 修复责任计划：`docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md`
- 责任切片：PF-M1/PF-M2。
- 交接原因：canonical shape result、cache value、线程/重试预算归Text09；neutral framework contract仍由真实外部consumer使用，不能在性能切片中旁路或删除。

## 失败现象与复现证据

internal session的cache miss调用neutral `TextLayoutService`。service构造owned style并得到internal `ShapedGlyphRun`，把它逐line/逐glyph投影为neutral `TextShapeResult`；session随后又逐line/逐glyph重建internal run和String。一次shape产生两轮DTO Vec/字符串物化。局部`detailed_run`还曾把新建line String再clone一次，本轮已直接move并加源码门禁。

service为保持font database generation一致，在整次shape前后检测generation并使用无上限`loop`重试；持续font publish/hot reload可让caller反复执行昂贵shape，没有restart计数或defer语义。静态证据见`docs/plans/performance/01/2026-07-18-text-root-static-review.md`。

## 最低共享层根因

neutral contract DTO同时被当作跨模块接口和runtime内部canonical storage，迫使实现层来回投影。font database只提供generation probe，没有为一次shape提供稳定snapshot/lease或有界optimistic retry政策，导致一致性通过无限重算维持。

## 架构修复验收

- `SharedTextLayoutService`实现与`SharedTextLayoutSession`共享一次canonical owned run或Arc；internal cache直接存该结果，neutral DTO只在真正跨framework contract调用时投影。
- 不得复制两份glyph/line/string所有权；外部trait consumer需要DTO时记录projected bytes，internal UI measure/layout路径该计数为0。
- shape读取稳定font snapshot，或最多执行明确次数的optimistic restart；超限返回typed generation-changed/retry-next-frame结果，不在caller线程无限循环。
- report增加canonical shape count、neutral projection glyph/bytes、restart count与deferred count；1/100/10k glyph规模记录alloc/owned bytes/CPU。
- 保持TextLayoutService外部ABI、font handle generation、source/visual ranges、metrics、BIDI/vertical、fallback report与shaped cache collision语义。
- 已落源码门禁禁止`detailed_run`恢复`text: line_text.clone()`；current-source Cargo与产品workbench/Console文本像素通过后回传。

## 禁止临时方案

- 不得让internal session绕过唯一shape实现另建第二套backend；应共享canonical实现，不是复制逻辑。
- 不得删除generation校验或在变化时返回混合font handles；必须stable snapshot或typed defer。
- 不得仅复用Vec capacity而保留完整internal→neutral→internal元素转换并宣称零拷贝。
- 不得以无限重试保证成功；font reload storm必须有帧预算和可观测退让。

## 修复结果与回传

Open state: `PERF-MVP-232已删除每line一次冗余String clone并通过源码/rustfmt/diff门禁；等待Text09回传single canonical run、有界generation retry、DTO/restart counter、current-source Cargo与产品trace`。
