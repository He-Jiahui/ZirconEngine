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

Open state: `共享 registration replay context 源实现、current-source focused 与完整规模基准已 GREEN；broad parity、独立复审、fixed return 与 owner milestone commit 仍待完成`。

## 2026-07-22 current-source 编译修复

- Layout15 snapshot `680` 的 fresh locked upward gate job `8e229f6cd2c749f495b0f701e0c07bc0` / run `b410b0de35d14f2d9980be50241c640e` 已进入 `zircon_runtime`，在 `registration_replay.rs:392` 暴露 E0308：新共享 context 的 `method_slot_result` 返回 `NativePluginBridgeMethodError`，而既有 `NativePluginRegistrationReplayError::BridgeMethodSlot` 稳定诊断合同继续存储 `source: String`。
- `map_err` 现在只在错误边界执行 `source.to_string()`；没有扩大 replay error enum、复制 method map 或改变 system 注册顺序/共享 `Arc<NativeHostBridgeCallScope>` 生命周期。
- scoped `rustfmt --check`、`git diff --check` 与 error-diagnostic adapter source contract 已通过。受管 current-source gate 必须与 BridgeImport `as_deref()` 修复一起重跑；在 fresh GREEN 前本 failure 保持 `open`。
- current-source owner gate job `378142f7f96e45baa800695a357b2002` / run `6b2c040c395a4adfb76ec31ea0dd6a8a` 执行 `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1`，exit `0` 并自动 release；同一 `registration_replay.rs` snapshot `df339dd58d5afe843cedf01e362ba53bf7b288470d3da3e431d7b0c5fd3ce176` 通过 owner feature 编译。Layout15 warm upward gate 也确认 E0308 消失，随后仅被外部 Text01 E0502 截断；下述 focused/scale 证据补齐后，仅 broad parity、独立复审与 fixed return 仍未完成。
- managed job `93f88e221e244b93b176afa90a07cdff` 保留的 current-source test binary（SHA-256
  `0EAD8F289E845A8730E84EAEB51D7A97C545C306421BF2D623EAC0BCFB12B5A7`）执行完整
  `registration_replay` 过滤组为 `5 passed / 0 failed / 1 ignored / 4304 filtered`。忽略的
  scale benchmark 又单独通过 `1/1`；`systems={1,100,1000} × methods={1,100}` 六组均为
  `manifest_snapshots=1, binding_snapshots=1, method_lookup_builds=1, bridge_call_scope_builds=1`，
  wall 为 `3026/3863/10249/14603/177688/134896 us`。

### 2026-07-22 typed live-host generation增量证据

shared replay context已把单次replay的manifest/binding/method lookup/scope build降为每plugin一次；但live-host所有查询仍先`format!("runtime:{plugin_id}")`分配String key，公开bridge scope/single-method辅助路径还会clone完整package manifest与installed binding Vec。registration manifest source也在每次replay从loaded entry clone全文并重新parse TOML。PERF-MVP-543要求把typed plugin key、parsed registration manifest、validated bindings和dense method slots纳入同一native load generation；replay继续借用一个context，其他查询不得绕过它。

补充验收：1M stable lookups key allocation=0，单generation manifest parse=1、package/binding full clone=0、method lookup O(1)；reload只重建受影响plugin generation，旧system/scope由in-flight Arc延寿。既有5+1 focused/scale证据不失效，但broad parity、typed generation验收、独立复审与fixed return完成前保持open。

### 2026-08-02 current-source 静态恢复审计

- `NativePluginLiveKey` / `NativePluginLiveRegistry` 已取代热路径 `format!("runtime:{plugin_id}")` 键；`NativePluginRegistrationReplayGeneration` 把 manifest、组件 stable-id、capability、预解析 system 与 `Arc<NativePluginRegistrationReplayBridgeContext>` 固定在同一 plugin generation。每个已注册 system 只接收 dense interface/method slot、`Arc<NativeHostBridgeCallScope>` 与 `Arc<NativeSystemAccessPlan>`。
- `build_runtime_registration_replay_generation` 在同一 loaded-generation guard 内读取 manifest source 与 bridge generation，缓存发布以 revision 和冻结 bridge table identity 校验；reload 的 invalidate/cache 两个交错次序都有当前源码测试，避免旧 manifest 与新 callback 混配。
- 不启动 Cargo 的当前源码守护复核覆盖 `native_live_host_uses_typed_borrowed_plugin_keys`、`native_registration_replay_generation_uses_one_validated_bridge_binding_authority`、`native_registration_replay_reads_manifest_and_callbacks_under_one_loaded_guard`，并确认 `native_registration_replay_and_reload_publish_both_consistent_generation_orders` 与 1/100/1000 × 1/100 scale benchmark 仍在本地 test owner。此项只证明结构和测试覆盖仍存在，不替代运行结果。
- 外部 session 持有 validation-copy `5945e3ef29d74bd69602adca02e243b5` 与当前 Cargo FIFO 预留；本 session 未重建、重试或清理该副本，也未执行 Cargo。须取得明确的受管授权后，以 current source 跑 focused、broad 和独立复核；在此之前本 handoff 继续为 `open`，不得创建 `fixed-*` return。
