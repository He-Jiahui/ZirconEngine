---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: native-load-report-repeated-projection
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_plugins/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/loading.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_load_report/manifests.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_load_report/registrations.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_load_report/diagnostics.rs
tests:
  - native load-report projection build-count benchmark
  - package/feature registration report byte-equivalence test
  - 1k-package scaling and deterministic-order test
---

# Plugins01：native load report 重复构建 manifest/diagnostic projection

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：MVP native plugin loader/load-report 逐文件静态审查
- 修复责任计划：`docs/plans/zircon_plugins/01-plugin-architecture-core.md`
- 交接原因：package manifest、feature 与 diagnostic 投影必须由 load report 统一冻结，单个 getter 本地缓存会形成多个不一致 authority。

## 失败现象与复现证据

`load_reported_plugins_result` 先调用 `runtime_plugin_registration_reports()`，紧接着调用
`runtime_plugin_feature_registration_reports()`。两者都会独立调用 `package_manifests()`：从 discovered candidates
clone 全部 package manifests，再 clone descriptor/runtime/editor entry manifests并逐字段 merge。

生成每个 package/feature report 时又调用 `diagnostics_for_runtime_plugin`。该函数全扫 raw diagnostics，随后多次
全扫 loaded plugins/entry reports/validation reports，再排序去重。P 个 packages、F 个 features、D 条 diagnostics
会把同一事实重复投影多次；`push_unique` 对 manifest 各 Vec 使用 `contains`，大 manifest 合并还可能二次增长。

性能审计 Session 已单独消除 `diagnostic_mentions_plugin` 每条 message 的两个临时 formatted needle，但重复全扫与
重复 manifest merge 仍需共享层修复。

## 最低共享层根因

`NativePluginLoadReport` 只有原始 discovered/loaded/diagnostics 三个容器，没有按 load generation 冻结的派生索引。
每个 consumer getter 因此自行 clone、merge、filter、sort，造成重复工作并让复杂度难以观测。

## 架构修复验收

- 每次 load generation 只构建一次 package-id→merged-manifest 与 `(plugin id,module kind)`→dedup diagnostics 投影。
- runtime package reports 与 feature reports 从同一不可变 projection 消费；确定性排序只在输出边界执行一次。
- manifest 字段去重使用可证明线性的辅助索引，最终 Vec 顺序保持当前首次出现顺序。
- 1/100/1000 packages、每 package 0/10 features、10k diagnostics 的 merge/scan 计数为 O(P+F+D)，无按 report 重扫。
- 新旧 package registration、feature registration 与 diagnostics 输出做字节等价/顺序回归。

## 禁止临时方案

- 不得分别给两个 public getter 各加一个缓存，生成两份 manifest authority。
- 不得删除 deterministic sort/dedup 或改变“discovered→descriptor→runtime entry→editor entry”的 merge precedence。
- 不得把无界缓存挂到全局 live host；projection 生命周期必须受单次 load report/generation 约束。

## 修复结果与回传

Open state: `待 Plugins01 建立单次 native load projection 与规模化 build-count 回归`。
