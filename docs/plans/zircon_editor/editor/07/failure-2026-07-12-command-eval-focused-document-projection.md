---
handoff_kind: failure
status: open
created_at: 2026-07-12
summary_slug: command-eval-focused-document-projection
origin_plan: docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
fixing_plan: docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md
origin_child_dir: docs/plans/zircon_editor/editor/08
fixing_child_dir: docs/plans/zircon_editor/editor/07
related_code:
  - zircon_editor/src/core/commands/document_kind.rs
  - zircon_editor/src/core/commands/when.rs
  - zircon_editor/src/ui/host/command_eval_projection.rs
  - zircon_editor/src/ui/asset_editor
  - zircon_editor/src/ui/animation_editor
tests:
  - cargo test -p zircon_editor --lib --locked command_eval
  - cargo test -p zircon_editor --lib --locked focused_document
---

# Editor 07：CommandEvalCtx 缺少 focused document kind 权威投影

## 产出记录与时间

| 状态 | 记录日期 | 完成项目与当前门禁 |
|---|---|---|
| `OPEN / typed focus owner 已实现 / Cargo 复验排队` | 2026-07-14 | `ViewDescriptor.document_kind` 成为领域文档类型 owner，内建 scene / prefab / material / UI asset / animation sequence / animation graph 均声明 typed kind；`EditorSessionState` 与 `ProjectEditorWorkspace` 无兼容字段地从 `active_center_tab` 硬切为跨主页面/浮动窗口统一的 `focused_view`。Chrome 只经 `focused_view -> ViewInstance -> ViewDescriptor` 投影 `CommandEvalCtx`；增加 floating activation + focused close 行为回归。静态 hard-cut 1/1 通过；新增责任从 907 行 `core/editor_extension.rs` 提取为 folder-backed `core/editor_extension/view_descriptor.rs`，root 收敛到 860 行，模块边界 guard RED→GREEN 1/1，日志 `.codex/tmp/editor07-focused-document-projection-module-boundary-red-20260714.log`、`.codex/tmp/editor07-focused-document-projection-module-boundary-green-20260714.log`。受管 Windows current-source Cargo exact 首次运行在进入 Editor 编译前被 Plugins08 reflection、Text02 variable shaping 与 Runtime04 reference resolver 共 31 项错误截断，日志 `.codex/tmp/editor07-focused-document-current-exact-20260714.log`；失败已写入各自功能计划且对应代码随后已有 owner 修正，第二轮 exact 等待同一受管 pool，暂不回传 fixed。 |
| `OPEN / focused-view 词汇硬切 GREEN / current Cargo 被 EditorUI03 阻断` | 2026-07-14 | 发现 animation session 错误分类仍残留旧 “active center tab” 词汇后，以静态 guard 先红后绿：私有 resolver、容错分类与断言统一为 `focused view`，旧字段/函数/错误词汇扫描为 0。受管 job `9cc782db74224c43887dfe73b46a4680` 的第二轮 exact 已实际编译当前源码；自有 E0432 仅为测试 import 未跟随 `EDITOR_MANAGER_NAME` 的唯一 `ui::host::module` owner，已直接修正且不恢复 root re-export。随后仍在测试体前被 retained paint-text `ShapedGlyph` fixture 缺少 `font_instance_id` 的 E0063 阻断，已并入 [EditorUI03 retained-text failure](../../editor_ui/03/failure-2026-07-11-retained-text-family-and-subpixel-contracts.md)，日志 `.codex/tmp/editor07-focused-document-current-exact-r2-20260714.log`；本 artifact 保持 open。 |

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md`
- 来源执行切片：Plan08 M1.2 类型化 when 谓词与统一求值环境
- 修复责任计划：`docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md`
- 交接原因：Plan08 定义 `DocumentKind` 与 `FocusedDocumentKind` 谓词；领域 toolkit/document session 的类型和焦点生命周期属于 Editor07，不能由命令层把“工程已打开”推断成 scene 文档焦点。

## 失败现象与复现证据

原始失败为 `CommandEvalCtx.focused_document_kind` 默认 `None`，`command_eval_ctx_from_chrome` 没有消费领域文档 owner。当前修复已补齐 typed descriptor 与唯一 `focused_view` 投影；Plan08 的 project-open 不得自动制造 focused scene document 回归保持不变。

修复前静态复现为生产 `with_focused_document_kind(` 调用为零；修复后 host 从 Chrome 的 typed snapshot 消费 `focused_document_kind`。current-source Cargo 已实际启动，不再是共享池等待状态；首次重跑未能进入 Editor 测试执行的原因是下层 owner 编译失败：[Plugins08 reflection（已回传 fixed）](../../../zircon_runtime/render/18/fixed-2026-07-14-derived-reflection-visibility-compilation.md)、[Text02 shaping](../../../zircon_runtime/text/02/failure-2026-07-14-variable-shaping-visibility-compilation.md) 与 [Runtime04 reference resolver](../../../zircon_runtime/runtime/04/failure-2026-07-13-stale-subasset-reference-repair.md)。

## 最低共享层根因

最低根因是旧 `active_center_tab` 只能描述主页面 tab，不能表达浮动窗口焦点，而且 `ViewDescriptor` 没有 typed document metadata。修复层位于 toolkit/view descriptor 与 workspace focus 生命周期；Workbench 不根据 tab 标题、显示名、路径后缀或 view id 猜类型。

## 架构修复验收

- 领域 toolkit/document session 提供类型化 `DocumentKind`，焦点切换、关闭、浮动窗口激活时更新唯一共享 `CommandEvalCtx`。
- Scene、asset、animation、graph、timeline 等实际 owner 采用稳定 kind id；无焦点时明确写回 `None`。
- `FocusedDocumentKind` 在菜单、命令面板、UI binding 三入口一致，headless 仍保持不适用。
- 增加跨文档焦点切换/关闭/浮动窗口回归，并回跑 Plan08 command eval 测试。

## 禁止临时方案

- 禁止从 tab title、显示名、路径后缀或 view id 字符串临时猜测文档类型。
- 禁止在各领域编辑器私建 command context，禁止 project-open 默认等同 focused scene。
- 禁止为旧文档架构保留 shim、fallback kind 或双路焦点状态。

## 修复结果与回传

Open state: `待修复`; no pass is claimed. 完成后由 Editor07 在本文件记录验证证据，并向 `../08-tool-orchestration-and-commands.md` 回传可关闭结论。
