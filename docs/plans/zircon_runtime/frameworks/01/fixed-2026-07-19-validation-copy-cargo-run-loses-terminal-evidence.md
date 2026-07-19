---
handoff_kind: fixed
status: fixed
created_at: 2026-07-19
summary_slug: validation-copy-cargo-run-loses-terminal-evidence
origin_plan: docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/workspace_copy.py
  - tools/session_coordinator/workspace_copy_terminal.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/workflows/milestones.py
  - tools/session_coordinator/tests/test_workspace_copy.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_workspace_copy
  - validation-copy run with a materialized full workspace and a nontrivial Cargo command
resolved_at: 2026-07-19
---


# Coordinator01：validation-copy Cargo run 丢失终态证据

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md`
- 来源执行者：`frameworks01-runtime11-error-task-atomic-prerequisite-20260719`
- 来源执行切片：Frameworks01 error owner 与 Runtime11 task diagnostics 的 44 路径原子前置门禁
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：不可变副本已正确物化，但非平凡 Cargo run 在进程终端边界返回内部异常、删除副本且没有持久化 `validation_copy_runs`，业务 owner 无法获得可审计验收结论。

## 失败现象与复现证据

两个完整工作区副本均固定 HEAD `e2a019278cc9f001d050b4fba16039c6a2494bfb`，manifest 24,598 路径，并只叠加当前 Session 归属的 44 路径：

- copy `c657a24304b0439b8a979da1851a08b2`：成功物化后运行 `cargo +1.94.1 test -p zircon_runtime --lib tasks --locked --jobs 1 --color never -- --nocapture --test-threads=1`，约 43 秒后命令返回 `invalid_request: 'NoneType' object is not subscriptable`；copy 变为 `removed`，对应 `validation_copy_runs` 为 0 行。
- copy `edeea5ac6d434592b533f0eac9a04c4e`：在 snapshot `566`、rustfmt/diff-check GREEN 后重新物化同一 24,598 路径并执行同一命令，约 38 秒后得到相同内部错误；copy 变为 `removed`，对应 run 仍为 0 行。

对照探针证明基础入口与可执行解析正常：

- copy `c1419eee2be94643bd9ed9a435cc4aaf` 运行 `python -c "print('validation-copy-ok')"`，run `7638daf6d154415f974a810ebc3e2dee`，exit 0，stdout 正常入账。
- copy `11b105a97d544fc1a9cb1283d46c7b91` 运行 `cargo +1.94.1 --version`，run `fd93be8872294daebeb79d6c5da76fc8`，exit 0，`cargo 1.94.1` 正常入账。

因此当前证据把问题收敛到 full-copy 非平凡 Cargo 子进程的终态持久化/返回/cleanup 边界，而不是 Cargo 不可用、物化失败或 Frameworks01/Runtime11 测试失败。两次业务 run 都没有 durable run row，禁止解释为 GREEN 或 RED。

## 最低共享层根因

Coordinator01 的 `validation_copy.run` 必须先持久化完整 run 终态，再导入 workflow binding，最后清理副本。2026-07-19 的只读 DB/控制流复核已把本次空值从“查询结果”继续下沉：`c657...` / `edee...` 在 `workflow_validation_bindings` 均为 0 行，故 `import_validation_result()` 的无 binding no-op 不是本次异常来源；copy 已进入 terminal cleanup，而 `validation_copy_runs` 仍为 0 行，说明异常发生在 `process.communicate()` 返回之后、run INSERT 之前。该区间唯一执行下标的生产语句是 `workspace_copy.py` 的 `stdout_full[-65536:]` / `stderr_full[-65536:]`，与 `'NoneType' object is not subscriptable` 精确吻合。当前实现假定 PIPE communicate 两个返回值永远非 `None`，且 `process_finished = True` 已先置位，所以任一流为空值都会跳过证据 INSERT、仍执行 cleanup，导致 exit code、另一路有效输出与副本一起丢失。

修复不能只做 `or ""` 止 panic：必须把“进程终态采集”“durable evidence INSERT”“workflow import”“副本 cleanup”分为单向阶段，保证采集异常也留下 typed failure evidence 或保留 copy；workflow hook 失败也不得反向污染已持久化的进程终态。

## 架构修复验收

- 新增 focused 回归：materialized full-input copy 运行可控的非平凡子进程时，无论 exit 0、非零、completion hook 无 binding、binding 查询为空或 hook 抛错，`validation_copy_runs` 必须先持久化且 API 返回 typed evidence/error。
- focused RED 必须直接覆盖 `communicate() -> (None, None)`、仅 stdout 为 `None`、仅 stderr 为 `None` 三种终态，断言空流规范化为空文本、实际 `returncode` 仍持久化；另加一个真实非零子进程（等价 Cargo exit 101）证明 stderr 与 exit code 不丢失。
- terminal cleanup 只能发生在 run row durable 之后；若证据持久化失败，copy 必须保留为可审计的 `materialized`/`failed`/`cleanup_pending` 状态，不得删除唯一 stdout/stderr 载体。
- workflow completion hook 的无 binding 分支必须是显式 no-op；hook 失败不得回滚已经完成的 validation run，也不得把内部 `NoneType` 暴露为无上下文 `invalid_request`。
- 修复后用 snapshot `566` 等价的 immutable manifest 重跑 Runtime11 focused gate，返回 exact copy/run ID、exit code、stdout/stderr 和 cleanup 终态，再由业务 owner继续 review/fixed return/commit。

## 禁止临时方案

- 不得改用共享工作树裸 Cargo、手工复制目录、人工补写 SQLite run row 或把命令输出粘贴成伪 managed evidence。
- 不得复用已删除的 `c657...` / `edee...` copy，也不得把两个 `NoneType` 解释为业务编译失败。
- 不得通过关闭 immutable-copy cleanup、workflow import 或 artifact governance 来规避空值路径。

## 修复结果与回传

- 根因：Validation-copy assumed subprocess communicate stdout/stderr were always text and sliced them before inserting validation_copy_runs. A None stream raised after the process became terminal, so cleanup removed the immutable copy without durable exit/output evidence.
- 架构修复：ValidationCopyTerminalLifecycle now normalizes nullable streams, persists actual exit/output before workflow completion, atomically finalizes only after the hook succeeds, preserves failed evidence/copies, and coordinates local run reservation with PID registration, recovery snapshots, restart handling, and cleanup.
- 验证：Snapshot586 exact5; test_workspace_copy 32/32; Server/Workflow cross gates 2/2; py_compile and diff-check GREEN; independent review C0/I0/M0; successor b07627e schema49; production copy 93b7be8b manifest24597 at HEAD d9bae9d; inner run b56ab61 exit101 with durable empty streams; outer job 3677f65 released exit0/live PIDs empty; copy removed only after evidence.
- 回传：Coordinator01 terminal-evidence loss is fixed and returned. Runtime11 remains an auditable business RED (exit 101), so Frameworks01 must run a fresh post-commit gate; no Runtime11 GREEN is claimed.
