---
related_code:
  - zircon_runtime/src/core/framework/render/light/mod.rs
  - zircon_runtime/src/core/framework/render/environment/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/mod.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow
  - zircon_runtime/src/graphics/scene/scene_renderer/environment
  - zircon_runtime/src/graphics/runtime/render_framework/environment_capture_scheduler
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - docs/assets-and-rendering/environment-lightmap-probe-consumption.md
  - docs/assets-and-rendering/hybrid-gi-lumen-scene-representation.md
tests:
  - zircon_runtime/src/graphics/tests/render_product_shadows
  - zircon_runtime/src/graphics/tests/render_product_shadow_captures
  - zircon_runtime/tests/runtime_environment_wgpu_cubemap_sampling_contract.rs
doc_type: module-detail
---

# 光照与环境

## 光源模型

当前中立快照支持 Ambient、Directional、Point、Spot 和 Rect light。GPU 侧通过 `GpuLightData`/`GpuLightType` 打包；`LightingExtract` 同时携带 baked lighting 和 advanced lighting 数据。Forward+ 与 Deferred 都能消费 clustered light 数据，具体材质 lighting model 决定响应。

Rect light、light cookie、irradiance volume 等高级族可能有独立 readiness 报告。`RenderLightReadinessReport` 应作为“能否按作者意图渲染”的事实，不要用 light 数量推断功能成功。

## 阴影

`LightShadowSettings` 控制是否投影、resolution tier、PCF quality 等。Directional light 使用 cascade 规划；point/spot 等分配 shadow atlas slot。Atlas allocator 在有限尺寸内安排区域，多灯时可能降级或拒绝；`SHADOW_SLOT_NONE` 表示没有可用阴影数据。

阴影绘制尊重材质 alpha mask、`cast_shadows`、mesh 可见性和 light layer/范围。`receive_shadows` 在 shading 阶段控制采样。透明和 advanced material 的阴影行为应以对应 shader pass 为准。

## 环境与 Skybox

`EnvironmentExtract` 聚合 skybox、环境光、source cubemap、IBL 状态、反射探针和 baked lighting。`SkyboxMode` 支持 Disabled、程序化天空或 source cubemap 等模式（以 enum 当前 variant 为准）。预览环境与场景环境可以来自不同 authority，编辑器不得把 preview 状态写回 World。

## Image Based Lighting

IBL 管线包含：

1. 输入 equirect/cubemap 或已持久化 artifact；
2. 构建 source cubemap mip chain；
3. PMREM 预过滤 specular；
4. 计算 SH9 或 irradiance cubemap；
5. 使用 canonical BRDF LUT 和 PBR recipe；
6. 上传并以 source revision/recipe identity 建立 residency；
7. 场景 shading 采样 diffuse irradiance、prefiltered specular 和 BRDF LUT。

`IblBakeRecipeIdentity`、`EnvironmentPbrRecipeIdentity` 和 algorithm version 是缓存兼容合同。不要仅按源图片 hash 复用旧 artifact。

`RealtimeIblStatusReport` 区分 readiness、generation 和 retry/terminal failure。`WgpuRenderFramework::realtime_ibl_status_report()` 会先结束 pending pipelined submission 再读取状态，调用频率应受控。

## 反射探针

`ReflectionProbeData` 描述位置、影响形状、box projection 和资源身份。`select_reflection_probe_blend` 根据 influence weight 选择有限集合；`reflection_probe_box_project_direction` 修正 box-projected cubemap 方向。

Planar reflection 是独立路径：`request_planar_reflection_capture(probe_id)` 标记 dirty，下一相机循环在主相机前提交 mirror capture。它是**可选可用**，且有额外场景渲染成本。

## Lightmap 与 Probe Grid

`LightmapConsumeContract` 将 atlas descriptor、instance slot 和 `LightProbeGridData` 连接到运行时。烘焙输入/输出包含显式版本与预算。当前代码支持消费合同、atlas page 和 probe 数据；是否有完整编辑器烘焙工作流需查看资产/工具分区，不能从 runtime DTO 推断。

## 环境捕获 API

```rust
use zircon_runtime::core::framework::render::{
    RenderEnvironmentCaptureRequest, RenderFramework, SceneViewportRenderPacket,
};

fn start_capture(
    framework: &dyn RenderFramework,
    scene: SceneViewportRenderPacket,
    request: RenderEnvironmentCaptureRequest,
) -> Result<(), Box<dyn std::error::Error>> {
    let handle = framework.request_environment_capture(scene, request)?;
    let status = framework.poll_environment_capture(handle)?;
    // status 未完成时，在后续 tick 再轮询；不要阻塞帧线程。
    let _ = status;
    Ok(())
}
```

完成后 `take_environment_capture_source_payload` 将 CPU 持久化 payload 从渲染 owner 移出。该操作不是普通 frame capture。

## 高级全局光照

- Hybrid GI 有 runtime provider、prepared sideband、surface/radiance/global-SDF 数据与 WGPU 产品测试，属于 **Advanced profile 的实验性能力**。
- Solari 有独立 provider 和 capability requirement，profile 名即 `SolariExperimental`，不能作为默认光追 GI 声明。
- Acceleration structure/inline ray query 仅在 capability summary 明确支持时可 admission。

## 性能建议

- 静态环境优先复用 bake artifact，避免运行时重复 PMREM/SH 计算。
- Reflection/planar probe 使用按需更新或分帧预算。
- Shadow resolution 与级联数直接影响 atlas 和 draw 工作量；大量 point light 成本尤其高。
- 实时 IBL timing/query 是诊断路径，发布版本按需开启。
