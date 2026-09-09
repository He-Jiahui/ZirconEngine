---
related_code:
  - zircon_editor/src/core/commands/descriptor.rs
  - zircon_editor/src/core/commands/registry.rs
  - zircon_editor/src/core/editor_operation.rs
  - zircon_editor/src/core/editing/engine/command.rs
  - zircon_editor/src/core/editing/engine/transaction.rs
  - zircon_editor/src/core/editing/engine/history.rs
implementation_files:
  - zircon_editor/src/core/commands
  - zircon_editor/src/core/editing/engine
plan_sources:
  - user: 2026-09-09 扩展 ZirconEngine Wiki 教程与机制说明
  - docs/editor-and-tooling/editor-command-workflow.md
tests:
  - zircon_editor/src/core/commands/registry/tests.rs
  - zircon_editor/src/core/editing/engine
doc_type: workflow-detail
---

# 把编辑器操作接入命令、事务与撤销

本教程以“重命名场景节点”为例说明作者态修改为何需要两层：命令系统负责入口、显示和可用性；事务系统负责实际 `EditCommand`、undo/redo、dirty 与 journal。完整约束见[命令、事务与撤销系统](../editor/commands-transactions.md)，编辑器的 host/gateway 边界见[编辑器架构与功能状态](../editor/architecture.md)。

## 前置条件和边界

- 操作必须有稳定的 `EditorOperationPath`，不要把菜单文案、快捷键或 UI 控件 ID 当作业务身份。
- 命令是否可用统一由 descriptor 与 `CommandEvalCtx` 判断；UI 不复制条件逻辑。
- 只有改变作者态的操作进入事务；打开窗口、聚焦面板等非文档变更不应制造 undo 条目。

```mermaid
flowchart LR
    A[菜单 / 快捷键 / 命令面板] --> B[EditorCommandRegistry]
    B --> C[WhenClause + CommandEvalCtx]
    C -->|可用| D[EditorOperationInvocation]
    D --> E[operation factory]
    E --> F[EditCommand apply]
    F --> G[EditorTransactionEngine commit]
    G --> H[history / dirty / journal]
    H --> I[undo 或 redo]
```

命令描述、operation 调用和事务引擎分别位于[descriptor](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_editor/src/core/commands/descriptor.rs)、[operation](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_editor/src/core/editor_operation.rs)与[transaction](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_editor/src/core/editing/engine/transaction.rs)。

## 步骤 1：先注册稳定的命令入口

以下是当前 descriptor 和 path API 的最小形状。描述符还可以携带本地化、菜单投影、快捷键、payload schema、capability 与资产写目标信息，应由插件或功能模块一次性贡献。

```rust
use zircon_editor::{EditorCommandDescriptor, EditorCommandRegistry};
use zircon_editor::core::editor_operation::EditorOperationPath;

let descriptor = EditorCommandDescriptor::operation(
    EditorOperationPath::parse("scene.node.rename")?
);

let mut registry = EditorCommandRegistry::default_workbench();
registry.register(descriptor)?;
```

对于插件贡献，使用 `EditorCommandContributionSet` 并同时提供 `Operation` 的 factory；插件退出时让 registry 统一撤销贡献。不要在卸载后遗留菜单项或绑定到已释放代码的回调。

## 步骤 2：把 UI 请求规范化为 invocation

UI、远程调用和 CLI 应构造同一种 `EditorOperationInvocation`。`operation_group` 将拖动滑块、连续文本提交等细粒度输入合并为一个逻辑事务；来源保留为 `Menu`、`UiBinding`、`Remote` 或 `Cli`，便于审计和策略判断。

```rust
use serde_json::json;
use zircon_editor::core::editor_operation::EditorOperationInvocation;

let invocation = EditorOperationInvocation::parse("scene.node.rename")?
    .with_arguments(json!({"entity": 42, "name": "Light"}))
    .with_operation_group("inspector-rename");
```

实体的参数结构应由实际 payload schema 与当前文档身份决定；示例中的 `entity: 42` 只是**调用机制示意**，不是跨会话可持久化实体 ID 格式。

## 步骤 3：在一个事务中执行可撤销修改

`EditCommand` 需要定义 `apply` 与 `revert`，并正确报告 `CommandEffect::{Unchanged, Applied}`，让引擎能在部分应用失败时补偿。下段为**机制示意**：真实 `rename_command`、`edit_context` 与 `document_id` 由场景编辑域提供，但 begin/push/commit 的责任划分与当前实现一致。

```rust
use zircon_editor::core::editing::engine::{
    EditorTransactionEngine, HistoryContextId,
};

let engine = EditorTransactionEngine::new(edit_context);
let mut tx = engine.begin(
    "Rename Actor",
    HistoryContextId::Document(document_id),
)?;
tx.add_participant(document_id);
tx.push(rename_command)?;
let transaction_id = tx.commit()?;
```

提交后，历史、dirty 状态和 journal 作为一个编辑结果更新。不要先直接改 scene 再“补写”一条命令；那会使 apply/revert 与 journal 不再表示同一事实。

## 失败处理

| 场景 | 处理 |
| --- | --- |
| descriptor 注册冲突或 path 不合法 | 保持命令未注册，修正稳定路径；不要静默覆盖已有 operation。 |
| command 在当前上下文不可用 | 返回或显示统一的可用性结果；不要绕过 `WhenClause` 从 UI 直接调用 factory。 |
| `apply` 失败且未修改 | 报告 `Unchanged`，不写入历史。 |
| `apply` 部分修改后失败 | 报告 `Applied`，让事务引擎执行补偿；不要自行追加一个伪 undo。 |
| 交互被取消 | 不提交事务；临时 hover/focus 状态不进入持久 undo 历史。 |

## 验收清单与实践建议

- [ ] 所有入口共享一个 `EditorOperationPath` 和 descriptor 可用性判断。
- [ ] 修改作者态的命令有可验证的 apply/revert，且进入单一事务。
- [ ] 连续交互选择合适的 `operation_group`，避免将一次手势拆成无意义的历史项。
- [ ] 外部副作用通过 `EditorEventEffect` 交给宿主执行，纯 event reducer 不直接打开对话框或启动进程。
- [ ] 从[命令 registry 测试](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_editor/src/core/commands/registry/tests.rs)和[编辑引擎测试目录](https://github.com/He-Jiahui/ZirconEngine/tree/main/zircon_editor/src/core/editing/engine)补齐行为覆盖。

把“请求操作”和“修改文档”分开，会让菜单、快捷键、远程控制与回放共享同一语义，而 undo/redo 只对真正的作者态变化负责。这是扩展编辑器工具时最重要的边界。
