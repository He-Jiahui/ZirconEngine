---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: native-plugin-callback-global-lock
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_plugins/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/runtime_behavior.rs
  - zircon_runtime/src/plugin/native_plugin_loader/behavior_calls.rs
  - zircon_runtime/src/plugin/native_plugin_loader/loaded_native_plugin.rs
tests:
  - slow and reentrant native callback lock test
  - concurrent descriptor callback and lifecycle stress test
  - 1/8/32 plugin broadcast benchmark
---

# Plugins01：foreign callback 在全局 live-host mutex 内执行

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：native plugin live host、runtime behavior 与 ABI callback 逐文件静态审查
- 修复责任计划：`docs/plans/zircon_plugins/01-plugin-architecture-core.md`
- 交接原因：稳定动态库句柄、callback 生命周期、hot reload/unload 安全属于 Plugins01 的 ABI v3 与生命周期契约；上层 runtime 不应绕过它复制另一套 owner。

## 失败现象与复现证据

`invoke_runtime_plugin_command_result`、broadcast、save/restore snapshot 与 play-mode helper 先锁住 `NativePluginLiveHost.loaded`，随后在 guard 存活期间调用插件提供的 `invoke_command`、`save_state` 或 `restore_state` foreign function pointer。广播对所有 runtime plugins 的串行 callback 全部发生在同一个全局 mutex 内。

因此一个慢插件会阻塞其他插件 descriptor、state 与 lifecycle 操作；callback 若重入需要同一 `loaded` mutex 的 live-host API，非重入 mutex 会自锁。当前没有 callback duration、mutex wait 或 active-callback/unload handoff 指标。

## 最低共享层根因

live host 把“动态库句柄所有权与卸载安全”和“注册表互斥访问”耦合成同一个长持有 mutex。读取目标与维持 library 存活没有独立、可克隆的稳定 owner，调用方无法在释放注册表锁后安全执行 ABI callback。

## 架构修复验收

- 将 loaded entry 迁移到可快照的稳定 owner（例如 `Arc` entry + 显式 unload/frame-boundary 协议）；锁内只查找/克隆句柄，锁外调用 foreign callback。
- broadcast 在锁内冻结确定顺序的 entry snapshot，锁外执行；保持现有 plugin-id 排序和 report 语义。
- 慢 callback 期间 descriptor/query 可前进；callback 重入 live-host API 不死锁；unload/hot reload 等待或拒绝 active snapshot，不产生悬空函数指针。
- 增加 1/8/32 插件 broadcast benchmark、callback duration/lock-wait 诊断以及并发生命周期 stress。

## 禁止临时方案

- 不得用递归 mutex 掩盖重入，或在 ABI callback 外再增加另一个全局 callback mutex。
- 不得为缩短锁期复制 `Library` 后立即允许 unload；动态库必须在最后一个 active callback snapshot 结束后才能释放。
- 不得打乱现有确定性 plugin-id 广播顺序而不先修改契约和测试。

## 修复结果与回传

Open state: `待 Plugins01 冻结稳定 handle snapshot 与 unload 协议后修复并压测`。
