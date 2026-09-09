---
related_code:
  - zircon_editor/src/scene/mod.rs
  - zircon_editor/src/scene/selection/mod.rs
  - zircon_editor/src/scene/modes/mod.rs
  - zircon_editor/src/core/editing/authoring_world.rs
  - zircon_editor/src/core/play/mod.rs
  - zircon_editor/src/ui/binding/selection/mod.rs
implementation_files:
  - zircon_editor/src/scene/selection/selection_model.rs
  - zircon_editor/src/scene/selection/domain_selection.rs
  - zircon_editor/src/scene/modes/scene_mode_stack.rs
  - zircon_editor/src/scene/modes/editor_scene_mode.rs
  - zircon_editor/src/core/editing/command.rs
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 编辑器详细 Wiki
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - docs/editor-and-tooling/editor-command-workflow.md
tests:
  - zircon_editor/src/scene/selection/tests.rs
  - zircon_editor/src/scene/modes/tests.rs
  - zircon_editor/src/tests/editing
doc_type: module-detail
---

# 场景作者态、层级与选择

## 概览

编辑器中的 Scene 是对 runtime world 的作者态视图。层级树、Inspector、Viewport 和 Gizmo 共享实体身份与选择模型，但通过命令/事务修改世界。UI 面板不是场景真源，`SelectionModel` 也不是实体存在性的真源。

## Edit 与 Play 世界域

`WorldDomain` 明确区分：

- `WorldDomain::Edit`：源 Scene 的作者态世界，保存和普通 undo 针对该域。
- `WorldDomain::Play(PlayInstanceId)`：运行实例的世界，生命周期随 Play session，修改默认不写回源 Scene。

进入 Play 时，`SelectionModel::activate_play_domain` 为实例建立独立 selection，并从 Edit selection 初始化。退出时 `retire_play_domain` 删除该域并回到 Edit。不要把 Play entity id 直接写入 Edit selection 或源资产。

## 层级工作流

用户在 Hierarchy 中执行创建、删除、重命名、改父子关系时，推荐路径是：

```text
Hierarchy UI
 -> typed selection/operation
 -> EditorIntent / operation factory
 -> EditCommand
 -> EditorTransactionEngine
 -> runtime authoring world route
 -> SceneInspection invalidation
 -> hierarchy/inspector snapshot rebuild
```

常见约束由命令层而不是控件层执行：

- 不允许产生父子环。
- 不允许空名称或违反命名约束的名称。
- 删除会保护必须存在的关键对象，例如规则要求保留的最后一个 camera。
- 多节点操作在一个事务中提交，失败时整体回滚。
- 改父级需要保留或明确重算 transform 语义，不能只改层级显示。

Hierarchy 的渲染行是 snapshot/projection。外部 world invalidation 到来后，projection cache 增量刷新；不要长期缓存裸 world 引用。

## SelectionModel

`SelectionModel` 为每个 world domain 保存有序 `IndexSet<EntityId>`、primary、generation，并用总 revision 通知跨域观察者。

```rust
use zircon_editor::scene::selection::{SelectionModel, SelectionMutation, WorldDomain};

let mut selection = SelectionModel::default();
selection.select_only(WorldDomain::Edit, entity);
selection.extend(WorldDomain::Edit, more_entities);
selection.toggle(WorldDomain::Edit, entity);

let primary = selection.primary(WorldDomain::Edit);
let revision = selection.revision();
```

选择语义：

| 操作 | 效果 |
| --- | --- |
| `Replace` | 以新集合替换，最后一项作为 primary |
| `Extend` | 保留现有项并加入新项 |
| `Toggle` | 已选则移除，未选则加入 |
| `clear` | 清空指定域 |

`SelectionMutation::from_modifier_flags(shift, control)` 中 Control 优先映射为 Toggle，Shift 映射为 Extend，无修饰键为 Replace。平台层负责把 native modifier 转成这两个逻辑标志。

`generation(domain)` 只反映一个域；`revision()` 反映模型整体变化。不存在的 Play domain 返回空集合，修改返回 false，而不是隐式创建。

## Primary selection

Primary 是 Inspector、Gizmo pivot 和 frame-selection 的主要 subject；多选集合仍用于批量变换和 highlight。扩展/切换操作后 primary 的确定应由 `DomainSelection` 规则维护，不应由不同面板各自选择“第一项”。

Selection 在事务中保存 before/after snapshot。Undo/redo 除了恢复世界，也恢复 selection，从而保证 Inspector 和 viewport focus 回到一致 subject。

## Scene mode

`EditorSceneMode` 是场景工具模式合同：

```rust
pub trait EditorSceneMode: Send {
    fn id(&self) -> &SceneModeId;
    fn enter(&mut self, ctx: &mut SceneModeCtx<'_>);
    fn exit(&mut self, ctx: &mut SceneModeCtx<'_>);
    fn handle_input(
        &mut self,
        input: &ViewportInput,
        ctx: &mut SceneModeCtx<'_>,
    ) -> InputOutcome;
    fn update(&mut self, ctx: &mut SceneModeCtx<'_>) { }
    fn build_overlay(&self, out: &mut ViewportOverlayBuilder) { }
}
```

内置 base mode 包括 Select 和 Transform；扩展可以通过 registry/factory 注册 custom mode，并作为 overlay 压入 `SceneModeStack`。

输入从栈顶 overlay 向下传播。返回 `Consumed` 后停止；返回 `PassThrough` 时 `SceneModeCtx` 会恢复到调用前 checkpoint，避免一个未消费输入的模式留下 selection 或反馈副作用。

## SceneModeStack 规则

- 一个 mode id 在栈中只能出现一次。
- 内置 Select/Transform 不能伪装成 custom overlay。
- activation id 必须与 mode 的 `id()` 一致。
- 替换 base mode 若 enter 失败，会尝试重新进入旧 base；双重失败会报告 rollback failure。
- 插件贡献退休时，属于该 ticket 的 overlay 被移除；若 base 也来自该插件，则回退 builtin Select。
- shutdown 按 overlay 逆序退出，最后退出 base。

这些规则保证插件卸载不会留下指向已卸载代码的 mode。

## Inspector 草稿与提交

Inspector 输入分成两阶段：

- Draft：文本框/数值控件变化进入 `DraftCommand`，只更新可展示草稿和校验结果。
- Commit：`InspectorFieldBatch` 经 `apply_inspector_binding` 生成正式编辑命令并进入事务。

selection 改变会切换 Inspector subject，但不应自动提交上一个 subject 的无效草稿。批量提交可以包含 name、parent、translation 等字段；验证失败时保持 authoring world 不变并在 UI 显示错误。

## Play 编辑保护

`PlaySessionController` 管理 Play/Simulate 生命周期；`PlayEditPolicy` 和 `PlayEditProtection` 决定编辑请求是允许、拒绝、排队还是需要用户决策。Pending edit 通过 `PendingEditIntent`/id 跟踪。

默认认知：在 Play world 中成功修改不等于 Keep Play Changes。应用回 Edit world 必须走显式的 pending decision、冲突检查和新事务，Play history 本身会随 session 丢弃。

## 状态与限制

- **已实现**：多域选择、primary、多选 mutation、Scene mode 栈、层级/Inspector 命令化、Play 编辑保护。
- **可扩展基础**：插件 scene mode、overlay builder、自定义 Inspector。
- **内部实现**：`SceneModeCtx` 中部分 authoring world route、viewport controller 与 projection cache。
- 选择只保存实体身份，不保证实体仍存在；world invalidation 后必须过滤失效 entity。
- runtime serialized endpoint 下不能借用 `World`；场景操作需走 operation/query ABI。
