---
related_code:
  - zircon_runtime/src/graphics/visibility/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/mod.rs
  - zircon_runtime/src/core/framework/render/post_process/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/mod.rs
implementation_files:
  - zircon_runtime/src/graphics/visibility
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - docs/assets-and-rendering/render-framework-architecture.md
tests:
  - zircon_runtime/src/graphics/tests/visibility
  - zircon_runtime/src/graphics/tests/render_product_post_process
  - zircon_runtime/src/graphics/tests/render_product_post_process_full_chain
doc_type: module-detail
---

# 可见性、后处理与高级渲染

## 可见性系统

可见性从 `VisibilityInput`/`VisibilityRenderableInput` 和每个 view 建立 `VisibilityContext`。CPU 路径支持 perspective/orthographic frustum 测试、bounds、静态空间索引、BVH 更新规划、历史快照、batching 和 draw command 生成。

主要输出：

- `FrameVisibility`：本帧按 view 的可见结果；
- `VisibilityBatch` / `VisibilityDrawCommand`：按 mesh/material/phase 聚合；
- `VisibilityInstanceUploadPlan`：实例上传；
- `VisibilityParticleUploadPlan`：粒子上传；
- `VisibilityBvhUpdatePlan`：静态/动态索引更新策略；
- Virtual Geometry 与 Hybrid GI 的专用反馈/上传计划。

`query_visible_spatial_snapshot` 暴露的是 renderer-visible snapshot，可用于编辑器查询或调试，但不等价于 World 的完整物理/场景索引。

## HZB 与遮挡

HZB 从深度纹理构建 mip pyramid，服务屏幕空间反射、遮挡和 advanced provider。`HzbBuilder`/`HzbBuildPlan` 描述构建；实际 GPU executor 使用 WGPU compute pipeline。HZB diagnostic readback 需显式 submission config，避免默认 CPU stall。

当前有 HZB 构建与相关计划实现；全面 GPU-driven occlusion 效果仍取决于具体管线和 consumer，不能将 `Hzb` feature 等同于所有对象都经过 GPU occlusion culling。

## 后处理 Volume

`RenderPostProcessVolumeProfile` 包含可覆盖参数；`VolumeEvaluator` 根据 shape、priority、blend weight/distance 和 component registry 合成为 `RenderResolvedPostProcessSettings`。不同参数使用 float/vector lerp、bool/discrete 等显式插值策略。

`PostProcessPassGraph` 将解析后的 effect stack 变为有依赖的后处理节点，并用 `PostProcessGraphValidationError` 报告资源或排序错误。

## 当前效果

| 效果 | 当前路径 | 状态说明 |
| --- | --- | --- |
| Tonemap / output transfer | HDR scene color 到 SDR/目标色彩空间 | 默认可用 |
| Exposure | histogram + resolve，带 readback report | 默认可用，自动曝光需历史 |
| Bloom | 下采样/模糊/合成 | 默认可用 |
| Color grading / LUT | LUT bake 与采样 | 默认可用 |
| FXAA / SMAA / TAA | profile/能力选择 | 默认可用；TAA 依赖 velocity/history |
| Upscale / dynamic resolution | view family resolution plan | 可选可用 |
| SSAO | 深度/法线 qualification 与 AO profile | 默认管线可用，失败会报告原因 |
| SSR | reflection pyramid、resolve、specular occlusion | 可选可用，依赖深度/HZB |
| Motion blur | velocity tile/neighbor max 后执行 | 可选可用 |
| Depth of field | prepare + execute | 可选可用 |
| Vignette、grain、dither、chromatic aberration、fog | uber/post effect 参数 | 可选可用 |
| Half-resolution transparency | 独立资源与 composite | 可选/管线配置 |

## 时域历史

TAA、motion blur、SSR、曝光和 GI 等依赖 history。`RenderTemporalHistoryKey`/`RenderHistoryDomain` 将历史绑定到 viewport、camera/view、尺寸、格式、quality 和 device generation。相机切换、投影不兼容、尺寸/管线变化或设备重建会产生明确 invalidation reason。

Velocity 同时使用当前/上一帧 transform 和 camera matrices。没有合法 previous 数据时应标记 history invalid 或使用零 velocity，而不是读取未初始化纹理。

## Anti-Alias 选择

`AntiAliasSettings` 与 `AntiAliasMode` 描述作者意图；capability/profile 解析后可能产生 `AntiAliasFallbackReport`。默认 profile 要求至少可用的 screen-space AA；在 MSAA 不可用或与 deferred/某些目标不兼容时，可按规则回退 FXAA，而不是默默关闭。

## 高级栅格能力

代码包含 OIT、volumetric fog、subsurface scattering、planar reflection、light cookie、irradiance volume 和 screen-space transmission 的合同与 executor。这些功能为**可选或实验性**：必须由管线/插件注册、满足 storage buffer/format 等能力，并提供输入资源。页面名称或 WGSL 文件存在不是启用证据。

## 性能和调试

- Dynamic resolution 应以完成的 GPU sample 驱动，避免本帧自反馈。
- TAA/SSR/GI 调试时记录 history invalidation，而不是只比较像素。
- 后处理 effect stack 应跳过 disabled effect，并复用 terminal resource cache。
- SSAO/SSR/DOF 需确认 depth convention、MSAA resolve 和 render rect qualification。
- 使用 `RenderGraphExecutionProfileReport`、Pass GPU timing 与 effect report 定位成本；CPU frame time 不能替代 GPU pass 证据。
