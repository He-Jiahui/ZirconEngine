---
handoff_kind: fixed
status: fixed
created_at: 2026-07-12
summary_slug: export-build-string-error-boundary
origin_plan: docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
fixing_plan: docs/plans/zircon_editor/editor/15-build-export-and-publishing.md
origin_child_dir: docs/plans/zircon_editor/editor/14
fixing_child_dir: docs/plans/zircon_editor/editor/15
related_code:
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/manager.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/cargo_build.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/execution.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/job_queue/worker.rs
tests:
  - static E1 audit of Result<_, String>, error.to_string(), and format! error boundaries under editor_manager_plugins_export/export_build
resolved_at: 2026-07-12
---


# Editor 15：导出构建 String 错误边界丢失 typed source

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md`
- 来源执行切片：Editor14 E1 typed job failure 硬切前置审计
- 修复责任计划：`docs/plans/zircon_editor/editor/15-build-export-and-publishing.md`
- 交接原因：Editor14 可以保证 `JobTicket` 保存传入的 typed source，但无法从 Editor15 已返回的 `String` 逆向恢复原始导出、物化、进程或 IO 错误；最低丢失点属于导出管线 owner。

## 失败现象与复现证据

2026-07-12 静态 E1 审计确认 `export_build/manager.rs` 的公开与 crate 内导出入口至少 8 处返回 `Result<..., String>`，并在物化路径存在多处 `.map_err(|error| error.to_string())?`。`cargo_build.rs`、wizard command runner 与进程等待辅助也继续返回 `String`，错误源在进入 Editor14 `JobError` 前已经被压扁。

当前 retained export worker 只能把该字符串包装为 job failure。即使 Editor14 把统一调度错误硬切为可 downcast source，Editor15 的底层 `ExportBuildPlan`、物化、Cargo/child process 和 IO source 仍不可恢复。

## 最低共享层根因

最低共享 owner 是 Editor15 导出管线的错误合同：导出计划、物化、native preparation、Cargo 调用与取消清理没有统一 typed `EditorExportBuildError`，而以 `String` 作为跨模块返回类型和组合协议。

## 架构修复验收

- 新建 Editor15 所有的 typed `EditorExportBuildError`（按计划解析/物化/native preparation/Cargo/IO/取消等真实来源分型），并通过 `#[source]` 保留底层错误链。
- `manager.rs`、`cargo_build.rs` 和生产 wizard execution 路径不再返回 `Result<_, String>`，不再用 `error.to_string()` 或 `format!()` 作为错误传播。
- retained export job 将 typed `EditorExportBuildError` 直接交给 Editor14 `JobError::failed`；ticket 可 downcast 回导出错误，`JobEventKind::Failed` 仅在消息投影层字符串化。
- 运行 Editor15 focused 导出错误矩阵，并向上重跑 Editor14 typed source ticket/event 合同。

## 禁止临时方案

- 禁止新增字符串包装 error、`From<String>`、文本相等、调用点特判、兼容重载或双轨返回类型。
- 禁止只在 Editor14 外包一层命名结构体就宣称恢复 source；已在 Editor15 丢失的底层 source 必须由原 owner 保留。

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| Editor14 E1 / Editor15 导出合同 | typed source 边界审计 | `open-待Editor15硬切` | 2026-07-12 | `manager.rs` 至少 8 个 `Result<..., String>` owner，物化路径 `.map_err(|error| error.to_string())?` 多处；`cargo_build.rs` 与 wizard execution 仍沿用 String error。 |

## 修复结果与回传

- 根因：Editor15 export plan/materialization/native/Cargo/wizard/retained action boundaries flattened domain and IO failures into String before Editor14 JobTicket could retain the source.
- 架构修复：Hard-cut all production export boundaries to typed ExportBuildPlanError, EditorExportBuildError, ExportProcessError, NativeDynamicPreparationError and DesktopExportActionError; preserve JobFailure through wizard and retained tickets; retain cancellation/termination dual failures; extend native staging ownership guard across pre-clean, materialization, explicit cleanup and Drop with two-root aggregation and typed source chaining; no legacy String compatibility path.
- 验证：Windows check coordinator job 1dfa3f85ec644e51a0b71c53658f4f0b exit 0; wizard ticket job 1a890f49e8e940db8385ad2e93e2ca09 3/3; retained worker ticket job 85b360f5462b400b83069ac76830ea73 1/1; native cleanup focused tests 2/2 from job 9607d8db858c42239757d06a6801567c latest binary; scoped String audit clean; SPEC APPROVED and QUALITY APPROVED.
- 回传：Editor15 typed export owner contract is repaired and upward ticket/source validation passed; return the fixed artifact to Editor14 for its functional plan record.
