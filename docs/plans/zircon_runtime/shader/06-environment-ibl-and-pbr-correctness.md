---
related_code:
  - zircon_runtime/src/core/framework/render/environment/skybox.rs
  - zircon_runtime/src/graphics/shader/wgsl/zr_environment.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/from_frame.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_texture/mod.rs
  - dev/cmft/src/cmft/cubemapfilter.cpp
  - dev/cmft/src/cmft/image.cpp
  - dev/cmft/src/cmft/cubemaputils.h
  - dev/cmftStudio/src/shaders/fs_mesh.shdr
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ReflectionEnvironmentCapture.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ReflectionEnvironmentRealTimeCapture.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ReflectionEnvironmentDiffuseIrradiance.cpp
  - dev/UnrealEngine/Engine/Shaders/Private/ReflectionEnvironmentShaders.usf
  - dev/UnrealEngine/Engine/Shaders/Private/ReflectionEnvironmentShared.ush
  - dev/UnrealEngine/Engine/Shaders/Private/MonteCarlo.ush
  - dev/UnrealEngine/Engine/Shaders/Private/BRDF.ush
  - dev/UnrealEngine/Engine/Shaders/Private/SkyLightingShared.ush
---

# 06 环境 IBL 资产链与 PBR 正确性验收

本子计划是 shader 计划集对 render 计划 11(环境光照)的**算法定稿与验收闭环补充**:计划 11 的"工程落地细化"仍是文件落点/契约类型/帧时序的实施权威;本计划定稿其中 IBL 相关 compute 的**具体算法**(equirect→cube、GGX 预滤波、SH9、BRDF LUT)、`zr_environment.wgsl` v2 的采样端契约,以及"8×8 金属度/光滑度材质球矩阵正确"的可量化验收标准。跨计划扩展按 shader index §4 惯例在计划 11/13 状态表登记镜像条目。

## 1. 问题定位:当前渲染为什么全是马赛克

2026-07-04 的 SH04/Plan 11 状态行(index §4.1)所验证的"sampled environment"路径存在三个结构性缺陷,与截图观察(天空盒马赛克、mip 无模糊、PBR 矩阵不可信)一一对应:

1. **没有环境贴图,只有 128 个数**。`SkyboxSettings.sampled_equirectangular` 是 16×8=128 个 vec4 的 CPU 数组(`skybox.rs` 的 `SAMPLED_EQUIRECT_ENVIRONMENT_WIDTH/HEIGHT`),经 scene uniform 上传;`zr_environment.wgsl` 的 `zr_environment_sampled_equirect_color` 做**最近邻**索引。天空盒的物理分辨率就是 16×8——马赛克是数据形态的必然,不是滤波 bug。
2. **没有预滤波,粗糙度是假的**。`zr_environment_reflection_color` 用 `mix(sky(reflect_dir), sky(normal), roughness)` 两方向插值冒充模糊;不存在 GGX 卷积 mip 链,"mipmap 全是马赛克"是因为根本没有生成过预滤波 mip。
3. **没有 split-sum,能量是拍的**。`zr_environment_pbr_indirect` 用 `0.35`/`0.20`/`1.0 - roughness*0.65` 经验常数近似 diffuse/specular 权重;无 SH irradiance、无环境 BRDF 项,金属度/光滑度扫描不可能呈现正确梯度。

前置缺口:`gpu_texture` 无 `Cube`/`D2Array` 维度,`CubemapAsset`(计划 13 TX-M3)未开工。

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

**取舍**:方向/立体角/双线性公式与"末级 mip 平均"原样采纳;radiance 的 Blinn-Phong power 滤波与 Warp fixup **不采纳**——前者被 UE 的 GGX 重要性采样取代(与实时 BRDF 同分布),后者在 wgpu(Vulkan/D3D12/Metal 均保证 seamless cubemap filtering)上无必要,只保留"低 mip 接缝亮度差"验收断言防回归。cmft 线性 lod 映射不采纳(换 mip 数会漂移),沿用计划 11 已定稿的 UE 对数映射。SH 取 3 阶 9 系数(irradiance 是低频信号,UE 同选择),A_l 与立体角权重照抄。

### 2.2 UE5:捕获管线、split-sum 与实时环境

| 机制 | 结论 | 出处 |
|------|------|------|
| 捕获流程 | 逐面渲染 → mip0 premultiply alpha → `CreateCubeMips` 降采样链 → `ComputeDiffuseIrradiance`(SH9)→ `FilterCubeMap`(GGX) → 拷入 `TextureCubeArray` | `ReflectionEnvironmentCapture.cpp:490` |
| 分辨率/格式 | 128 或 256,`PF_FloatRGBA`(FP16);mip 数 = `CeilLogTwo(size)+1` | `ReflectionEnvironmentCapture.cpp:274` |
| roughness↔mip | `LevelFrom1x1 = 1.0 - 1.2·log2(max(r, 0.001))`,`mip = maxMip - 1 - LevelFrom1x1`;逆映射 `r = exp2((1.0 - LevelFrom1x1)/1.2)`——mip↔roughness 与总 mip 数无关 | `ReflectionEnvironmentShared.ush:16–39` |
| GGX 重要性采样 | Hammersley 序列,`E.y *= 0.995` 防掠射;`CosTheta = √((1-E.y)/(1+(a²-1)E.y))`,`a² = r⁴`(α=r² 的平方);`L = 2(H·V)H - V` | `MonteCarlo.ush:347–363`,`ReflectionEnvironmentShaders.usf:612–648` |
| **filtered importance sampling** | 按样本 PDF 选源 mip:`PDF = D·NoH/4`,`Ω_s = 1/(N·PDF)`,`Ω_p = 4π/(6·size²)·2`,`srcMip = 0.5·log2(Ω_s/Ω_p)`;NoL 加权累加——这是 32/64 样本就能干净的关键 | `ReflectionEnvironmentShaders.usf:612–648` |
| 样本数 | 桌面 `r<0.1 ? 32 : 64`;参考路径 1024 | `ReflectionEnvironmentShaders.usf:561–580` |
| split-sum LUT | `PreIntegratedGF` 128×128,UV=(NoV, roughness),RG 双通道;`GF = F0·A + saturate(50·F0.g)·B`(F90 项) | `BRDF.ush:559–573` |
| 无 LUT 近似 | Lazarov 多项式 `EnvBRDFApprox`(常数 c0/c1 见出处),移动端用 | `BRDF.ush:583–604` |
| SH9 计算 | 实时路径:8×8=64 线程均匀球采样 mip2,树形归约,输出 8×float4(预乘 diffuse 卷积系数);离线路径:32³ 中间 cubemap 三 pass 归约 | `ReflectionEnvironmentShaders.usf:759–892`,`ReflectionEnvironmentDiffuseIrradiance.cpp:88–230` |
| SH9 重建 | `GetSkySHDiffuse`:9 点积无分支(L0+L1 一组 dot4、L2 双线性项、L2 m=±2 项),`max(0, …)` 截断 | `SkyLightingShared.ush:84–103` |
| 探针混合 | light grid 剔除后按覆盖半径大→小 under-operator 累加:`rgb += s.rgb·a_acc`,`a_acc *= 1-s.a`,`a<0.001` 早退;剩余 alpha 给 skylight | `ReflectionEnvironmentComposite.ush:12–188` |
| parallax | box:局部空间 AABB 最远交点,`DistanceAlpha = 1-smoothstep(0, 0.7·transition, boxDist)`;sphere:二次方程远交点 | `ReflectionEnvironmentShared.ush:136–201` |
| SSR/RT/Lumen 组合 | 高频源(SSR/Lumen/RT)先写,`Color.a = 1 - hit.a`,探针吃剩余 alpha,skylight 兜底,最后统一乘 `EnvBRDF` | `ReflectionEnvironmentPixelShader.usf:113–281` |
| 实时天空捕获 | 时间切片:每帧渲染 N 面(云默认 2 面/帧)→ `FDownsampleCubeFaceCS`(8×8)→ `FConvolveSpecularFaceCS` 分帧卷积 → SH 一次性 64 线程 | `ReflectionEnvironmentRealTimeCapture.cpp:340, 84–153` |

**取舍**:mip 映射常数(1.0/1.2)、GGX 采样式、FIS、样本数、LUT 尺寸/公式、SH9 归约与重建、SSR→探针→天空的 alpha under 组合契约全部原样采纳;探针混合沿用计划 11 已定稿的 top-2 截断(URP 量级)而非 UE 的 grid 全列表;light grid 剔除、Lumen 本体不在本计划范围。

## 3. 目标数据链(总览)

```
.hdr/.exr equirect(≥1K,建议 2K)
   │ 导入:计划 13 TX-M3 .zcube(RGBA16F,face-major,含全 mip)——就绪前由 EC-M1 的运行期路径顶替
   ▼
[env.equirect_to_cube]   compute:双线性重投影 → 源 cubemap(面尺寸 = min(pow2((H+1)/2), 1024))
   ▼                                + 常规 box mip 链(供 skybox 缩小采样,非 GGX 链)
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
- 面尺寸:`clamp(next_pow2((H+1)/2), 64, 1024)`。源 cubemap 生成后逐级 2×2 box 降采样出常规 mip 链(skybox 缩小与 FIS 的源 mip 都依赖它)。

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
- 命中来源 1/2 时运行期环境 compute dispatch 数为 0(验收断言),与计划 08 MS-M4"二次启动 miss=0"同构。
- **外部预滤波 cubemap 的处理**:cmft/cmftStudio 产出的含 mip DDS/KTX 导入时,其 mip 链只作为源 cubemap 的降采样链消费,PMREM 一律引擎内重新生成——cmft 是 Blinn-Phong power 滤波 + 线性 lod 映射(`fs_mesh.shdr:195`),与引擎 GGX + 对数映射不兼容,必须保持 roughness↔mip 映射唯一真理;`.zcube` 不设 `prefiltered` 声明位。
- 开发者使用说明:导入 `.hdr` → importer 自动产 `.zcube` + 伴生 PMREM/SH(来源 1);编辑器改 procedural 参数 → 运行期切片重烘 + 回写(来源 2/EC-M4);发布走 staged build 全量预烘,产品运行期零环境烘焙成本。

### 4.8 IEM(irradiance cubemap)替代 SH 的 diffuse 消费选项

cmftStudio 的 diffuse 消费端是 32³ irradiance cubemap 直采(`fs_mesh.shdr:210` 的 `textureCube(u_texIem, n)`),不是 SH 求值。引擎提供两种 diffuse 消费路径,由**引擎 define** `ZR_ENV_DIFFUSE_IEM`(全局维度,走计划 08 defines,不是材质 option,不增材质变体)切换:

| 路径 | 求值 | 优势 | 默认 |
|---|---|---|---|
| SH9 | `zr_env_sh9_eval(n)` 9 项多项式 | 零纹理带宽;与 ambient/probe grid 契约同一表示(计划 11) | 是 |
| IEM | `textureSample(zr_env_irradiance, zr_env_sampler, n)` 1 fetch | 无 3 阶截断 ringing;强方向性环境下更准;带宽换 ALU | 质量档选项 |

- 生成:32³ RGBA16F cubemap,余弦加权重要性采样卷积(复用 §4.2 compute 骨架,kernel 换 cosine lobe)。**不采用** cmft 的"SH 重建回 cubemap"式(`imageIrradianceFilterSh`)作为主路径——直接卷积无阶数截断,精度更高;SH 重建式仅作 Fast 档快捷路径。随 §4.7 三来源链持久化,同一 `IblBakeKey`。
- SH9 无条件仍产出:它是 ambient 三模式(计划 11 EL-M4)与 probe grid(EL-M3)契约的权威表示,IEM 只是 standard PBR diffuse 项的可选消费端,不替代契约层。
- 绑定:group1 环境段新增一槽 `zr_env_irradiance`(`texture_cube<f32>`),确切编号随 BRDF LUT 槽一并在计划 11 登记。
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

执行顺序 EC-M1 → EC-M2 → EC-M3 → EC-M4;EC-M1/M2 各自切片内硬切换删除被取代路径。

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
| `render_env_equirect_to_cube_golden_directions` | ±X/±Y/±Z 面中心与四角 texel 的方向→UV 与 cmft 公式闭式对拍 | `ibl_prefilter.rs` 或 equirect owner |
| `render_env_prefilter_cpu_gpu_match_16` | 16³ 源、全 mip CPU 参考(同 FIS/同序列)与 compute 输出逐 texel 容差对拍 | `ibl_prefilter.rs` |
| `render_env_prefilter_constant_env_is_identity` | 常色环境预滤波后所有 mip 仍为常色(权重归一正确性) | `ibl_prefilter.rs` |
| `render_env_sh9_matches_cpu_reference` | compute SH9 与 CPU 立体角参考逐系数容差对拍;常色→仅 band0 | `ibl_prefilter.rs` |
| `render_env_brdf_lut_corner_values` | (NoV→1, r→0) A→1,B→0;(任意, r→1) A+B < 1(能量守恒上界) | `env_brdf_lut.rs` |
| `render_env_mip_from_roughness_roundtrip` | 已在计划 11 清单,常数 1.0/1.2 双端一致 | 计划 11 |
| `render_env_seam_luminance_below_threshold` | §6 判据 6 | `render_product_environment.rs` |
| `render_product_environment_pbr_matrix_quantitative` | §6 判据 1–5 | `render_product_environment.rs` |
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

| 日期 | 里程碑/切片 | 状态 | 产出 | 验证与证据 | 后续 |
|------|-------------|------|------|------------|------|
| 2026-07-05 | EC-M1a sampled HDRI 高分辨率 mip 链 | 中间实现,待最终验证 | 新增 `equirect_samples.rs`: Poly Haven HDRI 从 16x8 table 提升为 128x64 base + 8 mip storage buffer;scene group0 binding1 上传环境样本,scene uniform 删除内嵌 `environment_samples`;`zr_environment.wgsl` 与 skybox shader 改为 bilinear + lod 采样,roughness→mip 采用 UE `ROUGHEST_MIP=1.0`/`ROUGHNESS_MIP_SCALE=1.2`,specular 使用 Lazarov EnvBRDF 近似;真实 HDRI export 改写到 `runtime_shader_pbr_real_hdri_lakes_pmrem_reflection_20260705.png`,不覆盖 2026-07-04 失败图。 | 静态扫描:旧 `16x8` 常量、旧 PNG 文件名、uniform 内嵌 sample table 残留为 0;`git diff --check` 通过(仅 CRLF 提示);`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-hdri-pmrem-core-check-0705 --message-format short --color never` 通过(9m46s);focused unit test 在并发默认特性链接负载下 304s 超时,不计通过证据。 | 继续跑默认特性 `cargo check`、shader tests、ignored export 并人工查看截图;下一切片按本计划 EC-M1 硬切到真正 GPU `texture_cube`,删除 sampled equirect storage-buffer 过渡路径,再进入 EC-M2 GGX FIS/SH9/BRDF LUT。 |
| 2026-07-05 | 计划建立 | 已完成源码研究与算法定稿 | cmft/cmftStudio 与 UE5 精读结论(§2)、数据链与算法定稿(§3/§4,含离线烘焙三来源持久化 §4.7 与 IEM 替代 SH 选项 §4.8)、量化验收(§6) | 精读出处见 §2 各表;现状缺陷定位:16×8 sample table(`skybox.rs`)、最近邻采样与两方向 lerp 伪粗糙度(`zr_environment.wgsl`)、无 Cube 纹理维度 | EC-M1 起实施 |
