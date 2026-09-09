---
related_code:
  - zircon_app/src/lib.rs
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/core/mod.rs
  - zircon_runtime/src/builtin/runtime_modules.rs
  - zircon_editor/src/lib.rs
implementation_files:
  - zircon_app/src/entry/engine_entry.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/builtin/runtime_modules/core_modules.rs
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - .codex/plans/全系统重构方案.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
tests:
  - zircon_app/src/tests
  - zircon_runtime/src/tests/runtime_absorption
  - zircon_runtime/src/builtin/runtime_modules/tests
doc_type: category-index
status: current
---

# ZirconEngine 架构

本分区从引擎使用者和模块开发者的视角说明 ZirconEngine 的整体结构。阅读顺序参考大型引擎文档的“概念 -> 系统 -> 编程接口”层级：先理解谁拥有进程、运行时世界和作者态数据，再进入模块组合、生命周期与 Rust 接口。

## 当前架构一句话说明

ZirconEngine 采用三个顶层产品包：`zircon_app` 是进程与产品 Profile 宿主，`zircon_runtime` 是运行时能力吸收层并内置 `core/{runtime,framework,manager,math,resource}` 主干，`zircon_editor` 是编辑器宿主和作者态逻辑所有者。具体能力必须以模块描述符、稳定管理器句柄、ECS/数据契约或插件协议接入，不能由上层直接拼接底层实现对象。

```text
zircon_app
  | 创建 CoreRuntime、选择 Profile、驱动宿主循环
  v
zircon_runtime
  +-- core::runtime    注册、依赖、生命周期、事件、时间、任务
  +-- core::manager    稳定服务名、类型化句柄、解析入口
  +-- core::framework  中性 trait 与跨模块 DTO
  +-- core::math       统一数学与精度边界
  +-- core::resource   统一资源身份、状态与 I/O 投影
  +-- built-in modules / optional runtime modules
  |
  +---- zircon_editor 仅在 EditorHost 产品中激活
```

## 文档导航

| 页面 | 适合读者 | 内容 |
| --- | --- | --- |
| [系统总览](system-overview.md) | 所有人 | 三包拓扑、权威数据、设计原则和实现状态 |
| [分层与所有权](layer-boundaries.md) | 架构与功能开发者 | `runtime/manager/framework/math/resource` 的职责及禁止依赖 |
| [模块与产品组合](module-and-profile-composition.md) | 宿主、插件、发布工具开发者 | 内建模块清单、Target/Profile、清单覆盖和启动链 |
| [Framework 模块目录](framework-module-catalog.md) | 所有模块开发者 | 逐项列出 `core::framework` 契约、Rust 入口、实现 owner 和启用条件 |
| [生命周期与数据流](lifecycle-and-data-flow.md) | 运行时与编辑器开发者 | 从进程启动、服务解析到帧推进、渲染提取和停机的数据流 |
| [核心运行时](../core-runtime/index.md) | Rust 调用者 | `CoreRuntime` 及其所有核心功能的细化说明 |

## 顶层运行时入口

除 `core/*` 主干外，`zircon_runtime` 还提供以下顶层模块。它们的实现仍由 CoreRuntime、产品组合和 feature gate 约束：

| 模块 | 责任 | 深入页面 |
| --- | --- | --- |
| `builtin` | 内建 RuntimePluginId、基础模块和默认产品组合 | [模块与产品组合](module-and-profile-composition.md)、[Framework 模块目录](framework-module-catalog.md) |
| `engine_module` | `EngineModule`、descriptor、factory 和 driver/manager/plugin 合同辅助 | [核心 Rust API 使用指南](../core-runtime/rust-api-guide.md)、[分层与所有权](layer-boundaries.md) |
| `operation` | Runtime operation handler、admission、prepare/apply 和 ABI 结果 | [Runtime Operation](../core-runtime/operations.md) |
| `diagnostic_log` | 进程日志过滤、控制台/文件 sink、metrics 和 panic flush | [诊断日志](../core-runtime/diagnostic-log.md) |

## 阅读约定

文档中的状态标签含义如下：

- **已实现**：当前工作树中存在公开 API、实现和对应测试。
- **部分实现**：主契约已经存在，但仍有明确的功能边界未纳入统一所有权。
- **目标架构**：来自绑定计划，不能当作当前可调用 API。
- **可选功能**：受 Cargo feature、Target 模式或插件清单约束。

## 与 Unreal 文档概念的对照

这是一种帮助阅读的概念映射，不表示 API 或实现完全等价。

| ZirconEngine | 可类比的成熟引擎概念 | 关键差异 |
| --- | --- | --- |
| `CoreRuntime` | Engine subsystem host / module manager | Rust 所有权、显式描述符和类型化错误是核心约束 |
| `EngineModule` / `ModuleDescriptor` | Engine module | 描述符同时携带依赖、服务工厂和启动策略 |
| `ManagerServiceHandle<T>` | Subsystem access handle | 句柄包含运行时身份、服务槽位与 generation 校验 |
| `RuntimeTargetMode` / `RuntimeProfileId` | Target / build configuration | 组合结果还受项目插件清单和编译 feature 共同约束 |
| `zircon_runtime::scene` | Runtime World | 编辑器选择、gizmo 和 viewport 作者态不属于 World |
| `zircon_editor` | Editor host | 通过运行时契约操作 World，不拥有运行时权威状态 |

## 源码入口

- `zircon_app/src/entry/engine_entry.rs`：实际创建、注册和激活运行时。
- `zircon_runtime/src/core/runtime/runtime.rs`：核心运行时公开门面。
- `zircon_runtime/src/builtin/runtime_modules/core_modules.rs`：基础模块候选集。
- `zircon_runtime/src/core/mod.rs`：核心公开导出面。
- `zircon_runtime/src/prelude.rs`：应用与模块开发的便利导入。
