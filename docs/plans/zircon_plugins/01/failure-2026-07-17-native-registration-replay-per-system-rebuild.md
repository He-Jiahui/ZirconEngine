---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: native-registration-replay-per-system-rebuild
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_plugins/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/registration_replay.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/bridge_methods.rs
  - zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter.rs
  - zircon_runtime/src/plugin/native_plugin_loader/registration_manifest.rs
tests:
  - native registration replay build-count benchmark for 1/100/1000 systems
  - shared bridge call-scope lifetime across reload and unload
  - registration system order/dependency behavior regression
---

# Plugins01：native registration replay 按 system 重建 bridge scope

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：MVP native plugin loader/live-host 逐文件静态审查
- 修复责任计划：`docs/plans/zircon_plugins/01-plugin-architecture-core.md`
- 交接原因：需要统一 registration manifest、installed bindings、bridge call context 与动态库 unload generation 的所有权，不能在单个 replay helper 内局部缓存。

## 失败现象与复现证据

`replay_runtime_plugin_registration_manifest_via_bridge_result` 遍历同一插件的每个 registration system。
每次迭代都会调用 `runtime_bridge_method_slot`，其内部通过
`loaded_runtime_package_manifest_required_result` clone 整份 package manifest 再线性查找 method；随后又调用
`runtime_bridge_call_scope_from_installed_bindings`，再次 clone package manifest 与全部 installed bindings，并从
完整 descriptor 集构造新的 `NativeHostBridgeCallScope`。

因此 S 个系统、M 个 bridge methods 的注册/热重载回放接近 O(S×M)，并产生 S 份 method map/context/handle。
这些 scope 最终被各 system closure 持有，不能在未定义 unload generation 与在途调用生命周期时简单共享裸句柄。

## 最低共享层根因

插件注册回放缺少按插件冻结的 `RegistrationReplayContext`：registration manifest、method-name→slot 索引、
validated bindings 和 bridge call owner 没有共同 generation/lifetime authority，system builder 被迫重复重建全部上下文。

## 架构修复验收

- 每个 plugin generation 只解析一次 registration manifest、clone/验证一次 bindings、构建一次 method lookup 与 call context。
- 每个 system registration 只保存预解析 stage/order/set/dependency、dense interface/method slot 与共享稳定 owner（例如 `Arc`）；不得复制完整 manifest/method map。
- reload 原子切换 generation；旧 generation 在最后一个 system/in-flight call owner 释放前保持动态库和 callback 有效。
- 1/100/1000 systems × 1/100 methods 的 manifest clone、binding clone、scope/context build 计数均为每插件一次，replay wall time随 system 数线性增长。
- system 注册顺序、before/after/set 语义、错误 diagnostics、hot-reload rollback 与 unload 安全契约保持不变。

## 禁止临时方案

- 不得用全局静态缓存保存 DLL callback 或 host handle，绕过 plugin generation 生命周期。
- 不得让每个 system 继续拥有深复制 method map，只把最外层 `Vec` 换成 `Arc<Vec<_>>`。
- 不得为了并行注册而改变 `RuntimeExtensionRegistry` 的确定顺序或错误提交原子性。

## 修复结果与回传

Open state: `待 Plugins01 建立按 plugin generation 共享的 registration replay context 并完成规模基准`。
