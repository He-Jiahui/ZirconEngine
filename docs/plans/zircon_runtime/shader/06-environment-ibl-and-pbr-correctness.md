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
  - zircon_runtime/src/core/framework/render/environment/skybox.rs
  - zircon_runtime/src/core/framework/render/environment/cubemap_projection.rs
  - zircon_runtime/tests/runtime_environment_cubemap_projection_contract.rs
  - zircon_runtime/src/core/framework/render/image/dimension.rs
  - zircon_runtime/src/asset/assets/texture/descriptor.rs
  - zircon_runtime/src/asset/assets/texture/upload_support.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_texture/gpu_texture_resource_from_asset.rs
  - zircon_runtime/tests/runtime_texture_cube_resource_contract.rs
  - zircon_runtime/src/graphics/shader/wgsl/zr_environment.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/shaders/skybox_procedural.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/viewport_overlay_renderer/construct/create_sky_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/from_frame.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_texture/mod.rs
  - dev/cmft/src/cmft/cubemapfilter.cpp
  - dev/cmft/src/cmft/image.cpp
  - dev/cmft/src/cmft/cubemaputils.h
  - dev/cmftStudio/src/shaders/fs_mesh.shdr
  - dev/cmftStudio/src/shaders/fs_skybox.sc
  - dev/cmftStudio/src/shaders/vs_skybox.sc
  - dev/cmftStudio/src/shaders/utils.shdr
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ReflectionEnvironmentCapture.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ReflectionEnvironmentRealTimeCapture.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ReflectionEnvironmentDiffuseIrradiance.cpp
  - dev/UnrealEngine/Engine/Shaders/Private/ReflectionEnvironmentShaders.usf
  - dev/UnrealEngine/Engine/Shaders/Private/ReflectionEnvironmentShared.ush
  - dev/UnrealEngine/Engine/Shaders/Private/MonteCarlo.ush
  - dev/UnrealEngine/Engine/Shaders/Private/BRDF.ush
  - dev/UnrealEngine/Engine/Shaders/Private/SkyLightingShared.ush
  - dev/UnrealEngine/Engine/Source/Developer/TextureCompressor/Private/TextureCompressorModule.cpp
milestone_status:
  M5: in_progress
---

# 06 环境 IBL 资产链与 PBR 正确性验收

本子计划是 shader 计划集对 render 计划 11(环境光照)的**算法定稿与验收闭环补充**:计划 11 的"工程落地细化"仍是文件落点/契约类型/帧时序的实施权威;本计划定稿其中 IBL 相关 compute 的**具体算法**(equirect→cube、GGX 预滤波、SH9、BRDF LUT)、`zr_environment.wgsl` v2 的采样端契约,以及"8×8 金属度/光滑度材质球矩阵正确"的可量化验收标准。跨计划扩展按 shader index §4 惯例在计划 11/13 状态表登记镜像条目。

```zircon-workflow
{
  "schema": 1,
  "workflow_id": "zircon-runtime-shader-environment-ibl-pbr-correctness",
  "goal": "完成真实环境 IBL 资产链、GGX PMREM、PBR 采样与多视角产品验收",
  "milestones": [
    {"id": "M1", "title": "EC-M1 方向与 PBR 数学正确性", "depends_on": []},
    {"id": "M2", "title": "EC-M2 cubemap 资源与环境绑定", "depends_on": ["M1"]},
    {"id": "M3", "title": "EC-M3 离线 IBL 资产与派生物", "depends_on": []},
    {"id": "M4", "title": "EC-M4 实时 IBL 与产品验收", "depends_on": ["M3"]},
    {"id": "M5", "title": "EC-M5 当前源码交互查看器交付", "depends_on": ["M1", "M2", "M3", "M4"]}
  ]
}
```

## 1. 问题定位:当前渲染为什么全是马赛克

2026-07-04 的 SH04/Plan 11 状态行(index §4.1)所验证的"sampled environment"路径存在三个结构性缺陷,与截图观察(天空盒马赛克、mip 无模糊、PBR 矩阵不可信)一一对应:

1. **没有环境贴图,只有 128 个数**。`SkyboxSettings.sampled_equirectangular` 是 16×8=128 个 vec4 的 CPU 数组(`skybox.rs` 的 `SAMPLED_EQUIRECT_ENVIRONMENT_WIDTH/HEIGHT`),经 scene uniform 上传;`zr_environment.wgsl` 的 `zr_environment_sampled_equirect_color` 做**最近邻**索引。天空盒的物理分辨率就是 16×8——马赛克是数据形态的必然,不是滤波 bug。
2. **没有预滤波,粗糙度是假的**。`zr_environment_reflection_color` 用 `mix(sky(reflect_dir), sky(normal), roughness)` 两方向插值冒充模糊;不存在 GGX 卷积 mip 链,"mipmap 全是马赛克"是因为根本没有生成过预滤波 mip。
3. **没有 split-sum,能量是拍的**。`zr_environment_pbr_indirect` 用 `0.35`/`0.20`/`1.0 - roughness*0.65` 经验常数近似 diffuse/specular 权重;无 SH irradiance、无环境 BRDF 项,金属度/光滑度扫描不可能呈现正确梯度。

前置缺口:`gpu_texture` 无 `Cube`/`D2Array` 维度,`CubemapAsset`(计划 13 TX-M3)未开工。

2026-07-06 复核结论:用户指出的 `runtime_shader_pbr_real_hdri_lakes_reflection_20260704.png` 问题成立。该图必须标记为 **rejected baseline**:它只能证明 16x8 sampled-equirect 路径不可用,不能证明 cubemap/PMREM 方案完成。当前可继续验收的路径从 2026-07-05 source-cubemap/2K PMREM 图开始,但 EC-M3 仍需补严格高频/roughness 单调截图、SSIM 与 GPU/offline bake 对拍。

## 2. 参考实现精读结论(cmft / cmftStudio / UE5)

动手前必读下表源文件(index §5 第 8 条参考对照纪律)。以下为已完成的精读结论,实现期直接对照。

### 2.1 cmft:equirect→cubemap 与离线滤波

| 机制 | 结论 | 出处 |
|------|------|------|
| 面尺寸 | `dst_face_size = (equirect_H + 1) / 2`(2:1 全景图 → 面 = H/2;1K HDRI 512 高 → 面 256) | `image.cpp:3143` |
| texel→方向 | `uu,vv ∈ [-1,1]`,`dir = normalize(uu*faceU[f] + vv*faceV[f] + faceN[f])`;六面轴表见 `cubemaputils.h:38–70` | `cubemaputils.h:149–161` |
| 方向→equirect UV | `u = (π + atan2(x,z)) / 2π`,`v = acos(y) / π`,**双线性**采样 | `cubemaputils.h:221–228`,`image.cpp:3185–3219` |
| 逐 texel 立体角 | `areaElement(x,y)=atan2(x·y, √(x²+y²+1))` 四角差分(Rory Driscoll 精确式),用作滤波/SH 权重 | `cubemaputils.h:291–313` |
| radiance 滤波 | 逐 texel `weight = solidAngle · dotᵖ`(p=specular power)锥角裁剪 + per-face bbox 加速;gloss→power 为指数映射 `2^(scale·g+bias)` | `cubemapfilter.cpp:774–862, 1811–1855, 2100–2108` |
| 末级 mip | 1×1 时六面取平均,消除方向噪声 | `cubemapfilter.cpp:2181–2199` |
| edge fixup(Warp) | 烘焙端 `u' = u + warp·u³` 三次扭曲(nvtt 出身),采样端 `fixCubeLookup` 把非主轴分量按 `1 - 2^lod/size` 压缩——**为无 seamless filtering 的旧 GL 准备** | `cubemaputils.h:163–185`,`utils.shdr:51–63` |
| SH | 5 阶 25 系数,立体角加权累加后 `norm = 4π/Σω`;重建乘 cosine lobe `A_l = 1, 2/3, 1/4, 0, -1/24`;**无 windowing** | `cubemapfilter.cpp:124–350` |
| 消费端 | `lod = (1-gloss)·(mipCount-1)` 线性;`envSpec = fresnel(F0, NoV) · radiance`——**无 split-sum LUT**,是简化 IBL | `fs_mesh.shdr:194–225` |
| 容器 | DDS face-major / KTX mip-major;RGBA16F/RGBA32F 全支持 | `image.cpp:1416–1432, 5236, 5243–5362` |

**2026-07-07 运行时验证链取舍修订（历史，已被 2026-07-11/13 UE GGX FIS 硬切取代）**:用户指出镜面/天空盒仍不对后,当时的 CPU source-cubemap PMREM 与采样端先按 cmft/cmftStudio 消费模型对齐。方向/立体角/双线性公式与"末级 mip 平均"继续原样采纳;PMREM mip1..N 使用 `specularPowerFor(mip,mipCount,glossScale=10,glossBias=3)` + `BlinnBrdf(power/4+1)` 的 cosine-power radiance lobe,以 Hammersley 锥采样近似 cmft 的逐 texel solid-angle 积分;采样端 roughness→LOD 改为 cmftStudio 的线性 `roughness * (mipCount - 1)`,并在 skybox/specular/IEM/source cube lookup 前加入 `fixCubeLookup` 压非主轴分量。该段只保留迁移历史,不得作为当前 PMREM 实现依据。

### 2.2 UE5:捕获管线、split-sum 与实时环境

| 机制 | 结论 | 出处 |
|------|------|------|
| 捕获流程 | 逐面渲染 → mip0 premultiply alpha → `CreateCubeMips` 降采样链 → `ComputeDiffuseIrradiance`(SH9)→ `FilterCubeMap`(GGX) → 拷入 `TextureCubeArray` | `ReflectionEnvironmentCapture.cpp:490` |
| 分辨率/格式 | 128 或 256,`PF_FloatRGBA`(FP16);mip 数 = `CeilLogTwo(size)+1` | `ReflectionEnvironmentCapture.cpp:274` |
| roughness↔mip | `LevelFrom1x1 = 1.0 - 1.2·log2(max(r, 0.001))`,`mip = maxMip - 1 - LevelFrom1x1`;逆映射 `r = exp2((1.0 - LevelFrom1x1)/1.2)`——mip↔roughness 与总 mip 数无关 | `ReflectionEnvironmentShared.ush:16–39` |
| GGX 重要性采样 | Hammersley 序列,`E.y *= 0.995` 防掠射;`CosTheta = √((1-E.y)/(1+(a²-1)E.y))`,`a² = r⁴`(α=r² 的平方);`L = 2(H·V)H - V` | `MonteCarlo.ush:347–363`,`ReflectionEnvironmentShaders.usf:612–648` |
| **filtered importance sampling** | 当 `V=N` 时按样本 PDF 选源 mip:`PDF_L = D·NoH/(4·VoH) = D/4`,`Ω_s = 1/(N·PDF_L)`,`Ω_p = 4π/(6·size²)·2`,`srcMip = 0.5·log2(Ω_s/Ω_p)`;NoL 加权累加——这是 32/64 样本就能干净的关键 | `ReflectionEnvironmentShaders.usf:612–648` |
| 样本数 | 桌面 `r<0.1 ? 32 : 64`;参考路径 1024 | `ReflectionEnvironmentShaders.usf:561–580` |
| split-sum LUT | `PreIntegratedGF` 128×128,UV=(NoV, roughness),RG 双通道;`GF = F0·A + saturate(50·F0.g)·B`(F90 项) | `BRDF.ush:559–573` |
| 无 LUT 近似 | Lazarov 多项式 `EnvBRDFApprox`(常数 c0/c1 见出处),移动端用 | `BRDF.ush:583–604` |
| SH9 计算 | 实时路径:8×8=64 线程均匀球采样 mip2,树形归约,输出 8×float4(预乘 diffuse 卷积系数);离线路径:32³ 中间 cubemap 三 pass 归约 | `ReflectionEnvironmentShaders.usf:759–892`,`ReflectionEnvironmentDiffuseIrradiance.cpp:88–230` |
| SH9 重建 | `GetSkySHDiffuse`:9 点积无分支(L0+L1 一组 dot4、L2 双线性项、L2 m=±2 项),`max(0, …)` 截断 | `SkyLightingShared.ush:84–103` |
| 探针混合 | light grid 剔除后按覆盖半径大→小 under-operator 累加:`rgb += s.rgb·a_acc`,`a_acc *= 1-s.a`,`a<0.001` 早退;剩余 alpha 给 skylight | `ReflectionEnvironmentComposite.ush:12–188` |
| parallax | box:局部空间 AABB 最远交点,`DistanceAlpha = 1-smoothstep(0, 0.7·transition, boxDist)`;sphere:二次方程远交点 | `ReflectionEnvironmentShared.ush:136–201` |
| SSR/RT/Lumen 组合 | 高频源(SSR/Lumen/RT)先写,`Color.a = 1 - hit.a`,探针吃剩余 alpha,skylight 兜底,最后统一乘 `EnvBRDF` | `ReflectionEnvironmentPixelShader.usf:113–281` |
| 实时天空捕获 | 时间切片:每帧渲染 N 面(云默认 2 面/帧)→ `FDownsampleCubeFaceCS`(8×8)→ `FConvolveSpecularFaceCS` 分帧卷积 → SH 一次性 64 线程 | `ReflectionEnvironmentRealTimeCapture.cpp:340, 84–153` |

**取舍**:mip 映射常数(1.0/1.2)、GGX 采样式、FIS、样本数、LUT 尺寸/公式、SH9 归约与重建、SSR→探针→天空的 alpha under 组合契约全部原样采纳;探针混合沿用计划 11 已定稿的 top-2 截断(URP 量级)而非 UE 的 grid 全列表;light grid 剔除、Lumen 本体不在本计划范围。filtered 分支的 source LOD 只由上述 PDF 公式和 source 最大 mip 约束,不得再以 destination texel footprint 作下限;destination footprint 只用于 mip0 直接缩放/普通 source mip 构建。

### 2.3 2026-07-06 cubemap/PMREM 设计修订

- **source 与 PMREM 分离**:source cubemap mip0 是天空盒显示源;specular PMREM 是反射卷积结果。任何把 PMREM 粗 mip 用作天空背景的实现都会把反射模糊直接变成背景马赛克。
- **分辨率规则**:按 cmft 的 equirect height 规则,`polyhaven_lakes_1k.hdr`(1024x512)的 source face 是 256,`polyhaven_lakes_2k.hdr`(2048x1024)的 source face 是 512。文档和测试不得再写"1K 默认 512 face"。
- **PMREM mip 生成**:当前 CPU `pmrem.rs` 与 GPU `ibl_prefilter.wgsl` 均采用 UE ReflectionEnvironment GGX filtered importance sampling:Hammersley GGX/cosine sampling、`V=N` 下 `PDF_L=D/4`、PDF 选择 source mip、UE roughness↔mip 映射和末级六面平均。cmft/cmftStudio 继续提供 face order、投影、跨面采样、edge/final-average 与 source/PMREM 角色边界参考,不再作为生产 PMREM 的 cosine-power/线性 LOD 算法。
- **mip 模糊纪律**:source mip chain 只是 FIS 的输入 pyramid,不能当 PMREM 结果。`roughness >= 0.99` 的高粗糙度尾部按 UE `FilterPS` 的 cosine hemisphere branch 从 source cubemap 金字塔卷积采样,不能从上一层 PMREM 普通降采样;末级 1x1 六面平均;验收要覆盖高频方差下降、cube seam 下降和截图级 roughness 单调。
- **cmft/cmftStudio 保留价值**:face order、lat-long、exact solid angle、邻面 edge 处理、final mip averaging 与 IEM/PMREM 双贴图消费分工继续作为实现纪律;Warp fixup 仅作旧 GL/调试参考,wgpu 原生 cube seamless filtering 下不作为默认算法。

### 2.4 UE TextureCompressor:普通 cubemap mip 与 PMREM 的边界

| 机制 | 结论 | 出处 |
|------|------|------|
| long-lat base cube | 源图先线性化;每个 face texel 用 texel center 得到 world direction,再转 longitude/latitude 采样源 HDRI。该阶段只生成 source cubemap mip0,不做反射卷积。 | `TextureCompressorModule.cpp:2254–2315` |
| 普通纹理 mip | `GenerateMipChain` 用可分离 Gaussian/sharpen kernel 与可选 2x2 average intermediate chain;这是普通 2D 纹理 mip,不是 cubemap angular mip。 | `TextureCompressorModule.cpp:1799–2057` |
| cubemap angular mip | `GenerateAngularFilteredMips` 先构建简单平均 mip 作为输入加速,再按每个输出 mip 的 cone angle 选择输入 mip,跨六面在角锥内积分并用平滑核归一化。该算法解决 cubemap 缩小/低 mip 的角域模糊与接缝一致性。 | `TextureCompressorModule.cpp:2473–2651` |
| 角色边界 | angular cubemap mip 是 source cubemap 的 mip pyramid/texture minification/FIS source LOD,不是 specular BRDF PMREM。标准 PBR 反射仍使用 §4.2 的 UE ReflectionEnvironment GGX FIS。 | `ReflectionEnvironmentShaders.usf:612–648` + `TextureCompressorModule.cpp:2473–2651` |

设计取舍:
- `source_cubemap_mipmap` 必须成为独立 owner:输入 source mip0,输出 source mip pyramid;它可服务 skybox 缩小、FIS 源 LOD、诊断 atlas 与资产 `.zcube` source mip,但不写 PMREM。
- 当前 `source_cubemap::mipmap` 已切到 UE angular-filtered 结构：先建平均输入链，按输出 mip 角面积与 quality bias 选输入 mip，跨六面 cone 积分，末级 1x1 六面平均；旧 face-aware bilinear/box pyramid 不再是当前 owner。
- `pmrem.rs` / `ibl_prefilter.wgsl` 继续只拥有 GGX/cosine radiance convolution;不得把 ordinary/angular source mip downsample 当作 PMREM 高粗糙度结果。
- `runtime_shader_pbr_real_hdri_lakes_reflection_20260704.png` 继续作为 rejected baseline;下一张有效截图必须同时给 source skybox、PMREM mip diagnostic 和 8x8 roughness/metallic matrix 的定量指标。

## 3. 目标数据链(总览)

```
.hdr/.exr equirect(≥1K,建议 2K)
   │ 导入:计划 13 TX-M3 .zcube(RGBA16F,face-major,含全 mip)——就绪前由 EC-M1 的运行期路径顶替
   ▼
[env.equirect_to_cube]   compute/EC-M1 CPU bridge:双线性重投影 → 源 cubemap mip0(面尺寸 = min(pow2((H+1)/2), 1024))
   ▼
[env.source_cubemap::mipmap] UE angular filtered cubemap mip pyramid(过渡期可用 face-aware CPU pyramid)
   ▼                                + EC-M2a CPU GGX FIS PMREM bridge(正式 GPU/RGBA16F PMREM 前过渡)
GpuEnvironmentMap(计划 11 落点):
   ├─ [env.ibl_prefilter]      compute:128³×8mip GGX 重要性采样 + FIS(mip0 直拷,末级 6 面平均)
   ├─ [env.ibl_irradiance_sh]  compute:SH9 立体角加权树形归约(源取 32³ 档 mip)
   └─ [env.brdf_lut]           compute:128×128 RG16F split-sum LUT(设备生命周期一次,与环境内容无关)
   ▼
采样端 zr_environment.wgsl v2:
   skybox pass   → 源 cubemap(全分辨率 trilinear)          ← 消灭天空马赛克
   specular 项   → prefiltered 链 mip=对数映射 × EnvBRDF(LUT) ← 消灭假粗糙度
   diffuse 项    → SH9 求值 × albedo × (1-metallic)          ← 消灭 0.35 常数
```

关键分离:**天空盒显示采样源 cubemap,不采样 128³ 预滤波链**。当前"天空盒分辨率极低"的另一半原因是显示与 IBL 共用同一低分辨率数据;UE/Unity 的天空背景都走全分辨率源(或天空材质),预滤波链只服务反射。

烘焙产物(PMREM mip 链、SH9、可选 IEM)按 §4.7 三来源优先序取用:资产伴生离线产物 → 运行期回写缓存 → 现场 compute;命中离线产物时运行期环境 compute dispatch 为 0。

## 4. 算法定稿

### 4.1 equirect → cubemap(`equirect_to_cube.wgsl`)

- 每 texel 一线程(8×8 workgroup,z=face)。texel 中心寻址:`uu = (2(x+0.5)/size) - 1`(修正 cmft 的非中心寻址)。
- 方向公式与六面轴表照抄 cmft(`cubemaputils.h:38–70, 149–161`);方向→UV 用 `u=(π+atan2(x,z))/2π, v=acos(clamp(y,-1,1))/π`,源图双线性(equirect 源以 `texture_2d` + linear sampler 上传,直接硬件双线性)。
- 面尺寸:`clamp(next_pow2((H+1)/2), 64, 1024)`。该节点只承诺 source cubemap mip0 的正确重投影;source mip pyramid 归 `env.source_cubemap_mipmap`/`source_cubemap_mipmap.rs` 后续节点所有,避免 equirect 投影、普通 mip 与 PMREM 混在同一 owner。

### 4.1a source cubemap angular mip(`source_cubemap/mipmap.rs` / `env.source_cubemap::mipmap`)

- 输入:source cubemap mip0(RGBA16F face-major,+X,-X,+Y,-Y,+Z,-Z)。输出:source cubemap 全 mip pyramid,供 skybox trilinear minification、GGX FIS `src_mip` 采样、离线 `.zcube` source mip 与诊断 atlas 使用。
- 算法标准:参照 UE `GenerateAngularFilteredMips`。先从 mip0 构建简单平均链作为积分输入;对每个输出 mip/face/texel,以该 texel direction 为 cone axis,按 mip texel angular extent 得到 cone angle;依据球面覆盖面积和 quality bias 选择输入 mip;遍历六面中落入 cone 的 texel,用 smoothstep 类核和 cubemap solid angle 加权,最后归一化写出。
- 末级纪律:source 最终 1x1 mip 做六面共同平均,与 cmft final mip averaging 一致;这样 skybox 低 mip 与 FIS 源 LOD 不保留 face-local 方向噪声。
- 当前实现（2026-08-01）：`source_cubemap/mipmap.rs` 已从 mip0 构建 average input chain，按输出角域和 quality bias 选输入 mip，跨六面做 cone traversal，以 cubemap solid angle 与平滑核归一化，并对最终 1x1 六面共同平均；face 级任务使用既有并行执行器。该 owner 已不再是 face-aware bilinear/box bridge，后续只按定量质量与吞吐门禁继续收敛。
- 测试:高频棋盘/条纹 cubemap 的 source mip luma variance 必须随 mip 单调下降;六面边界平均差随 mip 下降;`runtime_shader_pbr_real_hdri_lakes_*_mip_diagnostic_*.png` 中 source mip 区域不允许出现 nearest-neighbor 块状放大。

### 4.2 GGX 预滤波(`ibl_prefilter.wgsl`)

固定 128³×8 mip(计划 11 定稿),每 mip 一 dispatch,特化常量 = 该 mip 的 roughness 与样本数:

```
// 每 mip roughness(采样端映射的逆,常数 1.0/1.2,mip 数无关)
level_from_1x1 = (num_mips - 1) - mip
roughness(mip) = exp2((1.0 - level_from_1x1) / 1.2)
// mip0:roughness≈0,直拷源 cubemap(重采样到 128)

// 每 texel(法线 N = texel 方向,V=N 假设,切空间 Z=N):
alpha = roughness²;a2 = alpha²                    // UE Pow4 约定
for i in 0..N_samples:
    E = hammersley(i, N);E.y *= 0.995
    cos_theta = sqrt((1 - E.y) / (1 + (a2 - 1) * E.y))
    H = spherical(2π·E.x, cos_theta)               // 切空间半向量
    L = 2·H.z·H - vec3(0,0,1)                      // 反射方向
    NoL = L.z
    if NoL > 0:
        pdf = D_GGX(a2, H.z) * 0.25
        omega_s = 1.0 / (f32(N) * pdf)
        omega_p = 4π / (6 · src_size²) * 2.0
        src_mip = clamp(0.5 * log2(omega_s / omega_p), 0.0, src_max_mip)   // FIS
        color += textureSampleLevel(src_cube, world(L), src_mip).rgb * NoL
        weight += NoL
out = color / max(weight, 1e-4)
// 末级 mip(1³ 或 2³)dispatch 后追加六面平均 pass(cmft 纪律)
```

样本数档位沿用计划 11 表(Fast 32 / Normal 64 / High 128,`roughness<0.1` 减半),**FIS 是硬性前提**——没有按 PDF 选源 mip 时这些样本数不足以消噪,这正是当前直觉"需要几千样本"与 UE 实际 32/64 的差距来源。累加在 f32 域进行,写出 RGBA16F。

### 4.3 SH9 投影(`ibl_irradiance_sh.wgsl`)与重建

- 源取源 cubemap 的 32³ 档 mip(UE `GDiffuseIrradianceCubemapSize=32` 同量级)。
- 每面 8×8 线程读全部 texel,权重用 cmft 精确 `texelSolidAngle`(非均匀球近似);workgroup 共享内存树形归约(64→1),6 面结果二段归约,`norm = 4π/Σω`。
- 输出即计划 11 的 `array<vec4<f32>, 9>` 布局(L00,L1-1,L10,L11,L2-2,L2-1,L20,L21,L22),**烘焙端预乘** cosine lobe `A_l ∈ {1, 2/3, 1/4}` 与基函数常数(Stupid SH Tricks);采样端 `zr_environment_sh9_eval(n)` 为 9 项多项式无分支求值 + `max(vec3(0), …)` 截断(数学等价于 UE `GetSkySHDiffuse` 的 dot4 重排)。无 windowing(3 阶 + 低频信号,ringing 不构成问题;若未来夹具出现负瓣再补 Hanning)。
- CPU 参考实现(测试用)与 compute 用同一权重公式,逐系数对拍。

### 4.4 环境 BRDF LUT(`brdf_lut.wgsl`)

- 128×128 `rg16float`,UV=(NoV, roughness),1024 样本 GGX 重要性采样积分 Karis split-sum 的 A/B 两项;设备生命周期生成一次(不进 `IblBakeKey`)。
- 采样端 `GF = F0 · A + saturate(50·F0.g) · B`(F90 白色项,UE 式)。
- 同时在 `zr_environment.wgsl` 保留 `zr_environment_env_brdf_approx(F0, roughness, NoV)` Lazarov 多项式(常数照抄 `BRDF.ush:583–604`)作为低端档候选；当前 MVP 未建立对应的全局 define/permutation，Standard PBR 环境反射固定走 LUT。只有计划 08 同时关闭全局质量档、PSO 预热与启动成本门禁后才允许接线，不得成为材质局部 option。

### 4.5 采样端契约(`zr_environment.wgsl` v2)

当前 scene 环境绑定固定为 group0：source cube/sampler/BRDF LUT/specular PMREM/IEM/SH9 分别在 binding 1..6；group1 binding 16..18 只供反射探针，binding 29/30 供 planar reflection。删除 `scene.environment_samples` 后接口:

```wgsl
fn zr_environment_mip_from_roughness(roughness: f32, max_mip: f32) -> f32; // §2.2 对数式
fn zr_environment_sh9_eval(n: vec3<f32>) -> vec3<f32>;                     // 预乘 SH9,9 点积
fn zr_environment_sky_color(dir: vec3<f32>) -> vec3<f32>;                   // skybox:源 cube trilinear
fn zr_environment_pbr_indirect(
    world_position, normal_ws, view_dir_ws, roughness, metallic,
    diffuse_color, base_color, occlusion, is_standard_pbr,
) -> vec3<f32>;
// 内部:
//   F0 = mix(vec3(0.04), base_color, metallic);F90 = saturate(50·F0.g)
//   specular = prefiltered(R, mip(roughness)) · (F0·A + F90·B)
//   diffuse  = zr_environment_sh9_eval(n) · diffuse_color · (1 - metallic)
//   return (diffuse + specular) · occlusion · intensity      // intensity/rotation 采样期,不进 bake key
```

roughness 定义统一:材质 smoothness → `perceptual_roughness = 1 - smoothness`;预滤波/映射/LUT 全部消费 perceptual_roughness,GGX 内部 `α = r²`。source-cubemap 与 realtime-IBL 的 PBR 分支必须删除 0.35/0.20/0.65 经验常数和两方向 lerp；无 cubemap 的 procedural-sky 回退只采样单次 reflected direction，不能作为 HDRI/PMREM 验收路径或标准 PBR 的正确性依据，必须由 EC-M4 的实时 capture/PMREM 链替换而非扩散到 source 路径。

### 4.6 探针、实时捕获与光追(契约对齐,实施归计划 11)

- **探针捕获**:6 面渲染 → 降采样 → 复用 §4.2/§4.3 同一 compute 节点(UE `FilterReflectionEnvironment` 结构);混合用计划 11 top-2 + skybox fallback,box/sphere parallax 公式计划 11 已定稿(与 UE 同式)。本计划只保证 prefilter/SH 节点以"源 cube 任意尺寸输入"为参数化前提,探针 128³ 与天空源共用。
- **实时环境捕获**(登记为 EC-M4,对齐 UE 时间切片):每帧预算 = 渲染 N 面(默认 2)+ 降采样该面 + 分帧 GGX 卷积(每帧 1 mip)+ 末帧一次性 SH;`IblBakeKey` 增加 `realtime_slice_state` 不参与——实时模式下 bake key 判定退化为"每 6+8 帧一轮"。V1 只做 procedural sky 的实时重烘(参数拖动即触发),场景几何入捕获留 EL-M2 之后。
- **当前 procedural sun 身份与热路径合同（2026-08-01）**：`IblBakeKey.source_hash` 消费有效太阳的归一化方向、RGB、强度和钳制后内外余弦边界；方向向量缩放保持同键，禁用/无效太阳保持零 hash，最终采样 `intensity` / `rotation_radians` 继续不进 bake key。CPU scene/capture upload 统一解析太阳并预计算方向与余弦；直接 procedural scene uniform 在 CPU 侧应用采样旋转，realtime capture 保留未旋转源方向，三条 WGSL invocation 热路径不再重复 `length` / 除法 / `cos`。`SceneUniform` 的 availability 位只做 mode/source 结构判断，不为布尔值构造 BLAKE3 bake key；完整 key 仅由真实 artifact/scheduler 身份路径消费。最终环境强度统一作用于 gradient 与 sun 的完整 radiance，保证 direct fallback 与 realtime cube sampling 一致。
- **光追/SSR 组合契约**:预留 alpha under 组合序 = 高频源(SSR/RT)→ 探针 → 天空,`indirect_specular.a` 表示未覆盖率,统一末端乘 EnvBRDF(UE `ReflectionEnvironmentPixelShader.usf` 结构)。V1 不实现 RT；当前可复用的分量入口是 `zr_environment_pbr_components(...)`，其 `ZrEnvironmentPbrComponents.specular` 承载镜面 IBL。若后续 pass 需要以剩余 alpha 组合，须在该结构化入口上扩展，不能声称已有未实现的 `zr_env_specular_ibl(...)` 符号。

### 4.7 离线烘焙镜面反射(PMREM)与三来源持久化

术语:PMREM = prefiltered mipmapped radiance environment map,即 §4.2 GGX 预滤波 mip 链的持久化形态(cmft CLI 的同名产物)。HDR 镜面反射的离线烘焙**不引入第二套算法**——离线与运行期跑同一份 compute WGSL(headless wgpu)或其 CPU 参考实现,产物一致性由 `render_env_prefilter_cpu_gpu_match_16` 保证。

`GpuEnvironmentMap` 装配时按三来源优先序取产物,以 `IblBakeKey` + 算法版本号判定命中:

| 优先 | 来源 | 产出时机 | 说明 |
|---|---|---|---|
| 1 | 资产伴生 derived artifact | 导入期 / staged build 期离线烘焙 | importer(计划 13 texture_importer 插件)或 `tools/zircon_build.py` staged 阶段 headless 跑 equirect→cube + prefilter + SH(+IEM),随资产持久化;镜像 shader prewarm 纪律 |
| 2 | `.zircon-cache` 回写缓存 | 首次运行期烘焙后提交后 readback 回写(计划 11 既有机制,由"仅 SH"扩为 PMREM+SH9+IEM 全产物) | 编辑器内拖 procedural 参数 / 换 HDR 的迭代路径 |
| 3 | 现场 compute | prepare 期(计划 11 帧时序 §1) | 前两级全 miss 兜底;烘完即回写为来源 2 |

- 持久化格式:face-major RGBA16F 全 mip(cmft DDS 布局纪律,`image.cpp:1416` 同序)+ SH9 144B + 可选 IEM 32³;头部携带 `IblBakeKey` 与**算法版本号**——mip 映射常数(1.0/1.2)、FIS、样本档位、SH 权重公式全部受 `IBL_BAKE_ALGORITHM_VERSION` 的同一维护纪律约束。该版本是显式维护常数；任何影响 PMREM 内容或其采样 LOD 映射的变更必须在同一补丁提升它，使旧产物自动失效重烘，不能静默混用两代产物。
- 当前 payload 合同:`IblBakeArtifactPayload` 固定按 PMREM RGBA16F 全 mip → SH9(9×vec4 f32 little-endian,144B)→ 可选 IEM 32×32×6 RGBA16F(alpha=1.0) 写入;`expected_payload_size_bytes()` 与三段 byte-range helper 是 importer/cache/file container 的共同 offset 来源。
- 当前 blob 合同:`IblBakeArtifactBlob` 固定 header + payload 完整字节容器,`decode_current_for_request(...)` 同时验证 header 魔数/格式、payload 长度和 request/key/layout/content/algorithm-version 当前性。
- 2026-07-13 source-identity 复核后,`.zribl` format v2 header 同时持久化 source face/mip 与 derived PMREM face/mip;cache hash、asset-derived path、WGPU command plan 和 runtime writeback 必须从同一 `IblBakeArtifactRequest` 构造 descriptor。相同 key/PMREM 但 source layout 不同必须拒绝命中;旧 format v1 直接失效,不保留升级 shim。
- 当前 runtime cache 合同:`IblBakeArtifactCacheStore` 在 `.zircon-cache/render/ibl/v{IBL_BAKE_ALGORITHM_VERSION}/{request_hash}/face_####_mips_##.zribl` 读写 raw blob;missing/rejected 非 fatal,只有 current blob 会转成 `IblBakeArtifactCandidate::runtime_cache(...)`。
- 当前 runtime readback writeback 合同:`IblBakeArtifactReadbackSections` 以 descriptor 为真理接收 PMREM RGBA16F、SH9 与 optional IEM RGBA16F 分段字节,拒绝缺失/多余/长度错误分段;`write_ibl_bake_artifact_runtime_readback(...)` 对 stale descriptor 不写文件,current descriptor 则组装 payload/blob 并写入 `IblBakeArtifactCacheStore`。cache miss 由 runtime compile option 保留 `environment_ibl_bake_request`;compiled-scene 已注册环境 IBL compute executor，在同一帧提交后、transient backing 归还前通过 `IblBakeArtifactWgpuReadbackResources` 和 `read_ibl_bake_artifact_wgpu_sections(...)` 从 PMREM texture、SH9 buffer 与 optional IEM texture 取得分段并回写。该 artifact 路径当前是受控的提交后同步 readback/writeback，不伪称已具备后台异步文件写入。
- 当前 WGPU RGBA16F readback helper:`read_texture_rgba16float_region(...)` 可按 mip/origin/extent 读取 `Rgba16Float` texture region 并剥离 row padding;`read_texture_rgba16float_cube_mip_chain(...)` 输出 face-major、face 内 mip 顺序的 cubemap bytes,与 PMREM artifact section layout 对齐;`SceneEnvironmentCubemap` 的 source/specular/IEM cube texture 已具备 `COPY_SRC` usage。静态环境 artifact compute 已由 compiled-scene graph 驱动；procedural gradient 则走独立 `RealtimeIblRuntime` 的双缓冲分帧 graph（每帧最多两张 capture face、后续 source mip/PMREM/SH9），在同一 command encoder 录制、提交后完成 scheduler token 并可采集 GPU timestamp。该 runtime 的 GPU textures、capture recorder 与 timestamp collector 延后至首个 procedural frame 初始化，因此 HDRI-only PBR preview 的 renderer 构造不再分配两套 realtime cubemap 资源或创建其 capture pipeline。
- 当前 resolved payload 合同:`resolve_ibl_bake_artifact_payload(...)` 消费来源 1/2 的 blob-backed candidates,复用 asset-derived > runtime-cache > runtime-compute 优先级,命中时返回可直接应用的 payload 且 dispatch=0,miss 时 `payload()==None` 并保留 required contents dispatch 计数。文件 IO 仍归 asset artifact cache/importer/staged build owner。
- 当前 runtime dispatch/cache 集成合同:`resolve_ibl_bake_artifact_runtime_dispatch(...)` 在 asset artifact 边界读取 runtime cache,合并 asset-derived blob 候选,并返回 resolved payload 与 dispatch count;`write_ibl_bake_artifact_runtime_dispatch_readback(...)` 只在该 dispatch report 需要 runtime compute 时写入 readback sections,首次 miss -> writeback -> 下一次同 request runtime cache hit 且 dispatch=0 已有合同测试,asset-derived/cache hit 时 stray readback 被跳过。该 asset 边界只负责候选决议和持久化；GPU bake resource、graph record 与帧内调度由 graphics runtime 拥有。
- 当前 source-cubemap 应用合同:`source_cubemap_mip_chain_with_bake_artifact(...)` 保留 source/display mips,仅把 decoded PMREM/SH9 payload 应用到 specular PMREM 链与 diffuse SH9;`source_cubemap_environment_with_bake_artifact(...)` 进一步保留环境 source revision/hash、intensity、rotation,并在 payload 含 IEM 时附加 `SourceCubemapIrradianceCube`;这保证天空盒仍采全分辨率源 cubemap,反射/漫反射可来自 derived/runtime cache artifact。
- 当前 WGPU upload identity 合同:`SourceCubemapEnvironment::ibl_bake_key()` 继续只描述 source 输入,artifact 应用不会改变 bake key;完整 artifact hash 留作资产/provenance,不进入 GPU upload key。`texture_upload_key()` 仅包含构造期缓存的 PMREM section hash 与 IEM content hash。`SceneEnvironmentCubemap::ensure_uploaded(...)` 在同一 source 命中不同 PMREM/SH9/IEM artifact、或直接替换同尺寸 IEM 时仅重传受影响的 source/specular/IEM cube;SH9-only 更新不重传 texture,IEM-only 更新不重传 source/PMREM,且帧热路径不重算哈希。
- 命中来源 1/2 时运行期环境 compute dispatch 数为 0(验收断言),与计划 08 MS-M4"二次启动 miss=0"同构。
- **外部预滤波 cubemap 的处理**:cmft/cmftStudio 产出的含 mip DDS/KTX 导入时,其 mip 链默认只作为源 cubemap 的降采样链消费;PMREM 仍由引擎按当前算法版本重新生成。2026-07-07 后当前运行时验证链已对齐 cmftStudio 的 Blinn/cosine-power radiance + 线性 LOD,但外部容器仍缺 Zircon 的 `IblBakeKey`、算法版本、SH9/IEM payload 与 artifact header,因此不能静默声明为可直接消费的 PMREM artifact;`.zcube` 暂不设 `prefiltered` 声明位。
- 开发者使用说明:导入 `.hdr` → importer 自动产 `.zcube` + 伴生 PMREM/SH(来源 1);编辑器改 procedural 参数 → 运行期切片重烘 + 回写(来源 2/EC-M4);发布走 staged build 全量预烘,产品运行期零环境烘焙成本。

### 4.8 IEM(irradiance cubemap)替代 SH 的 diffuse 消费选项

cmftStudio 的 diffuse 消费端是 32³ irradiance cubemap 直采(`fs_mesh.shdr:210` 的 `textureCube(u_texIem, n)`),不是 SH 求值。引擎提供两种 diffuse 消费路径,由 source 环境的 IEM availability uniform 在运行时选择:当前 artifact/环境携带 IEM 时采样它,否则回退 SH9。该选择不使用 pipeline define,不新增 PSO 或材质变体。

| 路径 | 求值 | 优势 | 默认 |
|---|---|---|---|
| SH9 | `zr_environment_sh9_eval(n)` 9 项多项式 | 零纹理带宽;与 ambient/probe grid 契约同一表示(计划 11) | artifact/环境不含 IEM 时回退 |
| IEM | `zr_environment_irradiance_cube_color(n)` 1 fetch | 无 3 阶截断 ringing;强方向性环境下更准;带宽换 ALU | source artifact/环境含 IEM 时 |

- 生成:32³ RGBA16F cubemap,余弦加权重要性采样卷积(复用 §4.2 compute 骨架,kernel 换 cosine lobe)。**不采用** cmft 的"SH 重建回 cubemap"式(`imageIrradianceFilterSh`)作为主路径——直接卷积无阶数截断,精度更高;SH 重建式仅作 Fast 档快捷路径。随 §4.7 三来源链持久化,同一 `IblBakeKey`。
- SH9 无条件仍产出:它是 ambient 三模式(计划 11 EL-M4)与 probe grid(EL-M3)契约的权威表示,IEM 只是 standard PBR diffuse 项的可选消费端,不替代契约层。
- 绑定:当前 scene group0 环境段 binding5 为 `zr_environment_irradiance_cube`(`texture_cube<f32>`),主渲染路径无 IEM 时绑定 1x1 黑色 fallback 且 availability uniform 保持 SH9 分支不采样它,有 IEM 时上传 32x32x6 RGBA16F 并在同一 pipeline 中选择 IEM。
- 一致性纪律:IEM 与 SH9 求值在低频上必须一致(球面 64 方向对拍,均值/低频差异 < 容差);高频差异是 IEM 的合法优势,不设上限。

## 5. 里程碑与切片

依赖:计划 13 TX-M3(`.zcube` 资产)未就绪前,EC-M1 以"运行期从已加载 HDR 像素生成源 cubemap"打通(数据已在 `docs/tests/runtime/shader/assets/polyhaven_lakes_1k.hdr` 链路上);TX-M3 落地时导入期直接产出含 mip 的 `.zcube`,运行期 equirect_to_cube 节点仅保留给"源是 equirect 资产"的形态,**不留双轨**(同一 `GpuEnvironmentMap` 入口,来源判别切换)。

**EC-M1 源 cubemap + skybox 全分辨率(先消灭天空马赛克)**
- 触碰:`gpu_texture`(Cube 维度 + 逐面逐 mip 上传/视图)、`equirect_to_cube.wgsl`、`gpu_environment_map.rs`(计划 11 落点)、skybox 采样改走 cube 纹理;删除 `SAMPLED_EQUIRECT_ENVIRONMENT_*`、`SkyboxMode::SampledEquirectangular` 的 sample table 形态、scene uniform `environment_samples` 字段与 `zr_environment_sampled_equirect_*` 函数(硬切换)。
- 完成判据:`cargo check -p zircon_runtime --lib --locked` 过;真实 HDRI 天空盒截图无 16×8 块状(与 `runtime_shader_pbr_real_hdri_lakes_reflection_20260704.png` 对比);仓库内 `environment_samples`/`SAMPLED_EQUIRECT` 零命中。

**EC-M2 GGX 预滤波 + SH9 + BRDF LUT + 采样端 v2(消灭假粗糙度与经验常数)**
- 切片 1(算法与采样端):`ibl_prefilter.wgsl`/`ibl_irradiance_sh.wgsl`/`brdf_lut.wgsl` 与对应 Rust 构建(计划 11 落点 `ibl_prefilter.rs` + 新增 `env_brdf_lut.rs`)、`zr_environment.wgsl` v2、binding 接线、`IblBakeKey` 消费(procedural 参数/HDR 内容哈希触发重烘)。
- 切片 2(离线持久化,§4.7):derived artifact 读取 + readback 回写(PMREM+SH9)+ 三来源优先序 + 算法版本失效;importer/staged build 预烘入口。
- 切片 3(IEM 选项,§4.8):余弦卷积 compute、IEM availability uniform 接线、`zr_environment_irradiance_cube` 槽绑定、与 SH9 低频对拍。
- 完成判据:`render_env_mip_from_roughness_roundtrip`、SH 常色→band0、CPU/GPU prefilter 16³ 小尺寸对拍、LUT 角点特值(NoV→1,r→0 时 A→1,B→0)全绿;粗糙度阶梯抓帧模糊单调;bake key 命中来源 1/2 时二次启动环境 compute dispatch 数 = 0;IEM/SH9 低频对拍绿。

**EC-M3 8×8 材质球矩阵验收闭环(直到正确)**
- 触碰:升级 ignored export 用例为 IBL 路径 + 新增非 ignored 定量断言测试(离屏 readback,§6 判据);截图归档 `docs/tests/runtime/shader/`。
- 完成判据:§6 六条判据全部量化通过,截图与 Unity/UE 校准图目视一致。

**EC-M4 探针捕获接线 + 实时天空捕获时间切片(镜像计划 11 EL-M2)**
- 触碰:归计划 11 EL-M2 实施,本计划交付参数化 prefilter/SH 节点与时间切片调度表;procedural sky 参数拖动实时重烘走切片路径。
- 完成判据:计划 11 EL-M2 判据 + 拖动 procedural 参数时帧时间无尖峰(切片生效)。

**EC-M5 当前源码交互查看器交付(关闭手动复核入口漂移)**
- 触碰:`zircon_shader_pbr_viewer` 只通过真实 runtime module descriptor 和 `ProjectAssetManagerAccess` 创建生产 `SceneRenderer`;不恢复直接 concrete manager 构造器,不增加 viewer-only renderer 旁路。
- 完成判据:当前源码 Windows production build 通过;`--help` 通过;Lakes HDRI DX12 窗口在后台烘焙期间持续响应并进入 `Ready`;鼠标左键 orbit/滚轮 zoom 保持可用;当前窗口截图和一次性 DX12 RenderDoc capture/replay 证据归档到 `docs/tests/runtime/shader`,可执行文件位于协调器管理的外部构建目录而非仓库 `target`。

执行顺序 EC-M1 → EC-M2 → EC-M3 → EC-M4 → EC-M5;EC-M1/M2 各自切片内硬切换删除被取代路径。

## 6. "8×8 材质球矩阵正确"的量化判据(全局验收)

矩阵布局沿用现有 export 用例:列 = metallic 0→1,行 = smoothness 0→1,真实 HDRI 天空。六条判据(离屏 readback 自动断言 + 截图人工复核):

1. **镜面正确性**:smoothness=1、metallic=1 球面中心区域与"按反射向量直接采样源 cubemap"的参考渲染 SSIM ≥ 0.95(天空镜像清晰可辨,湖面/地平线不糊)。
2. **粗糙度单调性**:任一 metallic 列沿 smoothness 递减方向,球面高频能量(Laplacian 方差)严格单调下降,且相邻档差异 > 噪声阈值(证明 mip 链在被使用且逐级模糊)。
3. **电介质行正确**:metallic=0 行球面主体颜色与 `SH9(n)·albedo` 参考着色的平均色差 ΔE < 阈值,且仍带 4% F0 的弱边缘高光(Fresnel 掠射变亮可见)。
4. **金属能量合理**:metallic=1、smoothness=0 球的平均亮度 ∈ [SH 均值×F0 分量下限, 源图均值×1.05]——不为黑(当前实现的典型失败),也无能量爆炸。
5. **无块状伪影**:天空背景与球面反射的块边界检测(16×8 网格对齐的梯度峰)命中数 = 0。
6. **无 cube 接缝**:预滤波链每 mip 六面共 12 条边的跨边亮度差 < 阈值(替代 cmft Warp 的防回归断言)。

判据 1–4 同时构成"调节金属度/光滑度示意图"与 Unity 校准图表 / UE 材质示例的对照基础;截图命名 `runtime_shader_pbr_ibl_metallic_smoothness_matrix_<date>.png` 归档。

## 7. 测试清单

| 测试 | 断言 | 位置 |
|---|---|---|
| `runtime_environment_cubemap_projection_contract_matches_cmft_face_axes` / `runtime_environment_equirect_uv_contract_matches_cmft_latlong_axes` | ±X/±Y/±Z 面中心与四角 texel 的方向→UV 与 cmft 公式闭式对拍 | `zircon_runtime/tests/runtime_environment_cubemap_projection_contract.rs` + `core/framework/render/environment/cubemap_projection.rs` |
| `runtime_texture_cube_resource_contract` | `dimension = "cube"` 默认 6 面、`cubemap` 别名拒绝非 6 倍数层数、完整 RGBA8 六面 payload upload-ready | `zircon_runtime/tests/runtime_texture_cube_resource_contract.rs` + `asset/assets/texture/*` + `graphics/scene/resources/gpu_texture/*` |
| `render_env_prefilter_cpu_gpu_match_16` | 16³ 源、全 mip CPU 参考(同 FIS/同序列)与 compute 输出逐 texel 容差对拍 | `ibl_prefilter.rs` |
| `render_env_prefilter_constant_env_is_identity` | 常色环境预滤波后所有 mip 仍为常色(权重归一正确性) | `ibl_prefilter.rs` |
| `render_env_sh9_matches_cpu_reference` | compute SH9 与 CPU 立体角参考逐系数容差对拍;常色→仅 band0 | `ibl_prefilter.rs` |
| `render_env_brdf_lut_corner_values` | (NoV→1, r→0) A→1,B→0;(任意, r→1) A+B < 1(能量守恒上界) | `env_brdf_lut.rs` |
| `render_env_mip_from_roughness_roundtrip` | 已在计划 11 清单,常数 1.0/1.2 双端一致 | 计划 11 |
| `runtime_environment_source_cubemap_pmrem_rough_mips_reduce_cube_seam_energy` | §6 判据 6；逐 face/edge 比较 PMREM 边界亮度，并约束中等与粗糙 mip 的 mean/max seam energy | `zircon_runtime/tests/runtime_environment_source_cubemap_contract.rs` |
| `render_product_environment_pbr_matrix_quantitative` | §6 判据 1–5 | `zircon_runtime/tests/runtime_shader_pbr_hdri_export/pbr_matrix.rs` + `pbr_matrix_quantitative.rs` |
| `render_env_derived_cache_second_launch_zero_dispatch` | bake key 命中来源 1/2 时环境 compute dispatch 计数 = 0;算法版本号变更后失效重烘 | `gpu_environment_map.rs` |
| `render_env_pmrem_artifact_roundtrip` | PMREM+SH9(+IEM) 持久化 → 重载逐字节等值;`IblBakeKey`/版本号不匹配拒绝命中 | `gpu_environment_map.rs` |
| `render_env_external_prefiltered_container_is_source_only` | 含 mip DDS/KTX 导入后 PMREM 仍由引擎生成(源 mip 只作降采样链) | 计划 13 importer 用例 |
| `render_env_iem_matches_sh9_low_frequency` | 球面 64 方向 IEM 采样与 SH9 求值低频差异 < 容差 | `ibl_prefilter.rs` |

命令沿计划 11:`cargo test -p zircon_runtime environment --locked`、`cargo test -p zircon_runtime render_product_environment --locked`。

## 8. 跨计划登记

- **计划 11**(实施权威):本计划 §4 为其 EL-M1 切片 2 的算法定稿来源;新增差异需在其状态表登记——skybox 显示采样源 cubemap(非 128³ 链)、BRDF LUT 节点与 binding、实时捕获时间切片(EC-M4)、16×8 sample table 删除清单、readback 回写由"仅 SH"扩为 PMREM+SH9+IEM 全产物(§4.7)、IEM 新增 `zr_environment_irradiance_cube` 一槽(§4.8)。
- **计划 13**(资产权威):`.zcube` 需满足 RGBA16F、face-major、全 mip、equirect/六面/十字三形态导入(TX-M3 原有范围),源面尺寸上限 1024;SH 与预滤波产物不进 `.zcube` 本体;derived artifact(PMREM/SH9/IEM)按 §4.7 作为资产伴生产物由 importer/staged build 产出;外部含 mip 容器(cmft DDS/KTX)只作源消费,`.zcube` 无 `prefiltered` 声明位。
- **计划 08**:当前无变体键增维；`zr_environment.wgsl` 仍经既有 include 注册表分发，MVP 固定使用 BRDF LUT，低端近似仅保留未接线函数。未来若启用，只能由计划 08 建立引擎全局质量维度并同时验证 PSO 预热/启动成本，不新增材质局部 option。

## 9. 状态与产出记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

- M5 DX12 Standard PBR PSO 分段测量（2026-07-28，active）：release job `97b46d95a7894b8e9434727bee4bdafa` / run `e43a0c1754f440f893484c680f4b6622` 已以 exit 0 完成；其 DX12 screenshot job `28b65efd6e25444eb34a7048b70bf1c8` / run `8201296a315443188be426cbd1e4b63b` 生成并人工复核了 `zircon_shader_pbr_viewer_m5_deferred_lighting_20260728_dx12_decomposed.png`。分段真值是 `lighting_source_assembly=1.36ms`、`pipeline_foundation=13.91ms`、`standard_pso=21.80s`，同时 IBL cache restore 仅 `595.29ms`；故历史 `82--100s` 并非 HDRI/PMREM/缓存命中，而是以 DX12 同步创建 Standard PBR PSO 为主。随后当前源码修正了首 Ready-frame 的重叠计时边界、使 one-shot screenshot/exit-after-capture 在加载失败时退出，并从片元模板删除已有独立模块的冗余全屏顶点入口。精确 viewer job `d542cbca14a74cfb8ac0897063d35780` / run `78ebe34d640040d1b99f5d7b1ce3fc15` 在任何 viewer test 前被范围外 text-cache 的 22 个 `E0277/E0308/E0369` 共享编译错误终止；新的 release/数值未声明。absolute-path RenderDoc retry `dfd65c25913543a09a3a38d101172c0a` / `88d7e6318b8a40e5925c925871311059` 已运行到 app capture completed 且 exit 0，仍未写出 RDC；M5 保持 active，不把旧 capture 或未编译源码标作验收。完整证据见 `docs/tests/runtime/shader/zircon_shader_pbr_viewer_m5_deferred_lighting_20260728_dx12_startup.md`。
- M5 DX12 跨进程缓存复核（2026-07-28，active）：对同一 released baseline、同一 Lakes HDRI/256 face 和 caller-owned IBL cache 连续启动两次，GPU jobs `283b292927104fd3afbb247ee17120b7` / `ac130a1591e743479c716dd0ca0ac4f8` 分别报告 `IBL=Reused`、`ibl_restore=842.55ms/845.74ms`，但 `standard_pso=23.04s/23.82s`、`renderer_init=31.51s/32.32s`，完整 Ready 为 `42.19s/42.65s`。这排除了 IBL/HDR 作为主因，并证明当前 DX12 驱动状态没有对该 PSO 提供可观测的跨进程复用；wgpu 29.0.3 的 application-managed cache key 也只为 Vulkan 返回值。后处理同步初始化仍占 `3.26s/3.48s`，其所有权属于 Render07，Shader06 不越界修改。Shader06 的下一项性能实现必须经显式 renderer configuration 建立最小 PBR pipeline profile，保留 generic template 对 unlit/Blinn-Phong/subsurface/plugin 的正确性契约，并继续保持非关键 SSS MRT 变体按需创建；这不是删除通用路径的临时绕过。Text01 已持有 text-cache 最低层修复租约，当前源码 exact viewer test 和 rebuild 仍等待其真实编译修复，M5 不标记 blocked 或 accepted。
- M5 Standard PBR preview profile（2026-07-28，active）：已在 `SceneRendererStartupOptions` 建立显式 `standard_pbr_preview()` 配置，PBR mirror viewer 选择该配置；默认 `FullScene` 保持任意 scene 的 unlit/Blinn-Phong/subsurface/plugin 契约，preview profile 只组装 Standard PBR 内建 deferred include，并拒绝 custom shading model，故不是删改通用着色器路径。受管 release job `55aabeda3af9469ca3c9e3a1c1a6b6f6` / run `ccf34546597743a386f7fa93d58624bc` exit 0（15m59s）；同一 Lakes 256 face、caller-owned IBL cache 的 DX12 两次 current-release 启动分别为 `standard_pso=11.65s/11.16s`、`renderer_init=21.27s/19.62s`、`IBL=866.79ms/804.01ms`、Ready `32.47s/30.69s`，相比 constructor-split baseline `21.80s` 及其连续启动 `23.04s/23.82s`，确认 profile 将同步 PSO 主耗时近乎减半。新 `zircon_shader_pbr_viewer_m5_pbr_preview_20260728_dx12.png` 已人工复核为正确 Lakes HDRI 环境反射；viewer focused retry `d1582806a28242b395c35fb630dc6bcf` / `ee8bc9b3ac8c415e8b5c0f1028df99dd` 为 60/60 passed。RenderDoc v1.44 新 retry 到 capture start/stop 且 exit 0，却仍无 `.rdc`，因此 replay 和最小 runtime source-assembly test 仍为 open；Text01 CPU reservation 不被抢占，M5 保持 active，不标记 blocked 或 accepted。完整证据见 `docs/tests/runtime/shader/zircon_shader_pbr_viewer_m5_deferred_lighting_20260728_dx12_startup.md`。
- M5 RenderDoc capture bridge（2026-07-29，active）：根因已收束为 wgpu 29 Windows 仅接受进程中已加载的 `renderdoc.dll`，且 Release 以 `debug_assertions` 禁用该集成；旧的 `renderdoccmd --capture-file` 查找 `<template>_frame*.rdc` 因此不能证明应用 API 的实际输出路径。viewer 现只在显式 one-shot capture 下允许 `--renderdoc-dll <path>` 与 `--renderdoc-capture-path <template>`，启动 wgpu 前保持 DLL 句柄并用 RenderDoc 1.4.1 `SetCaptureFilePathTemplate` 配置输出，capture stop 后查询 `GetNumCaptures`/最新路径。current-source Debug job `da06618ee2f8493db53ca92b29f60b1d` / run `e7d000f5d85b43e99ac5f7093392be09` exit 0；Lakes 256 DX12 direct run 报 `count=1`，归档 [`zircon_shader_pbr_viewer_m5_pbr_preview_20260729_dx12_renderdoc_capture.rdc`](../../../tests/runtime/shader/zircon_shader_pbr_viewer_m5_pbr_preview_20260729_dx12_renderdoc_capture.rdc)（22,129,854 bytes，SHA-256 `B4D92546E2C14196356184ED82F497CF3885CCD9AAE3020F8D27120FCDA5399B`）并以 `renderdoccmd replay --loops 1` exit 0 回放。该 Debug 证据关闭 current-source RenderDoc replay 缺口；final Release job `b51f8156fde443288039efb174f8654c` 因 Text01-owned `TextRasterWorkerPoolDiagnostics.cancelled` 初始化遗漏而在 runtime 编译前终止，新增 CLI 单测也仍由 Text01 的 CPU reservation 排队，故 M5 继续 active，不标记 accepted。
- M5 RenderDoc CLI exact gate（2026-07-29，active）：Windows test compatibility（`zircon_app` / `zircon_shader_pbr_viewer` / `--locked`）的 FIFO reservation `4fb356253eaa418b9837647dae1b1093` 已消费为 job `f2824b427274442c88f80bad8488cfc3` / run `32f0e217ada346a5bfeccacd65ad6f37`，命令为 `cargo test -p zircon_app --bin zircon_shader_pbr_viewer --locked -- --test-threads=1`。它以 exit 0 完成：64 passed / 0 failed / 0 ignored（1.05s）；新增的 capture 必须是 Debug、显式 DLL 仅随 capture 接受、DLL/template 的反向拒绝路径均通过，不能再以此前 60/60 代替。已不占 Cargo lane 地复核 Debug viewer `--help` 和缺少 DLL 的 capture-template 拒绝路径；归档 RDC 的 SHA-256 仍为 `B4D92546E2C14196356184ED82F497CF3885CCD9AAE3020F8D27120FCDA5399B`，`D:\\Tools\\renderdoc\\renderdoccmd.exe replay --loops 1` 再次 exit 0。final current-source Release gate 已登记 FIFO reservation `dad3b8f4987445b59c022f728615ffe9`，命令固定为 `cargo build -p zircon_app --bin zircon_shader_pbr_viewer --locked --release`；截至本记录它尚未获得 CPU lane，M5 保持 active，不标记 accepted。
- M5 Release gate shared-text handoff（2026-07-29，active）：上述 reservation 已消费为 job `7bf1197f62d942eab53d8d2228c791e6` / run `aeb263813ed243428a20b30e82a61c28`，但在 viewer 链接前由 Text01-owned `zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs:357` 的 `E0433`（`GlyphAtlasStorageFormat` 未导入）终止。该精确、最低层 failure 已以 `glyph-atlas-storage-format-import` 交接给 `docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md` 的 child-record-only lifecycle；Shader06 不以 alias、删除 atlas fallback 或跳过 Release gate 掩盖它。64/64 exact viewer CLI 与 Debug RenderDoc replay 仍有效，Release help/rejection 与新的 DX12 Ready-frame 必须等待 Text01 architectural fix return 后重新运行；M5 保持 active，不标记 accepted 或 blocked。
- M5 Direct Present 性能跟进（2026-07-30，in_progress）：当前源码已将 viewer 的交互路径切换到 `SceneViewportSurface` GPU present，并保留 CPU `ViewportFrame` 只服务截图。`render_and_present_call` 是 CPU 墙钟范围，不作为 GPU 时长；真实 GPU pass 时间戳仍由 Render17 的单一 backend facade 负责。静态调用链复核表明 environment-only viewer 实际提交 `BaseScenePass`，不是 GBuffer pass；因此 Shader06 的 `MeshPipelineCache` 预热从同一 `ResourceStreamer` 取 builtin PBR 当前 registry revision，并只在同一设备/缓存创建静态 no-texture、no-shadow-receiver Standard-PBR **Base** PSO，不 render、不 submit、不 readback。`EnvironmentOnlyPbrPreview` 同时延后当前 public viewer 不会访问的 deferred-lighting foundation/PSO，保留其按需创建路径；FullScene 仍保持启动预热。Render17 startup report 现保留该预热的 cache-hit、shader-source resolution、pipeline creation 和 total 四项，用于分离资源解析与 DX12 PSO 创建。该 primitive 不是“首帧已降耗”的证据，且 Base 预热与 deferred 按需创建均待 fresh managed runtime test 与 DX12 direct-present frame 验证。此前默认 feature 的 managed job `38653135ceb24c93b7d21f0b4fc868af` 虽 exit 0，却运行 `0` tests（模块受 `dynamic-api` 条件编译），不能作为门禁结果；其后的 `--features dynamic-api` reservation `22b749013398431cbc08ed91cfdb5c0b` 已消费为 job `c42082d0763b404ca57813c7c18861ee` / run `1a0849506e3d4641b955918609fd7b65`，约 15 分钟后以 exit 101 在测试发现前终止。当时最低错误均在 native discovery 接线：`discover/authority.rs` 缺少 `native_plugin_load_manifest_path`、`collect_load_manifest`，并调用了 `discovery_refresh/contract.rs` 中不可见的 `root_scan`。当前 owner `plugins01-native-discovery-compile-boundary-r1-20260730` 已记录 open failure `docs/plans/zircon_plugins/01/failure-2026-07-30-native-discovery-compile-boundary.md`，并在当前源码实施最小 `root_scan` 可见性与 TOML 1.1 fallible-parser 修复；但尚无 focused/upward managed 结果、独立复审、fixed return 或 commit。Shader06 不复制修复或将该失败归因为预热；replacement reservation `1f3a2dc0ffb747e89371c85c556b1bdc` 已不再 pending，不能复用。待该跨会话最低层修复返回后创建新的 exact reservation，重跑 focused test、DX12 Debug capture/replay。具体范围与完成条件见 `06/2026-07-29-m5-direct-present-performance.md`；M5 保持 `in_progress`，不以历史截图或旧 RDC capture 声明闭环。
- M5 Base 预热门禁（2026-07-31，in_progress）：完整名 `graphics::scene::scene_renderer::mesh::mesh_pipeline_cache::ensure_pipeline::tests::runtime_environment_only_pbr_base_prewarm_populates_the_renderer_cache` 的 fresh managed dynamic-api job `ba675041d1894317b88fe9c1a53a6987` / run `eb7aa86ba49c4ad390432129a685f00f` 已 exit 0，实际 `running 1 test` 并通过（冷编译 31m34s、测试 5.63s）。它证明同一 renderer cache 中 environment-only Standard-PBR Base PSO 只创建一次、第二次请求复用；不把这一 Base 结果外推为 deferred PSO 或首帧性能的验收。剩余的 deferred on-demand exact test 与 current-source DX12 Debug direct-present build/capture/replay 由 coordinator-managed 后续门禁给出结果，M5 保持 `in_progress`，不轮询、不回滚、不创建重复 failure。
- M5 Base 预热 current-source 编译前置（2026-07-30，in_progress）：完整 filter replacement `c4db4b2512f74f989cb4efdb9c666903` 已消费为 job `6d8739ab43d540a38a043814e2eb7f3c` / run `1b76ad888525458397f5f9a3e4b632fe`，并因共享源码变更重新编译 dynamic-api runtime lib；它在测试发现前以 exit 101 终止，不是 Base PSO 行为失败。最低错误属于 Plugins01 existing `runtime-profile-availability-rebuild`：`profile_availability_projection.rs` 的 iterator `.first()` 两处和 `availability_projection.rs` 中 `const fn` 内的 enum equality。当前工作树已分别为 `.next()` 与普通非 const helper，但缺少该 owner 的 fresh managed focused/upward result、review 和 fixed return；Shader06 不复制其修复、不新建重复 handoff，也不重用失败 job。该 return 后必须登记 fresh exact reservation，结果须出现 `running 1 test` 才能继续 M5。

本子计划产出记录已超过 10 条，具体记录已迁入编号子目录。

- 迁入记录：[`06/2026-07-09-environment-ibl-and-pbr-correctness-output-records.md`](06/2026-07-09-environment-ibl-and-pbr-correctness-output-records.md)
- fixed 已修复：[rich-inline-provider-export-name](06/fixed-2026-07-11-rich-inline-provider-export-name.md)
- fixed 已修复：[editor10-runtime-layout-method-name](06/fixed-2026-07-11-editor10-runtime-layout-method-name.md)
- fixed 已修复：[realtime-ibl-graph-recorder-type-errors](../../zircon_editor/editor_layout/15/fixed-2026-07-12-realtime-ibl-graph-recorder-type-errors.md)
- fixed 已修复：[realtime-ibl-option-then-type-errors](../../zircon_editor/editor/08/fixed-2026-07-12-realtime-ibl-option-then-type-errors.md)
- fixed 已修复：[core-filter-runtime-fixture-contracts](../runtime/02/fixed-2026-07-12-core-filter-runtime-fixture-contracts.md)
- fixed 已修复：[deferred-lighting-nested-include-resolution](06/fixed-2026-07-15-deferred-lighting-nested-include-resolution.md)
- fixed 已修复：[runtime-operation-phase-terminal-matcher](06/fixed-2026-07-15-runtime-operation-phase-terminal-matcher.md)
- fixed 已修复：[external-source-cubemap-contract-api-drift](../../zircon_editor/editor/03/fixed-2026-07-15-external-source-cubemap-contract-api-drift.md)
- fixed 已修复：[rich-text-dto-shader06-source-staging-gate](06/fixed-2026-07-15-rich-text-dto-shader06-source-staging-gate.md)
- M1 已完成：[current-source HDRI PBR acceptance](06/2026-07-14-current-source-hdri-pbr-acceptance.md)
- M2 已完成：[current-source cubemap and IBL binding acceptance](06/2026-07-15-current-source-cubemap-ibl-binding-acceptance.md)
- M3 已完成：[PMREM artifact layout convergence](06/2026-07-14-pmrem-artifact-layout-convergence.md)
- M4 已完成：[realtime SH9 parallel reduction closeout](06/2026-07-14-realtime-sh9-parallel-reduction-closeout.md)
- M5 复审修正中：[current-source interactive viewer delivery](06/2026-07-15-current-source-interactive-viewer-delivery.md)
- M5 当前源码状态（2026-07-17）：WGPU 原生 seamless cubemap lookup 契约已由受管 exact gate 验证；历史 20260715 PBR/HDRI 截图只作基线，不可替代本轮证据。当前前置为 Plugins02 的完整 Sound hard-cutover（约 355 个 runtime/docs 路径，包含 manifest、根/插件 lockfile、Kira 生产桥接、output lifecycle 和测试）；只有该 broad owner 的 immutable managed SHA 产生且当前源码/锁状态干净后，才可重新运行非忽略定量门禁、生成不可变 dated 8x8 与 cardinal-120-degree 反射图、交互查看器及 RenderDoc capture。Render13 的 source-cubemap parallel staging 返还后，还必须在同一当前源码 viewer 中记录首次烘焙与缓存命中耗时，作为 M5 性能验收；上述证据完整后才能结束 M5 复审。
- M5 外部前置修正（2026-07-17）：Plugins02 的 canonical 1.94.1 Sound 验证曾以 exit 101 终止，原始诊断定位 Kira 0.12 的 device/playback-rate/output-lifecycle 生产 API 迁移。首轮修复后库本体已编译，当前同一 full-source gate 仍被 hard-cutover 后未收敛的 test module graph 阻断（`tap_mix`、`sidechain` 与 output-device catalog `support` 的 unresolved import）；因此仍非可单独提交的 lockfile 差异。该 failure 必须由完整 Sound hard-cutover owner 在最低共享层修复、以受管 focused 与 plugin broad green rerun 关闭，并随同 canonical full manifest 返回；Shader06 不吸收其源文件，也不在失败或局部锁状态下启动 PBR/F2 门禁。
- M5 最低层编译前置（2026-07-17）：Sound graph red test 的新鲜受管重试尚未进入 Sound crate，先被 Runtime12 `input/runtime/event_buffer` 的可见性错误阻断（`FrameEventBuffer`、`InputEventRecorder` 对同级 `input_state` 的 crate 内导出错误）。canonical Runtime12 failure `input-event-buffer-visibility` 已路由给 input owner；其 focused green 与 fixed SHA 前，Sound 的 red/green、F2 和 Shader06 PBR 结果均不得声明为 current-source 验证。
- M5 Runtime12 受管重试审计（2026-07-17）：job `eba5e6584ab043569474af75268e98e2` 以 canonical Rust 1.94.1 完成 `zircon_runtime` lib-test 编译（12m51s，原始 `E0365/E0603` 未重现），但协调器 JSON 参数被误传为 libtest 参数 `-Json`，终态 `exit 101` 且实际运行零个测试（`Unrecognized option: 'J'`）。该 job 只能证明当时源码可编译，不能关闭 Runtime12 failure；同一 owner 必须以 leaf declaration `pub(in crate::input::runtime)`、保持 `event_buffer/mod.rs` 的 `pub(super)` re-export，并用命令形状正确、能报告实际测试数的 fresh focused gate 返回 fixed SHA。
- M5 Runtime12 边界真值复核（2026-07-17）：原始 blocker `ca519ce03c9b45b4a0a3b23ad0dbd06a` 的 stderr 明确显示失败 leaf 是 `pub(super) struct`，并非 `pub(in crate::input::runtime)`；后者是与 `event_buffer/mod.rs` 的 `pub(super) use` 对齐的最窄修复。后续 job `54e50eb7fdf649dcb2c69e667ede841c` 虽完成 lib 编译且实际 `running 1 test`，但仅命中 Runtime12 plan-status anchor（`Runtime 12 输入契约/runtime/tests`）并以 0/1 失败，未验证 event buffer 行为。当前若回退为更宽的 `pub(crate)`，或只修 plan-status，不得作为该 failure 的 fixed return；同一 owner 仍须在 `pub(in crate::input::runtime)` 当前源码上运行全限定 `pointer_event_streams_are_frame_bounded_at_common_polling_rates` exact gate、复审并返回 fixed SHA。
- M5 Runtime12 归属收束（2026-07-17）：failure 已转交 exact successor `runtime12-input-event-buffer-visibility-closeout-20260717`，其 scope 包含 canonical failure、`event_buffer/{frame,recorder,mod}.rs` 与 `input_state.rs`。broad 同计划 session `runtime12-input-event-bounds-closeout-r2-20260717` 当前持有更大的 `zircon_runtime/src/input` subtree，并已在其 live lease 下恢复 final narrow hashes（frame `760BD442...`, recorder `7BB7E569...`）及未变更的 `mod.rs` outlet；故 exact owner 仍不能单独把 `9527` 的 1/1 转换为 fixed。协调器必须先让 broad owner 完成 source-manifest-bound focused gate 与独立复审，再完成审计式 same-plan scope return/transfer，使 exact owner 关闭 failure、完成 review 和 managed SHA；此前 Shader06 继续阻塞。
- M5 Runtime12 行为门禁（2026-07-17）：受管 job `9527e61918764f529a1f4ebddc7b6457` 已完成正确的全限定 event-buffer gate：`pointer_event_streams_are_frame_bounded_at_common_polling_rates` 实际 `running 1 test`，结果 1 passed / 0 failed / 8233 filtered，Rust 1.94.1 编译用时 10m58s。该结果解除“未执行真实行为测试”的问题；final narrow declaration/hash 已由 broad same-plan owner 恢复，但其 source-bound focused/review 和向 exact closeout 的审计式 scope return 尚未完成，canonical failure 仍无 atomic fixed return/managed commit。没有该 SHA，Sound、F2 与 Shader06 PBR 不得把本项记作完整 current-source 前置完成。
- M5 Runtime12 复审/预留核验（2026-07-17）：当前工作树的 `frame.rs` 与 `recorder.rs` 已为 `pub(in crate::input::runtime)`，`event_buffer/mod.rs` 保持 `pub(super)` re-export。source-manifest-bound job `d064840b0a8f40dcb405bab74b493ba1` / run `78454dffc1744c858bad697721992c7e` 已于 12:56+08 以 exit 0 终态释放，真实运行 39 个 `input::tests::`，结果 39 passed / 0 failed / 8202 filtered；canonical plan-guard job `586f1f84cf814180a1bc71c48a713a90` / run `a101c9a710634fa386a5f50fb7f3b475` 也以 exit 0 终态，实际 1 passed / 0 failed / 8240 filtered。Runtime12 的 `2026-07-17-m5-input-event-bounds-current-source-closeout.md` 记录 input-stack audit 1/1、边界契约和初次 review `Critical 0 / Important 0`，但该记录及 canonical failure files 仍是未跟踪工作树文件。final fixed SHA 仅余同一 26-path manifest 的受管 `cargo check --lib`、两条 failure 原子 return、复审与 managed commit；reservation FIFO 必须保留，且所有前方真实作业自然终态前不得插队。没有 SHA，不得授权 Coordinator01 rollover/load node372599、Render01 68-path source-manifest gate、Sound、F2 或 Shader06 M5 的 PBR/GPU 验证。
- M5 Sound 后继前置（2026-07-17）：Render01 当前核验显示 `plugins02-sound-m1-kira-core-closeout-r3-20260717` 已因 heartbeat 过期而 stale。即使 Runtime12 完成 SHA，也只能解除最低 Runtime input 层；Sound owner 仍须在原完整 hard-cutover scope 上恢复 canonical Rust 1.94.1 focused/broad green、独立复审与 managed SHA。Shader06 不认领 Sound 路径，也不以 Runtime12 green 代替 Sound/PBR current-source 验收。
- M5 协调状态复核（2026-07-17）：Runtime12 前置已在 managed commit `94da2b39e79722a030b5aeb27fbcdbf3f2611c27` 闭合（精确 37 文件）：两条 canonical return 均为 `status: fixed`，source-manifest input suite 39/39、plan guard 1/1、input-stack audit 1/1，独立 review `Critical 0 / Important 0`；r3/r4 的不合规 manifest 已不再是 Shader06 阻塞。Sound `plugins02-sound-m1-kira-core-closeout-r3-20260717` 的先前 155-file focused reservation `1b77411d13be4a768d42ae84b3852942` 已在后续源修改前释放，不能复用；第二次独立 current-source review 返回 `Critical 0 / Important 3`，最低修复为：旧 persistent track 的结构性编辑必须迁移 looping playback、公开 manager state-lock 的 10/100/1000 add/update/remove/send 必须提供 p50/p95 性能证据、device catalog 不得以 `min(2)` 静默钳制 `channel_count`。Sound owner 当前为 `resolving_failure` 并正在这些最低层修复，尚无 fresh focused/broad 产品结果、复审或 immutable managed SHA。其完整 hard-cutover manifest、根/插件 lockfile 必须随同 current-source 干净的 SHA 闭合后，才可新建 Shader06 PBR/GPU reservation；Shader06 不重用已释放 job，也不在局部锁状态下启动定量、截图、viewer、RenderDoc 或 milestone commit。Render01 的 68-Rust-path fingerprint 与其他外部队列不构成本计划验收替代证据。
- M5 非验收代码优先推进（2026-07-18 状态复核）：Render13/TX-M3 已在独立 scope 完成 environment IBL staging 的 runtime-task 并行实现：equirect 基础投影、source mip 与每个 PMREM mip 的六面 GGX 工作共享注入 `ParallelSliceExecutor`，captured-face 并行入口也不再串行 PMREM；所有结果按固定 face-major 回写，cache hit 在构建前返回 `Reused`。静态 `rustfmt`/`diff --check` 已通过；首轮独立审查暴露的 PMREM 调度可证明性与 captured-face 串行漏洞均已修复，PMREM 单测要求每个 mip 恰好一次执行器调度，最终独立静态复审为 `Critical 0 / Important 0 / Minor 0`。但原 Render13 session 已因 stale retention 归档，failure node `457786` 仍为 open，历史 replacement reservation 不得复用。必须恢复原 Render13 fixing owner，以当前源码运行受管验证、独立审查、fixed return 与 managed SHA；Shader06 不接管 importer/staging，也不将静态结论替代 M5 性能、截图或 RenderDoc 验收。
- M5 查看器并行接线（2026-07-17）：当前 `zircon_shader_pbr_viewer` 的后台 scene loader 已从 HDRI helper 的串行 staging API 切换为 `CoreRuntime` TasksModule 的 compute `TaskPool` 注入的 parallel staging API；因此 equirect 基础投影、source mip 和 PMREM 现在仍在后台、且复用真实 runtime compute owner，而不是占用资产 I/O worker。查看器的 linear RGBA32F HDR decode 现在直接交给 decoded-image staging API，曝光/尺寸与 PMREM staging 复用同一份像素，不再完整解码同一 HDR 两次。查看器也不再在 `AssetManager::open_project` 完成扫描后第二次 `ProjectManager::open + scan_and_import`，而是消费 `current_project_manager()` 返回的已打开项目。临时项目仍会在退出时删除，但 IBL cache 已移至稳定的系统临时目录 `zircon_shader_pbr_viewer_ibl_cache`，所以同一 HDRI 的后续独立查看器进程可以真的返回 `Reused`；`--ibl-cache-dir` 允许调用方提供同一 caller-owned、仓库/target 外的 cache 目录，形成冷/热两次启动的可控性能对。artifact request key 继续按源内容和 layout 隔离。最后，`Drop` 先显式释放 world、renderer 和 runtime/watcher，再删除临时项目目录，避免 Windows 句柄导致后续 viewer/RDC 运行残留目录。HDRI loader 的 `Written`/`Reused`、staging elapsed 与总 elapsed 现在作为不可变报告传到 scene，再显示在 Ready 标题中；orbit/zoom 会保持同一报告，且加载中的交互不会提前把标题伪造为 Ready，供相同 current-source 二次运行量化首次 bake 与缓存命中响应。源码契约禁止查看器回退到 `stage_environment_ibl_source(&context, ...)` 或重新打开项目，并要求 `asset_runtime.task_pools().compute()` 位于 TasksModule 激活之后。此改动只完成代码/静态格式检查，仍等待受管 compile/product gate，不能声明 viewer 首次 bake 响应门禁已通过。
- M5 查看器输入收束（2026-07-17）：viewer 当前直接使用 Radiance HDR decoder 与 caller-decoded linear HDR staging，因此参数层现在只接受大小写无关的 `.hdr` 文件；非 HDR 输入会在后台 loader 创建前返回明确错误，不再晚到 decode 阶段或暗示 EXR 已受支持。解码后的输入还必须为非空 2:1 经纬展开图，才允许进入 source-cubemap/PMREM staging，避免将方图或错误资产按经纬坐标投影。后续若引入 EXR 或其他投影，必须由 importer/viewer 的显式 format/projection policy 与当前源码产品门禁一起扩展，而不是放宽本次 HDR fast path。
- M5 查看器帧路径收束（2026-07-18）：镜面查看器的 `PreviewEnvironmentExtract` 仅由加载完成后不可变的 source-cubemap `EnvironmentExtract`、固定 lighting flag 与 clear color 决定。该 preview 现在在 `PbrMirrorScene::new` 构建一次并由 frame loop clone 小型 extract，不再每帧从已 clone 的 environment 重新匹配 skybox/preview 状态；source-contract red/green guard 明确禁止恢复该每帧推导。环境本身仍按 renderer packet 所有权 clone，不声称消除了 source cubemap/PMREM payload 的共享资源引用。该优化已完成 rustfmt、scoped diff 与静态契约检查，仍等待 Runtime07 解除共享编译后由受管 viewer gate 编译/运行。
- M5 Standard PBR 漫反射与直射 GGX 收束（2026-07-18）：参照标准 metallic-roughness 合同，`baseColor` 与漫反射能量比例已拆分：standard PBR 的 direct light、ambient、environment diffuse 与 baked indirect 均只应用一次 `1 - metallic`，Blinn-Phong 保持原有漫反射行为。此前 forward/deferred/fallback/template 分别混入 `metallic * 0.45` 或 `mix(1.0, 0.55, metallic)`，导致中间金属度在不同路径中额外衰减、fallback/deferred 的 full-metal 仍残留错误漫反射。当前已在共享 helper、deferred、fallback 和两种 baked template 中消除该漂移，并增加 assembled-source 合同测试，静态检查确认旧经验缩放为零。进一步复核发现 deferred 的 Standard PBR direct light 和 fallback mesh 均曾用 Blinn exponent；fallback 还固定使用 +Z 视线，导致旋转镜面球的掠射特征错误。两条路径现在以现有 GBuffer 的 normal/view/light/roughness/metallic/albedo 实现同一无资源绑定 isotropic GGX D/G/F，并对 diffuse 使用 `/π`；deferred 重建相机视线，fallback 将 `scene_view_dir_ws` 传到 light loop。两条路径的 GGX specular 也已同 forward 一样乘 `NoL`，避免背光面 retained-specular 能量泄漏。该行为变更尚待 Runtime07 current-source 编译解除后，以受管 Naga/shader compile 和真实 HDRI 8×8 metallic/smoothness GPU 图验证，不将静态合同当作视觉验收。
- M5 viewer 基础设施 CPU gate 排队（2026-07-17，当前状态已更新）：当前源码 `zircon_app` viewer 的唯一旧 reservation 为 `5d7acd201d2a410da9cac6c070a6e6a7`，命令为 `cargo test -p zircon_app --bin zircon_shader_pbr_viewer --locked -- --nocapture --test-threads=1`、canonical Rust 1.94.1、外部 target `F:\cargo-targets\zircon-engine\shader06-viewer-ibl-current-source-20260717`。它曾正确因 `cargo_cpu_reservation_not_fifo_head` 拒绝 consume，随后以 `expired` 终态结束，从未创建 job 或启动 Cargo，严禁复用。Text01 已在下述 SHA 关闭；现在必须等待 Runtime07 current-source fixed return/managed SHA，才可按当时最新 viewer 源码新建唯一精确 reservation，静态检查不能替代 viewer compile/test green。
- M5 当前验证协调状态（2026-07-18，已闭合的外部前置）：Text01 M2 的 34 项 Coordinator action suite 曾因测试夹具以 `ActionExecutor.leases=None` 执行实际 session heartbeat 而得到 33/34；低层夹具已由隔离受管提交 `9907ca73853d546a171cce7eb9ee9d7268e94fc9` 修复，范围仅 `tools/session_coordinator/tests/test_action_concurrency.py`（`+5/-1`），同一 suite 重跑 34/34。随后 Text01 已以 immutable managed SHA `a7607a306f9f00e37004f6d668aa6cea82d76876` 完成其 8-path 双锁闭合；它不再阻塞 Shader06。历史截图仍不能作为当前源码证据，当前唯一共享编译前置见下一条 Runtime07 记录。
- M5 shared-runtime compile 前置（2026-07-18）：Text01 已以 immutable managed SHA `a7607a306f9f00e37004f6d668aa6cea82d76876` 关闭，随后 Render01/RG-M2 的 fresh snapshot470 focused gate 才进入 `zircon_runtime` 编译，但在测试执行前以 23 个 EventBus delivery-policy 迁移错误终止（旧 `subscribe` 形状、诊断与 subscriber-lock API 未完全迁移）。这是 Runtime07 canonical failure `event-bus-backpressure-and-fanout` 的最低共享层，不是 RG-M2、MD-M2 或 Shader06 的实现回归；其原 owner `runtime07-event-bus-delivery-policy-20260717` 已恢复并重新认领 26-path exact scope。Shader06 不接管 EventBus 路径，也不在 Runtime07 fixed return/managed SHA 前创建新的 viewer/PBR Cargo 或 GPU reservation；Render02 的 snapshot471 仅为独立静态进展，不能替代该 shared compile gate。
- M5 Runtime07/Text01 编译快照碰撞（2026-07-18）：Runtime07 focused job `460c3ecc46fb4600813dad72c89eaec1` / run `624ab4ef980546dcae356696b38e1eaf` 已自然释放为 `exit 101`（tests executed=0，live PIDs=[]）。原始 stderr 只有 Text01 字体测试拆分的 4 个编译错误：`tests.rs` 在当时快照找不到 `composite.rs`、`fallback.rs`（E0583），以及 `sources.rs` 的 `shared_font_database_snapshot` 路径失效和其后的类型推导错误（E0425/E0282）。该 run 只证明 rustc 读取了 in-progress foreign tree，不能作为 Runtime07、Shader06 或 Render01 的 current-source 结果。Text01 现已将包含 `tests.rs`、`composite.rs`、`fallback.rs`、`sources.rs` 与相关字体模块的 source tree 冻结在 manifest `98f4c3ac55742b0e666d93950146948967210f9ccc13853ab960c9444261d843`，并以 `waiting_validation` 等待 Sound02 后的 focused `text_font` FIFO reservation；尚无当前 manifest 的 managed SHA。Shader06、Runtime07 均不得吸收该 scope。Text01 必须完成该 fresh validation/commit；然后 Runtime07 才能刷新 manifest，在保留的 warm target 上新建 fresh exact `event_bus_` gate。此前 Shader06 不创建 viewer、PBR、截图或 RenderDoc reservation。
- M5 viewer 重绘调度收束（2026-07-18）：`request_redraw` 已是唯一将内部 pending 状态转换为 OS `window.request_redraw()` 的边沿；`about_to_wait` 仅完成后台加载轮询和受节流的状态标题刷新，不再重复提交同一 pending redraw。新增源码契约测试保留该边界，防止加载等待期间形成无效窗口事件压力。该改动已完成 Rust 1.94.1 `rustfmt --check` 和 scoped `git diff --check`，仍需与同一 current-source viewer CPU gate 一起编译/运行，不能替代 M5 的 HDRI、PBR、截图或 RenderDoc 验收。
- M5 current-source 队列修正（2026-07-18）：Runtime12 job `cfad2eb23c19439398faaba96221d102` 虽在运行，但其启动后 Editor10 `project_asset_manager` exact scope 又发生受管源修改，因此该 job 即使 exit 0 也只能保留为污染快照日志，不能用于 Runtime12 fixed return 或作为 Shader06 前置。它必须自然终态；随后由 `editor10-project-reference-regression-20260717` 冻结、验证并提交资产引用修复，再由 Runtime12 新建 fresh exact job。Layout15、Text01、Runtime07 和 Shader06 viewer/PBR 均只能消费该稳定链上的新鲜证据，不复用 `cfad` 或历史 Shader06 reservation。
- M5 HDR 输入稳定性收束（2026-07-18）：查看器自动曝光采样改为只累计有限 RGB luma，并使用 `f64` 聚合，避免真实 HDR 中的 `NaN`、`Inf` 或极亮样本把 `EnvironmentExtract` intensity 污染为非有限数。全异常采样使用受限的确定性回退曝光；新增 mixed-invalid 与 all-invalid 纯单元契约。该改动不改变有效 Radiance HDR 的曝光标定，已通过 Rust 1.94.1 `rustfmt --check`、scoped `git diff --check` 和源码契约检查；仍待同一 current-source viewer CPU gate 编译/运行。
- M5 临时项目失败清理收束（2026-07-18）：viewer 的临时项目根目录现在由 RAII owner 持有，覆盖 `PbrMirrorScene::new` 在项目打开、场景读取或 HDRI staging 任一步骤提前返回的路径；不再依赖仅在完整 scene 构造后才执行的 cleanup。正常退出仍显式先 drop world、renderer、runtime/watcher，随后由 root owner 删除目录，避免 Windows 句柄竞争；新增 partial-layout 真实文件系统单元测试。该改动已通过 Rust 1.94.1 `rustfmt --check`、scoped `git diff --check` 和生命周期源码契约检查，仍待同一 current-source viewer CPU gate 编译/运行。
- M5 HDR 预检顺序收束（2026-07-18）：viewer 现在在创建临时项目、CoreRuntime、asset manager、world 和 renderer 前读取、解码并校验 HDRI；预解码输入一次性携带 source bytes、linear RGBA32F、曝光、解析后尺寸和计时进入 staging，避免损坏/非 2:1 HDR 在失败时建立完整 runtime，也避免 staging 二次读取或二次解码。scene/source 静态契约锁定该顺序。该改动已通过 Rust 1.94.1 `rustfmt --check`、scoped `git diff --check` 和预检源码契约检查，仍待同一 current-source viewer CPU gate 编译/运行。
- M5 窗口标题热路径节流（2026-07-18）：后台 HDRI/PMREM staging 未完成时，`about_to_wait` 仍会维持 `WaitUntil` 以保证状态可见，但标题写入现在由上次刷新时间显式限为每秒最多一次；外部唤醒不再反复触发平台 `set_title`。Ready 后的鼠标拖拽和滚轮只标记标题 dirty，由成功 present 的帧边界最多刷新一次，避免输入事件与已合并 redraw 脱钩地产生平台标题调用。初始标题由窗口创建承担，完成或失败会清除节流状态；纯时间边界与 dirty 消费测试锁定无漂移 deadline 和单帧合并行为。该性能收束仅完成静态格式/源码检查，仍随同 current-source viewer CPU 与产品 gate 验证。
- M5 环境反射短路（2026-07-18）：`zr_environment_reflection_color` 现在先计算 planar 候选并在命中时立即返回；因此 planar 覆盖的像素不再无条件支付 source/realtime PMREM sky cubemap 采样和 reflection-probe 选择成本。planar 未命中且 uniform probe header 的 `probe_count == 0` 时，函数直接返回已采样 sky，不再构造空 selection 或进行两次无效 probe lookup；有 probe 时保留原有 PMREM → probe → sky 混合顺序。`zr_environment_pbr_components` 同时将 full-metal 的 diffuse energy scale 明确为零并跳过 SH9/IEM/procedural diffuse 求值，只保留 specular PMREM/BRDF；非零 metallic diffuse 保持原公式。源码顺序契约先以 red 证明各处旧路径的浪费，再锁定新的 short-circuit。该 WGSL 改动仍待同一 current-source Naga/GPU 产品门禁，不将静态契约视为图像验收。
- M5 current-source PBR 直射材质收束（2026-07-27，实现完成、等待精确受管验证）：forward 与 deferred light-grid 现在各自在每像素入口只计算一次并钳制 `direct_metallic`，随后由 F0 混合与 Lambert `/π` diffuse 共用；per-light helper 仍只消费预计算输入。fallback mesh 在 `sampled_material` 边界钳制 `material.metallic`，并以同样的每像素 F0/diffuse 预计算传入逐灯 helper；其静态契约已从删除的旧签名和逐灯表达式收敛到当前数据流。这样既使 clamp 边界上的 F0/漫反射能量一致，也避免在每盏灯重复计算材质因子。环境反射同样收束为严格数据边界：source cubemap 与 realtime IBL 仍按 `roughness -> PMREM LOD` 采样；没有 PMREM 的程序天空回退只保留单次 `reflected` 方向采样，不再以法线方向混合伪造粗糙度，并少一次天空采样。探针存在但 primary 或 secondary blend weight 为零时，当前 WGSL 也会跳过该 probe cubemap fetch，避免计算结果最终乘零的纹理采样；混合公式及非零权重路径不变。已有 AO 语义钳制到零时，环境 PBR 分量现在也在法线归一化、SH/IEM、PMREM 和 BRDF 查表前返回零 diffuse/specular，严格等价于原路径的最终乘零。deferred 与 fallback 的源码契约、格式、scoped diff、环境绑定和 permutation 静态契约均已通过。协调器受管 dev build `cargo build -p zircon_app --locked` 已在 job `5fa4839c0a5644f0bd359fe917e04942` / run `b453e1db550c47cfb32e9216ea78ded3` 以 exit 0 完成；production build `cargo build -p zircon_app --locked --release` 已在 job `9c8e527cec8b4d02adf5cb26a5965ce6` / run `c2d9622d1aac444fb6a81669bf810587` 以 exit 0 完成，耗时 21 m 41 s，release viewer 的 `--help` 也返回 0。当前 viewer 的 `--help`、Lakes HDRI DX12 Ready、加载中响应、drag/wheel、截图 `docs/tests/runtime/shader/zircon_shader_pbr_viewer_m5_current_source_20260727_dx12.png`，以及 DX12 RenderDoc capture/replay `docs/tests/runtime/shader/zircon_shader_pbr_viewer_m5_current_source_20260727_dx12_capture.rdc` 均已完成。此次 gate 同时修复了 `RuntimeEntryApp::disable_surface_present` 对 `Result<bool, _>` 的过期 `Ok(())` 匹配，明确记录 successful/unavailable/error 三种 unbind 结果。新增的两项环境采样性能模板测试已由 coordinator job `226787c942914578b59b572e651ad9fd` / run `ca75fa3972664b4285b2b434ec8bcb2e` 启动，但在执行任何测试前被不属于 Shader06 scope 的 Runtime15 test-file-budget visibility 与相关 `E0432`/`E0599`/`E0609` 编译错误以 exit 101 阻断；不能将此 run 当作性能契约 green，也不能由 Shader06 吸收该共享 runtime 源码。稳定的 lib-test 编译返回后，必须重新新建受管 exact run；随后仍需协调器 validation、独立 review 与 managed commit，完整逐项证据见 `06/2026-07-27-m5-current-source-pbr-and-viewer-validation.md`。
- M5 Navigation05 library-check handoff（2026-07-27，open）：为绕开 test-only Runtime15 编译问题而创建的 production `cargo check -p zircon_runtime --lib --locked` job `822e07fa4e7249ae81ec3055e67ecbbc` / run `6c53756ddad14a24bd0b8042385a652f` 在 `zircon_runtime/src/navigation/runtime/world_scan.rs:256,271` 遇到两项 E0599：component row 已为 `&serde_json::Value`，却继续调用 `value.as_ref()`。同一最低共享层已由 Navigation05 canonical handoff [`navigation-world-scan-deserialize-value`](../../zircon_plugins/05/failure-2026-07-27-navigation-world-scan-deserialize-value.md) 持有，Shader06 不创建重复 lifecycle、不引入局部 adapter 或兼容分支；Navigation05 fixed return 后，重新新建 library check、环境采样 exact test 与 M5 受管验证。
- M5 current-source library recovery（2026-07-27）：Navigation05 的最窄 descriptor 修复进入共享源码后，Shader06 fresh coordinator job `bf8e5ba9b5de4ad5b87c8447446b7f0a` / run `46392def34cd404c9a17e0c9e8146979` 在独立 target 完成 `cargo check -p zircon_runtime --lib --locked`，exit 0（仅既有 484 warnings）。这关闭了 Shader06 对该 production-library 编译错误的等待，但不把 `--lib` 伪作 lib-test 结果：两项环境采样 assembled-template guard 仍须以新的 exact test 真实执行。当前外部 session `plugins09-compact-validate-report-closeout-r5-20260727` 的 pending exact CPU reservation `afa46aa70aa04d3bbfe0614c2013628b` 按 FIFO 优先，Shader06 已释放未启动的普通 test lease，待该前序 reservation 自然终态后重新排队，不插队也不标记 blocked。
- M5 exact validation reservation（2026-07-27）：Shader06 已将 `cargo test -p zircon_runtime --lib --locked forward_environment_skips -- --nocapture --test-threads=1` 固化为后继 CPU reservation `4c6aa63fb66941d9a98a101c00ce3f26`（command fingerprint `064a23eb5c69dadeefe5740f7d0c49d8913191c03953558fe0330541f4ff6328`，Rust 1.94.1 Windows compatibility，pending）。它只在 Plugins09 的前序 reservation 终态后才可 consume；随后必须使用 `cargo run-reserved` 的完全相同命令启动，不能替换为普通 acquire 或旧 job，保证两项性能 guard 的可审计 current-source 执行。
- M5 exact validation passed（2026-07-27）：前序 FIFO 终态后，reservation `4c6aa63fb66941d9a98a101c00ce3f26` 被消费为 job `9ed05a8257ac4e14b9d7f30bf4fad582`，并以 `cargo run-reserved` 生成 run `3fbdf9358ec54b368ba22dcba3de6a7b`。完整 `zircon_runtime` lib-test 冷编译 26m25s 后实际运行两项 Shader06 guard：`forward_environment_skips_all_sampling_when_occlusion_is_zero` 和 `forward_environment_skips_zero_weight_probe_samples` 均通过，结果 2 passed / 0 failed / 8968 filtered，job exit 0。该 fresh current-source result 解除旧 Runtime15 pre-test 编译阻断；余下仅为受管 M5 validation、独立 review 和 managed commit。
- M5 独立复审修正中（2026-07-27）：独立 session `shader06-m5-independent-review-20260727` 拒绝了先前候选，原因是 forward/deferred/fallback 把 `occlusion` 错乘到直接光 radiance、§4.5 的 procedural-sky 文字仍称“两方向 mix”，以及模板测试新增 `&str.clone()` 警告。本轮已将 AO 严格限制为 ambient、环境 IBL 与 baked indirect，并新增三条 direct-light 无 AO 源码守卫；程序天空合同改为单次 reflected-direction 采样，冗余 clone 已删除。上述源变更使此前 M5 managed validation/review 只保留为历史证据，必须重新运行受管验证与独立复审后才能申请 managed commit；会话保持 active。
- M5 post-review exact test host handoff（2026-07-27，active）：修正后的 `direct_lighting` 精确 lib-test reservation 已通过 coordinator 消费为 job `0d61b02cc26a42708b971bf3df07d563` / run `6304d6e3d1d545198467fd648f7019a4`，但 rustc 在实际测试发现前以 exit 101 终止。错误完全位于 Shader06 范围外：runtime UI render 的 `NotificationCenterVisual.focus_ring`、`popup_rows` 的 `Option<&String>`→`Option<&str>` 形状不一致，以及三处 `NativePluginEntryReport` fixture 缺少新的 `editor_contribution_batch`。前两项由既有 EditorUI06 runtime-render collection failure 接管；后两组 native contribution/report fixture 则仍由活跃的 Editor12 native-contribution 与 Framework04 report-projection recovery scopes 接管。Shader06 不修改这些外部路径、不新建重复 failure lifecycle，继续保持 active；待 owner 返回 current-source 修复后，重新新建并受管运行完全相同的 exact test，随后再执行 M5 validation、独立复审和 managed commit。
- M5 post-review release gate handoff（2026-07-27，active）：随后通过 FIFO reservation 启动的 production job `4e4620a1530040578143a2a2898e723a` / run `f714cc862ded43119948cadbe60f3ec2` 以 exit 101 自然释放，耗时 5m04s。它只在 `zircon_runtime` 编译阶段重现 EditorUI06 负责的两项 UI shape drift（`NotificationCenterVisual.focus_ring`、`popup_rows` 的 `Option<&String>`→`Option<&str>`）；release 不编译 test fixture，故没有新增 native contribution diagnostic。没有 post-review viewer executable，旧的 DX12 screenshot/RDC 仅保留为此前 successful build 的人工复核证据，严禁作为修正后源码的 release evidence。最终独立复审还要求证据页把旧产物明确标为 pre-review historical evidence；该文字修正不改变任何代码或验收结论。当前 M5 v7 isolated coordinator validation 已由 job `eec4621a646b4af1a43972b7715660bd` / run `e82ec2e18c2a4d38bf53cf31c33530ee` exit 0、`managed_validation_succeeded` 接受；最终 independent review 亦以 0 critical / 0 important 接受。它们只验证计划版本、候选 manifest 和修复边界，不能替代 release gate。Shader06 继续 active：待 EditorUI06 fixed return 后先受管重建 release，再运行 Lakes HDRI DX12、替换截图/RDC、写入 accepted plan-output record、关闭适用 failure，最后 managed commit。
- M5 DX12 deferred-lighting startup diagnosis（2026-07-28，active）：当前源码基线的计时参数已按实际顺序复核：HDRI/IBL cache restore 约 `1.08s`，而 `DeferredSceneResources` 约 `91.74s`，scene effects 约 `4.42ms`；82--100 秒等待不是 HDR decode、PMREM 或缓存命中问题，而是标准与 SSS MRT 两条 deferred-lighting PSO 串行创建时，DX12 反复编译完整 WGSL 顶点/片元阶段。wgpu `29.0.3` 的 application-managed `PipelineCache` 持久化仅支持 Vulkan，不能在 DX12 上提供跨进程 PSO blob 复用。当前改动只在启动时创建标准 PBR 管线，SSS MRT 变体改为首次实际需要时创建；全屏顶点阶段拆为极小 WGSL module，完整 deferred WGSL 只用于片元阶段，并复用 shader module/pipeline layout。离屏 WGPU 回归已扩展为在标准预热后显式请求 SSS 变体。静态 `rustfmt --check` 与 scoped `git diff --check` 通过；release rebuild reservation `4395c565d4bb4f4fb375b709fefacf4a` 仍在 FIFO 等待，必须以该次当前源码 DX12 viewer/RenderDoc 重测量确认后才可更新截图、RDC 或声明性能验收完成。
- M5 deferred-lighting 当前源码测量（2026-07-28，active）：历史 `82--100s` 等待的主因已经由 current-source phase accounting 确认为 DX12 串行 PSO 创建，而不是 HDR decode、PMREM 或 IBL cache：旧 `DeferredSceneResources` 约 `91.74s`，IBL restore 约 `1.08s`。当前仅预热标准 PBR deferred PSO、延后 SSS MRT、并将全屏顶点阶段拆为小 WGSL module 后，Lakes HDRI / 256 face/PMREM / caller-owned cache 的 release DX12 run `b9e200859078419da0eb34a161fa98a4` / `62442e37979b40c0bf0ec75336347963` 报告 `renderer_init=31.09s`、`scene_constructed=32.14s`、`ibl_restore=843.51ms`、`deferred=22.74s`。新的分段报告确认该 deferred 时间全部是 `lighting_pipelines=22.74s`，sampler/shadow/volumetric fallback resources 仅 `700.30us`；wgpu 29.0.3 的 application-managed `PipelineCache` 持久化仅支持 Vulkan，不能在 DX12 上跨进程复用 PSO blob。该 run 的首个 Ready render 仍为 `9.14s`，但 screenshot encode 仅 `19.48ms`、surface present `1.46ms`，故截图不是首帧延迟原因。当前源码 PNG 已人工复核为正确的 Lakes HDRI 镜面反射。使用相同 current-source executable 的 RenderDoc retry job `c7ab5e9e14bd4045a77d31828f7f0479` / run `21330796957b4c98b5695aa4ac13b50b` 也在 viewer start/stop capture log 后 exit 0，却没有生成 `<state-template>_frame*.rdc`；继续将 replay 视为 open，不伪造 capture evidence。exact runtime pipeline test 仍在 FIFO reservation 中。完整证据见 `docs/tests/runtime/shader/zircon_shader_pbr_viewer_m5_deferred_lighting_20260728_dx12_startup.md`。
- M5 Ready-frame evidence 基础设施（2026-07-28，active）：为了避免 Windows firewall dialog 污染 OS 合成图，viewer 的 `--screenshot <path.png>` 只对 `scene.render()` 返回、即将提交给 presenter 的首个 Ready `ViewportFrame` 编码一次，并在成功 present 后退出；正常交互没有额外 readback、复制、编码或 timing `Instant`。CLI、frame-order 契约、PNG RGBA8 roundtrip、无效像素长度拒绝和 screenshot-only timing gate 都有 focused tests。release rebuild `7cd443a5c1d044d7b1cc701f89cf4142` / `0a86be3e0c0244c68e4787666e447777` 以 exit 0 完成后，DX12 job `b9e200859078419da0eb34a161fa98a4` / `62442e37979b40c0bf0ec75336347963` 成功写入 `docs/tests/runtime/shader/zircon_shader_pbr_viewer_m5_deferred_lighting_20260728_dx12_timing.png`。人工检查确认镜面球、Lakes HDRI 天空、地平线和高光正确，无 OS/firewall 覆盖；`render=9.14s`、`PNG=19.48ms`、`present=1.46ms` 的实测值关闭了“截图导致 8--9 秒首帧延迟”的假设。现继续执行 current-source focused tests 和 RenderDoc replay，不以旧产物替代。
- M5 Ready-frame root-cause decomposition（2026-07-28，active）：viewer 当前的 `SoftbufferViewportPresenter` 消费 CPU `ViewportFrame.rgba`，因此 `SceneRenderer::render` 的离屏路径会在首帧执行 `copy_texture_to_buffer -> map_async -> poll(wait_indefinitely)`；这与引擎已有的 `ViewportSurface::present_texture` GPU 直呈现路径不同，不能把工具呈现的读回成本外推为所有产品 runtime 的固定开销。当前源码已增加 `extract`、`render_submission` 和 `readback_and_completion` 三段报告，后两段的边界分别包围离屏提交和该同步读回/完成等待；仅在 Ready-frame screenshot 测量时打印。首次精确 viewer CPU gate reservation `f6939393a2be445d897a153df955f119` 已消费为 job `0429ceea0c3d4a6fae46fb3ad5cc4069` / run `a155d78feb684a299fa789001be39748`，但在实际测试前以 exit 101 发现本次新增 `SceneRendererFrameTimingReport` 漏经 `core/mod.rs` 重导出；该单点 Shader06 编译错误已修复。替代 reservation `4ef5f2e94cc340eba8310f82b35c0159` 在 FIFO 等待时到期且未创建 job，已按协调器流程释放；现由同一精确命令的 reservation `1d8c27af1bac4ac0b850be68b4f3ff5b` 等待分配，尚无新的数值或截图验收。常规离屏帧仅检查显式的一次性计时请求标志；未请求时不读时钟、不写报告，也不增加 GPU readback、资源创建或 PNG 编码。wgpu 29.0.3 的 application-managed pipeline-cache 持久化仅支持 Vulkan，且 native `wgpu` 缺少异步 render-pipeline API；短期性能策略仍是减少启动必须创建的 WGSL/PSO 集合，而不是伪称存在跨进程 DX12 PSO blob cache 或后台 pipeline 编译。

## Code Review Disposition (2026-07-31)

The plan now mirrors the current source: environment resources occupy group0 bindings 1..6; probe and planar resources retain their separate group1 bindings; the public sampling names and full PBR indirect signature use the implemented `zr_environment_*` forms; and the source mip owner is `source_cubemap::mipmap`. The SSR/RT future hook is described through the existing `ZrEnvironmentPbrComponents.specular` output rather than a nonexistent `zr_env_specular_ibl` symbol.

`IBL_BAKE_ALGORITHM_VERSION` is an explicit cache-invalidating maintenance value, so a PMREM content or sampling-LOD mapping change must raise it in the same source change. The corrected `max_mip - 1.0 + 1.2 * log2(roughness)` lookup advances it to `2026_07_31_0004`; bake artifacts use that version, while the manual quantitative export combines it with the BLAKE3 of the running lib-test executable and the bytes of the 2K Lakes HDR input. The executable hash is frozen before rendering and is the code provenance for the pixels; the HDR hash is frozen before rendering and rechecked immediately before the immutable PNG/TXT pair commits. The TXT report records both values, so an unbuilt worktree edit cannot masquerade as code that produced the image, while a PBR/material/renderer/shader correction that is actually rebuilt obtains a new executable identity. The non-export quantitative render path performs neither hashing step.

The 2026-07-31 follow-up independent review found the realtime IBL lazy initialization clean: both render paths initialize before bind-group sampling, the first procedural frame still writes the work slot before scene draws, and submission/failed-record token handling is unchanged. HDRI-only frames leave diagnostics as safe no-ops and do not allocate realtime GPU resources.

M5 acceptance still requires current-source executions of `runtime_environment_source_cubemap_pmrem_rough_mips_reduce_cube_seam_energy`, `render_product_environment_pbr_matrix_quantitative`, `realtime_runtime_defers_gpu_resource_creation_until_a_procedural_frame`, and `first_procedural_frame_initializes_realtime_gpu_resources_and_starts_full_batch`, in addition to the managed direct-present gates above. Existing screenshots and historical RenderDoc artifacts remain visual baselines only, not substitutes for those quantitative readback assertions.

The 2026-08-01 procedural-sun convergence closes a stale realtime-bake identity hole and removes uniform normalization plus two cosine evaluations from each direct/capture shader invocation. It also makes sampling-time intensity and rotation consistent before and after realtime IBL publication. Scoped Rust formatting, diff integrity, and legacy per-invocation sun-math scans pass. The required second independent review reports 0 critical / 0 important / 0 minor after verifying bake identity, the 112-byte capture ABI, CPU/WGSL rotation signs, direct/realtime intensity semantics, lazy availability checks, and the invalid-direction regression. M5 remains `in_progress`: these source changes still require fresh managed tests, quantitative export, DX12 viewer timing, screenshot, and RenderDoc replay before accepted closeout.
