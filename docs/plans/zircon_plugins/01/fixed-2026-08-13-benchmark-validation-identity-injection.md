---
handoff_kind: fixed
status: fixed
created_at: 2026-08-11
summary_slug: benchmark-validation-identity-injection
origin_plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
origin_workflow_node: M1
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_plugins/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - docs/cli-and-tooling/local-session-coordinator.md
  - tools/session_coordinator/benchmark_validation_grants.py
  - tools/session_coordinator/benchmark_validation_schema.py
  - tools/session_coordinator/cli.py
  - tools/session_coordinator/control_plane/actions/catalog.py
  - tools/session_coordinator/control_plane/actions/models.py
  - tools/session_coordinator/control_plane/actions/executor.py
  - tools/session_coordinator/control_plane/actions/fingerprint.py
  - tools/session_coordinator/migrations.py
  - tools/session_coordinator/processes.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/tests/test_action_catalog.py
  - tools/session_coordinator/tests/test_action_execution.py
  - tools/session_coordinator/tests/test_action_fingerprint.py
  - tools/session_coordinator/tests/test_benchmark_validation_grants.py
  - tools/session_coordinator/tests/test_database.py
  - tools/session_coordinator/tests/test_milestone_cli.py
  - tools/session_coordinator/tests/test_processes.py
  - tools/session_coordinator/tests/test_windows_job_process.py
  - tools/session_coordinator/tests/test_workflow_commit.py
  - tools/session_coordinator/tests/test_workspace_copy.py
  - tools/session_coordinator/windows_job_process.py
  - tools/session_coordinator/workspace_copy.py
  - tools/session_coordinator/workspace_copy_terminal.py
  - tools/session_coordinator/workflows/milestones.py
tests:
  - python -X dev -W error::ResourceWarning -m unittest tools.session_coordinator.tests.test_processes tools.session_coordinator.tests.test_windows_job_process tools.session_coordinator.tests.test_benchmark_validation_grants tools.session_coordinator.tests.test_workspace_copy -v
  - python -X dev -W error::ResourceWarning -m unittest tools.session_coordinator.tests.test_action_catalog tools.session_coordinator.tests.test_action_execution tools.session_coordinator.tests.test_action_fingerprint -v
  - python -X dev -W error::ResourceWarning -m unittest tools.session_coordinator.tests.test_database tools.session_coordinator.tests.test_milestone_cli tools.session_coordinator.tests.test_workflow_commit -v
resolved_at: 2026-08-13
---


# Coordinator01: benchmark validation identity injection

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/01-plugin-architecture-core.md`
- 来源执行切片：M1 native plugin structural performance benchmark acceptance
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：受管 validation-copy 的 immutable input manifest、子进程环境与 PID/run identity 由 Coordinator01 独占；Plugins01 不得手工注入环境或直接消费 foreign materialized copy。

## 失败现象与复现证据

Plugins01 的四组 ignored native benchmark runner 均在计时前调用
`BenchmarkRunMetadata::from_environment`，要求 `ZR_BENCHMARK_SOURCE_MANIFEST` 为 64 位
coordinator manifest hash，且 `ZR_BENCHMARK_CARGO_PROFILE` 为 `release` 或 `profiling`。
RED 证明覆盖 typed 参数、命令、materialized identity、CLI、child environment、双清单绑定
与既有副本授权。实现前分别得到 benchmark enum/template 不存在、grant service/schema 不存在、
`bind_validation` 不接受完整 copy identity、`WorkspaceCopyService.run` 泄漏父进程 benchmark
变量，以及 executor 仍调用 `materialize_cargo` 等预期失败。

## 最低共享层根因

现有受管模板仅有 coordinator Python、web check 与 Runtime14 focused Rust，均不会在
materialization 完成后注入这两个值；普通 managed ignored benchmark 因缺失 metadata
确定性失败，手工环境又无法绑定 immutable copy。

## 架构修复验收

- 增加 allow-listed `native-plugin-benchmark` validation template；调用方必须声明一个命名
  ignored benchmark case 和 `release|profiling` profile，不能传任意 Cargo filter 或环境。
- 命令每个进程只运行一个命名 case，启用 `--ignored`、`--nocapture` 和单测试线程；release
  与 profiling 使用各自唯一 Cargo profile flag。
- 新增 maintainer-only `validation.benchmark_grant.issue`。调用方只能提交 source/target Session、
  workflow milestone、命名 case 与 profile，不能提交 `jobId` 或 grant ID；Coordinator 从同一
  numbered Plan 的 source Session 中唯一选择既有 `materialized` copy。
- grant 持久绑定 source/target Session、job、完整 `inputManifestHash`、milestone-scoped hash、
  server-generated command、case/profile 和 FIFO sequence。启动只自动消费目标 Session FIFO 头，
  issued -> launching -> consumed/denied 单向转换，禁止 replay 与跨 Session 使用。
- milestone stale gate 继续只比较 milestone-scoped hash；完整 Cargo closure 的 immutable
  `inputManifestHash` 独立写入 validation binding 与 grant，二者不再混用或弱化比较。
- 两个 benchmark 环境值只从已取得并复核的 grant 构造；`WorkspaceCopyService.start` 在启动事务
  中再次比对 copy owner/status/hash、target Session、server command、profile 和 exact-key env。
  缺失、畸形、mismatch、ungranted 或 foreign copy 均在 `Popen` 前拒绝。
- 环境只注入 benchmark child；普通 validation 即使父进程残留同名变量也必须移除，不能跨 run
  泄漏。同步 `run()` 与异步 `start()` 共用同一个净化环境构造器。
- 进程注册事务把 root PID、validation run ID 同时写入 grant 和 workflow binding，再放行后台
  terminal collector；terminal gate payload 提供 scoped/full manifests、profile、case、grant 与
  root PID，供 ETW 以受管 process tree 归因。
- Windows benchmark root 在执行第一条指令前已原子加入 non-inheritable kill-on-close Job；PID 与
  creation time 先持久化再 resume。terminal collector 在 root exit 后终止并等待整个 Job tree，
  然后才汇入 stdout/stderr、写入证据与释放 copy，不会因中间 parent 退出而遗漏 grandchild。
- 启动恢复拒绝无注册身份的 `launching` grant；对无 terminal evidence 的 `consumed` grant，先按
  persisted process identity 确认 Job 已终止，再拒绝 workflow validation。collector/evidence 失败
  形成的 `failed` + no-run copy 可重复恢复且保持内容不变，不会永久卡住 Coordinator 启动。
- active benchmark 的 cancel 权限绑定 target Session；source copy owner 无权终止目标 Session 的
  benchmark。所有 Job、pipe 与 retained process handle 在成功、拒绝和异常路径均幂等释放。
- denied launch 不改变 copy；成功的既有-copy benchmark 也保留 source tree，不走普通临时副本
  cleanup。已有 copy 只能在 Coordinator grant 全部复核通过后消费一次。

## 禁止临时方案

- 禁止修改 Plugins01 harness 来弱化 metadata 校验，禁止默认 development/debug profile。
- 禁止由 CLI 接收任意环境键、任意 Cargo test filter 或 shared-worktree hash。
- 禁止由调用方直接指定、重建、重试、清理或无授权复用
  `5945e3ef29d74bd69602adca02e243b5`；只有 Coordinator 发行且复核通过的一次性 grant 可启动。

## 修复结果与回传

- 根因：Managed validation templates never derived benchmark metadata from a durable full-copy identity, while scoped milestone hashes, existing-copy authority, FIFO ordering, child environment isolation, and process-tree ownership were not bound in one Coordinator transaction.
- 架构修复：Added a typed native-plugin benchmark template and one-shot same-plan cargo-copy grant, kept scoped and full manifests separate, sanitized child environments, atomically registered PID/creation identity, and used a kill-on-close Windows Job with crash-safe collector and startup reconciliation.
- 验证：Independent review reports zero Critical/Important; 80 low-level Job/grant/workspace tests, 71 schema/milestone/workflow tests, action suites, compileall, diff-check, and real CoordinatorApplication startup/restart tests passed.
- 回传：Plugins01 M1 may resume only through a Coordinator-issued FIFO grant for an eligible immutable cargo copy; stale copy 5945e3ef29d74bd69602adca02e243b5 remains untouched and unauthorized.
