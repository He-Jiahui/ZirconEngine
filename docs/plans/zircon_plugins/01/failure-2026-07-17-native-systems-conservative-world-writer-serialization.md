---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: native-systems-conservative-world-writer-serialization
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_plugins/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/plugin/native_plugin_loader/registration_manifest.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/registration_replay.rs
  - zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter.rs
  - zircon_runtime/src/plugin/extension_registry/register/system_registration.rs
  - zircon_runtime/src/scene/ecs/schedule_parallel_executor.rs
tests:
  - disjoint native-system parallel schedule fixture
  - conflicting native-system serialization fixture
  - native main-thread-affinity and invalid-access rejection fixture
---

# Plugins01：native systems 全部作为保守 World 写者串行化

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：MVP native registration replay 与 runtime schedule 静态审查
- 修复责任计划：`docs/plans/zircon_plugins/01-plugin-architecture-core.md`
- 共同验收：`docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md`、`docs/plans/zircon_runtime/runtime/11-job-system-task-model.md`
- 交接原因：这是插件 ABI access/thread-safety 契约与 ECS scheduler 的共同边界，不能由调度器猜测 foreign callback 是否安全。

## 失败现象与复现证据

`NativePluginRegistrationSystem` 已反序列化 `access: Vec<String>`，但 validation 与 replay 都不读取该字段。
`register_bridge_replay_system` 无条件调用
`register_native_system::<NativeDynamicAccess, _>(...)`；`NativeDynamicAccess` 通过
`SystemParamAccess::add_conservative_world_access()` 声明保守世界写访问。

因此每个逐帧 native system 都与所有 component/resource world access 冲突；多个彼此不相交的 native plugins
也无法进入 ECS parallel batch。把 callback 直接送入 worker 同样不安全，因为当前 ABI 没有 thread-safe、
main-thread-only 或外部状态同步契约。

## 最低共享层根因

NativeDynamic 注册 manifest 的 access 文本没有被编译为稳定、可验证的 `SystemParamAccess`，同时缺少 callback
thread-affinity/reentrancy 声明。当前 conservative writer 是正确安全 fallback，但会把插件扩展能力变成全局串行点。

## 架构修复验收

- 定义稳定 component/resource id 的 read/write access schema；注册期解析、resolve 并拒绝未知、重复冲突或越权声明。
- 增加明确 thread affinity：worker-safe、main-thread-only；默认继续保守，不得默认信任 DLL。
- 通过验证的 access 构造真实 `SystemParamAccess`；仅 unknown/legacy path 使用 conservative world writer，并输出可观测诊断。
- 两个 disjoint worker-safe native systems 在 schedule trace 中有并行 overlap；同一资源写/读写冲突确定串行。
- main-thread-only callback 始终在主线程；panic/重入/卸载 generation 与 in-flight callback 安全契约不退化。
- 记录 conflict count、ready delay、worker utilization 与 callback p95，纳入 Runtime08/11 性能预算。

## 参考引擎原则

- Bevy ECS 把系统 component/resource access 冻结进 schedule conflict graph，worker 并行来自显式 access 而非运行期猜测。
- Zircon 应迁移这一原则，同时保留 native ABI 的 capability、thread affinity 与动态库生命周期约束；不复制 Rust trait-object ABI。

## 禁止临时方案

- 不得把 `NativeDynamicAccess` 改成空 access 来制造伪并行。
- 不得仅凭 manifest 自报字符串就允许 worker 执行，必须 resolve 稳定 id、校验 capability 与 thread-safety。
- 不得为“并行”而让 hot reload/unload 提前释放仍在 worker 调用中的动态库。

## 修复结果与回传

Open state: `待 Plugins01 定义 native access/thread-affinity ABI，并由 Runtime08/11 验证 conflict graph 与 worker 调度`。
