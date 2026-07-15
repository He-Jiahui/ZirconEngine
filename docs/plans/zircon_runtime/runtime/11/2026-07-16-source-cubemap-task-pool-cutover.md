---
related_code:
  - zircon_runtime/src/core/framework/tasks/parallel_slice_executor.rs
  - zircon_runtime/src/core/framework/tasks/mod.rs
  - zircon_runtime/src/core/runtime/tasks/parallel_for.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap/mipmap.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap/tests.rs
  - tools/tests/test_runtime_job_system_audit.py
implementation_files:
  - zircon_runtime/src/core/framework/tasks/parallel_slice_executor.rs
  - zircon_runtime/src/core/framework/tasks/mod.rs
  - zircon_runtime/src/core/runtime/tasks/parallel_for.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap/mipmap.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap/tests.rs
  - tools/tests/test_runtime_job_system_audit.py
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Async/ParallelFor.h
  - dev/bevy/crates/bevy_tasks/src/slice.rs
tests:
  - python -m unittest tools.tests.test_runtime_job_system_audit -v
  - direct job_system_boundary_audit: expected_module_count = 9, direct_rayon_paths = expected_direct_rayon_paths = 2, unexpected_rayon_paths = [], unclassified_direct_rayon = [], risks = []
  - managed job a6415d1032364e1aabd63687227035aa: cargo build -p zircon_runtime --locked
  - rustfmt --edition 2021 on the six changed Rust files
  - python .codex/skills/zircon-project-skills/write-plan-output-records/scripts/audit_plan_output_records.py --repo-root E:\Git\ZirconEngine
  - scoped git diff --check
doc_type: milestone-detail
status_anchor: runtime_11_source_cubemap_task_pool_cutover_static_and_managed_build_passed_focused_test_pending
---

# Runtime 11 Source Cubemap Task-Pool Cutover

## 状态与完成项目

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M2 | Source cubemap direct-Rayon 旁路收编 | `runtime_11_source_cubemap_task_pool_cutover_static_and_managed_build_passed_focused_test_pending` | 2026-07-16 | TDD 红态命中第 3 个未分类 direct-Rayon owner；绿态聚焦回归 1/1，Runtime11 audit 恢复 2/2 白名单且风险 0；Windows managed 默认特性 build exit 0。 |

## 完成项

- 在 `core::framework::tasks` 增加中立、阻塞式 `ParallelSliceExecutor` 契约；framework 不依赖 `core::runtime`，也不创建进程全局线程池。
- `TaskPool` 在 runtime task owner 中通过现有 `parallel_for` 实现该契约，Rayon 仍只存在于 `pool.rs` 与 `parallel_for.rs` 两个计划白名单 owner。
- source cubemap angular mip owner 删除 direct Rayon 导入；大于等于 128 的输入通过调用方显式执行器保持一 face 一 worker 策略，小输入保持串行。
- `SourceCubemapMipChain` 增加 equirect/captured-face 显式执行器关联构造入口。既有同步入口明确保持串行，不隐式创建池或借用全局 Rayon。
- 新增源级边界回归与输出等价性 Rust 测试；模块文档同步执行所有权、线程预算与 API 选择。

## 验证状态

- 红态：改造前 `python -m unittest tools.tests.test_runtime_job_system_audit -v` 为 0/1；审计报告 source cubemap mipmap 是第 3 个未分类 direct-Rayon 路径，并产生两条 Runtime11 风险。
- 绿态：同一 Python 回归为 1/1；direct audit 报告 `expected_module_count = 9`、`direct_rayon_paths = expected_direct_rayon_paths = 2`、`unexpected_rayon_paths = []`、`unclassified_direct_rayon = []`、`risks = []`。
- 首次 managed build job `fa5beb348e5a46db92338a77a8391ac7` exit 1，唯一错误是 Plugins06 在租约文件中引用不存在的 `CapabilityStatus::Stable`；该 owner 随后硬切为 `Complete`，本切片未吸收 foreign 文件。
- 复跑 managed build job `a6415d1032364e1aabd63687227035aa` 执行 `cargo build -p zircon_runtime --locked`，exit 0，默认特性 `zircon_runtime` 编译和链接通过。
- 新增 Rust 测试 `source_cubemap_explicit_executor_entry_preserves_output_contract` 已进入测试树，但共享 test lane 被 EditorLayout15 长任务占用，本切片未把未执行测试误记为通过；该 gate 保持 pending。
- 完整 runtime 聚合审计在 302 秒工具窗口内完成扫描后，向 PowerShell 管道输出超大 JSON 时触发 `OSError 22`，不计为通过；Runtime11 聚焦审计是本切片的有效结构证据。

## 剩余边界

- `failure-2026-07-13-editor-full-harness-runtime-thread-budget.md` 的多 Runtime/asset worker 双线程预算 P0 继续保持 `open`；本切片只清理 direct-Rayon 旁路，不宣称解决该失败。
- Runtime11 父计划继续保持 `in_progress`，直到 P0 failure、聚焦 Rust test lane 与父计划最终门禁均闭合。
