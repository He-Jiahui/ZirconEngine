---
related_code:
  - zircon_runtime/src/graphics/runtime/render_framework/mod.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs
  - zircon_runtime/src/graphics/pipeline/declarations/compiled_render_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/render_graph/builder.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_queue.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/residual_fallback.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/fallback_tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue/stats_bridge.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue/stats_bridge_tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/build_sprite_vertices.rs
  - zircon_runtime/src/graphics/tests/m4_behavior_layers/queue_override.rs
  - zircon_runtime/src/graphics/tests/render_product_camera_targets/custom_target.rs
  - zircon_runtime/src/graphics/tests/render_product_camera_targets/custom_target/composite.rs
  - zircon_runtime/src/graphics/tests/render_product_camera_targets/custom_target/material_sampling.rs
  - zircon_runtime/src/graphics/tests/render_product_camera_targets/custom_target/ordering.rs
  - zircon_runtime/src/graphics/tests/render_product_camera_targets/custom_target/viewport.rs
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

- **F3（P0）每帧渲染提交整帧 `extract.clone()` ×3 + 相机循环二次 clone + 几何/5 类光源 Vec 全拷贝**（~10fps 头号嫌疑）：`submit_frame_extract/build_frame_submission_context/build.rs:43,267-273,305,404`、`submit/camera_loop.rs:69`。建议 `RenderFrameExtract` 改 `Arc` 共享 + `Cow` 增量覆盖。2026-06-22 已完成 Runtime 07 第一段 `Runtime 07 render submit source-extract sharing`：`FrameSubmissionContext` 持有 `source_extract: Arc<RenderFrameExtract>`，`build_frame_submission_context` 不再克隆 meshes、五类 lights 与 previous-particle Vec 到 context，并在 handoff 前先计算 particle stats，避免 borrowed source extract 与 context move 冲突；visibility/post-process effective helper 的全帧 clone 也已收敛为单个 `effective_extract` 原地修改。状态为 `render_submit_source_extract_shared_coremin_check_passed_partial`，守卫为 `runtime_07_submit_context_shares_large_extract_payloads`，core-min `cargo check` 已在 `target\codex-runtime07-f3-source-extract-0622` 通过。2026-06-22 第二段 `Runtime 07 render camera-loop descriptor submissions` 已把 `camera_loop_submissions()` 改成 descriptor-only 枚举，terminal-target 查询不再 materialize selected extract，`submit_camera_loop()` 最后一个 camera 直接 move source extract；状态为 `render_camera_loop_descriptor_submissions_coremin_check_passed_partial`，scoped rustfmt/static/standalone guards 与 core-min `cargo check` 通过，聚焦 `cargo test -p zircon_runtime --lib camera_loop` 异常退出无测试结果/无测试二进制。2026-06-22 第三段 `Runtime 07 render camera-loop frame terminal move` 已把 direct `submit_runtime_frame(...)` 改为向 `camera_loop_frame_submissions(frame)` 传入 owned `ViewportRenderFrame`，终端 child 通过 `source_frame.take()` 和 `project_owned_frame_to_selected_camera(...)` 移动原 frame/scene/extract/UI/sidebands，旧 `project_frame_to_selected_camera(&frame, ...)` borrowed helper 删除；状态为 `render_camera_loop_frame_terminal_move_coremin_check_passed_partial`，守卫锚为 `source_frame.take()` / `project_owned_frame_to_selected_camera` / `runtime_07_submit_context_shares_large_extract_payloads`。2026-06-22 第四段 `Runtime 07 render submit feedback sideband owned merge` 已把 `collect_runtime_feedback(...)` 改为从 `&mut PreparedRuntimeSubmission` 按值 take prepared sideband 回读输出，HGI/Particle/VG merge helper 移动 Vec 内容，不再通过 borrowed sideband `clone()` 合并；状态为 `render_submit_feedback_sidebands_owned_merge_coremin_check_passed_partial`。后续 generated camera-loop shared-extract 切片已关闭普通 submit/present source clone；剩余 F3 为 FPS/profiling/full gates。归 Runtime 07 + render。
2026-06-22 第五段 `Runtime 07 render prepared sideband frame owner move` 已让 `PreparedRuntimeSubmission::into_prepared_runtime_sidebands(self)` 直接把 prepared sideband 所有权移动到 `ViewportRenderFrame`，`collect_runtime_feedback(...)` 从 `ViewportRenderFrame::prepared_runtime_sidebands_mut()` drain frame-owned readback 与 evictable id，record/present/camera-history 路径不再接收 `PreparedRuntimeSubmission`；非终端 `project_borrowed_frame_to_selected_camera(...)` 也不再克隆会被 child prepare 覆盖的 `frame.prepared_runtime_sidebands`。状态为 `render_prepared_sideband_frame_owner_move_coremin_check_passed_partial`，守卫拒绝 `prepared.prepared_runtime_sidebands()`、`plugin_renderer_outputs.clone()`、evictable id Vec clone 与 `frame.prepared_runtime_sidebands.clone()` 回流。
2026-06-22 第六段 `Runtime 07 render direct runtime-frame streaming camera loop` 已把 production `submit_runtime_frame(...)` 改为 `submit_camera_loop_frame(...)` 流式提交一个 mutable `ViewportRenderFrame`，不再 materialize `camera_loop_frame_submissions(frame)`；`CameraLoopFrameSourceState` 在每个 child 前恢复会影响下一 camera context 的源字段，`RenderFrameExtract::select_camera_descriptor(...)` 原地切换 selected camera，terminal UI 只通过 `terminal_ui.take()` 移动到最终 child，`submit_selected_runtime_frame(...)` 现在接收 `&mut ViewportRenderFrame` 并原地挂载 prepared sideband。状态为 `render_direct_runtime_frame_streaming_camera_loop_coremin_check_passed_partial`，守卫拒绝 production `CameraLoopFrameSubmission` / `camera_loop_frame_submissions(frame)` 回流。后续 generated camera-loop shared-extract 切片已关闭普通 submit/present source clone，剩余 F3 为 FPS/profiling/full gates。
2026-06-22 第七段 `Runtime 07 render shared effective extract frame source` 已把
`ViewportRenderFrame.extract` 改为 `Arc<RenderFrameExtract>`，并让
`ViewportRenderFrame::from_shared_extract(...)` 直接消费
`FrameSubmissionContext::source_extract()` 返回的 shared effective extract。
generated submit / present 的 selected-camera source 当时已进入 shared context
builder，后续普通 camera-loop 也已切到 `Arc<RenderFrameExtract>` source；
builder 删除旧
`let mut sized_extract = extract.clone();`，并在同一个 `effective_extract`
上写入 viewport size、renderer-owned previous particles、有效 HGI/VG、
post-process settings、AA fallback、temporal jitter 与 post-process stack/graph。
direct runtime-frame 路径在 context 构建后以 `frame.extract = context.source_extract()`
使用这份 effective payload。状态为
`render_shared_effective_extract_frame_source_coremin_check_passed_partial`。
2026-06-22 第八段 `Runtime 07 render direct runtime-frame shared context extract`
已给 direct runtime-frame submit 增加
`build_frame_submission_context_from_runtime_frame_extract(...)`，用
`&mut Arc<RenderFrameExtract>` 执行
`Arc::make_mut(extract_source)`，并让 `submit_selected_runtime_frame(...)` 传入
`&mut frame.extract`，删除旧 direct context clone
`frame.extract.as_ref().clone()`。状态为
`render_direct_runtime_frame_shared_context_extract_coremin_check_passed_partial`。2026-06-22 第九段
`Runtime 07 render VG debug overlay frame override` 已让 `ViewportRenderFrame`
持有 `runtime_overlay_override: Option<RenderOverlayExtract>`，`build_runtime_frame(...)`
通过 `runtime_virtual_geometry_debug_overlays(...)` 只复制 overlay packet 并用
`with_runtime_overlays(runtime_overlays)` 附加 BVH/visbuffer gizmos，删除 production
`Arc::try_unwrap(extract).unwrap_or_else(...)` 整帧 fallback clone。
状态为 `render_vg_debug_overlay_frame_override_coremin_check_passed_partial`。
2026-06-22 第十段 `Runtime 07 render generated camera-loop shared extract`
已关闭普通 generated submit/present camera-loop source clone：
`submit_camera_loop(...)` 不再执行
`extract.clone().with_selected_camera_descriptor(...)`，而是通过
`stream_camera_loop_extract_submissions(...)` 复用一个
`Arc<RenderFrameExtract>` source，`CameraLoopExtractSourceState` 在每个
child 前恢复 view/post-process/VG/HGI 源字段，再用
`Arc::make_mut(&mut source_extract)` 原地选择 camera。
`submit_selected_camera_frame(...)` 与 `present_selected_camera_frame(...)`
现在接收 `&mut Arc<RenderFrameExtract>` 并复用
`build_frame_submission_context_from_runtime_frame_extract(...)`；旧 owned
`build_frame_submission_context(...)` 与 `FrameSubmissionExtractSource`
分支已删除。状态为
`render_generated_camera_loop_shared_extract_static_passed_cargo_locked_blocked`；
scoped rustfmt 与 standalone hotspot guard 通过，最终 core-min Cargo rerun 被当前
`Cargo.lock` 漂移阻断。剩余 F3 是 FPS/profiling/full Runtime 07 gates。
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

## 9. 当前状态总览(2026-06-23)

本总览不再维护计划级状态明细表。当前状态、未完成项与验收缺口已按计划迁入各子计划的 `## 状态与产出记录` 表，render 总索引只保留读取路由。

2026-06-24 Plan 09 camera-loop test owner split 继续按结构规范收束 selected-camera submit hotpath:`graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop.rs` 已退回 332 行文件/302 source lines production owner,只保留 production loop/stream helpers 与 `#[cfg(test)] mod tests;`。`graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop/tests.rs` 保留序列/extract 覆盖和共享 fixtures,新增 `graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop/tests/frame.rs` 承接 direct runtime-frame projection coverage 与 frame-only helpers。结构守卫 `runtime_15_render_submit_camera_loop_tests_are_child_owner` 锁定父/test root/frame child owner、moved tests/helper、800 行预算和 docs/status 锚点,历史状态为 `render_plan09_camera_loop_test_owner_split_static_passed_cargo_deferred_active_editor_lane`,本次 helper follow-up 状态为 `render_plan09_camera_loop_test_helper_owner_split_static_passed_cargo_deferred_active_editor_lane`;当前只声明 rustfmt/static/line-count/docs-anchor/diff-check 证据,因 active editor/plugin Cargo lanes 不声明新的 Cargo/WGPU/RenderDoc 通过。

2026-06-24 Plan 09 update-base-stats test owner split 继续按结构规范收束 submit-owned stats 热路径:`graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs` 从 1141 source lines 降到 753 source lines,只保留 production `update_base_stats(...)` 和 stats helper;新增 `graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats/tests.rs` 承接 graph execution coverage、visibility/static-index/HZB occlusion、effect-stack resource status 与 particle velocity diagnostics tests。结构守卫 `runtime_15_render_update_base_stats_tests_are_child_owner` 锁定父/test child owner、moved tests、800 行预算和 docs/status 锚点,状态为 `render_plan09_update_base_stats_test_owner_split_static_passed_cargo_deferred_active_editor_lane`;当前只声明 rustfmt/static/line-count/docs-anchor/diff-check 证据,因 active editor/plugin Cargo lanes 不声明新的 Cargo/WGPU/RenderDoc 通过。

2026-06-24 Render backend-types owner split 按结构规范继续收束 core render DTO 热点:`core/framework/render/backend_types.rs` 从 1969 行混合 owner 降到 374 行 façade + `RenderStats` owner,新增 `core/framework/render/backend_types/camera_target.rs`、`core/framework/render/backend_types/graph_reports.rs`、`core/framework/render/backend_types/capability.rs`、`core/framework/render/backend_types/command.rs`、`core/framework/render/backend_types/quality.rs` 等子 owner 承接 camera target reports、graph reports、capability summary、command/query/payload DTO、quality profile 与 tests。结构守卫 `runtime_15_render_backend_types_are_child_owners` 锁定 moved owner 不回流、父子 800 行预算和 docs/status 锚点,状态为 `render_backend_types_owner_split_static_passed_cargo_deferred_active_compile_lane`;当前只声明 rustfmt/static/line-count/docs-anchor/diff-check 证据,因 active compile lane 不声明新的 Cargo/WGPU/RenderDoc 通过。

2026-06-24 Plan 02 VG debug snapshot owner split 继续收束 WGPU submit -> mesh-level VG evidence 的 frame-owned debug snapshot 路径:`graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot.rs` 从 955 行 owner 降为 169 行 orchestration root;新增 `graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot/page.rs`、`graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot/node_cull.rs`、`graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot/execution.rs`、`support.rs` 分别承接 page residency/cull-input projection、node/cluster cull replay、execution/visbuffer/hardware-rasterization evidence 与 saturation helper。结构守卫 `runtime_15_render_vg_debug_snapshot_is_child_owner_split` 锁定 moved owner 不回流、父子 800 行预算和 Plan 02/render index/structure/review/module docs 锚点,状态为 `render_plan02_vg_debug_snapshot_owner_split_static_passed_cargo_deferred_active_compile_lane`;当前只声明 rustfmt/static/line-count/docs-anchor/diff-check 证据,因 active compile lane 不声明新的 Cargo/WGPU/RenderDoc 通过。

2026-06-23 Plan 09 CO-M2/CO-M3 继续补源码级产品守卫:`render_product_camera_targets/custom_target.rs`、`render_product_camera_targets/primary_surface.rs` 与 `render_product_camera_targets/texture_target.rs` 已覆盖规范名 `render_product_dual_camera_rt_then_main` 的 RT camera -> later PrimarySurface material sampling 入口、规范名 `render_product_camera_render_order_swap_changes_composite` 的 PrimarySurface Base render_order swap 入口、规范名 `render_product_overlay_stack_composites_over_base` 的 PrimarySurface Base+Overlay clear_depth 双策略入口、规范名 `render_product_split_screen_viewports` 的 PrimarySurface 左右半屏 viewport clear 入口、texture-target Base+Overlay layered mesh、linear texture final-product conversion、texture-target stack 后接 later PrimarySurface、multi custom-target Base+Overlay stack 被后续 PrimarySurface 材质独立采样、同一 custom target 左右 `RenderViewportRect` 分区写入后被后续 PrimarySurface 采样、同帧 source RT -> intermediate RT -> PrimarySurface 的链式材质采样、producer render_order 晚于 consumer 时 PrimarySurface 只读上一帧 prepared target 而不读同帧未来输出、custom target Base viewport_rect 被 Overlay 继承后不污染同一 RT 另一半,以及 `custom_target_two_viewport_stacks_preserve_independent_composites_before_primary_sample` 覆盖的同一 sRGB custom target 中左右两个 Base+Overlay viewport stack 独立合成后再被后续 PrimarySurface 采样。该 owner 已按结构规范拆为 7 行 root 编排加 `assertions`/`camera`/`custom_target`/`fixture`/`mesh`/`primary_surface`/`texture_target` 子模块,并由 fixture 统一持有 WGPU framework/viewport/pipeline/profile 初始化;当前 custom-target root 已继续收敛为 25 行子模块编排,双 viewport-stack composite 守卫位于 `custom_target/composite.rs`(195 行),三条 material-sampling 守卫位于 `custom_target/material_sampling.rs`(354 行),两条 viewport 守卫位于 `custom_target/viewport.rs`(250 行),previous-frame ordering 守卫位于 `custom_target/ordering.rs`(105 行);三个纯 PrimarySurface 产品守卫位于 247 行的 `primary_surface.rs`,三条 texture-target camera 产品守卫位于 307 行的 `texture_target.rs`。camera-target custom-target owner split 状态为 `render_plan09_camera_target_custom_owner_split_static_passed_guard_timeout_no_result`,custom-target sub-owner split 状态为 `render_plan09_custom_target_subowners_static_passed`,composite source guard 状态为 `render_plan09_custom_target_composite_source_guard_static_passed`,并由 `runtime_15_render_camera_target_products_are_folder_backed` 锁定 root/child owner 与 800 行预算。同轮已把 `m4_behavior_layers.rs` 的 transparent3d 与 particle 守卫分别迁入 `m4_behavior_layers/transparent3d.rs` 和 `m4_behavior_layers/particles.rs`,并新增 `m4_behavior_layers/queue_override.rs::render_product_queue_override_reorders_draws` 覆盖计划表规范名 queue=2900 -> Transparent queue=3000 前的产品源码守卫;父文件从 1595 行逐步降到 732 行,低于 R1.4 测试文件 800 行预算;透明子模块为 225 行,粒子子模块为 80 行,queue override 子模块为 167 行,状态锚为 `render_plan09_queue_override_product_source_guard_static_passed`。Plan 09 同日还补上 texture target compile-key format class 与 output-target format label 生产切片:compiled graph cache key 现在携带 texture `ResourceId`、解析尺寸和 `rgba8unorm_srgb`/`rgba8unorm` 格式类,`ViewportRenderOutputTarget::Texture` 也保存同一次 target preflight 解析出的格式标签,让 cache key、frame output target、writeback/graph-import planning 使用同一目标事实。camera-target、transparent3d、particle 与 queue override 产品守卫、编译键格式类和 output-target 格式标签切片均以 Plan 09 状态表为准;当前只有 rustfmt/static 证据,本轮 queue override focused Cargo 两次在 184s/304s 超时无测试结果且未产出 runnable lib-test binary,不计新的 WGPU/Cargo 通过。

2026-06-23 同轮继续补齐 Plan 09 output-target prepared-format drift guard:writeback 与 graph-import planner 会在 prepared WGPU resource descriptor format 与 selected camera preflight format label 不一致时阻断 direct import/copy/conversion,并把内部阻断投影到既有 neutral blocked-format report。该切片同样以 Plan 09 状态表为准,当前只声明 rustfmt/static 证据,不声明新的 Cargo/WGPU 通过。

2026-06-23 Plan 09 CO-M4 继续补齐 sprite render-layer snapshot typed mask:`RenderSpriteSnapshot.render_layer_mask` 已从 legacy `u32` 改为 `RenderLayerSet`。scene authoring 的实体 mask 仍在 `World::render_sprite_snapshot_for_camera(...)` 边界用 `RenderLayerSet::from_legacy_mask(...)` 包装,`build_sprite_vertices(...)` 在展开 sprite WGPU 顶点前直接用 `selected_camera_layers().intersects(&sprite.render_layer_mask)` 过滤,因此 layer 40+ sprite 不再因 32-bit legacy mask 截断而误合并。visibility 输入与 particle 互通仍通过显式 `to_legacy_mask_lossy()` 留在旧 ABI 边界。源码守卫 `build_sprite_vertices_filters_sprites_by_selected_camera_layers` 已改用 layer 40 camera/sprite 组合覆盖 non-lossy 过滤,状态锚为 `render_plan09_sprite_render_layer_set_snapshot_static_passed_cargo_lock_blocked`;当前只声明 rustfmt/static/line-count/diff-check 证据,focused Cargo 被当前 `Cargo.lock` 漂移和 `--locked` 阻断,不声明新的 WGPU/RenderDoc 通过。

2026-06-23 Plan 09 CO-M4 继续补齐 mesh selected-camera layer 生产过滤与大文件收敛:`zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/phase_ordering.rs` 从原 `build.rs` 抽出 mesh phase queue/material sort owner,并在 raw mesh-vector fallback 与 `RenderPhaseQueue` consumption 两条路径用 selected camera layers 过滤 `RenderMeshSnapshot.render_layer_mask`。源码守卫 `phase_ordered_meshes_filter_meshes_by_selected_camera_layers` 同时覆盖无 phase queue 与 phase queue 路径;`build.rs` 从 1036 行降到 721 行,新 owner 为 401 行。状态锚为 `render_plan09_mesh_selected_camera_layer_filter_static_passed_cargo_timeout_no_result`;当前只声明 rustfmt/static/line-count 证据,focused Cargo 124s 超时且无 test binary,不声明新的 WGPU/RenderDoc 通过。

2026-06-23 Plan 09 CO-M4 继续补齐 mesh render-layer snapshot typed mask:`RenderMeshSnapshot.render_layer_mask` 已从 legacy `u32` 改为 `RenderLayerSet`。`scene/world/render.rs` 只在 mesh DTO 边界包装 scene entity legacy mask;`phase_ordering.rs` 直接用 `selected_camera_layers().intersects(&mesh.render_layer_mask)` 过滤 raw mesh-vector fallback 与 phase queue consumption;`StaticMeshBatchExtract` 与 frame-history mesh validation key 已同步 typed。该切片完成后,visibility input DTO 由后续 `render_plan09_visibility_renderable_input_layer_set_static_passed_cargo_lock_blocked_timeout_no_result` 关闭 typed 边界。状态锚为 `render_plan09_mesh_render_layer_set_snapshot_static_passed_cargo_lock_blocked`;当前只声明 rustfmt/static/line-count/diff-check 证据,focused Cargo 被当前 `Cargo.lock` 漂移和 `--locked` 在编译前阻断,不声明新的 WGPU/RenderDoc 通过。

2026-06-23 Plan 09 CO-M4 继续补齐 PrimitiveRelevance typed layer filter:`PrimitiveRelevance::for_mesh_view(...)` 与 `view_visible_for_layers(...)` 已从 legacy `u32` mask 输入改为 `&RenderLayerSet`,main-view relevance 直接消费 typed `RenderMeshSnapshot.render_layer_mask`。该切片关闭 shared relevance 内部 lossy layer 判断;后续 batch-key row 已把 `VisibilityBatchKey` / `FrameVisibility.render_layer_masks` 迁到 typed layer set,再由 visibility-input row 关闭 `VisibilityRenderableInput` / `build_visibility_input(...)` 旧边界。新增 layer 40 guard `primitive_relevance_preserves_layers_above_legacy_mask_width`,状态锚为 `render_plan09_primitive_relevance_typed_layer_filter_static_passed_cargo_lock_blocked_timeout_no_result`;当前只声明 rustfmt/static/line-count/diff-check 证据,focused Cargo 超时且 locked check 被当前 `Cargo.lock` 漂移阻断,不声明新的 WGPU/RenderDoc 通过。

2026-06-23 Plan 09 CO-M4 继续补齐 VisibilityBatchKey typed layer set:`VisibilityBatchKey.render_layer_mask` 与 `FrameVisibility.render_layer_masks` 已从 legacy `u32` 收束为 `RenderLayerSet`。visibility batching 现在从 typed mesh snapshot layer set clone 出 batch key、BVH instance 与 history entry,custom-target view construction 直接把 typed layer set 传给 `PrimitiveRelevance::view_visible_for_layers(...)`。新增 layer 40 guard `visibility_batch_key_preserves_layers_above_legacy_mask_width`,状态锚为 `render_plan09_visibility_batch_key_layer_set_static_passed_cargo_lock_blocked`;当前只声明 rustfmt/static/line-count/diff-check 证据,focused Cargo 与 locked check 都被当前 `Cargo.lock` 漂移在编译前阻断,不声明新的 WGPU/RenderDoc 通过。后续 `render_plan09_visibility_renderable_input_layer_set_static_passed_cargo_lock_blocked_timeout_no_result` 已把 `VisibilityRenderableInput` 也收束为 typed layer set。

2026-06-23 Plan 09 CO-M4 继续补齐 VisibilityRenderableInput typed layer set:`VisibilityRenderableInput.render_layer_mask` 已从 legacy `u32` 改为 `RenderLayerSet`,并同步 `RenderFrameExtract::from_snapshot(...)`、visibility fallback 与 `scene/world/render.rs::build_visibility_input(...)` 的 mesh/sprite/particle emitter rows。particle emitter layer 聚合改用 `RenderLayerSet::union(...)`,避免 layer 32+ 在 visibility input DTO 边界被截断。`frame_extract.rs` 的 inline tests 已迁到 `frame_extract/tests.rs`,主文件降到 894 行;新增 `render_frame_extract_visibility_input_preserves_layers_above_legacy_mask_width`。状态锚为 `render_plan09_visibility_renderable_input_layer_set_static_passed_cargo_lock_blocked_timeout_no_result`;当前只声明 rustfmt/static/line-count/diff-check 证据,focused Cargo 单测在 124s 工具窗口超时且无 test binary; locked check 被当前 `Cargo.lock` 漂移在编译前阻断,不声明新的 WGPU/RenderDoc 通过。

2026-06-23 Plan 09 CO-M4 world visibility input owner split 继续收敛 world render extract 结构边界:`scene/world/render_visibility.rs` 已从 878 行 `scene/world/render.rs` root 中抽出,承接 visibility input DTO 拼装、particle emitter typed layer aggregation 和 empty visibility fallback。`scene/world/render.rs` 当前降到 878 行,新 owner 为 87 行,并由 `runtime_15_scene_world_render_visibility_input_is_child_owner` 锁定 moved functions 不回流、docs/status 锚点和 owner 预算。状态锚为 `render_plan09_world_visibility_input_owner_split_static_passed_cargo_timeout_no_result`;当前只声明 rustfmt/static/line-count/diff-check 证据,focused locked Cargo 124s 超时且无 test binary,不声明新的 WGPU/RenderDoc 通过。

2026-06-23 Plan 09 CO-M4 继续补齐 particle render-layer snapshot typed mask:`RenderParticleSpriteSnapshot.render_layer_mask` 已从 legacy `u32` 改为 `RenderLayerSet`。`scene/world/render_particles.rs` 仍接收 scene entity legacy mask,但只在 particle DTO 边界用 `RenderLayerSet::from_legacy_mask(...)` 包装;`build_particle_vertices(...)`、`build_particle_velocity_vertices(...)` 与 `RenderPipelineAsset::compile(...)` 直接用 `selected_camera_layers().intersects(&sprite.render_layer_mask)` 过滤,因此 layer 32+ particle sprite 不再在 CPU 侧被 legacy mask 截断。该切片完成时 `build_visibility_input(...)` 仍通过 `to_legacy_mask_lossy()` OR 聚合 emitter layer mask;后续 visibility-input row 已改为 typed `RenderLayerSet::union(...)`。源码守卫覆盖 sprite vertex、velocity current sprite、compile auto-insert 和 world-particle mask 保留;状态锚为 `render_plan09_particle_render_layer_set_snapshot_static_passed_cargo_lock_blocked`;当前只声明 rustfmt/static/source-anchor/line-count/diff-check 证据,focused Cargo 被当前 `Cargo.lock` 漂移和 `--locked` 阻断,不声明新的 WGPU/RenderDoc 通过。旧锚 `render_plan09_particle_selected_camera_layer_filter_static_passed_cargo_timeout_no_result` 仅代表初始 selected-camera filter 切片。

2026-06-23 Plan 09 CO-M4 继续补齐 light layer typed mask 收束:`RenderDirectionalLightSnapshot`、`RenderPointLightSnapshot`、`RenderSpotLightSnapshot` 和 `RenderRectLightSnapshot` 的 `layer_mask` 已从 legacy `u32` 改为 `RenderLayerSet`。scene extraction 在实体 legacy mask -> render DTO 边界包装 typed set,`light_buffer.rs` 只在写入 `GpuLightData.shadow_slot_layer[1]` 时用 `to_legacy_mask_lossy()` 适配当前 32-bit GPU ABI。状态锚为 `render_plan09_light_layer_set_snapshot_static_passed_cargo_timeout_no_result`;当前只声明 rustfmt/static/line-count/diff-check 证据,focused Cargo 188s 超时且无 test binary,不声明新的 WGPU/RenderDoc 通过。

2026-06-23 Plan 05/09 shadow view-projection owner split 已把 `graphics/scene/scene_renderer/shadow/view_projection.rs` 作为 child owner 挂到 `shadow/mod.rs`,承接 directional cascade、point face、spot shadow view-projection 矩阵构造和方向/距离 sanitizing;`shadow/plan.rs` 只保留 atlas allocation、slot pass、globals 与 light slot assignment 编排。新增 `runtime_15_shadow_plan_view_projection_is_child_owner`,状态锚为 `render_plan05_09_shadow_view_projection_owner_split_static_passed`;当前只声明 rustfmt/static/line-count/docs-anchor/diff-check 证据。focused locked `cargo check -p zircon_runtime --lib --no-default-features --features core-min` 在 304s 工具窗口超时且无输出,不声明新的 Cargo/WGPU/RenderDoc 通过。

2026-06-23 Plan 07 post-process stack owner split 已把 `core/framework/render/post_process/graph_resource_names.rs` 作为资源名表 owner；`stack.rs` 降到 586 行，只保留 `PostProcessStackDescriptor` 构造、validated graph 与 history-resource stripping；17 个原 inline stack tests 迁入 `core/framework/render/post_process/stack/tests/{exposure,terminal_chain,screen_space_reflection,temporal_history,effect_stack}.rs`。新增 `runtime_15_post_process_stack_is_folder_backed` 锁定 owner/预算/文档锚点，状态锚为 `render_plan07_post_process_stack_owner_split_static_passed`;当前只声明 scoped rustfmt/static/line-count/docs-anchor/diff-check 证据，不声明新的 Cargo/WGPU/RenderDoc 通过。

2026-06-23 Plan 07 volume component owner split 已把 `VolumeParamValue`、`VolumeParamSchema`、`VolumeParamInterpFn`、`interp_*` 与参数默认值工厂迁入 `core/framework/render/post_process/volume_component/params.rs`，并把 5 个原 inline volume component tests 迁入 `core/framework/render/post_process/volume_component/tests.rs`。`volume_component.rs` 降到 642 行，只保留 `VolumeComponentDescriptor`、内建 component descriptor 表和 read/apply 写回映射；新增 `runtime_15_post_process_volume_component_is_folder_backed` 锁定 owner/预算/文档锚点，状态锚为 `render_plan07_volume_component_owner_split_static_passed`;当前只声明 scoped rustfmt/static/line-count/docs-anchor/diff-check 证据；focused locked `render_volume_component` Cargo 测试 184s 超时无结果且未发现本 target 残留，不计 Cargo 通过，不声明新的 WGPU/RenderDoc 通过。

2026-06-23 Plan 07 volume camera transition product guard 已补 `graphics/tests/render_product_post_process_volume.rs::render_product_post_volume_camera_transition` 并挂到 `graphics/tests/mod.rs`。该守卫用真实 headless WGPU 三视口提交相机在 post-process sphere volume 外、blend 区与中心的帧，通过 `PostProcessVolumeExtract`/`VolumeShapeExtract::sphere` 驱动 `post.vignette`，断言 `post.uber`、`post.output-transfer`、角落 luma 逐步下降与最终帧 delta 增长；状态锚为 `render_plan07_volume_camera_transition_product_guard_static_passed_cargo_timeout_no_result`。当前 scoped rustfmt/static/line-count 通过；focused locked Cargo 245s 超时无结果且无 cargo/rustc 残留，不计 Cargo/WGPU/RenderDoc 通过。

2026-06-23 Plan 07 full-chain all-effects product guard 已补 `graphics/tests/render_product_post_process_full_chain.rs::render_product_post_full_chain_all_effects_on`，并把场景 fixture 拆入 `graphics/tests/render_product_post_process_full_chain/fixture.rs` 后挂到 `graphics/tests/mod.rs`。该守卫用真实 headless WGPU 同帧串起 histogram exposure、bloom、DoF、motion blur、SSR/fog scene-composite、blur、color LUT bake/tonemap/user LUT、vignette/grain/dither/CA、dynamic-resolution upscale 与 SMAA terminal AA，断言 executor 顺序、active families、关键 alias/backing、scene-velocity readback 和最终帧 delta；状态锚为 `render_plan07_full_chain_all_effects_product_guard_static_passed_cargo_timeout_no_result`。当前 scoped rustfmt/static/line-count 与 locked core-min `cargo check` 通过(既有 warnings)；focused locked Cargo test 604s 超时无结果且无本 target-dir 残留，不计 WGPU/RenderDoc 通过。

2026-06-23 Plan 07 built-in post-process executor owner split 已把 `graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors.rs` 拆成 folder-backed owner：根文件保留 registry-facing executor 函数并降到 574 行，`graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors/frame_effects.rs` 承接 frame effect predicate，`graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors/graph_resources.rs` 承接 `product_postprocess_executor(...)` 与 graph resource kind/external binding 校验，`graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors/resource_routing.rs` 承接 terminal/bloom/uber resource routing 和原 inline 路由测试。新增 `runtime_15_builtin_postprocess_executors_are_folder_backed` 锁定 owner 挂载、moved helper/test 不回流、docs/status 锚点和行数预算，状态锚为 `render_plan07_builtin_postprocess_executor_owner_split_static_passed`；当前 scoped rustfmt/static/line-count/docs-anchor/diff-check 通过，locked core-min `cargo check` 通过(既有 warnings)，focused locked structure Cargo test 被当前 `Cargo.lock` 更新需求在编译前阻断，不声明新的 WGPU/RenderDoc 通过。

2026-06-23 Plan 01 render graph execution record owner split 已把 `graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs` 拆成 folder-backed owner：根文件保留 `RenderGraphExecutionRecord` 聚合 API、stage/profile/resource/materialization/alias/light-grid report surface 并降到 550 行，`graphics/scene/scene_renderer/graph_execution/render_graph_execution_record/compute_workload.rs` 承接 compute dispatch record、workload dispatch context、dispatch group sizing、compute workload audit record/status 与 compute audit tests，`graphics/scene/scene_renderer/graph_execution/render_graph_execution_record/tests.rs` 承接非 compute record 行为测试。新增 `runtime_15_render_graph_execution_record_is_folder_backed` 锁定 owner 挂载、moved compute/test owner 不回流、docs/status 锚点和行数预算，状态锚为 `render_plan01_execution_record_owner_split_static_passed`；当前 scoped rustfmt/static/line-count/docs-anchor/diff-check 通过，locked core-min Cargo check 被当前 `Cargo.lock` 更新需求在编译前阻断，不计 Cargo/WGPU/RenderDoc 通过。

2026-06-23 Plan 02 MD-M2 static command cache virtual geometry residual product guard 已扩展 `zircon_runtime/src/graphics/tests/render_product_mesh_cache.rs`，新增 `render_product_virtual_geometry_extract_stays_out_of_pre_mesh_cache`。该产品守卫用带 advanced providers 的 WGPU 产品框架开启 `virtual_geometry` quality profile，提交同一实体的 Dynamic 可见性承载 mesh 与 authored VG extract 两帧，锁定 `last_virtual_geometry_payload_source == Authored` 和 `last_virtual_geometry_indirect_draw_count >= 1` 仍存在，同时 pending static command-cache candidate、pre-MeshDraw skipped draw/phase、cache hit/miss/rebuild 全为 0。状态锚为 `render_plan02_static_cache_virtual_geometry_residual_product_guard_static_passed_cargo_deferred_active_lanes`；scoped rustfmt/source-anchor/line-count 通过，focused Cargo 因本切片验证决策时其他 cargo/rustc lane 活跃而暂缓，不计新的 Cargo/WGPU/RenderDoc 通过。本记录不声明 mesh-level indirect draw buffer 接入完成。

2026-06-23 Plan 02 MD-M2 static command cache skinned residual product guard 已扩展 `zircon_runtime/src/graphics/tests/render_product_mesh_cache.rs`，新增 `render_product_static_skinned_mesh_stays_out_of_pre_mesh_cache`。该产品守卫注册最小 skinned mesh、root skeleton 和 pose，将 direct `RenderMeshSnapshot.mesh` 与 `RenderSkeletalPoseExtract` 送入两帧 `WgpuRenderFramework::submit_frame_extract(...)`，锁定 skinned/GPU-source draw 不会成为 pending static command-cache candidate，也不会产生 pre-MeshDraw skipped draw/phase 或 cache hit/miss/rebuild；skinned draw、skinned GPU-source candidate 与 dynamic command 仍存在，保留 residual path。状态锚为 `render_plan02_static_cache_skinned_residual_product_guard_static_passed_cargo_deferred_active_lanes`；scoped rustfmt/source-anchor/line-count/diff-check 通过，focused Cargo 因本切片验证决策时其他 cargo/rustc lane 活跃而暂缓，不计新的 Cargo/WGPU/RenderDoc 通过。

2026-06-23 Plan 02 MD-M2 static command cache transparent residual product guard 已扩展 `zircon_runtime/src/graphics/tests/render_product_mesh_cache.rs`，新增 `render_product_static_transparent_mesh_stays_out_of_pre_mesh_cache`。该产品守卫注册 `AlphaMode::Blend` 材质，并用 `GeometryExtract::from_meshes_and_phase_inputs(...)` 把 static mesh 放入 `Transparent3d` phase 连续提交两帧，锁定 transparent static mesh 不会成为 pending static command-cache candidate，也不会产生 pre-MeshDraw skipped draw/phase 或 cache hit/miss/rebuild；透明 draw 与 dynamic command 仍存在，保留每帧相机深度排序路径。状态锚为 `render_plan02_static_cache_transparent_residual_product_guard_static_passed_cargo_deferred_active_lanes`；scoped rustfmt/source-anchor/line-count 通过，focused Cargo 因本切片验证决策时其他 cargo/rustc lane 活跃而暂缓，不计新的 Cargo/WGPU/RenderDoc 通过。

2026-06-23 Plan 02 MD-M2 static command cache TAA reactive residual product guard 已扩展 `zircon_runtime/src/graphics/tests/render_product_mesh_cache.rs`，新增 `render_product_static_mesh_taa_reactive_mask_keeps_residual_mesh_draw_path`。该产品守卫用 static mesh + `taa_reactive_mask_strength = 1.0` 材质和 TAA/temporal history 两帧提交，锁定 ordinary static phases 仍可成为候选并在后续 command 层复用，但 reactive-mask 材质态不能在 pre-MeshDraw 阶段跳过 `MeshDraw` 构造；第二帧 pre-MeshDraw skipped draw/phase 为 0，cached hit 覆盖 ordinary phases，reactive-mask command 仍作为 dynamic command 存在。状态锚为 `render_plan02_static_cache_taa_reactive_residual_product_guard_static_passed_cargo_deferred_active_lanes`；scoped rustfmt/source-anchor/line-count 通过，focused Cargo 因本切片验证决策时其他 cargo/rustc lane 活跃而暂缓，不计新的 Cargo/WGPU/RenderDoc 通过。

2026-06-23 Plan 02 MD-M2 static command cache material revision product guard 已扩展 `zircon_runtime/src/graphics/tests/render_product_mesh_cache.rs`，新增 `render_product_static_mesh_material_revision_invalidates_pre_mesh_cache`。该产品守卫在两次 `WgpuRenderFramework::submit_frame_extract(...)` 之间用同一 material id/URI 和新的 `ResourceRecord::with_source_hash(...)` 推进 material revision，断言 pre-MeshDraw static cache 不会误 skip，material-bound residual、material invalidation 与 rebuild 计数可观测，transform/geometry invalidation 与 cache hit/miss 仍为 0。状态锚为 `render_plan02_static_cache_material_revision_product_guard_static_passed_cargo_deferred_active_lanes`；scoped rustfmt/source-anchor/line-count 通过，focused Cargo 因本切片验证决策时其他 cargo/rustc lane 活跃而暂缓，不计新的 Cargo/WGPU/RenderDoc 通过。

2026-06-23 Plan 02 MD-M2/MD-M4 virtual geometry mesh-level indirect buffers 已新增 `graphics/scene/scene_renderer/mesh/build_mesh_draws/build/virtual_geometry_indirect.rs` 作为 VG indirect child owner；`build_mesh_draws(...)` 现在从 `ViewportRenderFrame.virtual_geometry_debug_snapshot.execution_segments` 生成 per-draw WGPU indexed indirect args、submission、authority、draw-ref 和 segment buffers，并只给 VG execution segment 对应的 `MeshDraw` 携带 indirect args buffer/offset/detail。record/present submission 不再把 VG indirect segment 统计写死为 0，产品统计也在 executable segment 存在时报告 indirect buffer。`render_product_virtual_geometry_extract_stays_out_of_pre_mesh_cache` 已追加 indirect buffer/args/segment 非零断言。状态锚为 `render_plan02_virtual_geometry_mesh_indirect_buffers_static_passed_cargo_deferred_active_lanes`；scoped rustfmt/check、diff-check 和行数检查通过，验证决策时有 cargo/rustc lane 活跃，未声明新的 Cargo/WGPU/RenderDoc 通过。

2026-06-23 Plan 02 MD-M2 static command cache product stats guard 已新增 `zircon_runtime/src/graphics/tests/render_product_mesh_cache.rs`，在产品 `WgpuRenderFramework::submit_frame_extract(...)` 路径连续提交同一 eligible static mesh 两帧，首帧确认 pending static command-cache 候选，第二帧确认 `pre_mesh_draw_static_command_cache.skipped_*` 与 `cached_command_hit_count` 反映 pre-MeshDraw 复用且 miss/rebuild/residual 计数为 0。状态锚为 `render_plan02_static_cache_product_stats_guard_static_passed_cargo_timeout_no_result`；scoped rustfmt/source-anchor/line-count 通过，focused locked Cargo 在约 184s 超时无结果且匹配 target 的 cargo/rustc 已停止，不计新的 Cargo/WGPU/RenderDoc 通过。

2026-06-23 Plan 02 prepared queue stats bridge owner split 已把 `PreparedMeshQueueStats` 的跨系统统计桥接从 `graphics/scene/scene_renderer/mesh/prepared_queue.rs` 移入 `graphics/scene/scene_renderer/mesh/prepared_queue/stats_bridge.rs`。父文件降到 241 行，只保留队列统计字段、queue summarization 与 repeated-group helper；新 child owner 93 行，承接 pending command cache plan/extraction、mesh pass command buffer/replay、GPUScene upload stats forwarding。状态锚为 `render_plan02_prepared_queue_stats_bridge_owner_split_static_passed_cargo_timeout_no_result`；scoped static validation 通过，focused locked Cargo 180 秒超时无结果且无本 target 残留 cargo/rustc，不计新的 Cargo/WGPU/RenderDoc 通过。

2026-06-23 Plan 02 prepared queue stats bridge tests owner split 已把 stats forwarding 测试从近阈值 `prepared_queue/tests.rs` 拆入 `graphics/scene/scene_renderer/mesh/prepared_queue/stats_bridge_tests.rs`。`prepared_queue/tests.rs` 降到 599 行，只保留 queue behavior 与 fixture helpers；新 child owner 174 行覆盖 pending command cache plan/extraction、mesh pass command buffer/replay 和 GPUScene stats forwarding。状态锚为 `render_plan02_prepared_queue_stats_bridge_tests_owner_split_static_passed_cargo_lock_blocked`；scoped static validation 通过，focused locked Cargo 被当前 `Cargo.lock` 更新需求在编译前阻断，不计新的 Cargo/WGPU/RenderDoc 通过。

2026-06-23 Plan 02 MD-M2 residual fallback owner split 已把 pre-MeshDraw static command cache 抽取失败归因从 root extraction owner 拆入 `graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/residual_fallback.rs`。`pending_command_cache_extract.rs` 降到 294 行，继续只做提取流程、cache lookup/store 与 pending batch materialization；新 child owner 58 行承接 `PendingMeshCommandCacheResidualReason`、rebuild failure 到 `residual_*_draw_count` 的计数。状态锚为 `render_plan02_residual_fallback_owner_split_static_passed_cargo_lock_blocked`；scoped static validation 通过，focused locked Cargo 被当前 `Cargo.lock` 更新需求在编译前阻断；本切片不声明新的 Cargo/WGPU/RenderDoc 通过。

2026-06-23 Plan 02 MD-M2 pre-MeshDraw second-frame extraction guards 已新增 `graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/second_frame_tests.rs` 作为二帧/材质失效 focused owner。该 owner 锁定 full-hit 第二帧在 pre-MeshDraw 抽取层 `cached_command_hit_count == 3`、`command_rebuild_count == 0` 且不请求 rebuild batch，并锁定 shadow-only 材质 revision 改变时 opaque `ShadowDepth` 可在 `create_mesh_draw(...)` 前安全重建、记录 `cache_invalidated_material_count == 1`。状态锚为 `render_plan02_pre_mesh_draw_second_frame_extract_guards_static_passed_cargo_timeout_no_result`；scoped static validation 通过，focused locked Cargo 在 184s 工具窗口超时无结果且无本 target 残留 cargo/rustc，不计新的 Cargo/WGPU/RenderDoc 通过。

2026-06-23 Plan 02 MD-M2 pre-MeshDraw material-bound rebuild boundary guard 已新增 `zircon_runtime/src/tests/runtime_absorption/structure_convention/render_pending_command_cache_material_boundary.rs`。该结构守卫把 `non_material_rebuild.rs` 只允许 opaque `ShadowDepth` 的策略与 normal prepass、alpha-mask shadow、object velocity、TAA reactive mask replay 仍绑定 standard material 的事实绑定在一起，防止在 material bind group 构造仍晚于 pre-MeshDraw 抽取时误扩展 safe rebuild set。状态锚为 `render_plan02_pre_mesh_draw_material_boundary_guard_static_passed_cargo_lock_blocked`；scoped static validation 通过，focused locked Cargo 被当前 `Cargo.lock` 更新需求在编译前阻断，不计新的 Cargo/WGPU/RenderDoc 通过。

2026-06-23 Plan 02 MD-M2 pre-MeshDraw residual fallback diagnostics 已把 static command cache 抽取失败原因输出到产品诊断：material-bound phase miss/invalidated、shadow rebuild input 缺失、non-material rebuild 被拒绝分别进入 `render.mesh.queue.pre_mesh_draw_static_command_cache.residual_*_draw_count`。新增 `graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/fallback_tests.rs` 锁定三类原因；状态锚为 `render_plan02_pre_mesh_draw_residual_fallback_diagnostics_static_passed_cargo_lock_blocked`。scoped static validation 通过，focused locked Cargo 被当前 `Cargo.lock` 更新需求在编译前阻断；本切片只暴露 residual path 原因，不声明新的 Cargo/WGPU/RenderDoc 通过。

2026-06-23 Plan 02 MD-M2 visibility-pruned pre-MeshDraw diagnostics split 已把 visibility/relevance 全裁剪的零命令 skip 从普通 pre-MeshDraw skipped draw 中拆出独立诊断：`pending_command_cache_extract.rs` 为抽取结果新增 `visibility_pruned` 标记，`PendingMeshCommandCacheExtractionStats` 新增 `visibility_pruned_mesh_draw_count`，并通过 `PreparedMeshQueueStats`、`RenderStats`、`update_stats/base_stats.rs` 与 `product/mesh_queue.rs` 输出 `render.mesh.queue.pre_mesh_draw_static_command_cache.visibility_pruned_draw_count`。新增 `graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/visibility_tests.rs`，状态锚为 `render_plan02_visibility_pruned_pre_mesh_draw_diagnostics_static_passed_cargo_timeout_no_result`；focused locked lib-test compile 180 秒超时无结果，且无本 target 残留 cargo/rustc，不计 Cargo/WGPU/RenderDoc 通过。

2026-06-23 Plan 02 MD-M2 lazy pre-MeshDraw rebuild input 已把 `pending_command_cache_extract.rs` 的 rebuild batch 输入改为惰性物化：full-hit 静态 draw 和 material-bound phase miss 不再提前查询 GPUScene span/构造 `MeshBatchRef`，只有 opaque shadow miss/invalidated 这类 `non_material_rebuild` 允许的 phase 才进入 `pending_mesh_command_cache_rebuild_batch_for_phase(...)`。新增 `graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/lazy_rebuild_tests.rs`，状态锚为 `render_plan02_lazy_pre_mesh_draw_rebuild_input_static_passed_cargo_lock_blocked`，`runtime_15_pending_command_cache_plan_is_observable_before_mesh_draw_build` 同步锁定新 child owner 与 docs anchors；focused locked lib-test compile 被当前 `Cargo.lock` 更新需求阻断，不计 Cargo/WGPU/RenderDoc 通过。

2026-06-23 Plan 02 MD-M2 visibility-pruned pre-MeshDraw empty extraction 已把“visibility/relevance 全裁掉所有 cacheable phases”的直接 prepared static pending draw 收进 pre-MeshDraw skip path：`pending_command_cache_extract.rs` 返回空命令抽取结果并移除 residual `MeshDraw` 构造，不创建 material bind groups，也不请求 rebuild batch。状态锚为 `render_plan02_visibility_pruned_pre_mesh_draw_empty_extract_static_passed_cargo_lock_blocked`，focused guard 已由后续 diagnostics split 移入 `pending_command_cache_extract/visibility_tests.rs::pending_command_cache_extract_marks_visibility_pruned_static_draw`；focused locked lib-test compile 被当前 `Cargo.lock` 更新需求阻断，不计 Cargo/WGPU/RenderDoc 通过。

2026-06-23 Plan 02 MD-M2 pending command cache extract-item owner split 已把 `pending_command_cache_extract.rs` 的 item/eligibility/phase selection 移到 `graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/extract_item.rs`。父 owner 降到 255 行，继续只做抽取入口、cache lookup/store 与 rebuild dispatch；状态锚为 `render_plan02_pending_command_cache_extract_item_owner_split_static_passed_cargo_lock_blocked`，focused locked lib-test compile 被当前 `Cargo.lock` 更新需求阻断，不计 Cargo/WGPU/RenderDoc 通过。

2026-06-23 Plan 02 MD-M2 pre-MeshDraw opaque shadow cache rebuild 已在 full-hit extraction 后继续下沉 opaque shadow miss/invalidated：新增 `graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/non_material_rebuild.rs` 和 `graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/tests.rs`，让直接 prepared、非透明、非 skinned、无 reactive mask 的 static pending draw 在 `RenderPhase::Shadow` 为 opaque `ShadowDepth` 且其它 material-bound phases 已命中或被 visibility 裁掉时，可在 `create_mesh_draw(...)` 前重建并缓存命令。实现显式拒绝 depth prepass 与 alpha-mask shadow，因为现有 replay 会为这些路径绑定 standard material；状态锚为 `render_plan02_pre_mesh_draw_shadow_cache_rebuild_static_passed_cargo_timeout_no_result`，`runtime_15_pending_command_cache_plan_is_observable_before_mesh_draw_build` 同步锁定子 owner、source guards 与 docs anchors。

2026-06-23 Plan 02 MD-M2 pre-MeshDraw command cache extraction 已新增 `graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract.rs`，在 GPUScene 同步后、`create_mesh_draw(...)` 创建 WGPU material bind groups 之前，对直接 prepared、非透明、非 skinned、无 reactive mask 且 depth/shadow/opaque/alpha-mask cacheable phases 全命中的静态 pending draw 直接抽取 cached `MeshDrawCommand`。`BuiltMeshDraws` 保存 source prepared queue stats 与 prebuilt `MeshPassCommandBuffers`，compiled scene 将 prebuilt buffers 与 residual draw builder 输出合并，产品诊断新增 `render.mesh.queue.pre_mesh_draw_static_command_cache.skipped_*`；状态锚为 `render_plan02_pre_mesh_draw_command_cache_extraction_static_passed_cargo_lock_blocked`，`runtime_15_pending_command_cache_plan_is_observable_before_mesh_draw_build` 同步锁定 extraction owner、build hook、stats/diagnostics bridge 与 docs anchors。当前切片仍不覆盖 TAA reactive、skinned/GPU-skinning、indirect/VG、transparent 排序路径，且 material-bound phase miss 仍走 residual `MeshDraw` 构造。

2026-06-23 Plan 02 MD-M2 pending command cache plan diagnostics 已新增 `graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_plan.rs`，在 pending draw 尚未构造 `MeshDraw`/WGPU bind 资源前统计静态 command cache draw/phase 候选，并把 `pending_static_command_cache_*` 经 `BuiltMeshDraws`、`CompiledSceneDraws`、`PreparedMeshQueueStats`、`RenderStats` 与产品诊断输出。新增 `runtime_15_pending_command_cache_plan_is_observable_before_mesh_draw_build` 锁定 pre-MeshDraw owner、build 接入点、stats/diagnostics 桥接和 docs/status 锚点，状态锚为 `render_plan02_pending_command_cache_plan_static_passed_cargo_lock_blocked`；当前 scoped rustfmt/static/source-anchor/docs-anchor/diff-check 通过，locked Cargo 受当前 `Cargo.lock` 更新需求阻断，不计 Cargo/WGPU/RenderDoc 通过。该切片完成 MD-M2 下沉边界的可观察化，真实 skip path 已由后续 `render_plan02_pre_mesh_draw_command_cache_extraction_static_passed_cargo_lock_blocked` 开始覆盖。

2026-06-23 Plan 02 mesh draw command list owner split 已把 `graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs` 拆成 folder-backed owner：根文件保留 `MeshDrawCommandList`、`MeshPassCommandBuffers`、indirect batch stats 和排序/统计 helper 并降到 291 行，`graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/builder.rs` 承接 batch→command buffer 的 processor fan-out、静态 cache lookup/rebuild 与 dynamic command append，`graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/tests.rs` 承接原 inline 行为测试。新增 `runtime_15_mesh_draw_command_list_is_folder_backed` 锁定 owner 挂载、moved builder/test owner 不回流、docs/status 锚点和行数预算，状态锚为 `render_plan02_mesh_draw_command_list_owner_split_static_passed_cargo_lock_blocked`；当前 scoped rustfmt/static/line-count/docs-anchor/diff-check 通过，locked core-min Cargo check 被当前 `Cargo.lock` 更新需求在编译前阻断，不计 Cargo/WGPU/RenderDoc 通过。

2026-06-23 Plan 02 MD-M3 replay state-dedup focused tests 已把 `graphics/scene/scene_renderer/mesh/mesh_pass/replay.rs` 的 bind-group 与 geometry 状态判断抽成可测试纯 helper，并新增 focused tests 覆盖 pipeline change 计数、同 slot bind skip、pipeline 切换后 bind tracking reset、geometry 重复跳过与 pipeline 切换后重绑。状态锚为 `render_plan02_replay_state_dedup_focused_tests_static_passed_cargo_lock_blocked`；当前 rustfmt/source-anchor scan 通过，focused locked Cargo test 被当前 `Cargo.lock` 更新需求在编译前阻断，不计 Cargo/WGPU/RenderDoc 通过。

2026-06-23 Plan 02 mesh pass processor tests owner split 已把 `graphics/scene/scene_renderer/mesh/mesh_pass/processors/mod.rs` 收敛为 15 行 module declaration/re-export surface，并新增 `graphics/scene/scene_renderer/mesh/mesh_pass/processors/tests.rs` 承接 processor 行为测试和 Plan 02 focused guards。新增 `runtime_15_mesh_pass_processors_are_folder_backed` 锁定 processor root 不再承载测试/fixtures、docs/status 锚点和行数预算，状态锚为 `render_plan02_mesh_pass_processor_tests_owner_split_static_passed_cargo_lock_blocked`；当前 scoped rustfmt/source-anchor scan 通过，locked Cargo 受当前 `Cargo.lock` 更新需求阻断，不计 Cargo/WGPU/RenderDoc 通过。

2026-06-23 Plan 02 prepared queue tests owner split 已把 `graphics/scene/scene_renderer/mesh/prepared_queue.rs` 收敛为 272 行生产统计 owner，并新增 `graphics/scene/scene_renderer/mesh/prepared_queue/tests.rs` 承接 prepared queue 统计行为测试和 fixture helpers。新增 `runtime_15_prepared_mesh_queue_is_folder_backed` 锁定 parent 不再承载 inline tests、docs/status 锚点和父子行数预算，状态锚为 `render_plan02_prepared_queue_tests_owner_split_static_passed_cargo_lock_blocked`；当前 scoped rustfmt/static/source-anchor/docs-anchor/diff-check 通过，locked Cargo 受当前 `Cargo.lock` 更新需求阻断，不计 Cargo/WGPU/RenderDoc 通过。

2026-06-23 Plan 09 CO-M4 继续补齐 Volume mask 与 culling mask 分离:`RenderViewExtract::selected_camera_volume_layers()` 已成为 post-process Volume 评估读口,`scene/world/render_post_process.rs` 按 selected/stack camera `volume_mask` union 收集 Volume,`build_frame_submission_context(...)` 调 `resolved_settings_for_camera(...)` 时不再传 selected camera culling layers。新增 `scene/tests/render_post_process_extract.rs::explicit_request_camera_uses_volume_mask_for_post_process_volumes`,锁定显式 request camera `culling_mask` 与 `volume_mask` 不同仍按 volume mask 解析 bloom。状态锚为 `render_plan09_volume_mask_separate_from_culling_static_passed_cargo_lock_blocked_timeout_no_result`;当前只声明 rustfmt/static/line-count/diff-check 证据,focused Cargo 超时且 locked check 被 `Cargo.lock` 漂移阻断,不声明新的 WGPU/RenderDoc 通过。

2026-06-23 Plan 09 CO-M4 继续补齐 selected-camera history layer key:`ViewportCameraHistoryKey::from_camera(...)` 现在把 selected descriptor 的 `culling_mask` 与 `volume_mask` 纳入 per-camera history/runtime/product-report 槽位选择。私有 `ViewportCameraHistoryLayerKey` 通过 `RenderLayerSet::iter()` 记录 typed layer 列表,避免 layer 40/41 在 `to_legacy_mask_lossy()` 下归零后与 empty mask 共用 key。新增 `camera_history_key_distinguishes_culling_layers_without_legacy_loss` 与 `camera_history_key_distinguishes_volume_layers_without_legacy_loss`;状态锚为 `render_plan09_history_key_layer_masks_static_passed_cargo_lock_blocked`;当前只声明 rustfmt/static/line-count/diff-check 证据,focused Cargo 被当前 `Cargo.lock` 漂移和 `--locked` 在编译前阻断,不声明新的 WGPU/RenderDoc 通过。

2026-06-24 Plan 14 Screen-space UI render test owner split 已把 `graphics/scene/scene_renderer/ui/render.rs` 从 screen-space UI WGPU production owner + inline plan tests 的混合文件收束为 639 行 production owner，并新增 `graphics/scene/scene_renderer/ui/render/tests.rs` 承接 plan batch、SDF/native/auto text routing、resolved layout line、rich text run 与 text decoration tests。新增 `runtime_15_screen_space_ui_render_tests_are_child_owner_split` 锁定 moved tests 不回流、父/子 800 行预算与 docs/status 锚点；状态锚为 `render_plan14_screen_space_ui_render_test_owner_split_static_passed_cargo_deferred_active_compile_lane`。当前 scoped rustfmt/static scans、父子行数预算、docs-anchor、whitespace 和 diff-check 通过；Cargo/WGPU/RenderDoc 因 active compile lanes 暂缓,不计通过。

2026-06-24 Plan 02 VG debug snapshot stream types owner split 已把 `core/framework/render/virtual_geometry_debug_snapshot_streams.rs` 中的 stream DTO、decode error 与 summary 类型迁入 `core/framework/render/virtual_geometry_debug_snapshot_streams/types.rs`，父 owner 只保留 encode/decode orchestration、packing helpers 以及 diagnostics/metrics/types 子模块挂载。新增 `runtime_15_vg_debug_snapshot_stream_types_are_child_owner` 锁定 moved types 不回流、父/子 800 行预算和 docs/status 锚点；状态锚为 `render_plan02_vg_debug_snapshot_stream_types_owner_split_static_passed_cargo_deferred_active_compile_lane`。当前 scoped rustfmt/static scans、父子行数预算、docs-anchor、whitespace 和 diff-check 通过；Cargo/WGPU/RenderDoc 因 active compile lanes 暂缓,不计通过。

2026-06-24 Plan 04 HZB occlusion culler test owner split 已把 `graphics/scene/scene_renderer/hzb/hzb_occlusion_culler.rs` 的 inline WGPU/static tests 和 fixtures 移入 `graphics/scene/scene_renderer/hzb/hzb_occlusion_culler/tests.rs`，父 owner 保留 compute pipeline、bind group layout、dispatch/readback stats 和测试挂载。新增 `runtime_15_hzb_occlusion_culler_tests_are_child_owner` 锁定 moved tests 不回流、父/子 800 行预算和 docs/status 锚点；状态锚为 `render_plan04_hzb_occlusion_culler_test_owner_split_static_passed_cargo_deferred_active_compile_lane`。当前 scoped rustfmt/static scans、父子行数预算、docs-anchor、whitespace 和 diff-check 通过；Cargo/WGPU/RenderDoc 因 active compile lanes 暂缓,不计通过。

| 范围 | 记录位置 |
|---|---|
| Render 01-05 | `01-render-graph-rdg-alignment.md`、`02-mesh-draw-command-pipeline.md`、`03-gpu-scene-gpu-driven.md`、`04-visibility-culling.md`、`05-lighting-shadows.md` |
| Render 06-10 | `06-temporal-pipeline.md`、`07-postprocess-color-pipeline.md`、`08-material-shader-permutation.md`、`09-camera-render-ordering.md`、`10-renderer-family.md` |
| Render 11-15 | `11-environment-lighting.md`、`12-effects-particles.md`、`13-texture-pipeline.md`、`14-2d-stack.md`、`15-terrain-vegetation.md` |
| Render 16-19 | `16-compute-neural.md`、`17-performance-and-profiling.md`、`18-advanced-lighting-features.md`、`19-gpu-capability-optimizations.md` |

读取状态时以对应子计划状态表的最新记录、后续项和验证证据为准；跨计划优先级继续由上文子计划地图、能力矩阵和全局验收基线约束。
