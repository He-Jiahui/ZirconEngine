---
related_code:
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/light/snapshots.rs
  - zircon_runtime/src/core/framework/render/post_process/volume_component.rs
  - zircon_runtime/src/core/framework/render/post_process/volume_evaluator.rs
  - zircon_runtime/src/core/framework/render/post_process/effect.rs
  - zircon_runtime/src/graphics/feature/render_feature_descriptor/render_feature_descriptor.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_plugins/rendering/plugin.toml
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/VolumetricFog.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PostProcess/PostProcessSubsurface.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/LightCookieManager.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/ShaderLibrary/LightCookie/LightCookie.hlsl
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/Reflection/PlanarReflectionProbe.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/PlanarReflectionFiltering.compute
  - dev/bevy/crates/bevy_pbr/src/volumetric_fog/mod.rs
  - dev/bevy/crates/bevy_pbr/src/volumetric_fog/render.rs
  - dev/bevy/crates/bevy_light/src/volumetric.rs
  - dev/bevy/crates/bevy_light/src/point_light.rs
  - dev/bevy/crates/bevy_light/src/spot_light.rs
  - dev/bevy/crates/bevy_light/src/directional_light.rs
  - dev/bevy/crates/bevy_pbr/src/decal/clustered.rs
  - dev/bevy/crates/bevy_pbr/src/pbr_material.rs
  - dev/bevy/crates/bevy_pbr/src/transmission/mod.rs
  - dev/bevy/crates/bevy_pbr/src/transmission/node.rs
  - dev/bevy/crates/bevy_pbr/src/transmission/texture.rs
  - dev/bevy/crates/bevy_pbr/src/transmission/phase.rs
  - dev/bevy/crates/bevy_core_pipeline/src/oit/mod.rs
  - dev/bevy/crates/bevy_core_pipeline/src/oit/resolve/mod.rs
  - dev/bevy/crates/bevy_core_pipeline/src/oit/resolve/node.rs
  - dev/bevy/crates/bevy_light/src/probe.rs
  - dev/bevy/crates/bevy_pbr/src/light_probe/irradiance_volume.rs
  - dev/Fyrox/fyrox-impl/src/renderer/light_volume.rs
plan_sources:
  - .codex/plans/Rendering 插件选项补齐计划.md
---

# 计划 18:进阶光照与透明特性扩展

## 目标

在骨架层(01–08)与能力层(09–16)契约之上,以**相互独立、可单独启停的 feature** 形式补齐进阶光照与透明特性:每项经 RenderFeature descriptor 接入 graph,关闭时 compiled graph 不含对应 pass,默认全部关闭,不改变既有计划的任何契约定义。收录七项机制:

1. froxel 体积雾与体积光(light shafts)
2. light cookies(灯光投影遮罩)
3. 进阶材质特性族:clearcoat / anisotropy / transmission(含屏幕空间透射 pass)
4. OIT(顺序无关透明,可选替代透明排序)
5. irradiance volumes(局部体素探针体,逐像素漫反射 GI)
6. planar reflection(平面反射,镜像相机)
7. 屏幕空间次表面散射(Burley SSS)

## 现状与差距

- 体积雾/体积光:引擎无任何体积介质能力;计划 11 只有解析雾(距离/高度),计划 07 只有屏幕空间雾 post 槽位,均不与 light grid/shadow 交互。
- light cookies:`GpuLightData`(计划 05)无 cookie 槽位,灯光无投影遮罩能力。
- clearcoat / anisotropy / transmission:计划 08 的 `StandardPbr` 不含这三族参数,引擎无屏幕空间透射 pass,玻璃/车漆/拉丝金属不可表达。
- OIT:计划 09 定稿的是排序透明(sort_key 深度后向),无任何顺序无关路径,交叉半透明仍会出错。
- irradiance volumes:计划 11 EL-M3 只有**全局单个**均匀 SH 网格(`LightProbeGridData`,对象级按 instance 位置采样);无局部、可变换、逐像素采样的体素探针体。
- planar reflection:计划 11 反射探针是烘焙 cubemap,无镜面级实时平面反射。
- SSS:引擎与计划 08 均无 subsurface shading model 与散射 pass,皮肤/蜡/玉不可表达。

## 与既有计划的边界

本计划**只消费、不重定义**既有计划的契约;逐项衔接如下:

| 既有计划 | 本计划消费的契约 | 用途 |
|---|---|---|
| 01 | `RgTextureHandle` / `TransientResourcePool` | 全部中间纹理(froxel 3D 纹理、transmission scene copy、SSS 中间 RT、planar RT、cookie atlas)经资源池;froxel temporal history 为持久资源 |
| 05 | `GpuLightData` / `ShadowAtlas` / `zr_light_grid.wgsl` | cookie 在 `GpuLightData` 尾部加扩展位;体积光 scatter 只读消费 light grid 与 `ShadowAtlas`,不改 grid 结构 |
| 06 | jitter / history 槽位 | 体积雾 temporal 质量档复用计划 06 的 jitter 序列与 history 管理,不另建时域设施 |
| 07 | `VolumeComponentDescriptor` / `VolumeEvaluator` | `VolumetricFogSettings` 注册为 Volume 可覆写组件;07 post 链顺序不变(体积雾合成在 shading 端,SSS 在 lighting 段,均不插 post 链) |
| 08 | `ShadingModelDescriptor` / `ShaderVariantKey` | clearcoat/anisotropy/transmission 为 `StandardPbr` 扩展位 + 变体 feature flags;`Subsurface` 为新注册 shading model(G-buffer 通道超界走 08 的诊断机制) |
| 09 | `CameraRenderDescriptor` / `RenderLayerSet` / sort_key / RenderQueueValue | planar reflection 本质是一台镜像 RT 相机;transmission/OIT 沿用 09 的 transparent 段位段,**不另造位段**;各机制 layer 过滤共用 `RenderLayerSet` |
| 11 | `zr_environment.wgsl` 探针混合 / `LightProbeGridData` | planar 输出作为最高优先反射源进探针混合;irradiance volume 是 `LightProbeGridData` 的局部逐像素姊妹项,二者采样 ABI 并存 |
| 13 | `TextureMetadata` / 图集设施 / `Texture2DArrayAsset` | cookie atlas 复用 13 图集分配;irradiance volume 的 voxel 3D 纹理需 13 增加 Texture3D 资产维度(协调项,本计划不定义资产格式) |
| 16 | `ComputePassDescriptor` / `GpuReadbackQueue` | froxel 三段 compute 与 SSS compute 经 `ComputePassDescriptor` 声明;无 readback 需求,`GpuReadbackQueue` 仅 planar OnDemand 触发统计可选消费 |

接入纪律(index §6):全部七项经 RenderFeature descriptor 注册 pass(节点 + 资源 IO + executor id),feature 关闭时 compiled graph 无对应节点;材质特性族的 scene copy pass 仅在帧内存在 transmission 材质时进 graph。归属:契约进 `zircon_runtime::core::framework::render`(无 wgpu);执行在 `graphics/scene/scene_renderer/advanced_lighting/`;cookie/OIT/体积雾/planar/irradiance volume/SSS 六项为 `zircon_plugins/rendering` 插件 optional feature,材质特性族为 runtime 内建 08 扩展位(per-material 变体 gate,不占插件 feature)。

## 参考代码

| 文件 | 应重点阅读 |
|------|-----------|
| `dev/UnrealEngine/.../Renderer/Private/VolumetricFog.cpp` | froxel 三段结构:`FVolumetricFogMaterialSetupCS`(媒质注入,`MaterialSetupCS` entry)→ light scattering(`r.VolumetricFog.LightScatteringSampleJitterMultiplier` 时域抖动)→ 积分;`VolumetricFogGridInjectionGroupSize = 4`、`FVolumetricFogIntegrationParameterData` 的网格参数组织 |
| `dev/UnrealEngine/.../PostProcess/PostProcessSubsurface.cpp` | SSS 三段:setup(tile 分类)→ indirect dispatch Burley → recombine;`FSubsurfaceTiles::ETileType`(AFIS/Separable)、`ShadingModelMaskInView` 按 shading model 早退(:309)、`r.SSS.Burley.Quality` 降档 |
| `dev/Graphics/.../universal/Runtime/LightCookieManager.cs` | cookie atlas 管理:`Setup(cmd, lightData)`(:328)、`lightBufferIndex` 与灯光 buffer 的对应(:76)、主灯独立槽 + 附加灯进 atlas |
| `dev/Graphics/.../universal/ShaderLibrary/LightCookie/LightCookie.hlsl` | 三种投影 UV:`ComputeLightCookieUVDirectional`(平面投影 + uvWrap)/`UVSpot`(透视除法)/`UVPoint`(方向映射);`SampleMainLightCookie`/`SampleAdditionalLightCookie` 分流 |
| `dev/Graphics/.../high-definition/Runtime/Lighting/Reflection/PlanarReflectionProbe.cs` | `PlanarReflectionProbe : HDProbe`(:14):`localReferencePosition`/`referencePosition`(:21–23)镜像锚点语义、捕获设置随主相机派生 |
| `dev/Graphics/.../high-definition/Runtime/Lighting/PlanarReflectionFiltering.compute` | planar RT 的 roughness 模糊链(mip 逐级滤波),供粗糙表面采样 |

**Rust/wgpu 落地参照(防凭空实现)**:

| 文件 | 对应本计划机制 | 应重点阅读 |
|------|---------------|-----------|
| `dev/bevy/crates/bevy_light/src/volumetric.rs` | 体积雾组件契约 | `VolumetricFog`(:25,相机组件:ambient/jitter/step_count)、`FogVolume`(:78,局部雾体)、`VolumetricLight`(:16,灯光参与标记) |
| `dev/bevy/crates/bevy_pbr/src/volumetric_fog/render.rs` | 体积雾 GPU 面 | `VolumetricFogPipeline`(:98)/`VolumetricFogUniform`(:146)/`ViewVolumetricFog`(:180)/`extract_volumetric_fog`(:256)/`prepare_volumetric_fog_uniforms`(:685):雾体收集→per-view uniform→raymarch pipeline 的全链(配 `volumetric_fog.wgsl`);注意 bevy 是**逐雾体 raymarch** 形态,froxel 网格注入部分无 Rust 同类参照 |
| `dev/Fyrox/fyrox-impl/src/renderer/light_volume.rs` | 体积光(light shafts) | `LightVolumeRenderer`(:46):屏幕空间光柱的独立渲染器组织(GL 系 API,仅取结构) |
| `dev/bevy/crates/bevy_light/src/{point,spot,directional}_light.rs` | light cookies 契约面 | `PointLightTexture`(:162)/`SpotLightTexture`(:207)/`DirectionalLightTexture`(:176):cookie 即灯光纹理组件的字段形态 |
| `dev/bevy/crates/bevy_pbr/src/decal/clustered.rs` | cookie GPU 化 | `RenderClusteredDecals`(:68)/`DecalsBuffer`(:152)/`RenderClusteredDecal`(:209):灯光纹理与 clustered 数据共用一条上传链(:249–:388 的三类 LightTexture 查询) |
| `dev/bevy/crates/bevy_pbr/src/pbr_material.rs` | 材质特性族字段 | `clearcoat`(:498)/`clearcoat_perceptual_roughness`(:524)/`anisotropy_strength`(:583)/`specular_transmission`(:260)/`diffuse_transmission`(:218)/`thickness`(:290)/`ior`(:338)/`attenuation_distance`(:351):字段语义与默认值的权威清单 |
| `dev/bevy/crates/bevy_pbr/src/transmission/{mod,node,texture,phase}.rs` | 屏幕空间透射 pass | `ScreenSpaceTransmission`(mod.rs :67,steps 语义)/`main_transmissive_pass_3d`(node.rs :19,逐 step scene copy + 重绘)/`ViewTransmissionTexture`(texture.rs :24)/`Transmissive3d` phase(phase.rs :24) |
| `dev/bevy/crates/bevy_core_pipeline/src/oit/mod.rs` | OIT 存储 pass | `OrderIndependentTransparencySettings`(:42,`sorted_fragment_max_count=8`/`fragments_per_pixel_average=4.0`/`alpha_threshold`)、`OitBuffers`(:152)、`prepare_oit_buffers`(:222,按视口尺寸分配) |
| `dev/bevy/crates/bevy_core_pipeline/src/oit/resolve/{mod,node}.rs` | OIT resolve | `OIT_REQUIRED_STORAGE_BUFFERS = 3`(:35)/`is_oit_supported`(:69,能力检测样板)/`OitResolvePipeline`(:103)/`oit_resolve`(node.rs :15)(配 `oit_resolve.wgsl`、`oit_draw.wgsl`) |
| `dev/bevy/crates/bevy_light/src/probe.rs` | irradiance volume 契约 | `IrradianceVolume`(:329):`voxels: Handle<Image>`(ambient cube 编码 3D 纹理)+ `intensity` + `affects_lightmapped_meshes` 三字段即完整契约 |
| `dev/bevy/crates/bevy_pbr/src/light_probe/irradiance_volume.rs` | irradiance volume 绑定 | `IRRADIANCE_VOLUMES_ARE_USABLE`(:161,平台 gate 样板)、`RenderViewIrradianceVolumeBindGroupEntries`(:165)的 single/multiple 双绑定形态(配 `irradiance_volume.wgsl` 采样) |

无 Rust 同类参照、按 index §8 第 8 条执行(对拍测试先行、逐切片抓帧)的机制:**froxel 网格注入与积分**(bevy 仅 raymarch 形态,UE `VolumetricFog.cpp` 为唯一样板)、**planar reflection**(HDRP 为唯一样板)、**Burley SSS**(UE `PostProcessSubsurface.cpp` 为唯一样板)。

## 目标架构

归属:契约进 `core/framework/render/advanced_lighting/`(纯数据可序列化,无 wgpu;与既有 `advanced` 模块的 feature 计划设施不混用);执行在 `graphics/scene/scene_renderer/advanced_lighting/`;六项插件 feature 落 `zircon_plugins/rendering`(`rendering.volumetric_fog` / `rendering.light_cookies` / `rendering.oit` / `rendering.irradiance_volumes` / `rendering.planar_reflections` / `rendering.subsurface_scattering`)。

### 18.1 froxel 体积雾与体积光

- 数据契约:`VolumetricFogSettings`(密度、albedo、相位各向异性 g、高度衰减、散射强度、froxel 深度分布指数、temporal 开关)注册为计划 07 `VolumeComponentDescriptor` 组件,经 `VolumeEvaluator` 求值后进 extract;`FogVolumeData`(局部雾体:bounds、density、albedo,对齐 bevy `FogVolume`)与灯光参与标记(`VolumetricLight` 同义的 per-light bool,进 `GpuLightData` flags 位)。
- pass 接入:三个 compute 节点(经 `ComputePassDescriptor`):`volumetric.media_inject`(全局+局部雾体写 froxel 3D 纹理:rgb=scattering、a=extinction)→ `volumetric.light_scatter`(逐 froxel 采 light grid + `ShadowAtlas`,HG 相位,temporal 档混合 history)→ `volumetric.integrate`(沿 z 前向积分);forward/deferred shading 端与 skybox 经 `zr_volumetric.wgsl` 按像素深度采积分纹理合成。
- 能力 gate:compute + 3D storage 纹理写(wgpu 基线能力,无需扩展);feature 关闭时三节点不进 graph,`zr_volumetric` 变体 flag 关闭。
- 质量档:froxel 160×90×{48|64|96},Low 关 temporal/关局部雾体,High 开 temporal(消费 06 jitter/history)。

### 18.2 light cookies

- 数据契约:`LightCookieData`(cookie 纹理 AssetId、wrap 模式、directional 的 2D offset/scale)进 `light/` 契约扩展;`GpuLightData` 尾部追加 cookie 扩展位(atlas uv rect + flags,见工程细化)。
- pass 接入:`cookie.atlas_build`(prepare 期把新增/变更 cookie blit 进 2D atlas;point 灯导入期预转 octahedral 2D 投影,对齐 URP `ComputeLightCookieUVPoint` 的方向映射思路)。采样在 forward/deferred 光照循环内(`zr_light_cookie.wgsl`),无独立着色 pass。
- 能力 gate:无特殊;feature 关闭时 atlas 节点不进 graph,`LIGHT_COOKIES` 变体 flag 关闭,`GpuLightData` 扩展位写无效值。
- 质量档:atlas 1024/2048/4096(复用计划 13 图集分配)。

### 18.3 材质特性族(clearcoat / anisotropy / transmission)

- 数据契约:`StandardPbr` 材质字段扩展(命名对齐 bevy `StandardMaterial`):`clearcoat`/`clearcoat_perceptual_roughness`/`clearcoat_normal_texture`、`anisotropy_strength`/`anisotropy_rotation`、`specular_transmission`/`diffuse_transmission`/`thickness`/`ior`/`attenuation_color`/`attenuation_distance`;`ShaderVariantKey` 增加 `PBR_CLEARCOAT`/`PBR_ANISOTROPY`/`PBR_TRANSMISSION` feature flags(字段为默认值时变体不含对应代码)。
- pass 接入:clearcoat/anisotropy 纯 shading 位无 pass;specular transmission 走 `transmission.scene_copy` 节点(opaque+skybox 之后把 `SCENE_COLOR` 拷为可采样纹理,bevy `ViewTransmissionTexture` 同型)+ transmissive 队列(RenderQueueValue 固定 2900,经 09 材质 queue 覆写机制落 Transparent 段内、先于普通透明;sort_key 沿用 09 位段)。多层折射 steps>1 为质量档(逐 step 重拷,bevy `ScreenSpaceTransmission.steps` 语义)。
- deferred 边界:三族材质强制 forward 路径(08 描述符声明 forward-only,G-buffer 不扩通道),对齐 bevy 形态。
- 能力 gate:无;帧内无 transmission 材质时 scene_copy 节点不进 graph。

### 18.4 OIT

- 数据契约:`OitSettings`(per camera,挂 `CameraRenderDescriptor` 可选扩展):`fragments_per_pixel_average`、`sorted_fragment_max_count`、`alpha_threshold`(bevy 同名字段语义)。
- pass 接入:`oit.fragment_store`(transparent 物体改用 OIT 变体:fragment 写层缓冲 + 计数缓冲,不写 color,`zr_oit.wgsl` 提供 `oit_draw` 同型函数)→ `oit.resolve`(全屏按深度排序混合写 `SCENE_COLOR`)。开启相机替换 09 排序透明路径(该相机的 transparent phase 改投递 OIT 变体);关闭时 graph 无 OIT 节点,默认仍是 09 排序——OIT 是可选 feature,不是默认路径。
- 能力 gate:fragment 阶段 storage buffer ≥ 3 且 buffer 容量充足(bevy `is_oit_supported`/`OIT_REQUIRED_STORAGE_BUFFERS` 同款检测);不满足时 feature 静默降级回排序透明并出诊断。
- 质量档:fragments_per_pixel_average 2/4/8(内存 = 视口像素 × avg × 8B)。

### 18.5 irradiance volumes

- 数据契约:`IrradianceVolumeData`(transform、voxel 3D 纹理 AssetId(ambient cube 编码,bevy 格式)、`intensity`、`affects_lightmapped_meshes`、`layer_mask: RenderLayerSet`)——字段对齐 bevy `IrradianceVolume`;可同场景多个,extract 端按相机 layer/距离裁剪,V1 着色每像素取包含点的最高优先 1 个(bevy single 绑定形态),binding array 多体留远期。
- pass 接入:无独立 pass;prepare 期 per-view 绑定选中 volume 的 3D 纹理,着色端 `zr_irradiance_volume.wgsl`(世界→volume 本地 UVW,ambient cube 六方向加权)替换/叠加 ambient 漫反射项,与计划 11 的全局 `LightProbeGridData`(对象级)互斥优先:像素落在 volume 内取 volume,否则回落全局网格/ambient。
- 烘焙:体素烘焙器归 rendering 插件(`baked_lighting` 延伸),runtime 只消费 3D 纹理资产(13 的 Texture3D 维度协调项)。
- 能力 gate:3D 纹理采样(基线能力);feature 关闭时变体 flag 关闭、不绑定。

### 18.6 planar reflection

- 数据契约:`PlanarReflectionProbeData`(平面 transform、影响域 bounds、分辨率档、`layer_mask`、更新策略 `EveryFrame | OnDemand`、锚点 `local_reference_position` 对齐 HDRP 语义)。
- pass 接入:激活 probe 在相机循环里派生一台镜像 RT 相机(`CameraRenderDescriptor`:view 沿平面反射、oblique near-clip 斜裁剪、target=Texture、culling_mask 取 probe layer_mask)——复用 09 RT 相机机制,**不在 graph 内另造场景渲染旁路**;随后 `planar.filter` 节点做 roughness mip 模糊链(HDRP `PlanarReflectionFiltering.compute` 同型,复用 11 预滤波 reduce 框架)。输出进 `zr_environment.wgsl` 探针混合作为最高优先反射源(权重在 cubemap 探针之上)。
- 能力 gate:成本为一次额外场景渲染;默认 OnDemand;质量档分辨率 256/512/1024。
- 无 Rust 同类参照(标注):实现时 CPU 侧镜像矩阵/斜裁剪闭式先行对拍。

### 18.7 屏幕空间次表面散射(Burley SSS)

- 数据契约:`SubsurfaceProfileData`(散射半径 RGB、falloff、world unit scale;profile 资产表 ≤16 项进 uniform);`Subsurface` shading model 经 08 `ShadingModelDescriptor` 注册(G-buffer 写 shading model id + profile index,通道超界走 08 诊断而非静默)。
- pass 接入:deferred lighting 合成之后、transparent 之前三个节点:`sss.setup`(按 shading model id 标 tile,UE `FSubsurfaceTiles` 同型)→ `sss.scatter`(Burley 径向采样 compute,indirect dispatch 只跑标记 tile)→ `sss.recombine`(散射结果与 specular 重组回 `SCENE_COLOR`)。
- 边界:V1 deferred-only(forward 路径材质声明 Subsurface 时回落 StandardPbr 并诊断);不进 07 post 链(发生在 lighting 段)。
- 能力 gate:compute + deferred 开启;feature 关闭时三节点不进 graph、shading model 注册不生效。
- 无 Rust 同类参照(标注):Burley 核 CPU 参考实现先行,逐切片抓帧。

## 里程碑

按依赖排序;milestone-first:切片期只 `cargo check -p zircon_runtime --lib --locked`,里程碑末进测试阶段。

### AF-M1 材质特性族(依赖 08 落地)

实施切片:
1. 材质字段 + `PBR_CLEARCOAT`/`PBR_ANISOTROPY` 变体位与 BRDF 项(`zr_pbr_extras.wgsl`);forward-only 声明。
2. transmission 字段 + `transmission.scene_copy` 节点 + transmissive 队列(queue 2900)接 09 排序。

测试阶段:
- `cargo test -p zircon_runtime advanced_lighting --locked` + `render_product` 对拍
- 验收证据:车漆球(clearcoat)/拉丝金属(anisotropy)抓帧对照 bevy 同参数;玻璃球折射背景正确,无 transmission 材质帧 graph dump 无 scene_copy 节点。

### AF-M2 灯光数据扩展:light cookies + irradiance volumes(依赖 05/11/13)

实施切片:
1. `LightCookieData` 契约 + `GpuLightData` 扩展位 + cookie atlas + `zr_light_cookie.wgsl` 三投影采样。
2. `IrradianceVolumeData` 契约 + per-view 选体绑定 + `zr_irradiance_volume.wgsl`;与 11 全局网格的互斥优先接线。

测试阶段:
- `cargo test -p zircon_runtime advanced_lighting --locked` + 插件 workspace 测试
- 验收证据:spot/point/directional 三类 cookie 投影抓帧(窗格光、手电筒遮罩);动态物体进出 volume 时漫反射 GI 平滑变化;两 feature 分别关闭后 graph 无对应节点、产物回归基线。

### AF-M3 体积介质:froxel 体积雾与体积光(依赖 05/06/07/16)

实施切片:
1. `VolumetricFogSettings`(Volume 组件注册)+ `FogVolumeData` extract + froxel 资源与 `volumetric.media_inject`。
2. `volumetric.light_scatter`(light grid + ShadowAtlas 消费、HG 相位)+ `volumetric.integrate` + shading 端合成。
3. temporal 档(06 jitter/history)与质量档表。

测试阶段:
- `cargo test -p zircon_runtime advanced_lighting --locked`(froxel 索引/相位/积分闭式 CPU 对拍)
- 验收证据:体积光柱穿过窗口抓帧(对照 UE 同场景行为);关 feature 后 graph 无三节点;temporal 开关前后噪点对比;`render_perf_*` 计数:froxel 上传字节与 dispatch 数稳定。

### AF-M4 重型可选项:OIT + planar reflection + SSS(依赖 09 RT 相机/排序、08 G-buffer 位)

实施切片:
1. OIT:`OitSettings` + 双 buffer + `oit.fragment_store`/`oit.resolve` + 能力检测降级。
2. planar:`PlanarReflectionProbeData` + 镜像相机派生(09 相机循环)+ `planar.filter` + 探针混合接线。
3. SSS:`SubsurfaceProfileData` + `Subsurface` shading model 注册 + 三节点链。

测试阶段:
- `cargo test -p zircon_runtime advanced_lighting --locked`、`cargo test --manifest-path zircon_plugins/Cargo.toml -p <rendering 受影响 crate> --locked`
- 验收证据:三层交叉透明片 OIT 与排序路径对比图(交叉处无翻转);镜面地板反射与镜像基线对拍(斜裁剪无穿帮);皮肤球 SSS 开关对照 + Burley 核 CPU 对拍;三 feature 关闭后 compiled graph 与基线 dump 完全一致。

## 工程落地细化

本章节为计划 18 的实施权威(index §8.7)。bind group 槽位、std430 基线、`zr_` include 约定、RenderQueueValue 数值段、sort_key 位段(归 09)、测试命名直接引用 index §8,不再重述。契约层无 wgpu;只消费 `RenderFrameExtract`;全部 pass 经 graph 节点 + executor id;一律硬切换。

### 模块与文件落点

| 路径 | 内容 |
|------|------|
| `zircon_runtime/src/core/framework/render/advanced_lighting/mod.rs` | 模块声明与 curated re-export(薄) |
| `zircon_runtime/src/core/framework/render/advanced_lighting/volumetric.rs` | `VolumetricFogSettings` / `FogVolumeData` / `FroxelGridParams` |
| `zircon_runtime/src/core/framework/render/advanced_lighting/cookie.rs` | `LightCookieData` / `CookieWrapMode` / `CookieProjection` |
| `zircon_runtime/src/core/framework/render/advanced_lighting/material_features.rs` | 材质特性族字段 DTO 与默认值表(08 材质资产消费) |
| `zircon_runtime/src/core/framework/render/advanced_lighting/oit.rs` | `OitSettings`(camera 扩展) |
| `zircon_runtime/src/core/framework/render/advanced_lighting/irradiance_volume.rs` | `IrradianceVolumeData` |
| `zircon_runtime/src/core/framework/render/advanced_lighting/planar.rs` | `PlanarReflectionProbeData` / `PlanarUpdateMode` |
| `zircon_runtime/src/core/framework/render/advanced_lighting/subsurface.rs` | `SubsurfaceProfileData` / profile 表上限常量 |
| `zircon_runtime/src/core/framework/render/advanced_lighting/extract.rs` | `AdvancedLightingExtract`(进 `RenderFrameExtract` 的可选段,全 None 时零成本) |
| `zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/mod.rs` | 实现侧模块声明(薄) |
| `zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel.rs` | froxel 资源(双 3D 纹理 + history)与三 compute 节点构建 |
| `zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/cookie_atlas.rs` | `CookieAtlas`(13 图集分配消费、blit 节点、主灯独立槽) |
| `zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/transmission.rs` | scene copy 节点 + transmissive 队列投递 |
| `zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/oit_buffers.rs` | OIT 层/计数 buffer 分配、能力检测、resolve pipeline |
| `zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/irradiance_binding.rs` | per-view volume 选择与 3D 纹理绑定 |
| `zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/planar_mirror.rs` | 镜像 `CameraRenderDescriptor` 派生(反射矩阵 + 斜裁剪)与 filter 节点 |
| `zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/subsurface_pass.rs` | SSS tile 分类/scatter/recombine 三节点 |
| `zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/shaders/*.wgsl` | `froxel_inject` / `froxel_scatter` / `froxel_integrate` / `oit_resolve` / `sss_setup` / `sss_scatter` / `sss_recombine` / `planar_filter` |
| `zircon_runtime/src/graphics/shader/includes/zr_volumetric.wgsl` | 积分纹理采样合成(forward/deferred/skybox 共用) |
| `zircon_runtime/src/graphics/shader/includes/zr_light_cookie.wgsl` | 三投影 UV + atlas 采样(URP `LightCookie.hlsl` 同型) |
| `zircon_runtime/src/graphics/shader/includes/zr_pbr_extras.wgsl` | clearcoat 第二 specular 叶 / anisotropic GGX / transmission BTDF |
| `zircon_runtime/src/graphics/shader/includes/zr_oit.wgsl` | `oit_draw`(层写入,bevy `oit_draw.wgsl` 同型) |
| `zircon_runtime/src/graphics/shader/includes/zr_irradiance_volume.wgsl` | UVW 变换 + ambient cube 六方向加权 |
| `zircon_runtime/src/graphics/tests/render_product_advanced_lighting.rs` | 七机制产物对拍 |

修改文件(契约衔接,均为追加扩展位,不动既有字段):`frame_extract.rs`(可选 `advanced_lighting` 段)、`light/snapshots.rs`(cookie 引用 + volumetric 参与位)、`camera.rs`(OIT 扩展挂点,归 09 契约文件)、05 的 `GpuLightData` WGSL/Rust 镜像(尾部追加)、08 的 `ShaderVariantKey` flags 位与 `ShadingModelDescriptor` 注册表、11 的 `zr_environment.wgsl`(planar 槽)、`render_pass_executor_registry.rs`(新 executor 注册)、`zircon_plugins/rendering/plugin.toml`(六个 optional feature 声明)。

### 核心类型与接口

```rust
// volumetric.rs
pub struct VolumetricFogSettings {
    pub density: Real,                 // 全局基础密度(m^-1)
    pub albedo: Vec3,
    pub phase_g: Real,                 // HG 各向异性 [-0.9, 0.9]
    pub height_falloff: Real,
    pub scattering_intensity: Real,
    pub depth_distribution_exp: Real,  // froxel z 切片指数(默认 2.0)
    pub temporal: bool,
}
pub struct FogVolumeData {
    pub volume_id: u64,
    pub bounds_min: Vec3, pub bounds_max: Vec3,
    pub density: Real, pub albedo: Vec3,
    pub layer_mask: RenderLayerSet,
}

// cookie.rs
pub enum CookieProjection { Directional { offset: Vec2, scale: Vec2, wrap: CookieWrapMode }, Spot, PointOctahedral }
pub struct LightCookieData { pub light_id: u64, pub texture: AssetId, pub projection: CookieProjection }

// oit.rs(字段语义对齐 bevy OrderIndependentTransparencySettings)
pub struct OitSettings {
    pub fragments_per_pixel_average: Real,  // 默认 4.0
    pub sorted_fragment_max_count: u32,     // 默认 8
    pub alpha_threshold: Real,              // 默认 0.0
}

// irradiance_volume.rs(字段对齐 bevy IrradianceVolume)
pub struct IrradianceVolumeData {
    pub volume_id: u64,
    pub transform: Mat4,                // 世界→volume 本地(含非均匀缩放)
    pub voxels: AssetId,                // ambient cube 编码 3D 纹理(13 Texture3D)
    pub intensity: Real,
    pub affects_lightmapped_meshes: bool,
    pub priority: i32,
    pub layer_mask: RenderLayerSet,
}

// planar.rs
pub enum PlanarUpdateMode { EveryFrame, OnDemand }
pub struct PlanarReflectionProbeData {
    pub probe_id: u64,
    pub plane_transform: Mat4,
    pub local_reference_position: Vec3,   // HDRP 同义镜像锚点
    pub bounds_min: Vec3, pub bounds_max: Vec3,
    pub resolution: u32,                  // 256/512/1024
    pub update: PlanarUpdateMode,
    pub layer_mask: RenderLayerSet,
}

// subsurface.rs
pub struct SubsurfaceProfileData {
    pub profile_id: u32,                 // 进 G-buffer 的 index(≤ ZR_SSS_MAX_PROFILES=16)
    pub scatter_radius_rgb: Vec3,        // mm
    pub world_unit_scale: Real,
}

// extract.rs —— 进 RenderFrameExtract,全部可选;extract 仅由 runtime 生成
pub struct AdvancedLightingExtract {
    pub volumetric: Option<VolumetricFogSettings>,    // 已经 VolumeEvaluator 解析的终值(07)
    pub fog_volumes: Vec<FogVolumeData>,
    pub cookies: Vec<LightCookieData>,
    pub irradiance_volumes: Vec<IrradianceVolumeData>,
    pub planar_probes: Vec<PlanarReflectionProbeData>,
    pub subsurface_profiles: Vec<SubsurfaceProfileData>,
}
```

### GPU 数据布局与 WGSL 约定

binding:本计划占 `group1` 的 `@binding(24..29)` 区段(05 light grid/shadow 低位段、11 环境段 16–23 之后,不冲突):24 cookie atlas `texture_2d`、25 cookie sampler、26 froxel 积分 `texture_3d<f32>`、27 froxel sampler、28 irradiance volume `texture_3d<f32>`、29 planar 反射 `texture_2d`。OIT 与 SSS 的 storage 资源为 pass 私有 bind group(经 `ComputePassDescriptor`/pass 声明,不占 group1 公共段)。

`GpuLightData` 尾部追加 32 bytes(std430,offset 接在 05 定稿布局之后,05 侧同步镜像注释):

```wgsl
// 追加段(05 GpuLightData 之后):
    cookie_uv_rect: vec4<f32>,  // +0:  atlas uv rect(scale.xy + offset.xy);无 cookie 时全 0
    cookie_misc: vec4<u32>,     // +16: x=projection(0=无,1=dir,2=spot,3=point_oct),
                                //      y=wrap(0=clamp,1=repeat), z=volumetric 参与位, w=pad
```

froxel:`media` 与 `integrated` 两张 RGBA16F 3D 纹理(rgb=in-scatter,a=extinction/透过率),history 一张(temporal 档,持久资源经 01);z 切片深度 `z = near * pow(far/near, (slice+0.5)/dims.z)^depth_distribution_exp` 闭式进 CPU 单测。OIT:层缓冲 `array<vec2<u32>>`(x=rgba16 打包色,y=depth bits)+ 计数缓冲,容量 = 视口像素 × `fragments_per_pixel_average`,resolve 端插入排序上限 `sorted_fragment_max_count`(bevy 双 buffer 同型)。SSS profile 表:16 × 32B uniform(scatter_radius_rgb + world_unit_scale + pad)。

WGSL include 只暴露函数与 struct(index §8.3):`zr_volumetric_apply(color, uv, view_depth) -> vec3f`、`zr_sample_light_cookie(light, world_pos) -> vec3f`、`zr_clearcoat_lobe(...)`/`zr_aniso_ggx(...)`/`zr_transmission_btdf(...)`、`oit_draw(frag_pos, color)`、`zr_irradiance_volume_sample(world_pos, normal) -> vec3f`。

### 帧时序与集成点

1. **prepare 期**:cookie atlas 增量 blit(`cookie.atlas_build`);OIT buffer 按视口/设置重分配;froxel history 槽位轮换;planar OnDemand 判定。
2. **planar 镜像相机**:在 09 相机循环中先于主相机执行(render_order 派生为主相机之前),输出 RT → `planar.filter` mip 模糊链。
3. **场景渲染段**:opaque → skybox → `transmission.scene_copy` → transmissive 队列(2900)→(OIT 开启相机:`oit.fragment_store`,否则 09 排序透明)→ `oit.resolve`。
4. **lighting 段(deferred)**:lighting 合成(消费 cookie/irradiance volume/volumetric include)→ `sss.setup` → `sss.scatter` → `sss.recombine` → 进入 3 的透明段。
5. **froxel compute**:`volumetric.media_inject` → `volumetric.light_scatter` → `volumetric.integrate` 排在 shadow pass 之后、场景着色之前(scatter 读 `ShadowAtlas`);合成发生在 forward shading / deferred lighting / skybox 的输出端(`zr_volumetric_apply`),不加 post 节点,07 链不感知。
6. **feature 关断面**:六个插件 feature 各自对应 descriptor;关闭时对应节点不进 compiled graph、变体 flag 不进 `ShaderVariantKey`、`AdvancedLightingExtract` 对应段为空集——graph dump 与关闭前基线逐字节一致(验收项)。

### 实施切片细化

- **AF-M1/切片 1**:`material_features.rs` + 08 变体位 + `zr_pbr_extras.wgsl`;完成判据:check 过,clearcoat/aniso 字段为默认值的材质变体哈希与改前一致(零成本断言)。
- **AF-M1/切片 2**:`transmission.rs` + scene copy 节点 + 队列 2900 投递;完成判据:check 过,无 transmission 材质帧 graph 无新节点。
- **AF-M2/切片 1**:`cookie.rs` + `cookie_atlas.rs` + `GpuLightData` 追加段 + `zr_light_cookie.wgsl`;完成判据:三投影 UV 的 CPU/WGSL 闭式对拍函数就位。
- **AF-M2/切片 2**:`irradiance_volume.rs` + `irradiance_binding.rs` + `zr_irradiance_volume.wgsl` + 与 11 互斥优先;完成判据:check 过,volume 选择函数 CPU 单测绿。
- **AF-M3/切片 1–3**:froxel 资源 → inject → scatter(light grid/ShadowAtlas 只读)→ integrate → 合成 → temporal;每切片 check,z 分布/HG 相位/前向积分三个闭式先行进单测。
- **AF-M4/切片 1**:OIT(buffer + 双节点 + `is_oit_supported` 同款能力检测与降级诊断)。
- **AF-M4/切片 2**:planar(反射矩阵 `R = I - 2nn^T` 与 oblique near-clip 闭式 CPU 先行)+ filter + 探针混合接线。
- **AF-M4/切片 3**:SSS(`Subsurface` shading model 注册 + 三节点;Burley 核 `R(r)` CPU 参考实现先行对拍)。

### 测试与验收清单

单元测试(就近 `#[cfg(test)]`,过滤词 `advanced_lighting`):

| 测试函数 | 断言 |
|---|---|
| `render_volumetric_froxel_slice_depth_matches_closed_form` | z 切片闭式逐档对拍(指数分布、边界) |
| `render_volumetric_hg_phase_normalizes` | HG 相位球面积分 ≈ 1(g 多档) |
| `render_volumetric_integration_constant_medium_matches_beer_lambert` | 常密度介质积分与 Beer-Lambert 闭式对拍 |
| `render_cookie_uv_three_projections_match_reference` | dir/spot/point_oct 三投影 UV 与 CPU 参考逐点对拍 |
| `render_cookie_gpu_light_data_extension_offsets` | 追加段 32B、offset 静态断言、无 cookie 全 0 |
| `render_transmission_queue_value_is_2900_in_transparent_band` | queue 落 Transparent 段、先于 3000 默认 |
| `render_oit_capability_gate_falls_back_to_sorted` | storage buffer 不足时降级 + 诊断记录 |
| `render_oit_resolve_sorts_within_max_count` | 乱序片段 resolve 后深度有序、超限合并 |
| `render_irrvol_world_to_uvw_roundtrip` | transform 含非均匀缩放时 UVW 变换可逆 |
| `render_irrvol_selection_prefers_priority_inside` | 嵌套 volume 按 priority 选择、外部回落全局网格 |
| `render_planar_mirror_matrix_reflects_view` | `R` 矩阵自反(`R*R=I`)、镜像相机看镜像点 == 主相机看原点 |
| `render_planar_oblique_clip_contains_plane` | 斜裁剪后近平面与反射面共面(容差) |
| `render_sss_burley_kernel_integrates_to_one` | Burley `R(r)` 数值积分归一(三半径档) |
| `render_sss_profile_table_caps_at_16` | 超限 profile 报诊断不越界 |
| `render_advanced_extract_empty_keeps_graph_baseline` | `AdvancedLightingExtract` 全空时 compiled graph == 基线 |

产物对拍(`render_product_advanced_lighting.rs`,配 `ZR_RENDERDOC_CAPTURE_NEXT=1`):光柱窗口场景(对照 UE)、三 cookie 投影、clearcoat/aniso/玻璃三球、三层交叉透明 OIT vs 排序、volume 内外漫反射、镜面地板、皮肤球 SSS 开关;每项含 feature-off 回归基线断言。里程碑命令:切片期 `cargo check -p zircon_runtime --lib --locked`;测试阶段 `cargo test -p zircon_runtime advanced_lighting --locked`、`cargo test -p zircon_runtime render_product_advanced_lighting --locked`;插件接缝 `cargo test --manifest-path zircon_plugins/Cargo.toml -p <rendering 受影响 crate> --locked`。

## 状态与产出记录

| 日期 | 里程碑/切片 | 状态 | 产出 | 验证与证据 | 后续 |
|------|-------------|------|------|------------|------|
| 2026-06-23 | Render index 当前状态总览拆分 | 全部未启动 | 从 docs/plans/zircon_runtime/render/index.md 的第 9 节迁入本计划；本行保留 18 Advanced Lighting 的当前事实，render 总索引不再维护计划级明细。 | 文档重组；本次未改生产代码，render/index.md 只保留状态路由说明。 | 仍未完成：clearcoat/anisotropy/transmission/SSS、cookies/irradiance volumes、froxel fog/OIT/planar reflection；验收缺口：需要等 05/07/08/09/11/13 地基完成后逐 feature 做 parity 和 RenderDoc 验收 |
| 2026-06-15 | AF-M1 material feature family | 未启动: 依赖计划 08 shading model registry | clearcoat、anisotropy、transmission 和 SSS 所需材质参数、G-buffer 位与 shading model 仍未落地。 | 本文件 `现状与差距` 明确 StandardPBR 不含这些参数,计划 08 MS-M3 状态表也标为未启动。 | 先完成计划 08 的 shading model registry、G-buffer packing 与材质资产验证。 |
| 2026-06-15 | AF-M2 light cookies and irradiance volumes | 未启动: 依赖 05/11/13 | `GpuLightData` 尚无 cookie 槽位,计划 11 的 probe/lightmap 仍未完整,texture array/cubemap 资产也未完成;irradiance volumes 仍是后续设计。 | 本文件 `现状与差距` 记录 light cookies 和 irradiance volumes 缺口;计划 05/11/13 状态表分别记录相关基础仍有后续项。 | 扩展 light data ABI、cookie texture binding、local irradiance volume asset 和 sampling path。 |
| 2026-06-15 | AF-M3 froxel volumetric fog and lighting | 未启动: 依赖 light grid/history/post/compute 地基 | 体积介质、froxel volume、volumetric shadowing 和 temporal reprojection 均未实现。 | 本文件 `现状与差距` 明确计划 11 只有解析雾、计划 07 只有 screen-space fog 槽位,不与 light grid/shadow 交互。 | 等计划 05/06/07/16 稳定后实施 froxel grid、lighting injection、history filter 和 composite pass。 |
| 2026-06-15 | AF-M4 OIT, planar reflection and SSS | 未启动: 重型可选项待核心排序/G-buffer 完成 | OIT、planar reflection、Burley SSS 都无实现;当前透明仍依赖排序,反射探针为 cubemap 思路,无实时平面反射。 | 本文件 `现状与差距` 分别记录 OIT、planar reflection、SSS 空缺;计划 09 CO-M3 和计划 08 MS-M3 尚未完成。 | 等 camera/render queue、advanced material G-buffer 和 texture/cubemap 地基完成后分切片实施。 |

### 参考实现精读笔记

(只记录本计划撰写时实际读到的符号;实现前仍须按 index §8.8 通读对应文件。)

**bevy `bevy_light/src/volumetric.rs` + `bevy_pbr/src/volumetric_fog/render.rs`**:契约面是三个组件——相机挂 `VolumetricFog`(:25)、雾体实体挂 `FogVolume`(:78)、灯光挂 `VolumetricLight`(:16)空标记;GPU 面 `extract_volumetric_fog`(:256)收集雾体、`prepare_volumetric_fog_uniforms`(:685)按 view×volume 笛卡尔积填 `VolumetricFogUniform`(:146),`prepare_view_depth_textures_for_volumetric_fog`(:771)说明其依赖深度可采样。Zircon 取舍:组件三分法(settings/雾体/灯光参与位)原样采用;但 bevy 是逐雾体全屏 raymarch,Zircon 走 UE froxel(光照在 3D 网格内一次求值,与 light grid 天然契合),raymarch 仅作 Low 档备选不实现。

**UE `VolumetricFog.cpp`**:`FVolumetricFogMaterialSetupCS`(:249,entry `MaterialSetupCS`)把全局+局部介质写入 VBuffer(`VolumetricFogGridInjectionGroupSize = 4` 的 4×4×4 dispatch,:241);scatter 阶段以 `r.VolumetricFog.LightScatteringSampleJitterMultiplier`(:154)做时域抖动;`FVolumetricFogIntegrationParameterData`(:217)打包网格维度/切片分布进各 shader。Zircon 对应:三段节点同构(inject/scatter/integrate),workgroup 同取 4×4×4;jitter 接 06 的序列而非自带 cvar。

**Fyrox `light_volume.rs`**:`LightVolumeRenderer`(:46)是独立小渲染器(per-light 屏幕空间光柱),证明"体积光可以独立于雾框架做轻量版"——Zircon 不采纳该形态(光柱由 froxel scatter 自然产生),仅在风险节备案为降级路径。

**bevy `oit/mod.rs` + `oit/resolve/`**:`OrderIndependentTransparencySettings`(:42)三参数(默认 8/4.0/0.0)即全部用户面;`OitBuffers`(:152)+`prepare_oit_buffers`(:222)按视口重分配;`OIT_REQUIRED_STORAGE_BUFFERS = 3` 与 `is_oit_supported`(resolve/mod.rs :69)是能力检测样板(检查 fragment 阶段 storage buffer 数);resolve 入口 `oit_resolve`(node.rs :15)。Zircon 取舍:参数名与默认值原样采用;bevy 把 OIT 做成相机组件全局替换透明路径,Zircon 同样按相机粒度,但要求 feature 关闭时连 buffer 都不分配。

**bevy `pbr_material.rs` + `transmission/`**:特性族字段(clearcoat :498、anisotropy_strength :583、specular_transmission :260、thickness :290、ior :338、attenuation :351–363)与 KHR 扩展一一对应,导入器可直通;`main_transmissive_pass_3d`(node.rs :19)按 `ScreenSpaceTransmission.steps`(mod.rs :67)循环"拷 scene → 画一层",`ViewTransmissionTexture`(texture.rs :24)即拷贝目标,`Transmissive3d`(phase.rs :24)是独立 phase。Zircon 取舍:字段名直通 bevy;不新设 phase 而用 09 queue 2900 段(我们的 phase 由 queue 段映射生成,新 phase 即新段,语义等价且不动 02 命令管线)。

**bevy `probe.rs` + `light_probe/irradiance_volume.rs`**:`IrradianceVolume`(:329)契约仅三字段(voxels/intensity/affects_lightmapped_meshes),体素为 ambient cube 编码 3D 纹理;`RenderViewIrradianceVolumeBindGroupEntries`(:165)对"支持 binding array 与否"给 single/multiple 双实现,`IRRADIANCE_VOLUMES_ARE_USABLE`(:161)按平台关断。Zircon 取舍:契约字段照搬 + 加 `transform`/`priority`/`layer_mask`(bevy 靠 ECS transform,我们进 extract);V1 只做 single 形态,binding array 留远期。

**URP `LightCookieManager.cs` + `LightCookie.hlsl`**:主灯 cookie 独立纹理槽、附加灯进 2D atlas,`lightBufferIndex`(:76)注释明示 cookie 数据与灯光 buffer 按索引对齐——与 Zircon "`GpuLightData` 内嵌 uv rect" 同构但我们省去间接表;三投影函数(:16/:38/:53)中 point 灯先变换到灯本地再做方向→2D 映射。Zircon 取舍:point 灯采用导入期 octahedral 预投影,运行时只做方向→oct UV,免 cube atlas。

**HDRP `PlanarReflectionProbe.cs` + `PlanarReflectionFiltering.compute`**:`PlanarReflectionProbe : HDProbe`(:14)把平面探针建模为"捕获设置随主相机镜像派生的探针",`localReferencePosition`(:21)是镜像锚点;滤波 compute 对 RT 做逐 mip roughness 模糊使粗糙材质可采样。Zircon 取舍:同样归入探针体系(进 `zr_environment.wgsl` 混合)而非独立合成 pass;镜像相机经 09 相机循环,捕获即一次普通场景渲染。

**UE `PostProcessSubsurface.cpp`**:文件头注释(:7–8)即三段结构(setup 记录 tile → indirect dispatch Burley → recombine);`FSubsurfaceTiles::ETileType`(:785)区分 AFIS/Separable tile,`ShadingModelMaskInView`(:309)在 view 级早退,`SUBSURFACE_GROUP_SIZE`(:619)= tile 尺寸。Zircon 取舍:三段与 tile indirect 原样采用,只实现 Burley 一种核(Separable 不做);view 级早退对应"帧内无 Subsurface 材质时三节点不进 graph"。

## 风险与回退

- froxel 网格注入无 Rust 同类参照:闭式单测 + 逐切片抓帧先行(index §8.8);若 3D storage 写在个别后端受限,回退为逐切片 2D dispatch(语义不变)。
- OIT 内存:1080p × avg4 ≈ 66 MB,超预算时按计划 17 降级阶梯先降 avg 再整体降回排序透明;OIT 与 09 排序路径长期共存,排序永远是默认。
- planar 成本为整次场景渲染:默认 OnDemand;EveryFrame 仅编辑器预览推荐;镜像相机的阴影/体积雾等重 feature 在镜像 view 内默认裁掉(`RenderLayerSet` + feature 白名单)。
- SSS deferred-only 是 V1 边界:forward 路径回落 StandardPbr 并诊断,不做 forward 近似。
- 与 HGI/VG 插件:froxel scatter 只读 light grid/ShadowAtlas,不感知 GI;HGI 开启时体积光是否消费其辐照由 HGI 计划后续裁决,本计划留采样函数注入点。
- **远期不承诺**(只列名,不展开、不预留接口):体积云、distance field AO、SDF 软阴影、毛发 strand 渲染、水体焦散、光追反射/RTGI(归 ray_tracing policy 与 solari 计划域)。
