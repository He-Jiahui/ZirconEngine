---
related_code:
  - zircon_runtime/src/plugin/export_build_plan/error.rs
  - zircon_editor/src/core/jobs/error.rs
  - zircon_editor/src/ui/host/export_process_support/error.rs
  - zircon_editor/src/ui/host/native_dynamic_export_preparation/error.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/error.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/execution.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/controller/job.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/error.rs
plan_sources:
  - docs/plans/zircon_editor/editor/15-build-export-and-publishing.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - cargo check -p zircon_editor --lib --locked
  - cargo test -p zircon_editor --lib --locked core::jobs::error::tests
  - cargo test -p zircon_runtime --lib --locked missing_profile_returns_typed_plan_error
  - cargo test -p zircon_editor --lib --locked export_wizard_job_controller_preserves_ -- --test-threads=1
  - cargo test -p zircon_editor --lib --locked retained_export_worker_ticket_preserves_typed_editor_export_error -- --test-threads=1
doc_type: milestone-detail
---

# Editor 15 typed export error 硬切产出归档

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 完成项目与证据（命令输出 / 文件 / 测试名） |
|---|---|---|---|---|
| M1 前置合同 | Runtime plan / Editor export / Job ticket typed source 硬切 | `完成-局部门禁通过` | 2026-07-12 | `ExportBuildPlanError`、`EditorExportBuildError`、`ExportProcessError`、`NativeDynamicPreparationError` 与 `JobFailure` 形成连续 `Error::source()` 链；manager、Cargo、native preparation、wizard、retained worker 不再用 `String` 作为跨模块失败合同。Wizard stage 同时保留 UI diagnostics 与原始 `JobFailure`，controller 直接把原 failure 交给 ticket；无 `From<String>`、字符串相等或旧返回类型兼容层。 |
| M1 前置合同 | 取消、清理、进程终止与 retained action 收束 | `完成-架构复审修正` | 2026-07-12 | native staging 以 guard 在早退时清理，主失败与 cleanup 失败可组合保留；取消仅在 source-free `Cancelled` 时映射为 `JobError::Cancelled`，清理/IO/终止失败不会被 token 吞掉；Cargo 与通用 process 独立分型；process-tree termination 保存 command spawn/fallback kill IO source；output picker/reveal/project manifest 等 retained action 改为 `DesktopExportActionError`，只在状态栏展示时格式化。 |
| M1 前置合同 | Windows 编译与底层回归 | `完成-通过` | 2026-07-12 | 最终 coordinator check job `1dfa3f85ec644e51a0b71c53658f4f0b`：`cargo check -p zircon_editor --lib --locked --jobs 1` exit 0（57.07s，297 条既有 warning）；日志 `D:/cargo-targets/zircon-engine/editor15-final-check-7.err.log`。此前 job `03bdbea88e554212ab09dd1e12818b84`：Job typed error 2/2；job `81b44518c14940f59ce8e4a4b3832cf8`：Runtime missing-profile typed error 1/1。 |
| M1 前置合同 | Wizard 与 retained worker 真实 ticket source 验收 | `完成-通过` | 2026-07-12 | coordinator job `1a890f49e8e940db8385ad2e93e2ca09`：wizard `preserves_` 3/3（普通 IO source、运行中取消与真实失败并发、typed submit error）；job `85b360f5462b400b83069ac76830ea73`：实际 `DesktopExportEditorJob -> EditorJobSystem -> JobTicket` retained worker source 1/1。二者均从 `JobError` downcast `EditorExportBuildError`，未使用文本断言。日志 `D:/cargo-targets/zircon-engine/editor15-wizard-source-final-3.{out,err}.log`、`editor15-retained-worker-final-2.{out,err}.log`。 |
| M1 前置合同 | Native staging 全生命周期与双根聚合清理 | `完成-通过` | 2026-07-12 | guard 在任一文件系统操作前接管 staging/build roots；pre-clean、早退、显式 cleanup 与 Drop 共用无短路双根 helper；`CleanupBatch` 以首个 `NativeDynamicCleanupError` 为 `#[source]` 并保留 additional failures。coordinator test job `9607d8db858c42239757d06a6801567c` 产出最新 test binary；经 `--list` 确认准确名称后，`cleanup_attempts_both_roots_and_aggregates_both_failures` 1/1、`cleanup_ignores_missing_root_but_still_attempts_the_other_root` 1/1，均 exit 0。 |
| M1 前置合同 | Failure 生命周期与独立复审 | `完成-已回传` | 2026-07-12 | `SPEC APPROVED`、`QUALITY APPROVED`；canonical failure 已由 coordinator 移回来源计划为 [`../14/fixed-2026-07-12-export-build-string-error-boundary.md`](../14/fixed-2026-07-12-export-build-string-error-boundary.md)，Editor15 不保留 duplicate failure artifact。 |
| M1 前置合同 | 外部测试目标锁失败归属 | `已修复-由对应功能处理` | 2026-07-12 | 广泛 wizard 尝试曾被 Editor Layout 15 直接占用共享测试产物阻断，已按功能归属立案并由 Layout owner 修复；回传见 [`fixed-2026-07-12-blend-space-visual-test-target-lock.md`](fixed-2026-07-12-blend-space-visual-test-target-lock.md)。本切片未修改 Layout 功能代码或增加规避兼容层。 |
