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

阶段划分:

- 阶段 A(地基):01 + 02。先把"图"和"命令"两条骨架立起来,后续一切 pass 与 draw 都在其上表达。16 的 compute 框架切片(CN-M1)可与阶段 A 末尾并行;17 的观测底座(PF-M1:GPU 计时/分层 stats/抓帧钩子)与阶段 A 同步启动,为全部后续计划提供量化验收手段。
- 阶段 B(GPU 场景):03 → 04。数据上 GPU,剔除走 GPU,打开 indirect 提交。
- 阶段 C(光照阴影):05。light grid 与 shadow atlas 在 GPUScene 之上落地。09(相机/排序)可在本阶段并行启动。
- 阶段 D(时域与后处理):06 → 07。velocity/jitter/TAA 解链后定稿后处理顺序、色彩空间与 Volume 容器。
- 阶段 E(材质收敛):08。几何源、光照模型与材质排列正交化,GPU skinning 全材质可用。
- 阶段 F(能力铺开):10 → {11、12、13、14 任意并行} → 15;16 的 NN 插件部分随需启动。能力层各计划共享骨架层产出的注册表、排序键、instancing 与资源池,不允许另起旁路。

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
