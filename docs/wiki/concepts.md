---
related_code:
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/core/mod.rs
  - zircon_runtime/src/core/framework/mod.rs
  - zircon_runtime/src/core/manager/mod.rs
  - zircon_runtime_interface/src/lib.rs
  - zircon_editor/src/lib.rs
  - zircon_plugins/plugin_sdk/src/lib.rs
implementation_files:
  - zircon_runtime/src/core
  - zircon_runtime_interface/src
  - zircon_editor/src/core
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - docs/engine-architecture/runtime-tech-stack.md
  - docs/engine-architecture/workspace-root-rules-and-hard-cutover.md
tests:
  - zircon_runtime/src/tests/runtime_absorption
  - zircon_runtime_interface/src/tests
  - zircon_editor/src/tests
doc_type: category-index
---

# 基础概念

## 三个边界

ZirconEngine 把“产品入口”“引擎运行时”“作者态编辑器”拆成独立职责：

```text
zircon_hub  ->  zircon_app  ->  zircon_runtime
                              ^       ^
                              |       |
                       zircon_editor  |
                              |       |
                    zircon_runtime_interface
```

- **App** 决定产品角色、profile、窗口/主循环和插件组合。
- **Runtime** 持有生命周期、服务、World、资源、渲染和运行时 UI 的权威。
- **Editor** 持有选择、viewport 工具、命令历史、文档和布局等作者态。
- **Interface** 只传可序列化 DTO、`#[repr(C)]` 值、句柄和受限 byte buffer；trait 对象、`World` 引用、wgpu/winit 对象不能跨动态边界。

## Module、Driver、Manager、System

Runtime 以 `ModuleDescriptor` 描述可激活的功能单元。一个模块可声明：

- `InitLevel`：决定模块在 Kernel、Servers、Scene、Editor、Post 等阶段的顺序。
- `ModuleDependencySpec`：模块之间的拓扑依赖。
- `DriverDescriptor`：底层生命周期长、被 manager 使用的服务。
- `ManagerDescriptor`：按稳定服务名解析的高层服务。
- `PluginDescriptor`：可注册的扩展实例。
- `ModuleLifecycle`：`build`、`ready`、`finish`、`cleanup` 阶段。

`System` 是调度阶段中执行逻辑的工作单元，不等同于模块或服务。模块负责装配和生命周期，manager 负责访问入口，system 负责按帧执行。

## World、快照和 DTO

`zircon_runtime::scene` 的 World/ECS 是运行时实体和组件的权威。派生数据（例如世界变换、渲染可见性）由系统计算，不应由 editor 或 renderer 直接持久化。

跨层数据按三种形态流动：

1. **服务面**：稳定服务名 + `ServiceHandle`/resolver。
2. **数据面**：RenderExtract、WorldQuery、SceneSnapshot 等只读 DTO。
3. **扩展面**：模块/插件 registry 注册描述符、事件和 callback。

## Resource 与 Asset

Asset 是项目目录中的可引用输入；Resource 是 runtime 管理的、带状态和 generation 的运行时对象。`AssetReference`/`ResourceLocator` 解决持久身份，`ResourceHandle` 解决进程内访问，`ResourceReadinessGeneration` 解决异步就绪观察。显示路径不是身份键，必须使用规范化 locator、UUID 或 session identity。

## Editor 作者态

作者态包括选择集、编辑/Play 模式、viewport 相机 override、gizmo、Workbench 布局、命令历史、dirty/save token 和文档会话。这些状态不能写入 runtime World 的持久序列化。改变 World 的操作必须经过编辑器 command/transaction 或 gateway operation，以便撤销、回放和恢复。

## Frame 与时间

每帧一般按 `PreUpdate -> Update -> LateUpdate -> FixedUpdate(可多步) -> RenderExtract -> render/present` 组织。`FrameClock` 提供单调时间、帧计数、固定步累积和 discontinuity/rebase；渲染侧只消费抽取结果，不直接推进模拟时间。

## Handle 与 Generation

跨生命周期对象不应暴露裸引用。服务、session、viewport、operation、watch 和 resource 都使用带身份/版本的 handle 或 token。调用者在异步完成时必须重新验证 generation/session identity，避免项目切换或插件重载后把旧结果提交到新对象。

## Status 与 Capability

状态不是简单的布尔值：模块有 lifecycle state，资源有 readiness state，operation 有 phase，产品有 exit class，插件有 maturity/capability。`zircon_runtime_interface` 的 status code 和 report DTO 用于把失败阶段、期望/实际和修复提示带回宿主。

## 常见误区

| 误区 | 正确做法 |
| --- | --- |
| 在 editor 直接改 runtime ECS | 构造 typed intent/operation，交给 transaction/gateway |
| 把 runtime `pub` 当作稳定 ABI | 只有 interface 的 `#[repr(C)]` 函数表/DTO 才能跨动态库 |
| 用显示路径做对象身份 | 使用 `AssetUuid`、`ResourceId`、`ProjectSessionId` 等稳定键 |
| 认为有 descriptor 就代表功能完整 | 检查 feature、宿主 wiring、测试和状态标签 |
| 用 `unwrap` 处理帧循环错误 | 返回域错误并通过 diagnostics/exit report 暴露 |
