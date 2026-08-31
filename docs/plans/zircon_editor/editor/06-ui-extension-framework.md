---
related_code:
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/core/editor_authoring_extension.rs
  - zircon_editor/src/ui/workbench/mod.rs
  - zircon_editor/src/ui/workbench/view/view_host.rs
  - zircon_editor/src/ui/host/layout_persistence.rs
  - zircon_editor/src/tests/workbench/layout/layout_preset_persistence.rs
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
status: in_progress
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

`editor_authoring_extension.rs` 是 authoring 描述符的纯类型定义入口之一，其中 scene mode 元数据已硬切为 `SceneModeDescriptor`；可执行行为由 `SceneModeRegistration` 的 factory 单独持有并通过 `register_scene_mode` 注册，不保留 descriptor-only tool mode 接口。图/时间轴词汇由 07 直接消费，其余描述符定义在 `editor_extension.rs` 内。

### 宿主与布局

`ViewHost` 四态（`view_host.rs:5-11`，serde 可持久化）：`Drawer(ActivityDrawerSlot) | Document(MainPageId, Vec<usize>) | FloatingWindow(MainPageId, Vec<usize>) | ExclusivePage(MainPageId)`。`LayoutPreset { name, drawer_states, size_overrides, center_split }` 四内建预设（Authoring/Review/Focus/Debug）；持久化已硬切到 `ui/host/layout_persistence.rs`，经配置与项目 layout asset 保存/恢复全局、preset 和按 `(user_id, page_id)` 作用域的版本化布局。

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
| 已删除的 `ui/workbench/layout/manager/persistence.rs` 空实现 | `ui/host/layout_persistence.rs` 的真实配置/项目资产持久化；`LayoutPresetPersistenceStore` 按用户/页面作用域版本化恢复，失配回退 Authoring |

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
- 切片 3.2：`default_presets` 声明 + `ui/host/layout_persistence.rs` 实 IO + 预设往返（实现与 focused 验收由 editor_layout/04 owner 收敛）。
- 测试阶段：toolkit 开闭/脏态/保存钩子生命周期；布局保存→重载逐字段等价（补现空实现的测试缺口）；editor_layout/04 口径对账记状态节。

## 风险与开放问题

- 调用方迁移面：extension 注册散布 host/workbench/plugins，Grep 清点超 80 处则按描述符族分三步硬切（仍在 M1 内）。
- `FieldEditorDefinition` 以反射类型名为 key，依赖 runtime 反射名稳定性——类型改名需 11 迁移链同步，契约注释显式声明。
- `ExclusivePage` 是否也是 toolkit：倾向是（欢迎页/设置页=无资产 toolkit），M3 定稿记状态节。
- `register_operation/operations` 在 store 内的过渡存放与 08 合一注册表的迁移时点：08 M1 先行则直接跳过过渡；排程冲突时 store 保留 operations 字段一个里程碑并记债。
- 2026-07-22 Workbench menu/control性能交接：PERF-MVP-560已把responsive toolbar约39×全tree control scan止损为单次借用HashMap index，静态合同1/1。asset creation menu仍每layout重建labels/map/set/String，单action点击又重建整map；Editor06联动Editor09发布template+asset generation的compiled action/control slots，见[open failure](06/failure-2026-07-22-workbench-menu-control-generation.md)。
- 2026-07-22 Workbench test反查补充：PERF-MVP-128已让componentized template surface构造期一次建立`control_id→UiNodeId`，required frame与visible frame不再每项全tree scan，Editor06源码合同2/2。remaining `RetainedUiHostModel/Projection`与动态virtual node必须由EditorUI01/08的surface generation owner维护duplicate-aware索引；禁止插件/bridge缓存跨generation node id。

## Code Review 状态 (2026-08-02)

- 上次 review 所述 `ContributionStore`、ticket、`revoke`、`changed_since` 未落地的观察已过期：这些能力现在由 `core/extension/store/` 的 `ContributionStore`、批模型和 immutable snapshot 统一持有；旧 `EditorExtensionRegistry` materializer 的删除仍由 [plugin ticket revoke failure](06/failure-2026-07-28-plugin-contribution-ticket-revoke-contract.md) 中的 Editor12 修复责任计划追踪。
- `core/extension` 当前生产 owner 已按 Store snapshot、ticket lifecycle、field editor 与 DocumentToolkit 子域拆分。2026-08-24 再将 `store/model.rs` 的 records 与状态机硬切为命名子 owner；Store production leaf 最大为 645 行，不保留平行 Store、alias 或 compatibility fallback。
- 已删除的 `ui/workbench/layout/manager/persistence.rs` 不再是 M3.2 owner：真实 I/O 由 `ui/host/layout_persistence.rs` 收敛，按全局配置、项目 preset asset 及 `(user_id, page_id)` 版本化 store 保存/恢复；`layout_preset_persistence` focused 2/2 已由 editor_layout/04 记录。Editor06 只消费该结果，不复建旧路径。当前五项 failure 仍维持 `open`，受管 Cargo/性能证据取得 durable receipt 前不得标记 accepted 或 `fixed`。

## 产出记录与时间

| 日期 | 项目 | 状态 | 证据 |
| --- | --- | --- | --- |
| 2026-08-02 | M1 `ContributionStore` 快照模型与 UI template replacement 原子性 | core_contract_complete / hard_cut_dependency_open / managed_validation_pending | `store/model/snapshot.rs` 承担 immutable snapshot/index；replacement 在发布 generation/snapshot 前返回 `UnknownTicket`，无已发布部分状态。`changed_since` 现仅保留 4096 条变更，过期或未来 cursor 返回 reset，消费者必须重建 immutable snapshot，避免长生命周期插件 reload 无界累积。`store/batch.rs` 仍引用旧 `core/editor_extension` 的 view/drawer 描述符，且 Host 尚未从 Store snapshot 可撤销物化；完整 hard cut 继续由 plugin ticket revoke failure 的 Editor12 上游迁移处理。静态合同、格式检查与独立二次审查（0/0/0）均已完成；受管 Cargo 尚无 durable receipt。 |
| 2026-08-02 | M2 ticket-owned `FieldEditorContainer`、inspector 注册与多选 batch mutation | core_contract_complete / second_review_complete_0_0_0 / managed_validation_pending | field editor 已拆至 `core/extension/inspector/field_editor.rs`，按 ticket 归属和精确 type key 解析；多选变更走单个 disable-merge transaction。`InspectorCustomizationDescriptor` 现在发布前校验 target 与 retained surface；责任链也对任意自定义实现复查 `.zui`、可选 template/data root 与 operation binding，`ContributionBatch` 保留动态 trait 防线，三者均在发布前拒绝无效声明。跨 ticket field-editor duplicate fixture 现以同一 `BrandColor` key 验证原子拒绝。静态合同与格式检查通过，独立二次审查为 `0/0/0`；受管 Cargo 尚无 durable receipt。 |
| 2026-08-02 | M3 `DocumentToolkit` 保存钩子与 workbench generation hot path | core_contract_complete / second_review_complete_0_0_0 / managed_validation_pending | toolkit save 生命周期、公开 `CloseView` 的 document close lease 收束与 workbench menu/control generation 静态合同通过；同 normalized label 的 collision suffix 改为增量索引，避免二次扫描。两轮独立静态审查均为 `0/0/0`；受管性能/Cargo evidence 尚待 coordinator wakeup。 |
| 2026-08-02 | M3.2 layout persistence owner hard cut | implemented-focused-passed / upstream broader validation pending | 已删除的 workbench manager `persistence.rs` 不再保留；`ui/host/layout_persistence.rs` 通过配置和项目 asset 保存全局、preset 与按用户/页面作用域的版本化 layout，`layout_preset_persistence` focused 2/2 由 editor_layout/04 记录。Editor06 仅消费该 owner。 |
| 2026-08-02 | 失败交接状态 | open | [DocumentToolkit save hook](06/failure-2026-07-22-document-toolkit-save-hook-contract-missing.md)、[workbench generation](06/failure-2026-07-22-workbench-menu-control-generation.md)、[plugin ticket revoke](06/failure-2026-07-28-plugin-contribution-ticket-revoke-contract.md)、[inspector multi-selection](06/failure-2026-08-01-inspector-multi-selection-batch-mutation-missing.md) 与 [ticket-owned field editor](06/failure-2026-08-02-ticket-owned-field-editor-container-missing.md) 均保持 `open`；Editor12 的 legacy materializer hard-cut 仍是 plugin ticket revoke 的跨计划依赖。 |
| 2026-08-24 CST | M1 `ContributionStore` records/state-machine owner hard cut | source_complete_static_green / independent_second_review_pending / managed_validation_pending | 为收敛 `engine-code-structure-convention.md` 的生产 owner budget，844 行 `store/model.rs` 已硬切为 13 行 facade、`records.rs`（capability/source/ticket/change journal identity）和 `contribution_store.rs`（唯一 generation、immutable snapshot publish、template replacement、revoke 与 delta replay 状态机）；既有 `lifecycle.rs` 与 `snapshot.rs` 继续承担 namespace/error 和 immutable index 查询。`store.rs` 仍导出同一 12 项公共 surface，`ContributionStore` 10 个 public/`pub(crate)` 方法签名、18 个 contribution publication、18 个 collision checks、20 个 removal calls 与 3 个 generation advances 均与拆分前一致；未保留旧实现、parallel store、alias 或 compatibility fallback。限定 rustfmt、public surface/mutation-count 静态清点、scoped diff whitespace 通过；未运行 Cargo、产品插件 reload 或性能 capture，未声明 ticket/revoke 集成闭环或性能结论。 |
| 2026-08-28 CST | M3.2 layout persistence 统一版本壳硬切 | implementation_complete / isolated_contract_7_of_7 / full_editor_blocked | Plan11 layout 子面已把 global default、named presets、user/page presets、project workspace 与 project preset asset 收敛到五个独立 `$zircon` v1 schema；删除裸 JSON reader、内嵌 `format_version/layout_version` 和逐条 preset version，工程 writer 使用 borrowed payload + atomic write。D 盘实际源码隔离合同 7/7；完整 editor check 被当前 `zircon_runtime_interface/ui/text/model_update.rs` 4 个无关 E0609 阻断。完整状态、性能边界与未完成项见 [Plan11 子计划](11/2026-08-28-workbench-layout-version-shell-hard-cut.md)。 |
