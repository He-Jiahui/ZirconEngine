# Simplified Session Management Acceptance

## 状态与产出记录

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M6 | 简化 Session 管理可用化 | `completed` | 2026-07-13 | 用户明确将范围收敛为“简单管理 Session”，不再要求复杂工程门禁。生产 daemon 已滚动到 PID `32712` / instance `fec522a1f0fc4e2a9e13c0744221fffe`；网页、托盘、Hook 与持久日志均已启用。 |

## 最终可用范围

- 网页控制台可以查看 Session、Workflow、Failure、文件租约、Cargo 作业和审计状态。
- Windows 托盘保持一个仓库一个实例，显示协调器状态并提供打开控制台与受控生命周期入口。
- Codex Hook 自动把 ZirconEngine Session 同步到协调器，生产状态为 `healthy` 且没有失败运行。
- Session 中间状态由服务管理；文件租约和 Cargo 占用仍作为并行写入提示，不再追加复杂发布工程。
- 守护进程标准输出和错误输出保存在 LocalAppData 的仓库键目录，不进入 Git。

## 生产冒烟证据

- `/health`：HTTP 200。
- `/control/v1/snapshot`：HTTP 200、`272.854ms`、238 workflows、232 business Sessions、27 Codex Sessions、54 Failure nodes、500 条有界 Cargo 记录。
- `/ui/`：HTTP 200；CSP、`X-Frame-Options: DENY`、`Referrer-Policy: no-referrer`、Permissions Policy 均已加载。
- 托盘：唯一 PID `31360`；仓库根、仓库键、daemon PID 创建时间、可执行文件与命令行身份全部匹配。
- daemon 日志：`latest.json` 与本轮 stdout/stderr 日志已写入 LocalAppData；本轮 stderr 为 0 bytes。
- 简化前已完成的 2 小时隔离观察也自然结束：`passed`，7200.393 秒、240 样本、一次受控重启、两代实例、0 errors、workspace 已清理。该结果作为补充诊断证据，不再是用户要求的复杂门禁。

## 已知边界

共享 `main` 当前仍有其他 Session 的活动 Cargo 作业和文件租约，这是页面需要展示的正常协同状态，不代表 Session 管理服务故障。全仓计划/Failure 审计仍报告其他子计划既有问题；本子计划没有开放 Failure 交接。
