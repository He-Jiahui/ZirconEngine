---
related_code:
  - zircon_runtime/src/core/framework/render/framework.rs
  - zircon_runtime/src/graphics/runtime/render_framework/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mod.rs
  - zircon_runtime/src/render_graph/mod.rs
  - zircon_runtime/crates/zr_rhi/src/device/render_device.rs
implementation_files:
  - zircon_runtime/src/graphics/runtime/render_framework/wgpu_render_framework/wgpu_render_framework.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render/render_frame.rs
  - zircon_runtime/src/render_graph/builder/compile.rs
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - docs/assets-and-rendering/render-framework-architecture.md
tests:
  - zircon_runtime/src/graphics/tests/project_render.rs
  - zircon_runtime/src/render_graph/tests
  - zircon_runtime/crates/zr_rhi_wgpu/src/production/tests
doc_type: module-detail
---

# 渲染架构与帧生命周期

## 系统目标

ZirconEngine 将“业务可描述的数据”和“后端可执行的 GPU 工作”分开。场景、编辑器和插件产生中立 DTO；渲染产品层决定管线、资源和 Pass；渲染图编译依赖；RHI 负责代际资源、命令和提交生命周期；WGPU 后端将其映射为实际 GPU 工作。

这一结构类似 Unreal 的 Game Thread / Render Thread / RHI 分层，但 ZirconEngine 当前并不承诺完全相同的线程模型。`RenderSubmissionConfig::pipelined()` 可启用流水提交 worker，默认同步模式仍是合法产品路径。

## 分层职责

```text
World / Editor / Plugin
        |
        v
RenderFrameExtract + UiRenderSubmission
        |
        v
RenderFramework (稳定的跨模块契约)
        |
        +-- viewport/profile/surface/history/capture
        v
SceneRenderer + ResourceStreamer
        |
        +-- Core2D / Forward+ / Deferred
        +-- mesh/sprite/light/shadow/post/UI executors
        v
CompiledRenderGraph
        |
        +-- ordering/culling/version/access/state/allocation
        v
graphics::backend + zr_rhi(_wgpu)
        |
        v
WGPU device / queue / surface
```

## 一帧的所有权转换

1. **提取**：世界或宿主生成 `RenderFrameExtract`。它包含视图、几何、光照、粒子、后处理、可见性输入及变更信息，不含可由后端随意持有的世界可变引用。
2. **提交视口**：调用 `RenderFramework::submit_frame_extract` 或带 UI 的变体。框架验证视口句柄、profile 和设备状态。
3. **准备资源**：`SceneRenderer` 先轮询旧提交完成状态，再由 `ResourceStreamer` 确保 Mesh、Material、Texture、外部目标等资源存在，并把上传纳入本帧事务。
4. **建立帧事务**：`RenderFrameSubmissionTransaction` 记录 frame generation、上传提交、场景提交与产品输出，防止失败路径遗漏已接受的 GPU 工作。
5. **编译/复用渲染图**：管线资产生成 Pass 与资源声明；编译器计算顺序、裁剪、版本读写、状态转换和瞬态别名。缓存命中时可复用编译结果。
6. **录制与提交**：各 executor 在其 Pass 中录制命令。整个场景提交获得设备代际限定的 `SubmissionTicket`/receipt。
7. **呈现或离屏输出**：有绑定 Surface 时呈现；离屏路径保留最终颜色纹理，并可按请求生成 CPU capture 或共享 GPU viewport product。
8. **完成回收**：后续非阻塞 poll 路由提交完成，释放延迟销毁资源、消费诊断 readback、更新时间历史与纹理 residency。

## 核心不变量

- **句柄属于一个 owner generation**：渲染图句柄不能跨 builder 使用，RHI 句柄和提交票据不能跨设备/设备代际使用。
- **提交只有一个权威入口**：已经交给 native queue 的工作不能伪装为取消；Surface present 必须引用同设备且已提交/完成的票据。
- **读回不是普通渲染输出**：捕获、诊断和 GPU query 是有预算、有延迟的异步路径。不要在帧循环中无条件同步读回。
- **UI 是图的一部分或独立 Surface 产品**：viewport UI 通过 `UiRenderSubmission` 进入场景最终阶段；编辑器外壳等 retained UI 通过 `UiSurfacePresenter` 呈现，两者不是同一 API。
- **插件能力必须显式注册**：feature descriptor 只描述图合同；真正执行还需要 executor/provider、后端能力与 profile 同时满足。

## 当前实现状态

### 默认可用

- WGPU 离屏和窗口 Surface 渲染。
- Core2D、Forward+、Deferred 管线资产。
- Mesh、Sprite、阴影、clustered lighting、后处理、screen-space UI。
- Render Graph 编译、Pass 裁剪、资源版本、纹理子资源范围、瞬态分配和状态计划。
- 帧捕获、GPU timing/统计的能力门控、RenderDoc 请求接口。

### 可选或实验性

- Virtual Geometry、Hybrid GI、Solari 通过 runtime provider 与 advanced profile 接入。
- 体积雾、OIT、次表面散射、平面反射等有 executor 与测试，但依赖能力、管线配置或插件注册。
- 异步 pipeline compile、并行录制、pipelined submission 由 `RenderSubmissionConfig` 显式启用。

### 尚未完成的收敛

- 场景后端仍有直接 WGPU 所有权，尚未完全经 `RenderDevice` 中立命令面执行。
- `BuiltinRenderFeature::RayTracing`、`NeuralCompute` 等描述符或能力槽不能单独证明产品实现完整。
- 多后端 RHI 不是当前事实；生产实现是 WGPU。

## 线程与锁

`RenderFramework: Send + Sync`，但这不表示所有方法无锁或可任意并发。`WgpuRenderFramework` 使用 operation/state 锁串行化 renderer 所有权；流水 worker 用于提交调度。调用者应把 framework 视为线程安全的服务对象，而不是把内部 `SceneRenderer` 当成可共享的渲染上下文。
