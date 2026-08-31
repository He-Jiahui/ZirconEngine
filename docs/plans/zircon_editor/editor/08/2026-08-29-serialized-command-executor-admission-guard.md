---
plan: docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
child_plan: docs/plans/zircon_editor/editor/08
status: source_complete_validation_pending
date: 2026-08-29
---

# Editor08 Serialized Command Executor Admission Guard

## 目标

在 stable native callback/host-route executor、owner generation lease 与 unload quiescence 尚未形成同一 admission contract 前，禁止 serialized plugin `Command` 发布为可发现但必然执行失败的 operation descriptor。

## Current-source 根因

`zircon.editor.command/2` 目前只声明 identity 与 localization metadata。materializer 曾把它转换成 `EditorCommandDescriptor::localized_operation`，但没有 `OperationCommandFactoryRegistration` 或独立 command executor；dispatcher 因而稳定进入 `MissingFactory`。native editor behavior 虽已有 `invoke_command` ABI、manifest slot table 和 callback generation owner，Editor 注册路径只复制 serialized batch，未保留 callback binding/provenance，也未接入 unload 前的 callback quiescence。

## M0 硬切

- materializer 对 serialized `Command` 返回 typed `MissingExecutor`，不再注册 metadata-only operation。
- materializer 继续采用 clone-then-publish；batch 在 command 前即使已物化 view/menu 等候选内容，错误也保证原 registry 零变化。
- 非 executable contribution 仍可物化；menu 只能引用宿主已存在的 command，不能用同批 metadata-only command 伪造执行闭环。
- native fixture 的空 command manifest 与 serialized command 不再被视为可用产品能力；完整 executor 到位前 package 应 fail closed，而不是展示一个不可执行菜单。

## 完整 executor 前置设计

后续实现不能把 native callback 包装成可撤销 edit factory。应建立 command execution registration，明确区分 event、transactional operation 与 external/native endpoint；native binding 必须由 Runtime 暴露 generation-owned editor behavior snapshot，执行前由 manifest table 解析 command slot，执行期间持有 callback lease，revoke/unload 先关闭 admission 并等待 in-flight callback，再原子移除 definition/executor/menu projection。

payload codec、结果 payload、surface policy、principal permission 和 resource budget 必须进入 versioned definition/admission。没有这些合同前，不增加空 factory、no-op transaction、字符串 callback 分派或 legacy fallback。

## 验证计划

- materializer 定向回归证明 command 返回 exact `MissingExecutor`，且 command 前已写入 candidate 的 view 不泄露到 live registry。
- 既有 supported-kind fixture 改为只覆盖非 executable contributions，并使用宿主预注册 command 验证 menu metadata 投影。
- 受管 Windows Cargo 与独立复核取得 current-source receipt 后，才可把本 M0 guard 标记为 validated；完整 executor 仍是后续 P0/M5，不因 fail-close 而关闭。

## 产出记录与时间

| 日期 | 项目 | 状态 | 证据 |
| --- | --- | --- | --- |
| 2026-08-29 | metadata-only serialized command fail-close | `completed` | materializer 的 `Command` 分支返回 `SerializedContributionMaterializationError::MissingExecutor`；旧 `localized_operation` 发布路径删除，无兼容 fallback。 |
| 2026-08-29 | atomic rejection regression source | `completed` | 新增 candidate view 后遇到 command 的回归，要求 exact command ID、live views 保持原 1 项、command count `0`；supported-kind fixture 不再把 metadata-only command 当作支持项。 |
| 2026-08-29 | stable native callback executor + owner lease | `pending` | 需跨 Runtime native loader 建立 editor callback snapshot/binding、manifest slot、payload codec、surface policy、callback lease 与 unload quiescence；本 guard 仅封闭发布错误，不声明执行闭环。 |
| 2026-08-29 | managed Cargo validation and independent review | `pending` | current source 尚无受管 Windows receipt，不声明测试、C/I/M、commit 或企微完成。 |
