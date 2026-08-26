---
handoff_kind: failure
status: open
created_at: 2026-08-26
summary_slug: validation-copy-baseline-resource-live-worktree-drift
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/validation_copies.py
  - tools/session_coordinator/tests/test_validation_copies.py
tests:
  - "python -B -m unittest tools.session_coordinator.tests.test_validation_copies -v"
---

# Session Coordinator 01: validation copy baseline resource follows live worktree

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：Coordinator01 unblocked failure-chain cleanup and managed validation-copy closure planning
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：失败发生在 Cargo 启动前的 validation-copy 输入闭包规划，最低共享原因属于 Coordinator01，不属于触发该闭包的 Runtime 或 Editor 产品路径。

## 失败现象与复现证据

两个独立的 Editor validation ticket 在 `closure_planning` 阶段以
`validation_copy_compile_time_resource_missing` 终止。精确资源
`zircon_runtime_interface/src/runtime_api/host_requests.rs` 已存在于 Git
index/HEAD，却仅在共享 live worktree 中被其他 owner 删除；materializer 随后本应从
固定 baseline tree 恢复该文件，因此本次拒绝不是 immutable copy 的真实缺失。

RED 回归进一步证明相反边界也不成立：当 `include_str!("schema.txt")` 指向一个
只存在于 live worktree、未被 Git 跟踪的文件时，planner 的 `Path.exists()` 放行该
路径，而后续 `git ls-files` 返回空集，闭包静默遗漏资源而没有 fail-closed。

## 最低共享层根因

`CargoInputClosurePlanner` 混用了两个身份域：先用 live filesystem 的
`Path.exists()` 判断 compile-time resource，再用 Git tracked paths 构造将被固定
baseline materializer 消费的闭包。外部 worktree 删除会错误否定 HEAD 输入；live
未跟踪文件则会错误通过第一道检查并在 tracked 枚举中静默消失。

## 架构修复验收

- compile-time resource 的可用性必须由 immutable tracked input domain 判定，不得依赖共享 live worktree 是否存在。
- HEAD/index 已跟踪但 live worktree 被删除的精确文件或目录必须仍进入 closure。
- 仅在 live worktree 存在、未进入 tracked input domain 的资源必须以原有 actionable code fail-closed，并持久保留 `sourcePath` / `resourcePath`。
- 保持 Windows Git pathspec 有界批处理、package selection 与 repository escape 检查不变。
- 不修改或恢复任何 Runtime、Editor 或其他 owner 的产品文件。

## 禁止临时方案

- 不得恢复共享 worktree 中的外部删除来掩盖 planner 身份错误。
- 不得忽略 missing resource、扩大为全仓复制或放宽 unmanaged/ownership 边界。
- 不得运行 raw Cargo；本修复以 Coordinator Python 回归和受管 finalizer 验证收口。

## 修复结果与回传

待完成。
