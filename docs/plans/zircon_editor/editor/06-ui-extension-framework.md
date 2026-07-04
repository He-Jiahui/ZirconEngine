---
related_code:
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/core/editor_authoring_extension.rs
  - zircon_editor/src/ui/workbench/mod.rs
  - zircon_editor/src/ui/workbench/view/view_host.rs
  - zircon_editor/src/ui/workbench/layout/manager/persistence.rs
  - zircon_editor/src/ui/layouts/views
  - zircon_editor/src/ui/reflection/builder.rs
reference_sources:
  - dev/godot/editor/plugins/editor_plugin.h
  - dev/UnrealEngine/Engine/Source/Editor/PropertyEditor/Public/IDetailCustomization.h
  - dev/UnrealEngine/Engine/Source/Editor/PropertyEditor/Public/IPropertyTypeCustomization.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/Toolkits/AssetEditorToolkit.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Docking/TabManager.h
  - dev/Fyrox/editor/src/plugins/inspector/mod.rs
plan_sources:
  - docs/plans/zircon_editor/editor/00-editor-architecture-overview.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/zircon_editor/editor_layout/03-jetbrains-docking-workbench.md
  - docs/plans/zircon_editor/editor_layout/04-layout-presets-and-persistence.md
  - docs/plans/zircon_editor/editor_layout/08-plugin-page-interface-and-messaging.md
status: planned
---

# 06 编辑器 UI 扩展框架（drawer / window / inspector / field / 自定义区域）

本计划落地 00 §6 的「扩展贡献」权威 `ContributionStore` 与「布局/视图态」持久化。

## 参照证据（dev/）

**godot 贡献位与注册**（`editor_plugin.h:76-89, 122-165, 248-249, 339-378`）：`CustomControlContainer` **12 位有限枚举**（TOOLBAR / SPATIAL_EDITOR_{MENU,SIDE_LEFT,SIDE_RIGHT,BOTTOM} / CANVAS_EDITOR_{...} / INSPECTOR_BOTTOM / PROJECT_SETTING_TAB_{LEFT,RIGHT}）；`add_control_to_container / add_inspector_plugin`；生命周期虚函数 `_edit/_handles/_make_visible/_get_state/_set_state/_enable_plugin/_disable_plugin`；inspector 责任链 `_can_handle → _parse_property`（返回 true 截获）。要点：**贡献位是有限枚举**（可持久化可校验）。

**UE 属性定制双层**（`IDetailCustomization.h`/`IPropertyTypeCustomization.h`）：类级 `CustomizeDetails(IDetailLayoutBuilder&)` 整面板重排；类型级 `CustomizeHeader/CustomizeChildren`；注册 `RegisterCustomClassLayout(ClassName, factory)` / `RegisterCustomPropertyTypeLayout(TypeName, factory)`。

**UE 文档工作台**（`AssetEditorToolkit.h`、`TabManager.h:824-891`）：`InitAssetEditor(mode, host, AppId, FLayout&, ...)`——资产编辑器 = AppId + 默认布局对象 + tab spawner 集 + 菜单/工具栏扩展器；布局声明式 `NewLayout(name)->AddArea(...)->AddTab(tab_id, state)`。

**Fyrox 字段容器**（`plugins/inspector/mod.rs:120-129`）：全局 `Arc<PropertyEditorDefinitionContainer>`——字段编辑器按类型注册一处、处处生效。

## 现状与证据（zircon，2026-07-05 实读）

### 注册表：13 张表 + 批模型已存在（v3 新核）

`EditorExtensionRegistry`（`editor_extension.rs`，全文 896 行）：13 张 `BTreeMap<String, *Descriptor>` 表 + `operations: EditorOperationRegistry`；**14 个 `register_*`**（:34-211，均 `Result<(), EditorExtensionRegistryError>`）+ 13 个 `Vec<&T>` 只读访问器（:213-265）。`EditorMenuItemDescriptor` 已带 `priority/shortcut/enabled/required_capabilities` builder（:393-431）。

**批模型已在**（v2 未记，本次核准）：`EditorExtensionRegistration`（:277-303）= `{ registry: EditorExtensionRegistry, required_capabilities: Vec<String> }` + `is_enabled_by(enabled_capabilities) -> bool`——插件按「整表批」贡献，能力门控在批级已有。`EditorEventRuntimeState.editor_extensions: Vec<EditorExtensionRegistration>` 即批列表。**真缺口由此精确化**：查询需逐批迭代无合并索引；无 revoke（批只增不减）；无 delta（工作台无从增量物化）。

### 描述符类型定义分居两文件

`editor_authoring_extension.rs`（474 行）是 8 类 authoring 描述符的**纯类型定义**：`AssetCreationTemplateDescriptor / ViewportToolModeDescriptor / GraphPinDescriptor / GraphNodeDescriptor / GraphNodePaletteDescriptor / GraphEditorDescriptor / TimelineTrackDescriptor / TimelineEditorDescriptor`——图/时间轴词汇已有雏形（07 直接消费）；其余描述符定义在 `editor_extension.rs` 内。

### 宿主与布局

`ViewHost` 四态（`view_host.rs:5-11`，serde 可持久化）：`Drawer(ActivityDrawerSlot) | Document(MainPageId, Vec<usize>) | FloatingWindow(MainPageId, Vec<usize>) | ExclusivePage(MainPageId)`。`LayoutPreset { name, drawer_states, size_overrides, center_split }` 四内建预设（Authoring/Review/Focus/Debug）；`persistence.rs:6-27` 四函数（`load/save_global_default`、`load/save_project_workspace`）**返回克隆零 IO**。

### inspector 反射面

`SnapshotBuilder { tree_id, next_id, nodes: BTreeMap<UiNodeId, UiNodeDescriptor> }`（`reflection/builder.rs:8-60`）自动面板生成在；`component_drawers` 是组件级雏形；**无字段类型级容器**。视图实体 8 个（`ui/layouts/views/`：animation_editor/asset_browser/console/hierarchy/inspector/preview_images/viewport_chrome/welcome），是否全经 `views` 表分派待执行时核验。

## 目标

1. **贡献生命周期统一**：批模型升级为 `ContributionStore`——保留描述符类型与批语义（`EditorExtensionRegistration` 即 `ContributionBatch` 前身），补齐**合并索引 + ticket/revoke + changed_since**；`ContributionSource::{Builtin, Plugin(PluginId)}` 前缀命名空间（12 同源）。
2. **贡献位枚举**：`WorkbenchSlot` 有限枚举（godot 12 位思路 × 现 `ViewHost/ActivityDrawerSlot` 实际区域）；`DrawerDescriptor/ViewDescriptor` 增 slot 声明。
3. **inspector 双层定制**：`InspectorCustomization`（类级责任链，`component_drawers` 迁为实例）+ `FieldEditorDefinition`（类型级容器，内建六类：数值/布尔/颜色/枚举/资产引用带 21-marker 过滤/曲线占位）；全 miss 回退 `SnapshotBuilder` 自动面板。
4. **`DocumentToolkit` 契约**（UE Toolkit 等价）：tab 工厂 + 默认布局声明 + 菜单贡献 + 脏态（03 `saved_top` 投影）+ 保存钩子 + history context 申请。
5. **预设可见性声明与布局落盘**：贡献声明 `default_presets`；`persistence.rs` 实 IO（11 格式 + 17 路径分层），与 editor_layout/04 会签。

## 非目标

- 不重做停靠引擎与外观（editor_layout/03、07）；插件 ABI 序列化贡献通道属 12（本计划提供物化目标 API）；图/时间轴内容属 07（本计划只保证四张表接入统一生命周期）。

## 架构设计

### 模块布局

```
zircon_editor/src/core/extension/
  mod.rs               # 薄声明
  store.rs             # ContributionStore：批 + 合并索引 + ticket/revoke/changed_since
  slots.rs             # WorkbenchSlot
  descriptors/
    mod.rs
    workbench.rs       # View/Drawer/MenuItem/UiTemplate（自 editor_extension.rs 迁入）
    assets.rs          # AssetImporter/AssetEditor/AssetCreationTemplate
    graph.rs           # GraphPin/GraphNode/GraphNodePalette/GraphEditor（自 editor_authoring_extension.rs 迁入）
    timeline.rs        # TimelineTrack/TimelineEditor（同上）
    viewport.rs        # ViewportToolMode
  inspector.rs         # InspectorCustomization + FieldEditorContainer
  toolkit.rs           # DocumentToolkit trait
```

两个源文件（896 + 474 行）目录化后删除；`register_operation/operations` 随 08 命令合一迁移（本计划过渡期原样搬 store 内）。

### 关键类型

```rust
// store.rs —— 批语义升级
pub struct ContributionBatch { /* EditorExtensionRegistry 更名收编：13 表 + required_capabilities */ }
pub struct ContributionTicket(u64);
pub enum ContributionSource { Builtin, Plugin(PluginId) }
impl ContributionStore {
    pub fn contribute(&mut self, src: ContributionSource, batch: ContributionBatch)
        -> Result<ContributionTicket, EditorExtensionRegistryError>;   // 错误类型沿用
    pub fn revoke(&mut self, ticket: ContributionTicket) -> RevokeReport;   // 12 热禁用消费
    pub fn changed_since(&self, generation: u64) -> ContributionDelta;      // 工作台增量物化
    // 合并查询面：现 13 个 Vec<&T> 访问器语义保留，但跨批合并 + 能力过滤后返回
    pub fn views(&self, enabled_capabilities: &CapabilitySet) -> impl Iterator<Item = &ViewDescriptor>;
    // ……逐表同型
}
```

合并索引维护规则：contribute/revoke 时增量更新 `BTreeMap<String, (ContributionTicket, 表内值)>`（id 冲突=Err，同 godot 命名空间纪律：Plugin 源强制 `plugin.<id>.` 前缀）；`is_enabled_by` 既有批级门控保留为查询期过滤。

```rust
// inspector.rs
pub trait InspectorCustomization: Send + Sync {
    fn can_handle(&self, target: &InspectTargetType) -> bool;
    fn build(&self, target: &InspectTarget, b: &mut InspectorLayoutBuilder);
}
pub struct FieldEditorDefinition {
    pub type_name: &'static str,                            // 反射全限定名（02 同口径）
    pub make: fn(FieldEditorInit) -> FieldEditorInstance,
}
// 责任链：注册序问询，首 true 截获；全 false → SnapshotBuilder 自动面板

// toolkit.rs
pub trait DocumentToolkit: Send {
    fn document_id(&self) -> DocumentId;
    fn title(&self) -> String;
    fn default_layout(&self) -> ToolkitLayout;              // 声明式：区域→tab（TabManager 形状）
    fn contribute_menus(&self, b: &mut MenuContributionBuilder);  // 08 消费
    fn history_context(&self) -> HistoryContextId;          // 03 路由
    fn is_dirty(&self) -> bool;                             // 03 saved_top 投影
    fn save(&mut self, ctx: &mut SaveCtx) -> Result<(), SaveError>;  // 09 钩子
}
```

### 迁移映射（执行合同）

| 现物 | 去向 |
| --- | --- |
| `editor_extension.rs`（896 行）+ `editor_authoring_extension.rs`（474 行） | `core/extension/` 目录；两文件删除 |
| `EditorExtensionRegistry` + 14 `register_*` | `ContributionBatch` + `contribute`；调用方迁移后删旧 API（Grep `register_view\|register_drawer\|EditorExtensionRegistry` 清点入状态节） |
| `EditorExtensionRegistration{registry, required_capabilities}` + `is_enabled_by` | `ContributionBatch` 本体 + store 查询期能力过滤 |
| `EditorEventRuntimeState.editor_extensions: Vec<_>` | store 批列表（01 M1 批 4 协同） |
| `component_drawers` 表 | `InspectorCustomization` 实例族（表删除） |
| `ViewHost/ActivityDrawerSlot` | 保留为运行位；`WorkbenchSlot` 是声明位，映射在 workbench 物化器 |
| `persistence.rs` 四空函数 | 实 IO（签名保留：global→17 User 目录、project→`.zircon/`；格式 11 契约） |

### 深度测试

夹具插件源投一批（view+drawer+field editor+toolkit）→ 物化 → revoke → 工作台回收，store 外零改动；`changed_since` 增量断言（二次 contribute 只物化 delta）；能力门控矩阵（批级 required_capabilities 缺失时全批不可见——既有 `is_enabled_by` 行为保持）。

## 里程碑

### M1 ContributionStore 与表收拢

- 切片 1.1：`core/extension/` 目录化（描述符原样迁移分文件）；store + ticket/revoke/changed_since + 合并索引；`register_*` 调用方全量迁移删旧 API。
- 切片 1.2：`WorkbenchSlot` 定稿（editor_layout/03 区域名会签）；`editor_extensions` 批列表收编（配 01 M1 批 4）。
- 测试阶段：`cargo test -p zircon_editor --lib --locked`（extension 既有注册测试迁移后须过 + 生命周期/命名空间冲突/revoke 回收/能力过滤矩阵）。更新 `docs/zircon_editor/core/extension.md`。

### M2 Inspector 双层定制

- 切片 2.1：`FieldEditorContainer` + 内建六类；`SnapshotBuilder` 渲染路径改查容器、miss 回退自动行。
- 切片 2.2：`InspectorCustomization` 责任链 + `component_drawers` 迁移删除；多选批量改值（05 多选集 + 03 同事务多 push）。
- 测试阶段：reflection 既有测试 + 责任链命中/回退矩阵 + 字段编辑往返（编辑→事务→撤销→UI 值还原）+ 多选批量改值撤销为单条历史。

### M3 DocumentToolkit 与布局落盘

- 切片 3.1：toolkit trait + workbench 物化（`ToolkitLayout`→`ViewHost::Document/ExclusivePage`）；asset_editor 先行迁实例（animation/material 随 07）。
- 切片 3.2：`default_presets` 声明 + `persistence.rs` 实 IO + 预设往返。
- 测试阶段：toolkit 开闭/脏态/保存钩子生命周期；布局保存→重载逐字段等价（补现空实现的测试缺口）；editor_layout/04 口径对账记状态节。

## 风险与开放问题

- 调用方迁移面：extension 注册散布 host/workbench/plugins，Grep 清点超 80 处则按描述符族分三步硬切（仍在 M1 内）。
- `FieldEditorDefinition` 以反射类型名为 key，依赖 runtime 反射名稳定性——类型改名需 11 迁移链同步，契约注释显式声明。
- `ExclusivePage` 是否也是 toolkit：倾向是（欢迎页/设置页=无资产 toolkit），M3 定稿记状态节。
- `register_operation/operations` 在 store 内的过渡存放与 08 合一注册表的迁移时点：08 M1 先行则直接跳过过渡；排程冲突时 store 保留 operations 字段一个里程碑并记债。
