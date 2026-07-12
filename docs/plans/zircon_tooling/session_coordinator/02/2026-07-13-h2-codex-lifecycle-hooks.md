# H2 Codex Lifecycle Hooks

Plan: docs/plans/zircon_tooling/session_coordinator/02-codex-session-hook-sync.md
Milestone: H2
Status: completed
Files: [".codex/config.toml", ".codex/hooks.json", ".codex/hooks/zircon_session_sync.py", "docs/cli-and-tooling/workflow-control-center.md", "docs/plans/zircon_tooling/session_coordinator/02/2026-07-13-h2-codex-lifecycle-hooks.md", "tools/install-codex-session-hook.ps1", "tools/session_coordinator/codex_sync/__init__.py", "tools/session_coordinator/codex_sync/hook.py", "tools/session_coordinator/codex_sync/spool.py", "tools/session_coordinator/tests/test_codex_hook.py", "tools/session_coordinator/tests/test_codex_spool.py", "tools/tests/codex-session-hook.Tests.ps1"]

## 状态与产出记录

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| H2 | H2.1 Hook 失败优先测试 | `completed` | 2026-07-13 | 覆盖空/畸形/64 KiB 超限 stdin、事件不匹配、SessionStart 来源、Prompt/assistant/tool/transcript secret、subagent 字段、外部与同名前缀 cwd、在线/陈旧/250 ms 慢 daemon、真实入口与逐事件 stdout。慢服务请求在 0.5 秒门禁内返回，陈旧 PID creation identity 在网络前失败关闭。 |
| H2 | H2.2 外部脱敏 spool | `completed` | 2026-07-13 | `CodexTriggerSpool` 在 LocalAppData 仓库 SHA-256 key 下以 temp+fsync+replace 原子写入；单项 ≤4 KiB、pending ≤1,024。外部读取严格校验 schema、仓库 key、事件/来源/turn/subagent/permission 组合、绝对 cwd 与带时区 ISO 时间；单项损坏隔离，不阻塞有效项；确认删除必须携带已提交 reconcile run ID 且路径必须是直属 pending JSON。 |
| H2 | H2.3–H2.4 正式生命周期 Hook | `completed` | 2026-07-13 | `.codex/hooks.json` 声明 SessionStart/UserPromptSubmit/Stop/SubagentStart/SubagentStop command handler，Windows 通过 Git 根和 `py -3` 解析入口，timeout 5 秒且无 unsupported async/prompt/agent handler。Hook 先耐久写 spool，再用运行时 token、仓库 key、PID creation time、schema/API 与 exact loopback 做 250 ms wake；从不启动 daemon。`Stop` 在输入或内部导入失败时仍输出合法 continuation JSON，其他事件静默。 |
| H2 | H2.5–H2.6 安装与 feature | `completed` | 2026-07-13 | `Query/Install/Update/Remove/DryRun` 保留不相关 TOML key、table、行内/独立注释；重复 Update 字节稳定。Remove 只删除 exact managed hooks、owned `features.hooks` 行和经 base/key 边界校验的仓库 spool；变更过的 hooks.json 拒绝删除。项目启用 canonical `[features].hooks = true`，未读取、写入或绕过信任库。 |
| H2 | H2.7 文档与信任 | `completed` | 2026-07-13 | 操作文档列出 Query/Install/Update/Remove 命令、隐私边界、离线恢复与 `/hooks` 人工审阅；明确项目 Hook 与全局 notify、用户/managed/plugin Hook 并行且互不覆盖。当前 Query：configured=true、featureEnabled=true、reviewRequired=true、daemonCompatible=false（旧 daemon 预期排队）。 |
| H2 | H2-T 验证 | `completed` | 2026-07-13 | 计划命令通过：16 项 Python Hook/spool/runtime identity 测试；`tools/tests/codex-session-hook.Tests.ps1` 实际安装生命周期与真实 Stop 入口通过；hooks JSON 解析、`compileall`、scoped `git diff --check` 通过。fixture secret 未进入 spool，临时 LocalAppData spool 全部清理。 |

## Architecture notes

- Codex 信任是不可绕过的外部门禁。定义 hash 变化后必须由操作者在 `/hooks` 审阅；周期 reconcile 会补偿审阅前错过的事件。
- Hook 不是业务协调器，也不是数据库 writer。它只产生脱敏 wake intent；H3 单飞 worker 将在事务 reconcile 成功后确认队列。
- `daemonCompatible=false` 不会阻塞 Codex 或丢弃触发，只禁止网络 wake。旧/慢/离线/歧义 daemon 都由 spool 加周期扫描恢复。
- 项目 hooks.json 是本安装器唯一管理的 Hook source；用户目录、requirements managed Hooks、plugin Hooks 和现有 global notify 永不纳入其读写范围。
