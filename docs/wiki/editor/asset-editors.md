---
related_code:
  - zircon_editor/src/core/asset/mod.rs
  - zircon_editor/src/ui/asset_editor/mod.rs
  - zircon_editor/src/ui/animation_editor/mod.rs
  - zircon_editor/src/ui/material_editor/mod.rs
  - zircon_editor/src/ui/preview_scene/mod.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/mod.rs
implementation_files:
  - zircon_editor/src/ui/asset_editor/session/ui_asset_editor_session.rs
  - zircon_editor/src/ui/asset_editor/session/lifecycle.rs
  - zircon_editor/src/ui/asset_editor/undo_stack.rs
  - zircon_editor/src/ui/asset_editor/preview/preview_host.rs
  - zircon_editor/src/ui/animation_editor/session.rs
  - zircon_editor/src/ui/material_editor/projection.rs
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 编辑器详细 Wiki
  - .codex/plans/Zircon UI 资产化 Widget Editor 与共享 Layout.md
  - .codex/plans/UI Asset Editor 与共享 Layout 未完成内容归档.md
  - docs/editor-and-tooling/ui-asset-editor-host-session.md
tests:
  - zircon_editor/src/ui/asset_editor
  - zircon_editor/src/ui/animation_editor
  - zircon_editor/src/tests/ui
doc_type: module-detail
---

# 资产编辑器

## 统一资产作者态模型

资产浏览、打开和导入从 `core::asset` 开始。`EditorAssetIndex` 组合 runtime `AssetRegistryIndex` 与编辑器 transient 状态，提供按 UUID/path 查询、watch event 应用和 import generation 防陈旧提交。Runtime registry 是资产身份与依赖真源；Editor index 增加导入中、dirty path 等作者态信息。

`AssetTypeRegistry` 为不同资产类型注册：

- `AssetTypeId` 和 presentation。
- 创建模板。
- `AssetToolkitDescriptor`，决定用哪个编辑器打开。
- context command。
- thumbnail provider 或 placeholder palette。

插件贡献资产类型时应注册完整 definition，而不是在资产浏览器中按扩展名写特殊分支。

## UI Asset Editor

### 用途与模式

`UiAssetEditorSession` 是 UI 资产的作者态权威。它同时服务 Source、Hierarchy 和 Canvas，当前支持 legacy template document 与 UI v2 文档的投影/序列化。

`UiAssetEditorMode` 控制 Source/Design 等工作模式；`UiAssetPreviewPreset` 控制预览尺寸/设备语义；`UiDesignerToolMode` 控制 canvas 工具。

### 打开会话

典型 host 流程：

1. 解析 `UiAssetEditorRoute` 和资产来源。
2. 读取源文本，调用 `UiAssetEditorSession::from_source` 或 `from_v2_source`。
3. 保存 session 到 host 的 asset-editor session registry。
4. 打开 `UI_ASSET_EDITOR_WINDOW_ID` 对应 view/window。
5. 每次变化构建 `pane_presentation()` 和 `reflection_model()`。

构造失败返回 `UiAssetEditorSessionError`；parse/compile diagnostics 应投影到 pane，而不是丢失 last-good preview。

### Source、Hierarchy、Canvas 同步

三种表面共享 `UiDesignerSelectionModel` 和稳定 node id：

- Source 选择行/字节偏移后映射到 outline node。
- Hierarchy 选择节点后定位 source block 并更新 Inspector subject。
- Canvas 点击 preview node 后回到相同 selection。

源文本重新 parse 或 tree edit 后，session 按 node id/parent/mount/sibling 重建选择。若节点不再存在，按 reconciliation 规则降级；不能用旧数组 index 继续选择。

### 结构编辑

`UiAssetEditorCommand::TreeEdit` 和 `UiAssetEditorTreeEditKind` 表达插入、移动、reparent、wrap、unwrap 等结构变更。Palette 支持插入为子节点/相邻节点和 drag target chooser；会话提供 move up/down、into previous/next、outdent 等操作。

所有结构编辑更新 canonical source、document projection、selection、preview 和 diagnostics。Tree edit 失败应保持原 source/selection。

### 属性、Binding 与 Style

Inspector 编辑 typed widget props/state、slot mount/padding/size 和 layout size。Binding inspector 使用 `UiEventKind + UiActionRef + payload`，不保存宿主 callback 名称。

Style 工具支持：

- class 增删和 inline override。
- 从 selection 创建 rule、将 inline override 提取为 rule。
- token 和 stylesheet rule 增删、重命名、排序。
- declaration 编辑和 matched rule inspection。
- pseudo-state preview。
- theme source clone/detach/refactor 与重复 override 清理。

Promotion 工作流支持将节点转换为 reference、提取为 local component、提升为 external widget，以及提升 local theme 到外部 style 资产。涉及外部文件时会生成 `UiAssetEditorExternalEffect`，由 host 在事务边界外执行并记录 replay 信息。

### Preview

`UiAssetPreviewHost` 接受预览尺寸、资产标识和已编译 UI 文档，构建共享 `UiSurface`：

```rust
let mut preview = UiAssetPreviewHost::new(size, asset_id, &compiled)?;
preview.rebuild_with_size(next_size, asset_id, &compiled)?;
let surface = preview.surface();
```

UI v2 使用 `new_v2(size, &document, &compiled)`。Preview 可注入 mock property/nested value、locale 和 pseudo-state；这些是预览作者态，不应写入 runtime gameplay model。

### Undo、重做与 Replay

UI Asset Editor 使用专用 `UiAssetEditorUndoStack`，每个 transition 可保存：

- source cursor/snapshot。
- selection snapshot。
- tree edit 与 inverse edit。
- external effects。
- document replay commands。

`can_undo/can_redo` 和 `next_undo_label/next_redo_label` 驱动界面。`UiAssetEditorCommandJournal` 支持会话 replay；bug report artifact 汇总 route、source、selection、commands 和 external effects，并带 schema version。

它与 Scene 的 `EditorTransactionEngine` 是两个明确的历史域。Host 可把 UI session dirty 状态纳入文档 tab，但不能直接把两种命令记录混合 replay。

## Animation Editor

`AnimationEditorSession` 为 sequence、graph 和 state machine 提供最小真实会话。`AnimationEditorCapabilityDescriptor`/capability table 描述每类文档允许的命令，拒绝原因通过 `AnimationEditorCommandDiagnostic` 与 `AnimationEditorCommandRejectionReason` 暴露。

`AnimationCommand` 覆盖轨道、绑定、关键帧、scrub 和 playback 等 typed 行为；`AnimationEditorPanePresentation` 负责工作台投影。Curve 与 timeline foundation view 是共享编辑控件，不代表所有高级动画工具已完成。

状态：**可扩展基础**。文档类型检查、会话和 pane 已建立；高级图编辑、完整资产保存与所有命令覆盖仍需按 capability 判断，不应仅凭 pane 存在宣称完整。

## Material Editor

`MaterialEditorProjection` 把材质属性、纹理 slot 和 diagnostics 投影为工作台数据；`RendererDataEditorProjection` 展示 renderer feature/data 行。该模块当前主要是 typed projection。

状态：**可扩展基础**。调用者应检查是否有相应 command/host save 路径；只读 projection 不等于完整材质图编辑器。

## Preview Scene 与通用编辑控件

`preview_scene` 为资产编辑器提供隔离预览场景基础。`curve`、`graph`、`timeline`、`control` 是可复用 UI/数据模型。具体资产编辑器负责建立 session、命令、保存和 runtime preview 边界，不能只复用控件就跳过作者态所有者。

## 资源与导入限制

- Asset index watch/import 使用 generation 拒绝过时的异步完成。
- 删除/移动先执行 preflight，引用关系决定是否拒绝、级联或要求确认。
- 外部效果由 host 执行；session 仅描述并记录效果。
- UI v2 是当前收敛方向，legacy 与 v2 转换函数为内部迁移工具，不构成长久双格式承诺。
- Last-good preview 可以在源文本有错误时继续显示，但保存/导出必须使用明确的 canonical source 和 diagnostics 策略。
