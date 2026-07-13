---
related_code:
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/graphics/backend/render_backend/request_device.rs
  - zircon_runtime/src/rhi_wgpu/capabilities.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_texture.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_texture/gpu_texture_resource_from_asset.rs
  - dev/bevy/crates/bevy_pbr/src/material_bind_groups.rs
  - dev/bevy/crates/bevy_render/src/render_resource/bindless.rs
  - dev/bevy/crates/bevy_render/src/batching/gpu_preprocessing.rs
  - dev/bevy/crates/bevy_render/src/render_phase/draw_state.rs
  - dev/bevy/crates/bevy_core_pipeline/src/mip_generation/mod.rs
  - dev/bevy/crates/bevy_core_pipeline/src/mip_generation/downsample.wgsl
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/GPUSort.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Streaming/StreamingManagerTexture.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Streaming/AsyncTextureStreaming.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ShadowSetup.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ScenePrivate.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/DeferredShadingRenderer.cpp
  - dev/UnrealEngine/Engine/Shaders/Private/MaterialTemplate.ush
  - dev/Graphics/Packages/com.unity.render-pipelines.core/ShaderLibrary/CommonMaterial.hlsl
plan_sources:
  - docs/plans/zircon_runtime/render/index.md
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - .codex/plans/Runtime 渲染风险清单与 RenderDoc 调试支持计划.md
  - .codex/plans/ZirconEngine Bevy-Level Rendering Completion Plan.md
---

# 计划 19:GPU 能力利用与带宽优化扩展

## 目标

在计划 01–17 的骨架/能力/横切三层之上,补齐"硬件能力利用 + 带宽/缓存"两族 dev/ 参考引擎已验证、wgpu 当前可实现、而既有计划未覆盖的渲染优化机制。完成后:

1. 设备能力申请与 `RenderCapabilitySummary` 的 gate 字段一一对应:summary 里声明的能力(binding array 系)在设备创建期真实请求,新增 subgroup / pipeline statistics 能力位;"声明了却用不上"的能力位清零。
2. 提交热路径深化:bindless 材质纹理把 group2 材质 bind group 切换成本归零;`multi_draw_indexed_indirect_count` 让 GPU 剔除直接决定 draw 数,杜绝零实例 draw 空转。
3. 带宽大头收口:常规纹理 mip 流送(距离驱动)、半分辨率透明/粒子合成、静态阴影缓存三件套,把显存常驻量与每帧带宽从"全量"压到"按需"。
4. compute 加速与质量增益:subgroup 归约双路径、GPU 粒子深度排序、specular AA(Toksvig 导入烘焙 + 几何项着色端)落地。
5. 每个机制都带能力 gate 与回退路径:能力缺失或 feature 关闭时回到现行路径,渲染产物正确性等价,外部接口不变。

## 现状与差距

基于实读代码的现状盘点:

- **能力申请与 gate 脱节**:`graphics/backend/render_backend/request_device.rs` 只请求 `MULTI_DRAW_INDIRECT_COUNT` 与 `INDIRECT_FIRST_INSTANCE` 两个 feature;而 `rhi_wgpu/capabilities.rs` 按 adapter features 填充 `supports_buffer_binding_array` / `supports_texture_binding_array` / `supports_non_uniform_resource_indexing` / `supports_partially_bound_binding_array` 四个 gate 字段——这些 feature 从未在 `DeviceDescriptor.required_features` 中请求,gate 即使为 true 也没有任何 wgpu 路径可用。本计划先纠正"探测了不请求"的断链。
- **`RenderCapabilitySummary` 字段实勘**(`core/framework/render/backend_types.rs`):已有 indirect 三件套(`supports_indirect_draw`/`supports_multi_draw_indirect`/`supports_indirect_first_instance`,组合判据 `gpu_driven_submission_supported()`)、binding array 四件套、压缩纹理探测(`gpu_texture_resource_from_asset.rs` 读 BC/ETC2/ASTC features)、async compute/copy、sparse texture、pipeline cache。**没有** subgroup、timestamp/pipeline statistics 任何能力位;`RenderCapabilityKind` 17 个枚举值中无本计划新增机制的档位。
- **多 draw 提交停在固定 draw 数**:计划 03 GS-M4 的 `IndirectDrawBatcher` 走 `multi_draw_indexed_indirect`(CPU 决定 draw 数),计划 04 剔除 compute 只改写 args 内 `instance_count`(被剔除 draw 以零实例空转);`MULTI_DRAW_INDIRECT_COUNT` 已请求但 count buffer 路径(GPU 决定 draw 数)无人消费。
- **纹理无流送概念**:`resource_streamer/resource_streamer_ensure_texture.rs` 的 `ensure_texture` 按 `ResourceId` + revision 全量加载整条 mip 链,`PreparedTexture` 无常驻 mip 区间概念;计划 13 只规划了 SVT(巨型纹理特例)与"退化为最高常驻 mip 链截断"一句,常规纹理的距离驱动 mip 流送无承接计划。
- **阴影逐帧全量重画**:计划 05 的 `ShadowAtlas` 是槽位分配器,无缓存语义;静态场景的点/聚光阴影每帧重画全部 caster,带宽与 draw 全浪费。
- **透明全分辨率直绘**:粒子/透明在主分辨率 HDR target 上直绘,overdraw 高的粒子场景带宽不可控;计划 07 只有 DoF/exposure 的半分辨率中间 RT,无透明半分辨率段。
- **GPU 排序空缺**:计划 12 显式取舍"不做 per-view GPU 粒子排序,V1 透明排序停在 emitter 粒度",粒子级深度排序与未来 OIT 无底座。
- **compute 归约全标量**:HZB(计划 04)、exposure histogram(计划 07)、mip 生成(计划 13)的 reduce 全走 workgroup shared memory 标量路径,无 subgroup 加速档。
- **观测无图元维度**:计划 17 PF-M1 只规划 `TIMESTAMP_QUERY`;`RenderStats` 百余字段无 GPU 图元/顶点/片元调用计数,`PIPELINE_STATISTICS_QUERY` 未触及。
- **specular AA 空缺**:计划 08 光照模型与计划 13 normal 管线均未覆盖法线方差→roughness 的导入期烘焙与着色端几何项,高频法线 + 低 roughness 的高光闪烁无解。

## 与既有计划的边界

本计划是既有计划的"消费者与深化者",不开任何旁路;逐机制的契约消费关系:

| 机制 | 消费的既有契约 | 接入方式 | 边界说明 |
|------|---------------|---------|---------|
| A. bindless 材质纹理 | 计划 03 `GpuScene`(材质槽索引进 instance/material payload)、计划 08 `ShaderVariantKey`(bindless 变体位)与模板拼接、计划 02 `MeshDrawCommand`(命令 ABI 不变) | 能力 gate + 变体位 | group2 槽位语义不变(index §8 第 1 条),只是 bind group 构造方式与 WGSL 索引方式切换 |
| B. indirect count 提交 | 计划 03 `IndirectDrawBatcher`(扩展 count buffer 槽)、计划 04 剔除 compute(升级为压实写 args)、计划 01 `RgBufferHandle`(count buffer 经 graph 声明) | 能力 gate 升档 | 不改 `MeshDrawCommand` ABI;GS-M4 的 CPU batch plan 仍是输入 |
| C. subgroup 归约 | 计划 04 `HzbBuilder`、计划 07 exposure histogram、计划 13 `MipGenPass`、计划 16 `ComputePassDescriptor`(compute 注册形态) | 能力 gate + WGSL 双路径 | 只改 kernel 内部归约手法,绑定 schema 与 dispatch 形状不变 |
| D. pipeline statistics | 计划 17 `GpuPassTimer` / `RenderFrameProfile`(pass 条目扩展)、计划 16 `GpuReadbackQueue`(回读收口,与 timestamp 同期迁移) | 能力 gate + 观测扩展 | 只加观测字段,不进任何断言(时间/硬件相关计数只观测,对齐 index §8 第 6 条) |
| E. 静态阴影缓存 | 计划 05 `ShadowAtlas`(槽位增加 cache 状态)、计划 04 relevance(static/dynamic caster 分类)、计划 17 PF-M3 预算(超预算先弃缓存) | feature/profile gate | 级联划分、texel snapping、PCF 全部归 05;本计划只管"槽位内容何时可以不重画" |
| F. 半分辨率透明 | 计划 01 `TransientResourcePool`(半分辨率 color/depth)、计划 09 `sort_key` 与 Transparent queue 段(位段不动)、计划 07 后处理链(合成 pass 插槽)、计划 12 粒子(默认进半分辨率段) | profile gate | sort_key 位段唯一归 09,本计划仅按材质标记切分提交段 |
| G. mip 流送 | 计划 13 `TextureMetadata`(mip 策略与 streaming 标记)、`resource_streamer`(现 revision 机制为改造基础)、计划 17 PF-M3 预算账本(显存上限与 mip bias 降级互通) | 软件机制 + profile gate | 与计划 13 SVT 的分工:mip 流送是**所有常规纹理的默认路径**,SVT 是巨型纹理特例;二者共享预算账本,互不替代 |
| H. GPU 排序 | 计划 16 `ComputePassDescriptor`(sort pass 注册形态)、计划 12 `ParticleSimOutput`(alive list 为排序输入)、计划 03 `IndirectDrawBatcher`(排序后间接提交不变) | feature gate | 排序键语义仍由计划 09 sort_key/计划 12 深度键定义,本计划只提供"GPU 上把它排好"的设施 |
| I. specular AA | 计划 13 导入器(TX-M2 normal mip 管线扩展烘 roughness)、计划 08 `ShaderVariantKey`(质量档变体位)与 `zr_` include 体系 | 导入期 + 变体位 | 不新增材质参数语义,roughness 合成进既有槽位 |

全部机制经 RenderFeature descriptor 或能力 gate 接入(index §6 第 3/4 条);pass 一律经 graph 声明;只消费 `RenderFrameExtract`;facade 固定 `zircon_runtime::core::framework::render`,framework 契约层零 wgpu(能力以 `RenderCapabilitySummary` 中立字段表达)。

## 参考代码

| 文件 | 应重点阅读 |
|------|-----------|
| `dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/GPUSort.cpp` + `GPUSortManager.cpp`(头在 `Engine/Public/`) | bitonic GPU 排序的 pass 组织(逐阶段 dispatch 链)、键值对缓冲乒乓;GPUSortManager 的多客户批量提交与时机(模拟后/渲染前) |
| `dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ShadowSetup.cpp` + `ScenePrivate.h` | `FCachedShadowMapData`(ScenePrivate.h:2685)与 `CachedShadowMaps` per-light 缓存表;cached whole scene shadow 的失效判据(光源参数变化、caster 集变化)与"静态缓存 + 动态增量"双 pass 划分 |
| `dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/DeferredShadingRenderer.cpp`(+ `MobileSeparateTranslucencyPass.cpp`) | SeparateTranslucency/DownsampledTranslucency:半分辨率透明段的判定(屏占比/配置)、独立 RT 渲染与深度感知合成回主链的时序 |
| `dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Streaming/StreamingManagerTexture.cpp` + `AsyncTextureStreaming.cpp` | 纹理流送总控:per-texture wanted mips 计算(距离/屏幕尺寸启发式)、预算内优先级分配、异步加载任务切分与 mip 升降级状态机 |
| `dev/Graphics/Packages/com.unity.render-pipelines.core/ShaderLibrary/CommonMaterial.hlsl` | specular AA 权威样板:`TextureNormalVariance`/`TextureNormalFiltering`(:241/:263,Toksvig 纹理项)与 `GeometricNormalFiltering`(:218,Kaplanyan 屏幕空间几何项)的完整公式 |
| `dev/UnrealEngine/Engine/Shaders/Private/MaterialTemplate.ush` | `NormalCurvatureToRoughness`:UE 侧几何 specular AA 的等价实现,与 Unity 公式互证 |

**Rust/wgpu 落地参照(防凭空实现)**:

| 文件 | 对应本计划机制 | 应重点阅读 |
|------|---------------|-----------|
| `dev/bevy/crates/bevy_pbr/src/material_bind_groups.rs` | A. bindless 材质槽位分配 | `MaterialBindGroupAllocator` 的 bindless/非 bindless 双形态;slab 式槽位分配与回收、纹理去重;fallback image 填充空槽(`PARTIALLY_BOUND_BINDING_ARRAY` 不可用时的兜底手法) |
| `dev/bevy/crates/bevy_render/src/render_resource/bindless.rs` + `bindless.wgsl` | A. bindless 能力判定与 WGSL 索引 ABI | bindless 启用判据(features + limits 联合检查、平台黑名单思路)、`BindlessIndex` 槽位表;WGSL 侧 binding_array 声明与 non-uniform 索引包装 |
| `dev/bevy/crates/bevy_render/src/batching/gpu_preprocessing.rs` | B. indirect count 三档降级 | `GpuPreprocessingMode`(Off/PreprocessingOnly/Culling)按能力分档;`IndirectParametersBuffers` 的 cpu/gpu metadata 与 count buffer(batch_sets)组织 |
| `dev/bevy/crates/bevy_render/src/render_phase/draw_state.rs` | B. count 提交调用形态 | `multi_draw_indexed_indirect_count` 的调用点:args buffer + count buffer + max_count 三参组合与回落分支 |
| `dev/bevy/crates/bevy_core_pipeline/src/mip_generation/mod.rs`(:818)+ `downsample.wgsl` | C. subgroup 能力 gate 与双路径 kernel | `Features::SUBGROUP` → `SUBGROUP_SUPPORT` shader def 的注入手法;downsample.wgsl 内 `#ifdef SUBGROUP_SUPPORT` 子群归约与 `#else` workgroup shared memory 标量路径的完整双实现 |
| `dev/bevy/crates/bevy_pbr/src/light_probe/generate.rs`(:334) | C. subgroup 第二实例 | 探针生成 compute 的同款 gate 注入,佐证"def 注入 + 双路径 include"是 bevy 通行手法 |
| `dev/bevy/crates/bevy_render/src/render_resource/gpu_array_buffer.rs` | A/B. 能力回落组织 | "同一 ABI、按 limits 切换实现"的范式(storage vs batched uniform),本计划所有 gate 回退遵循同型 |

无 Rust 同类参照、按 index §8 第 8 条对拍测试先行的机制:**E. 静态阴影缓存**(bevy/Fyrox 均逐帧全量重画,UE `FCachedShadowMapData` 为唯一样板)、**F. 半分辨率透明**(bevy 无 separate translucency,UE 为唯一样板)、**G. mip 流送**(bevy 无纹理流送,UE StreamingManager 为唯一样板;Zircon 侧 `resource_streamer` revision 机制是落地基座)、**H. GPU 排序**(bevy 无通用 GPU sort,UE GPUSort 为唯一样板)、**I. specular AA 的 WGSL 侧**(bevy 无 specular AA,公式以 Unity CommonMaterial.hlsl 为准)。**D. pipeline statistics** 无任何引擎级参照(bevy 仅用 timestamp,learn-wgpu-zh 无此主题),以 wgpu 29.0.1 API 为据(`Features::PIPELINE_STATISTICS_QUERY`、`QueryType::PipelineStatistics(PipelineStatisticsTypes)`),实现组织对照计划 17 `GpuPassTimer` 的 timestamp 同型路径。

## 目标架构

归属:能力契约扩展在 `core/framework/render/backend_types.rs`;设备申请在 `graphics/backend/render_backend/request_device.rs` 与 `rhi_wgpu/capabilities.rs`;各机制实现落在所属既有计划的模块目录(见工程落地细化)。每机制三要素:能力 gate / 数据契约 / 回退路径——**回退路径是本计划的灵魂,能力缺失时必须存在正确性等价路径**。

机制一览(详述见各小节):

| # | 机制 | 能力 gate | 回退路径 | 类别 |
|---|------|----------|---------|------|
| A | bindless 材质纹理 | `bindless_material_supported()`(binding array 三件套) | per-material group2 bind group(现路径),产物逐像素一致 | 硬件能力 |
| B | indirect count 提交 | `supports_multi_draw_indirect` + `gpu_driven_submission_supported()` | GS-M4 固定 draw 数 multi-draw → 逐 draw | 硬件能力 |
| C | subgroup 归约 | `supports_subgroup`(新增) | workgroup shared memory 标量归约,min/max bit-exact | 硬件能力 |
| D | pipeline statistics | `supports_pipeline_statistics_query`(新增) | profile 字段为 `None`,CPU 计数照旧 | 硬件能力 |
| E | 静态阴影缓存 | 无硬件 gate;profile + 预算 | 逐帧全量重画(现行为) | 带宽/缓存 |
| F | 半分辨率透明 | 无硬件 gate;profile 质量档 | 全分辨率 Transparent 段原位提交(现行为) | 带宽 |
| G | mip 流送 | 软件机制;streaming 标记 + 预算 | 整链常驻(现行为);最低常驻 mip 永不驱逐 | 带宽/显存 |
| H | GPU 排序 | `supports_storage_buffers`(基线) | emitter 粒度 CPU 排序(计划 12 V1 行为) | compute |
| I | specular AA | 无硬件 gate;变体位 | 变体位关闭 = 现状,逐像素一致 | 质量 |


### A. bindless 材质纹理(binding_array)

- **能力 gate**:`bindless_material_supported()` 组合判据(对齐 `gpu_driven_submission_supported()` 形态)= `supports_texture_binding_array && supports_partially_bound_binding_array && supports_non_uniform_resource_indexing`,另查 limits 的 binding array 容量上限决定槽位表大小;`request_device.rs` 据 adapter 探测请求 `TEXTURE_BINDING_ARRAY`、`PARTIALLY_BOUND_BINDING_ARRAY`、`SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING`(修复"探测不请求"断链)。
- **数据契约**:group2 材质级(index §8 第 1 条槽位语义不变)在 bindless 档改为"一个全局 `binding_array<texture_2d<f32>>` 槽位表 + 材质 uniform 内 u32 槽位索引";`BindlessMaterialSlab` 负责槽位分配/回收/纹理去重,空槽以 fallback 纹理填充(bevy 同手法);槽位索引随材质数据进计划 03 `GpuScene` 的 material payload。计划 08 `ShaderVariantKey` 增加 bindless 变体位,WGSL 经模板条件拼接 `zr_bindless_material.wgsl`(non-uniform 索引包装函数)。
- **回退路径**:gate 为 false 时维持现行 per-material group2 bind group,同一材质数据源、同一 `MeshDrawCommand` ABI,仅 bind group 构造与采样代码不同;渲染产物逐像素一致(对拍测试)。

### B. multi_draw_indirect_count 提交升档

- **能力 gate**:`supports_multi_draw_indirect`(已对接 `MULTI_DRAW_INDIRECT_COUNT` 且已请求)+ `gpu_driven_submission_supported()`;在计划 03 GS-M4 三档之上加第四档,对照 bevy `GpuPreprocessingMode` 的分档思路。
- **数据契约**:计划 04 剔除 compute 由"零写 instance_count"升级为"压实写":通过 subgroup ballot 或原子计数把存活 draw 的 args 紧凑写入输出 args buffer,并原子累加 draw count 到 count buffer(u32 @ offset 0);count buffer 经计划 01 `RgBufferHandle` 声明、生命周期归 graph;提交端 `IndirectDrawBatcher` 的 eligible batch 改调 `multi_draw_indexed_indirect_count(args, count_buffer, max_count)`,`max_count` = CPU batch plan 的 draw 上限。
- **回退路径**:gate 缺失或剔除 compute 关闭时回到 GS-M4 现行档(固定 draw 数 + 零实例空转),再回落逐 draw;三档产物一致(空转 draw 不产生像素)。

### C. subgroup 归约双路径

- **能力 gate**:新增 `supports_subgroup` 字段(wgpu `Features::SUBGROUP`,按 adapter 探测请求);`RenderCapabilityKind` 增 `SubgroupOps`(Experimental 类)。
- **数据契约**:新增 `zr_reduce.wgsl` include(只暴露函数,index §8 第 3 条):`zr_reduce_min/max/add` 双实现——subgroup 路径(`subgroupMin`/`subgroupMax`/`subgroupAdd` + workgroup 级二段归约)与标量路径(workgroup shared array 折半);计划 08 模板按 `SUBGROUP_SUPPORT` def 条件拼接(bevy mip_generation 同手法)。消费方:计划 04 `HzbBuilder` 深度 min/max reduce、计划 07 exposure histogram 累加、计划 13 `MipGenPass`、机制 B 的剔除压实(ballot 档)。
- **回退路径**:标量路径数值上 bit-exact(min/max)或容差内(浮点加法,对拍容差进测试);kernel 绑定 schema 与 dispatch 形状两路径完全相同,切换零接口影响。

### D. pipeline statistics query

- **能力 gate**:新增 `supports_pipeline_statistics_query` 字段(wgpu `Features::PIPELINE_STATISTICS_QUERY`,按 adapter 探测请求;主要在 Vulkan/DX12 后端可用)。
- **数据契约**:复用计划 17 `GpuPassTimer` 的 QuerySet 池与 N 帧 in-flight 槽轮转,按 graph pass 增挂 `QueryType::PipelineStatistics`(vertex/clipper/fragment invocations 与 primitives out);解析结果进 `RenderFrameProfile` pass 条目的新增 `Option<PipelineStatisticsSample>` 字段;计划 16 `GpuReadbackQueue` 落地后与 timestamp 小环同批迁移(硬切换)。
- **回退路径**:gate 为 false 时字段为 `None`,CPU 侧确定性计数(draw 数/实例数)照旧;该类硬件计数永不进 `render_perf_*` 断言(对齐 index §8 第 6 条"时间类只观测")。

### E. 静态阴影缓存(cached shadow maps)

- **能力 gate**:无硬件 gate;profile feature 开关 + 计划 17 PF-M3 显存预算(降级阶梯中"弃阴影缓存"排在 render scale 降档之前,作为第 0 级)。
- **数据契约**:`ShadowAtlas`(计划 05)槽位增加 `ShadowCacheEntry`(缓存深度纹理引用 + 失效键:光源参数 hash、静态 caster 集 revision、atlas 槽位变更代际);计划 04 relevance 提供 caster 的 static/dynamic 分类。命中时:点/聚光阴影 pass 改为"copy 缓存深度到槽位 → 仅绘制 dynamic caster";未命中时全量重画并回填缓存。方向光 CSM 仅远级联参与缓存(近级联随相机每帧变化,不缓存),远级联按 N 帧轮换更新。
- **回退路径**:feature 关闭或预算不足时逐帧全量重画(现行为);缓存命中与全量重画的深度产物一致(静态场景对拍断言 bit-exact,动态 caster 场景对拍合成结果)。

### F. 半分辨率透明/粒子(separate translucency)

- **能力 gate**:无硬件 gate;profile 质量档开关(默认关,带宽受限档开)。
- **数据契约**:Transparent queue 段(计划 09 数值段不动)内按材质/渲染器标记(粒子默认标记)切出 half-res 子段;graph 声明半分辨率 color(RGBA16Float)+ 深度(主深度 downsample,max 取样保守)两张瞬态纹理(计划 01 `TransientResourcePool`);子段 draw 在半分辨率 RT 上执行后,由深度感知上采样合成 pass(双边权重,对照 UE 同型)在计划 07 链定稿的透明合成插槽写回主 HDR target。
- **回退路径**:feature 关闭时该子段回到全分辨率 Transparent 段原位提交,产物即现行为。注意:这是有损带宽优化,开启时与全分辨率产物存在预期差异——`render_product_*` 对拍用专项半分辨率基线,不与全分辨率基线互比;回退路径才是正确性等价路径。

### G. 纹理 mip 流送(distance-driven streaming)

- **能力 gate**:软件机制,无硬件 gate;`TextureMetadata`(计划 13)增加 streaming 标记(默认开,UI/小纹理/无 mip 资产豁免);预算上限归计划 17 PF-M3(与全局 mip bias 降级互通:先流送驱逐,后全局 bias)。
- **数据契约**:`resource_streamer` 的 `PreparedTexture` 增加 resident mip 区间(`resident_mip_range`);extract 携带可见渲染器的纹理引用 + 包围球距离/屏幕占比,streamer 帧首计算 per-texture wanted mip(UE wanted mips 启发式),按预算与优先级(屏占比降序)生成升/降级任务;升级 = 新建含更高 mip 的 texture → copy 既有 mip(`copy_texture_to_texture`)→ 拉取缺失 mip 上传 → 原子换绑(bindless 档只改槽位表一个索引,非 bindless 档重建材质 bind group,复用既有 revision 失效机制);降级走 LRU 驱逐重建。wgpu 无 sparse binding,重建-换绑是唯一形态,频率由迟滞阈值控制。
- **回退路径**:streaming 关闭或预算充裕时整链常驻(现行为);流送中纹理始终有合法 resident mip 链可采样(最低常驻 mip 永不驱逐),只有清晰度时滞、无错误产物。与计划 13 SVT 的关系:本机制覆盖全部常规纹理,SVT 只接巨型纹理(页粒度、feedback 驱动),两者共享预算账本互不嵌套。

### H. GPU 排序(bitonic,粒子深度排序)

- **能力 gate**:`supports_storage_buffers`(基线 compute 即可,无特殊 feature);feature 开关挂粒子质量档。
- **数据契约**:`GpuSortPass` 以计划 16 `ComputePassDescriptor` 形态注册(dispatch 链 = bitonic 逐阶段,pass 数 = log²(n) 级,经 graph 声明);排序对象为 (key: u32, value: u32) 键值对缓冲——key 取视深量化 u32(键语义由计划 12 深度键定义,sort_key 位段仍唯一归 09),value 为 `ParticleSimOutput` alive list 索引;输出排好序的 index buffer 供透明段间接提交消费(`IndirectDrawBatcher` 槽不变)。容量按 2 的幂 pad(哨兵键),UE GPUSort 同型。
- **回退路径**:feature 关闭时维持计划 12 V1 行为(emitter 粒度 CPU 排序、粒子级不排);GPU 排序是质量增强,关闭无正确性损失。对拍:同输入下 GPU 排序结果与 CPU 参考排序全序一致(键相等容许稳定性差异,测试键去重)。

### I. specular AA(Toksvig 烘焙 + 几何项)

- **能力 gate**:无硬件 gate;材质级开关 + 质量档(计划 08 `ShaderVariantKey` 变体位)。
- **数据契约**:导入期(计划 13 导入器 TX-M2 normal 管线扩展):normal map 逐 mip 降采样时记录平均法线长度 `avg_normal_length`,按 Unity `TextureNormalVariance`(:241)公式换算方差并合成进同 mip 级 roughness(写入材质 roughness 贴图 mip 或独立通道,元数据标记"已烘焙");着色端:`zr_specular_aa.wgsl` 实现 `GeometricNormalFiltering` 等价(屏幕空间法线导数→方差→roughness 钳制),经计划 08 模板按变体位拼接进 shading 路径。
- **回退路径**:变体位关闭 = 现状(无 specular AA);烘焙与几何项相互独立可单开;关闭时产物与现行为逐像素一致。

## 里程碑

按 milestone-first 政策(index §7):切片期只 `cargo check`,里程碑末进测试阶段。依赖排序:GC-M1 是能力地基,GC-M2 依赖计划 03/04/08 的对应里程碑,GC-M3 依赖计划 01/07/13,GC-M4 依赖计划 05/12/16。

### GC-M1 能力面与 compute 地基(机制 C/D + gate 修复)

依赖:计划 17 PF-M1(观测底座)。

实施切片:
1. `request_device.rs` 改为按 adapter 探测请求 binding array 三件套、`SUBGROUP`、`PIPELINE_STATISTICS_QUERY`(连同 PF-M1 的 `TIMESTAMP_QUERY` 一次收口);`rhi_wgpu/capabilities.rs` 与 `RenderCapabilitySummary` 增加 `supports_subgroup`、`supports_pipeline_statistics_query` 与 `bindless_material_supported()` 判据;`RenderCapabilityKind` 增 `SubgroupOps`/`PipelineStatisticsQuery` 档位与 class 归类。
2. `zr_reduce.wgsl` 双路径 include + `SUBGROUP_SUPPORT` def 注入;`HzbBuilder` reduce 切换为该 include(首个消费方,产物对拍)。
3. pipeline statistics QuerySet 挂接 `GpuPassTimer` 路径;`RenderFrameProfile` pass 条目增加 `Option<PipelineStatisticsSample>`。

测试阶段:`cargo test -p zircon_runtime render_capability --locked`、`cargo test -p zircon_runtime hzb --locked`;无适配器环境 GPU 用例 skip。

### GC-M2 提交热路径(机制 A/B)

依赖:GC-M1;计划 03 GS-M4、计划 04 VC-M3(剔除 compute)、计划 08 M1(变体键)。

实施切片:
1. `BindlessMaterialSlab` 槽位分配器 + fallback 填充;材质槽索引进 `GpuScene` material payload;`zr_bindless_material.wgsl` 与 bindless 变体位;非 bindless 回退路径对拍。
2. 剔除 compute 压实写改造(args 紧凑 + count 原子累加);count buffer 经 graph 声明;`IndirectDrawBatcher` 增 `multi_draw_indexed_indirect_count` 档与 `max_count` 上限;四档降级链(count → 固定 multi-draw → 逐 draw indirect → direct)统计可解释。

测试阶段:`cargo test -p zircon_runtime mesh --locked`、`cargo test -p zircon_runtime gpu_scene --locked`、`render_product_*` bindless/非 bindless 对拍;RenderDoc 抓帧确认 count 调用。

### GC-M3 带宽三件套之流送与半分辨率(机制 F/G)

依赖:计划 01 RG-M2(瞬态池)、计划 07 链定稿、计划 09(queue 段)、计划 13 TX-M1(元数据)。

实施切片:
1. `TextureMetadata` streaming 标记 + extract 纹理引用/距离采集;streamer wanted mip 计算与升降级状态机;重建-换绑与最低常驻 mip 保障;预算与 PF-M3 账本对接。
2. Transparent half-res 子段切分;半分辨率 RT 声明与深度 downsample;深度感知上采样合成 pass;粒子默认进子段。

测试阶段:`cargo test -p zircon_runtime resource_streamer --locked`、`cargo test -p zircon_runtime render_mip_streaming --locked`;半分辨率专项 `render_product_*` 基线;流送驱逐/回填确定性计数进 `render_perf_*`。

### GC-M4 缓存与质量增益(机制 E/H/I)

依赖:计划 05 LS-M3(atlas)、计划 12 PT 系列(GPU 粒子)、计划 16 CN-M1(compute 框架)、计划 13 TX-M2(normal mip)。

实施切片:
1. `ShadowCacheEntry` 失效键与命中路径(copy + 动态增量);CSM 远级联轮换;预算第 0 级降级接 PF-M3。
2. `GpuSortPass` bitonic 链(CPU 参考排序对拍先行);粒子深度排序接透明段。
3. 导入器 roughness 烘焙(avg normal length → 方差合成)+ `zr_specular_aa.wgsl` 几何项 + 变体位。

测试阶段:`cargo test -p zircon_runtime shadow --locked`、`cargo test -p zircon_runtime render_gpu_sort --locked`、`cargo test --manifest-path zircon_plugins/Cargo.toml -p <texture importer 插件> --locked`;静态场景阴影缓存 bit-exact 对拍。

## 工程落地细化

本章是计划 19 的实施权威(index §8 第 7 条)。bind group 槽位、std430、`zr_` include、queue 数值段、sort_key 位段、测试命名等全局约定直接引用 index §8,本章不重定义。跨计划契约原样消费:计划 01 `RgTextureHandle`/`RgBufferHandle`/`TransientResourcePool`;计划 02 `MeshDrawCommand`;计划 03 `GpuScene`/`IndirectDrawBatcher`;计划 04 `HzbBuilder`;计划 05 `GpuLightData`/`ShadowAtlas`;计划 08 `ShaderVariantKey`;计划 12 `ParticleSimOutput`;计划 13 `TextureMetadata`;计划 16 `ComputePassDescriptor`/`GpuReadbackQueue`;计划 17 `RenderFrameProfile`。

### 模块与文件落点

| 路径 | 内容 | 层 |
|------|------|----|
| `zircon_runtime/src/core/framework/render/backend_types.rs` | `supports_subgroup`/`supports_pipeline_statistics_query` 字段、`bindless_material_supported()`、`RenderCapabilityKind::{SubgroupOps, PipelineStatisticsQuery}`、`PipelineStatisticsSample` | framework 契约(零 wgpu) |
| `zircon_runtime/src/graphics/backend/render_backend/request_device.rs` | 按 adapter 探测请求新增 features(binding array 三件套 / SUBGROUP / 查询系) | graphics |
| `zircon_runtime/src/rhi_wgpu/capabilities.rs` | 新能力位填充(与 request 同步,杜绝"探测不请求") | RHI |
| `zircon_runtime/src/graphics/scene/scene_renderer/material/bindless_slab.rs`(新) | `BindlessMaterialSlab` 槽位分配/回收/去重/fallback 填充 | graphics |
| `zircon_runtime/src/graphics/scene/scene_renderer/mesh/`(计划 03 落点延续) | `IndirectDrawBatcher` count 档、count buffer 声明与提交分支 | graphics |
| `zircon_runtime/src/graphics/visibility/occlusion/`(计划 04 落点延续) | 剔除 compute 压实写改造 | graphics |
| `zircon_runtime/src/graphics/scene/scene_renderer/shadow/shadow_cache.rs`(新,计划 05 目录) | `ShadowCacheEntry`、失效键、命中/回填路径 | graphics |
| `zircon_runtime/src/graphics/scene/scene_renderer/transparency/half_res.rs`(新) | half-res 子段切分、上采样合成 pass executor | graphics |
| `zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_mip_streaming.rs`(新,目录式分文件惯例) | wanted mip 计算、升降级状态机、重建-换绑 | graphics |
| `zircon_runtime/src/graphics/compute/sort/`(新,计划 16 框架内) | `GpuSortPass` bitonic 链组织 | graphics |
| `zircon_runtime/assets/shaders/includes/zr_reduce.wgsl`、`zr_bindless_material.wgsl`、`zr_specular_aa.wgsl`、`zr_sort_bitonic.wgsl`(新) | 双路径归约 / bindless 索引 / specular AA / 排序 kernel | shader(计划 08 模板消费) |
| 计划 17 profiling 落点(`GpuPassTimer` 同目录) | pipeline statistics 查询挂接与解析 | graphics |

导入侧(specular AA 烘焙、streaming 标记默认值)改动走 texture importer 插件,不进 runtime(对齐计划 13 归属)。

### 核心类型与接口

```rust
// core/framework/render/backend_types.rs(契约层,零 wgpu)
impl RenderCapabilitySummary {
    pub const fn bindless_material_supported(&self) -> bool {
        self.supports_texture_binding_array
            && self.supports_partially_bound_binding_array
            && self.supports_non_uniform_resource_indexing
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PipelineStatisticsSample {
    pub vertex_invocations: u64,
    pub clipper_primitives_out: u64,
    pub fragment_invocations: u64,
    pub compute_invocations: u64,
}

// graphics 层
pub struct BindlessMaterialSlab { /* slots, free list, texture dedup map, fallback */ }
impl BindlessMaterialSlab {
    pub fn allocate(&mut self, texture: &GpuTextureResource) -> BindlessSlotIndex;
    pub fn release(&mut self, slot: BindlessSlotIndex);
}

pub struct ShadowCacheEntry {
    pub light_params_hash: u64,
    pub static_caster_revision: u64,
    pub atlas_slot_generation: u64,
    pub depth: RgTextureHandle, // 持久注册的缓存深度
}

pub struct MipStreamingPlan {
    pub texture: ResourceId,
    pub resident_mips: core::ops::Range<u8>,
    pub wanted_mips: core::ops::Range<u8>,
    pub priority: u32, // 屏占比量化,降序执行
}
```

`GpuSortPass` 经 `ComputePassDescriptor` 注册,不另设公开类型;half-res 子段以 `MeshDrawCommand` 既有 phase 标记扩展位表达,不新增命令类型。

### GPU 数据布局与 WGSL 约定

- **bindless 槽位表**:group2 binding 0 = `binding_array<texture_2d<f32>>`(容量取 limits,启动期定容);材质 uniform 内槽位索引为 u32(std430,index §8 第 2 条);采样必须经 `zr_bindless_material.wgsl` 的包装函数(内部 non-uniform 索引修饰),禁止裸索引。
- **count buffer**:`array<u32>` 首元素为 draw count,4 字节对齐即可;args buffer 布局沿用计划 03 `IndexedIndirectArgs` 5 词,压实写不改词序。
- **排序键值对**:`struct ZrSortEntry { key: u32, value: u32 }` std430 数组;深度键 = 视深 [near, far] 量化 u32(远→小,透明后向前);容量 pad 至 2 的幂,哨兵键 0xFFFFFFFF。
- **zr_reduce.wgsl**:只暴露 `zr_reduce_min_f32` / `zr_reduce_max_f32` / `zr_reduce_add_u32` 等函数与所需 workgroup var 声明宏式段落;入口 kernel 自持 entry point(include 无 entry,index §8 第 3 条)。
- **半分辨率深度 downsample**:2x2 max(保守远值)单 pass;上采样合成双边权重 = 深度差 σ 的指数核,系数进 profile。

### 帧时序与集成点

```
extract(纹理引用+距离 / 粒子输出 / caster 分类)
  → streamer 帧首:wanted mip 计算 → 升降级任务(预算内)→ 重建-换绑
  → prepare:BindlessMaterialSlab 维护、ShadowCacheEntry 失效判定、half-res 子段切分
  → compute(graph):剔除压实写 args+count → GpuSortPass(粒子深度)→ HZB/histogram(zr_reduce)
  → execute:shadow(缓存 copy + 动态增量)→ opaque → half-res translucency → 上采样合成 → post
  → 帧末:timestamp + pipeline statistics resolve → RenderFrameProfile(N 帧延迟非阻塞)
```

所有新 pass(downsample、合成、sort 链、缓存 copy)均有 graph 节点、资源 IO 声明与 executor id(index §6 第 3 条);feature 关闭时 compiled graph 不含对应 pass。

### 实施切片细化

GC-M1:①能力请求与 summary 字段(纯 CPU,单测即可)→ ②`zr_reduce.wgsl` 标量路径先行(行为不变重构)→ ③subgroup 路径 + def 注入 → ④HZB 切换与对拍 → ⑤pipeline statistics 挂接(观测字段,无消费方阻塞)。
GC-M2:①slab 分配器纯 CPU 单测 → ②bindless 变体位与 WGSL 包装 → ③产物对拍(同场景 bindless on/off)→ ④压实 compute(先 CPU 模拟对拍 args/count)→ ⑤count 提交分支与降级链统计。
GC-M3:①wanted mip 纯函数(距离→mip 公式单测)→ ②升降级状态机(无 GPU,假资产)→ ③重建-换绑与最低常驻保障 → ④half-res RT 声明与子段切分 → ⑤合成 pass 与专项基线。
GC-M4:①失效键纯函数 → ②缓存命中 copy 路径与 bit-exact 对拍 → ③CSM 远级联轮换 → ④bitonic 单阶段 kernel(CPU 对拍)→ ⑤全链排序与粒子接入 → ⑥烘焙公式纯函数(Unity 公式数值对拍)→ ⑦WGSL 几何项与变体位。

### 测试与验收清单

| 测试 | 断言 |
|------|------|
| `render_capability_bindless_gate_*` | 判据组合真值表;gate=false 时 compiled graph 与 shader 变体不含 bindless 位 |
| `render_bindless_slab_*` | 分配/回收/去重/fallback 填充;槽位泄漏为零 |
| `render_product_bindless_parity` | 同场景 bindless on/off 产物逐像素一致 |
| `render_indirect_count_compaction_*` | CPU 模拟压实:args 紧凑性、count 正确性;四档降级链统计可解释 |
| `render_reduce_parity_*` | 标量 vs subgroup 归约:min/max bit-exact、add 容差内(无适配器 skip GPU 侧) |
| `render_mip_streaming_*` | wanted mip 公式、预算内升降级决策确定性、最低常驻 mip 不可驱逐 |
| `render_perf_mip_streaming_*` | 稳态帧上传字节上限、驱逐计数(确定性计数,index §8 第 6 条) |
| `render_halfres_translucency_*` | 子段切分判据、graph 含/不含 downsample+合成 pass;专项产物基线 |
| `render_shadow_cache_*` | 失效键(参数变化/静态集 revision/槽位代际)三因子;静态场景命中帧深度 bit-exact |
| `render_gpu_sort_*` | GPU 输出 vs CPU 参考全序一致(去重键);2 的幂 pad 哨兵不漏入结果 |
| `render_specular_aa_bake_*` | avg normal length → 方差 → roughness 合成数值对 Unity 公式样例 |
| `render_capability_pipeline_stats_*` | gate=false 时字段 None;样本字段仅观测导出、不进任何断言 |

## 状态与产出记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

本子计划产出记录已超过 10 条，具体记录已迁入编号子目录。

- 迁入记录：[`19/2026-07-09-gpu-capability-optimizations-output-records.md`](19/2026-07-09-gpu-capability-optimizations-output-records.md)
