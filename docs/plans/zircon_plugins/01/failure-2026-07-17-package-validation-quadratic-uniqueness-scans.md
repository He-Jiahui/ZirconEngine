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

当前状态：`implementation_complete / current_source_validation_queued`。

已完成：

- 单次 borrowed package projection 已覆盖无界字符串 identity、duplicate ordinal 与 capability/interface/module/system membership；package、embedded feature、module validator 和 registration report 共享该 generation projection。
- diagnostics 仍由原 manifest Vec validation loop 发出；duplicate projection 只回答 ordinal lookup，不参与迭代，因此保留第二次及以后报告、原文本和原顺序。
- 1/100/1000 完整 fixture 每行覆盖 root、importer、dependency/interface/method、四类 contribution、embedded provider/feature/module、status/reference 和 package module/system；断言 `26*N` identity rows、`27*N` probes、package build `1`、standalone build `0`、embedded views `N`。
- standalone feature 回归独立断言 local projection build `1`、embedded view `0`；duplicate diagnostic bytes/order 另有精确断言。
- builder 已按 contributions、embedded features、interfaces、modules 拆成 folder-backed owners；主 orchestration 文件低于 300 行混域阈值。
- focused `rustfmt --check`、scoped `git diff --check` 和 conflict-marker scan 已通过；独立只读复审为 Critical `0` / Important `0` / Minor `0`。

未完成：

- canonical Rust 1.94.1 focused Cargo gate、Runtime broad/upward gate、failure audit、milestone review 登记与 coordinator atomic commit 尚未完成。
- coordinator 已恢复 healthy；旧 reservation `4041d5e7f66f4ff6877e609826b0392e` 因共享 `zircon_runtime/Cargo.toml` 合法接线 `arc-swap` 后 source-bound stale，随后 `1118a15edaae487087c611724f6e5010` 又因共享格式化改写 `package_validation.rs` 而 stale；两者均已由 owner 释放。相同 111-path scope 已重绑合并后的 current-source fingerprint `00f71a063615542ca52754c1c6eaf7928ea92465368cad8717724392ec834d29`，fresh reservation 为 `7d29e2e8d62a4c6795c24e6af12336cb`。
- 该 fresh reservation 后续未绑定 job 并已 expired；下述更新的 managed binary 证据取代它，
  不把 expired reservation 写成验收。

### 2026-07-22 current-source focused 证据

managed job `93f88e221e244b93b176afa90a07cdff` 保留的 `zircon_runtime` test binary（SHA-256
`0EAD8F289E845A8730E84EAEB51D7A97C545C306421BF2D623EAC0BCFB12B5A7`）执行完整
`package_validation` 过滤组为 `13 passed / 0 failed / 4297 filtered`；追加精确执行
`complete_package_validation_builds_one_linear_projection` 为 `1 passed / 0 failed / 4309 filtered`。
这补齐了 projection-local 与完整 registration-report fixture 的 current-source 线性验证。
Runtime/plugin broad gate、failure return 与 milestone
review 仍待完成，因此保持 `open`。

### 2026-07-30 current-source 恢复状态

- `plugins01-package-validation-current-source-r1-20260730` 已取得 package validation、module
  validation、registration report 与本 handoff 的四个目录级 lease；未与 bridge、native-host 或 discovery
  当前 owner 重叠。
- 当前 209 个 Rust 文件通过 Rust `1.94.1` scoped `rustfmt --check` 与 `git diff --check`。projection
  静态 contract 确认 duplicate identity `HashSet`、保序 identity Vec、membership probe 计数和 package
  projection build observation 均由同一 generation 提供。
- 上述静态检查不构成 current-source Cargo GREEN。历史 job
  `93f88e221e244b93b176afa90a07cdff` 的 binary 仅保留诊断背景；新的 focused linear fixture、scale
  benchmark 和 runtime/plugin broad gate 必须经 coordinator-managed Windows validator 按 FIFO 产生终态
  证据。该 failure 继续保持 `open`。
