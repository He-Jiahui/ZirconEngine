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
  - projection_preserves_registration_and_diagnostic_outputs_as_json_bytes
  - native_load_projection_preserves_order_and_projection_statistics
  - public_load_report_getters_share_one_frozen_projection
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

当前状态：`implementation_complete / managed_current_source_validation_queued`。

- 每次 native report operation 只建立一个 immutable `NativePluginLoadProjection`；runtime live host 与 editor export/status/enablement consumers 复用同一 package/feature/diagnostic projection。
- manifest precedence、package/feature registrations、runtime/editor/all diagnostics 均有 JSON byte-and-order parity；1/100/1000 package fixture 约束 projection build count 恒为一次并保持线性计数。
- 独立只读终审为 Critical `0` / Important `0` / Minor `0`；scoped rustfmt、diff-check、conflict-marker 与文件预算检查已通过。
- maintenance 后原 21-file source manifest `ab84546f` 曾按当前源码复核 `21/21`、零漂移；随后共享 `zircon_runtime/Cargo.toml` 合法接线 `arc-swap`，旧 reservation `015ebc59344347bcad7238e054fe8500` 已由 owner 释放。相同 21-path scope 已重绑 current-source fingerprint `9f593d78c8a0c4c93e82df275287989f70d50625a92821d0d94c675b4688602a`，fresh canonical Rust 1.94.1 reservation 为 `afa24bc2ccbb4cfd94caba5143cbc542`。
- 在 fresh focused/broad GREEN、failure return、milestone review 与 coordinator atomic commit 完成前，本 failure 保持 `open`。

### 2026-07-22 focused 回归追踪

coordinator job `93f88e221e244b93b176afa90a07cdff` 保留的 current-source test binary 执行
`native_plugin_load_report::tests` 得到 `7 passed / 1 failed`。唯一失败
`projection_preserves_registration_and_diagnostic_outputs_as_json_bytes` 的 projection 产出包含
optional-feature/package-module validation diagnostics，而 hard-coded parity expected 不包含它们。追溯
`RuntimePluginRegistrationReport::from_native_package_manifest` 确认生产契约一直执行完整
package validation；根因是 scale/parity fixture 构造了缺失 capability、primary dependency 与
target mode 的非法 feature/module，不是 projection 遗失诊断。

fixture 现在为 package/feature 建立完整 capability owner、唯一 primary dependency 和
`ClientRuntime` module target，并让 discovered/runtime entry 复用字节等价的有效 runtime
module，使 parity 测试只比较 projection 本身的 manifest/diagnostic 顺序。scoped Rust
`1.94.1` rustfmt 与 diff-check 已通过；重编译 focused 复验前不宣称整组 GREEN。

broad `native_registration_reports_preserve_per_plugin_loader_diagnostics` 另暴露历史文本断言
`library is missing` 已落后于 typed `PluginLoadError::MissingArtifact` 合同。断言已改为同时
核对 `native plugin weather library-open failed` 以及
`expected native dist library, actual artifact missing`，从而继续证明 projection 把带 plugin-id 边界的
typed loader diagnostic 保留到 registration report，而不回退到旧字符串。fresh 编译复验待完成。

### 2026-07-30 current-source projection authority correction

`NativePluginLoadReport` 当前已以 report-owned `OnceLock<NativePluginLoadProjection>` 在首次请求时冻结
manifest/diagnostic indexes；`package_manifests`、entry/descriptor/plugin diagnostics 与 runtime package/feature
registration getters 都经同一 `self.projection()` authority，不会重建或另设 getter-local cache。
`public_load_report_getters_share_one_frozen_projection` 覆盖 `100 packages x 10 features x 10K diagnostics` 的
连续 public getter 调用，并断言同一 projection 指针、`projection_builds=1`、manifest source scan `400` 与 raw
diagnostic scan `10K`。本次复核同时修正了 manifests/registrations/loading 的 import-order formatting drift；
Rust 1.94.1 scoped rustfmt 与 diff-check 已通过。

该 current-source 证据不替代 managed focused/broad Cargo、byte/order parity 与 failure return；它们完成前
handoff 继续保持 `open`。
