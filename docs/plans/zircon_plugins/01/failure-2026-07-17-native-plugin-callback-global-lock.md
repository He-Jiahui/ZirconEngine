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

当前状态：`implementation_complete / current_source_focused_and_scale_green / broad_validation_pending`。

### 2026-07-22 current-source 实现

- `LoadedNativePlugin` 现在共享 `Arc<NativePluginStableLibrary>`；runtime/editor behavior snapshot 在
  live-host 表锁内只取得每插件 callback lease 并复制由四个 optional ABI function pointer 组成的
  `NativePluginBehaviorCallbacks`，不会克隆 manifest 字符串；foreign callback 一律在表锁外执行。
- callback owner 维护 active count 与 lifecycle transition；unload、hot reload 和 bulk replacement 在
  active snapshot/bridge scope 存活时立即返回 typed busy error。transition 期间旧 entry 保持可查询，
  但拒绝新 callback；成功替换前先发布新 bridge bindings，再原子替换 live entry，失败路径恢复旧 owner。
- broadcast 从 `BTreeMap` 冻结 plugin-id 有序 snapshot 后锁外串行调用；单播重入 descriptor、慢 callback
  并发 descriptor、busy unload/hot-reload/bulk-load、慢 unload 锁外执行、bridge scope owner pin 与
  1/8/32 broadcast 诊断基准均已有回归。
- 诊断公开 callback active/completed/total/max duration、transition 状态，以及 live-host loaded-table
  acquire/total/max wait。后续 PERF-MVP-541 已把 per-owner state-lock 硬切为 atomic transition-bit +
  in-flight count，并让 duration/count 写入 64 个 cache-line shard；历史 lock-wait 字段与新增
  state-mutex acquisition 指标均为 0。纯 owner pin 不计入 completed/duration。
- native host bridge context 已使用 generational `ArcSwap` slot registry；稳定 lookup 不取得 writer mutex，
  scope drop 阻止新 lookup，而在途调用继续持有 context 与动态库 owner。

Rust `1.94.1` scoped rustfmt 与 `git diff --check` 已通过。首个托管 RED reservation
`226e5974212640de8a81b5db858f1d5b`、job `9f186eaafc6946748b1f07ded964d17e`、run
`c2b30ba68dca4763bc05ad56cb621ee0`（exit `101`）已证明原 focused lib-test 能进入
`zircon_runtime` 编译；其中 11 个插件/测试支撑错误已按最低 owner 修复。剩余 Render10
`RenderMeshSnapshot` 测试字段漂移已登记为
`docs/plans/zircon_runtime/render/10/failure-2026-07-22-renderer-common-direct-extract-test-field-drift.md`，
未越权修改其活跃租约源码。

相同 compatibility pool 的 current-source focused reservation
`a807fd0b35b74da1b7fe62b51dacaf63` 已绑定 job `acb2896d1cd24b61a84050784dc3f69e` / run
`32a8189d1d854365b3f4aa8d9b0438c7`，exit `101`。该轮没有 native callback/loader 编译错误；6 个
feature projection test-observation 重导出错误已在最低 owner 修正。余下 2 个错误均来自并行新增的
`zircon_runtime/src/platform/preferences/atomic_file.rs`：固定 Rust `1.94.1` 不提供
`ErrorKind::FilesystemQuotaExceeded`。跨计划阻断已登记为
`docs/plans/zircon_runtime/frameworks/05/failure-2026-07-22-preference-quota-error-kind-toolchain-drift.md`，
并绑定另一独立 Runtime10 run 的相同错误证据。该时点的外部阻断与待验收状态由以下
current-source 执行证据更新。

### 2026-07-22 current-source focused 与 broadcast 证据

Frameworks05 编译阻断已由其 owner 修复。managed job `93f88e221e244b93b176afa90a07cdff`
保留的 test binary（SHA-256
`0EAD8F289E845A8730E84EAEB51D7A97C545C306421BF2D623EAC0BCFB12B5A7`）执行
`runtime_behavior` 整组为 `10 passed / 0 failed / 1 ignored / 4299 filtered`；`behavior_calls::tests`
另为 `3 passed / 0 failed / 4307 filtered`。这覆盖重入 descriptor、慢 callback 期间查询、
busy unload、bulk unload 锁外执行、sorted broadcast snapshot、aborted snapshot 计数与 typed
unloaded error。

忽略的 1/8/32-plugin broadcast benchmark 已单独通过 `1/1`（各100 iterations）：

- 1 plugin：`2,964,900 ns`, `100` completed，loaded-lock wait total/max `37,100/24,900 ns`；
- 8 plugins：`2,763,500 ns`, `800` completed，wait `12,200/1,200 ns`；
- 32 plugins：`10,975,800 ns`, `3,200` completed，wait `19,100/1,700 ns`。

broad native-loader/plugin gate、failure return 与 owner milestone 记录仍待完成，因此保持
`open`。
