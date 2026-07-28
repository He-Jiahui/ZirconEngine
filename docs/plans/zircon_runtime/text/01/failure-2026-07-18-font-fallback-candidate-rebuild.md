---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: font-fallback-candidate-rebuild
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/text/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/text/font/composite_resolve.rs
  - zircon_runtime/src/text/font/matching.rs
  - zircon_runtime/src/text/font/fallback.rs
  - zircon_runtime/src/text/font/database.rs
---

# Font fallback candidate每cluster重建

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_runtime/src/text/font`当前源32/32 Rust文件
- 修复责任计划：`docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md`
- 联动责任：Text02提供cluster/script/language输入，Text09提供cache容量与telemetry预算。
- 交接原因：family/composite/coverage索引和font generation失效属于Text01数据库所有权。

## 失败现象与复现证据

PERF-MVP-247：每个primary缺字cluster重新clone family列表；去重过程O(F²)且反复分配normalized String；每family clone并排序全部face IDs，再按首codepoint coverage过滤，最终候选继续Vec线性去重。现有primary match cache不缓存fallback chain，混合CJK/emoji/combining文本会重复相同工作。

## 最低共享层根因

family identity、match-order face list、composite range/culture选择和coverage查询没有编译为font-generation-owned索引；fallback resolver只缓存诊断，不缓存候选或resolution。

## 架构修复验收

- 注册/发布阶段只规范化一次family identity，并为family+weight/style/stretch建立稳定排序face view或有界generation cache。
- composite script/culture/range预编译为可binary search/interval lookup的索引；参考UE cached/merged character ranges。
- fallback candidate/resolution cache包含query、script/language、cluster coverage identity与font/composite generation，容量/bytes/eviction可观测。
- 首codepoint只允许预筛；最终face仍必须覆盖完整cluster，emoji ZWJ、variation selector、combining mark不可回退错误。
- 1/8/64 families×1/100/10k clusters记录normalization alloc、family/face visits、sort、coverage probes、cache hit/miss/bytes与p50/p95；稳定重复文本sort=0。
- Latin/CJK/emoji/combining/RTL、culture priority、partial coverage、missing diagnostics与font hot reload回归及current-source Cargo通过。

## 禁止临时方案

- 不得只把Vec去重换成HashSet而仍每cluster clone/sort全部families/faces。
- 不得仅按first codepoint缓存最终face，破坏完整grapheme cluster覆盖。
- 不得建立无界codepoint/cluster cache或跨font generation复用旧face。

## 修复结果与回传

Open state: `implementation_complete / managed_validation_pending`。Text01 已把 family identity 固定为 128-bit digest，注册去重改为 `HashSet`，family+query 的排序 face view 进入 generation cache；CompositeFont script/culture/range 在项目设置阶段编译为 direct script map + prefix-max interval index。candidate/resolution key 包含固定 query/composite identity、script/language、完整 cluster、primary 与 depth，family/composite/candidate/resolution cache 总预算 2 MiB 且暴露 hit/miss、normalization allocation、sort、family/face visit、coverage probe、bytes 与 eviction。clone database mutation 改为 detach 当前 generation 的派生 cache，阻断相同数值 face ID 的跨 generation 命中。完整 cluster、emoji ZWJ、combining/VS、RTL、hot reload 与 1/8/64 families x 1/100/10k clusters scale tests 已落代码；独立终审为 0 Critical / 0 Important / 0 Minor、Ready，仍待 fresh managed focused/broad Cargo、ignored scale、product 上行门禁与 fixed return。

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 验证与未完成项 |
| --- | --- | --- | --- |
| 2026-07-19 03:50 +08:00 | `implementation_complete / managed_validation_pending` | generation-owned family/composite index；full-cluster bounded candidate/resolution LRU；culture/range priority；hot-reload invalidation；telemetry 与规模矩阵；rustfmt/scoped diff/structure guard。 | 当前外部 whole-lib diagnostic 已越过 cache `Debug` 后只留下已修 Text visibility/type问题与 1 条 foreign Scene include；待 fresh source-bound Text jobs、review、fixed return。 |
| 2026-07-19 04:35 +08:00 | `implementation_complete / managed_validation_pending` | 补齐 normalization allocation=0、face visits telemetry；emoji ZWJ/RTL focused 回归；双 clone 同 face ID 不同 coverage 的 generation isolation 回归；mutation cache detach 并在新 generation 重新登记 CompositeFont index/bytes。 | Rust 1.94.1 rustfmt 与 scoped diff 静态检查通过；Cargo 队列受 blocking controlled action 与前序 reservations 占用，尚未执行测试断言。 |
| 2026-07-19 08:20 +08:00 | `implementation_complete / review_green / managed_validation_pending` | generation-owned glyph map 让 fallback coverage 与 SDF glyph-id 共享同一 metadata 投影；offline manifest positive/negative cache 随 font generation 清空的行为回归已补齐。 | 48 个 leased Rust 文件 rustfmt + scoped diff、9/9 结构断言通过；独立终审 0/0/0 Ready。exact Cargo 仍受 Render18 queue-1 barrier 阻塞。 |
| 2026-07-28 01:45 +08:00 | `implementation_complete / managed_broad_runtime_passed / upward_pending` | Managed current-source job `8f1c073d40ce4bee8483c046e6ee6b9b` / run `48f0711c4ca1468d90b7545df7c6e047` completed the declared `text::font` broad return. | Exit 0: `79 passed / 0 failed / 2 ignored / 8922 filtered`, covering CJK/emoji/RTL fallback, full-cluster cache identity, bounded candidate/resolution cache, hot-reload generation isolation, and composite priority. Ignored scale and product/upward responsibilities remain explicit. |
| 2026-07-28 02:42 +08:00 | `Text01_runtime_return_passed / external_editor_return_failed` | Editor job `4eefa547982a4bd896813d9fad698f21` / run `ceff37fc13224768af1c365287f242e5` compiled Runtime/Text then exited 101. | Its 56 diagnostics belong to editor-owned API, projection, lifetime, and test drift, with no Text01 source diagnostic. Keep the record open until the external editor return is repaired. |
