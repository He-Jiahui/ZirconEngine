---
related_code:
  - zircon_editor/src/lib.rs
  - zircon_editor/src/core/mod.rs
  - zircon_editor/src/scene/mod.rs
  - zircon_editor/src/ui/mod.rs
implementation_files:
  - zircon_editor/src/lib.rs
  - zircon_editor/src/core/mod.rs
  - zircon_editor/src/scene/mod.rs
  - zircon_editor/src/ui/mod.rs
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 编辑器详细 Wiki
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/布局系统.md
tests:
  - zircon_editor/src/tests
  - zircon_editor/tests
doc_type: category-index
---

# Zircon Editor 文档

Zircon Editor 是 `zircon_editor` crate 提供的作者态宿主。它负责项目与文档会话、命令与事务、场景编辑工具、工作台布局、资产编辑器以及与 `zircon_runtime` 的通信；它不接管运行时世界、渲染器、资产注册表或共享 UI 算法的最终权威。

本分区采用与 Unreal Engine 文档相似的渐进结构：先解释使用工作流，再说明系统边界，最后给出 Rust 类型和调用入口。文档描述的是当前源码，而不是目标路线图中的理想形态。

## 从这里开始

| 读者目标 | 建议文档 |
| --- | --- |
| 理解编辑器整体分层、成熟度和模块归属 | [架构与功能状态](architecture.md) |
| 启动编辑器、创建/打开项目、管理场景文档 | [编辑器宿主、项目与文档会话](host-project-session.md) |
| 接入命令、事务、撤销/重做和恢复日志 | [命令、事务与撤销系统](commands-transactions.md) |
| 操作层级、选择、编辑/Play 世界与场景模式 | [场景作者态、层级与选择](scene-authoring.md) |
| 使用相机、拾取、网格、Gizmo 和变换手柄 | [Scene Viewport 与 Gizmo](viewport-gizmo.md) |
| 理解 Workbench、面板、窗口、布局和绑定 | [Workbench、面板与 UI 绑定](workbench-ui.md) |
| 开发 UI、动画、材质等资产编辑器 | [资产编辑器](asset-editors.md) |
| 连接 in-process 或 serialized runtime | [Runtime Gateway](runtime-gateway.md) |
| 快速查找公开 Rust 类型、构造器与限制 | [Rust API 索引](rust-api-reference.md) |

## 核心心智模型

```text
用户输入 / 自动化 / 菜单 / 绑定
              |
              v
EditorOperation / EditorEvent / typed command
              |
              v
EditorManager + EditorContext + Transaction Engine
        |                         |
        | 作者态状态              | 可撤销变更
        v                         v
Workbench / Scene tools ----> EditorRuntimeGateway
                                  |
                                  v
                     Runtime world / renderer / assets
```

必须始终区分两类状态：

- **作者态权威**：项目会话、打开的文档、选择、草稿、布局、编辑器插件、命令历史与资产编辑会话，由 `zircon_editor` 所有。
- **运行时权威**：世界实体与组件的执行语义、渲染帧、资产注册和资源驻留、Play 实例、runtime operation，由 `zircon_runtime` 或 runtime endpoint 所有。

编辑器可以通过 gateway 借用/查询运行时世界，也可以提交操作、拾取请求、highlight set 和帧请求，但不应把 runtime 内部对象作为长期作者态所有权保存。

## 功能状态标记

本 Wiki 使用四种状态：

- **已实现**：存在生产路径和对应类型，当前宿主已调用。
- **可扩展基础**：公共合同存在，默认能力或完整 UI 尚未全部接入。
- **受限**：可用，但受 gateway 类型、功能开关、会话状态或数据格式限制。
- **内部实现**：`pub(crate)` 或宿主内部细节，不应作为第三方稳定 API 使用。

## 稳定性约定

`zircon_editor/src/lib.rs` 的 re-export 是最稳定的 crate 入口。子模块中的 `pub` 类型可供仓库内其他 crate 使用，但并不等于长期兼容承诺。`pub(crate)`、测试 feature 与 `integration-contracts` 下的符号仅用于编辑器内部和验证。

Wiki 页面中的代码示例优先使用 crate root 或 owner module 的真实路径。若源码把类型降为私有，应同步更新文档，而不是通过兼容 re-export 保留旧路径。
