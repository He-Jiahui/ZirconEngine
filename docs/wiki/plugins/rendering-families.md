---
related_code:
  - zircon_plugins/rendering/runtime/src/lib.rs
  - zircon_plugins/rendering/features
  - zircon_plugins/neural/runtime/src/lib.rs
  - zircon_plugins/hybrid_gi/runtime/src/lib.rs
  - zircon_plugins/virtual_geometry/runtime/src/lib.rs
  - zircon_plugins/solari/runtime/src/lib.rs
implementation_files:
  - zircon_plugins/rendering/runtime/src
  - zircon_plugins/rendering/features
  - zircon_plugins/neural/runtime/src
  - zircon_plugins/hybrid_gi/runtime/src
  - zircon_plugins/virtual_geometry/runtime/src
  - zircon_plugins/solari/runtime/src
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - docs/plans/zircon_plugins/01
tests:
  - zircon_plugins/rendering
  - zircon_plugins/hybrid_gi
  - zircon_plugins/virtual_geometry
doc_type: module-detail
title: 渲染与高级图形插件族
status: source-audited
---

# 渲染与高级图形插件族

`rendering` 是 stable 的 umbrella 插件，主 capability 标记 complete。它拥有可选渲染 feature 的清单与依赖关系；具体 pass、executor、资源和 editor capability 分散在 feature crate。`hybrid_gi`、`virtual_geometry`、`solari` 和 `neural` 是独立 experimental 包，不属于默认渲染保证。

## Rendering 主插件

`RenderingFeatureKind` 当前枚举 15 项：Post Process、SSAO、Contact Shadow、Volumetric Fog、OIT、Light Cookies、Irradiance Volumes、Planar Reflections、Subsurface Scattering、Decals、Reflection Probes、Baked Lighting、Ray Tracing Policy、Shader Graph、VFX Graph。

只有 `post_process`、`reflection_probes`、`baked_lighting` 默认启用，其余需要显式选择。`feature_manifest(kind)` 生成 `rendering.<suffix>` feature ID、runtime/editor capability 和对应模块。VFX Graph 额外强依赖 `particles` 和 `rendering.shader_graph`。

```rust
use zircon_plugin_rendering_runtime::{
    feature_manifest, RenderingFeatureKind, RENDERING_FEATURES,
};

for feature in RENDERING_FEATURES {
    let manifest = feature_manifest(*feature);
    // 根据项目配置选择 manifest，而不是假定全部默认开启。
    let _ = manifest;
}
```

## Feature 功能表

| Feature | 当前源码职责 |
|---|---|
| Post Process | 后处理图与资源命名、历史资源、pass 调度 |
| SSAO | 屏幕空间环境遮蔽，compute/合成路径 |
| Contact Shadow | 近场接触阴影 feature 与 editor 设置 |
| Volumetric Fog | froxel media inject、light scatter、integrate 三阶段 |
| OIT | 次序无关透明度资源与 pass |
| Light Cookies | 灯光 cookie 数据和 shading 集成 |
| Irradiance Volumes | irradiance volume 描述与采样路径 |
| Planar Reflections | 平面反射采集/合成 |
| Subsurface Scattering | 自定义 shading model，deferred setup/scatter/recombine；forward 回退 standard PBR |
| Decals | decal 数据与渲染 pass |
| Reflection Probes | 反射探针采集、资源和默认 feature |
| Baked Lighting | 烘焙光照资源/集成和默认 feature |
| Ray Tracing Policy | 光追能力/回退策略，不等于具体 GI 实现 |
| Shader Graph | shader graph 资产编译与材质路径 |
| VFX Graph | 粒子 simulation compute + transparent pass，依赖 particles/shader graph |

feature crate 通常导出 `runtime_plugin_feature()`、`plugin_feature_registration()`、`feature_manifest()`、`render_feature_descriptor()` 和 `render_pass_executor_registrations()`。调用方应注册 feature 报告，让 render graph 获取显式读写资源和 queue lane；不要手工直接执行 pass。

部分 executor 仍可能是 noop 或基础实现。例如 VFX Graph 源码的两个 executor 当前返回成功但不执行实际工作；它有资产校验和 workload 描述，但应视为实验路径，而非完成的 GPU 粒子产品。

## Neural

`neural`（experimental，partial）提供 `.zrnn` 模型资产/格式/校验、算子图、CPU interpreter、GPU graph executor、tensor layout、weight upload，以及 editor 侧 ONNX reader/converter 和 `zr_onnx_convert` 工具。权重对齐常量 `NN_WEIGHT_ALIGNMENT` 为 256。

主要 API：`NnModelAsset`、`NnModelFormat`、`NnGraph`/模型验证错误、`NnOp`/`NnOpCode` 及 Conv2d/Gemm/Pool attrs、`run_cpu()`、GPU graph executor 与 weight upload 类型。CPU 与 GPU 支持范围必须按具体 op 验证，模型能够被解析不代表所有算子可执行。

`neural.features.post_process` 是独立 feature，将模型接入后处理链；需要 neural 主能力及渲染上下文。包声明的 `runtime.asset.neural_model` 仍为 partial。

## Hybrid GI

`hybrid_gi`（experimental，advanced capability partial）实现探针/trace region 请求、预算与驻留、场景数据映射、prepare frame、GPU 资源/dispatch/readback、pending completion、radiance cache 与 surface-cache/scene fallback 路径。它通过 `PluginHybridGiRuntimeProvider` 注册高级 GI provider。

公开入口为 `module_descriptor()`、`render_feature_descriptor()`、`render_pass_executor_registrations()`、`runtime_prepare_collector_registration()` 和 `hybrid_gi_runtime_provider_registration()`. 调用方应通过 provider 注册接入，不应直接操纵内部 residency map。

## Virtual Geometry

`virtual_geometry`（experimental，advanced capability partial）实现层次节点/cluster 数据、page request、slot allocator、resident/pending/eviction 状态、frame preparation、GPU uploader/culling/readback、indirect draw 与 Nanite 风格 CPU reference/自动提取。清单声明自定义 geometry source ID 4、WGSL include、顶点属性和 pages/clusters 绑定。

主要入口与 Hybrid GI 对称：`PluginVirtualGeometryRuntimeProvider`、`virtual_geometry_runtime_provider_registration()`、render feature/executor 注册、runtime prepare collector。geometry source token `custom:virtual_geometry` 和 ID 是 shader permutation 契约，不能在资产与运行时两侧分别修改。

## Solari

`solari`（experimental，partial）目前声明 realtime ray-traced lighting provider 契约，但清单明确标注 pass executor 尚未实现。它适合能力协商和模块边界验证，不应在产品说明中称为可用的实时光追照明器。

## 能力与回退

Advanced render profile 可以选择 Hybrid GI 或 Virtual Geometry provider，但默认 profile 不要求它们。Ray tracing policy 负责决定硬件/软件/关闭路径；feature 必须为能力缺失定义显式禁用或 fallback。Subsurface Scattering 已定义 forward StandardPBR fallback，Solari 则应直接标为不可执行，而不是伪造成功帧。

## Render graph 接入原则

- 用 `RenderFeatureDescriptor` 声明 stage pass、queue lane 和资源读写。
- 用 executor registration 将 executor ID 绑定到实现。
- compute workload 要声明 pipeline label、workgroup size 与 dispatch extent。
- 持久历史资源必须在 schema 中声明 persistent，不能依靠全局缓存隐式存活。
- 异步 GPU readback 的库卸载与 frame completion 必须同步；不得让回调指向已卸载插件。
