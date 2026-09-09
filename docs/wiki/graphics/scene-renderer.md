---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/mod.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_core2d.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_forward_plus.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_deferred.rs
  - zircon_runtime/src/graphics/feature/mod.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/core
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - docs/assets-and-rendering/render-framework-architecture.md
tests:
  - zircon_runtime/src/graphics/tests/project_render.rs
  - zircon_runtime/src/graphics/tests/render_product_submit
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests.rs
doc_type: module-detail
---

# 渲染管线与场景渲染器

## SceneRenderer 的职责

`SceneRenderer` 是当前 WGPU 图形产品的帧级执行所有者。它组合后端、资源流送器、GPU Scene、Pass executor、后处理资源、历史纹理、诊断和延迟完成队列。外部系统通常通过 `RenderFramework` 间接使用它。

渲染器的关键顺序为：完成轮询 -> 场景 admission -> GPU Scene membership -> 资源确保/上传 -> 输出目标准备 -> compiled graph 执行 -> 提交事务完成 -> 输出发布。失败发生在 scene submission 之后时，错误会携带 receipt/提交上下文，避免把已经进入 GPU 队列的工作当作未发生。

## 内置管线

### Core2D（默认可用）

阶段顺序：`Opaque2d -> AlphaMask2d -> Transparent2d -> PostProcess -> Ui -> Overlay -> Debug`。

内置 feature：Sprite、PostProcess、UI、DebugOverlay。适合 2D 游戏、工具画布和不需要 3D 光照的视口。当前 Core2D 的 UI 在 post process 后，overlay/debug 的精确上下关系与 3D 默认管线不同；自定义效果不能假设三种管线拥有完全一致的阶段序列。

### Forward+（默认可用）

阶段顺序：`Prepass -> Shadow -> Opaque3d -> AlphaMask3d -> Transparent3d -> PostProcess -> Overlay -> Debug -> Ui`。

内置 feature 包括 Mesh、Shadows、HZB、Temporal、ClusteredLighting、PostProcess、Bloom、ColorGrading、AntiAlias、DebugOverlay、UI。它适合透明物体较多或需要直接前向着色的场景。

### Deferred（默认可用）

阶段顺序：`Prepass -> Shadow -> Deferred -> AlphaMask3d -> AmbientOcclusion -> Lighting -> Transparent3d -> PostProcess -> Overlay -> Debug -> Ui`。

内置 feature 包括 DeferredGeometry 和 DeferredLighting，并共享阴影、HZB、时域、clustered lighting 与终端后处理。透明物体仍走透明前向阶段，不进入 GBuffer。

## RenderPipelineAsset

`RenderPipelineAsset` 描述 handle、revision、core pipeline kind、phase mapping 和 `RendererAsset`。`RendererAsset` 再声明 stages 与 feature 集合。编译器把它们降低为 `CompiledRenderPipeline` 和 Render Graph。

内置构造函数：

```rust
use zircon_runtime::graphics::RenderPipelineAsset;

let forward = RenderPipelineAsset::default_forward_plus();
let deferred = RenderPipelineAsset::default_deferred();
let core_2d = RenderPipelineAsset::default_core2d();
```

`revision` 参与 cache/reload 语义。修改自定义资产却不更新 revision，可能得到旧编译结果。`RenderPipelineCompileReport` 包含 feature 合同诊断，不应只判断是否返回 `CompiledRenderPipeline`。

## Render Feature 与 Pass Executor

一个扩展通常包含两个层面：

1. `RenderFeatureDescriptor` / `RenderFeaturePassDescriptor`：声明 Pass、资源 schema、读写、fallback、terminal surface 关系与能力要求。
2. `RenderPassExecutorRegistration`：把 executor ID 绑定到真实录制函数。

只注册 descriptor 而没有 executor，会在编译或执行阶段得到明确失败/降级；只注册 executor 而管线资产不引用，也不会自动执行。

`GraphicsModule::with_render_extensions_and_runtime_providers` 可同时注册 feature、几何源、着色模型、shader module source、Pass executor、runtime prepare collector，以及 Hybrid GI、Solari、Virtual Geometry provider。

## Runtime Prepare

`RuntimePrepareCollector` 位于图执行前，用于插件把 CPU 侧更新转换为受本帧事务管理的 buffer upload、GPU pass scope、readback request 或材质捕获 seed。它不是任意命令旁路：上传和 readback 必须记录到 `RuntimePrepareFrameTransaction`，以便成功和失败都能正确终结资源生命周期。

## 管线选择建议

| 需求 | 建议 |
| --- | --- |
| 纯 2D/Sprite/UI | Core2D |
| 3D，透明和材质灵活性优先 | Forward+ |
| 3D，大量灯光、GBuffer 后效需求 | Deferred |
| Virtual Geometry / Hybrid GI | Advanced profile + 对应 provider，先检查能力与降级报告 |
| 光追/神经计算 | 当前仅可作为显式实验/插件合同，不能作为默认能力假定 |

## 状态与限制

- 三个内置管线和主要 raster Pass 为**默认可用**。
- `BuiltinRenderFeature` 中的 advanced slot 是扩展位，不等于完整实现。
- Pass 并行录制和 async compute 是否实际发生取决于提交配置、图依赖和后端能力。
- `SceneRenderer` 的公开 startup/timing report 可用于诊断；其主要 render 方法并非外部稳定 API。
