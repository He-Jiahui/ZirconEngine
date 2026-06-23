---
related_code:
  - zircon_runtime_interface/src/ui/layout/slot.rs
  - zircon_runtime_interface/src/ui/component/descriptor/slot_schema.rs
  - zircon_runtime_interface/src/ui/component/descriptor/prop_schema.rs
  - zircon_runtime_interface/src/ui/component/descriptor/component_descriptor.rs
  - zircon_runtime_interface/src/ui/component/descriptor/component_model.rs
  - zircon_runtime_interface/src/ui/widget.rs
  - zircon_runtime_interface/src/ui/template/asset/prototype.rs
  - zircon_runtime/src/ui/template/instance.rs
  - zircon_runtime/src/ui/template/asset/compiler/prototype_instancer.rs
  - zircon_editor/assets/ui/editor/components
design_references:
  - docs/ui-and-layout/ai-workbench-style/component-prototype/web-native-handoff-matrix.md
  - docs/ui-and-layout/editor-workbench-designs/STYLE-NOTES.md
plan_sources:
  - docs/plans/zircon_editor/editor_layout/02-declarative-layout-interface.md
  - docs/plans/zircon_editor/editor_layout/05-page-layout-templates.md
  - docs/plans/zircon_editor/editor_layout/11-data-binding-and-reactive-contract.md
status: planned
---
# 12 Widget / Slot 组件化与泛用化规范

## 1. 目标

把"编辑器界面由**可复用、可组合、泛用化的 widget 组件**拼装,组件之间通过 **slot(插槽)** 嵌套填充"沉淀为一份**组件化规范**。借鉴 Unreal Slate 的 `SWidget + FSlot` 与 React 的 `children`/composition,但落到 zircon 既有的 `UiSlotKind` / `UiSlotSchema` / 组件 prototype + slot 填充上。目标:编辑器作者用**少量泛用组件 + slot 嵌套**拼出 13 个页面,而不是为每页造一套专用控件。本计划只定**组件/插槽契约与组件目录规范**,不改 prototype 实例化器内部(已存在)。

## 2. 现状(按代码核实)

### 2.1 已存在的设施(组件/插槽模型已成立,不重做)

| 能力 | 落点 | 证据 |
| --- | --- | --- |
| 插槽种类 | `iface .../layout/slot.rs` | `UiSlotKind`:Free / Container / Overlay / Linear / Grid / Flow / Canvas / Scrollable / Splitter / Scale |
| 插槽 schema | `iface .../component/descriptor/slot_schema.rs` | `UiSlotSchema`:name / required / multiple |
| 插槽放置 | `iface .../layout/slot.rs` | `UiCanvasSlotPlacement`(anchor+pivot)、`UiGridSlotPlacement`、`UiMargin`、`UiAlignment2D` |
| 组件描述 | `iface .../component/descriptor/component_descriptor.rs` / `component_model.rs` | 组件身份 + 模型 |
| 组件参数 | `iface .../component/descriptor/prop_schema.rs` | `UiComponentParamSchema`(实例化入参) |
| widget 行为 | `iface .../ui/widget.rs` | `UiWidgetEvent`、`UiWidgetEventSource`、`UiWidgetBehavior` |
| 组件 prototype | `iface .../template/asset/prototype.rs` | `UiComponentPrototype`:root / style_scope / contract / params / slots |
| slot 填充展开 | `runtime .../template/instance.rs` / `prototype_instancer.rs` | `expand_node()` 解析组件边界 + slot 内容挂载;`UiTemplateNode{slots: Map<name, Vec<UiTemplateNode>>}` |
| 编辑器组件物料 | `zircon_editor/assets/ui/editor/components` | material_foundation catalog + workbench primitives |

### 2.2 真实缺口

- 缺**编辑器组件分层与泛用化规范**:已有 primitive 物料,但没固化"primitive(原子)→ composite(组合)→ region/panel(区域级)"的分层与复用规则,易出现"每页各造控件"。
- 缺**slot 契约规范**:`UiSlotSchema` 有 required/multiple,但没把"哪类组件暴露哪些命名 slot、slot 接受什么"写成可校验契约(类似 Slate 的 named slot / React 的 named children)。
- 缺**组件 prop 契约与默认值规范**:`UiComponentParamSchema` 存在,但缺"编辑器组件 prop 必须有类型 + 默认 + token 关联"的规范,接 11 的 `$param` 解析。
- 缺**组件目录(catalog)作为布局物料的索引**:02/05 声明区域填充时按资产路径引用,缺一个"泛用组件清单 + slot 形态"目录,供作者按能力选组件而非按路径硬找。

## 3. 设计

### 3.1 组件三层(原子 → 组合 → 区域)

| 层 | 定义 | 例 | 复用规则 |
| --- | --- | --- | --- |
| **Primitive(原子)** | 不可再拆的最小可视/可交互单元,无业务语义 | Button / Label / Icon / Field / Row / Divider / ScrollArea | 全编辑器共享,只受 01 token 驱动 |
| **Composite(组合)** | primitive 经 slot 组合的可复用块,弱业务语义 | Toolbar / ListRow / PropertyField / TabStrip / TreeItem | 跨页面复用,slot 暴露给上层填内容 |
| **Region/Panel(区域级)** | 填进 03 骨架六区域的面板,装 composite | SceneTreePanel / InspectorPanel / ConsolePanel | 经 02 区域绑定入槽,内部全用上两层 |

硬规则:**新页面不得新增 primitive**(除非确属新原子能力);页面差异通过"composite 的 slot 填不同内容"表达(React composition 思想)。

### 3.2 Slot 契约(命名插槽 + 接受约束)

每个 composite/panel 组件声明它暴露的命名 slot 及接受约束(基于 `UiSlotSchema`):

```
组件: PropertyField
  slot "label"   : required, single, accepts {Label}
  slot "editor"  : required, single, accepts {Field|Dropdown|Slider|Toggle}
  slot "actions" : optional, multiple, accepts {Button|Icon}
```

- slot 种类用 `UiSlotKind`(Linear/Grid/Overlay…)决定子节点排布算法(接 13 Taffy)。
- 填充期(`prototype_instancer::expand_node`)校验:required slot 必填、single/multiple 数量、accepts 类型——类似 Slate named slot 的契约,杜绝乱塞。

### 3.3 Prop 契约(类型 + 默认 + token 关联)

每个组件 prop 声明类型、默认值、可选的 token 关联(接 11 `$param` 与 01 token):

```
组件: Button
  prop "label"    : Text,    default ""
  prop "variant"  : enum{Primary|Ghost|Danger}, default Ghost
  prop "icon"     : Option<IconRef>, default None
  prop "height"   : Dimension, default $control.height   // 关联 01 token
```

prop 求值走 11 的 `$` 分流:token 关联走 01,实例入参走 `ParamRef`。

### 3.4 组件目录(catalog)= 布局物料索引

把泛用组件清单做成目录资产,记录每组件:身份、层级、暴露 slot、prop 契约、默认 token。02/05 声明布局时按**能力**选组件(如"要一个属性字段" → `PropertyField`),作者不必硬记资产路径;catalog 同时作为"是否在重复造组件"的去重闸。

### 3.5 与既有物料的关系

现有 `components/workbench/primitives` 归入 Primitive 层并补 slot/prop 契约;`modules`/`shell` 归 Region/Panel 层;缺的 Composite 层按需补。不重造已有资产(对照 `web-native-handoff-matrix.md` 清单)。

## 4. 接口与数据结构草案(Rust)

```rust
// 组件层级标注
pub enum EditorComponentTier { Primitive, Composite, RegionPanel }
// slot 契约(在 UiSlotSchema 之上加 accepts)
pub struct EditorSlotContract {
    pub schema: UiSlotSchema,                 // name/required/multiple(既有)
    pub kind: UiSlotKind,                     // 排布算法(接 13)
    pub accepts: SlotAcceptSet,               // 接受的组件种类
}
// prop 契约
pub struct EditorPropContract {
    pub name: String,
    pub ty: EditorPropType,
    pub default: EditorPropDefault,           // 字面量 | $token 关联(接 01/11)
}
// 组件目录条目
pub struct EditorComponentCatalogEntry {
    pub id: EditorComponentId,
    pub tier: EditorComponentTier,
    pub slots: Vec<EditorSlotContract>,
    pub props: Vec<EditorPropContract>,
}
pub fn validate_slot_fill(entry: &EditorComponentCatalogEntry, fills: &SlotFills) -> Result<(), SlotFillError>;
```

## 5. 模块与文件落点

| 动作 | 文件 | 说明 |
| --- | --- | --- |
| 新增 | `zircon_editor/src/ui/workbench/component_catalog/mod.rs` | 组件层级 + slot/prop 契约 + 目录 |
| 新增 | `zircon_editor/assets/ui/editor/components/catalog.v2.ui.toml` | 泛用组件目录(物料索引) |
| 修改 | `prototype_instancer.rs`(只读校验,不改展开逻辑) | slot 填充期接 `validate_slot_fill` |
| 新增 | `docs/ui-and-layout/component-composition-contract.md` | 三层 + slot/prop 契约规范 |

## 6. 里程碑切片化

| # | 切片 | 涉及文件 | 验证命令 | 硬切换 |
| -- | --- | --- | --- | --- |
| S1 | 组件三层 + slot/prop 契约 + 目录骨架 | component_catalog/mod.rs / catalog.v2.ui.toml / component-composition-contract.md | `cargo test -p zircon_editor --lib component_catalog --locked` | 新建 |
| S2 | slot 填充校验接入 + 现有物料归层补契约 | prototype_instancer.rs / 现有 components 资产 | `cargo test -p zircon_editor --lib --locked` | 现有组件补 slot/prop 契约,去重重复控件 |

## 7. 测试矩阵

- required slot 缺填 / 数量违反 single|multiple / accepts 类型不符时,`validate_slot_fill` 报错。
- prop 默认值 token 关联解析为 01 token 值。
- 同一 composite 不同 slot 填充产出不同页面外观(复用证明)。
- catalog 去重:新增页面未引入新 primitive(扫描证明)。
- 组件三层职责不串(primitive 不含业务、panel 不直接画原子)。

## 8. 风险与对策

- 风险:slot accepts 过严挡住合理组合。对策:accepts 支持"类别集"而非具体类型,留 `Any` 逃生(仅 center 自由区组件用)。
- 风险:归层时大改既有资产引发回退。对策:S2 只加契约元数据 + 校验,不改组件视觉;逐资产族迁移。

## 9. 完成定义

编辑器组件三层清晰;slot/prop 契约可校验;组件目录作为布局物料索引;新页面靠 slot 组合而非新增控件;现有物料归层补契约。

## 10. 边界约束

不改 `prototype_instancer` 展开算法(只加校验);slot 排布算法归 13;prop token 解析归 01/11;不重造已有组件资产。

## 11. 参考实现对照(dev/ 源码锚点)

- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Widgets`(`SWidget`)+ `Layout`(`FSlot`、`SBoxPanel`):widget + named slot 组合 + slot 填充契约样板。
- `dev/UnrealEngine/.../UMG/Public/Components`:UMG `UWidget`/`UPanelWidget` 的 slot 化封装(把 Slate slot 暴露给作者)参考。
- `dev/slint/internal/compiler/widgets`(common/fluent/cupertino…):同一组件库按 style 复用、slot/children 组合参考。
- `dev/material-ui/packages/mui-material/src`(如 `List`/`ListItem`/`Card`):composite 经 children/slots 组合的 React 样板(取 composition 理念)。

## 12. 状态与产出记录

planned。后续项:S1 组件三层 + slot/prop 契约 + 目录骨架。
