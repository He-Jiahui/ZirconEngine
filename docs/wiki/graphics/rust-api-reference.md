---
related_code:
  - zircon_runtime/src/graphics/mod.rs
  - zircon_runtime/src/graphics/prelude.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/render_graph/mod.rs
  - zircon_runtime/src/rhi.rs
  - zircon_runtime/crates/zr_rhi/src/lib.rs
implementation_files:
  - zircon_runtime/src/graphics/mod.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/render_graph/mod.rs
  - zircon_runtime/src/rhi.rs
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
tests:
  - zircon_runtime/src/tests/runtime_absorption/structure_convention
  - zircon_runtime/src/graphics/tests
  - zircon_runtime/src/render_graph/tests
  - zircon_runtime/crates/zr_rhi/src/tests
doc_type: module-detail
---

# Rust API 总览

本页按调用者场景列出稳定入口。它不是自动生成的逐项 rustdoc；签名以源码为准，页面重点是所有权、可见性和调用顺序。

## Cargo 与模块入口

图形 API 由 `zircon_runtime` 的 `graphics` feature 控制：

```toml
[dependencies]
zircon_runtime = { path = "../zircon_runtime", features = ["graphics", "ui"] }
```

主要导入路径：

```rust
use zircon_runtime::core::framework::render::*; // 中立 DTO 与 RenderFramework
use zircon_runtime::graphics::*;                // 图形产品公开门面
use zircon_runtime::graphics::prelude::*;       // 高频集成类型
use zircon_runtime::render_graph::*;            // Render Graph authoring/compile
use zircon_runtime::rhi::*;                     // 中立 RHI 门面
```

避免从 `graphics::scene::*`、`graphics::backend::*` 等 crate-private implementation owner 导入；它们不会对外可见，且不构成稳定 API。

## RenderFramework

| API | 输入/输出 | 关键语义 |
| --- | --- | --- |
| `create_viewport` | descriptor -> handle | 创建逻辑视口 |
| `destroy_viewport` | handle | handle 之后失效 |
| `submit_frame_extract(_with_ui)` | handle + extract | 离屏接受一帧；不保证 GPU 已完成 |
| `bind/unbind_viewport_surface` | handle + native descriptor | 管理窗口 Surface |
| `present_frame_extract(_with_ui)` | handle + extract | 向已绑定 Surface 呈现 |
| `set_pipeline_asset` | viewport + pipeline handle | 切换管线 |
| `reload_pipeline` | pipeline handle | 重新编译已注册资产 |
| `set_quality_profile` | viewport + profile | 应用能力感知质量配置 |
| `set_submission_config` | config | 可选；自定义后端可能不支持 |
| `query_stats` | `RenderStats` | 状态快照，不是 completion fence |
| `capture_frame` | optional captured frame | 可涉及 readback |
| `poll_*_if_newer` | generation -> optional | 非阻塞 |
| `request/poll/cancel_viewport_pick` | request/ticket | 异步、帧代际限定 |
| `request/poll/cancel_environment_capture` | scene/request/handle | 异步环境捕获 |
| `create_ui_surface_presenter` | UI descriptor -> presenter | 同设备 retained UI |

所有可选 trait method 都可能返回 `UnsupportedCapability`。面向多个后端的代码必须处理该 variant。

## WgpuRenderFramework

公开构造/专用方法包括 `new`、带 startup report/options 的构造、带 plugin render extensions/provider 的构造、shader pipeline prewarm、GPU timing report、realtime IBL report 和 planar reflection invalidation。

当宿主已有 engine module/service registry 时，优先解析注册的 `RenderFramework` 服务；直接构造适合测试、离线工具或独立 runtime host。构造需要 `ProjectAssetManagerAccess`，因为 shader/material/texture/font 等资源都通过项目资产 authority 解析。

## GraphicsModule 与扩展注册

```rust
use zircon_runtime::graphics::{
    GraphicsModule, RenderFeatureDescriptor, RenderPassExecutorRegistration,
};

fn module_with_feature(
    feature: RenderFeatureDescriptor,
    executor: RenderPassExecutorRegistration,
) -> GraphicsModule {
    GraphicsModule::with_render_extensions(
        [feature],
        [executor],
        [], // VirtualGeometryRuntimeProviderRegistration
    )
}
```

完整构造还可传 geometry source、shading model、plugin shader modules、runtime prepare collectors、Hybrid GI/Solari/Virtual Geometry providers。注册记录是不可变 module descriptor 的输入，运行中临时修改不是标准工作流。

## Render Graph

核心类型：`RenderGraphBuilder`、`RenderPassId`、`RgTextureHandle`、`RgBufferHandle`、`ExternalResource`、`RenderGraphResourceVersionToken`、`CompiledRenderGraph`、`RenderGraphError`。

调用顺序：创建 builder -> 声明 Pass/资源 -> 声明每次 read/write 与版本 -> 标记 present/readback/persistent 根 -> `compile` -> 将 compiled graph 与 executor registry/外部绑定交给执行层。

句柄带 builder generation，不可跨 graph 保存。外部系统若要缓存，应缓存资源 schema/asset，而不是 `RgTextureHandle`。

## RHI RenderDevice

`RenderDevice: Send + Sync`，主要方法族：

- 能力/身份：`caps`、`device_id`、`generation`、`require_operation`；
- 资源：create/desc/destroy buffer、texture、view、sampler、binding、shader、pipeline；
- Surface：create/reconfigure/acquire/present/discard/destroy session；
- 命令：`create_command_list`；
- 提交：create/enqueue/flush/poll/status/cancel/wait packet；
- IO：batched buffer/texture write、read；
- 统计：transient allocator 与 memory snapshot。

不要使用公开 `SubmissionTicket::new` 伪造完成依赖；后端会验证 issued table。

## 资产与渲染 DTO

| 类别 | 常用类型 |
| --- | --- |
| 帧/视图 | `RenderFrameExtract`, `RenderViewExtract`, `ViewportCameraSnapshot`, `SceneViewportRenderPacket` |
| Mesh | `MeshAsset`, `RenderMeshDescriptor`, `RenderMeshSnapshot`, `RenderMeshTopology` |
| Material | `StandardMaterialDescriptor`, `RenderMaterialAlphaMode`, `MaterialPropertyOverrideBlock` |
| Texture | `TextureAsset`, `TextureMetadata`, `RenderImageDescriptor`, `RenderSamplerDescriptor` |
| Light | `RenderDirectionalLightSnapshot`, `RenderPointLightSnapshot`, `LightShadowSettings` |
| Environment | `EnvironmentExtract`, `SkyboxSettings`, `SourceCubemapEnvironment`, `ReflectionProbeData` |
| Post | `RenderPostProcessVolumeProfile`, `RenderResolvedPostProcessSettings`, `AntiAliasSettings` |
| UI | `UiRenderSubmission`, `UiSurfaceDrawList`, `UiSurfacePresenter` |

## 错误类型

- `RenderFrameworkError`：视口/管线/后端/能力级错误。
- `GraphicsError`：SceneRenderer、资源准备、提交事务和 capture 级错误。
- `RenderGraphError`：图声明与编译错误。
- `RhiError`：描述符、句柄、命令、提交、Surface 和设备错误。
- 各资产拥有自己的 validation error；应在导入/保存时处理，而不是推迟到 GPU 创建。

## API 稳定性分区

| 路径 | 稳定性建议 |
| --- | --- |
| `core::framework::render` | 跨模块契约，优先依赖 |
| `graphics` 顶层 re-export / prelude | 支持的产品门面 |
| `render_graph` 顶层 re-export | 支持的图 authoring/诊断面 |
| `rhi` 顶层 re-export | 支持的中立 RHI 面 |
| `zr_rhi_wgpu` public native 类型 | WGPU 专用；仅图形产品/工具使用 |
| crate-private implementation modules | 非 API，不可从外部使用 |

## 非阻塞编程约定

渲染 API 中的 `poll_*` 一律按非阻塞理解；`None` 通常表示 pending 或没有新 generation，不是失败。显式 `capture_frame`、`wait_for_submission` 等才可能等待。实时循环应 request 一次、跨 tick poll，并始终给 wait 设置 timeout。
