---
related_code:
  - zircon_runtime_interface/src/ui/widget.rs
  - zircon_runtime_interface/src/ui/component/descriptor/component_descriptor.rs
  - zircon_runtime_interface/src/ui/layout/slot.rs
  - zircon_runtime_interface/src/ui/template/asset/prototype.rs
  - zircon_runtime/src/ui/template/instance.rs
  - zircon_editor/src/core/editor_message/mod.rs
design_references:
  - docs/ui-and-layout/ai-workbench-style/component-prototype/web-native-handoff-matrix.md
  - docs/ui-and-layout/editor-workbench-designs/STYLE-NOTES.md
plan_sources:
  - docs/plans/zircon_editor/editor_layout/10-real-rendering-pipeline-and-contract.md
  - docs/plans/zircon_editor/editor_layout/11-data-binding-and-reactive-contract.md
  - docs/plans/zircon_editor/editor_layout/12-widget-slot-componentization.md
  - docs/plans/zircon_editor/editor_layout/13-taffy-css-constraint-language.md
status: planned
---
# 14 Unreal + React 组件思想综述(统领 10–13 的心智模型)

## 1. 目标

这是 10–13 的**统领性思想文档**:把 `dev/UnrealEngine`(Slate/UMG)的 **widget + slot + 失效驱动重绘**思想,与 **React** 的 **组件化 + 单向数据流 + 受控组件 + composition** 思想,提炼为一套**适配 zircon 既有架构**的统一心智模型,并明确每条思想落到 10/11/12/13 的哪个具体规范。本计划**不产出新代码模块**,只产出心智模型文档 + 把四思想对齐到既有 DTO 的映射表,作为 10–13 实现时的"为什么这么定"的权威依据。

> 取舍原则:**取思想,不取运行时**。不照搬 Slate 的 C++ 对象模型,也不引入 React 的虚拟 DOM/运行时 diff。zircon 已有 retained 模板 + Taffy + 绑定表达式 + 09 增量脏集,这些思想必须落到既有 seam 上,不另起炉灶。

## 2. 四条核心思想与 zircon 落点

### 2.1 Widget = 自描述可视单元(Unreal SWidget / React Component)

| 思想 | 来源 | zircon 落点 | 计划 |
| --- | --- | --- | --- |
| 界面由自包含、可复用的 widget 组成 | Slate `SWidget` / React Component | `UiWidgetBehavior` + `UiComponentPrototype`,组件三层(原子/组合/区域) | 12 |
| widget 暴露参数/属性 | UMG `UWidget` props / React props | `UiComponentParamSchema` + prop 契约 + `$param` 解析 | 12 + 11 |
| widget 无内部业务状态(尽量受控) | React 受控组件 | 单向数据流 + view 不持真状态 | 11 |

### 2.2 Slot = 组合与嵌套契约(Unreal FSlot / React children)

| 思想 | 来源 | zircon 落点 | 计划 |
| --- | --- | --- | --- |
| 父 widget 通过具名 slot 接纳子内容 | Slate `FSlot` named slot / React named children | `UiSlotKind` + `UiSlotSchema` + slot 契约(accepts/required/multiple) | 12 |
| slot 决定子节点排布算法 | Slate `SBoxPanel`/`SGridPanel` slot | `UiSlotKind` ↔ Taffy family 映射 | 13 |
| 组合优于继承,差异靠填不同 slot | React composition | "新页面不新增 primitive,靠 slot 填充表达差异" | 12 + 05 |

### 2.3 单向数据流 + 受控(React state→props / Unreal property binding)

| 思想 | 来源 | zircon 落点 | 计划 |
| --- | --- | --- | --- |
| 数据单向 state→view,view 不直接改 state | React 单向流 | EditorState → view-model 派生 → 绑定 → view;写回走事件→命令 | 11 |
| 受控组件:值来自上层,改动上报 | React controlled input | `UiComponentEventEnvelope` → 09 命令,不直接 mutate | 11 + 09 |
| 派生纯函数 + 只读投影 | React derived state / selector | view-model 纯函数派生 + `ProjectionPatch` | 11 |

### 2.4 失效驱动的增量更新(Unreal FastUpdate / React reconciliation 的"只更新变化")

| 思想 | 来源 | zircon 落点 | 计划 |
| --- | --- | --- | --- |
| 只重算/重绘变化的部分 | Slate invalidation panel(FastUpdate)/ React 只 re-render 变化子树 | 09 视图脏集 + 11 绑定级脏依赖 + 10 脏视图增量提取 | 09 + 11 + 10 |
| 字段级变更通知 | UMG `FieldNotification` | 绑定依赖图:数据键变 → 只刷依赖绑定 | 11 |
| 帧末批合并刷新 | Slate deferred paint | 09 帧末 drain 脏集 + 10 批次合并 | 09 + 10 |

## 3. 统一心智模型(一张图串起 10–13)

```
              ┌─────────────────── 编辑器状态(single source of truth)──────────────────┐
              │                                                                          │
        (11) 派生纯函数                                                          (11) 写回:事件→命令
              ▼                                                                          ▲
   view-model 只读投影 ──(11) 绑定 $prop/$param 求值──▶ widget 树(12 组件三层 + slot)    │
              │                                              │                            │
        (01) $token 解析                                (12) slot 决定容器                │
              ▼                                              ▼                            │
   设计 token(色/尺寸/密度)─────────────────▶ (13) 类 CSS 约束 → Taffy 求解 → 排布几何   │
              │                                              │                            │
              └──────────────▶ (10) RESOLVE→EXTRACT→COMMAND→BATCH→上屏 ◀──────────────────┘
                                            ▲
                                  (09) 视图脏集 + (11) 绑定脏依赖 → 只重算/重提取脏部分
```

- **横轴(组合)**:widget(12)+ slot(12)+ 类 CSS 约束(13)+ token(01)= 界面**怎么长出来**。
- **纵轴(数据)**:state → 派生 → 绑定 → view → 事件 → 命令 → state(11)= 数据**怎么单向流动**。
- **增量(性能)**:09 视图脏 + 11 绑定脏 + 10 增量提取 = 复杂编辑器**为什么不卡**。

## 4. 设计原则(10–13 实现时的硬约束总纲)

1. **取思想不取运行时**:不引入虚拟 DOM、不照搬 Slate C++ 对象图;落到既有 retained 模板 + Taffy + 绑定表达式 + 09 脏集。
2. **组合优于继承**:页面差异用 slot 填充表达(12),不为每页造控件;不新增 primitive 除非新原子能力。
3. **单向受控**:数据 state→view 单向(11),view 改动一律走事件→命令,禁止 view 侧 mutate state/view-model。
4. **增量即默认**:任何变更产生最小脏集(09 视图级 + 11 绑定级),只重算/重提取(10)脏部分;无全量刷新常规入口。
5. **token 单源**:色/尺寸/密度全部 token 化(01),chrome 资产三处(资产扫描/13 约束/10 渲染)一致禁裸值。
6. **契约可校验**:slot/prop 契约(12)、绑定方向(11)、渲染禁用视觉(10)、约束 family(13)都落为可测试的 guard,不靠自觉。

## 5. dev/ 源码思想锚点(详读清单)

| 思想维度 | Unreal 锚点 | React/Web 锚点 |
| --- | --- | --- |
| widget 自描述 | `SlateCore/Public/Widgets/SWidget.h` | `dev/material-ui` component 封装 |
| slot 组合 | `SlateCore/Public/Layout`(`FSlot`/`SBoxPanel`/`SGridPanel`) | React children / slots(material-ui `List`/`Card`) |
| property 绑定 | `UMG/Public/Binding` + `Binding/States` + `FieldNotification` | React 受控组件 + selector |
| 失效增量 | `SlateCore/Public/FastUpdate`(invalidation) | React reconciliation(只更新变化) |
| 派生/依赖追踪 | — | `dev/theatre/packages/dataverse`、`dev/slint/internal/core/properties.rs` |

## 6. 与子计划的关系(本文不重复实现)

| 思想 | 由哪个子计划实现 |
| --- | --- |
| 组件化 + slot | 12 |
| 类 CSS 约束 + Taffy | 13 |
| 单向数据流 + 绑定增量 | 11 |
| 真实渲染 + 增量提取 | 10 |
| 视图级增量总线 | 09 |
| token 单源 | 01 |

本文是**为什么**;10–13/09/01 是**怎么做**。实现细节、切片、验证全在子计划,本文不重复。

## 7. 完成定义

统一心智模型成文;四思想(widget/slot/单向受控/失效增量)逐条对齐到既有 DTO 与子计划;作为 10–13 实现的设计依据被各子计划 frontmatter 反向引用。

## 8. 边界约束

不产出新代码模块(纯思想 + 映射文档);不引入虚拟 DOM/运行时整树 diff;不照搬 Slate 对象模型;一切落到既有 seam。

## 9. 状态与产出记录

planned。后续项:把本文的设计原则 §4 抽为各子计划共享的验收前言,并在 `docs/ui-and-layout/` 落一份 `composition-thesis.md` 摘要。
