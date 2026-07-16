# 受管 Git 提交门禁

- 日期：2026-07-14
- 会话：`session-coordinator-milestone-control-20260714`
- 范围：协调器里程碑提交、仓库本地 Git Hook、受忽略的仓库技能/Hook。

## 已完成

1. 业务提交仅允许通过 `milestone prepare → validate → review → commit`。旧 `finalize --milestone` 和编号计划的通用完成状态均不能替代该流程。
2. 本地 `pre-commit` 与 `prepare-commit-msg` 双门禁拒绝普通 `git commit`；后者覆盖 `--no-verify`，且不再存在环境变量绕过口。Codex Hook 同时拦截 `-c core.hooksPath=... commit` 等尝试绕开本地 Hook 的写法，以及直接变更共享暂存区的 `git add`、`rm`、`mv`、`reset`、`restore --staged`。协调器以 Git plumbing 创建经过门禁验证的精确提交，不会读取或提交其他会话的暂存/工作区内容。
3. 归属写入本身也强制要求同一 Session 的有效租约；无法先领取再归属，不能将路径标记为可提交。里程碑提交随后对每个路径复核当前 Session、当前内容哈希、有效租约、清单完整性、计划输出、Failure、受控验证与独立评审。任何不满足项都会拒绝提交。
4. `.codex` 默认忽略不再阻止仓库技能和 Hook 入库：仅 `.codex/skills/**`、`.codex/hooks/**` 与 `.codex/hooks.json` 可在已经归属且租约有效时由协调器强制加入。会话笔记与 `.codex/state/**` 仍不能入库。
5. 成功里程碑提交后由服务一次性生成企业微信四行通知；Git subject 保持纯 Conventional Commit，模块标签只出现在通知首行。

## 验证

```text
python -m unittest tools.session_coordinator.tests.test_git_finalize tools.session_coordinator.tests.test_git_guard -v
29 tests: OK

python -m unittest tools.session_coordinator.tests.test_action_execution tools.session_coordinator.tests.test_workspace_copy -v
27 tests: OK
```

覆盖了精确提交不吞并外部脏文件、归属缺失拒绝、忽略技能可受管提交、会话笔记拒绝、普通提交与 `--no-verify` 拒绝、验证副本与异步动作恢复。

```text
python -m unittest tools.session_coordinator.tests.test_server.ServerTests.test_baseline_attribution_requires_the_session_live_lease -v
1 test: OK

python -m unittest tools.session_coordinator.tests.test_cargo_guard -v
6 tests: OK

python -m unittest tools.session_coordinator.tests.test_server.ServerTests.test_foreground_mutation_is_not_blocked_by_slow_workspace_observation tools.session_coordinator.tests.test_server.ServerTests.test_foreground_mutation_is_not_blocked_by_manual_workspace_scan tools.session_coordinator.tests.test_server.ServerTests.test_foreground_mutation_is_not_blocked_by_long_control_action -v
3 tests: OK
```

服务重载后，外部 Shader04 Session 的受管 `validate-matrix.ps1 -Package zircon_runtime -SkipBuild` 作业完整经过 `acquire → running → finish → release`；Cargo 测试自身以 `exit 1` 结束，但协调器持续健康，未再发生 `offline`。根因是手动 `watch scan` 在旧实现中占用前台 mutation mutex；该命令现为只读路径。

随后 Shader04 的全量 `validation_copy.materialize` 再次暴露同一架构问题：作业 `2d741fba03ce4f949c94469dc01bb3ee` 已持久化为 `planned`，但旧实现仍在前台 mutation mutex 内逐文件运行 `git show`；24,022 个跟踪文件使 HTTP 调用超过五分钟，连心跳与 Cargo 生命周期也被饿死。该已超时副本已由服务清理为 `removed`，未修改 Shader 业务文件。

修复后的物化操作先在 SQLite 中持久化 `materialization_started_at`，立刻返回 `materializing` 作业；后台工作线程在锁外用单个 `git archive <pinned-HEAD>` 流物化所有基线文件，再叠加当前 Session 的哈希归属覆盖层。`validation-copy status <job-id>` 提供可轮询状态；物化完成/失败只在短事务中提交，物化中拒绝 run、cancel、cleanup，重启恢复会移除中断的物化树。`validation_copy.materialize` 因此成为允许锁外启动后台作业的窄命令，仍保留只读分支与监督健康门禁。

排查期间还确认 `baseline scan` 在 HEAD 已变化时调用 `_commit_manifest`，旧实现会对每个跟踪路径启动 `git cat-file`。虽然后台 watcher 不持有前台锁，但显式 `baseline scan` 会持有它，且 24,022 个子进程会耗尽调度，使 Cargo `start` 长时间排队。该路径现改为单条流式 `git archive <pinned-HEAD>`：逐个 tar 成员计算相同 SHA-256 基线哈希，不将整个归档读入内存，并保留 Git 的工作树过滤结果；`baseline.scan` 同时加入锁外准备命令集，仍由 epoch/HEAD 比对和 SQLite 短事务拒绝陈旧观察写入。

对客户端断开也补齐了语义：请求达到期限现在返回 `command_timeout`（包含原 command、deadline 和恢复建议），不再把可用的 coordinator 写成 `offline`；HTTP 回写遭遇断开的 socket 只结束响应，不触发第二次写入或 traceback。并发回归人为让 `baseline.scan` 在 50ms 后断开，同时验证 `cargo.acquire`、`cargo.finish`、`baseline.attribute` 与 `session.heartbeat` 均在 0.75 秒内完成。`session heartbeat`、lease claim/release 与完整 Cargo lifecycle 均加入短事务直通集；全局 RLock 只保留给受控 force-stop 确认这类极短生命周期交接。各长操作依赖其已有的 SQLite 事务、cleanup reservation、Git mutex 或专属后台 worker，而不再占用共享前台写通道。

```text
python -m unittest tools.session_coordinator.tests.test_server.ServerTests.test_disconnected_baseline_scan_does_not_block_finish_or_attribution tools.session_coordinator.tests.test_server.ServerTests.test_foreground_mutation_is_not_blocked_by_baseline_scan tools.session_coordinator.tests.test_server.ServerTests.test_foreground_mutation_is_not_blocked_by_validation_copy_materialize tools.session_coordinator.tests.test_baselines.BaselineTests.test_head_refresh_uses_one_archive_instead_of_cat_file_per_tracked_path tools.session_coordinator.tests.test_baselines.BaselineTests.test_archive_manifest_preserves_git_worktree_filters tools.session_coordinator.tests.test_workspace_copy.WorkspaceCopyTests.test_materialize_uses_a_single_baseline_archive_for_large_manifests tools.session_coordinator.tests.test_workspace_copy.WorkspaceCopyTests.test_async_materialize_returns_before_copy_finishes_and_exposes_status tools.session_coordinator.tests.test_control_snapshot.ControlSnapshotTests.test_snapshot_omits_heavy_internal_manifests_and_patch_objects
8 tests: OK
```

在线复验（本机端口 `127.0.0.1:65189`）：重载后的协调器 Schema 为 29。另一条受管 Cargo 作业运行时，对当前协调器 Session 请求 `validation-copy materialize --path Cargo.toml` 在 1.15 秒内返回 `materializing`，轮询到 `materialized` 后成功 cleanup；未阻塞 Cargo。此前对不存在的 `README.md` 路径的失败副本按记录清理为 `removed`，没有修改任何 Shader 业务文件。Shader04 的受管 Cargo 验证此前亦已完整执行 `acquire → start → finish → release`；其 `exit 1` 为测试结果，而不是协调器离线或生命周期失败。

## 当前工作区审计

- 快照（2026-07-14 10:32 CST）：210 个脏路径，`git diff --cached` 确认共享暂存区为 0；其中 22 个为未跟踪路径。131 个可由当前哈希准确映射到 10 个 Session，62 个归属已过期，17 个尚未归属。
- 未跟踪和已归属路径均保持不动，防止覆盖其他会话的意图；其所属执行会话必须通过协调器完成范围核验与里程碑提交。此后 Codex Session 不能再直接修改共享 index。
- 未归属路径不允许被任何 Session 的里程碑提交吸收；其执行会话恢复后必须先重新领取租约并写入当前哈希归属，再走完整里程碑门禁。
