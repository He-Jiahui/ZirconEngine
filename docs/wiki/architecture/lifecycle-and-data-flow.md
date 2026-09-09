---
related_code:
  - zircon_app/src/entry/engine_entry.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/handle/activation.rs
  - zircon_runtime/src/core/runtime/handle/resolution.rs
  - zircon_runtime/src/core/runtime/time.rs
  - zircon_runtime/src/scene/mod.rs
  - zircon_runtime/src/graphics/mod.rs
  - zircon_editor/src/scene/mod.rs
implementation_files:
  - zircon_app/src/entry/engine_entry.rs
  - zircon_runtime/src/core/runtime/handle/activation.rs
  - zircon_runtime/src/core/runtime/handle/resolution.rs
  - zircon_runtime/src/core/runtime/time.rs
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - .codex/plans/全系统重构方案.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
tests:
  - zircon_runtime/src/core/runtime/tests/activation
  - zircon_runtime/src/core/runtime/tests/resolution
  - zircon_runtime/src/tests/scene_boundary
  - zircon_editor/src/tests
doc_type: workflow-detail
status: current
---

# 生命周期与数据流

## 进程启动序列

```text
1. zircon_app 解析产品配置、Target、Profile
2. builtin/plugin composition 生成有序 EngineModule 集合
3. CoreRuntime::new / try_with_task_graph_options
4. 对每个模块执行 register_module(descriptor)
5. activate_registered_modules()
6. 模块 lifecycle: build -> ready(轮询到成功/超时) -> finish
7. 创建产品控制器并进入 editor/runtime/headless 主循环
```

注册与激活分离非常重要。注册阶段构建并冻结依赖事实；激活阶段才执行模块回调和 `Immediate` 服务工厂。这样宿主可以在副作用发生前得到重复名、缺失依赖、依赖环和非法服务依赖类型等错误。

## 模块激活

`activate_module(name)` 不是只激活一个表项。核心从冻结模块图取出目标模块的 dependency closure，验证整个闭包当前可激活，然后按拓扑顺序逐一进入生命周期。

同一模块的并发生命周期命令由 `LifecycleCoordinator` 串行化：同命令的其他线程等待并复用完成结果；同线程重入会返回类型化错误。回调在注册表锁外执行，避免用户生命周期代码造成全局锁重入。

### 生命周期阶段

| 阶段 | 模块状态 | 行为 |
| --- | --- | --- |
| 注册完成 | `Registered` | 描述符与服务槽可见，实例通常尚未创建 |
| 激活开始 | `Initializing` | 执行 module `build`，创建 immediate 服务，检查 `ready` |
| 激活完成 | `Running` | 执行 `finish`，开放服务调用 admission |
| 停机开始 | `Stopping` | 拒绝新调用并等待 in-flight 调用排空 |
| 停机完成 | `Unloaded` | 清除实例、递增 generation，旧句柄失效 |

## 服务解析数据流

```text
consumer
  -> ManagerResolver / CoreRuntime::resolve_manager
  -> 查找 RegistryName 与种类
  -> 若已有 Running 实例，直接返回
  -> 若 owner module 仍 Registered，先激活模块
  -> 深度解析 descriptor dependencies
  -> 执行 factory(CoreWeak 或 PluginContext)
  -> 提交 instance，状态改为 Running，开放 admission
```

解析栈检测单线程依赖环；线程等待图检测并发初始化中的跨线程环。工厂 panic 被捕获并转成 `CoreError::ServiceFactoryPanicked`，初始化失败会复位状态，允许后续重试。

## 帧时间与 World 数据流

Core 只拥有外层 monotonic-real 时间与“新 World 默认使用的时间策略”。`tick_time(max_fixed_steps)` 或 `advance_time_by(delta, budget)` 产生 `FrameTimeSnapshot`，其中包含 outer frame index、原始 real delta、累计 real time、fixed-step budget 和时钟不连续证据。

World 的 virtual/fixed 时间、暂停、时间债务与实际 fixed-step commit 属于 `LevelSystem`，不能放回进程级 Core。典型帧流是：

```text
platform/input events
        |
CoreRuntime FrameTimeSnapshot
        |
LevelSystem 为各 World 派生 WorldTimeSnapshot
        |
PreUpdate -> Fixed/Update -> LateUpdate
        |
runtime scene 基础 RenderExtract
        +-- editor state 生成 overlay（仅 EditorHost）
        |
framework::render 中性 packet
        |
graphics backend / presentation
```

## 编辑器写入流

编辑器命令、selection、gizmo 和 viewport camera override 先存在于 `zircon_editor` 作者态。对运行时实体的真实修改必须通过 scene manager/level system/命令桥提交到 World。Hierarchy、Inspector 和 viewport snapshot 是 World 与 Editor state 的派生视图，不能反向成为权威存储。

## 停机序列

`shutdown_registered_modules_with_drain_timeout` 按成功激活顺序的逆序停机。每个模块：

1. 检查是否仍有外部模块/服务依赖阻止卸载。
2. 通知上层 runtime lifecycle observer，允许其阻止不安全卸载。
3. 关闭服务新调用 admission。
4. 等待 in-flight 调用在 deadline 前排空。
5. 按预计算的服务 shutdown order 清除实例。
6. 执行 module `cleanup_until`。
7. 标记 `Unloaded` 并使句柄 generation 过期。

任务图有独立的 `shutdown_task_graph(Duration)`；它关闭 scoped task admission 并等待 task census 静止，但源码明确说明这还不是完整 runtime teardown，进程 timer 与部分私有 worker 尚未全部归 task graph 所有。

## 故障语义

- 激活闭包预检失败时，不应先重建其中已卸载的依赖模块。
- blocked unload 不得留下部分服务已卸载的中间状态。
- ready 超时、cleanup 超时、依赖环、缺失模块/服务、类型不匹配和过期句柄均返回 `CoreError`。
- 时钟前后台切换、窗口遮挡和 surface 重建通过 `ClockDiscontinuity` 触发 rebase，下一帧携带 generation receipt。

## 实现状态

模块/服务生命周期、并发协调、依赖闭包、服务调用排空、核心帧时间和逆序停机均已实现。完整“所有进程异步资源均由 task graph 统一回收”仍是部分实现，不能将 `shutdown_task_graph` 解释为整个引擎已经完全析构。
