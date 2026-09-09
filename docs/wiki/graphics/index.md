---
related_code:
  - zircon_runtime/src/graphics/mod.rs
  - zircon_runtime/src/render_graph/mod.rs
  - zircon_runtime/src/rhi.rs
  - zircon_runtime/src/core/framework/render/mod.rs
implementation_files:
  - zircon_runtime/src/graphics/mod.rs
  - zircon_runtime/src/render_graph/mod.rs
  - zircon_runtime/src/rhi.rs
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - docs/assets-and-rendering/render-framework-architecture.md
tests:
  - zircon_runtime/src/graphics/tests
  - zircon_runtime/src/render_graph/tests
  - zircon_runtime/crates/zr_rhi/src/tests
doc_type: category-index
---

# 图形与渲染

本分区说明 ZirconEngine 当前图形栈，从场景数据如何成为一帧画面，一直到 Rust 调用接口、RHI 资源生命周期和 WGPU 呈现。写法采用“概念 -> 工作流 -> API”的渐进层次；页面中的能力状态以当前源码为准，而不是以计划标题或类型名称推断。

## 功能状态约定

| 标记 | 含义 |
| --- | --- |
| **默认可用** | 位于默认 `graphics` 产品路径，并由运行时/产品测试覆盖 |
| **可选可用** | 有真实实现，但需要 profile、能力或插件显式启用 |
| **实验性** | 已有端到端或局部实现，接口/性能/后端覆盖仍可能变化 |
| **契约/骨架** | 类型、能力位或注册槽已存在，但不能视为完整产品功能 |

“存在 Rust 类型”不等于“默认渲染器会执行该功能”。尤其是 `RayTracing`、`NeuralCompute`、Solari、Hybrid GI 和 Virtual Geometry，必须同时检查 profile、后端能力与 provider 注册。

## 推荐阅读顺序

1. [渲染架构与帧生命周期](architecture.md)：理解运行时、渲染框架、场景渲染器、渲染图和 RHI 的职责。
2. [渲染框架与视口](render-framework.md)：创建视口、提交场景、绑定窗口、读取统计与捕获。
3. [渲染管线与场景渲染器](scene-renderer.md)：Core2D、Forward+、Deferred、阶段排序和扩展点。
4. [渲染图](render-graph.md)：资源版本、依赖、裁剪、瞬态别名和编译结果。
5. [RHI 与 WGPU 后端](rhi-and-wgpu.md)：设备、句柄、命令、提交票据、Surface 生命周期。
6. [着色器系统](shaders.md)：WGSL、模板、Shader IDE、变体缓存和插件模块。
7. [网格、材质与纹理](mesh-material-texture.md)：资产到 GPU 资源与绘制命令的转换。
8. [光照与环境](lighting-and-environment.md)：光源、阴影、IBL、反射探针、烘焙数据与高级能力。
9. [可见性、后处理与高级渲染](visibility-and-post-processing.md)：裁剪、HZB、时域历史、AO、SSR、Bloom 等。
10. [UI 渲染交界](ui-rendering.md)：场景内 UI、原生 retained UI Surface、文本和共享 GPU 图像。
11. [Rust API 总览](rust-api-reference.md)：按使用场景查询公开类型和调用约束。

## 模块地图

| 层 | 主要代码 | 对外职责 | 当前状态 |
| --- | --- | --- | --- |
| 中立渲染契约 | `core::framework::render` | 场景快照、视口、profile、材质、光照、后处理 DTO 和 `RenderFramework` | 默认可用 |
| 图形产品实现 | `graphics` | WGPU 框架、资源流送、场景绘制、插件扩展、捕获与诊断 | 默认可用；部分高级能力可选 |
| 渲染图 | `render_graph` | Pass/资源声明、依赖编译、资源状态和瞬态分配计划 | 默认可用 |
| 中立 RHI | `zr_rhi`，经 `zircon_runtime::rhi` 暴露 | 描述符、代际句柄、命令列表、提交与 Surface 契约 | 默认可用 |
| WGPU RHI | `zr_rhi_wgpu` | 中立 RHI 的 WGPU 映射、生产提交、诊断、UI Surface | 默认可用，但场景后端尚未全部迁入中立 RHI |
| 资产 | `asset::assets` | Mesh、Texture、Material、Shader 资产模型和导入数据 | 默认可用 |
| 插件渲染 | `zircon_plugins/rendering` 等 | Pass executor、几何源、着色模型、advanced provider | 可选/实验性 |

## 架构边界警告

当前源码明确说明：产品场景/离屏渲染仍由 `graphics::backend` 直接拥有 WGPU 对象；`zr_rhi_wgpu::production::WgpuRenderDevice` 是正在收敛的中立 RHI 所有者。应用代码应优先依赖 `RenderFramework`、`zircon_runtime::graphics` 和 `zircon_runtime::rhi` 门面，不应把 `wgpu::Device`、`wgpu::Texture` 或 crate-private `SceneRenderer` 内部字段传播到业务模块。

## 功能开关

`zircon_runtime::graphics`、`render_graph`、`rhi` 均受 Cargo feature `graphics` 控制；UI 模块还需要 `ui`。文档中的导入示例默认启用了相应 feature。Headless profile 可拥有渲染契约，但不意味着创建了 GPU 设备或 Surface。
