# M5 Windows Tray and Supervision

Plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
Milestone: M5
Status: completed
Files: ["docs/cli-and-tooling/local-session-coordinator.md", "docs/plans/zircon_tooling/session_coordinator/01/2026-07-12-m5-windows-tray-and-supervision.md", "tools/install-session-coordinator-task.ps1", "tools/install-session-tray-startup.ps1", "tools/session_coordinator/baselines.py", "tools/session_coordinator/cli.py", "tools/session_coordinator/client.py", "tools/session_coordinator/control_plane/actions", "tools/session_coordinator/migrations.py", "tools/session_coordinator/processes.py", "tools/session_coordinator/server.py", "tools/session_coordinator/supervision", "tools/session_coordinator/tests/test_runtime_descriptor.py", "tools/session_coordinator/tests/test_supervision_actions.py", "tools/session_coordinator/tests/test_supervision_schema.py", "tools/session_coordinator/tests/test_supervision_service.py", "tools/session_coordinator/tests/test_watch.py", "tools/session_coordinator/tests/test_workflow_schema.py", "tools/session_coordinator/tests/test_workflow_topology.py", "tools/session_coordinator/watch.py", "tools/session_tray", "tools/zircon-session.ps1"]

## 状态与产出记录

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M5 | M5.0 后台扫描去全局锁饥饿 | `completed` | 2026-07-12 | `BaselineService` 将全量 Git/文件哈希拆成锁外 `prepare_scan` 与锁内 epoch/HEAD CAS `apply_scan`；陈旧观察直接丢弃，不覆盖并发 baseline；维护线程仅在短提交阶段持有 mutation lock。新增陈旧观察与阻塞扫描并发回归测试，统一在 M5-T 执行。 |
| M5 | M5.0b Cargo 产物复用与即时清理 | `completed` | 2026-07-12 | schema 21 为 Cargo job 增加完整兼容文档、复用 key、保留/一次性策略、清理状态与来源 job；仅在仓库、平台、Rust toolchain、target、workspace、profile/features/flags 完整匹配时复用唯一空闲 pool，兼容信息缺失则 fail-closed 为 ephemeral；release 后立即经 reservation/PID/lease/managed-root 复核删除，失败由守护维护循环重试。61 个 Cargo/cleanup/schema 聚焦测试通过。 |
| M5 | M5.1a schema 20 监督持久化 | `completed` | 2026-07-12 | 新增独立 `supervision` 迁移模块；安全重建 action request/approval 表以扩展 drain/resume/stop/restart/force-stop 封闭动作枚举，同时复制 v19 历史并重建外键、索引、枚举触发器与审批不可变触发器。新增严格枚举的监督事件、恢复状态、持久生命周期意图表及不可变事件约束；迁移/历史/约束测试统一在 M5-T 执行。 |
| M5 | M5.1b 独立托盘 workspace | `completed` | 2026-07-12 | 新增不加入根 workspace 的 `tools/session_tray`，固定 Tauri 2、serde、Windows API 依赖，无可见主窗口；首次受管 `cargo check` 在 `D:\cargo-targets\session-tray-m5-20260712` 通过。 |
| M5 | M5.2 运行时与进程身份 | `completed` | 2026-07-12 | runtime descriptor v2 同时绑定规范化仓库 key、PID、进程创建时间、可执行文件、命令行、daemon instance、schema/API 版本和认证 health；仓库级命名 mutex 阻止重复托盘；secret 类型禁止复制和 Debug 泄漏。 |
| M5 | M5.3 原生托盘状态与通知 | `completed` | 2026-07-12 | 严格监督枚举映射 tooltip/菜单权限，Busy 仅为活动阻塞项派生态；控制台通过一次性 Observer ticket 打开；诊断 JSON 不含 token；Windows Toast 只在状态变化时发送并限频；退出托盘只退出客户端。 |
| M5 | M5.4 受控生命周期 | `completed` | 2026-07-12 | drain/resume/stop/restart/force-stop 全部经过受控 action 预览与确认；服务排空门允许在途 Cargo/Lease 收尾；stop/restart 使用持久 intent；force-stop 在打开终止句柄后再次核验完整进程身份；旧 `/shutdown` 已改为拒绝并要求 controlled action。 |
| M5 | M5.5 恢复与独立启动项 | `completed` | 2026-07-12 | 托盘恢复控制器实现 1/2/5/15/30 秒退避、十分钟五次熔断、十分钟健康清零；无最后可信健康态、身份不匹配、迁移/完整性失败、显式停止、维护 hold 均禁止猜测重启。协调器和托盘分别注册当前用户启动项，使用绝对路径且不保存凭据。 |
| M5 | M5.6 单元与操作文档 | `completed` | 2026-07-12 | Cargo/cleanup/schema 聚焦 61 tests 通过；监督门次序与 schema 期望回归修复后 5 个聚焦测试通过；托盘 `cargo fmt --check`、`cargo build --locked` 与 13 个 Rust tests 通过。`-TrayLifecycle` 临时 main 仓库 smoke 验证 descriptor v2、托盘认证观察、退出托盘不停止 daemon、受控 stop 和 explicit-stop 抑制自动重启；Windows 子进程 fixture 验证 stale creation identity 和无关进程都不会获得终止授权。协调器/托盘启动项 dry-run 通过且不包含凭据。 |

## Architecture notes

- 原计划中的 schema v17 已被 M4 的 schema 17-19 占用；M5 只能追加 schema 20，不重写任何已发布迁移。
- Python 监督状态、排空门和可恢复生命周期意图是 Rust 托盘的硬前置。托盘只调用版本化 loopback API，不读取 SQLite，也不直接终止未经二次身份核验的进程。
- 全量扫描允许与普通 Session/Lease/Cargo 写请求并发，但提交观察结果必须验证源 baseline epoch 与 HEAD；任何变化都会使观察失效并触发后续重扫。
- 自动启动入口携带 `-Automatic`，服务读取持久 `explicit_stop` 后失败关闭；用户手动 `start` 不携带该标记，因此可以明确解除停止意图。迁移或 SQLite 完整性失败只写无 secret 的 `startup-failure.json`，托盘据此进入 fatal 状态并禁止恢复循环。
