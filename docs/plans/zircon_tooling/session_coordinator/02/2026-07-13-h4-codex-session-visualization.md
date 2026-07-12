# H4 Codex Session Visualization

Plan: docs/plans/zircon_tooling/session_coordinator/02-codex-session-hook-sync.md
Milestone: H4
Status: completed
Files: ["docs/cli-and-tooling/workflow-control-center.md", "docs/plans/zircon_tooling/session_coordinator/02/2026-07-13-h4-codex-session-visualization.md", "tools/session_coordinator/control_plane/snapshot.py", "tools/session_coordinator/server.py", "tools/session_coordinator/run-control-validation.ps1", "tools/session_coordinator/tests/load_fixture.py", "tools/session_coordinator/tests/test_control_load.py", "tools/session_coordinator/tests/test_control_snapshot.py", "tools/session_coordinator/web/src/App.tsx", "tools/session_coordinator/web/src/api/contracts.ts", "tools/session_coordinator/web/src/api/validation.ts", "tools/session_coordinator/web/src/pages/SessionsPage.tsx", "tools/session_coordinator/web/src/__tests__/components.test.tsx", "tools/session_coordinator/web/src/__tests__/contracts.test.ts", "tools/session_coordinator/web/dist"]

## 状态与产出记录

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| H4 | H4.1–H4.3 投影与严格契约 | `completed` | 2026-07-13 | Snapshot 最多输出 1,000 行，按 active/idle/archive/unavailable 与活动时间排序；输出总数、截断、状态/来源统计、队列深度和最近终态。浏览器逐层验证精确字段、闭集枚举、长度与集合上限；旧 daemon 缺少 `codexSessions` 时只补空投影。 |
| H4 | H4.4–H4.5 Session 页面 | `completed` | 2026-07-13 | 业务 Session 与 Codex 来源 Session 分为两个语义独立面板。Codex 行只展示短 thread ID（完整值仅 title）、文字状态/来源、生命周期时间、安全 origin/CLI、精确绑定与诊断代码；恶意 HTML 诊断只作为文本。 |
| H4 | H4.6 有界负载与日志 | `completed` | 2026-07-13 | 日常负载改为 Quick：40 Sessions、20 workflows、500 nodes、5,000 events、500 artifacts、16 MiB sparse log；原 release 规模保留为显式 `ZIRCON_CONTROL_LOAD_PROFILE=release`。统一脚本把成功/失败完整 transcript 写到 LocalAppData，不把运行日志或隐私材料写入 Git。 |
| H4 | H4-T 验证 | `completed` | 2026-07-13 | `run-control-validation.ps1 -Profile Quick -Suite H4`：17 项 Python 投影/事件/负载测试与 41 项 Web 测试全部通过；TypeScript、Vite production build、27 个 hashed asset dist 审计通过。最终总耗时 56.5 秒；日志位于外部 validation 目录。 |

## Architecture notes

- Browser projection intentionally omits cwd, rollout path and source revision; Web cannot turn Codex source presence into business authority.
- Quick and Release exercise the same behavior assertions. Release is retained for intentional capacity work, not run automatically during every shared-main iteration.
- Validation logs live outside the repository and include exact command output and failure stack so reduced routine scale does not reduce diagnosability.
