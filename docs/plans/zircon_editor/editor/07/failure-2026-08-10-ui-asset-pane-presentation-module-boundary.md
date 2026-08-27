---
handoff_kind: failure
status: open
created_at: 2026-08-10
summary_slug: ui-asset-pane-presentation-module-boundary
origin_plan: docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md
fixing_plan: docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md
origin_child_dir: docs/plans/zircon_editor/editor/07
fixing_child_dir: docs/plans/zircon_editor/editor/07
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - zircon_editor/src/ui/asset_editor/session/mod.rs
  - zircon_editor/src/ui/asset_editor/session/presentation
  - zircon_editor/src/ui/asset_editor/session/ui_asset_editor_session.rs
  - zircon_editor/src/ui/asset_editor/session/lifecycle.rs
tests:
  - UiAssetEditorSession pane presentation equivalence after hard-cut module relocation
  - static domain build/reuse ordering across document, source, selection, preview and import generations
  - current-source managed zircon_editor Rust validation
---

# Editor07: UI Asset pane presentation folder boundary

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md`
- 来源执行切片：2026-08-10 UI Asset pane presentation 独立二次审查。
- 修复责任计划：`docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md`
- 交接原因：问题局限在 Editor07 自身的 UI Asset session 模块边界，使用 local failure
  记录该计划内的前向 hard cut，避免伪造跨计划 owner。

## 失败现象与复现证据

独立二次审查确认 `zircon_editor/src/ui/asset_editor/session/presentation_state.rs` 已有 973 行，其中 `UiAssetEditorSession::pane_presentation` 单一编排路径约 864 行，同时承担 reflection、preview/drag、source、inspector、style、theme、command availability 和 DTO assembly。继续在该文件中加入 generation artifact 或 typed dirty-domain 会跨越模块边界阈值，并使性能域无法独立测试和失效。

当前受管快照 `1548` 仅冻结了审查前源状态；由于本结构性问题仍为 Important，不能用于此 failure 的 accepted validation 或 fixed return。

## 最低共享层根因

UI Asset session 将多个独立 pane domain 的生产、失效和最终 DTO 编排堆叠在单一平面
`presentation_state.rs` 中。模块边界无法表达 document/import/source/selection/preview generation
的所有权，导致每个新增 domain 同时扩大编排函数和其测试、失效范围。

## 架构修复验收

- 删除 `session/presentation_state.rs` 模块路径，改由 `session/presentation/` 文件夹承载 pane domain producer；不得保留 `presentation_state` re-export、wrapper 或 compatibility facade。
- `session/mod.rs` 只挂载新的 folder root。按 reflection、preview、source、inspector、style、theme、command 与最终 pane assembly 划分生产模块；每个模块只公开其必要 typed artifact 或 builder。
- document/import/source/selection/preview generation 的静态 artifact 必须由 session 所有，失效入口集中在 lifecycle 与相关 mutation；host 不得持有完整 pane DTO cache。
- 先以 Rust 行为测试冻结 DTO/ordering 等价和 stable generation reuse，再迁移生产实现；完成后运行独立二次审查和 current-source managed Rust validation。

## 禁止临时方案

- 不保留 `presentation_state` 的 re-export、wrapper、compatibility facade 或双路径实现。
- 不把完整 pane DTO cache 移到 host，或用全量重建掩盖 generation 失效边界。
- 不以审查前快照、静态格式化或 diff 检查替代 hard cut 后的行为测试与 current-source managed validation。

## 修复结果与回传

Open state: `folder_backed_presentation_hard_cut_code_complete_managed_validation_pending`。生产模块已迁移，
独立复审的 P1 已前向修复并完成复核；但尚未取得 hard cut 后的受管 current-source Rust
终态，因此不得标记 fixed 或 accepted。

## 产出记录与时间

| 日期 | 里程碑/切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-08-10 | Pane presentation module boundary | open / forward fix required | 独立只读二次审查为 Critical/Important/Minor `0/1/0`：唯一 Important 为 973 行 `presentation_state.rs` 中 864 行的跨域编排。静态合同 12/12、`rustfmt --check` 与 scoped `git diff --check` 已通过，但这些不能替代 folder-backed hard cut、managed Rust 验证或 failure return。 |
| 2026-08-10 | Folder hard cut implementation | code complete / managed validation pending | 删除 `session/presentation_state.rs`，`session/mod.rs` 只挂载 `presentation/`；最终 production split 为 `reflection`、`preview`、`source`、`inspector`、`style`、`theme`、`commands` 与 `pane`。pane 只映射 typed domain artifacts；source outline build counter 继续只归属 lifecycle/navigation 的实际索引构建。复审 P1 的 style 旁路已前向修复，`selected_node_selector` 与 `stylesheet_items` 仅由 `style.rs` 生成，pane 只消费 typed artifact。`structure_split` 边界回归、`navigation_state` 的无 mutation pane 值等价测试、248 个 DTO 字段精确映射、`rustfmt --check`、scoped `git diff --check` 与 handoff validator `579/0` 已完成静态验证；Editor07 import/pane/preview/source Python contracts 13/13 通过。独立复审终态 Critical/Important 为 `0/0`。尚未运行 Cargo，failure 保持 open。 |

## 2026-08-27 retired-path metadata repair

Commit `7a20f921bb97ed428ae248cbcaf3c2fac5442ddf` completed the recorded hard cut by
deleting `session/presentation_state.rs`; the existing `session/presentation` module
tree is already the canonical structured owner above. The retired leaf is therefore
removed from `related_code` rather than preserved as a compatibility anchor. Live
presentation sources were not modified, and managed Rust validation is still pending,
so the failure remains open.
