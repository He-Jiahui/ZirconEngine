# H1 Codex Session Projection

Plan: docs/plans/zircon_tooling/session_coordinator/02-codex-session-hook-sync.md
Milestone: H1
Status: completed
Files: ["docs/cli-and-tooling/workflow-control-center.md", "docs/plans/zircon_tooling/session_coordinator/02/2026-07-13-h1-codex-session-projection.md", "tools/session_coordinator/codex_sync/__init__.py", "tools/session_coordinator/codex_sync/discovery.py", "tools/session_coordinator/codex_sync/models.py", "tools/session_coordinator/codex_sync/store.py", "tools/session_coordinator/migrations.py", "tools/session_coordinator/tests/codex_rollout_fixture.py", "tools/session_coordinator/tests/test_codex_discovery.py", "tools/session_coordinator/tests/test_codex_store.py", "tools/session_coordinator/tests/test_database.py"]

## 状态与产出记录

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| H1 | H1.1–H1.3 类型与只读发现 | `completed` | 2026-07-13 | 新增封闭来源位置、Session 状态、生命周期事件和触发来源枚举；发现器只读取首条 `session_meta` 与 64 KiB 尾部，最多扫描 10,000 个 rollout。规范路径必须位于 Codex 根和 ZirconEngine 仓库内；跨盘、同名前缀、控制字符、可用时的符号链接逃逸均失败关闭。提示词、助手输出、指令、工具参数、附件、环境值、token 与 webhook 样式内容不会进入投影；可选元数据只接受简单安全字符。 |
| H1 | H1.4 schema v27 | `completed` | 2026-07-13 | `codex_sessions` 与 `codex_sync_runs` 具有严格状态/来源/事件/触发枚举约束、非负计数、来源/状态组合约束、精确业务 Session 外键和有界查询索引。注入 v27 DDL 失败后数据库保持完整 v26、无半迁移表；恢复后迁移和重复迁移均到 v27。 |
| H1 | H1.5 事务投影 | `completed` | 2026-07-13 | 事务 reconcile 只按 `sessions.session_id == thread_id` 精确绑定，不做标题、目标、计划或消息模糊匹配；不变来源不会刷新投影行或重复发出 `codex.session.*` 事件。完整成员扫描连续缺失两次才切换 `unavailable`，计数封顶为 2；不完整扫描不会删除存在性。 |
| H1 | H1.6 模块边界文档 | `completed` | 2026-07-13 | 操作文档 machine-readable header 纳入实现、计划与测试映射，并明确 Codex 来源存在性与业务 Session 权威分离；来源投影不会创建租约、Patch、Cargo、工作流或 Git 提交。 |
| H1 | H1-T 验证 | `completed` | 2026-07-13 | `python -m unittest -v tools.session_coordinator.tests.test_codex_discovery tools.session_coordinator.tests.test_codex_store tools.session_coordinator.tests.test_database`：22 项通过，1 项因当前 Windows 未授予目录符号链接权限而跳过；同名前缀、跨盘和规范路径 containment 仍通过。`python -m compileall -q tools/session_coordinator` 与 scoped `git diff --check` 通过。 |

## Architecture notes

- rollout 文件始终只读；SQLite 保存的 canonical path、大小和 mtime 只是来源修订身份，不保存 JSONL 原文或完整内容哈希。
- `Stop` 在后续 Hook 层只表示 turn boundary；持久 archive 仍由 active/archived 目录成员关系决定。
- `codex_sessions` 是可丢失、可重建的来源投影。业务 `sessions` 继续独占计划、写租约、延迟 Patch、验证、Failure、里程碑和提交权威。
- H2 只可向外部受管 spool 写经过 allowlist 缩减的触发，不可直接写 v27 数据库；H3 的单飞 worker 是唯一 reconcile 执行者。
