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
- 当前 CPU bridge 的 face-aware bilinear/box pyramid 只允许作为过渡实现;产品/离线标准切到 UE angular filtered cubemap mip:先建平均链,按输出 mip 的角面积与 quality bias 选输入 mip,跨六面 cone 积分,末级 1x1 六面平均。
- `pmrem.rs` / `ibl_prefilter.wgsl` 继续只拥有 GGX/cosine radiance convolution;不得把 ordinary/angular source mip downsample 当作 PMREM 高粗糙度结果。
- `runtime_shader_pbr_real_hdri_lakes_reflection_20260704.png` 继续作为 rejected baseline;下一张有效截图必须同时给 source skybox、PMREM mip diagnostic 和 8x8 roughness/metallic matrix 的定量指标。

## 3. 目标数据链(总览)

```
.hdr/.exr equirect(≥1K,建议 2K)
   │ 导入:计划 13 TX-M3 .zcube(RGBA16F,face-major,含全 mip)——就绪前由 EC-M1 的运行期路径顶替
   ▼
[env.equirect_to_cube]   compute/EC-M1 CPU bridge:双线性重投影 → 源 cubemap mip0(面尺寸 = min(pow2((H+1)/2), 1024))
   ▼
[env.source_cubemap_mipmap] UE angular filtered cubemap mip pyramid(过渡期可用 face-aware CPU pyramid)
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

### 4.1a source cubemap angular mip(`source_cubemap_mipmap.rs` / `env.source_cubemap_mipmap`)

- 输入:source cubemap mip0(RGBA16F face-major,+X,-X,+Y,-Y,+Z,-Z)。输出:source cubemap 全 mip pyramid,供 skybox trilinear minification、GGX FIS `src_mip` 采样、离线 `.zcube` source mip 与诊断 atlas 使用。
- 算法标准:参照 UE `GenerateAngularFilteredMips`。先从 mip0 构建简单平均链作为积分输入;对每个输出 mip/face/texel,以该 texel direction 为 cone axis,按 mip texel angular extent 得到 cone angle;依据球面覆盖面积和 quality bias 选择输入 mip;遍历六面中落入 cone 的 texel,用 smoothstep 类核和 cubemap solid angle 加权,最后归一化写出。
- 末级纪律:source 最终 1x1 mip 做六面共同平均,与 cmft final mip averaging 一致;这样 skybox 低 mip 与 FIS 源 LOD 不保留 face-local 方向噪声。
- 过渡实现:当前 CPU bridge 可继续使用 face-aware bilinear/box source pyramid 解锁 PMREM/FIS 验证,但状态表必须标记为 bridge;产品验收和离线 artifact 不得把该过渡链声明为 Unreal angular mip 完成。
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
- 输出即计划 11 的 `array<vec4<f32>, 9>` 布局(L00,L1-1,L10,L11,L2-2,L2-1,L20,L21,L22),**烘焙端预乘** cosine lobe `A_l ∈ {1, 2/3, 1/4}` 与基函数常数(Stupid SH Tricks);采样端 `zr_env_sh9_eval(n)` 为 9 项多项式无分支求值 + `max(vec3(0), …)` 截断(数学等价于 UE `GetSkySHDiffuse` 的 dot4 重排)。无 windowing(3 阶 + 低频信号,ringing 不构成问题;若未来夹具出现负瓣再补 Hanning)。
- CPU 参考实现(测试用)与 compute 用同一权重公式,逐系数对拍。

### 4.4 环境 BRDF LUT(`brdf_lut.wgsl`)

- 128×128 `rg16float`,UV=(NoV, roughness),1024 样本 GGX 重要性采样积分 Karis split-sum 的 A/B 两项;设备生命周期生成一次(不进 `IblBakeKey`)。
- 采样端 `GF = F0 · A + saturate(50·F0.g) · B`(F90 白色项,UE 式)。
- 同时在 `zr_environment.wgsl` 提供 `zr_env_brdf_approx(F0, roughness, NoV)` Lazarov 多项式(常数照抄 `BRDF.ush:583–604`)作为低端档 fallback,由引擎 define 选择,默认走 LUT。

### 4.5 采样端契约(`zr_environment.wgsl` v2)

绑定沿计划 11 group1 binding 16..23 表(sky specular cube、sampler、SH9 buffer),新增 BRDF LUT 一槽(占用计划 11 表的 23 预留位之前的空位由实施期在计划 11 登记确切编号)。删除 `scene.environment_samples` 后接口:

```wgsl
fn zr_env_mip_from_roughness(roughness: f32, max_mip: f32) -> f32;   // §2.2 对数式
fn zr_env_sh9_eval(n: vec3<f32>) -> vec3<f32>;                        // 预乘 SH9,9 点积
fn zr_environment_sky_color(dir: vec3<f32>) -> vec3<f32>;             // skybox:源 cube trilinear
fn zr_environment_pbr_indirect(n, v, roughness, metallic, diffuse_color, base_color, occlusion, …) -> vec3<f32>;
// 内部:
//   F0 = mix(vec3(0.04), base_color, metallic);F90 = saturate(50·F0.g)
//   specular = prefiltered(R, mip(roughness)) · (F0·A + F90·B)
//   diffuse  = zr_env_sh9_eval(n) · diffuse_color · (1 - metallic)
//   return (diffuse + specular) · occlusion · intensity      // intensity/rotation 采样期,不进 bake key
```

roughness 定义统一:材质 smoothness → `perceptual_roughness = 1 - smoothness`;预滤波/映射/LUT 全部消费 perceptual_roughness,GGX 内部 `α = r²`。经验常数路径(0.35/0.20/0.65 与两方向 lerp)整体删除。

### 4.6 探针、实时捕获与光追(契约对齐,实施归计划 11)

- **探针捕获**:6 面渲染 → 降采样 → 复用 §4.2/§4.3 同一 compute 节点(UE `FilterReflectionEnvironment` 结构);混合用计划 11 top-2 + skybox fallback,box/sphere parallax 公式计划 11 已定稿(与 UE 同式)。本计划只保证 prefilter/SH 节点以"源 cube 任意尺寸输入"为参数化前提,探针 128³ 与天空源共用。
- **实时环境捕获**(登记为 EC-M4,对齐 UE 时间切片):每帧预算 = 渲染 N 面(默认 2)+ 降采样该面 + 分帧 GGX 卷积(每帧 1 mip)+ 末帧一次性 SH;`IblBakeKey` 增加 `realtime_slice_state` 不参与——实时模式下 bake key 判定退化为"每 6+8 帧一轮"。V1 只做 procedural sky 的实时重烘(参数拖动即触发),场景几何入捕获留 EL-M2 之后。
- **光追/SSR 组合契约**:预留 alpha under 组合序 = 高频源(SSR/RT)→ 探针 → 天空,`indirect_specular.a` 表示未覆盖率,统一末端乘 EnvBRDF(UE `ReflectionEnvironmentPixelShader.usf` 结构)。V1 不实现 RT,只保证 `zr_environment_pbr_indirect` 拆出 `zr_env_specular_ibl(...)` 可被后续 pass 以剩余 alpha 调用。

### 4.7 离线烘焙镜面反射(PMREM)与三来源持久化

术语:PMREM = prefiltered mipmapped radiance environment map,即 §4.2 GGX 预滤波 mip 链的持久化形态(cmft CLI 的同名产物)。HDR 镜面反射的离线烘焙**不引入第二套算法**——离线与运行期跑同一份 compute WGSL(headless wgpu)或其 CPU 参考实现,产物一致性由 `render_env_prefilter_cpu_gpu_match_16` 保证。

`GpuEnvironmentMap` 装配时按三来源优先序取产物,以 `IblBakeKey` + 算法版本号判定命中:

| 优先 | 来源 | 产出时机 | 说明 |
|---|---|---|---|
| 1 | 资产伴生 derived artifact | 导入期 / staged build 期离线烘焙 | importer(计划 13 texture_importer 插件)或 `tools/zircon_build.py` staged 阶段 headless 跑 equirect→cube + prefilter + SH(+IEM),随资产持久化;镜像 shader prewarm 纪律 |
| 2 | `.zircon-cache` 回写缓存 | 首次运行期烘焙后异步 readback 回写(计划 11 既有机制,由"仅 SH"扩为 PMREM+SH9+IEM 全产物) | 编辑器内拖 procedural 参数 / 换 HDR 的迭代路径 |
| 3 | 现场 compute | prepare 期(计划 11 帧时序 §1) | 前两级全 miss 兜底;烘完即回写为来源 2 |

- 持久化格式:face-major RGBA16F 全 mip(cmft DDS 布局纪律,`image.cpp:1416` 同序)+ SH9 144B + 可选 IEM 32³;头部携带 `IblBakeKey` 与**算法版本号**——mip 映射常数(1.0/1.2)、FIS、样本档位、SH 权重公式全部入版本,算法升级旧产物自动失效重烘,不会静默混用两代产物。
- 当前 payload 合同:`IblBakeArtifactPayload` 固定按 PMREM RGBA16F 全 mip → SH9(9×vec4 f32 little-endian,144B)→ 可选 IEM 32×32×6 RGBA16F(alpha=1.0) 写入;`expected_payload_size_bytes()` 与三段 byte-range helper 是 importer/cache/file container 的共同 offset 来源。
- 当前 blob 合同:`IblBakeArtifactBlob` 固定 header + payload 完整字节容器,`decode_current_for_request(...)` 同时验证 header 魔数/格式、payload 长度和 request/key/layout/content/algorithm-version 当前性。
- 2026-07-13 source-identity 复核后,`.zribl` format v2 header 同时持久化 source face/mip 与 derived PMREM face/mip;cache hash、asset-derived path、WGPU command plan 和 runtime writeback 必须从同一 `IblBakeArtifactRequest` 构造 descriptor。相同 key/PMREM 但 source layout 不同必须拒绝命中;旧 format v1 直接失效,不保留升级 shim。
- 当前 runtime cache 合同:`IblBakeArtifactCacheStore` 在 `.zircon-cache/render/ibl/v{IBL_BAKE_ALGORITHM_VERSION}/{request_hash}/face_####_mips_##.zribl` 读写 raw blob;missing/rejected 非 fatal,只有 current blob 会转成 `IblBakeArtifactCandidate::runtime_cache(...)`。
- 当前 runtime readback writeback 合同:`IblBakeArtifactReadbackSections` 以 descriptor 为真理接收 PMREM RGBA16F、SH9 与 optional IEM RGBA16F 分段字节,拒绝缺失/多余/长度错误分段;`write_ibl_bake_artifact_runtime_readback(...)` 对 stale descriptor 不写文件,current descriptor 则组装 payload/blob 并写入 `IblBakeArtifactCacheStore`。WGPU section acquisition 由 `IblBakeArtifactWgpuReadbackResources` 与 `read_ibl_bake_artifact_wgpu_sections(...)` 装配已有 PMREM texture、SH9 buffer 与 optional IEM texture,实际 render graph/async scheduling 仍由后续 runtime/GPU 切片拥有。
- 当前 WGPU RGBA16F readback helper:`read_texture_rgba16float_region(...)` 可按 mip/origin/extent 读取 `Rgba16Float` texture region 并剥离 row padding;`read_texture_rgba16float_cube_mip_chain(...)` 输出 face-major、face 内 mip 顺序的 cubemap bytes,与 PMREM artifact section layout 对齐;`SceneEnvironmentCubemap` 的 source/specular/IEM cube texture 已具备 `COPY_SRC` usage。SH9 buffer readback helper 与 IBL artifact section acquisition helper 已存在;IBL compute 生产和调度仍待后续切片。
- 当前 resolved payload 合同:`resolve_ibl_bake_artifact_payload(...)` 消费来源 1/2 的 blob-backed candidates,复用 asset-derived > runtime-cache > runtime-compute 优先级,命中时返回可直接应用的 payload 且 dispatch=0,miss 时 `payload()==None` 并保留 required contents dispatch 计数。文件 IO 仍归 asset artifact cache/importer/staged build owner。
- 当前 runtime dispatch/cache 集成合同:`resolve_ibl_bake_artifact_runtime_dispatch(...)` 在 asset artifact 边界读取 runtime cache,合并 asset-derived blob 候选,并返回 resolved payload 与 dispatch count;`write_ibl_bake_artifact_runtime_dispatch_readback(...)` 只在该 dispatch report 需要 runtime compute 时写入 readback sections,首次 miss -> writeback -> 下一次同 request runtime cache hit 且 dispatch=0 已有合同测试,asset-derived/cache hit 时 stray readback 被跳过。该合同仍不生产 GPU bake 资源,也不拥有 render graph async scheduling。
- 当前 source-cubemap 应用合同:`source_cubemap_mip_chain_with_bake_artifact(...)` 保留 source/display mips,仅把 decoded PMREM/SH9 payload 应用到 specular PMREM 链与 diffuse SH9;`source_cubemap_environment_with_bake_artifact(...)` 进一步保留环境 source revision/hash、intensity、rotation,并在 payload 含 IEM 时附加 `SourceCubemapIrradianceCube`;这保证天空盒仍采全分辨率源 cubemap,反射/漫反射可来自 derived/runtime cache artifact。
- 当前 WGPU upload identity 合同:`SourceCubemapEnvironment::ibl_bake_key()` 继续只描述 source 输入,artifact 应用不会改变 bake key;`texture_upload_key()` 额外包含 artifact payload hash,让 `SceneEnvironmentCubemap::ensure_uploaded(...)` 在同一 source 命中不同 PMREM/SH9/IEM artifact 时重传 source/specular/IEM cube。
- 命中来源 1/2 时运行期环境 compute dispatch 数为 0(验收断言),与计划 08 MS-M4"二次启动 miss=0"同构。
- **外部预滤波 cubemap 的处理**:cmft/cmftStudio 产出的含 mip DDS/KTX 导入时,其 mip 链默认只作为源 cubemap 的降采样链消费;PMREM 仍由引擎按当前算法版本重新生成。2026-07-07 后当前运行时验证链已对齐 cmftStudio 的 Blinn/cosine-power radiance + 线性 LOD,但外部容器仍缺 Zircon 的 `IblBakeKey`、算法版本、SH9/IEM payload 与 artifact header,因此不能静默声明为可直接消费的 PMREM artifact;`.zcube` 暂不设 `prefiltered` 声明位。
- 开发者使用说明:导入 `.hdr` → importer 自动产 `.zcube` + 伴生 PMREM/SH(来源 1);编辑器改 procedural 参数 → 运行期切片重烘 + 回写(来源 2/EC-M4);发布走 staged build 全量预烘,产品运行期零环境烘焙成本。

### 4.8 IEM(irradiance cubemap)替代 SH 的 diffuse 消费选项

cmftStudio 的 diffuse 消费端是 32³ irradiance cubemap 直采(`fs_mesh.shdr:210` 的 `textureCube(u_texIem, n)`),不是 SH 求值。引擎提供两种 diffuse 消费路径,由**引擎 define** `ZR_ENV_DIFFUSE_IEM`(全局维度,走计划 08 defines,不是材质 option,不增材质变体)切换:

| 路径 | 求值 | 优势 | 默认 |
|---|---|---|---|
| SH9 | `zr_env_sh9_eval(n)` 9 项多项式 | 零纹理带宽;与 ambient/probe grid 契约同一表示(计划 11) | 是 |
| IEM | `textureSample(zr_env_irradiance, zr_env_sampler, n)` 1 fetch | 无 3 阶截断 ringing;强方向性环境下更准;带宽换 ALU | 质量档选项 |

- 生成:32³ RGBA16F cubemap,余弦加权重要性采样卷积(复用 §4.2 compute 骨架,kernel 换 cosine lobe)。**不采用** cmft 的"SH 重建回 cubemap"式(`imageIrradianceFilterSh`)作为主路径——直接卷积无阶数截断,精度更高;SH 重建式仅作 Fast 档快捷路径。随 §4.7 三来源链持久化,同一 `IblBakeKey`。
- SH9 无条件仍产出:它是 ambient 三模式(计划 11 EL-M4)与 probe grid(EL-M3)契约的权威表示,IEM 只是 standard PBR diffuse 项的可选消费端,不替代契约层。
- 绑定:当前 scene group0 环境段 binding5 为 `zr_environment_irradiance_cube`(`texture_cube<f32>`),主渲染路径无 IEM 时绑定 1x1 黑色 fallback,有 IEM 时上传 32x32x6 RGBA16F;`ZR_ENV_DIFFUSE_IEM` 默认 false,后续由 engine quality/specialization 决定启用。
- 一致性纪律:IEM 与 SH9 求值在低频上必须一致(球面 64 方向对拍,均值/低频差异 < 容差);高频差异是 IEM 的合法优势,不设上限。

## 5. 里程碑与切片

依赖:计划 13 TX-M3(`.zcube` 资产)未就绪前,EC-M1 以"运行期从已加载 HDR 像素生成源 cubemap"打通(数据已在 `docs/tests/runtime/shader/assets/polyhaven_lakes_1k.hdr` 链路上);TX-M3 落地时导入期直接产出含 mip 的 `.zcube`,运行期 equirect_to_cube 节点仅保留给"源是 equirect 资产"的形态,**不留双轨**(同一 `GpuEnvironmentMap` 入口,来源判别切换)。

**EC-M1 源 cubemap + skybox 全分辨率(先消灭天空马赛克)**
- 触碰:`gpu_texture`(Cube 维度 + 逐面逐 mip 上传/视图)、`equirect_to_cube.wgsl`、`gpu_environment_map.rs`(计划 11 落点)、skybox 采样改走 cube 纹理;删除 `SAMPLED_EQUIRECT_ENVIRONMENT_*`、`SkyboxMode::SampledEquirectangular` 的 sample table 形态、scene uniform `environment_samples` 字段与 `zr_environment_sampled_equirect_*` 函数(硬切换)。
- 完成判据:`cargo check -p zircon_runtime --lib --locked` 过;真实 HDRI 天空盒截图无 16×8 块状(与 `runtime_shader_pbr_real_hdri_lakes_reflection_20260704.png` 对比);仓库内 `environment_samples`/`SAMPLED_EQUIRECT` 零命中。

**EC-M2 GGX 预滤波 + SH9 + BRDF LUT + 采样端 v2(消灭假粗糙度与经验常数)**
- 切片 1(算法与采样端):`ibl_prefilter.wgsl`/`ibl_irradiance_sh.wgsl`/`brdf_lut.wgsl` 与对应 Rust 构建(计划 11 落点 `ibl_prefilter.rs` + 新增 `env_brdf_lut.rs`)、`zr_environment.wgsl` v2、binding 接线、`IblBakeKey` 消费(procedural 参数/HDR 内容哈希触发重烘)。
- 切片 2(离线持久化,§4.7):derived artifact 读取 + readback 回写(PMREM+SH9)+ 三来源优先序 + 算法版本失效;importer/staged build 预烘入口。
- 切片 3(IEM 选项,§4.8):余弦卷积 compute、`ZR_ENV_DIFFUSE_IEM` define 接线、`zr_env_irradiance` 槽绑定、与 SH9 低频对拍。
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
| `render_env_seam_luminance_below_threshold` | §6 判据 6 | `render_product_environment.rs` |
| `render_product_environment_pbr_matrix_quantitative` | §6 判据 1–5 | `zircon_runtime/tests/runtime_shader_pbr_hdri_export/pbr_matrix.rs` + `pbr_matrix_quantitative.rs` |
| `render_env_derived_cache_second_launch_zero_dispatch` | bake key 命中来源 1/2 时环境 compute dispatch 计数 = 0;算法版本号变更后失效重烘 | `gpu_environment_map.rs` |
| `render_env_pmrem_artifact_roundtrip` | PMREM+SH9(+IEM) 持久化 → 重载逐字节等值;`IblBakeKey`/版本号不匹配拒绝命中 | `gpu_environment_map.rs` |
| `render_env_external_prefiltered_container_is_source_only` | 含 mip DDS/KTX 导入后 PMREM 仍由引擎生成(源 mip 只作降采样链) | 计划 13 importer 用例 |
| `render_env_iem_matches_sh9_low_frequency` | 球面 64 方向 IEM 采样与 SH9 求值低频差异 < 容差 | `ibl_prefilter.rs` |

命令沿计划 11:`cargo test -p zircon_runtime environment --locked`、`cargo test -p zircon_runtime render_product_environment --locked`。

## 8. 跨计划登记

- **计划 11**(实施权威):本计划 §4 为其 EL-M1 切片 2 的算法定稿来源;新增差异需在其状态表登记——skybox 显示采样源 cubemap(非 128³ 链)、BRDF LUT 节点与 binding、实时捕获时间切片(EC-M4)、16×8 sample table 删除清单、readback 回写由"仅 SH"扩为 PMREM+SH9+IEM 全产物(§4.7)、IEM 新增 `zr_env_irradiance` 一槽(§4.8)。
- **计划 13**(资产权威):`.zcube` 需满足 RGBA16F、face-major、全 mip、equirect/六面/十字三形态导入(TX-M3 原有范围),源面尺寸上限 1024;SH 与预滤波产物不进 `.zcube` 本体;derived artifact(PMREM/SH9/IEM)按 §4.7 作为资产伴生产物由 importer/staged build 产出;外部含 mip 容器(cmft DDS/KTX)只作源消费,`.zcube` 无 `prefiltered` 声明位。
- **计划 08**:无变体键增维;`zr_environment.wgsl` 仍经既有 include 注册表分发,BRDF LUT/低端近似由引擎 define(全局维度)切换,不新增材质局部 option。

## 9. 状态与产出记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

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
