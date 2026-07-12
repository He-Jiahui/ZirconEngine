# H3 Codex Reconciliation Service

Plan: docs/plans/zircon_tooling/session_coordinator/02-codex-session-hook-sync.md
Milestone: H3
Status: completed
Files: ["docs/cli-and-tooling/workflow-control-center.md", "docs/plans/zircon_tooling/session_coordinator/02/2026-07-13-h3-codex-reconciliation-service.md", "tools/install-codex-session-hook.ps1", "tools/session_coordinator/codex_sync/discovery.py", "tools/session_coordinator/codex_sync/hook.py", "tools/session_coordinator/codex_sync/worker.py", "tools/session_coordinator/config.py", "tools/session_coordinator/control_plane/actions/catalog.py", "tools/session_coordinator/control_plane/actions/executor.py", "tools/session_coordinator/control_plane/actions/fingerprint.py", "tools/session_coordinator/control_plane/actions/models.py", "tools/session_coordinator/control_plane/router.py", "tools/session_coordinator/migrations.py", "tools/session_coordinator/server.py", "tools/session_coordinator/tests/test_action_catalog.py", "tools/session_coordinator/tests/test_action_execution.py", "tools/session_coordinator/tests/test_codex_discovery.py", "tools/session_coordinator/tests/test_codex_hook.py", "tools/session_coordinator/tests/test_codex_worker.py", "tools/session_coordinator/tests/test_control_http.py", "tools/session_coordinator/tests/test_database.py"]

## 状态与产出记录

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| H3 | H3.1–H3.2 单飞 worker | `completed` | 2026-07-13 | 单 daemon thread、wake event 与 monotonic deadline；startup full、30 秒 membership incremental、15 分钟 forced full。运行中 20 次 wake 合并为一个 follow-up，max active=1；停止等待 in-flight 完成并 join。只读状态不 reconcile/ack；失败只留 `codex_sync_failed`，后续 wake 可恢复。 |
| H3 | H3.3 服务生命周期 | `completed` | 2026-07-13 | 显式 state root 的测试服务使用隔离 Codex home/spool，生产默认使用 `$CODEX_HOME` 与 LocalAppData。schema/identity/supervision 初始化后构造 worker，daemon healthy/read-only 确定后启动；health 输出有界 worker 状态，关闭时先 stop/join worker 再关闭控制 HTTP、数据库与 runtime/lock。 |
| H3 | H3.4 authenticated wake | `completed` | 2026-07-13 | runtime bearer、exact loopback、≤4 KiB、repository key、schema 1 均为硬门禁；有效请求在 0.5 秒内返回 202 并异步增加 run count。无 runtime auth 得 401，仓库不匹配得 409。Hook 同时要求 runtime PID/creation time 与 lock PID 一致；分叉时网络调用次数为 0。 |
| H3 | H3.5 controlled reconcile | `completed` | 2026-07-13 | `codex.sessions.reconcile` 为 maintainer-only、service-scoped、参数严格 `{}` 的黄色动作；operator 越权、path/thread/raw payload 均拒绝。Preview 指纹包含 latest Codex run 与 session count；Confirm 只执行 `worker.wake("controlled")`，结果为 queued，不产生第二执行器。 |
| H3 | H3.6 schema/recovery | `completed` | 2026-07-13 | schema v28 原子扩展 action_kind 闭集；v27 历史 action、approval、supervision event、lifecycle intent 各一条迁移后全部保留，三个外键均重新指向 action_requests，不可变触发器和单 active lifecycle 唯一索引恢复；Codex action 可写、arbitrary action 仍拒绝。 |
| H3 | H3-T 验证 | `completed` | 2026-07-13 | 最终 10 模块聚焦组 84 项通过，1 项仅因当前 Windows 未授予目录 symlink 权限而跳过；包含计划 7 模块及 v28、Hook identity、增量缓存新增回归。`compileall` 与 scoped `git diff --check` 通过。无 worker thread 泄漏，spool 只在 reconcile 返回 committed run ID 后 ack。 |

## Architecture notes

- schema v27 仍只拥有 Codex 来源投影；v28 仅为新受控动作扩展既有 action audit 闭集，不回写已发布迁移。
- incremental 仍执行完整目录成员枚举，以可靠发现 archive/missing；只有 JSONL 首行/尾部解析按 revision 缓存。15 分钟 full 不信任 revision 缓存。
- Hook、HTTP、周期和 controlled action 都只设置同一 worker wake；任何调用者都不能直接执行 discovery/store 或选择来源路径。
- Worker ack 的 item 集合在本次 discovery 前捕获。运行期间新到的 Hook 文件不会被误删，wake event 会安排下一次读取。
