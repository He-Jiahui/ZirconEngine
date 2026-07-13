---
related_code:
  - zircon_runtime_interface/src/ui/template/asset/binding/expression.rs
  - zircon_runtime_interface/src/ui/component/data_binding/mod.rs
  - zircon_runtime_interface/src/ui/component/data_binding/binding_target.rs
  - zircon_runtime_interface/src/ui/component/data_binding/data_source.rs
  - zircon_runtime_interface/src/ui/component/data_binding/projection_patch.rs
  - zircon_runtime_interface/src/ui/component/data_binding/event_envelope.rs
  - zircon_runtime/src/ui/binding/mod.rs
  - zircon_runtime/src/ui/binding/router.rs
  - zircon_runtime/src/ui/binding/update_report.rs
  - zircon_editor/src/core/editor_message/view_dirty_set.rs
  - zircon_editor/src/core/editor_event/service/state.rs
design_references:
  - docs/ui-and-layout/ai-workbench-style/component-prototype/web-native-handoff-matrix.md
plan_sources:
  - docs/plans/zircon_editor/editor_layout/09-incremental-message-bus-and-refresh.md
  - docs/plans/zircon_editor/editor_layout/01-design-tokens-and-language-contract.md
  - docs/plans/zircon_editor/editor_layout/08-plugin-page-interface-and-messaging.md
status: planned
---
# 11 数据绑定与响应式刷新规范(token / 表达式 / view-model / 增量)

## 1. 目标

把"`.zui` 资产里的 `$--token`/`$prop`/`$param` 怎么解析、面板内容怎么绑定到编辑器状态、状态变了怎么**只刷新受影响绑定**"沉淀为一份**数据绑定规范**。借鉴 React 的**单向数据流 + 受控组件 + 派生(derived)**思想,但落到 zircon 既有的绑定表达式 + 组件数据源 + 09 增量脏集上,**不引入运行时虚拟 DOM**,绑定求值确定且可缓存。本计划只定**绑定语义与刷新契约**,不改绑定表达式解析器内部(已存在)。

## 2. 现状(按代码核实)

### 2.1 已存在的设施(绑定地基已成立,不重做)

| 能力 | 落点 | 证据 |
| --- | --- | --- |
| 绑定表达式 | `iface .../template/asset/binding/expression.rs` | `UiBindingExpression`:Literal / ParamRef / PropRef / Equals / NotEquals / And / Or / Not;`parse(&str)`(解析 `$` 语法) |
| 绑定目标 | `iface .../component/data_binding/binding_target.rs` | `UiComponentBindingTarget`:目标属性 + 赋值模式 |
| 数据源契约 | `iface .../component/data_binding/data_source.rs` | `UiComponentDataSourceDescriptor`、`UiComponentDataSourceKind` |
| 投影补丁 | `iface .../component/data_binding/projection_patch.rs` | `ProjectionPatch`(局部更新载体) |
| 事件信封 | `iface .../component/data_binding/event_envelope.rs` | `UiComponentEventEnvelope`(捕获/冒泡) |
| 绑定运行时 | `runtime ui/binding/mod.rs` / `router.rs` | `UiEventRouter::dispatch(&UiEventBinding)` |
| 绑定更新报告 | `runtime ui/binding/update_report.rs` | `retained_attribute_update()`、`component_state_value_update()`、`reflected_property_update()` |
| 视图脏集(09) | `core/editor_message/view_dirty_set.rs` | `ViewDirtySet`、`EditorViewInvalidationMask` |

### 2.2 真实缺口

- 缺**绑定方向/受控语义规范**:`UiComponentBindingTarget` 有赋值模式,但没固化"编辑器面板默认单向(state→view),写回走显式命令(view→state)"这套受控组件契约。
- 缺**token 绑定与数据绑定的统一解析规范**:`$--left-drawer-width`(设计 token,01)与 `$selection.name`(数据 prop)都是 `$` 前缀,需明确二者解析路径(token 走 01 资产,prop 走数据源)分流。
- 缺**view-model / snapshot 派生规范**:编辑器状态 → 面板 view-model 的派生没有"纯函数派生 + 仅脏依赖重算"的契约(目前 `refresh_reflection` 全量物化,09.S2 已知)。
- 缺**绑定级脏依赖**:09 是视图级脏集;缺"哪个绑定依赖哪个数据键 → 数据键变只重算依赖它的绑定"的细粒度依赖图,这是"编辑器复杂也不卡"在绑定层的落点。

## 3. 设计

### 3.1 单向数据流 + 受控组件(借鉴 React,落到既有绑定)

```
编辑器状态(EditorState / reflection)   ← 唯一事实源(single source of truth)
        │  派生(derive,纯函数)
        ▼
   面板 view-model(只读投影,不可被 view 直接改)
        │  绑定(UiBindingExpression 求值)
        ▼
   .zui 节点属性(view,受控)
        │  用户交互 → 事件信封(UiComponentEventEnvelope)
        ▼
   显式命令/请求(经 09 总线 publish,非直接写 view-model)
        │
        └──→ 改编辑器状态 → 重新派生 → 增量回流(闭环)
```

- **单向**:数据只从 state 流向 view;view 不直接改 state,改动以**事件→命令**表达(React 受控组件思想)。
- **受控**:面板属性值始终来自 view-model 派生,不在 view 侧持有真实状态(避免双源不一致)。
- **派生纯函数**:state→view-model 是纯函数,便于"仅脏依赖重算"与缓存。

### 3.2 `$` 表达式统一解析与分流

| 前缀形态 | 含义 | 解析路径 | 来源计划 |
| --- | --- | --- | --- |
| `$--name` / `$editor.*` | 设计 token | 01 token 资产解析 → 值(色/尺寸/密度) | 01 |
| `$prop.path` | 数据 prop(view-model 字段) | `UiBindingExpression::PropRef` → 数据源 | 本 11 |
| `$param` | 组件参数(实例化入参) | `UiBindingExpression::ParamRef` → 组件 param schema | 12 |
| 逻辑表达式 | `a == b && !c` | `UiBindingExpression` 组合求值 | 本 11 |

解析期先按前缀分流:token 引用不进数据依赖图(静态,改 token 走 01/渲染全局),prop/param 引用进绑定依赖图(动态,改值走增量)。

### 3.3 view-model 派生与 snapshot 契约

- 每个面板/视图声明它**依赖哪些 state 键**(selection / asset.<id> / scene.<node> …),派生函数据此产出 view-model snapshot。
- snapshot 是**不可变投影**;变更产生新 snapshot 或 `ProjectionPatch`(局部补丁),而非原地改。
- 派生与 09 对齐:state 键变 → 标记依赖该键的视图脏 → 帧末只重派生脏视图的 view-model(替代全量 `refresh_reflection`)。

### 3.4 绑定级脏依赖图(比 09 视图级更细)

```
数据键(selection.name / asset.42.thumbnail / scene.node.7.transform)
   ↕ 依赖边(绑定求值时登记)
绑定实例(节点属性 ← UiBindingExpression)
```

- 绑定求值时登记"此绑定读了哪些数据键"。
- 数据键变更 → 经依赖图找到依赖它的绑定 → 只重算这些绑定 → 经 `update_report` 局部写回属性 → 标记该绑定所属视图脏(喂 09)。
- 与 09 关系:09 管"哪个视图脏",11 管"视图内哪些绑定脏";两层都增量,避免视图内全量重绑。

### 3.5 写回路径(view → state,受控)

用户改输入框/拖滑块 → `UiComponentEventEnvelope` → 转为**显式编辑器请求**(09 `EditorMessage` / 08 `EditorPageRequest`)→ 改 state → 重派生回流。**禁止** view 侧直接 mutate view-model 或 state(保证单向、可撤销、可审计)。

## 4. 接口与数据结构草案(Rust)

```rust
// 绑定方向/受控契约
pub enum BindingDirection { OneWay /* state→view 默认 */, EventOut /* view→命令,非直接写回 */ }
pub struct EditorBindingContract {
    pub direction: BindingDirection,
    pub reads: Vec<DataKey>,        // 此绑定依赖的 state 键(进依赖图)
}
// view-model 派生(纯函数 + 脏依赖)
pub trait ViewModelDerive {
    fn dependencies(&self) -> &[DataKey];
    fn derive(&self, state: &EditorStateView) -> ViewModelSnapshot; // 纯函数
}
// 绑定级脏依赖图
pub struct BindingDependencyGraph {
    key_to_bindings: BTreeMap<DataKey, BTreeSet<BindingInstanceId>>,
}
impl BindingDependencyGraph {
    pub fn record(&mut self, binding: BindingInstanceId, reads: &[DataKey]);
    pub fn invalidate(&mut self, changed: &[DataKey]) -> BTreeSet<BindingInstanceId>; // 只返回受影响绑定
}
```

## 5. 模块与文件落点

| 动作 | 文件 | 说明 |
| --- | --- | --- |
| 新增 | `zircon_editor/src/core/editor_binding/mod.rs` | 受控契约 + view-model 派生 + 绑定依赖图 |
| 新增 | `zircon_editor/src/core/editor_binding/dependency_graph.rs` | 数据键↔绑定依赖图 |
| 修改 | `runtime ui/binding/update_report.rs` | 局部写回挂依赖图失效 |
| 修改 | `core/editor_event/runtime/editor_event_runtime_state.rs` | 派生接 09 脏集,替全量物化 |
| 新增 | `docs/ui-and-layout/data-binding-contract.md` | 单向流 + `$` 分流 + 派生 + 写回规范 |

## 6. 里程碑切片化

| # | 切片 | 涉及文件 | 验证命令 | 硬切换 |
| -- | --- | --- | --- | --- |
| S1 | 受控契约 + `$` 分流规范 + 绑定依赖图 | editor_binding/mod.rs / dependency_graph.rs / data-binding-contract.md | `cargo test -p zircon_editor --lib editor_binding --locked` | 新建 |
| S2 | view-model 派生接 09 脏集 + 局部写回 | editor_event_runtime_state.rs / update_report.rs | `cargo test -p zircon_editor --lib --locked` | 删除依赖该路径的全量物化(配合 09.S2 后端) |

## 7. 测试矩阵

- `$--token` 解析走 01,不进数据依赖图;`$prop` 进依赖图。
- 单数据键变更只重算依赖它的绑定,其余绑定不重算(依赖图计数可证)。
- view-model 派生纯函数:同 state 多次派生结果一致。
- 用户交互转命令,不直接写回 view-model/state(受控)。
- 派生脏只标记依赖该键的视图(喂 09),不波及无关视图。

## 8. 风险与对策

- 风险:依赖图登记遗漏致漏刷。对策:保留显式整图重算的兜底命令(仅调试);依赖边在绑定求值期强制登记。
- 风险:`$` 前缀歧义(token vs prop)。对策:命名空间分流(`--`/`editor.` → token;其余 → prop),解析期硬规则。
- 风险:绑定依赖图与 09 视图脏集职责重叠。对策:09 视图级、11 绑定级,11 的绑定脏聚合后喂 09 视图脏,不互相替代。

## 9. 完成定义

`$` 表达式统一解析并分流;面板单向受控数据流成文;view-model 纯函数派生;绑定级脏依赖图只重算受影响绑定并聚合喂 09;无视图内全量重绑常规路径。

## 10. 边界约束

不改 `UiBindingExpression` 解析器内部(已存在);不引入虚拟 DOM/运行时 diff 整棵树;token 解析归 01,组件 param 归 12,视图级脏归 09;view 不直接 mutate state。

## 11. 参考实现对照(dev/ 源码锚点)

- `dev/UnrealEngine/Engine/Source/Runtime/UMG/Public/Binding` 与 `Binding/States`、`FieldNotification`:属性绑定 + 字段变更通知(field notify)= "只通知变更字段"的增量绑定样板。
- `dev/material-ui/packages/mui-system`(styled/sx 求值)与 React 受控组件:单向数据流 + 受控输入思想(取理念,不取运行时)。
- `dev/theatre/packages/dataverse`:derivation/pointer — 派生 + 细粒度依赖追踪的增量传播参考。
- `dev/slint/internal/core/properties.rs`:property 依赖追踪 + 惰性重算,取"依赖图驱动最小重算"理念。

## 12. 状态与产出记录

planned。后续项:S1 受控契约 + `$` 分流规范 + 绑定依赖图。
