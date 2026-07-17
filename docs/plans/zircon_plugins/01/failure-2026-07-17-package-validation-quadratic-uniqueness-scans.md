---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: package-validation-quadratic-uniqueness-scans
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_plugins/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/plugin/runtime_plugin/package_validation
  - zircon_runtime/src/plugin/runtime_plugin/module_validation
  - zircon_runtime/src/plugin/runtime_plugin/registration_report
tests:
  - package validation 1/100/1000 row scaling benchmark
  - package validation diagnostic byte-and-order parity matrix
  - validation projection build-count regression
---

# Plugins01：package validation 二次扫描与二次复杂度

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：MVP plugin runtime/package-validation 逐文件静态审查
- 修复责任计划：`docs/plans/zircon_plugins/01-plugin-architecture-core.md`
- 交接原因：capability、interface、module、contribution、feature 与 package owner/target lookup 必须共享一次有序 validation projection，不能在几十个 validator 内各自增加局部缓存。

## 失败现象与复现证据

`package_validation` 当前 175 个 Rust 文件已逐文件静态通读。asset importer id/capability、package capability、dependency pair、capability status/reference、component/event/option/UI contribution、interface/method/capability、module/system、root 与 embedded feature provider 等唯一性状态普遍使用 `Vec`，每行先 `contains` 再 `push`。每一组 N 行的最坏 lookup 数为 O(N²)。

`capability_status` 还把 package capability 与所有 optional-feature capability 收集到 borrowed Vec，再由每条 status 线性 `contains`；target coverage 与 embedded feature module coverage又分别扫描 package target Vec。registration/module/feature validation 存在相邻的重复 export/import/declaration/system-anchor 扫描。小型 enum target/platform 列表有固定低上限，不应单独扩大设计；字符串身份集合与跨 validator 重扫才是规模根因。

## 最低共享层根因

manifest validation 没有一次性的、保序的 identity/membership/owner/target projection。每个细分 validator 自建 Vec state，既无法共享 lookup，也无法统一记录 probe/build-count；简单把每个 Vec 换成 HashSet 会制造多份 authority，并可能改变 diagnostics 的首次出现顺序。

## 架构修复验收

- 单次 package/registration generation 建立 borrowed 或 interned validation projection，至少覆盖 capability、interface/method、module/system、contribution、root、provider 与 owner/target membership。
- diagnostic 仍按 manifest 原始顺序发出；duplicate 仍只在第二次及以后出现时报告，文本、数量与顺序逐项等价。
- 1/100/1000 rows 的完整 validation benchmark 证明总 membership probe 线性，projection build count 为一次，String clone 只来自实际 diagnostic。
- module/feature/registration report 消费同一 generation projection，不在上层重新扫描 exports/imports/declarations/system anchors。
- 产品 bootstrap、editor discovery/hot reload 与 export trace 标注 validation frequency；该项按注册规模处理，不描述成稳定帧热点。

## 禁止临时方案

- 不得在每个 validator 内各建一份 HashSet/BTreeSet cache。
- 不得为换容器改变 diagnostics 顺序、重复项判定或首次出现语义。
- 不得把上限很小的 target/platform enum Vec 与无界字符串身份集合混为同一优先级。
- 不得把 validation 跳过、fail-fast 或缓存陈旧结果当作性能修复。

## 修复结果与回传

Open state: `待 Plugins01 建立单 generation 有序 validation projection，并回传规模 benchmark、diagnostic parity 与 build-count 证据`。
