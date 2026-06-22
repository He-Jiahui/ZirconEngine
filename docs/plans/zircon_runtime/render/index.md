---
related_code:
  - zircon_runtime/src/graphics/runtime/render_framework/mod.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs
  - zircon_runtime/src/graphics/pipeline/declarations/compiled_render_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/render_graph/builder.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_queue.rs
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/DeferredShadingRenderer.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/RenderGraphBuilder.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Public/MeshPassProcessor.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/GPUScene.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/UniversalRenderer.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/ScriptableRenderer.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Volume/VolumeManager.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraph.cs
plan_sources:
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - .codex/plans/Zircon SRPRHI 渲染管线补全计划.md
  - .codex/plans/ZirconEngine Bevy-Level Rendering Completion Plan.md
  - .codex/plans/Runtime 渲染风险清单与 RenderDoc 调试支持计划.md
  - .codex/plans/Hybrid GI Lumen-Style V1 三阶段计划.md
  - .codex/plans/M5 Nanite-Like Virtual Geometry 全链收束计划.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
---

# Zircon 渲染管线 Unreal/Unity 对齐总体架构计划

本目录是 `zircon_runtime` wgpu 渲染管线向 `dev/UnrealEngine` 渲染器架构与 `dev/Graphics`(Unity SRP/URP/HDRP)管线设计对齐的总计划。它承接 `.codex/plans` 中已有的 SRP/RHI、GI、VG 等计划,分两层组织:

- **骨架层(计划 01–08)**:现有计划没有覆盖、且是 UE 渲染器性能与正确性来源的中间层 —— RDG 资源图、MeshDrawCommand 管线、GPUScene、可见性剔除、light grid、时域管线、后处理链定稿、shader permutation 与光照模型。
- **能力层(计划 09–16)**:面向用户的渲染能力族 —— 相机与渲染顺序体系(Unity 语义)、渲染器组件族、环境光照、特效与粒子、纹理体系、2D 栈、地形植被、compute 与神经网络。

参考引擎分工(对齐 zr-reference-engine-routing 技能):UE 主导引擎规模系统的内部结构(RDG/MeshPass/GPUScene/Nanite/Lumen/Landscape/Niagara);Unity Graphics 主导管线资产化、Volume 容器、相机栈、排序体系、URP 量级的简洁实现与 2D renderer;bevy/Fyrox 提供 Rust/wgpu 落地形态;godot 提供 tilemap 等通用设施;slint 提供 UI/wgpu 文本渲染参照。

## 1. 现状评审结论

当前管线(入口 `WgpuRenderFramework::submit_frame_extract`,见
`zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs`)
已经具备 Extract → Prepare → Queue/Sort → Execute → Present 的骨架、
RenderFeatureDescriptor 驱动的 pass 编排、Forward+/Deferred 双路径、阴影/后处理/SSAO/SSR 等执行器。
框架层次(framework 契约 / graphics 实现 / plugins 扩展)与 UE 的 Engine / Renderer / RenderCore / RHI 分层方向一致。

与 UE 渲染器对照,差距集中在以下骨架层,这也是"不及预期"的根因:

| # | 差距 | 现状表现 | UE 对应物 |
|---|------|---------|----------|
| 1 | RenderGraph 偏统计层 | 资源生命周期固定、无 transient 复用、无 pass culling、feature 图每帧重新解析编译 | FRDGBuilder / transient allocator / pass culling |
| 2 | 无 MeshDrawCommand 缓存管线 | `MeshDraw` 每帧全量重建,draw call 逐条提交,无静态命令跨帧缓存与状态去重 | FMeshPassProcessor / FMeshDrawCommand / cached commands |
| 3 | 无 GPUScene | 逐 draw model uniform,instancing 仅有候选统计,无 indirect 提交 | GPUScene SOA buffers / instance culling / indirect draw |
| 4 | 可见性单薄 | 仅 BVH frustum 剔除,无 relevance、无 HZB occlusion、无并行任务化 | InitViews / SceneVisibility / HZB |
| 5 | 灯光数量受 uniform 限制 | cluster grid 已建但无 per-cluster light list,灯光走场景 uniform | LightGridInjection (froxel light grid) |
| 6 | 时域管线断链 | 有 motion vector 基础与 history 槽位,但无 jitter、无 TAA resolve、history ghosting 是 P0 风险 | VelocityRendering / TemporalAA |
| 7 | 后处理链未定稿 | effect stack DAG 存在,但顺序、HDR 色彩空间、exposure、tonemap 无权威定义 | FPostProcessing 链 |
| 8 | shader 排列无管理 | fallback shader 拼接 skinning,GPU skinning 不适用自定义材质,无 permutation 缓存 | VertexFactory / MaterialShaderMap |

## 2. 目标分层映射

固定映射,所有子计划共享,不再新增 crate:

| UE 层 | Zircon 归属 | 说明 |
|-------|------------|------|
| RHI / RHICore | `zircon_runtime` RHI + wgpu backend(`graphics/backend/`) | wgpu 即 RHI;descriptor 不携带场景语义 |
| RenderCore(RDG、VertexFactory、GlobalShader) | `zircon_runtime/src/render_graph/` + `graphics/shader`、`graphics/pipeline` | RDG 升级见计划 01;VertexFactory 等价物见计划 08 |
| Renderer(SceneRenderer、MeshPass、GPUScene、Visibility、Lights、Shadows、PostProcess) | `zircon_runtime/src/graphics/scene/scene_renderer/` + `graphics/visibility/` | 计划 02–07 的主战场 |
| Engine(SceneProxy、LightSceneInfo) | `zircon_runtime::core::framework::render` extract 契约(`frame_extract.rs`、`scene_extract.rs`、`light/`) | extract 即 proxy 快照;公共 facade 固定于此 |
| 插件(Nanite/Lumen 类比) | `zircon_plugins/`(virtual_geometry、hybrid_gi、rendering) | 经 RenderFeature descriptor 接入 graph |

Unity SRP 概念到 Zircon 的补充映射:`RenderPipelineAsset/ScriptableRenderer` ↔ pipeline asset + compiled pipeline(`graphics/pipeline/`);`ScriptableRenderPass/RendererFeature` ↔ RenderFeature descriptor + pass executor;`VolumeManager/VolumeComponent` ↔ 计划 07 的 Volume 容器框架;`RTHandle` ↔ 计划 01 资源池 + 计划 07 动态分辨率;URP `ForwardLights`(zbin+tile) ↔ 计划 05 light grid。

## 3. 子计划地图与执行顺序

骨架层:

| 计划 | 文档 | 依赖 |
|------|------|------|
| 01 RenderGraph RDG 化 | `01-render-graph-rdg-alignment.md` | 无(最先) |
| 02 MeshDrawCommand 管线 | `02-mesh-draw-command-pipeline.md` | 01(执行接口) |
| 03 GPUScene 与 GPU-driven | `03-gpu-scene-gpu-driven.md` | 02(command ABI) |
| 04 可见性与剔除 | `04-visibility-culling.md` | 01(HZB pass)、03(indirect 部分) |
| 05 光照与阴影 | `05-lighting-shadows.md` | 01、03(light buffer) |
| 06 时域管线(velocity/jitter/TAA) | `06-temporal-pipeline.md` | 01(持久资源)、03(prev transform) |
| 07 后处理、色彩与 Volume 容器 | `07-postprocess-color-pipeline.md` | 01;与 06 协调顺序 |
| 08 材质、光照模型与 permutation | `08-material-shader-permutation.md` | 02、03(ABI 定稿后);可与 05–07 并行 |

能力层:

| 计划 | 文档 | 依赖 |
|------|------|------|
| 09 相机与渲染顺序体系 | `09-camera-render-ordering.md` | 01、02(排序键);可与阶段 C/D 并行 |
| 10 渲染器组件族 | `10-renderer-family.md` | 02、03、04;LOD 过渡依赖 08 |
| 11 环境光照 | `11-environment-lighting.md` | 05、08、13(cubemap) |
| 12 特效与粒子 | `12-effects-particles.md` | 03(indirect)、04(HZB)、10(注册表) |
| 13 纹理体系 | `13-texture-pipeline.md` | 01(资源池);SVT 依赖 16 readback 队列 |
| 14 2D 栈 | `14-2d-stack.md` | 09(排序)、10(注册表) |
| 15 地形与植被 | `15-terrain-vegetation.md` | 03、04、08、10、13(最后启动) |
| 16 compute 与神经网络 | `16-compute-neural.md` | 01;框架部分可提前到阶段 B 并行 |

横切层:

| 计划 | 文档 | 依赖 |
|------|------|------|
| 17 性能体系与优化 | `17-performance-and-profiling.md` | PF-M1(观测底座)无依赖、最先启动,是各计划 stats 验收的前置;PF-M2(CPU 并行)依赖 01/02;PF-M3(预算降级)接 01/07/13;PF-M4(编译治理与防回归)接 08 |

扩展层(每项机制独立 feature、可单独启停,在其依赖计划的里程碑完成后即可逐项启动,不占用阶段序):

| 计划 | 文档 | 依赖 |
|------|------|------|
| 18 进阶光照与透明特性 | `18-advanced-lighting-features.md` | 体积雾/体积光(05/07)、light cookies(05/13)、clearcoat/anisotropy/transmission(08)、OIT(09 排序之上可选)、局部 irradiance volumes(11 姊妹项)、planar reflection(09 RT 相机)、Burley SSS(07/08) |
| 19 GPU 能力利用与带宽优化 | `19-gpu-capability-optimizations.md` | GC-M1 能力 gate 修复建议随阶段 A;bindless(03/08)、multi_draw_indirect_count(03/04)、subgroup 归约(04/07/13/16)、pipeline statistics(17)、静态阴影缓存(05)、半分辨率透明(07/09/12)、mip 流送(13,与 SVT 分工)、GPU 排序(12/16)、specular AA(08/13) |

阶段划分:

- 阶段 A(地基):01 + 02。先把"图"和"命令"两条骨架立起来,后续一切 pass 与 draw 都在其上表达。16 的 compute 框架切片(CN-M1)可与阶段 A 末尾并行;17 的观测底座(PF-M1:GPU 计时/分层 stats/抓帧钩子)与阶段 A 同步启动,为全部后续计划提供量化验收手段。
- 阶段 B(GPU 场景):03 → 04。数据上 GPU,剔除走 GPU,打开 indirect 提交。
- 阶段 C(光照阴影):05。light grid 与 shadow atlas 在 GPUScene 之上落地。09(相机/排序)可在本阶段并行启动。
- 阶段 D(时域与后处理):06 → 07。velocity/jitter/TAA 解链后定稿后处理顺序、色彩空间与 Volume 容器。
- 阶段 E(材质收敛):08。几何源、光照模型与材质排列正交化,GPU skinning 全材质可用。
- 阶段 F(能力铺开):10 → {11、12、13、14 任意并行} → 15;16 的 NN 插件部分随需启动。能力层各计划共享骨架层产出的注册表、排序键、instancing 与资源池,不允许另起旁路。
- 扩展层(18/19)不占用阶段序:每项机制在其依赖计划的里程碑完成后即可独立启动;19 的 GC-M1(能力 gate 探测/请求断链修复)建议随阶段 A 一并完成。

阶段 B 之后,既有的 Hybrid GI(Lumen-style)与 Virtual Geometry(Nanite-like)计划可以切换到这套基础设施继续推进:VG 的 N3/N4(GPU 剔除、indirect)直接复用 03/04;HGI 的多灯型与 grid 复用 05。

## 4. 能力覆盖矩阵(需求 → 承接计划)

| 需求项 | 承接计划 | 备注 |
|--------|---------|------|
| 前向 / 延迟渲染 | 既有双管线 + 05 | grid 化后两管线共用光照数据 |
| 多相机、相机栈、RT 相机 | 09 | Base/Overlay、viewport rect、clear 策略 |
| render layer / 多 layer 过滤 | 09(+04/05/07 消费) | 相机/灯光/volume/渲染器同一 mask |
| render queue / order in layer / depth / ui z-index | 09 | Unity queue 数值段;统一 sort_key |
| mesh / skinned mesh / sprite / ui renderer 定制裁剪 | 10 | RendererCommon 基座 + 注册表 |
| LOD(组、过渡)| 10 | dither cross-fade 走 08 变体 |
| 静态合批 / 动态合批 / GPU instancing | 10(策略)+ 03(机制) | 互斥优先级固定、stats 可解释 |
| early-z | 02 + 04 | depth prepass 既有,HZB 补遮挡 |
| 光照模型(unlit / blinn-phong / PBR / 自定义) | 08 | ShadingModelDescriptor 注册 |
| shader / material / renderer 管理 | 08 + 10 | 变体缓存、模板拼接、注册表 |
| compute shader 框架 | 16 | descriptor 化、indirect、readback 队列 |
| 神经网络支持 | 16 | NN 插件:算子库 + 图执行器 + NN 后处理 |
| 后处理全家桶(LUT/bloom/blur/grading/DoF/SSR/dither/vignette/grain/CA) | 07 | uber pass 合并轻效果 |
| 局部容器组件化 / 全局容器 | 07 | Unity Volume 框架对齐 |
| 雾效 | 11(解析雾/高度雾)+ 07(屏幕空间) | Volume 可覆写 |
| 抗锯齿 | 06(TAA)+ 07(FXAA/SMAA)+ 既有 MSAA | 互斥/共存策略在 07 定稿 |
| 环境光遮蔽 | 既有 SSAO feature + 04(HZB 共享) | 不另立计划 |
| HDR 支持 | 07 | linear 全链、HDR 中间格式、输出转换 |
| 反射探针 | 11 | box/sphere、box projection、混合 |
| 光照烘焙(lightmap / light probe) | 11 | runtime 消费契约;烘焙器归插件 |
| skybox / cubemap | 11 + 13 | IBL 预滤波(mip 链 + SH) |
| 稀疏纹理(SVT) | 13 | feedback 驱动页加载,feature gate |
| texture2dArray / normal map / mipmap / 色彩空间 | 13 | 元数据权威化;07 互为表里 |
| 粒子(CPU/GPU) | 12 + particles 插件 | GPU 模拟写 indirect args |
| halo / lens flare / trail / billboard / projector | 12 | projector 与 decals 收敛同源 |
| 2D 文本渲染与排版 | 14 | shaping/字形图集下沉共享,UI 切换消费 |
| 图像渲染 / 九宫切片 | 14 | 拉伸/平铺/填充模式 |
| tilemap(矩形/六边形/等距、画笔、图集) | 14 | chunk 化 + 增量重建,godot 对照 |
| 动态分辨率 | 07 | render scale + 链尾 upscale |
| terrain | 15 | 插件包族;四叉树 LOD + splat |
| tree(speedtree 风格)/ grass | 15 | LOD 链 + imposter + 风动画 |
| 虚拟几何(Nanite-like)/ 动态 GI(Lumen-like) | 既有 VG/HGI 计划 | 阶段 B 后切换到新底座 |
| UI 渲染(screen-space) | 既有闭环 | 本计划不改动 |
| 性能观测(GPU 计时 / 分层 stats / RenderDoc 抓帧)与防回归 | 17 | PF-M1 最先启动;`render_perf_*` 计数断言进测试 |
| 多线程渲染(extract 双缓冲 / 并行 prepare / 并行录制) | 17 | PF-M2,依赖 01/02 |
| 内存与带宽预算、超预算降级阶梯 | 17(+01/07/13 消费) | render scale→mip bias→关 feature 顺序定稿 |
| pipeline 异步编译与首帧卡顿治理 | 17 + 08 | 变体磁盘缓存预热衔接 |
| 体积雾 / 体积光(froxel)、light cookies、平面反射 | 18 | 消费 05 light grid、07 Volume、09 RT 相机 |
| clearcoat / anisotropy / transmission / SSS、OIT、局部 irradiance volumes | 18(+08/11) | shading 扩展位 + 独立 pass,feature 可关 |
| bindless、indirect_count、subgroup、pipeline statistics | 19 | 能力 gate + 回退路径双轨,产物逐像素一致 |
| 静态阴影缓存、半分辨率透明、纹理 mip 流送、GPU 排序、specular AA | 19 | 带宽/缓存类优化,逐项接 05/07/12/13 |

## 5. 与既有 .codex/plans 计划的关系

- 承接:`Zircon SRP_RHI Rendering Architecture Roadmap.md` 已落地的 RHI v1、RenderGraph 资源图、feature descriptor、executor registry 是本计划的起点;本计划相当于该路线图的下一大段。
- 吸收:`Runtime 渲染风险清单与 RenderDoc 调试支持计划.md` 的 P0(history ghosting、缺 motion vector 重投影)由计划 06 正面解决;P1(RenderGraph 偏统计、资源生命周期固定)由计划 01 解决;RenderDoc/debug marker 接口在各计划的诊断切片中复用。
- 协同:`Hybrid GI Lumen-Style V1 三阶段计划.md` 与 `M5 Nanite-Like Virtual Geometry 全链收束计划.md` 保持独立推进,但其 GPU 数据面与剔除面应在阶段 B 完成后切换到 GPUScene/可见性新底座,避免插件各自维护私有场景缓冲。
- 细化替代:`ZirconEngine Bevy-Level Rendering Completion Plan.md` 中 post/AA 相关条目由计划 06/07 按 UE 对齐口径细化;冲突时以本目录为准。
- 能力层协同:`UI SDF 字体真实 Bake 收束计划.md` 的 SDF 产物由计划 14 的共享文本服务消费;`ZirconEngine Particles 插件完善计划.md` 的模拟面与计划 12 的渲染契约对接;`ZirconEngine 资产、Texture、模型、ZShaderZMaterialZMesh 缺口补齐计划.md` 的纹理条目并入计划 13 执行;`Rendering 插件选项补齐计划.md` 的 8 个 feature 选项分别由 05(contact shadow)、07(post 效果)、11(探针/烘焙)、12(decals/vfx)承接深化。
- 不触碰:Editor/Runtime UI 渲染链(GPU Command Stream、direct-screen、damage cache)已闭环,本计划不改其路径,仅要求 UI pass 继续作为 graph 末端 executor 存在。

## 6. 全局边界约束(各子计划必须遵守)

来自 `Runtime 吸收层与 Editor_Scene 边界收束计划.md`、`全系统重构方案.md` 与 SRP/RHI 路线图:

1. 渲染公共 facade 固定为 `zircon_runtime::core::framework::render`;不新增渲染 crate,不使用非网络语义的 `server` 命名。
2. `zircon_editor` / `zircon_app` / framework 契约层不得直接 import `wgpu`;RHI descriptor 不出现 Mesh/Material/Light/Scene 场景语义。
3. 每个实际 pass 必须有 RenderGraph 节点、资源 IO 声明与 executor id;不允许绕过 graph 的旁路提交。
4. 插件能力(GI/VG/SSAO/decals 等)只经 RenderFeature descriptor 接入;feature 关闭时 compiled graph 不含对应 pass。
5. 硬切换:新路径落地的同一变更内迁移调用方并删除旧路径,不保留兼容 re-export 或双路径。
6. 渲染模块只消费 `RenderFrameExtract`,不直接访问 ECS World;extract 仅由 runtime 生成。

## 7. 全局验收与测试基线

按 milestone-first 政策:实现切片期间只做轻量 `cargo check`,每个里程碑末进入测试阶段。

- 切片期:`cargo check -p zircon_runtime --lib --locked`
- 里程碑测试阶段:`cargo test -p zircon_runtime --lib --locked`(优先按各子计划列出的模块过滤词收窄)
- 渲染产物对拍:`render_product_*` 系列测试 + `ZR_RENDERDOC_CAPTURE_NEXT=1` 抓帧人工比对(对照 UE 同场景行为)
- 插件接缝:`cargo test --manifest-path zircon_plugins/Cargo.toml -p <受影响插件> --locked`
- 每个里程碑完成后,按源码镜像路径更新 `docs/zircon_runtime/**` 模块文档,并保持本目录子计划中的状态标记最新。

## 8. 全局工程约定(各子计划"工程落地细化"章节共享)

跨计划的实现级约定在此唯一定义,子计划不得重定义、只能引用:

1. **bind group 槽位**:`group0` = frame/view 级(相机矩阵、时间、曝光、jitter);`group1` = pass 级输入(light grid、shadow map、HZB、attachment 采样);`group2` = material 级(材质 uniform 与纹理);`group3` = object/instance 级(GPUScene instance index / instance buffer)。所有新 pass 与 shader 模板按此布局,不得私设。
2. **GPU 数据布局**:跨帧/大块数据一律 storage buffer(std430,显式 padding 注释偏移);仅 frame 级小块用 uniform;矩阵列主序;基元 f32/u32,fp16 走能力检测。
3. **WGSL 共享 include**:统一 `zr_` 前缀(如 `zr_gpu_scene.wgsl`、`zr_light_grid.wgsl`、`zr_shadow.wgsl`、`zr_fog.wgsl`、`zr_wind.wgsl`),由计划 08 的模板拼接消费;include 只暴露函数与 struct,不含 entry point。
4. **RenderQueueValue 数值段**(对齐 Unity):Background=1000、Geometry=2000、AlphaTest=2450、Transparent=3000、Overlay=4000;材质可覆写 ±100 内偏移。
5. **统一排序键** `sort_key: u64` 的位段布局唯一由计划 09 定义;其余计划(02 的命令排序、10 的合批切分、14 的 2D 排序)只消费该布局,不得另造位段。
6. **测试命名**:`render_<topic>_*` 单测、`render_product_*` 产物对拍、`render_perf_*` 性能计数断言(确定性计数:draw 数/状态切换/上传字节/瞬态峰值,归计划 17;时间类指标只观测不断言);各子计划"工程落地细化"章节给出函数级测试清单。
7. **实施权威**:每份子计划的"## 工程落地细化"章节是该计划的实施权威 —— 文件落点、类型签名、GPU 布局、切片步骤、测试清单以该章节为准;与正文概述冲突时以细化章节为新。
8. **参考对照纪律(防凭空实现)**:每个新机制动手前必须先读对应子计划"参考代码"表列出的文件 —— UE/Unity 提供设计与算法样板,`dev/bevy`/`dev/Fyrox` 提供 Rust/wgpu 落地形态(API 形态、所有权组织、wgpu 资源管理),两类都要读,不得只凭记忆或常识实现;计划中显式标注"无 Rust 同类参照"的机制(如 SVT、NN 算子),实现时必须对拍测试先行、逐切片抓帧验证。

## 代码结构规范

graphics/render 代码同样遵守引擎级 [`engine-code-structure-convention.md`](../../engine-code-structure-convention.md)：

- `graphics/**` 的大文件热点(如 `graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs`(1510)、`core/framework/render/post_process/stack.rs`(1683)、`submit_frame_extract/update_stats/base_stats.rs`)纳入 [Runtime 15](../runtime/15-code-structure-and-module-conventions.md) 的 `module_convention_gate` 与 `large_file_ownership_gate` 共同治理,按 ownership 拆 owner 叶子,root 留薄 façade。
- `runtime_*` 前缀模块(`hybrid_gi_runtime_provider/`、`virtual_geometry_runtime_provider/` 内部)按规范 §2 去冗余前缀。
- WGSL 共享 include 的 `zr_` 前缀(本文 §8.3 已定)与渲染资源描述放置遵循规范 §5(资源/描述文件归属)。
- 渲染相关测试沿用规范 §4 单一规则与 `render_*` 命名;render 子计划触及上述大文件时按 Runtime 15 的 owner 边界落地,不在旧巨型文件上叠加。

2026-06 代码审查（[`engine-code-review-findings-2026-06.md`](../../engine-code-review-findings-2026-06.md)）登记的渲染侧待补项（建议各子计划吸收）：

- **F3（P0）每帧渲染提交整帧 `extract.clone()` ×3 + 相机循环二次 clone + 几何/5 类光源 Vec 全拷贝**（~10fps 头号嫌疑）：`submit_frame_extract/build_frame_submission_context/build.rs:43,267-273,305,404`、`submit/camera_loop.rs:69`。建议 `RenderFrameExtract` 改 `Arc` 共享 + `Cow` 增量覆盖。2026-06-22 已完成 Runtime 07 第一段 `Runtime 07 render submit source-extract sharing`：`FrameSubmissionContext` 持有 `source_extract: Arc<RenderFrameExtract>`，`build_frame_submission_context` 不再克隆 meshes、五类 lights 与 previous-particle Vec 到 context，并在 handoff 前先计算 particle stats，避免 borrowed source extract 与 context move 冲突；visibility/post-process effective helper 的全帧 clone 也已收敛为单个 `effective_extract` 原地修改。状态为 `render_submit_source_extract_shared_coremin_check_passed_partial`，守卫为 `runtime_07_submit_context_shares_large_extract_payloads`，core-min `cargo check` 已在 `target\codex-runtime07-f3-source-extract-0622` 通过。2026-06-22 第二段 `Runtime 07 render camera-loop descriptor submissions` 已把 `camera_loop_submissions()` 改成 descriptor-only 枚举，terminal-target 查询不再 materialize selected extract，`submit_camera_loop()` 最后一个 camera 直接 move source extract；状态为 `render_camera_loop_descriptor_submissions_coremin_check_passed_partial`，scoped rustfmt/static/standalone guards 与 core-min `cargo check` 通过，聚焦 `cargo test -p zircon_runtime --lib camera_loop` 异常退出无测试结果/无测试二进制。2026-06-22 第三段 `Runtime 07 render camera-loop frame terminal move` 已把 direct `submit_runtime_frame(...)` 改为向 `camera_loop_frame_submissions(frame)` 传入 owned `ViewportRenderFrame`，终端 child 通过 `source_frame.take()` 和 `project_owned_frame_to_selected_camera(...)` 移动原 frame/scene/extract/UI/sidebands，旧 `project_frame_to_selected_camera(&frame, ...)` borrowed helper 删除；状态为 `render_camera_loop_frame_terminal_move_coremin_check_passed_partial`，守卫锚为 `source_frame.take()` / `project_owned_frame_to_selected_camera` / `runtime_07_submit_context_shares_large_extract_payloads`。2026-06-22 第四段 `Runtime 07 render submit feedback sideband owned merge` 已把 `collect_runtime_feedback(...)` 改为从 `&mut PreparedRuntimeSubmission` 按值 take prepared sideband 回读输出，HGI/Particle/VG merge helper 移动 Vec 内容，不再通过 borrowed sideband `clone()` 合并；状态为 `render_submit_feedback_sidebands_owned_merge_coremin_check_passed_partial`。剩余 F3：初始 viewport-sized clone，以及更大的 `RenderFrameExtract`/`ViewportRenderFrame` 共享模型与 FPS/profiling gates。归 Runtime 07 + render。
2026-06-22 第五段 `Runtime 07 render prepared sideband frame owner move` 已让 `PreparedRuntimeSubmission::into_prepared_runtime_sidebands(self)` 直接把 prepared sideband 所有权移动到 `ViewportRenderFrame`，`collect_runtime_feedback(...)` 从 `ViewportRenderFrame::prepared_runtime_sidebands_mut()` drain frame-owned readback 与 evictable id，record/present/camera-history 路径不再接收 `PreparedRuntimeSubmission`；非终端 `project_borrowed_frame_to_selected_camera(...)` 也不再克隆会被 child prepare 覆盖的 `frame.prepared_runtime_sidebands`。状态为 `render_prepared_sideband_frame_owner_move_coremin_check_passed_partial`，守卫拒绝 `prepared.prepared_runtime_sidebands()`、`plugin_renderer_outputs.clone()`、evictable id Vec clone 与 `frame.prepared_runtime_sidebands.clone()` 回流。
2026-06-22 第六段 `Runtime 07 render direct runtime-frame streaming camera loop` 已把 production `submit_runtime_frame(...)` 改为 `submit_camera_loop_frame(...)` 流式提交一个 mutable `ViewportRenderFrame`，不再 materialize `camera_loop_frame_submissions(frame)`；`CameraLoopFrameSourceState` 在每个 child 前恢复会影响下一 camera context 的源字段，`RenderFrameExtract::select_camera_descriptor(...)` 原地切换 selected camera，terminal UI 只通过 `terminal_ui.take()` 移动到最终 child，`submit_selected_runtime_frame(...)` 现在接收 `&mut ViewportRenderFrame` 并原地挂载 prepared sideband。状态为 `render_direct_runtime_frame_streaming_camera_loop_coremin_check_passed_partial`，守卫拒绝 production `CameraLoopFrameSubmission` / `camera_loop_frame_submissions(frame)` 回流。剩余 F3 仍是初始 viewport-sized clone，以及更大的共享模型与 FPS/profiling gates。
2026-06-22 第七段 `Runtime 07 render shared effective extract frame source` 已把
`ViewportRenderFrame.extract` 改为 `Arc<RenderFrameExtract>`，并让
`ViewportRenderFrame::from_shared_extract(...)` 直接消费
`FrameSubmissionContext::source_extract()` 返回的 shared effective extract。
generated submit / present 的 selected-camera extract 以 owned value 进入
`build_frame_submission_context(...)`，builder 删除旧
`let mut sized_extract = extract.clone();`，并在同一个 `effective_extract`
上写入 viewport size、renderer-owned previous particles、有效 HGI/VG、
post-process settings、AA fallback、temporal jitter 与 post-process stack/graph。
direct runtime-frame 路径在 context 构建后以 `frame.extract = context.source_extract()`
使用这份 effective payload。状态为
`render_shared_effective_extract_frame_source_coremin_check_passed_partial`。
2026-06-22 第八段 `Runtime 07 render direct runtime-frame shared context extract`
已给 direct runtime-frame submit 增加
`build_frame_submission_context_from_runtime_frame_extract(...)`，用
`FrameSubmissionExtractSource::RuntimeFrame` 对 `&mut Arc<RenderFrameExtract>` 执行
`Arc::make_mut(extract)`，并让 `submit_selected_runtime_frame(...)` 传入
`&mut frame.extract`，删除旧 direct context clone
`frame.extract.as_ref().clone()`。状态为
`render_direct_runtime_frame_shared_context_extract_coremin_check_passed_partial`。2026-06-22 第九段
`Runtime 07 render VG debug overlay frame override` 已让 `ViewportRenderFrame`
持有 `runtime_overlay_override: Option<RenderOverlayExtract>`，`build_runtime_frame(...)`
通过 `runtime_virtual_geometry_debug_overlays(...)` 只复制 overlay packet 并用
`with_runtime_overlays(runtime_overlays)` 附加 BVH/visbuffer gizmos，删除 production
`Arc::try_unwrap(extract).unwrap_or_else(...)` 整帧 fallback clone。
状态为 `render_vg_debug_overlay_frame_override_coremin_check_passed_partial`；剩余 F3 是
FPS/profiling/full Runtime 07 gates。
2026-06-22 Runtime 07 M0.3 又补齐 direct runtime-frame profiling anchors：
`submit_runtime_frame.rs` 在 selected-camera body 内记录 `build_submission_context` /
`prepare_runtime_submission` / `render_frame_with_pipeline` / `collect_runtime_feedback`
分段，状态沿用 `frame_spans_static_passed_trace_pending`；这只关闭 direct submit
CPU trace 可见性，profiling 构建/trace、权威 FPS 与 full Runtime 07 gates 仍 pending。
2026-06-22 Runtime 07 F3 `render_direct_runtime_frame_trace_export_static_passed_profile_timeout_fps_pending`
继续把 direct runtime-frame trace 从内存 snapshot 推进到 profiling artifact：`render_profiling.rs`
新增 `direct_runtime_frame_submit_exports_perfetto_trace_artifacts`，在 `profiling-chrome`
构建下提交 direct `ViewportRenderFrame`，导出 `timeline.zrtrace.json` /
`timeline.perfetto.json` / `hotspots.json` / `summary.md`，并要求 native 与 Perfetto
trace 均包含 `submit_runtime_frame`、`render_frame_with_pipeline`、`DepthPrepass`
和 `depth-prepass`。静态守卫已通过；core-min cargo 被 Runtime 06/plugin 侧 private-field
编译错误挡住，profiling-chrome 聚焦 cargo 10 分钟超时无测试结果。权威 vampire FPS、
profiling-tracy 构建耗时与 full Runtime 07 gates 仍 pending。
- **F4（P0）提交路径 viewport/VG `expect` 无降级**：`submit/{present_frame_extract,submit_runtime_frame,submit}.rs`、`prepare_runtime_submission/prepare.rs:66,104`。2026-06-22 已完成 production submit/prepare panic slice：`viewport_generation_guard.rs` 提供 `viewport_record_mut_after_generation_check(...)`，三条 submit lane 写回 viewport record 前均返回 `UnknownViewport` / `ViewportChanged` 而不是 panic；HGI/VG context 启用但 provider registration 缺席时返回 `RenderFrameworkError::UnsupportedCapability` 并清理 stale runtime state。状态为 `render_submit_viewport_provider_errors_coremin_passed`，守卫为 `runtime_07_submit_paths_return_errors_for_checked_viewport_records`，core-min focused guard 与 `cargo check` 已在 `target\codex-runtime-f16-0622-coremin` 通过。
- **F16（P1）`render_compiled_scene()` 单函数 ~533 行**：`scene_renderer/core/.../render/render.rs:80`，按 `bind_resources/execute_graph_stages/present` 拆。2026-06-22 已完成资源绑定、graph stage 执行、present/readback/pool-release 拆分：新增 `bind_compiled_scene_graph_resources.rs`、`execute_compiled_scene_graph_stages.rs`、`submit_compiled_scene_frame.rs`，`render.rs` 从 1217 行降到 409 行；core-min `cargo check`、`active_late_graph_stages_follow_compiled_pipeline_order` 聚焦测试 1/1 与 `compiled_scene` 过滤 22/22 均通过，F16 结构项闭合。
- **F11（P1）shading-model 插件注册路径半成品僵尸**（`#[allow(dead_code)]` 藏未接线 API）：`graphics/material/shading_models/registry.rs:24-60`。2026-06-22 已完成 dead API removal：未接线的 `register_plugin()` / `supported_channels()` / `len()` 与 `PluginIdBelowReservedRange` 删除，`resolve_token(...)` 作为 registry 内部 live helper 被 `resolve_lighting_model(...)` 消费；custom shading-model plugin registration remains a future Plan 08 surface，当前 custom lighting model 继续显式诊断并回退 StandardPBR。状态为 `render_shading_model_registry_dead_api_removed_coremin_passed`，守卫为 `review_f11_shading_model_registry_has_no_dead_plugin_registration_surface`。
- **F19（P1）scene renderer construction module rename**：`graphics/scene/scene_renderer/core/mod.rs:7,12` 原 `*_new` construction owner 命名读如迁移残留。2026-06-22 已硬切 `scene_renderer_core_construct` 与 `scene_renderer_construct`，所有 live caller、结构测试和 docs 路径直接指向新 owner，不保留旧目录、compat re-export 或 shim。状态为 `render_scene_renderer_construct_modules_coremin_passed`，守卫为 `review_f19_scene_renderer_construction_modules_use_construct_names`。
- **F13（P1）runtime provider 样板重复**：`graphics/{hybrid_gi,virtual_geometry,solari}_runtime_provider/provider_registration.rs` 原先各自复制 provider ID、priority、provider trait-object 与 debug 实现；HGI/VG `runtime_update.rs` 原先各自复制 stats storage/constructor/getter；HGI/VG `runtime_feedback.rs` 原先各自复制 GPU completion + visibility feedback payload；HGI/VG `prepare_input.rs` 原先各自复制 optional extract + generation 存储/getter。2026-06-22 已完成 registration 子切片：新增 `graphics/runtime_provider/registration.rs`，`RuntimeProviderRegistration<P: ?Sized>` + `define_runtime_provider_registration!` 生成 HGI、Virtual Geometry、Solari 三套 public registration wrapper；状态为 `runtime_15_provider_registration_shared_owner_coremin_check_passed`，守卫为 `runtime_15_provider_registration_uses_shared_owner`。同日已完成 update stats 子切片：新增 `graphics/runtime_provider/update.rs`，`RuntimeProviderUpdate<S>` + `define_runtime_provider_update!` 生成 HGI/VG update wrapper，原 public API 名称和 `stats()` 返回形状不变；状态为 `runtime_15_provider_update_shared_stats_owner_coremin_check_passed`，守卫为 `runtime_15_provider_update_uses_shared_stats_owner`。同日已完成 Runtime 15 F13 provider feedback shared payload owner 子切片：新增 `graphics/runtime_provider/feedback.rs`，`RuntimeProviderFeedback<G, V>` 承接共同的 GPU completion 与 visibility feedback payload，HGI/VG public feedback surface 不变；状态为 `runtime_15_provider_feedback_shared_payload_owner_coremin_check_passed`，守卫为 `runtime_15_provider_feedback_uses_shared_payload_owner`。同日已完成 Runtime 15 F13 provider prepare input shared frame owner 子切片：新增 `graphics/runtime_provider/prepare_input.rs`，`RuntimeProviderPrepareInput<'a, E>` 承接共同的 optional extract 与 generation，HGI/VG public prepare-input surface 不变；状态为 `runtime_15_provider_prepare_input_shared_frame_owner_coremin_check_passed`，守卫为 `runtime_15_provider_prepare_input_uses_shared_extract_generation_owner`。同日 `Runtime 15 F13 full provider boilerplate audit` 已由 `runtime_15_no_duplicated_provider_boilerplate` 总守卫闭合，状态为 `runtime_15_provider_boilerplate_full_audit_coremin_check_passed`。

- **F12（P1）OffscreenTarget 固定帧 texture owner**：`graphics/backend/render_backend/offscreen_target.rs` 原先用 `#[allow(dead_code)]` 掩盖 GI、scene color、bloom、G-buffer、normal、depth 等 WGPU texture owner 只通过 view 间接支撑帧图导入的问题。2026-06-22 已完成 `Runtime 15 F12 offscreen target texture owner cleanup`：`OffscreenTarget::RETAINED_FRAME_TEXTURE_COUNT` / `retained_frame_texture_count()` 显式读取 9 个固定帧 texture owner，compiled-scene `bind_frame_graph_resources(...)` 通过 debug assertion 消费该生产保活契约，状态为 `runtime_15_offscreen_target_texture_owner_cleanup_static_passed_cargo_timeout_no_result`，守卫为 `runtime_15_offscreen_target_texture_owner_cleanup`；更宽 graphics resources 与全量 F12 sweep 仍由 Runtime 15 后续执行。

## 9. 当前状态总览(2026-06-22)

本总览只汇总各子计划 `## 状态与产出记录` 的当前事实,不替代子计划状态表。完成判定以每个子计划最后一列的后续项和验证证据为准。

| 计划 | 当前状态 | 仍未完成的主要项 | 验收缺口 |
|------|----------|------------------|----------|
| 01 RenderGraph/RDG | 四个里程碑均部分完成;RG-M2 transient pool 预算/eviction、RG-M3 compiled graph cache key/invalidation、compile fingerprint/hit audit、root-driven pass culling、RG-M4 graph dump/capture artifact、alias map/profile timings、typed materialization validation、required/report-only External materialization coverage split、stale materialization binding lifetime validation、HZB executor-owned external buffer typed binding、required External binding contract、required External texture descriptor/validation 合同、`SHADOW_ATLAS` 首个生产 required External texture 绑定、optional External typed report-only descriptor/compile/materialization metadata、built-in history External actual binding、renderer-owned frame resource graph-lifetime-aware binding、first-party plugin buffer External fallback binding、runtime-prepare plugin buffer External real-binding handoff、Virtual Geometry prepared feedback producer、particles neutral GPU-frame buffer producer、particles real `ParticleGpuBackend` runtime-prepare owner、particles runtime-prepare ping-pong graph buffer aliases、particles runtime-prepare multi-system aggregation、particles transparent GPU draw shared-owner consumption、particles transparent offscreen visual readback、particles GPU readback count parity support、particles scene neutral GPU-frame auto-collection、`RgResourceResolver` pass-scoped physical lookup/postprocess validation 起步、GPU context resolver propagation、deferred-lighting/depth-prepass/deferred-G-buffer lookup 迁移与 `gpu/deferred.rs` bridge 拆分、mesh-stage/TAA reactive-mask mesh lookup 迁移、sprite/preview-sky/UI/overlay surface bridge lookup 迁移与 `gpu/surface.rs` 拆分、particle transparent/velocity bridge lookup 迁移、SSR-specific bridge lookup 迁移与 `light-list` descriptor read 声明、root postprocess bridge lookup 迁移与 `post_process/{effects,computed_resources,temporal,terminal}.rs` 拆分、HZB/shadow/velocity lookup cutover 与 raw lookup visibility 收紧,以及 `graph_resources.rs`/`materialization_validation.rs`/`resource_descriptors.rs`/`descriptor_filtering.rs`/`pass_authoring.rs`/`compile_tests.rs`/`transient_materialization.rs` 边界拆分、graph transient allocation descriptor bucket plan 与 execution bucket-local materialization 已接入 | 进一步 raw lookup 结构隔离、particles GPU transparent draw 的产品/RenderDoc 验收 | 需要 transient pool/dump/culling/compiled_graph_cache/fingerprint/alias-profile/materialization/HZB external/history external/frame external/plugin external/required External/required texture/typed optional External/RgResourceResolver/descriptor filtering/pass-authoring/transient-materialization/transient-allocation-bucket focused tests、shadow atlas compiled graph focused tests、plugin package locked check 解阻、second-frame hit integration 补跑、External typed ownership 泛化校验、test-crate root-surface/import drift 解阻、RenderDoc marker/profile/resource 对拍;stale-binding texture/buffer focused filters 已在 2026-06-18 通过;particles runtime-prepare/CPU extract/shared-manager/GPU owner/ping-pong graph binding/multi-system aggregation/transparent shared-owner draw/offscreen visual readback/count parity/scene neutral GPU-frame auto-collection focused filters 已在 2026-06-18 通过 |
| 02 MeshDrawCommand | MD-M4 基础 handoff 完成,MD-M2 command cache miss/失效诊断已接入,MD-M3 command sort input/state bucket depth 语义已接入,MD-M1~M2 仍部分完成 | per-pass processor 收敛、静态命令缓存下沉到批次构建、replay 产物对拍 | 需要 focused tests 补跑、cached command 二帧/材质变更复用对拍、state-dedup replay 产物对拍和 RenderDoc 绑定序列确认 |
| 03 GPUScene/GPU-driven | GS-M1~M3 已完成,GS-M4 部分完成 | GPU-decided draw count 与更高阶 submit 留到计划 19 | 需要 real-adapter WGPU pipeline、render-product 逐像素回归、RenderDoc multi-draw 确认 |
| 04 Visibility/HZB | VC-M1/M2/M4 完成,VC-M1 legacy visibility 平铺字段已收束,VC-M1/CO-M1 custom-target visibility payload bridge 已接入,VC-M3 compact replay 核心、phase-local dispatch diagnostics 与 HZB phase helper 模块化已接入但仍部分完成;2026-06-18 `hzb_occlusion_culler`/phase-dispatch/indirect-compaction/mesh-indirect-draw/multi-draw focused filters、`render_product_hzb_occlusion_wall_scene` clean rerun、custom-target camera visibility focused tests 与 diagnostics focused tests 已通过,并修复 `post.uber` 对 `light-list` 的缺失声明;2026-06-21 `render_product_hzb_occlusion_respects_storage_buffer_limit_fallback` 补齐 HZB per-stage storage-buffer capacity 产品 fallback 证据,同一二进制复跑 wall-scene 通过;同轮 HZB/visibility/runtime diagnostics exact focused sweep 3/3 通过;2026-06-22 `ssao_quality_profile_darkens_scene_when_enabled` 补齐 SSAO 使用共享 `hzb-furthest` 的非 RenderDoc 产品路径守卫,并修复 plugin post-process 内建 executor 的 `TONEMAPPED`/terminal-AA 路由合同 | 完整 custom-target WGPU 多相机输出链、RenderDoc HZB capture、SSR/SSAO 抓帧视觉对拍 | 需要 RenderDoc、完整 render-product/custom-target 输出对拍 |
| 05 Lighting/Shadows | light buffer/light grid 完成,2026-06-21 `render_product_many_point_lights_forward_deferred_capture_parity` 已补齐 64 点光 Forward+/Deferred 真实 WGPU 捕获对拍,`render_product_hundred_point_lights_report_local_density_stats` 已补齐百灯局部密度统计守卫,directional/multi-spot shadow-atlas receiver darkening、`render_product_csm_directional_remains_stable_under_subtexel_camera_shift`、`render_product_directional_shadow_atlas_forward_deferred_darkening_parity`、`render_product_spot_shadow_pcf_quality_changes_receiver_edge_capture`、`contact_shadow_wgpu_product_capture_darkens_screen_space_contact_region` 与 `contact_shadow_wgpu_product_capture_darkens_multiple_screen_space_contact_regions` 已补齐真实 WGPU 捕获证据,2026-06-22 `render_product_mixed_shadow_atlas_lights_darkens_multiple_receivers_capture` 已补齐 mixed directional+spot wider shadow-atlas 产品守卫 | RenderDoc 与 root wider locked checks 风险 | 需要 RenderDoc、root wider locked checks |
| 06 Temporal | 计划内切片记录齐全,主体实现已大量落地 | 宽回归与若干 shared lib-test 超时项仍需复跑 | 需要完整 temporal/product sweep、RenderDoc 和 CI 级验证 |
| 07 PostProcess/Color | exposure 完成,LUT bake compute、neutral/user LUT readback reference path、非中性 tonemap/grading CPU reference/曝光读回产品守卫、真实 user LUT WGPU 产品守卫、DoF/motion blur 独立 WGPU pass、motion blur split WGPU 产品守卫、DoF split WGPU 产品守卫、split pass HDR 中间格式修正、`post.scene-composite` SSR/fog split 与 fog/SSR wider 产品守卫、`post.blur` 通用 blur split、terminal FXAA/SMAA、SMAA 内部三阶段 edge/blend/resolve、dynamic-resolution upscale、terminal AA/upscale 产品守卫、render-scale 池统计和 uber 轻效果产品守卫已接入但仍部分完成 | terminal AA/upscale RenderDoc、blur/DoF/scene-composite RenderDoc | 需要 terminal AA/upscale RenderDoc、DoF/scene-composite/blur RenderDoc,以及更宽 product scene/RenderDoc;当前整包 fmt 被无关 UI 文件格式漂移阻塞 |
| 08 Material/Permutation | MS-M1/M2 部分完成,MS-M3 注册底座、deferred id 通路、asset/runtime 诊断、稳定 key 维度与内建 BlinnPhong forward/deferred 分支部分完成,MS-M4 key 契约、`PipelineKey`/mesh variant 派生桥、base mesh 磁盘缓存/miss 诊断、预热 manifest/CLI、staged cache handoff、asset-root shader 扫描、entry-point 多 pass 预热、`.zmaterial` alpha-test/double-sided feature、内置 shading-model 预热、初始 runtime revision 对齐、材质实例 alpha-blend Forward-only 预热、运行时 shader quality key 接线、quality-tier prewarm 枚举与 Base mesh quality-aware cache owner 已落地 | GeometrySource/template owner、三模型完整 WGSL 模板/parity、非静态 geometry/custom shading-model plugin id 的完整 prewarm manifest、Velocity/TAA/deferred/template 运行时统一 cache、编辑后 `ResourceRecord.revision` registry 导出、剩余 pass 的 mesh pipeline/disk cache 以 `ShaderVariantKey` 做最终命中主键 | 需要 Unlit/BlinnPhong/StandardPbr forward-deferred 产品对拍、custom shading model 插件注册入口、variant cache hit/miss 宽验证、prewarm 二次启动 miss=0 |
| 09 Camera/Ordering | CO-M1/3/4 部分完成,CO-M2 sequence/output policy 与 per-camera state owner 部分完成;CO-M1 `CameraRenderDescriptor`/Base+Overlay sequence resolver、extract-side `RenderViewExtract.cameras`、descriptor-driven scene layer union/custom-target visibility consumer、selected descriptor target/layer submit preflight、descriptor-owned snapshot hard cutover、offscreen/present/direct runtime-frame camera loop、terminal UI routing、target-sensitive graph cache key、camera clear plan + region-scoped scene clear draw、fixed `scene-color`/`scene-depth` physical target binding、stack-terminal final-target owner gate with suppressed diagnostics、generated offscreen/surface-present/direct runtime-frame viewport-terminal shared record/present owner gate、explicit shared viewport product/debug owner helper、shared light-grid product report boundary、selected-camera light-grid report/VG debug snapshot product-report slots、selected-camera temporal history/static-index/motion-vector/particle previous-state/HGI runtime/VG runtime slots、selected-camera `ViewportRenderRegion` viewport/scissor graph-raster enforcement、fullscreen post-process origin policy、terminal output-transfer/FXAA/SMAA local/physical region split、graph-owned Hybrid GI history copy、texture-target overlay clear-preservation 与 layered mesh product guards、linear texture conversion final-product guard、split-screen Base viewport product guard、independent multi-texture target guard source/readback helper、screen-space UI graph-tail ordering、UI/scene overlay focused pixel product guard、dynamic-resolution terminal AA target sizing、render-target texture material sampling guard、Bloom-before-Uber pass-authoring fix、FXAA product executor output-access split、CO-M3 `RenderQueueValue` queue authority/material snapshot validation、final `u64` sort-key hard cutover、transparent Sprite/Mesh mixed submission path 与 world-space UI-like transparent member product path 已接入 core framework/scene/visibility/submit/graphics;三条 texture-target overlay/linear focused runs、split-screen focused product run、independent multi-texture focused product run、render-target texture material sampling focused product run、output-policy/camera-loop owner helper focused runs、screen-space UI graph-tail focused runs、UI/scene overlay pixel focused run、dynamic-resolution terminal AA descriptor test、light-grid stats/product diagnostics filter、`render_sort_key` 7 tests、phase sort key 3 tests、geometry/sprite/material-adjusted phase queue focused tests、transparent submission/replayer focused tests 和 transparent3d product tests 已通过;per-camera history owner、per-camera previous-state owner 与 shared light-grid product-report `core-min` checks 已通过;dynamic-scene value helper visibility blocker 已修复 | editor authoring 面板、更宽像素/product custom-target composite、透明混排 RenderDoc、UI/scene overlay RenderDoc/宽场景对拍 | 需要更宽 Base/Overlay/custom-target WGPU 合成产品规则、history/previous-state 产品规则、remaining focused lib-test wrapper 清理、像素/Product/RenderDoc 验收 |
| 10 Renderer Family | RF-M1/M2 部分完成,RF-M3/RF-M4 未启动 | LOD Group、custom renderer registry、RendererCommon 基座 | 需要 LOD cross-fade、renderer registry 消费端和 batching reason diagnostics |
| 11 Environment Lighting | EL-M1~M3 部分完成,EL-M4 未启动 | analytic fog/ambient modes、正式 skybox/IBL、probe capture/bake | 需要 cubemap/IBL 资产链、probe blending/parallax、fog/ambient product tests |
| 12 Effects/Particles | FX-M1/M2 部分完成,FX-M3/FX-M4 未启动;GPU particle backend owner、runtime-prepare 多 system 聚合、indirect args 生成、shared-owner transparent draw、offscreen visual readback、count parity 与 scene neutral GPU-frame auto-collection 已在 focused lane 起步 | decals/projector 收敛、lens flare/halo、GPU transparent draw product/RenderDoc 验收 | 需要 transparent draw 产品对拍、decal receiver filtering、flare occlusion/product tests、concrete scene-to-manager `ParticleSystemComponent` integration 与 wider CPU/GPU image parity |
| 13 Texture Pipeline | TX-M1 部分完成,TX-M2~M4 未启动 | mip/normal pipeline、array/cubemap assets、sparse virtual texture | 需要 importer metadata、mip/normal compute、cubemap/array ABI、SVT feedback/readback |
| 14 2D Stack | TD-M2/TD-M4 部分完成,TD-M1/TD-M3 未启动 | text service、scene text renderer、tilemap data plane/renderer | 需要 shaping/atlas service、nine-slice/image renderer、tilemap chunk/dirty upload tests |
| 15 Terrain/Vegetation | 全部未启动 | terrain renderer/plugin skeleton、editor delta、grass scatter、tree/imposter | 需要等 03/04/08/10/13 地基稳定后进入实现和产品场景验收 |
| 16 Compute/Neural | CN-M1 部分完成,CN-M2~M4 未启动 | NN operators、graph executor、NN postprocess、统一 compute framework | 需要 compute descriptor/readback/dispatch helper、NN CPU reference tests、e2e inference |
| 17 Performance/Profiling | PF-M1/PF-M3 部分完成,PF-M2/PF-M4 未启动 | CPU parallelization、compile stutter/perf baseline、预算降级阶梯 | 需要 GPU timestamp/profile hierarchy、perf fixtures、threshold policy、shader warmup reporting |
| 18 Advanced Lighting | 全部未启动 | clearcoat/anisotropy/transmission/SSS、cookies/irradiance volumes、froxel fog/OIT/planar reflection | 需要等 05/07/08/09/11/13 地基完成后逐 feature 做 parity 和 RenderDoc 验收 |
| 19 GPU Capability/Bandwidth | GC-M2 部分完成,GC-M1/3/4 未启动 | capability surface、streaming/separate translucency、cache/quality improvements | 需要 feature request/gate 对齐、indirect-count/bindless fallback、bandwidth/cache/product parity tests |

2026-06-20 Plan 09 CO-M2 补充验证:复用 `D:\cargo-targets\zircon-runtime-texture-linear-product-0620\debug\deps\zircon_runtime-c339c28ec98a5de7.exe` 直接执行 focused tests,补齐 independent multi-texture texture target product guard(1 test)、stack/viewport/final/shared owner helper tests(2 tests) 和 light-grid stats/product diagnostics(3 tests) 的通过证据。随后默认 Forward+/Deferred 的 screen-space UI graph-tail ordering contract 也已补齐:focused `pipeline_overlay_order` 通过 2 tests,默认 Forward+/Deferred pass-order exact tests 各通过 1 test,pluginized legacy Forward+/Deferred pass-order focused run 通过 2 tests。本轮继续补齐 UI/scene overlay focused WGPU product: `render_product_ui` 通过 5 tests,覆盖 Core2D `postprocess-ui-overlay`、默认 3D `postprocess-overlay-ui`、动态分辨率 primary/direct texture target 和 dense overlay 下 UI 像素顶层;`dynamic_resolution_keeps_terminal_anti_alias_input_at_viewport_size` 通过 1 test,并通过 `zircon_runtime --features core-min` check。per-camera product-report ownership 也已补齐:`viewport_record_keeps_product_reports_per_camera_key` 通过 1 test,同一 target dir 的 `zircon_runtime --features core-min` check 通过。`primary_surface_base_camera_render_order_swap_changes_composite` 已用直接 lib-test 二进制通过 1 test,验证 full-viewport PrimarySurface 多 Base 相机最终 composite 随 `render_order` 后绘相机切换;对应 Cargo wrapper 在生成二进制后因 Windows target fingerprint 写入路径缺失失败,不计作测试失败也不计作 wrapper 通过。`texture_target_stack_preserves_composite_when_primary_surface_renders_later` 已用更新后的直接 lib-test 二进制通过 1 test,验证同帧 texture-target Base+Overlay red/green composite 不被后续 PrimarySurface blue clear 覆盖,且 viewport capture 归后续 PrimarySurface。`texture_target_render_order_feeds_later_primary_surface_material_sample` 已补齐 render-target texture 被后续 PrimarySurface mesh material 采样的产品守卫:生产修改前 red run 失败为 `red=0`,修改后 focused Cargo wrapper 通过 1 test,并用直接二进制复验三条 camera-target 产品守卫均通过。Plan 09 仍未完成 UI/scene overlay RenderDoc/宽场景 product 对拍、更宽 custom-target composite/product 规则与 product/RenderDoc 对拍;3D 透明 Sprite/Mesh 与 world-space UI-like transparent member 产品像素对拍已由 CO-M3 M3-S3 补齐,透明混排 RenderDoc 仍待补。

2026-06-20 Plan 09 CO-M2 PrimarySurface Overlay clear-depth 补充验证:`primary_surface_overlay_clear_depth_controls_depth_reuse` focused Cargo wrapper 通过 1 test,验证 Base red near mesh 写入 depth 后,Overlay green farther mesh 在 `clear_depth=false` 时被 Base depth 挡住,在 `clear_depth=true` 时清空 depth 并替换中心像素。该证据补齐原始 `render_product_overlay_stack_composites_over_base` clearDepth 双策略的 focused WGPU 产品守卫;剩余仍是 UI/scene overlay RenderDoc/宽场景对拍、更宽 custom-target composite/product 规则和 product/RenderDoc 对拍。

2026-06-20 Plan 09 CO-M3 M3-S1 进展:新增 `RenderQueueValue` 作为 Unity queue 数值权威并接入 mesh/sprite phase queue。alpha mode 只提供默认 queue,authored Unity-range queue 值先决定 queue 段和 phase,legacy 小 offset 仍按 ±100 窗口 clamp;旧 `RenderPhase::mesh_phase(pipeline, alpha_mask, transparent)` 布尔签名已删除。随后补齐材质快照/导入期矛盾校验:`StandardMaterialDescriptor.render_queue_value` 与 `resolved_render_queue_value()` 保存 typed queue snapshot,`MaterialAsset::render_queue_value()` 只为有效显式 queue 生成 `RenderQueueValue`,blend 材质显式落在 opaque/alpha-test queue 段会通过 `RenderQueueAlphaModeConflict` 进入 readiness,opaque queue=2900 保持有效并进入后段;`MaterialRuntime.render_queue_value` 把该 snapshot 带到 runtime streamer 边界。当前通过 scoped rustfmt、`zircon_runtime --features core-min` check、direct binary `render_queue` 5 tests、旧排序回归 2 tests、`material_owned_render_queue` 2 tests、`material_owned_sort_fields` 2 tests 和 material runtime streamer focused tests 2 tests。Plan 09 CO-M3 仍未完成最终 `u64` sort key 位段硬切换、sprite/world-space UI 混排和产品/RenderDoc 对拍。

2026-06-20 Plan 09 CO-M3 M3-S2 进展:最终 `u64` sort key hard cutover 已落地。`packed_sort_key_u64(...)` 现在唯一实现 `[camera_order:8][queue:13][domain:33][tie:10]` 位段;`RenderPhaseSortKey` raw 类型为 `u64`;`RenderPhaseSortComponents` 消费 typed `RenderQueueValue` 和 camera/sorting-layer/y/ui/depth 输入;breakdown/decision 诊断改为 camera_order、queue、domain、tie-breaker;mesh/sprite phase inputs 不再把 raw `render_queue`/`material_queue` 送进 core pipeline,raw authoring 字段只在 extract/mesh draw construction 边界折叠。当前通过 scoped rustfmt/check、`zircon_runtime --features core-min` check、`render_sort_key` 7 tests、direct binary `render_phase_sort_key` 3 tests、geometry phase queue 1 test、sprite phase queue 2 tests 和 material-adjusted mesh phase queue 1 test。该 checkpoint 的透明 Sprite/Mesh 混排提交路径已由 M3-S3 继续推进。

2026-06-20 Plan 09 CO-M3 M3-S3 进展:透明 Sprite/Mesh mixed submission path、world-space UI-like transparent member 场景和产品像素补测已接入。`scene_renderer/transparent/mixed_submission.rs` 从透明 mesh commands 与 sprite phase queue 构建统一提交序列,`BaseScenePass` 在 `TransparentMixedScenePass` 同一 WGPU render pass 内按统一 `u64` key 交替提交 Mesh 和 Sprite,并通过 `MeshDrawCommandReplayer::invalidate_state_after_external_pipeline()` 防止 Sprite draw 后 Mesh 状态复用错误;默认 Forward+/Deferred 的 `Transparent3d` mesh stage 现在提供 `SpriteRenderer`,实际混排仍由透明 3D Sprite phase item gate。带高 `ui_z_index` 的 world-space UI-like Sprite 仍作为 transparent queue 普通成员按 3D 透明深度排序,screen-space UI 仍留在 graph 末端。当前通过 scoped rustfmt、`zircon_runtime --features core-min` check、direct binary `transparent_submission` 2 tests、`transparent_sprite_submission` 1 test、`mesh_draw_command_replayer_rebinds_after_external_pipeline` 1 test、`render_sort_key` 7 tests,Cargo wrapper `build_sprite_vertices_routes_transparent3d_to_transparent3d_phase` 1 test,以及 product tests `transparent3d_product_interleaves_mesh_and_sprite_pixels_by_phase_sort_key` 1 test、`transparent3d_product_treats_world_space_ui_sprite_as_transparent_member` 1 test。当前环境没有运行中的 RenderDoc MCP Bridge 实例,所以透明混排 RenderDoc 捕获仍未完成。

2026-06-21 Plan 05 LS-M2 补充验证:新增 `render_product_hundred_point_lights_report_local_density_stats`,用 128 点 dense/spread 等总数场景同时检查 CPU light-grid 与真实 WGPU Forward+ 产品 stats 的 peak/average cluster 负载差异,证明 light-grid 统计随局部密度而非总灯数变化。验证通过 `cargo test -p zircon_runtime --lib render_product_hundred_point_lights_report_local_density_stats --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-hzb-storage-limit-0620 --message-format short --color never -- --test-threads=1 --nocapture` 1/1,并直接 exact 复跑该用例和 64 点 Forward+/Deferred 捕获守卫均通过;contact shadow WGPU 捕获已由同日 LS-M4 行补齐,剩余 Plan 05 缺口是 RenderDoc 与更宽产品/locked 验收。

2026-06-21 Plan 05 LS-M3 补充验证:新增 `render_product_directional_shadow_atlas_capture_records_receiver_path` 和 `render_product_directional_shadow_atlas_darkens_receiver_capture`,用真实 WGPU Forward+ receiver/caster 场景证明 directional `shadow.atlas` executor、atlas 写入、receiver 读取、caster draw、可见 receiver sample,并通过同色 receive-shadow 开关对拍证明 receiver 区域可见暗化。调试确认 atlas depth 与投影有效后,`ShadowAtlasResources` 比较采样器改为 `GreaterEqual`,并用 `render_shadow_atlas_compare_function_matches_forward_depth_contract` 锁定深度比较合同。验证通过 `cargo test -p zircon_runtime --lib render_shadow_atlas_compare_function_matches_forward_depth_contract --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-hzb-storage-limit-0620 --quiet -- --test-threads=1 --nocapture` 1/1,直接 exact 复跑 `render_product_directional_shadow_atlas_capture_records_receiver_path` 与 `render_product_directional_shadow_atlas_darkens_receiver_capture` 均通过,并通过 scoped rustfmt check。RenderDoc 与更宽产品/locked 验收仍未完成。

2026-06-21 Plan 05 LS-M3 多 spot 捕获守卫完成:新增 `render_product_multi_spot_shadow_atlas_darkens_receivers_capture`,用 3 spot/3 caster/1 receiver 的真实 Forward+ 场景对 receive-shadow 开关做全帧暗化像素、luma delta 与 RGB delta 守卫,同时新增 `RenderShadowExecutionReport.shadowed_light_count` 以报告 directional/point/spot shadow-casting light 总数。验证通过 scoped rustfmt 与 `zircon_runtime --features core-min` library check;同步 `virtual_geometry_debug_snapshot_contract.rs` 的 4 个直接 `RenderMeshSnapshot` 夹具后,`cargo check -p zircon_runtime --tests --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-shadow-spot-0621 --message-format short --color never` 已通过;长窗口 `cargo test -p zircon_runtime --lib --no-run --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-shadow-spot-0621 --message-format short --color never` 已通过并生成 `zircon_runtime-c339c28ec98a5de7.exe`;直接二进制过滤 `render_product_multi_spot_shadow_atlas_darkens_receivers_capture --nocapture --test-threads=1` 通过 1/1。剩余仍是 RenderDoc 与更宽产品/locked 验收。

2026-06-21 Plan 05 LS-M3 CSM 平移稳定补充验证:新增 `render_product_csm_directional_remains_stable_under_subtexel_camera_shift`,复用 directional shadow receiver/caster 产品场景,分别在 baseline 与 x=0.006 subtexel camera shift 下提交 shadowed/unshadowed 对拍,用同相机 unshadowed 基线抵消普通投影位移,再比较暗化像素数和 luma delta。验证通过 `cargo test -p zircon_runtime --lib render_product_csm_directional_remains_stable_under_subtexel_camera_shift --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-shadow-spot-0621 --message-format short --color never -- --test-threads=1 --nocapture` 1/1(编译/运行 6m34s,测试执行 3.50s,仓库既有 warnings)。后续 parity、PCF 与 contact shadow 切片已补齐,剩余 Plan 05 缺口是 RenderDoc 与更宽产品/locked 验收。

2026-06-21 Plan 05 LS-M3 forward/deferred shadow parity 补充验证:新增 `render_product_directional_shadow_atlas_forward_deferred_darkening_parity`,同一 directional receiver/caster 场景分别跑 Forward+ 与 Deferred shadowed/unshadowed WGPU 捕获。红灯暴露 Deferred 侧暗化像素为 0,根因是 Deferred G-buffer material alpha 只保留 shading model,没有携带 receive-shadow flag;修复后 alpha 低 7 位继续编码 shading model,高位编码 receive-shadow,`lighting.deferred` 解码后才采样 atlas shadow。验证通过 parity 产品守卫 1/1(编译/运行 6m38s,测试执行 5.92s)、`deferred_geometry_shader` 7/7、`deferred_lighting_shader` 6/6、`deferred_material_gbuffer_shaders_encode_and_decode_material_channels` 1/1,均使用 `target\codex-runtime-shadow-spot-0621`、`--locked`、`core-min`,仓库既有 warnings 保留。剩余 Plan 05 缺口是 RenderDoc 与更宽产品/locked 验收。

2026-06-21 Plan 05 LS-M4 PCF 质量补充验证:新增 `render_product_spot_shadow_pcf_quality_changes_receiver_edge_capture`,在真实 WGPU spot receiver/caster 场景下提交 Low、High 与 unshadowed baseline 三帧,同时断言两档均产生 receiver 暗化且 High 宽核改变边缘截图产物。方向光早期场景的 Low/High 截图完全相同,临时 diagnostic 已证明 quality flag 进入 shader;最终以 spot 边缘场景和 High 8 texel 半径锁定产品差异。验证通过 `cargo test -p zircon_runtime --lib render_product_spot_shadow_pcf_quality_changes_receiver_edge_capture --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-shadow-spot-0621 --message-format short --color never -- --test-threads=1 --nocapture` 1/1(测试执行 4.48s,仓库既有 warnings),以及 `cargo test -p zircon_runtime --lib shadow_atlas_resources --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-shadow-spot-0621 --message-format short --color never -- --test-threads=1 --nocapture` 3/3。contact shadow WGPU 捕获由下一段补齐,剩余 Plan 05 缺口是 RenderDoc 与更宽产品/locked 验收。

2026-06-21 Plan 05 LS-M4 contact shadow 补充验证:新增 `contact_shadow_wgpu_product_capture_darkens_screen_space_contact_region`,用真实 WGPU Forward+ receiver/blocker 场景对比 plugin-enabled 与 baseline pipeline,断言 `rendering.contact_shadow` effective feature、`contact-shadow` pass、`lighting.contact-shadow` executor、compute dispatch/workload、graph coverage 零缺口,并用最终帧暗化统计证明接触阴影乘入产品输出。`RenderPassGpuExecutionContext::require_texture_view(...)` 为插件 executor 提供 resolver-aware texture view 解析,`zircon_plugins/Cargo.lock` 已同步以满足插件 workspace `--locked` 验证。验证通过 exact filter 1/1 和 `zircon_plugin_rendering_contact_shadow_runtime --lib` 7/7,均使用 `..\target\codex-plugin-contact-shadow-0621`,仓库既有 warnings 保留。

2026-06-22 Plan 05 LS-M3 mixed shadow-atlas 宽场景产品守卫:新增 `render_product_mixed_shadow_atlas_lights_darkens_multiple_receivers_capture`,用真实 Forward+ WGPU 场景同帧提交 1 directional + 3 spot shadow-casting lights、宽 receiver 和多 caster groups,对 receive-shadow 开关做全帧与左/中/右区域暗化对拍,并断言 `shadow.atlas`、`lighting.light-grid`、directional/spot ready counts、`RenderShadowExecutionReport.shadowed_light_count == 4` 与 caster draw count。验证通过 `cargo test -p zircon_runtime --lib render_product_mixed_shadow_atlas_lights_darkens_multiple_receivers_capture --locked --jobs 1 --target-dir target\codex-shadow-wide-0622 --message-format short --color never -- --test-threads=1 --nocapture` 1/1(首次默认特性 lib-test 构建 28m46s,测试执行 5.21s,仓库既有 warnings)。contact-shadow 更宽场景由下一段 LS-M4 守卫补齐,当前剩余 Plan 05 缺口是 RenderDoc 与 root wider locked checks。

2026-06-22 Plan 05 LS-M4 contact shadow 更宽产品守卫:新增 `contact_shadow_wgpu_product_capture_darkens_multiple_screen_space_contact_regions`,用 192x128 Forward+ 宽 receiver 与左/中/右三组 blocker 对比 plugin-enabled/baseline pipeline,断言 `rendering.contact_shadow`、`contact-shadow`、`lighting.contact-shadow`、compute dispatch/workload 与 graph coverage,并要求全帧和三个接触窗口均暗化且 open receiver 区域不吞掉主要统计。验证通过 exact filter 1/1(`cargo test -p zircon_plugin_rendering_contact_shadow_runtime contact_shadow_wgpu_product_capture_darkens_multiple_screen_space_contact_regions --locked --jobs 1 --target-dir ..\target\codex-plugin-contact-shadow-0621 --message-format short --color never -- --test-threads=1 --nocapture`) 和 `zircon_plugin_rendering_contact_shadow_runtime --lib` 8/8。本切片已按 `docs/plans/engine-code-structure-convention.md` 与 `docs/plans/engine-code-review-findings-2026-06.md` 复核,仅扩展测试层,未新增生产 String error、dead-code suppression、FFI 或 builder/API 债,测试文件 531 行低于拆分阈值。剩余 Plan 05 缺口是 RenderDoc 与 root wider locked checks。

2026-06-22 Review F16 compiled-scene 结构拆分闭合代码切片:`render_compiled_scene()` 现在只保留 compiled-scene 顶层编排、draw preparation、renderer output assembly 与 GPUScene previous-frame roll。资源绑定由 `bind_compiled_scene_graph_resources.rs` 聚合并继续委派给各 binding owner,graph stage 执行由 `execute_compiled_scene_graph_stages.rs` 拥有,present/readback/transient-pool release 由 `submit_compiled_scene_frame.rs` 拥有。`render.rs` 从 1217 行降到 409 行,`submit_compiled_scene_frame.rs` 为 555 行,无新增生产 `#[allow(dead_code)]`、`Result<_, String>` 或兼容 shim。sprite-stage fixture 只验证 stage filtering,synthetic passes 通过 `PassFlags::has_side_effects` 直接作为 graph root。验证通过 scoped rustfmt check、`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-f16-0622-coremin --message-format short --color never`(既有 warnings)、focused lib-test `active_late_graph_stages_follow_compiled_pipeline_order` 1/1,以及 `cargo test -p zircon_runtime --lib compiled_scene --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-f16-0622-coremin --message-format short --color never -- --test-threads=1 --nocapture` 22/22。

2026-06-22 Review F4 submit viewport/provider panic slice 已闭合: `submit_frame_extract/viewport_generation_guard.rs` 现在集中提供 checked viewport-record lookup，generated offscreen submit、native present submit 与 direct runtime-frame submit 在 generation revalidation 后写回 record 时统一返回 typed `RenderFrameworkError`，不再走 `expect("viewport generation checked above")`。`prepare_runtime_submission/prepare.rs` 对启用 HGI/VG 但 provider registration 缺席的帧返回 `UnsupportedCapability`，同时清理 stale runtime state，避免 provider 缺席或时序竞态把渲染线程打成 panic。验证通过 scoped rustfmt、`runtime_07_submit_paths_return_errors_for_checked_viewport_records` focused guard 1/1 和 core-min `cargo check`。

2026-06-22 F11 shading-model registry dead API removal 已闭合当前结构债切片：`graphics/material/shading_models/registry.rs` 只保留内建 shading-model registry 与 live token resolver，删除未接线的 production `register_plugin()` surface 和专用 `PluginIdBelowReservedRange` 错误分支；`resolve_lighting_model(...)` 现在统一走 `RenderMaterialLightingModel::as_token()` -> registry token map。没有真实项目/插件 shading-model descriptor owner 前，不保留假的插件注册 API；custom lighting model 仍作为 `UnregisteredShadingModel` 进入 readiness 诊断并回退 StandardPBR。验证已跑 scoped rustfmt、core-min `cargo check` 和 focused code-review guard 1/1。

2026-06-22 F19 scene renderer construction module rename 已闭合当前结构债切片：原 `_new` 后缀的 scene renderer core construction owner 与 renderer construction owner 已分别硬切为 `scene_renderer_core_construct` 与 `scene_renderer_construct`，`core/mod.rs` 直接声明新 owner，构造行为继续下沉在 existing `construct/` 和 `new.rs` / `new_with_icon_source.rs` 文件内。该切片只消除目录级迁移气味，不改变 `SceneRenderer::new(...)` 等 Rust 构造 API。验证记录为 `render_scene_renderer_construct_modules_coremin_passed`，守卫为 `review_f19_scene_renderer_construction_modules_use_construct_names`，并已同步 Runtime 15 / convention / code-review finding 文档状态。

2026-06-22 F3 camera-loop frame terminal move 已闭合当前 direct runtime-frame 子切片：`submit_runtime_frame(...)` 现在消耗 owned `ViewportRenderFrame` 并交给 `camera_loop_frame_submissions(frame)`，终端 selected-camera child 使用 `source_frame.take()` 与 `project_owned_frame_to_selected_camera(...)` 移动原 frame、extract、scene、UI 与 prepared sidebands，旧 `project_frame_to_selected_camera(&frame, ...)` borrowed helper 删除。验证记录为 `render_camera_loop_frame_terminal_move_coremin_check_passed_partial`，守卫为 `runtime_07_submit_context_shares_large_extract_payloads` 的 frame-projection anchors；scoped rustfmt/static、standalone hotspot/status-output 守卫、core-min `cargo check` 与 direct exact camera-loop lib-test 均通过。该切片只移除终端 direct runtime-frame projection clone；非终端 child borrowed projection clone、初始 viewport-sized source clone、FPS/profiling/full Runtime 07 gates 仍留给后续共享模型切片。

2026-06-22 F3 feedback sideband owned merge 已闭合当前 `collect_runtime_feedback` 子切片：`PreparedRuntimeSubmission` 暴露 `take_hybrid_gi_readback_outputs()`、`take_particle_readback_outputs()`、`take_virtual_geometry_readback_outputs()`，`collect_runtime_feedback(...)` 通过 `&mut PreparedRuntimeSubmission` 消费 prepared sideband 回读输出；HGI、Particle 与 Virtual Geometry merge helper 改为接收 owned sideband 输出并移动 Vec 内容，避免 `collect_runtime_feedback.rs` 在 renderer 输出为空或混合合并时 clone 整个 prepared sideband 回读包。`prepared_runtime_sidebands()` 仍在 render 前为 `ViewportRenderFrame` 保留快照，记录阶段继续从 `PreparedRuntimeSubmission` 取可逐出 probe/page id。状态记录为 `render_submit_feedback_sidebands_owned_merge_coremin_check_passed_partial`，守卫为 `runtime_07_submit_context_shares_large_extract_payloads` 的 owned-merge anchors；该切片只移除 feedback sideband clone，初始 viewport-sized source clone、非终端 child borrowed projection clone、FPS/profiling/full Runtime 07 gates 仍留给后续共享模型切片。

2026-06-22 F3 prepared sideband frame-owner move 已闭合后续 sideband snapshot 子切片：`PreparedRuntimeSubmission` 不再提供 clone-based `prepared_runtime_sidebands()`，而是通过 `into_prepared_runtime_sidebands(self)` 消耗 prepared owner 并把 plugin renderer outputs、HGI evictable probe ids 与 VG evictable page ids 移入 `ViewportRenderFrame`。`collect_runtime_feedback(...)` 现在接收 frame-owned `RenderPreparedRuntimeSidebands` 可变引用，drain readback outputs 后再通过 `take_hybrid_gi_evictable_probe_ids()` / `take_virtual_geometry_evictable_page_ids()` 生成 runtime feedback；record/present/camera-history 路径不再持有 prepared submission。非终端 direct runtime-frame borrowed projection 也不再复制会被后续 prepare 覆盖的 frame sideband。状态记录为 `render_prepared_sideband_frame_owner_move_coremin_check_passed_partial`；该切片只移除 prepared sideband frame snapshot clone 与非终端 stale sideband projection clone。

2026-06-22 F3 direct runtime-frame streaming camera loop 已闭合当前非终端 projection clone 子切片：production `submit_runtime_frame(...)` 现在调用 `submit_camera_loop_frame(...)` 并流式复用一个 mutable `ViewportRenderFrame`，`CameraLoopFrameSourceState` 只恢复会影响下一 selected-camera context 的源字段，`select_camera_descriptor(...)` 原地切换相机，terminal UI 通过 `terminal_ui.take()` 只移动一次；旧 `camera_loop_frame_submissions(...)`、`CameraLoopFrameSubmission` 与 borrowed projection helper 收窄为 test-only 行为守卫。状态记录为 `render_direct_runtime_frame_streaming_camera_loop_coremin_check_passed_partial`；初始 viewport-sized source clone、完整共享模型、FPS/profiling/full Runtime 07 gates 仍留给后续切片。

2026-06-22 F3 direct runtime-frame trace/export 测试与静态守卫已补齐：`direct_runtime_frame_submit_exports_perfetto_trace_artifacts` 在 `profiling-chrome` 构建下通过 direct `ViewportRenderFrame` 提交生成 profiling snapshot，并通过 `export_report()` 生成 `timeline.zrtrace.json`、`timeline.perfetto.json`、`hotspots.json` 与 `summary.md`。native 和 Perfetto trace 同时锁定 `submit_runtime_frame` -> `render_frame_with_pipeline` -> `DepthPrepass/depth-prepass` 路径；状态记录为 `render_direct_runtime_frame_trace_export_static_passed_profile_timeout_fps_pending`。当前 cargo 验证未闭合：core-min 被 Runtime 06/plugin 侧 private-field 编译错误挡住，profiling-chrome 聚焦测试 10 分钟超时无结果。该切片只登记 direct runtime-frame trace artifact 代码和静态证据，不关闭权威 FPS、profiling-tracy 构建耗时或 full Runtime 07 gates。

2026-06-21 Plan 07 PP-M3-S3 轻效果产品守卫:新增 `render_product_post_uber_light_effects_change_final_frame`,用真实 headless WGPU baseline/effect-stack 双视口捕获验证 `post.uber` 和 `post.output-transfer` executor 执行,`vignette`/`film-grain`/`dither`/`chromatic-aberration` active families 无缺失资源,并通过最终帧角落 luma 与全帧 RGB delta 证明轻效果进入产品输出。验证通过 `cargo test -p zircon_runtime --lib render_product_post_uber_light_effects_change_final_frame --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-postprocess-0621 --message-format short --color never -- --test-threads=1 --nocapture` 1/1 和 scoped rustfmt check;默认 `exposure-resolve` 出现在执行节点属于当前曝光链路合同,测试按 executor/最终帧证据验收。Plan 07 剩余仍是非中性 CPU reference/曝光读回、SMAA/terminal AA/upscale 产品与 RenderDoc 验收。

2026-06-21 Plan 07 PP-M3-S2 非中性色彩产品守卫:新增 `render_product_post_non_neutral_tonemap_grading_changes_final_frame`,用真实 headless WGPU baseline 与 ACES/exposure/color-grading 双视口捕获验证 `post.color-lut-bake`、`post.uber`、`post.output-transfer` executor 执行,并通过 `RenderColorLutReadbackReport` 断言 32^3 baked LUT 有效且不再是 identity;最终帧 RGB delta 和中心 luma delta 证明非中性 LUT 进入产品输出。验证通过 `cargo test -p zircon_runtime --lib render_product_post_non_neutral_tonemap_grading_changes_final_frame --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-postprocess-0621 --message-format short --color never -- --test-threads=1 --nocapture` 1/1;首次 240s 调用超时发生在共享 lib-test 编译阶段,等待残留 cargo/rustc 完成后复用目标目录通过。此行只覆盖非 identity/final-frame 产品差异;同日后续 CPU reference/曝光读回补充记录已闭环非中性误差测量。

2026-06-21 Plan 07 PP-M3-S2 user LUT 产品守卫:新增 `render_product_post_user_lut_texture_changes_final_frame_and_matches_readback_reference`,在测试资产管理器中注册 32^3 对应的 1024x32 线性 RGBA8 2D-strip 用户 LUT,用真实 headless WGPU baseline 与 user-LUT 双视口捕获验证 ResourceStreamer LUT ready 统计、`post.color-lut-bake`/`post.uber`/`post.output-transfer` executor、`RenderColorLutReadbackReference::UserLut` 和 `user_lut_within_epsilon()`,并用最终帧 RGB delta 与红/绿通道下降证明用户 LUT 进入产品输出。验证通过 exact filter 1/1;随后同一 warmed target dir 的 `render_product_post_` 过滤通过 15/15。

2026-06-21 Plan 07 post-process graph 合同刷新:复跑 `render_product_post_` 过滤时暴露 5 个旧 framework 测试仍按 scene-composite/exposure 前的链路断言。已更新这些断言到当前 planned chain:默认链路为 `ExposureResolve -> OutputTransfer`,fog/SSR 先进入 `SceneComposite`,再由 `post.uber` 读取 `SCENE_COMPOSITED` 和 `COLOR_LUT`。同一过滤命令 `cargo test -p zircon_runtime --lib render_product_post_ --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-postprocess-0621 --message-format short --color never -- --test-threads=1 --nocapture` 通过 15/15,覆盖 12 个 graph 合同测试和 3 个产品守卫。

2026-06-21 Plan 07 PP-M3-S3 motion blur split 产品守卫:新增 `render_product_post_motion_blur_split_uses_velocity_and_changes_final_frame`,用真实 headless WGPU 验证 object/particle velocity、motion-vector tile/coarse/neighbor-max、`post.motion-blur`、`post.uber`、`post.output-transfer` 的执行顺序,读取 `scene-velocity` 证明 velocity 非零,并用最终帧 RGB delta 证明 split motion blur 进入产品输出。该红跑同时修正 DoF/motion-blur/blur/scene-composite split 输出 pipeline 的 HDR 中间格式绑定,并修正 effect-stack resource status 对 split node motion-vector 输入的误报。验证通过 exact product filter 1/1、低层 stats filter 1/1,随后 `render_product_post_` 过滤通过 16/16。

2026-06-21 Plan 07 PP-M3-S3 blur split 产品守卫:新增 `render_product_post_blur_split_changes_final_frame`,用真实 headless WGPU baseline/blur 捕获验证 `post.blur`、`post.uber`、`post.output-transfer` 执行顺序、`blur` active family、`postprocess.blurred` 与 `postprocess.tonemapped` 独立资源别名,并用最终帧 RGB delta 证明 split blur 进入产品输出。红跑发现 split passes 与 uber 共用 `post_process_params_buffer` 时,同一 command submission 内后续参数写入会覆盖前序 pass 绑定参数,导致 `post.blur` 实际读取零半径;已改为 blur/motion-blur/DoF/scene-composite/SSR/uber pass-local params uniform buffer,并删除过时共享字段。验证通过 scoped rustfmt、blur compile route 1/1、blur product exact 1/1、motion blur product 复跑 1/1、effect-stack resource-status 复跑 1/1,随后 `render_product_post_` 过滤通过 17/17。

2026-06-21 Plan 07 PP-M3-S3 DoF split 产品守卫:新增 `render_product_post_depth_of_field_split_changes_final_frame`,独立放入 `render_product_post_process_depth_of_field.rs`,用真实 headless WGPU baseline/DoF 捕获验证 `post.depth-of-field-prepare`、`post.depth-of-field`、`post.uber`、`post.output-transfer` 执行顺序,确认 `DEPTH_OF_FIELD_COC`/`DEPTH_OF_FIELD_BOKEH` alias 与 `postprocess.depth-of-fielded` 独立 backing,并用最终帧 RGB delta 证明 split DoF 进入产品输出。验证通过 scoped rustfmt、exact DoF product 1/1,随后 `cargo test -p zircon_runtime --lib render_product_post_ --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-postprocess-0621 --color never -- --test-threads=1 --nocapture` 通过 18/18。

2026-06-21 Plan 07 PP-M3-S3 scene-composite split 产品守卫:新增 `render_product_post_scene_composite_fog_changes_final_frame`,独立放入 `render_product_post_process_scene_composite.rs`,用真实 headless WGPU baseline/fog 捕获验证 `post.scene-composite`、`post.uber`、`post.output-transfer` 执行顺序,确认 `fog` active family、`postprocess.scene-composited` 独立 backing,并用最终帧 luma/RGB delta 证明屏幕空间雾进入 split scene-composite 产品输出。红跑修复 `product_postprocess_executor(...)` 对 required/produced resources 的类型分派:buffer 声明如 `history.current.exposure` 现在走 `require_buffer_by_name`,texture 声明继续走 texture view 校验。验证通过直接 lib-test 二进制 exact 1/1,随后同一二进制 `render_product_post_` 过滤通过 19/19。

2026-06-21 Plan 07 PP-M3-S2 非中性 CPU reference/曝光读回闭环:新增 `RenderExposureReadbackReport`、test-only `read_buffer_f32x4`、曝光读回统计/诊断字段,并将 non-neutral LUT readback reference 扩展为 `RenderColorLutReadbackReference::ColorTransform`。CPU reference 复刻 `color_lut_bake.wgsl` 的 tonemap、曝光 multiplier 与 color grading 公式,产品守卫现在验证曝光 multiplier/EV100、32^3 LUT `color_transform_within_epsilon()`、非 identity 与最终帧差异。红跑修复历史曝光持久 buffer 缺少 `COPY_SRC` 的真实 WGPU usage 合同。验证通过 focused `cargo test -p zircon_runtime --lib render_product_post_non_neutral_tonemap_grading_changes_final_frame --locked --target-dir target\codex-runtime-postprocess-0621 -- --test-threads=1 --nocapture` 1/1 和 broader `cargo test -p zircon_runtime --lib render_product_post_ --locked --target-dir target\codex-runtime-postprocess-0621 -- --test-threads=1 --nocapture` 19/19,既有 warnings 保留。

2026-06-21 Plan 07 PP-M3-S3 scene-composite SSR wider 产品守卫:新增 `render_product_post_scene_composite_ssr_changes_final_frame`,用真实 headless WGPU 材质/几何/灯光场景验证 SSR reflection pyramid/coarse/specular occlusion/resolve、`post.scene-composite`、`post.uber`、`post.output-transfer` 执行顺序,确认 SSR 中间/历史 backing 存在且 `screen-space-reflection` active family 无缺失资源,并用区域/全帧 RGB delta 证明 SSR 进入最终产品输出。红跑修复 SSR reflection-pyramid descriptor 缺失 `SCENE_DEPTH` 导致 RawDepth bind group sample type mismatch,并把 post-process graph 执行统计改为优先从真实 `executed_executor_ids` 反查节点,避免实际执行后漏报 `scene-composite`。验证通过 exact SSR 1/1、执行统计单测 1/1、broader `render_product_post_` 20/20、`cargo check -p zircon_runtime --lib --locked`、scoped rustfmt、`git diff --check` 与冲突标记扫描。

2026-06-22 Plan 07 PP-M4 terminal AA/upscale 产品守卫:新增 `render_product_post_process_terminal.rs`,覆盖 FXAA terminal 链与 dynamic-resolution upscale + SMAA terminal 链。FXAA 用 baseline/FXAA 双视口验证 `post.output-transfer -> post.fxaa`、terminal input backing、particle executor 和最终帧 RGB delta;SMAA 用 160x120 viewport + 0.5 render-scale 验证 80x60 内部渲染、full-viewport upscale、`post.uber -> post.upscale -> post.output-transfer -> post.smaa`、Auto->SMAA capability resolution、upscaled/terminal backing 和最终帧可见。红跑修复 `base_stats.rs` 中 terminal AA 执行计数漏掉 `post.smaa` 的生产统计。验证通过 `render_product_post_process_terminal` 2/2、`smaa` 11/11、`dynamic_resolution` 8/8、`render_product_post_process` 22/22 和 `cargo check -p zircon_runtime --lib --locked`,既有 warnings 保留。

2026-06-22 Plan 04 VC-M2 SSAO 共享 HZB 产品路径守卫:`ssao_quality_profile_darkens_scene_when_enabled` 现在在真实 headless WGPU SSAO 产品帧中断言 `hzb-build` 与 `ssao-evaluate` graph pass/executor 执行、HZB mip chain、`hzb-furthest` transient alias、required materialization 完整性和最终 SSAO darkening。红跑修复 plugin post-process descriptor 复用内建 `post.*` executor 时未经过内建 stack filtering/routing 的问题:启用 effect stack 时 `post.uber` 明确写 `TONEMAPPED`,terminal AA 下 `output-transfer` 写 `FINAL_COMPOSITED`,`AMBIENT_OCCLUSION` 保持为 `post.uber` active input。验证通过 scoped rustfmt 和 focused core-min filters 3/3;RenderDoc MCP 无运行实例,所以 mip 链抓帧、SSR/SSAO 视觉对拍与反射 hit gating 质量确认仍待后续。

后续优先级保持:先收口 04 HZB、05 shadows、07 postprocess、01/02 graph + draw command,再推进 08 以后大块未启动功能。当前最大的未完成类别仍是验收与结构债:03-07 已有核心代码,Plan 05 mixed shadow-atlas 宽场景、Plan 05 contact-shadow 更宽产品证据和 Plan 07 terminal AA/upscale 产品证据已补,F16 的 resource/stage/present-readback 结构拆分已通过 core-min 编译与 focused/broad compiled-scene 测试闭合;F4 的 production submit/prepare panic slice 已闭合;F11 的 shading-model registry dead API removal 已闭合当前半接线 API 问题;F19 的 scene renderer construction 目录命名硬切已闭合;F13 的 provider registration shared owner、provider update shared stats owner、provider feedback shared payload owner、provider prepare input shared frame owner 与 full provider boilerplate audit 已闭合;F3 的 submit-context large payload clone 已由 `source_extract: Arc<RenderFrameExtract>` 部分收口,direct runtime-frame terminal child 已改为 owned move,feedback sideband merge 已改为按值移动,direct runtime-frame 非终端 projection clone 已改为流式复用 frame,direct runtime-frame context clone 已改为 `Arc::make_mut` shared path,VG debug overlay fallback clone 已改为 frame-local overlay override,direct runtime-frame profiling anchors 已补齐。RenderDoc 截帧、root 宽回归测试、部分 heavy Cargo/editor 检查,以及 `engine-code-review-findings-2026-06.md` 标出的 profiling build/trace、FPS 与 full Runtime 07 F3 剩余验收仍未形成闭环。
