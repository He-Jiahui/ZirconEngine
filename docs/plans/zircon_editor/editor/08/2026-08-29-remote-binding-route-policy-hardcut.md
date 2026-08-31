---
plan: docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
child_plan: docs/plans/zircon_editor/editor/08
status: source_complete_validation_pending
date: 2026-08-29
---

# Editor08 Remote Binding And Route Policy Hard Cut

## 目标

封闭 `UiControlRequest::InvokeBinding` 与 `InvokeRoute` 对 command remote policy 的绕过，确保控制面来源不会在 binding 解析后被改写为可信本地 UI 来源。

## Current-source 根因

`CallAction` 会检查 reflected action 的 `callable_from_remote`，direct Remote/Cli operation 也会检查 descriptor policy；但 `InvokeBinding`/`InvokeRoute` 最终把 `EditorOperation` 以 `EditorOperationSource::UiBinding` 交给 dispatcher。该来源不触发 remote gate，并在 journal 中被投影为 `RetainedHost`，因此调用者可以跳过 `CallAction`，直接提交同一 binding 或 route ID 执行 remote-disabled command。

## 架构产出

- `handle_control_request` 仍是当前 Ui control transport 的 remote trust boundary；从该边界解析出的 `EditorOperation` 统一以 `EditorOperationSource::Remote` 调用。
- binding native path 继续记录在 operation journal 中，但不能覆盖调用来源；成功调用投影为 `Headless`，remote-disabled 调用记录 `ControlFailure`。
- `InvokeBinding` 与 `InvokeRoute` 共享同一 policy gate；route lookup、argument overlay 或 binding normalization 均不能把来源降级为本地 UI。
- 本切片只封闭当前 P0 绕过，不冒充完整 `InvocationPrincipal/SourceProvenance` 链；typed principal、transport/request stage 和 surface policy matrix 仍属于后续 gateway 里程碑。

## 验证计划

- 回归注册一个 `callable_from_remote(false)` 的 event-backed command 及 reflected menu route，分别直接提交 binding 和 route ID。
- 两条路径均必须返回 remote policy 错误，journal 各记录一个 `Headless + ControlFailure + operation_id`，不得执行业务 event。
- 既有可远程调用 operation binding 仍成功生成 transaction，并保留 native binding path；其来源期望从错误的 `RetainedHost` 修正为 `Headless`。
- 受管 Windows Cargo 使用 E 盘 target；验证终态和独立复核前不关闭该计划。

## 产出记录与时间

| 日期 | 项目 | 状态 | 证据 |
| --- | --- | --- | --- |
| 2026-08-29 | InvokeBinding/InvokeRoute remote source hard cut | `completed` | `invoke_editor_binding` 的 operation 分支从 `UiBinding` 改为 `Remote`，复用现有 descriptor remote gate，同时保留 native binding path。 |
| 2026-08-29 | 双入口 policy regression source | `completed` | 新增 direct binding + reflected route ID 双路径拒绝用例；要求错误包含 canonical command ID，journal 为 2 条 `Headless` `ControlFailure`，业务 event 零执行；既有成功 provenance 用例同步要求 `Headless`。 |
| 2026-08-29 | managed Cargo validation and independent review | `pending` | 当前受管验证已有 accepted/no-terminal 作业；本 source 尚未取得 source-bound receipt，不声明测试、C/I/M、commit 或企微完成。 |
