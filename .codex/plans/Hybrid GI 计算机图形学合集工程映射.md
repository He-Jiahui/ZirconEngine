# Hybrid GI “计算机图形学”合集工程映射

## 1. 来源与使用规则

- 来源账号：内核观察。
- 合集：[计算机图形学（45 篇）](https://mp.weixin.qq.com/mp/appmsgalbum?__biz=MzkwNDAxMTAxOA==&action=getalbum&album_id=3075945039568273412)。
- 抓取状态：已索引 45/45 篇标题与原文链接，编号连续为 1..45，45 个 canonical 链接互不重复；用户给出的短链接 `https://mp.weixin.qq.com/s/yPkUrdYIwtWvgxH2cETMJA` 对应第 45 篇 `mid=2247490496`、`sn=72d1515470ad6f72b01b3e1c3ce57618`。文章内容只用于工程分类与交叉核对，仓库不保存文章全文、图片或付费内容副本。
- 证据等级：文章是设计线索，不是 Zircon 的正确性、性能或 API 权威。实施前必须回查当前代码、`dev/LumenInUE5.5.4WithComputeShader`、官方 WGPU/WebGPU 文档、论文或 SDK。
- 版权边界：本文件只保留标题、链接、简短工程归纳和计划归属。

## 2. 45 篇目录与工程归属

| # | 文章 | 分类 | 对 Zircon 的工程用途 | 计划归属 |
| --- | --- | --- | --- | --- |
| 1 | [计算机图形学基础（译）- 2.5.2 二维上的梯度](https://mp.weixin.qq.com/s?__biz=MzkwNDAxMTAxOA==&mid=2247484368&idx=1&sn=25c846eebe82a425d7d20d66ce733b32) | 数学基础 | 梯度、隐式函数与 shader 数值测试背景，不直接产生功能承诺 | Shader/Plan 06 |
| 2 | [计算机图形学基础（译）- 2.5.2 二维上的梯度（中）Implicit 2D Lines](https://mp.weixin.qq.com/s?__biz=MzkwNDAxMTAxOA==&mid=2247484372&idx=1&sn=02b31fa3678f762f38565a282c33fb7a) | 数学基础 | 隐式线、梯度方向与屏幕空间边界测试背景 | Shader/Plan 06 |
| 3 | [Texel、Pixel和Fragment的关系](https://mp.weixin.qq.com/s?__biz=MzkwNDAxMTAxOA==&mid=2247484492&idx=1&sn=0582d0b28e0eee52bd04a8d99442b8df) | 光栅基础 | 统一文档术语，约束 sample/fragment/texel 与 MSAA resolve 语义 | Plan 01/05/06 |
| 4 | [Metal程序设计-多重采样抗锯齿](https://mp.weixin.qq.com/s?__biz=MzkwNDAxMTAxOA==&mid=2247484516&idx=1&sn=a7a3276af37af43a41aedc1a98782162) | MSAA | 提醒 HGI 的 depth/normal 必须与 conservative sample 选择一致 | Plan 01/05/06 + HGI-M3 |
| 5 | [渲染基础更新-Direct3D12 简介](https://mp.weixin.qq.com/s?__biz=MzkwNDAxMTAxOA==&mid=2247484571&idx=2&sn=0ca07fad95e04f0e75441d938653a5af) | RHI | 资源状态与显式提交仅作跨 API 对照，不泄漏到 WGPU 公共合同 | Plan 01/02/19 |
| 6 | [Direct3D 12单线程双缓冲渲染框架](https://mp.weixin.qq.com/s?__biz=MzkwNDAxMTAxOA==&mid=2247484589&idx=1&sn=3d4a8e7e926daaaab944f5ac5d1a3fce) | 帧调度 | 对照 frame resources、双缓冲和同步；HGI history 仍服从 RenderGraph 所有权 | Plan 01/02 |
| 7 | [Metal平台-RT、Material、StaticMesh、SkinedMesh、RenderPass](https://mp.weixin.qq.com/s?__biz=MzkwNDAxMTAxOA==&mid=2247484601&idx=1&sn=2229db20e681a9ca1d5439281576b784) | 渲染对象 | 用于核对资源、材质、网格和 pass 的职责拆分 | Plan 02/08/13 |
| 8 | [在线模型转换工具-推进中](https://mp.weixin.qq.com/s?__biz=MzkwNDAxMTAxOA==&mid=2247484606&idx=1&sn=f59129524edd3b3a394a0b02b644f090) | 资产工具 | SDF/card 数据应进入版本化导入产物，不在首帧同步生成 | 资产导入 + HGI-M5 |
| 9 | [大小端编码 - JS编码了一个int，到了C++里读不出来？](https://mp.weixin.qq.com/s?__biz=MzkwNDAxMTAxOA==&mid=2247484619&idx=1&sn=4d1891ca2903b8984dae9495d65d9210) | GPU ABI | 强化序列化、字节序、对齐和跨语言 buffer round-trip 测试 | Plan 03/08/13 |
| 10 | [卡通渲染 - 游戏渲染大全](https://mp.weixin.qq.com/s?__biz=MzkwNDAxMTAxOA==&mid=2247484634&idx=1&sn=cd2d515515539f04ad60d60c0d0600e9) | 风格化渲染 | Toon lighting/material model，不并入 HGI 核心 | Plan 08/12 |
| 11 | [卡渲描边碰到的问题之二 - 游戏渲染大全之卡通渲染](https://mp.weixin.qq.com/s?__biz=MzkwNDAxMTAxOA==&mid=2247484653&idx=1&sn=cf8f7e16a52f794e63925bfe7aa6eb8e) | 描边 | Outline 的法线/深度边界可复用 GBuffer，但由风格化后处理拥有 | Plan 08/12 |
| 12 | [PBR(概率密度函数) - 之我在东汉书院搞Lumen](https://mp.weixin.qq.com/s?__biz=MzkwNDAxMTAxOA==&mid=2247485427&idx=1&sn=c770b91b1bc45cdfb8ac33f045529613) | 采样理论 | 为 probe ray PDF、重要性采样和能量无偏测试提供线索 | HGI-M5 + Shader/Plan 06 |
| 13 | [Lumen的教学工具打造完毕 - 我更加有信心让萌新小白都能学明白了！](https://mp.weixin.qq.com/s?__biz=MzkwNDAxMTAxOA==&mid=2247490071&idx=1&sn=187981f26e2fa836ca8fc609279ffd83) | 可视化 | 推动 cards/cache/probe/trace source/debug view 与编辑器诊断 | HGI-M6 |
| 14 | [四大图形API深度对决！OpenGL/Vulkan/WebGPU/DX12到底谁才是渲染天花板](https://mp.weixin.qq.com/s?__biz=MzkwNDAxMTAxOA==&mid=2247490079&idx=1&sn=3c39b7de787ac3ac83d9d84d102cf608) | API 对比 | 仅作为 capability 调研入口，最终以官方规范和实测为准 | Plan 19 |
| 15 | [OpenGL调试三剑客：RenderDoc / Nsight / APITrace 全方位对比，渲染BUG与性能瓶颈一键定位](https://mp.weixin.qq.com/s?__biz=MzkwNDAxMTAxOA==&mid=2247490084&idx=1&sn=992ad320460f6fa31a33fc01dfe8f490) | 调试工具 | 将 RenderDoc `.rdc`、资源命名和 pass 顺序纳入完成门 | HGI-M3/M4/M5/M6 |
| 16 | [全图形API的核心差异对比 - 值得引擎程序员一生典藏](https://mp.weixin.qq.com/s?__biz=MzkwNDAxMTAxOA==&mid=2247490102&idx=1&sn=f0801c44943cf9589b499cab97b931db) | API 对比 | 维护 WGPU capability 与 native backend 差异表 | Plan 19 |
| 17 | [OpenGL/Vulkan/D3D/Metal/WebGPU全方位横评](https://mp.weixin.qq.com/s?__biz=MzkwNDAxMTAxOA==&mid=2247490107&idx=1&sn=7b246cb2e4c74c1f5e207ca121e9b200) | API 对比 | 防止把桌面 API 的格式、同步和 RT 假设带入 WebGPU | Plan 19 + HGI-M6 |
| 18 | [Vulkan里最快的CPU与GPU的数据传输方式](https://mp.weixin.qq.com/s?__biz=MzkwNDAxMTAxOA==&mid=2247490123&idx=1&sn=b3664350a9c173937452c97ec072f831) | 数据上传 | 映射到 WGPU uniform/storage/write_buffer/ring-buffer 分级策略 | Plan 03/19 |
| 19 | [(Lumen基础)DistanceFieldObjectBounds](https://mp.weixin.qq.com/s?__biz=MzkwNDAxMTAxOA==&mid=2247490130&idx=1&sn=c93f06ba2b4917c65b15d5be62a851bc) | SDF | 规划 object bounds、mobility/visibility/emissive flags 和 influence culling | HGI-M5-S1 |
| 20 | [(Lumen基础)Clipmap.TranslatedWorldCenter - 细品虚幻源码](https://mp.weixin.qq.com/s?__biz=MzkwNDAxMTAxOA==&mid=2247490134&idx=1&sn=fceb1057bef07e583cfef6250c1258d7) | Clipmap | 相机中心按 cell/page 稳定取整，减少时间抖动和无效更新 | HGI-M5-S3 |
| 21 | [(7月19日开)SIGGRAPH 2026前沿技术报道](https://mp.weixin.qq.com/s?__biz=MzkwNDAxMTAxOA==&mid=2247490152&idx=1&sn=bb444d094e032788fdf892b9a2c9ff4f) | 未来研究 | 只在会议论文公开后复核，不提前转成功能承诺 | Plan 16 研究队列 |
| 22 | [避坑指南｜从桌面图形API转Web，近期踩遍的血泪坑](https://mp.weixin.qq.com/s?__biz=MzkwNDAxMTAxOA==&mid=2247490177&idx=1&sn=ef9bec1a916868aa7b8d73caaa0c37ab) | WebGPU | 补齐 format/filter/renderability/storage/readback/mip 严格验证与降级测试 | Plan 19 + HGI-M6 |
| 23 | [从零手搓Lumen，我们要开始最难的一部分教学内容了](https://mp.weixin.qq.com/s?__biz=MzkwNDAxMTAxOA==&mid=2247490206&idx=1&sn=65714d81c2e4fa202e2913380f410066) | Lumen/SDF | 将 mesh voxelization、SDF asset、global SDF 与 viewport 更新拆成独立切片 | HGI-M5 |
| 24 | [Vulkan生态爆发——从"没人用"到"全平台通吃"，底层API的胜利](https://mp.weixin.qq.com/s?__biz=MzkwNDAxMTAxOA==&mid=2247490217&idx=1&sn=96eced4c04c9d19c6e9cddc342e1805c) | 平台生态 | 平台选型背景，不作为 WGPU/HGI 架构变更依据 | RHI/平台计划 |
| 25 | [NeRF 完整学习指南：从基础到落地](https://mp.weixin.qq.com/s?__biz=MzkwNDAxMTAxOA==&mid=2247490228&idx=1&sn=a7e1d1c240ffb9a8698729f4e1f8019b) | 神经场 | 研究性场景表示，不进入 HGI V1/V1.1 | Plan 16 研究队列 |
| 26 | [图形学领域最重要的是什么？不是表面花里胡哨的东西，而是底层的成像原理](https://mp.weixin.qq.com/s?__biz=MzkwNDAxMTAxOA==&mid=2247490272&idx=1&sn=a0369345e1e31b69f0c9c62ba26b4072) | 成像基础 | 强化能量守恒、BRDF/PDF 和可测量验收，不产生独立功能 | 渲染总则 |
| 27 | [图形渲染领域的AI](https://mp.weixin.qq.com/s?__biz=MzkwNDAxMTAxOA==&mid=2247490277&idx=1&sn=d3e8e76d7ca8dc3e1e229ff352802997) | 神经渲染 | 仅做未来调研，需官方论文、训练数据与许可证复核 | Plan 16 研究队列 |
| 28 | [DLSS 5全栈神经实时渲染：AI 重构传统光追管线](https://mp.weixin.qq.com/s?__biz=MzkwNDAxMTAxOA==&mid=2247490288&idx=1&sn=60e6b6ba3a1b98d660450d90f7b70f4b) | 神经渲染 | 文章中的版本和性能主张不采信；只保留官方材料复核任务 | Plan 16 研究队列 |
| 29 | [下一代渲染核心：光栅化+硬件光追，什么是混合渲染管线？](https://mp.weixin.qq.com/s?__biz=MzkwNDAxMTAxOA==&mid=2247490314&idx=1&sn=0f6c175162a8d1b411e2e0dfc6ce8a1a) | 混合渲染 | 光栅基线 + 可选高成本 trace；WGPU V1 仍以 compute tracing 成立 | Render Plan 20 + HGI-M5 |
| 30 | [(非理论、工程化视角)一文看懂Lumen的全流程(让AI写小作文，我们就当个审稿人)](https://mp.weixin.qq.com/s?__biz=MzkwNDAxMTAxOA==&mid=2247490332&idx=1&sn=2d56de6957ada2423b929430756b43a0) | Lumen 流程 | 用作 cards/cache/probes/radiance cache/trace/composite 顺序检查表，逐项回查本地源码 | HGI-M1..M6 |
| 31 | [社招/应届，图形学岗只写一句话，必拿offer](https://mp.weixin.qq.com/s?__biz=MzkwNDAxMTAxOA==&mid=2247490344&idx=1&sn=db7861a7790561307877ead333a7b127) | 工程表达 | 只保留“可运行、可测量、可解释”的交付原则，不产生验收捷径 | 计划治理 |
| 32 | [从零手搓Lumen - 不写装逼小作文、只做让你能装逼的教程](https://mp.weixin.qq.com/s?__biz=MzkwNDAxMTAxOA==&mid=2247490401&idx=1&sn=36ab60417f97ac82861840ad59a258ce) | Lumen 实践 | 将实现拆为 scene data、cache、probe、trace、composite 和 debug 可运行切片 | HGI-M1..M6 |
| 33 | [从零手搓Lumen - 东汉书院朴实无华版思路整理，后续我要改用rust了](https://mp.weixin.qq.com/s?__biz=MzkwNDAxMTAxOA==&mid=2247490405&idx=1&sn=a0887ef511758148eadacc01748879d9) | Lumen 实践 | 核对 clipmap、surface lighting、screen probe、radiance cache 和分层追踪依赖顺序 | HGI-M1..M5 |
| 34 | [手搓lumen教学将于本月落地到游戏引擎开发实践-东汉书院近期预告](https://mp.weixin.qq.com/s?__biz=MzkwNDAxMTAxOA==&mid=2247490412&idx=1&sn=7f87b77ab961b7352075e6d36a29e521) | Lumen 实践 | 强化 mesh -> voxel/SDF -> global scene 与 probe history/cache indirection 的数据所有权 | HGI-M5 |
| 35 | [烘焙+Lumen混合光照工作流 - 来自拥有0.01秒虚幻开发经验的工程师的告白](https://mp.weixin.qq.com/s?__biz=MzkwNDAxMTAxOA==&mid=2247490435&idx=1&sn=8aa69c2099640d5ee5be3e70ef4a38e2) | 混合 GI | 静态 baked baseline + 动态 HGI delta、避免 double count、按场景提供 profile | Plan 11 EL-M3 + HGI-M4 |
| 36 | [从零手搓Lumen - 掌握Lumen原理，才能知道如何优化Lumen性能](https://mp.weixin.qq.com/s?__biz=MzkwNDAxMTAxOA==&mid=2247490443&idx=1&sn=56b26f679d2b0371afe0cee1b6888004) | Lumen 性能 | 推动静态 Surface/Radiance Cache 预热、增量更新和首帧预算；文章中的百分比性能数字不作为门槛 | HGI-M5-RC + Render Plan 17 |
| 37 | [UE5 Nanite是纯虚幻原创？真相颠覆0.1%的开发者认知，因为另外的99.9%根本不知道我们在说什么](https://mp.weixin.qq.com/s?__biz=MzkwNDAxMTAxOA==&mid=2247490447&idx=1&sn=d1b35be86bae309551e578fadef74a0a) | 虚拟几何 | 强化离线 cluster/cook 与运行时 stream/cull 的拆分；历史归因和宣传数字不进入架构依据 | VG N1/N5 + 资产导入 |
| 38 | [通过推进Lumen课程居然解决了一个BUG](https://mp.weixin.qq.com/s?__biz=MzkwNDAxMTAxOA==&mid=2247490454&idx=1&sn=1849d839e631457dc3f958951b537c92) | Lumen 回归 | 将 BRDF/PDF、命中光照和能量 ledger 的负向对拍纳入 shader/HGI 回归，具体 bug 需本地复现后才承认 | Shader/Plan 06 + HGI 验证 |
| 39 | [从零手搓Lumen - Lumen没有黑科技，只是基础渲染的组装](https://mp.weixin.qq.com/s?__biz=MzkwNDAxMTAxOA==&mid=2247490461&idx=1&sn=ee0818ef5c8eef776ebff023f2c85bc6) | Lumen 基础 | 用作 scene representation、cache、probe、trace、denoise、composite 依赖复核，不新增并行子系统 | HGI-M1..M6 |
| 40 | [最后 3 天优惠｜Lumen 教程收尾，5套2026 图形新课直接免费领](https://mp.weixin.qq.com/s?__biz=MzkwNDAxMTAxOA==&mid=2247490465&idx=1&sn=ced6ae975325c8c4c416e21d49721ea3) | 课程推广 | 不产生工程承诺、完成度或性能验收 | 无 |
| 41 | [DXR与Vulkan光线追踪异同](https://mp.weixin.qq.com/s?__biz=MzkwNDAxMTAxOA==&mid=2247490477&idx=1&sn=7618ad7b92df2ea29b9cfd2247755e23) | RT API 对比 | 提炼 AS、RayQuery、ray pipeline/SBT 的中立合同；stage、limit 和同步必须回查官方规范 | Render Plan 20 |
| 42 | [混合渲染管线：光栅化+光追，什么时候该用谁？](https://mp.weixin.qq.com/s?__biz=MzkwNDAxMTAxOA==&mid=2247490482&idx=1&sn=f025954b5f7d1e7651851af380f14630) | 混合渲染 | 把 Raster/Compute/InlineQuery/RayPipeline 写成 feature 有序候选与确定性 fallback，而非按后端名称分支 | Render Plan 20 + Plan 05/07/18/HGI |
| 43 | [生活在AI时代 - 去关注非AI的事情](https://mp.weixin.qq.com/s?__biz=MzkwNDAxMTAxOA==&mid=2247490486&idx=1&sn=b50560fb7582d6090c21cfc06ba8cdcb) | 随笔 | 不产生渲染功能、进度或技术选型承诺 | 无 |
| 44 | [Vulkan 1.4+VK_EXT_descriptor_heap深度解析](https://mp.weixin.qq.com/s?__biz=MzkwNDAxMTAxOA==&mid=2247490492&idx=1&sn=59b70df1dce3c52fdc462d7e664c16ea) | 描述符/RHI | 作为未来 Vulkan binding adapter 研究项；该能力是可选 EXT,不是 Zircon 或 Vulkan 1.4 无条件基线 | Render Plan 19/20 |
| 45 | [自研渲染必看！混合光栅+光追管线](https://mp.weixin.qq.com/s?__biz=MzkwNDAxMTAxOA==&mid=2247490496&idx=1&sn=72d1515470ad6f72b01b3e1c3ce57618) | 统一 RHI | 采纳 shared mesh buffer、BLAS/TLAS 生命周期、GBuffer 复用和混合调度；当前仍以 wgpu 为主,原生后端只定义未来映射 | Render Plan 20 |

## 3. 已采纳的计划优化

- 新增 HGI-M4，明确 baked/static 与 dynamic/HGI 的所有权、参与策略、失效和 composite 防重复计能量规则。
- 将 HGI-M5 的单一顺序改为 `trace domain / intersection backend / hit-lighting source` 三层 capability graph；Mesh/Global SDF、Voxel 和可选硬件 RT 是求交后端，Surface/Radiance/Baked Cache 是命中光照源。
- 新增 HGI-M5-RC，独立覆盖 Radiance Cache demand marking、clipmap、indirection/history、free-list、淘汰、trace/filter/border/mip 生命周期，禁止与 Global SDF Clipmap 混为一个资源。
- 新增 HGI-M6，覆盖产品 profile、debug views、stats、WebGPU capability matrix 和 RenderDoc 完成门。
- 将 Indoor Static、Open World、Cinematic、Fully Dynamic 作为使用 profile，而不是四套独立算法。
- 将文章 35 的 baked + dynamic 建议收束为逐 `surface + lobe + light_set_generation + participation_epoch` 的 source ledger；无法可靠扣除旧 bounce 时必须二选一或降级，禁止简单相加。
- 新增 Render Plan 20,将文章 29/41/42/44/45 的混合渲染线索收束为 `设备实际能力 + 项目策略 + feature 候选 -> compiled selection`；AS、inline RayQuery 和完整 ray pipeline 分别启停,上层禁止按 API 名称分支。
- 保持 `wgpu` 为当前主后端；DX12/Vulkan/Metal 只记录 future adapter mapping。`VK_EXT_descriptor_heap` 仅在扩展存在且驱动验证通过时可选,不作为 Vulkan 1.4 基线。
- 文章 36 的 cache 预热归 HGI/资产/性能计划,文章 37 的 cook/stream 边界归 Virtual Geometry；二者不扩张 Plan 20 所有权。
- 文章 30 中“Radiance Cache 只来自动态 LightMap”“所有层最终简单累加”等简化说法已明确否决；详细审查矩阵由 Hybrid GI 主计划拥有。
- 将文章中的 AI、DLSS、NeRF 和未来会议信息降级为研究线索，不进入当前完成百分比或验收声明。

## 4. 本地参考实现核对点

- Surface Cache：`CompressToSurfaceCacheAlbedo/Normal/Emissive`、`CopyToSurfaceCacheDepth/Opacity`。
- Screen Probe：`UniformPlacementScreenProbe`、`AdapativePlacementProbe*`、`TraceScreen`、`CompactTraces`、`CompositeTraces`、`ScreenProbeToIrradiance`。
- Radiance Cache：`MarkRadianceProbes`、`AllocateUsedProbes`、`AllocateProbeTraces`、`TraceFromProbes`、`FilterProbeRadiance`、`UpdateCacheForUsedProbes` 和 indirection texture clear。
- Miss fallback：`TraceVoxels` 与仓库 `Res/Data/GlobalDistanceField.*` 只证明参考工程的数据分层；Zircon 的 Mesh/Global SDF 必须按自身 WGPU 资源合同重建，不能直接复制 HLSL/数据布局。

## 5. 完成状态

以下记录保留各日期当时的 35 篇目录与实现证据,不回写历史语义。当前合集身份事实以本文第 1/2 节的 45/45 为准;新增 36..45 的本轮工作仅为调研映射与计划边界完善,不代表相应运行时代码已完成。

| 日期 | 状态 | 完成项目 | 未完成项目 |
| --- | --- | --- | --- |
| 2026-07-14 | `collection_35_of_35_m4_editor_actual_fallback_evidence_complete` | 35/35 目录身份、1..35 连续编号、35 个唯一 canonical 链接与工程分类保持完整；第 35 篇的 `mid/sn` 与用户链接一致。文章 35 的 baked baseline + dynamic delta 已落为 Plan 11 只读合同、surface participation、per-probe source ledger、generation/epoch 失效和四个产品 Profile，未采用“完整 baked bounce 与完整 dynamic bounce 直接相加”。四 Profile Forward+/Deferred WGPU、mobility/Emissive 往返、8 份 RenderDoc DX12 捕获、standalone build、124 项 crate behavior 以及真实 Editor Custom actual / IndoorStatic missing-bake fallback 两张 PNG 均已有证据；仓库未复制文章正文。 | Editor actual/fallback 子门关闭，broad/full runtime/workspace 验证仍开放；M5-RC、Mesh/Global SDF 分层追踪和 M6 长时间性能治理尚未开始。文章中的 AI/DLSS/NeRF、未来会议信息和经验性能数字继续不计入完成度。 |
| 2026-07-13 | `collection_35_of_35_identity_verified_and_engine_claims_reconciled` | 35 个文章身份字段逐项复核无缺号、重复或差异；文章 30/33/34/35 已回查本地 Lumen/Unreal 参考并写入 capability graph、Radiance Cache 生命周期和 dynamic-delta ledger；Plan 11 EL-M3 已开始落地版本化 CPU lightmap/probe/bake 消费合同 | EL-M3 GPU binding/双管线采样/真实 fixture、HGI-M4 composite 与 M5-RC 运行时代码仍未完成；文章中的性能结论未被采纳为门槛 |
| 2026-07-12 | `collection_35_of_35_indexed_and_hgi_plan_reconciled` | 35 篇目录全部建立原文链接与工程归属；Hybrid GI 主计划完成现状校准、M4-M6、产品 profile、证据等级和非目标约束 | M4-M6 仍是计划项；本次没有申领对应运行时代码、截图或性能完成 |
