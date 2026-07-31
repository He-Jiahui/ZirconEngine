---
related_code:
  - zircon_runtime/src/core/framework/render/environment/source_cubemap.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap/pmrem.rs
  - zircon_runtime/src/core/framework/render/environment/ibl_bake_artifact.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_compute_executor.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export/frame_assertions.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export/scene_fixtures.rs
  - zircon_runtime/src/core/framework/render/material/standard_material.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_material_uniform/gpu_material_uniform_resource.rs
  - zircon_runtime/src/graphics/shader/template/material_surface.rs
  - zircon_runtime/src/graphics/shader/wgsl/zr_gbuffer_encode_standard_pbr.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/shaders/deferred_lighting.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/post_process_screen_space_reflection.wgsl
  - zircon_runtime/src/core/framework/render/light/snapshots.rs
  - zircon_runtime/src/core/framework/render/environment/cubemap_projection.rs
  - zircon_runtime/tests/runtime_environment_cubemap_projection_contract.rs
  - zircon_runtime/src/core/framework/render/image/dimension.rs
  - zircon_runtime/src/asset/assets/texture/descriptor.rs
  - zircon_runtime/src/asset/assets/texture/upload_support.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_texture/gpu_texture_resource_from_asset.rs
  - zircon_runtime/tests/runtime_texture_cube_resource_contract.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/mesh.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_texture/mod.rs
  - zircon_plugins/rendering/plugin.toml
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ReflectionEnvironment.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ReflectionEnvironmentCapture.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ReflectionEnvironmentDiffuseIrradiance.cpp
  - dev/UnrealEngine/Engine/Source/Developer/TextureCompressor/Private/TextureCompressorModule.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/Passes/DrawSkyboxPass.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Lighting/ProbeVolume/ProbeReferenceVolume.cs
  - dev/bevy/crates/bevy_core_pipeline/src/skybox/mod.rs
  - dev/bevy/crates/bevy_light/src/probe.rs
  - dev/bevy/crates/bevy_pbr/src/light_probe/mod.rs
  - dev/bevy/crates/bevy_pbr/src/light_probe/environment_map.rs
  - dev/bevy/crates/bevy_pbr/src/light_probe/generate.rs
  - dev/bevy/crates/bevy_pbr/src/light_probe/irradiance_volume.rs
  - dev/bevy/crates/bevy_pbr/src/lightmap/mod.rs
  - dev/bevy/crates/bevy_pbr/src/fog.rs
  - dev/Fyrox/fyrox-impl/src/scene/probe.rs
plan_sources:
  - .codex/plans/Rendering 插件选项补齐计划.md
  - .codex/plans/Hybrid GI Lumen-Style V1 三阶段计划.md
---

# 计划 11:环境光照(skybox / 反射探针 / 烘焙 / 雾)

## 目标

补齐场景级环境光照资产链,与 HGI 动态 GI 形成"烘焙基线 + 动态增强"的两档结构:

1. skybox/cubemap:天空盒材质域(程序化渐变 + cubemap 两种)、preview-sky 收编、IBL 预滤波(specular mip 链 + irradiance SH)。
2. 反射探针:box/sphere 影响域、烘焙捕获、box projection 校正、探针混合与 fallback 到 skybox;接入既有 rendering.reflection_probes 插件 feature。
3. 光照烘焙:lightmap(2U 通道、烘焙图集)与 light probe(SH L2)采样路径定稿;烘焙器本体走离线工具/插件,runtime 只定消费契约;接入 rendering.baked_lighting feature。
4. 解析雾:线性/指数/指数平方 + 高度雾,场景级设置(可被 Volume 覆写,计划 07 协同);与后处理屏幕空间雾区分:解析雾在前向着色/deferred 合成中计算。
5. ambient:平坦色 / 三段渐变 / SH 环境光三模式。

## 现状与差距

- 有 preview-sky executor(预览天空),但无正式 skybox 材质域、无 IBL 预滤波链;cubemap 资产格式未定。
- reflection_probes/baked_lighting 插件 feature 是描述符与合成占位,无捕获/烘焙数据链与采样 ABI。
- 无解析雾;ambient 只有单色。

## 参考代码

| 文件 | 应重点阅读 |
|------|-----------|
| `dev/Graphics/.../Runtime/Passes/DrawSkyboxPass.cs` | skybox 作为 opaque 末尾 pass 的插入位置(early-z 受益)与深度处理 |
| `dev/UnrealEngine/.../Renderer/Private/ReflectionEnvironmentCapture.cpp` | 探针捕获:cubemap 渲染 → mip 预滤波(GGX 重要性采样)→ diffuse irradiance 提取 |
| `dev/UnrealEngine/.../Developer/TextureCompressor/Private/TextureCompressorModule.cpp` | equirect/long-lat → source cubemap mip0、普通 texture mip、angular filtered cubemap mip 的边界;source mip pyramid 不等同 PMREM |
| `dev/UnrealEngine/.../Renderer/Private/ReflectionEnvironment.cpp` | 探针混合与按距离/影响域排序合成;skybox fallback |
| `dev/Graphics/.../core/Runtime/Lighting/ProbeVolume/ProbeReferenceVolume.cs` | probe volume 的索引与插值组织(远期 probe GI 参考,本计划只取 SH 采样 ABI) |

次参考:`dev/bevy/crates/bevy_pbr`(environment map light 的 wgsl 采样与 binding 组织)。

**Rust/wgpu 落地参照(防凭空实现)**:

| 文件 | 对应本计划机制 | 应重点阅读 |
|------|---------------|-----------|
| `dev/bevy/crates/bevy_core_pipeline/src/skybox/mod.rs` | skybox pass(Cubemap 模式) | `SkyboxUniforms`(:89)/`SkyboxPipeline` 与 `prepare_skybox_pipelines`/`prepare_skybox_bind_groups`:cubemap 天空的 pipeline/深度策略组织(配同目录 `skybox.wgsl`) |
| `dev/bevy/crates/bevy_light/src/probe.rs` | `ReflectionProbeData` 契约面 | `LightProbe`(:75)影响域组件 + `EnvironmentMapLight`(diffuse/specular 双贴图、intensity、旋转)与 `GeneratedEnvironmentMapLight`(:260)字段清单 |
| `dev/bevy/crates/bevy_pbr/src/light_probe/mod.rs` | 探针选择与 per-view 上传 | `LightProbesUniform`(:122)/`RenderViewLightProbes`(:255):view 内探针收集、裁剪进 uniform 的 Rust 全链(对应"extract 端按 layer/距离裁到 64") |
| `dev/bevy/crates/bevy_pbr/src/light_probe/environment_map.rs` | `GpuEnvironmentMap` 绑定 | `EnvironmentMapUniform` 与 binding 组织(rotation 采样期生效,与 `IblBakeKey` 排除项同思路;配 `environment_map.wgsl` 采样) |
| `dev/bevy/crates/bevy_pbr/src/light_probe/generate.rs` | IBL 预滤波 compute(`ibl_prefilter.rs` 直接同类) | 运行时 GGX specular mip 链卷积 + irradiance 生成的 compute 编排、radiance/irradiance 双 bind layout(:73)(配 `environment_filter.wgsl`/`copy.wgsl`) |
| `dev/bevy/crates/bevy_pbr/src/light_probe/irradiance_volume.rs` | `LightProbeGridData`(V1 均匀网格) | 体素化 probe 网格的 GPU 资源与 per-view bind group(`RenderViewIrradianceVolumeBindGroupEntries` :165) |
| `dev/bevy/crates/bevy_pbr/src/lightmap/mod.rs` | lightmap 消费契约(EL-M3) | lightmap slot/uv_rect 实例数据组织与采样接线(配 `lightmap.wgsl`)—— `LightmapInstanceSlot` 的 Rust 同型 |
| `dev/bevy/crates/bevy_pbr/src/fog.rs` | `FogSettings` 距离雾三模式 | `DistanceFog`(:52)/`FogFalloff`(Linear/Exponential/ExponentialSquared,:17–19)参数契约与文档内衰减公式 |
| `dev/Fyrox/fyrox-impl/src/scene/probe.rs` | 探针捕获(EL-M2)与 ambient 来源 | `ReflectionProbe` 场景节点:捕获 cube 纹理 + `EnvironmentLightingSource`(:176)把探针作为环境光来源的组件契约 |

高度雾(height fog)无 Rust 同类参照(bevy 仅距离/大气雾),实现时以 UE 高度雾为唯一样板,按 index §8 第 8 条配对拍测试先行。

## 目标架构

归属:契约(skybox 设置、探针/lightmap 采样 ABI、雾参数)进 `core/framework/render/`;捕获/预滤波 pass 在 `scene_renderer/` 新增 `environment/`;烘焙器与高级合成留在 rendering 插件。

核心设计:

- `SkyboxSettings`:模式(Procedural | Cubemap)、材质引用、强度/旋转;skybox pass 插在 opaque 之后 transparent 之前(深度 = far,early-z 剔除);preview-sky executor 收编为 Procedural 模式实现,删除独立路径。
- IBL 预滤波:cubemap 导入或捕获后离线/加载期生成 specular mip 链(GGX)与 irradiance SH9;`GpuEnvironmentMap` 资源含两者。
- `ReflectionProbeData`:影响域(box/sphere)、box projection 参数、烘焙 cubemap 引用、优先级;着色端按"对象位置选 ≤2 探针加权 + skybox fallback";探针数据进 GpuScene 风格 buffer(计划 03 模式)。
- lightmap 消费:静态对象 UV2 + 图集页索引(`RendererCommon.is_static` 前提);light probe SH 按 tetrahedral/网格插值留接口,V1 均匀网格。
- `FogSettings`:模式/颜色/密度/起止距离 + 高度雾(基准高度/衰减);WGSL include 同时服务 forward 与 deferred 合成;Volume 可覆写(计划 07 的 volume 组件之一)。

## 里程碑

### EL-M1 skybox 与 IBL

实施切片:
1. `SkyboxSettings` 契约 + skybox pass(两模式);preview-sky 收编。
2. cubemap 资产导入与预滤波(mip 链 + SH);PBR 环境项接 `GpuEnvironmentMap`(替代当前隐式 ambient)。

测试阶段:
- `cargo check -p zircon_runtime --lib --locked`;`cargo test -p zircon_runtime environment --locked` + `render_product` 回归
- 验收证据:金属球在 cubemap 天空下反射正确(抓帧);粗糙度扫描条带对应 mip 链。

### EL-M2 反射探针

实施切片:
1. `ReflectionProbeData` 契约与探针 buffer;着色端双探针混合 + box projection。
2. 捕获 pass(编辑器触发,渲染 6 面 + 预滤波)走 rendering 插件;探针资产持久化。

测试阶段:
- `cargo test -p zircon_runtime environment --locked` 与插件 workspace 测试
- 验收证据:室内/室外两探针交界处反射平滑过渡;feature 关闭时回落 skybox。

### EL-M3 lightmap 与 light probe 消费

实施切片:
1. lightmap 采样 ABI(UV2/图集/编码格式定稿)与 shader 变体 flag(计划 08);probe SH 网格插值。
2. 烘焙器输入输出契约定稿(烘焙实现归 baked_lighting 插件后续计划)。

测试阶段:
- `cargo test -p zircon_runtime environment --locked`(外部烘焙夹具数据渲染对拍)
- 验收证据:夹具 lightmap 场景静态光影正确;动态物从 probe 取间接光。

### EL-M4 解析雾与 ambient 模式

实施切片:
1. `FogSettings` 三模式 + 高度雾;forward/deferred 双路径 include;Volume 覆写组件。
2. ambient 三模式(色/渐变/SH)。

测试阶段:
- `cargo test -p zircon_runtime environment --locked`(雾衰减曲线 readback 断言)
- 验收证据:距离/高度雾抓帧;deferred 与 forward 雾产物一致。

## 工程落地细化

本章节为计划 11 的实施权威(index.md §8.7)。bind group 槽位、storage buffer std430 基线、WGSL `zr_` include 约定、RenderQueueValue 数值段、测试命名直接引用 index.md §8,不再重述。契约类型全部落在 `zircon_runtime::core::framework::render`(facade 固定,无 wgpu);所有 pass 经 RenderGraph 节点 + executor id 接入,无旁路提交;只消费 `RenderFrameExtract`;烘焙器本体经 rendering 插件;一律硬切换,不留兼容路径。

跨计划契约名原样消费:01 `RgTextureHandle`/`TransientResourcePool`;05 `GpuLightData` 与 `zr_light_grid.wgsl`(probe 数据不进 light grid,走本计划独立 buffer,cluster 化留后续与 05 协调);07 `VolumeComponentDescriptor`/`VolumeEvaluator`(`FogSettings` 注册为 Volume 可覆写组件);13 `TextureMetadata`/`CubemapAsset`(本计划只消费,不定义 cubemap 资产格式);16 `ComputePassDescriptor`(IBL 预滤波与 SH 投影 compute 经它声明)。

### 模块与文件落点

新增文件:

| 路径 | 内容 |
|------|------|
| `zircon_runtime/src/core/framework/render/environment/mod.rs` | 模块声明与 curated re-export(薄) |
| `zircon_runtime/src/core/framework/render/environment/cubemap_projection.rs` | equirect→cube/GGX/SH 测试共用的 CPU 黄金投影:face order、texel center UV、lat-long UV、solid angle |
| `zircon_runtime/src/core/framework/render/environment/skybox.rs` | `SkyboxSettings` / `SkyboxMode` / `ProceduralSkyParams` / `CubemapSkyParams` / `IblBakeKey` |
| `zircon_runtime/src/core/framework/render/environment/reflection_probe.rs` | `ReflectionProbeData` / `ProbeInfluenceShape` / `ProbeBakeTiming` |
| `zircon_runtime/src/core/framework/render/environment/fog.rs` | `FogSettings` / `FogMode` / `HeightFogParams` |
| `zircon_runtime/src/core/framework/render/environment/ambient.rs` | `AmbientMode` / `ShL2Rgb`(SH9 系数容器) |
| `zircon_runtime/src/core/framework/render/environment/lightmap.rs` | `LightmapConsumeContract` / `LightmapInstanceSlot` / `LightProbeGridData` / `LightmapBakeRequest` / `LightmapBakeOutput`(DTO,序列化跨插件边界) |
| `zircon_runtime/src/core/framework/render/environment/extract.rs` | `EnvironmentExtract`(进 `RenderFrameExtract` 的环境快照) |
| `zircon_runtime/src/graphics/scene/scene_renderer/environment/mod.rs` | 实现侧模块声明(薄) |
| `zircon_runtime/src/graphics/scene/scene_renderer/environment/skybox_executor.rs` | skybox pass executor(Procedural / Cubemap 两 pipeline) |
| `zircon_runtime/src/graphics/scene/scene_renderer/environment/gpu_environment_map.rs` | `GpuEnvironmentMap`(specular mip 链 cube 纹理 + SH9 buffer + bake key 缓存) |
| `zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_prefilter.rs` | 预滤波/SH 投影 compute pass 构建(经 `ComputePassDescriptor`) |
| `zircon_runtime/src/graphics/scene/scene_renderer/environment/probe_buffer.rs` | `GpuReflectionProbe` 上传、cube array 槽位分配器 |
| `zircon_runtime/src/graphics/scene/scene_renderer/environment/lightmap_binding.rs` | lightmap atlas / probe grid GPU 资源绑定 |
| `zircon_runtime/src/graphics/scene/scene_renderer/environment/shaders/skybox_procedural.wgsl` | 程序化天空(吸收 `overlay/shaders/sky.wgsl` 渐变逻辑,深度 = far) |
| `zircon_runtime/src/graphics/scene/scene_renderer/environment/shaders/skybox_cubemap.wgsl` | cubemap 天空(rotation/intensity/tint 采样期生效) |
| `zircon_runtime/src/graphics/scene/scene_renderer/environment/shaders/ibl_prefilter.wgsl` | GGX 重要性采样 specular mip 烘焙 compute |
| `zircon_runtime/src/graphics/scene/scene_renderer/environment/shaders/ibl_irradiance_sh.wgsl` | SH9 投影 compute(workgroup 归约) |
| `zircon_runtime/src/graphics/shader/includes/zr_fog.wgsl` | 解析雾函数 include(若计划 08 已定 include 目录则随其落点,内容不变) |
| `zircon_runtime/src/graphics/shader/includes/zr_environment.wgsl` | probe 混合 + box projection + SH 求值 + env specular 采样 include |
| `zircon_runtime/src/graphics/tests/render_product_environment.rs` | 环境光照产物对拍测试 |

修改文件:

| 路径 | 改动 |
|------|------|
| `zircon_runtime/src/core/framework/render/mod.rs` | 声明 `environment` 模块,re-export 契约类型 |
| `zircon_runtime/src/core/framework/render/frame_extract.rs` | `RenderFrameExtract` 增加 `environment: EnvironmentExtract` 字段 |
| `zircon_runtime/src/core/framework/render/scene_extract.rs` | `PreviewEnvironmentExtract` 的 `skybox_enabled`/`fallback_skybox` 字段删除,改引 `EnvironmentExtract` |
| `zircon_runtime/src/core/framework/render/camera.rs` | 删除 `FallbackSkyboxKind`;`ViewportRenderSettings::preview_skybox` 改为映射 `SkyboxSettings` 启停 |
| `zircon_runtime/src/core/framework/render/light/snapshots.rs` | `RenderReflectionProbeSnapshot` 删除(被 `ReflectionProbeData` 取代);`RenderBakedLightingExtract` 扩展为 lightmap/probe grid 引用 |
| `zircon_runtime/src/scene/world/render.rs` | extract 端填充 `EnvironmentExtract`;删除 `FallbackSkyboxKind::ProceduralGradient` 映射(L911–915) |
| `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs` | 注册 `skybox`/`ibl_prefilter`/`ibl_irradiance_sh` executor;删除 `preview_sky_scene_color_executor`/`preview_sky_final_color_executor` 注册 |
| `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs` | 删除 `record_preview_sky_to_resources`/`with_preview_sky_renderer`;新增 environment 资源访问 |
| `zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/mesh.rs`、`deferred_geometry.rs` | preview-sky pass 声明改为 skybox pass 节点(opaque 后、transparent 前) |
| `zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/from_frame.rs` | `authored_ambient_color` 改读 resolved ambient SH band0;删除 preview lighting 的硬编码 ambient 常量分支 |
| `zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl` | shading 末尾接 `zr_apply_fog`;环境项接 `zr_environment.wgsl` |
| `zircon_runtime/src/graphics/scene/scene_renderer/deferred/shaders/deferred_lighting.wgsl` | 合成末尾同上(与 forward 同一 include,产物一致) |
| `zircon_runtime/src/graphics/scene/resources/gpu_texture/mod.rs` | cube / cube array 维度支持(消费计划 13 `CubemapAsset` 上传产物) |
| `zircon_plugins/rendering/plugin.toml` 及 `rendering.reflection_probes.*`、`rendering.baked_lighting.*` crate | 探针捕获调度(渲染 6 面 + 调 runtime 预滤波)、烘焙器接口实现 |

preview-sky 硬切换删除清单(EL-M1 切片 1 同一变更内完成,详见"帧时序与集成点"):`graph_execution/preview_sky_executor.rs`、`overlay/passes/preview_sky_pass.rs`、`overlay/viewport_overlay_renderer/record/scene_content/record_preview_sky.rs`、`overlay/shaders/sky.wgsl`、`overlay/passes/pass_order.rs` 的 `"PreviewSkyPass"` 项、`overlay/viewport_overlay_renderer/construct/new.rs` 的 `preview_sky` 字段。

### 核心类型与接口

契约层(`core/framework/render/environment/`,纯数据可序列化,无 wgpu):

```rust
// skybox.rs
pub enum SkyboxMode {
    Procedural(ProceduralSkyParams),
    Cubemap(CubemapSkyParams),
}

pub struct ProceduralSkyParams {
    pub zenith_color: Vec3,
    pub horizon_color: Vec3,
    pub ground_color: Vec3,
    pub horizon_falloff: Real,     // 渐变指数,默认 1.0
    pub sun_disk_enabled: bool,
    pub sun_disk_size_degrees: Real,
    pub sun_disk_intensity: Real,
}

pub struct CubemapSkyParams {
    pub cubemap: AssetId,          // 计划 13 CubemapAsset 引用
    pub tint: Vec3,
}

pub struct SkyboxSettings {
    pub mode: SkyboxMode,
    pub intensity: Real,           // 采样期乘子,不进 bake key
    pub rotation_y_degrees: Real,  // 采样期旋转方向向量,不进 bake key
    pub ibl_enabled: bool,         // 关闭时环境项退回 ambient 模式
}

/// 重烘判定键:mode 判别值 + Procedural 参数 hash 或 cubemap 内容版本 + 质量档。
/// intensity / rotation 显式排除(采样期生效)。
pub struct IblBakeKey(pub u64);

// reflection_probe.rs
pub enum ProbeInfluenceShape {
    Box { half_extents: Vec3, blend_distance: Real },
    Sphere { radius: Real, blend_distance: Real },
}

pub enum ProbeBakeTiming { EditorManual, RuntimeManual }  // V1 无自动重烘

pub struct ReflectionProbeData {
    pub probe_id: u64,                       // 稳定 id(对齐 05 light id 机制)
    pub position: Vec3,
    pub rotation: Quat,
    pub shape: ProbeInfluenceShape,
    pub box_projection: bool,
    pub projection_half_extents: Vec3,       // 投影盒可与影响域不同
    pub baked_cubemap: Option<AssetId>,      // 预滤波后的 CubemapAsset;None = 未烘焙,着色端跳过
    pub intensity: Real,
    pub priority: i32,                       // 同权重并列时高优先
    pub layer_mask: RenderLayerSet,          // 计划 09 同一 mask
    pub bake_timing: ProbeBakeTiming,
}

// fog.rs
pub enum FogMode {
    None,
    Linear { start: Real, end: Real },
    Exponential { density: Real },
    ExponentialSquared { density: Real },
}

pub struct HeightFogParams {
    pub enabled: bool,
    pub base_height: Real,    // 世界 Y,低于此高度雾全强
    pub falloff: Real,        // 高度衰减系数(1/m)
    pub max_opacity: Real,    // [0,1],雾不透明度上限
}

pub struct FogSettings {
    pub mode: FogMode,
    pub color: Vec3,
    pub height: HeightFogParams,
    pub affects_skybox: bool,
}

// ambient.rs
pub struct ShL2Rgb(pub [Vec3; 9]);   // 9×RGB f32,band 顺序 L00,L1-1,L10,L11,L2-2,L2-1,L20,L21,L22

pub enum AmbientMode {
    Flat { color: Vec3, intensity: Real },                       // 收编 RenderAmbientLightSnapshot
    Gradient { sky: Vec3, equator: Vec3, ground: Vec3, intensity: Real },
    SkyboxSh,                                                    // 消费 GpuEnvironmentMap 的 SH9
}

// lightmap.rs
pub struct LightmapInstanceSlot {
    pub atlas_page: u32,           // Texture2DArrayAsset 切片索引(计划 13)
    pub uv_rect: Vec4,             // scale.xy + offset.xy,UV2 → atlas 页变换
}

pub struct LightmapConsumeContract {
    pub atlas: AssetId,            // RGBA16F Texture2DArrayAsset
    pub slots: Vec<(u64, LightmapInstanceSlot)>,   // renderer 稳定 id → slot
}

pub struct LightProbeGridData {    // V1 均匀网格,接口不锁死插值结构
    pub bounds_min: Vec3,
    pub cell_size: Vec3,
    pub dims: [u32; 3],
    pub sh: Vec<ShL2Rgb>,          // dims.x*y*z 个
}

// extract.rs —— 进 RenderFrameExtract,extract 仅由 runtime 生成
pub struct EnvironmentExtract {
    pub skybox: Option<SkyboxSettings>,
    pub fog: FogSettings,                       // 已经 VolumeEvaluator 解析后的最终值(计划 07)
    pub ambient: AmbientMode,
    pub probes: Vec<ReflectionProbeData>,
    pub baked_lighting: Option<LightmapConsumeContract>,
    pub probe_grid: Option<LightProbeGridData>,
}
```

归属裁决:`FogSettings` 同时注册为计划 07 的 Volume 组件(`VolumeComponentDescriptor` schema 见下表);Volume 求值发生在 extract 之前,`EnvironmentExtract.fog` 已是解析后终值,渲染侧不再做 volume 插值。烘焙器(lightmap/probe 捕获)实现归 `rendering.baked_lighting` / `rendering.reflection_probes` 插件,跨边界只走 `LightmapBakeRequest`/`LightmapBakeOutput` 序列化 DTO(`zircon_runtime_interface` ABI 规则:不传 trait 对象/wgpu 对象)。

`FogSettings` Volume 可覆写字段表(mode 取最高权重 volume 的判别值,不插值;其余 lerp):

| 字段 | 插值 | 默认 |
|------|------|------|
| `color` | lerp | (0.5, 0.6, 0.7) |
| `Linear.start` / `Linear.end` | lerp | 30 / 300 |
| `Exponential.density` / `ExponentialSquared.density` | lerp | 0.01 |
| `height.base_height` / `height.falloff` / `height.max_opacity` | lerp | 0 / 0.1 / 1.0 |
| `mode` / `affects_skybox` / `height.enabled` | 权重最高者 | None / true / false |

### GPU 数据布局与 WGSL 约定

binding 编号:环境段固定占用 `group1`(pass 级)的 `@binding(16..23)` 区段,与计划 05 light grid/shadow 的低位段错开;material(`group2`)、instance(`group3`)不变。

| group/binding | 资源 | 类型 |
|---|---|---|
| 1/16 | `zr_env_probes` | `var<storage, read> array<GpuReflectionProbe>` |
| 1/17 | `zr_env_probe_header` | uniform(`probe_count: u32` + pad) |
| 1/18 | `zr_env_probe_cubemaps` | `texture_cube_array<f32>`(预滤波 mip 链) |
| 1/19 | `zr_env_sky_specular` | `texture_cube<f32>`(skybox 预滤波 mip 链,fallback 源) |
| 1/20 | `zr_env_sampler` | `sampler`(linear,clamp) |
| 1/21 | `zr_env_sh` | `var<storage, read> array<vec4<f32>, 9>`(active ambient SH9) |
| 1/22 | `zr_fog_uniform` | uniform `ZrFogParams` |
| 1/23 | 预留(probe SH grid buffer,EL-M3) | storage |

`GpuReflectionProbe`(std430,96 bytes,offset 注释):

```wgsl
struct GpuReflectionProbe {
    position_blend: vec4<f32>,  // 0:  xyz=position, w=blend_distance
    box_min: vec4<f32>,         // 16: xyz=influence min(世界轴对齐前的本地),w=f32(priority)
    box_max: vec4<f32>,         // 32: xyz=influence max, w=shape(0=box, 1=sphere)
    proj_params: vec4<f32>,     // 48: xyz=projection_half_extents, w=box_projection(0/1)
    rotation: vec4<f32>,        // 64: 单位四元数(世界→probe 本地)
    misc: vec4<f32>,            // 80: x=intensity, y=mip_count, z=array_slice, w=bitcast(layer_mask)
};                              // total 96
```

SH9 存储:9×`vec4<f32>`(RGB + pad)= 144 bytes,storage buffer;CPU 侧 `ShL2Rgb` 与之一一对应,投影 compute 输出后异步 readback 回写资产持久化(探针/天空共用同一布局)。

`zr_fog.wgsl`(URP `unity_FogParams` 打包思路,见精读笔记):

```wgsl
struct ZrFogParams {
    color: vec4<f32>,          // rgb=fog color, a=affects_skybox(0/1)
    params: vec4<f32>,         // mode 打包:Linear: z=-1/(end-start), w=end/(end-start)
                               //           Exp/Exp2: x=density/ln(2)(exp2 域)
    height_params: vec4<f32>,  // x=base_height, y=falloff, z=max_opacity, w=enabled
    mode: vec4<u32>,           // x=FogMode 判别值(0/1/2/3)
};

// 返回"剩余透过率"(1=无雾,0=全雾),与 URP fogIntensity 同语义
fn zr_fog_factor(view_distance: f32, world_height: f32, fog: ZrFogParams) -> f32;
fn zr_apply_fog(color: vec3<f32>, view_distance: f32, world_pos: vec3<f32>,
                fog: ZrFogParams) -> vec3<f32>;   // mix(fog.color.rgb, color, factor)
```

公式定稿:Linear `f = saturate(d * params.z + params.w)`;Exp `f = saturate(exp2(-density_ln2 * d))`;Exp2 `f = saturate(exp2(-(density_ln2 * d)^2))`;height fog 乘项 `h = mix(1.0, saturate(exp(-(world_height - base_height) * falloff)), height.enabled)`,合成 `f_final = max(1 - (1 - f * (1 - h * (1 - f))), 1 - max_opacity)` 简化实现为:`opacity = (1 - f) * h_weight`,`opacity = min(opacity, max_opacity)`,`return 1 - opacity`(单测对拍闭式)。

box projection(`zr_environment.wgsl`,UE `GetLookupVectorForBoxCapture` 的 AABB 求交式):

```wgsl
fn zr_box_project(reflect_dir: vec3<f32>, world_pos: vec3<f32>,
                  probe: GpuReflectionProbe) -> vec3<f32> {
    let p = zr_quat_rotate_inv(probe.rotation, world_pos - probe.position_blend.xyz);
    let d = zr_quat_rotate_inv(probe.rotation, reflect_dir);
    let inv_d = 1.0 / d;
    let t1 = (-probe.proj_params.xyz - p) * inv_d;   // 三个 min 面交点
    let t2 = ( probe.proj_params.xyz - p) * inv_d;   // 三个 max 面交点
    let t_far = max(t1, t2);
    let t = min(t_far.x, min(t_far.y, t_far.z));     // 最近的"最远面"
    let hit = world_pos + t * reflect_dir;
    return hit - probe.position_blend.xyz;           // 修正后的采样向量
}
```

≤2 探针混合权重(URP `CalculateProbeWeight` 式边缘权重 + 排序截断):每像素全量扫 probe buffer(V1 上限 64,extract 端按 layer/距离裁到该数),单 probe 权重 `w = saturate(min_axis((p_edge_dist) / blend_distance))`(box 取三轴边距最小值,sphere 取 `(radius - dist) / blend_distance`);取权重最大两个(权重并列比 priority),`w1 = min(w1, 1 - w0)`,skybox fallback 权重 `w_sky = 1 - w0 - w1`;三项各按自身校正向量采 `mip = zr_env_mip_from_roughness(roughness)` 后线性加权。roughness→mip 采用 UE 对数映射(常数 `ZR_IBL_ROUGHEST_MIP = 1.0`、`ZR_IBL_MIP_SCALE = 1.2`):`mip = max_mip - 1 - (ZR_IBL_ROUGHEST_MIP - ZR_IBL_MIP_SCALE * log2(max(roughness, 0.001)))`;烘焙端按逆映射给每 mip 的 roughness,保证 mip 数无关的粗糙度一致性。

IBL 预滤波样本数档位表(`ibl_prefilter.wgsl` 特化常量):

| 档位 | mip0 | 中段 mip | 末两级 mip | 用途 |
|------|------|---------|-----------|------|
| Fast | 直拷 | 32 | 64 | 编辑器拖参数实时迭代 |
| Normal | 直拷 | 64 | 128 | 默认导入/捕获 |
| High | 直拷 | 128 | 256 | 显式高质量重烘 |

cubemap 基准 128×128×6,mip 数 = `log2(128)+1 = 8`(UE `GetNumMips = CeilLogTwo(CaptureSize)+1` 同式);SH9 投影 compute 按面分 workgroup 归约(立体角加权),输出 9×vec4 storage。

### 帧时序与集成点

帧内顺序(全部经 graph 节点声明,executor id 注册在 `render_pass_executor_registry.rs`):

1. **prepare 阶段**:`gpu_environment_map.rs` 比对 `IblBakeKey`,脏则经 `ComputePassDescriptor` 入队 `env.ibl_prefilter`(逐 mip 一 dispatch)+ `env.ibl_irradiance_sh` 两个 compute 节点;Procedural 模式先跑 `env.sky_capture`(渐变烘成 cubemap mip0)再预滤波。重烘当帧完成(128³ 成本低);probe 捕获(6 面渲染)只由编辑器/运行期手动触发,经 `rendering.reflection_probes` 插件 feature 的 capture 调度复用同一预滤波节点。
2. **skybox pass**:节点声明 `SCENE_COLOR` write + `SCENE_DEPTH` read(depth test `LessEqual`,depth write off,全屏三角形顶点深度置 far)——插在 opaque 之后、transparent 之前(URP `DrawSkyboxPass` 同位,享受 early-z);Procedural/Cubemap 各一 pipeline,按 `SkyboxMode` 选。`EnvironmentExtract.skybox == None` 时 compiled graph 不含该节点(约束 4)。
3. **shading 中的环境项**:forward(`fallback_mesh.wgsl` 与计划 08 模板)与 deferred lighting 合成共用 `zr_environment.wgsl`:specular = probe 混合 + skybox fallback;diffuse ambient = `zr_env_sh` 求值(三模式统一为 SH9:Flat 只写 band0 = color×intensity;Gradient 用解析投影闭式写 L0+L1;SkyboxSh 直接用预滤波产物 SH)。shader 内无 ambient 模式分支。
4. **雾**:`zr_apply_fog` 在 forward shading 输出前、deferred lighting 合成输出前各调用一次(同一 include 保证产物一致);skybox pass 按 `affects_skybox` 在自身输出端调用(view_distance 取 z_far)。**与计划 07 边界**:本计划的解析雾/高度雾发生在 shading/lighting pass 内,不新增 post pass;07 的屏幕空间雾(散射/体积感,SSR 槽位附近)是独立 post 节点,消费同一 `FogSettings.color` 但有独立强度参数,两者共存时由 07 的 volume stack 管理开关,本计划不做屏幕空间合成。

preview-sky 硬切换(EL-M1 切片 1,同一变更):删除"模块与文件落点"清单所列 6 个文件/条目,并同步删除 `render_pass_execution_context/gpu.rs` 的 `record_preview_sky_to_resources`/`with_preview_sky_renderer`、`camera.rs` 的 `FallbackSkyboxKind`、`scene/world/render.rs` 的映射分支与 `scene_extract.rs` 的 `skybox_enabled`/`fallback_skybox` 字段;`ViewportRenderSettings::preview_skybox = true` 改为 extract 出默认 `SkyboxSettings { mode: Procedural(默认渐变), .. }`,编辑器预览行为不变。受影响测试(`graphics/tests/pipeline_compile.rs`、`scene_overlay.rs`、`scene/tests/render_extract.rs` 等)同变更内改断言,不留旧符号。

GPUScene 衔接(计划 03):`LightmapInstanceSlot`(uv_rect + atlas_page)进 instance 数据扩展位;`RendererCommon.is_static` 为 lightmap 采样前提,shader 变体 flag(`LIGHTMAP_ON` 等价物)走计划 08 permutation。

### 实施切片细化

**EL-M1 / 切片 1 —— SkyboxSettings 契约 + skybox pass + preview-sky 收编**
- 触碰:新增 `environment/skybox.rs`、`environment/extract.rs`、`skybox_executor.rs`、两个 skybox wgsl;修改 `mod.rs`、`frame_extract.rs`、`scene_extract.rs`、`camera.rs`、`scene/world/render.rs`、executor registry、`feature_descriptors/{mesh,deferred_geometry}.rs`;删除 preview-sky 清单全部文件。
- 要点:pass 位置与深度策略按"帧时序"§2;Procedural 渐变公式从 `sky.wgsl` 平移并参数化(zenith/horizon/ground)。
- 完成判据:`cargo check -p zircon_runtime --lib --locked` 过;仓库内 `preview_sky` 零命中(`grep -r preview_sky zircon_runtime/src` 为空);编辑器视口天空渲染行为与改前一致。

**EL-M1 / 切片 2 —— IBL 预滤波 + GpuEnvironmentMap 接 PBR 环境项**
- 触碰:新增 `gpu_environment_map.rs`、`ibl_prefilter.rs`、`ibl_prefilter.wgsl`、`ibl_irradiance_sh.wgsl`、`zr_environment.wgsl`;修改 `gpu_texture/mod.rs`(cube 维度)、`fallback_mesh.wgsl`、`deferred_lighting.wgsl`、`scene_uniform/from_frame.rs`。
- 要点:bake key 判定与档位表;roughness→mip 常数定稿;环境 specular/diffuse 替代现有隐式 ambient 常量。依赖计划 13 TX-M3 的 `CubemapAsset` 导入(联调用例共享)。
- 完成判据:粗糙度 0→1 扫描条带对应 mip 链(抓帧);`render_env_*` M1 测试组绿。

**EL-M2 / 切片 1 —— ReflectionProbeData 契约 + probe buffer + 着色混合**
- 触碰:新增 `environment/reflection_probe.rs`、`probe_buffer.rs`;修改 `light/snapshots.rs`(删 `RenderReflectionProbeSnapshot`)、`extract.rs`、`zr_environment.wgsl`(混合 + box projection)、binding 16–18 接线。
- 要点:96-byte 布局与 offset 断言;≤2 混合 + fallback 权重法;cube array 槽位分配器(2 的幂槽,LRU 驱逐)。
- 完成判据:CPU 参考实现与 WGSL 权重/投影对拍;`render_probe_*` M2 单测绿。

**EL-M2 / 切片 2 —— 捕获 pass 经 rendering 插件 + 探针资产持久化**
- 触碰:`zircon_plugins/rendering` 的 reflection_probes runtime/editor crate、`plugin.toml`;runtime 侧暴露捕获请求入口(序列化 DTO)与预滤波节点复用。
- 要点:编辑器触发渲染 6 面(临时 RT 经 `TransientResourcePool`)→ 预滤波 → 写 `CubemapAsset`;feature 关闭时 graph 无捕获节点且着色回落 skybox。
- 完成判据:插件 workspace 测试绿;两探针交界过渡平滑(产物对拍);关 feature 后 graph dump 无 probe pass。

**EL-M3 / 切片 1 —— lightmap 采样 ABI + probe grid 插值**
- 触碰:新增 `environment/lightmap.rs`、`lightmap_binding.rs`;GPUScene instance 扩展(协调计划 03);shader 变体 flag(协调计划 08);binding 23 probe grid buffer。
- 要点:UV2 = 顶点第二 UV 通道,`uv_rect` 变换定稿;atlas 用 RGBA16F `Texture2DArrayAsset`(encoding 结论见下);probe grid GPU 端 trilinear(对象用 instance 世界位置采样,避免逐实例 CPU 上传)。
- 完成判据:外部烘焙夹具数据渲染对拍;`render_env_lightmap_*` 测试绿。
- encoding 结论:**RGBA16F(half)**。理由:wgpu 各后端对 `rgba16float` 过滤采样全覆盖、无 RGBM 解码乘加与低亮度 banding、HDR 余量足;RGBM8 仅作未来移动档可选项,不进 V1;BC6H 压缩留给 importer 插件后续转码,消费 ABI 不变。

**EL-M3 / 切片 2 —— 烘焙器输入输出契约定稿**
- 触碰:`environment/lightmap.rs` 的 `LightmapBakeRequest { scene_snapshot, atlas_budget, texel_density }` / `LightmapBakeOutput { atlas_pages, slots, probe_grid }` DTO;`rendering.baked_lighting` 插件 manifest 声明烘焙 capability。
- 要点:runtime 不含烘焙实现;DTO 序列化 round-trip 测试;atlas 分配(skyline/shelf 二选一,插件内实现,runtime 只认 `uv_rect`)。
- 完成判据:DTO round-trip + 契约静态断言绿;夹具 `LightmapBakeOutput` 喂给 runtime 渲染正确。

**EL-M4 / 切片 1 —— FogSettings + zr_fog.wgsl + Volume 覆写**
- 触碰:新增 `environment/fog.rs`、`zr_fog.wgsl`;修改两条 shading 路径 wgsl、skybox wgsl、binding 22;向计划 07 注册 fog 的 `VolumeComponentDescriptor` schema(覆写字段表见上)。
- 要点:三模式 + 高度雾公式定稿(见 GPU 约定节);extract 携带解析后终值。
- 完成判据:`render_fog_*` 闭式对拍绿;forward/deferred 雾产物一致(对拍);相机进雾 volume 时参数平滑过渡(07 的求值测试覆盖,本计划只消费)。

**EL-M4 / 切片 2 —— ambient 三模式**
- 触碰:`environment/ambient.rs`、`scene_uniform/from_frame.rs`、`zr_environment.wgsl` SH 求值、extract 端 `RenderAmbientLightSnapshot` → `AmbientMode::Flat` 收编。
- 要点:三模式统一 SH9 表示(Flat=band0、Gradient=解析 L0+L1 投影、SkyboxSh=预滤波产物);shader 单路径。
- 完成判据:三模式切换产物差异符合预期(对拍);`render_env_ambient_*` 绿。

### 测试与验收清单

单元测试(随实现文件就近放 `#[cfg(test)]`,模块过滤词 `environment`):

| 测试函数 | 断言 | 位置 |
|---|---|---|
| `render_env_bake_key_ignores_intensity_and_rotation` | 仅 intensity/rotation 变化时 `IblBakeKey` 不变;渐变参数/cubemap 版本变化时改变 | `environment/skybox.rs` |
| `render_env_mip_from_roughness_roundtrip` | 烘焙端逆映射与采样端映射互逆(8 mip 全档,误差 < 1e-4) | `ibl_prefilter.rs` |
| `render_env_sh9_constant_color_projects_to_band0_only` | 常色 cubemap 投影后 L1/L2 系数 ≈ 0,band0 = 颜色×归一化常数 | `ibl_prefilter.rs`(CPU 参考)|
| `render_env_ambient_gradient_sh_matches_analytic` | Gradient 闭式投影与数值积分对拍 | `environment/ambient.rs` |
| `render_env_extract_skybox_none_removes_graph_node` | `skybox: None` 时 compiled graph 无 skybox 节点 | `graphics/tests/pipeline_compile.rs` |
| `render_probe_gpu_layout_is_96_bytes_with_documented_offsets` | `size_of::<GpuReflectionProbe>() == 96` + 各字段 offset 静态断言 | `probe_buffer.rs` |
| `render_probe_weight_box_edge_matches_blend_distance` | 边缘内 blend_distance 处权重线性 0→1,中心饱和 1 | `probe_buffer.rs`(CPU 参考)|
| `render_probe_two_probe_blend_weights_sum_to_one_with_sky_fallback` | w0+w1+w_sky == 1;无探针时 w_sky == 1 | `probe_buffer.rs` |
| `render_probe_box_projection_axis_ray_hits_face_center` | 盒心沿 +X 反射,校正向量命中 +X 面中心(CPU 复刻 WGSL 公式) | `probe_buffer.rs` |
| `render_probe_slot_allocator_evicts_lru_on_pressure` | cube array 槽位满时按 LRU 驱逐且老句柄失效 | `probe_buffer.rs` |
| `render_env_lightmap_uv_rect_transform_roundtrip` | UV2 经 `uv_rect` 变换落在页内,逆变换还原 | `environment/lightmap.rs` |
| `render_env_lightmap_bake_dto_serde_roundtrip` | `LightmapBakeRequest`/`Output` 序列化 round-trip 等值 | `environment/lightmap.rs` |
| `render_env_probe_grid_trilinear_center_equals_cell_average` | 网格中心插值 = 8 邻 cell 均值 | `environment/lightmap.rs` |
| `render_fog_linear_factor_matches_closed_form` | 与 `(end-d)/(end-start)` 闭式逐点对拍(start/end/边界外) | `environment/fog.rs` |
| `render_fog_exp2_monotonic_and_clamped` | 距离单调递减、[0,1] 夹紧 | `environment/fog.rs` |
| `render_fog_height_max_opacity_clamps` | 深谷远距下不透明度 ≤ max_opacity | `environment/fog.rs` |
| `render_fog_volume_schema_fields_match_contract` | 注册到 07 的 schema 字段集与覆写字段表一致 | `environment/fog.rs` |

产物对拍(`zircon_runtime/src/graphics/tests/render_product_environment.rs`,配合 `ZR_RENDERDOC_CAPTURE_NEXT=1` 人工抓帧):

| 场景 | 断言 |
|---|---|
| `render_product_environment_skybox_procedural_after_opaque` | skybox 像素只出现在深度 == far 区域(early-z 生效);与旧 preview-sky 基线图容差内一致 |
| `render_product_environment_cubemap_roughness_ladder` | 金属球粗糙度阶梯反射模糊单调,相邻档位 SSIM 阈值 |
| `render_product_probe_blend_boundary_smooth` | 两探针交界采样带无突变(逐行梯度上限断言) |
| `render_product_probe_feature_off_falls_back_to_skybox` | 关 `rendering.reflection_probes` feature 后产物 == 纯 skybox 基线 |
| `render_product_fog_forward_deferred_consistent` | 同场景两管线雾产物逐像素容差内一致 |
| `render_product_environment_lightmap_fixture` | 烘焙夹具场景与参考图对拍;动态物 ambient 来自 probe grid |

里程碑命令:切片期 `cargo check -p zircon_runtime --lib --locked`;测试阶段 `cargo test -p zircon_runtime environment --locked`、`cargo test -p zircon_runtime render_product_environment --locked`;插件接缝 `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_rendering_reflection_probes_runtime --locked`(crate 名见 `zircon_plugins/rendering/plugin.toml`)。

## 状态与产出记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

本子计划产出记录已超过 10 条，具体记录已迁入编号子目录。

- 当前里程碑概述：EL-M3 lightmap 与 light probe 消费已于 2026-07-13 完成；外部 bake fixture、Forward+/Deferred WGPU 数值对拍和 PNG 证据均已闭合。HGI 动态增量合成归 HGI-M4，不计入 EL-M3。

- 迁入记录：[`11/2026-07-09-environment-lighting-output-records.md`](11/2026-07-09-environment-lighting-output-records.md)
- fixed 已修复：[reflection-probe-product-type-inference](../text/01/fixed-2026-07-12-reflection-probe-product-type-inference.md)
- fixed 已修复：[source-cubemap-source-texel-test-api-drift](../../zircon_editor/editor/10/fixed-2026-07-12-source-cubemap-source-texel-test-api-drift.md)
- fixed 已修复：[lightmap-forward-bind-group-integration-compile](../../zircon_editor/editor/15/fixed-2026-07-13-lightmap-forward-bind-group-integration-compile.md)
- 2026-07-18 environment 性能交接：SceneRenderer 构造期约 1,678 万 BRDF LUT 积分、source-cubemap 稳定帧同步读盘/深拷贝、lightmap 每 draw 线性 slot 查询及 production serial PMREM 已回链 PERF-MVP-351..354。Render11 联动 Runtime04、Render03/17交付版本化预计算/设备共享 BRDF 工件、generation resident IBL artifact、共享 lightmap id→slot 索引及 asset compute pool/GPU bake；见 `docs/plans/performance/01/2026-07-18-runtime-core-framework-render-environment-static-review.md`。
- 2026-07-18 irradiance-volume选择交接：存在volume时，compiled-scene每camera/frame收集全部layer-visible mesh positions，再做volumes×positions containment并clone选中volume。Render11联动Render18/17按scene/lighting generation发布visible bounds/spatial candidates与priority index，多camera共享且返回borrowed/handle identity；见PERF-MVP-377。
- 2026-07-18 cubemap GPU上传交接：environment generation变化时renderer仍在submission线程把source/PMREM/irradiance全部f32 texel编码RGBA16F并按face×mip碎片`write_texture`；per-face/mip编码Vec已局部止损。Render11联动Runtime04、Render13/17发布预编码row-aligned upload artifact与持久staging batch，stable转换/上传=0、changed artifact build≤1/generation且upload batch≤1；见PERF-MVP-380。
- 2026-07-18 minimal renderer resource owner交接：`MeshPipelineCache::new`当前即使environment features关闭也创建irradiance-volume、reflection-probe与lightmap owners/fallbacks。Render11联动Render02按device共享neutral资源、按feature generation懒建真实resident owner；minimal F2 optional真实资源create=0、第二renderer neutral增量近0。见PERF-MVP-390。
- 2026-07-18 shadow neutral环境补充交接：`ShadowMapRenderer::new`为full scene layout另建3个1×1 cube、BRDF LUT、sampler与SH buffer。Render11提供per-device共享neutral environment handle或与Render02会签shadow最小scene layout，shadow renderer重复环境对象=0。见PERF-MVP-390。
- 2026-07-18 reflection/planar probe性能交接：`environment/probe_buffer/**`当前11/11个Rust文件、1,549行已静态读完；候选资产revision读取已从最多64次registry读锁降为一次短锁。Render11联动Render18按probe/camera/layer/asset generation发布prepared rows/slot与planar params，相同bytes不写；真实cubemap array/planar mip texture按feature single-flight创建，off只用device共享neutral并按需求增长capacity。见PERF-MVP-400及probe-buffer静态证据。
- 2026-07-18 realtime IBL性能交接：runtime/time-slice/graph/GPU/timestamp当前13/13个Rust文件、2,838行已静态读完；bake key重复派生、label中间Vec与112B capture heap分配已止损。Render11联动Render01预编译state variants，首次也按预算分帧；params/bindings/sampler/readback持久复用，无真实cloud输入不重复capture，feature-off资源归PERF-MVP-390。见PERF-MVP-401及realtime-IBL静态证据。
- 2026-07-18 environment全目录收口：当前46/46个Rust文件、10,894行已静态读完。余下IBL bake/lightmap/root 22/22中，source sampler已提升为每pipeline cache一个；Render11联动Render01让10-pass command artifact只build一次，并把params/binding/view持久化。runtime cache miss的MAP_READ/payload/文件写改有界异步worker，render线程不wait/I/O；见PERF-MVP-402及environment IBL bake静态证据。
- 2026-07-18 backend IBL readback补充交接：cube face×mip同步readback已由每region独立buffer/encoder/submit/wait止损为单staging batch，128/8等待48→1、完整PMREM+SH9+IEM约55→3。Render11仍须按PERF-MVP-402采用固定in-flight staging ring、主graph copy和非阻塞ticket，render线程`wait_indefinitely`=0；参考Bevy跨帧map/buffer pool与UE可复用staging+fence边界。
- 2026-07-18 frame IBL decision交接：context builder每camera调用runtime dispatch决定IBL bake option，stable source也可能重复cache store/stat/read路径并参与第二次compile key。Render11按source/environment/cache-artifact generation发布immutable bake decision，miss由PERF-MVP-402 worker推进，frame线程只读handle；stable fs/stat/read=0，多camera decision build≤1/generation。见PERF-MVP-414。
- 2026-07-18 planar camera plan交接：planar target查重已改一次camera target HashSet，但update锁仍跨probe扫描/derive且loop末才mark全部captured。Render11按probe dirty/target/main-camera generation增量派生plan entries，成功camera立即提交对应generation，后续camera失败不重复已完成capture；锁不跨camera derive/render。见PERF-MVP-417。
- 2026-07-22 IBL importer cache-hit补充：`stage_environment_ibl_source*`当前在检查`.zcube/.zribl` current前已完整decode RGBA32F，current检查又读取/解码完整source与derived artifact，source revision/hash还遍历bytes两次。Render11联动Runtime04/13按PERF-MVP-504与352/354先用header/content generation判定命中，hit decode/cubemap/irradiance/read payload=0，miss才在bounded asset compute job构建唯一artifact。
- 2026-07-18 offline reflection bake补充：当前manual helper已对zero budget/empty mesh早退并按eligible probe count精确预留Vec，默认预算4。Render11若扩大到真实cubemap或1k+ probes，必须按dirty probe generation增量产出artifact并支持job/timeslice/cancel，不能在UI/render锁内整批扫描/捕获；Render17记录visits/alloc/GPU work。
