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

## 9. 当前状态总览(2026-06-20)

本总览只汇总各子计划 `## 状态与产出记录` 的当前事实,不替代子计划状态表。完成判定以每个子计划最后一列的后续项和验证证据为准。

| 计划 | 当前状态 | 仍未完成的主要项 | 验收缺口 |
|------|----------|------------------|----------|
| 01 RenderGraph/RDG | 四个里程碑均部分完成;RG-M2 transient pool 预算/eviction、RG-M3 compiled graph cache key/invalidation、compile fingerprint/hit audit、root-driven pass culling、RG-M4 graph dump/capture artifact、alias map/profile timings、typed materialization validation、required/report-only External materialization coverage split、stale materialization binding lifetime validation、HZB executor-owned external buffer typed binding、required External binding contract、required External texture descriptor/validation 合同、`SHADOW_ATLAS` 首个生产 required External texture 绑定、optional External typed report-only descriptor/compile/materialization metadata、built-in history External actual binding、renderer-owned frame resource graph-lifetime-aware binding、first-party plugin buffer External fallback binding、runtime-prepare plugin buffer External real-binding handoff、Virtual Geometry prepared feedback producer、particles neutral GPU-frame buffer producer、particles real `ParticleGpuBackend` runtime-prepare owner、particles runtime-prepare ping-pong graph buffer aliases、particles runtime-prepare multi-system aggregation、particles transparent GPU draw shared-owner consumption、particles transparent offscreen visual readback、particles GPU readback count parity support、particles scene neutral GPU-frame auto-collection、`RgResourceResolver` pass-scoped physical lookup/postprocess validation 起步、GPU context resolver propagation、deferred-lighting/depth-prepass/deferred-G-buffer lookup 迁移与 `gpu/deferred.rs` bridge 拆分、mesh-stage/TAA reactive-mask mesh lookup 迁移、sprite/preview-sky/UI/overlay surface bridge lookup 迁移与 `gpu/surface.rs` 拆分、particle transparent/velocity bridge lookup 迁移、SSR-specific bridge lookup 迁移与 `light-list` descriptor read 声明、root postprocess bridge lookup 迁移与 `post_process/{effects,computed_resources,temporal,terminal}.rs` 拆分、HZB/shadow/velocity lookup cutover 与 raw lookup visibility 收紧,以及 `graph_resources.rs`/`materialization_validation.rs`/`resource_descriptors.rs`/`descriptor_filtering.rs`/`pass_authoring.rs`/`compile_tests.rs`/`transient_materialization.rs` 边界拆分、graph transient allocation descriptor bucket plan 与 execution bucket-local materialization 已接入 | 进一步 raw lookup 结构隔离、particles GPU transparent draw 的产品/RenderDoc 验收 | 需要 transient pool/dump/culling/compiled_graph_cache/fingerprint/alias-profile/materialization/HZB external/history external/frame external/plugin external/required External/required texture/typed optional External/RgResourceResolver/descriptor filtering/pass-authoring/transient-materialization/transient-allocation-bucket focused tests、shadow atlas compiled graph focused tests、plugin package locked check 解阻、second-frame hit integration 补跑、External typed ownership 泛化校验、test-crate root-surface/import drift 解阻、RenderDoc marker/profile/resource 对拍;stale-binding texture/buffer focused filters 已在 2026-06-18 通过;particles runtime-prepare/CPU extract/shared-manager/GPU owner/ping-pong graph binding/multi-system aggregation/transparent shared-owner draw/offscreen visual readback/count parity/scene neutral GPU-frame auto-collection focused filters 已在 2026-06-18 通过 |
| 02 MeshDrawCommand | MD-M4 基础 handoff 完成,MD-M2 command cache miss/失效诊断已接入,MD-M3 command sort input/state bucket depth 语义已接入,MD-M1~M2 仍部分完成 | per-pass processor 收敛、静态命令缓存下沉到批次构建、replay 产物对拍 | 需要 focused tests 补跑、cached command 二帧/材质变更复用对拍、state-dedup replay 产物对拍和 RenderDoc 绑定序列确认 |
| 03 GPUScene/GPU-driven | GS-M1~M3 已完成,GS-M4 部分完成 | GPU-decided draw count 与更高阶 submit 留到计划 19 | 需要 real-adapter WGPU pipeline、render-product 逐像素回归、RenderDoc multi-draw 确认 |
| 04 Visibility/HZB | VC-M1/M2/M4 完成,VC-M1 legacy visibility 平铺字段已收束,VC-M1/CO-M1 custom-target visibility payload bridge 已接入,VC-M3 compact replay 核心、phase-local dispatch diagnostics 与 HZB phase helper 模块化已接入但仍部分完成;2026-06-18 `hzb_occlusion_culler`/phase-dispatch/indirect-compaction/mesh-indirect-draw/multi-draw focused filters、`render_product_hzb_occlusion_wall_scene` clean rerun、custom-target camera visibility focused tests 与 diagnostics focused tests 已通过,并修复 `post.uber` 对 `light-list` 的缺失声明;2026-06-21 `render_product_hzb_occlusion_respects_storage_buffer_limit_fallback` 补齐 HZB per-stage storage-buffer capacity 产品 fallback 证据,同一二进制复跑 wall-scene 通过;同轮 HZB/visibility/runtime diagnostics exact focused sweep 3/3 通过 | 完整 custom-target WGPU 多相机输出链、RenderDoc HZB capture | 需要 RenderDoc、完整 render-product/custom-target 输出对拍 |
| 05 Lighting/Shadows | light buffer/light grid 完成,2026-06-21 `render_product_many_point_lights_forward_deferred_capture_parity` 已补齐 64 点光 Forward+/Deferred 真实 WGPU 捕获对拍;CSM/PCF/contact shadow 仍部分完成 | 百灯成本/局部密度统计、CSM 多光型稳定性、shadow atlas 风险、PCF/contact shadow 真实捕获 | 需要更宽 render-product sweep、真实多灯阴影场景、RenderDoc 与 locked checks |
| 06 Temporal | 计划内切片记录齐全,主体实现已大量落地 | 宽回归与若干 shared lib-test 超时项仍需复跑 | 需要完整 temporal/product sweep、RenderDoc 和 CI 级验证 |
| 07 PostProcess/Color | exposure 完成,LUT bake compute、neutral/user LUT readback reference path、DoF/motion blur 独立 WGPU pass、`post.scene-composite` SSR/fog split、`post.blur` 通用 blur split、terminal FXAA/SMAA、SMAA 内部三阶段 edge/blend/resolve、dynamic-resolution upscale 和 render-scale 池统计验收已接入但仍部分完成 | user LUT 产品场景/非中性 tonemap-grading 验证、剩余 uber 轻效果归属验收(CA/vignette/grain/dither) | 需要 neutral/user LUT、DoF/motion blur、scene-composite/blur/SMAA focused tests 补跑,以及 product scene、RenderDoc;当前整包 fmt 被无关 UI 文件格式漂移阻塞 |
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

后续优先级保持:先收口 04 HZB、05 shadows、07 postprocess、01/02 graph + draw command,再推进 08 以后大块未启动功能。当前最大的未完成类别仍是验收:03-07 已有核心代码,但真实产品场景、RenderDoc 截帧、宽回归测试和部分 heavy Cargo/editor 检查仍未形成闭环。
