# Windows Tray Path Identity Fix and Production Activation

Plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
Milestone: M6
Status: completed
Files: ["docs/cli-and-tooling/workflow-control-center.md", "docs/plans/zircon_tooling/session_coordinator/01/2026-07-13-tray-windows-path-identity-fix.md", "tools/session_tray/src/repository_identity.rs", "tools/session_tray/src/runtime_descriptor.rs"]

## 状态与产出记录

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M6 | Windows 跨进程仓库身份修复 | `completed` | 2026-07-13 | 真实托盘诊断显示 Rust `canonicalize()` 产生 `\\?\E:\Git\ZirconEngine`，哈希短键为错误的 `5030B1D88D`，而 daemon/runtime 使用 `E:\Git\ZirconEngine` 与 `76B56A19E1`。身份算法在哈希前只移除 Windows verbatim namespace，UNC 形式映射回标准 UNC；runtime 路径比较复用同一规范化函数，不放宽仓库/PID/创建时间/可执行文件/命令行/instance/authenticated-health 门禁。 |
| M6 | RED/GREEN 与生产替换 | `completed` | 2026-07-13 | 新回归首先因 `normalize_identity_path` 不存在而编译失败；实现后聚焦 Windows 测试 1/1 通过，最终 `cargo fmt --check` 与完整 Tray 31/31 测试通过。协调器管理的 D: test pool 和复用 E: release pool 内完成测试与 release build；托盘重新注册当前用户启动项并以新二进制启动。 |
| M6 | 真实服务启用 | `completed` | 2026-07-13 | 实际 daemon 从 schema v24 切换为 v28；Hook Query 为 configured/featureEnabled/daemonCompatible=true，pending=0、quarantine=0，wake 返回 202。真实 snapshot 返回 `codexSessions`，投影 27 个来源 Session；`/ui/` 返回 200 和生产 root，动作 catalog 返回 19 个闭集动作，包含 `service.stop` 与 `codex.sessions.reconcile`。托盘与 daemon 共同使用 repository key `76b56a19…`。 |

## Architecture notes

- 修复的是等价 Windows 路径的文本表示，不是兼容别名；不同仓库、盘符或 UNC owner 仍产生不同身份。
- 托盘仍要求 descriptor、process inspection 与 bearer-authenticated health 三方一致，任何一层不一致继续显示身份不匹配并禁用生命周期操作。
- 生产 Web、Hook 和托盘已即时启用；长时稳定性观察继续写入 Git 外 LocalAppData，不作为本修复提交内容。
