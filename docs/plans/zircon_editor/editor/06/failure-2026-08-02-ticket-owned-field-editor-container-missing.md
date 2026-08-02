---
handoff_kind: failure
status: open
created_at: 2026-08-02
summary_slug: ticket-owned-field-editor-container-missing
origin_plan: docs/plans/zircon_editor/editor/06-ui-extension-framework.md
origin_workflow_node: M2
fixing_plan: docs/plans/zircon_editor/editor/06-ui-extension-framework.md
origin_child_dir: docs/plans/zircon_editor/editor/06
fixing_child_dir: docs/plans/zircon_editor/editor/06
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - zircon_editor/src/core/extension/inspector.rs
  - zircon_editor/src/core/extension/store/model.rs
  - zircon_editor/src/ui/host/editor_event_runtime_reflection.rs
  - zircon_editor/src/ui/workbench/snapshot/data/editor_state_snapshot_build.rs
tests:
  - plugin_field_editor_is_ticket_owned_capability_filtered_and_revocable
  - editor_snapshot_resolves_plugin_field_editors_from_active_contributions
  - cargo test -p zircon_editor --lib --locked
---

# Editor06: Field editor container is not ticket owned

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/06-ui-extension-framework.md`
- 来源执行切片：M2 Inspector 双层定制，字段类型级 editor container
- 修复责任计划：`docs/plans/zircon_editor/editor/06-ui-extension-framework.md`
- 交接原因：该缺口属于 Editor06 内部贡献存储与 Inspector 投影边界的本地修复，不转移到其他编号计划。
- 当前执行会话：`editor06-document-toolkit-hardcut-r1-20260801`

## 失败现象与复现证据

`FieldEditorContainer` 虽已提供六类内建 editor，却由 `global_field_editors()` 的进程全局
`RwLock` 提供。`ContributionBatch`、`ContributionSnapshot`、ticket keys/counts 与 revoke 路径均没有
field-editor family；`InspectorPluginComponentPropertySnapshot::field_editor()` 又在 pane 投影时重新读取该
全局容器。

因此插件来源的字段编辑器既不能经过 ticket/能力门控，也不能随 revoke 从下一代 UI 快照消失；已发布快照还会被后续全局
注册改写其 pane 元数据。这违背 Plan06 对统一贡献生命周期和 fixture plugin field-editor materialize/revoke 的合同。

## 最低共享层根因

`ContributionStore` 已经是所有可撤销 editor 贡献的权威索引，宿主也已经从 capability-filtered snapshot
构造 `InspectorCustomizationChain`。缺口仅是 field-editor 定义未进入同一批模型，以及 snapshot 未在构建时固定
resolved editor。不得再增设第二套 registry、全局锁或跨 generation 可变缓存。

## 架构修复验收

- `FieldEditorDefinition` 成为 `ContributionBatch` 的一族，具有插件命名空间校验、冲突拒绝、counts/keys、
  capability-filtered `ContributionSnapshot` 查询与 ticket revoke 回收；旧 generation reader 保持不可变。
- host 从同一 capability-filtered Store snapshot 构造带六类 builtins 的 field-editor container；贡献定义在
  snapshot 构建期间解析为 property metadata，pane 投影不得读取任何全局 field-editor 注册表。
- capability 缺失或 ticket revoke 后的新 editor snapshot 必须退回内建/auto editor；revoke 前已生成的
  snapshot 继续保留其解析结果。
- focused regressions 与受管 `cargo test -p zircon_editor --lib --locked` 成功，再经独立复审后回填 fixed record。

## 禁止临时方案

- 禁止保留 `global_field_editors()`、`RwLock` 或任何 process-global 可变 editor catalog 作为回退。
- 禁止在 pane/retained UI 投影时重新解析 field editor，或通过 capability flag 隐藏但保留可达定义。
- 禁止为了接入旧 `EditorExtensionRegistry` 而增加平行注册 API；插件 catalog materialization 的独立硬切继续由
  Editor12 failure 链处理。

## 修复结果与回传

Open state: 待修复; no pass is claimed. Forward fix in progress. The repair owns the Editor06 Store, host snapshot, and inspector projection boundary only;
Editor12 remains the owner of plugin catalog materialization into `ContributionBatch`.

## 产出记录与时间

- 2026-08-02：状态 `fixing`。已确认全局 field-editor container 绕过 Store ticket/revoke 与快照不可变性；开始以
  Store family、capability-filtered host materialization 和 snapshot-time resolution 前向修复。
- 2026-08-02：状态仍为 `fixing`。Field editor 已进入 `ContributionBatch`/ticket keys/counts/revoke，host 从
  capability-filtered contribution snapshot 构造 container，property 在 editor snapshot 构建时冻结 resolved
  metadata，pane 只投影该冻结值；能力移除、旧快照稳定、命名空间/冲突和 asset marker 投影回归均已补齐。
  静态合同 4/4、格式与 diff 检查通过，二次独立复审为 `0/0/0`；受管 broad gate
  `9e4a430e1a69408ca8c1a9c8393f372a` 已提交，未收到 terminal evidence 前不得 fixed return。
- 2026-08-02：状态仍为 `fixing`。后续自审修正了一个 qualified-type 边界：container 现在先精确匹配
  `plugin.*` 类型、再回退六类内建归一化规则；仅原始 `f32`/`boolean` 等内建别名被拒绝，
  `plugin.sample.CloudColor` 可注册并覆盖 color fallback。相应回归及静态合同再次通过；此前 broad gate
  不再作为当前源码验收，待新的 source-bound gate 与该最终小改动复审后再 fixed return。
- 2026-08-02：状态仍为 `fixing`。qualified-type/revoke 的最终独立复审为
  `Critical/Important/Minor = 0/0/0`；container 保持 ticket-owned、capability-filtered 与 immutable
  snapshot-time resolution，未恢复全局 field-editor registry 或旧架构 fallback。当前 Editor06 静态合同
  `16/16`、局部格式与 diff 检查通过；仍缺最终 current-source 受管 Cargo terminal evidence，禁止 fixed return。
