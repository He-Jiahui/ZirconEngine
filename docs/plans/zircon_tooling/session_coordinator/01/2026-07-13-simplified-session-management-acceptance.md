# Simplified Session Management Acceptance

## 状态与产出记录

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M6 | 简化 Session 管理可用化 | `completed` | 2026-07-13 | 用户明确将范围收敛为“简单管理 Session”，不再要求复杂工程门禁。生产 daemon 已滚动到 PID `32712` / instance `fec522a1f0fc4e2a9e13c0744221fffe`；网页、托盘、Hook 与持久日志均已启用。 |
| M6 | 本机控制中心固定端口与免令牌访问 | `completed` | 2026-07-13 | 服务改为固定监听 `127.0.0.1:65189`，本机 HTTP 控制请求不再校验 bearer token；`test_default_config_uses_stable_local_control_port`、`test_local_health_and_session_commands_accept_requests_without_token` 与 `test_direct_browser_session_query_accepts_no_bearer_or_cookie` 已通过。 |
| M6 | Cargo 实时基线与历史隔离 | `completed` | 2026-07-13 | 控制快照为每个磁盘现存 Cargo 目录仅投影最新记录；实时面板只使用该投影。历史删除/失败记录不再计入 `可复用池`、`用后即删`、`待清理`、`清理失败`，并由 `test_validation_lifecycle_summary_counts_only_existing_latest_targets` 覆盖。 |
| M6 | Failure Priority Gate 技能收敛 | `completed` | 2026-07-14 | `cross-session-coordination`、`handle-plan-failure-handoffs`、`continuous-milestone-execution` 与里程碑策略统一规定：修复计划发现适用 failure 后必须进入 `resolving_failure`，在 `failure return` 前不得启动普通切片；来源计划仅可继续无依赖切片。三份修改技能已通过 `quick_validate.py`。 |

## 最终可用范围

- 网页控制台可以查看 Session、Workflow、Failure、文件租约、Cargo 作业和审计状态。
- Windows 托盘保持一个仓库一个实例，显示协调器状态并提供打开控制台与受控生命周期入口。
- Codex Hook 自动把 ZirconEngine Session 同步到协调器，生产状态为 `healthy` 且没有失败运行。
- Session 中间状态由服务管理；文件租约和 Cargo 占用仍作为并行写入提示，不再追加复杂发布工程。
- 守护进程标准输出和错误输出保存在 LocalAppData 的仓库键目录，不进入 Git。

## 生产冒烟证据

- `/health`：HTTP 200。
- `/control/v1/snapshot`：HTTP 200、`272.854ms`、238 workflows、232 business Sessions、27 Codex Sessions、54 Failure nodes、500 条有界 Cargo 记录。
- `/ui/`：HTTP 200；当前不发送 `Content-Security-Policy`，保留 `X-Frame-Options: DENY` 与 Permissions Policy，并使用 `Referrer-Policy: same-origin`。本轮早期冒烟曾验证 CSP 与 `no-referrer`，后续状态由 [`2026-07-13-web-csp-disabled.md`](2026-07-13-web-csp-disabled.md) 取代。
- 托盘：唯一 PID `31360`；仓库根、仓库键、daemon PID 创建时间、可执行文件与命令行身份全部匹配。
- daemon 日志：`latest.json` 与本轮 stdout/stderr 日志已写入 LocalAppData；本轮 stderr 为 0 bytes。
- 简化前已完成的 2 小时隔离观察也自然结束：`passed`，7200.393 秒、240 样本、一次受控重启、两代实例、0 errors、workspace 已清理。该结果作为补充诊断证据，不再是用户要求的复杂门禁。

## 已知边界

共享 `main` 当前仍有其他 Session 的活动 Cargo 作业和文件租约，这是页面需要展示的正常协同状态，不代表 Session 管理服务故障。全仓计划/Failure 审计仍报告其他子计划既有问题；本子计划没有开放 Failure 交接。
