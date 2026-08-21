---
related_code:
  - zircon_runtime/crates/zr_rhi
  - zircon_runtime/crates/zr_rhi_wgpu
  - zircon_runtime/src/core/framework/render/backend_types/capability.rs
  - zircon_runtime/src/core/framework/render/solari
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/ray_tracing.rs
  - zircon_runtime/src/graphics/runtime/render_framework/capability_summary
  - zircon_runtime/src/graphics/solari_runtime_provider
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/advanced_rendering.rs
  - zircon_plugins/rendering/features/ray_tracing_policy
  - zircon_plugins/solari
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/scene_representation/trace_capability_graph
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/gpu_resources/execute_prepare/execute/dispatch_probe_trace_tiles.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/shaders/trace_probe_tiles.wgsl
  - zircon_plugins/sound/runtime/src/ray_tracing
tests:
  - zircon_runtime/crates/zr_rhi/src/tests
  - zircon_runtime/crates/zr_rhi_wgpu/src/tests/capabilities.rs
  - zircon_runtime/src/graphics/tests/advanced_followup_slots.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile/validation_core.rs
  - zircon_runtime/src/graphics/tests/render_framework_bridge/advanced_providers.rs
  - zircon_runtime/src/graphics/tests/render_product_solari.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/scene_representation/trace_capability_graph/tests.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_runtime/09d-render-asset-streaming-residency-review.md
  - docs/plans/optimize/zircon_runtime/09f3-hybrid-global-illumination-review.md
  - docs/plans/optimize/zircon_runtime/23-coordinate-space-unit-precision-transform-numeric-robustness-large-world-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_editor/22-render-pipeline-frame-capture-lighting-bake-reflection-probe-post-process-debug-authoring-review.md
  - docs/plans/optimize/zircon_plugins/04-rendering-umbrella-feature-bundles-solari-native-provider-product-integration-review.md
reference_engines:
  - dev/bevy/crates/bevy_solari/src
  - dev/UnrealEngine/Engine/Source/Runtime/RHI/Public/RHIResources.h
  - dev/UnrealEngine/Engine/Source/Runtime/RHI/Public/DynamicRHI.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/Rendering/RayTracingGeometryManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Rendering/RayTracingGeometryManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/RayTracing
  - dev/UnrealEngine/Engine/Shaders/Private/RayTracing
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/UnifiedRayTracing
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/RenderPipeline/Raytracing
  - dev/godot/modules/lightmapper_rd
  - dev/Fyrox/fyrox-impl/src/scene/accel.rs
  - dev/Fyrox/fyrox-impl/src/utils/lightmap.rs
---

# 28 · Hardware Ray Tracing、BLAS/TLAS、Ray Query/Pipeline、SBT、Denoising、Scalability 与 Product Integration 工程化差距

## 1. 结论

Zircon 当前没有硬件光追实现。现有内容由四类声明组成：`AccelerationStructureCaps` 的四个字段、内建 `ray_tracing` 零 pass feature、插件 `ray_tracing_policy` 零 pass feature，以及始终返回 `Unavailable` 的 Solari provider。`RenderDevice` 没有 acceleration-structure handle/descriptor/create/build/update/compact API，`CommandListCommand` 没有 AS build 或 ray dispatch，pipeline 只有 Raster/Compute，shader stage 也没有 ray generation、miss、closest-hit、any-hit、intersection 或 callable。WGPU 后端无条件返回 `AccelerationStructureCaps::disabled()`，仓库内也没有第二个可执行图形后端。

因此，当前差距不是“Solari 还差一个 pass”，而是整个 `scene geometry -> BLAS -> TLAS -> query/pipeline -> hit material -> denoise -> effect output -> evidence` 产品链尚未建立。Hybrid GI 的 `HardwareRayTracing` route 只编码为 `1 << 3` bitmask；实际 WGSL 只绑定 surface cache、Global SDF 与 voxel 数据，没有 acceleration structure binding。Sound 的 `RayTraced` 状态则接收已经生成的 impulse-response samples，并不执行几何射线追踪。两者都不能作为通用硬件 RT 后端存在的证据。

本篇登记 **0 P0 / 56 P1 / 14 P2**。0 P0 不是完成度认可：Solari 明确标记 Experimental/Partial/Unavailable，WGPU fail-close，默认与 advanced profile 均不请求它，当前没有把缺失实现交付成可用产品。若任何 profile、插件、Hub/Editor UI 或 capability receipt 把零 pass feature、bitmask route、测试注入的布尔值或外部声学样本宣称为“Hardware RT Ready/Executed”，应立即升级为 P0。后续必须按 `RtCapabilityProfile -> RtSceneCompiler -> BLAS/TLAS stores -> RayQuery/RayPipeline+SBT -> effect gateway -> denoiser/history -> scalability/evidence` 硬切，不能继续给枚举补字段来模拟进展。

## 2. 审查边界、方法与 currentness

### 2.1 冻结输入

本篇冻结 116 个输入、48,319 行、1,856,680 bytes：53 个 Zircon production 输入为 8,418 行、277,321 bytes；8 个 Zircon test/product 输入为 1,763 行、63,174 bytes；55 个参考实现输入为 38,138 行、1,516,185 bytes。组合指纹按相对路径排序，对每个文件计算 SHA-256，再对 `path<TAB>hash` 的 LF 拼接文本计算 SHA-256，结果为 `bb811f12e85721313d27747aea759785e423e3eae8d834e88f1480018b3f67a4`。

冻结基线为 `main@25e09a23178000f2e783ce2143cf70a8b118d404`。所选 Zircon 输入中 `solari/status.rs` 因工作区 CRLF 被 Git 标为 modified，但 working blob 与 HEAD blob 均为 `a2ebade0a16b11682e3902e0a22d46646f87a61f`，没有语义 diff；其余所选输入无工作区修改。`dev/` 参考源码不纳入 Git dirty 判定。

### 2.2 纵向生产链

本轮逐层核对：backend feature negotiation -> framework capability projection -> feature/profile compile -> RHI resource/pipeline/command surface -> scene/mesh/material extraction -> BLAS build/update/compaction -> TLAS instances -> inline query/ray pipeline/SBT -> Hybrid GI/Solari/Sound consumers -> denoise/history -> quality/degrade -> Editor descriptor -> product tests/diagnostics。搜索同时覆盖 `create/build/update/compact acceleration structure`、BLAS/TLAS、ray query、trace ray、SBT 与 shader-binding-table 等同义入口；Zircon production 未发现被 facade 漏出的实现。

### 2.3 证据等级与未覆盖范围

本轮达到 E3 source-level review：当前 production/test/product 链与五套参考源码均已交叉核对。未修改 production，也未运行需要真实 RT adapter 的 GPU 测试，因为当前 RHI 没有可调用的 RT API、WGPU 后端明确关闭能力，运行普通 raster 产品测试不能增加结论强度。离线 light bake 的完整产品流程由独立 lighting-bake owner 管理；本篇只吸收 Godot/Fyrox 的软件/CPU tracing 作为 fallback、geometry preparation 与可验证参考路径。

## 3. 当前可保留的工程基础

1. `AccelerationStructureCaps` 已把 AS、Inline Query、Ray Pipeline 与 max instance count 分开，说明架构没有把所有 RT 能力压成一个总布尔值；它适合作为新 capability profile 的迁移入口。
2. WGPU 后端无条件 disabled，Solari provider 明确返回 `Unavailable` 且 package 标记 Experimental/Partial；当前 fail-close 比伪执行正确，应保留这种产品诚实性。
3. `RayTracingPolicyReport` 能按用户请求的 InlineQuery/Pipeline 分别报告缺失 gate；可迁移为 compiled path selection，但不应继续独立维护另一套 capability truth。
4. Solari report 能区分 NotRequested、CapabilityMissing、ProviderMissing、ExperimentalDisabled 与 Unavailable；这些状态可扩展为真实 runtime receipt。
5. Render feature compiler 已有 capability mismatch 类型，Runtime09A/09C/09D 已分别规划通用 GPU lifetime、shader artifact 与 residency；RT 无需另造平行基础设施。
6. Hybrid GI 已有明确的软件 trace route、fallback reason 与诊断 payload；未来硬件 route 应作为同一 effect gateway 的一个真实 executor，而不是删除现有软件路径。
7. 现有产品测试证明默认 profile 不会误请求 Solari，并验证 unavailable/provider/capability 分支；应保留并新增真实执行证据，而不是把这些状态测试当作最终产品测试。

## 4. 参考实现给出的工程边界

### 4.1 Bevy：仓库内最近的 Solari 对照已经是真实 GPU 产品链

Bevy `bevy_solari` 不只是 capability 名称。`RaytracingScenePlugin` 请求 `EXPERIMENTAL_RAY_QUERY` 与 bindless features，把 mesh buffer 增加 `BLAS_INPUT` usage；`blas.rs` 对新增/修改/删除 mesh 管理 BLAS，按帧限制 400,000 vertices 的 compaction；`binder.rs` 创建 TLAS、实例、前后帧 transform、geometry/material/texture/light arrays 并提交 AS build。Realtime 分支包含 initial path、ReSTIR、world cache 与 DLSS-RR integration，另有独立 path tracer 作为验证路径。Zircon 的 Solari 名称来自同一概念，但当前只复制了门禁与 unavailable provider。

### 4.2 Unreal：AS 是有预算、流送、LOD 与代际所有权的长期资源

Unreal RHI 显式拥有 geometry/scene AS initializer、size query、resource、pipeline initializer 与 dispatch 接口；Geometry Manager 管理 build request handle、priority、build/update mode、强制构建、LOD group、stream-in/out、resident/evict、dynamic update 与引用集合。Renderer 再按 scene/view 构建 TLAS、组织 material hit shaders 和 pipeline/SBT binding。关键不是 API 数量，而是 BLAS 被当作可流送、有代际、有预算、有 GPU lifetime 的资产派生物，而非每帧临时 buffer。

### 4.3 Unity Graphics：Hardware 与 Compute 软件 BVH 共用一条产品接口

Unity Core UnifiedRayTracing 通过 `RayTracingContext` 选择 Hardware 或 Compute backend；同一 `IRayTracingAccelStruct/Shader` 合同覆盖 instance add/update/remove、build scratch size、build、dispatch 与 disposal。Compute backend 包含 HLBVH/TLAS builder、geometry pool 与 software ray query，硬件不可用时仍有语义一致的 fallback。HDRP 上层再管理 RTAS build mode、camera state、light cluster、temporal filter、diffuse/reflection denoiser、fallback hierarchy 与平台/build-target 资格。这比“InlineQuery/Pipeline 两个 bool”多出完整的资源、执行、降级和产品层。

### 4.4 Godot：没有实时硬件 RT，也没有伪装成支持

当前 Godot 参考树没有实时硬件 RT renderer；`lightmapper_rd` 使用 compute 构建/遍历软件 BVH，处理 closest hit、透明度、多 bounce、bias、samples 与 bake quality。它的价值是两条约束：没有硬件能力时必须诚实缺席；如果提供软件 fallback，也要有真实几何结构、trace kernel、质量参数与可取消/可诊断执行，不能靠一个 route enum 代替。

### 4.5 Fyrox：CPU lightmapper/Octree 是不同产品，不可冒充硬件 RT

Fyrox 以 CPU lightmapper、scene Octree 与物理 ray cast 覆盖离线/CPU 查询，没有图形硬件 RT pipeline。其 lightmap input preparation、取消、progress、surface/material eligibility 与序列化可用于离线 fallback 参考，但不能拿来证明实时画面中的 BLAS/TLAS、ray query、SBT 或 denoiser 已完成。

## 5. Owner 裁决与非重复边界

| Owner | 本篇拥有 | 本篇不重复拥有 |
|---|---|---|
| Runtime28 | RT capability profile、AS/Ray RHI 合同、RT scene compiler、BLAS/TLAS、Ray Query/Pipeline/SBT、effect gateway、RT denoise/scalability/evidence | 通用 render graph/barrier/device lifetime |
| Runtime09A | 通用 RHI handle/generation/fence、queue、barrier、device loss | AS 专属 descriptor/build/update/compact/dispatch 语义由 Runtime28 定义 |
| Runtime09B | GPU Scene、visibility、instance/LOD 主数据 | 从主数据派生 RT instance 与 TLAS 的资格、mask、generation |
| Runtime09C | shader compiler、PSO artifact/cache、permutation | ray stages、payload/hit group/SBT compatibility profile |
| Runtime09D | 通用 mesh/material streaming/residency | BLAS compact/serialize/stream/refit 与 AS memory receipt |
| Runtime09F3 | Hybrid GI surface cache/SDF/voxel 算法及其当前 false hardware route | 通用硬件 RT executor；未来只通过 Runtime28 gateway 消费 |
| Runtime23/24 | space/unit/projection 与稳定 identity/generation | RT instance transform、AS/SBT generation 的专属组合规则 |
| Editor22 | 通用 render authoring/capture/debug framework | RT settings、AS visualizer、fallback/effect receipt schema |
| Plugins04 | rendering/Solari package、capability、native provider 交付真实性 | RT backend、scene、pipeline、denoise 与画质算法 |

`ray_tracing` 内建 feature 与 `ray_tracing_policy` 插件 feature 的重复身份由 Runtime28 裁决；Plugins04 仍负责删除 package/catalog 的旧声明。Sound 的 geometry/acoustics provider 属于 Sound owner，本篇只禁止把外部 IR samples 计入 graphics hardware RT receipt。

## 6. P0 裁决与升级条件

当前没有 active P0：默认/advanced profile 不请求 Solari，真实 WGPU capability 为 false，Solari provider 明确 Unavailable，零 pass feature 没有被证明执行或影响产品画面。以下任一情况出现时，不需要等待下一轮 review，直接登记 P0：

1. `override_capabilities_for_tests` 或静态 bool 被带入 shipping/product receipt，并把没有 RHI API 的设备报告为 AS/InlineQuery/Pipeline ready。
2. Hybrid GI 选择 `HardwareRayTracing` 后仍执行 surface-cache/SDF/voxel shader，却把 receipt 标成 hardware rays executed。
3. Editor、Hub、export profile 或插件 capability 把 Experimental/Partial/Unavailable 展示为 Complete、Supported 或 package-ready。
4. 内建/插件零 pass descriptor 被 quality profile 启用，并将“编译成功”解释为光追效果已执行。
5. 声学 `rays_traced` 输入计数被汇入 graphics RT metrics，造成跨域 capability/evidence 伪造。

## 7. P1：Capability、RHI 与 Backend Contract

| ID | 当前差距 | 重构要求 |
|---|---|---|
| HRT-P1-001 | `AccelerationStructureCaps` 只有 supported、inline、pipeline、max instances | 建 versioned `RtCapabilityProfile`，覆盖 AS type/build/update/compact/serialize、geometry/instance limits、scratch/result alignment、SBT alignment/stride、payload/attribute/recursion、indirect dispatch 与 motion capability |
| HRT-P1-002 | WGPU backend 无条件 disabled，不读取 adapter feature，也没有启用请求 | backend negotiation 必须形成 available/requested/enabled 三态及拒绝原因；未启用不得只看 adapter available 报 ready |
| HRT-P1-003 | 仓库没有 DX12/Vulkan/Metal RT backend 或其它真实 executor | 选择首个产品 backend 并定义 milestone；未实现的平台通过同一 typed unavailable/fallback receipt 收口 |
| HRT-P1-004 | `RenderDevice` 没有 BLAS/TLAS handle、descriptor、create/destroy/query | 在 zr_rhi 建稳定 generational handle 与 backend-neutral descriptor，遵循 Runtime09A device-generation/fence 所有权 |
| HRT-P1-005 | CommandList 只有 copy/render/draw/compute，没有 AS build/update/copy/compact | 增加显式 command family、queue eligibility、resource access/barrier 与 completion receipt |
| HRT-P1-006 | PipelineKind 只有 Raster/Compute，没有 RayTracing | 定义 ray pipeline/hit-group/library descriptor；禁止把 ray pipeline 编译为 generic compute 来绕过能力合同 |
| HRT-P1-007 | ShaderStage 没有 raygen/miss/closest-hit/any-hit/intersection/callable | 扩展 Runtime09C stage、reflection、entry validation、artifact key 与 backend lowering |
| HRT-P1-008 | bind resource 只有 buffer/texture/sampler，shader 无 AS binding type | 增加 read-only TLAS binding及 layout/reflection validation；不得用 raw buffer 冒充硬件 AS |
| HRT-P1-009 | RhiError 没有 unsupported RT operation、invalid geometry/SBT/scratch 等终态 | 建可诊断错误族，区分 unsupported、invalid、OOM、device lost、compile failed 与 stale generation |
| HRT-P1-010 | 测试可直接 override 三个 bool，而设备没有相应方法 | capability 必须由 device implementation 与 conformance probe 共同证明；test fake 需实现完整 mock RT surface |
| HRT-P1-011 | RHI 的 max_instance_count 投影到 framework 时被丢弃 | capability summary/diagnostics 保留所有 negotiated limits 与来源，profile compiler按需求比较数值 |
| HRT-P1-012 | AS、InlineQuery、Pipeline 三个 bool 不能表达独立 stage/format/operation矩阵 | feature request 使用 required operations/limits，不再把总布尔值当可执行证明 |
| HRT-P1-013 | 内建 `ray_tracing` 与插件 `ray_tracing_policy` 是两套名称、两套 gate、均零 pass | 硬切到一个 canonical RT service/feature identity；plugin 只贡献 provider/policy，不复制 descriptor authority |
| HRT-P1-014 | policy descriptor 同时要求 AS+InlineQuery+Pipeline，合法的 query-only/pipeline-only 设备均会被拒绝 | descriptor 改为 alternatives/compiled route；每个 consumer 声明 Query 或 Pipeline 的真实最小需求 |

## 8. P1：RT Scene、Geometry、BLAS/TLAS 与 Residency

| ID | 当前差距 | 重构要求 |
|---|---|---|
| HRT-P1-015 | Mesh 没有 RT opt-in、primitive/attribute/index compatibility contract | 定义 `RtGeometryEligibility` 与 importer/build diagnostics；不合格 mesh 有明确 fallback/skip reason |
| HRT-P1-016 | 没有 BLAS identity/store，asset 修改与删除无法失效 GPU 结构 | 建 `BlasId { slot,generation,device }`、recipe hash、source generations 与 retire fence |
| HRT-P1-017 | 没有 PreferFastTrace/FastBuild/MinMemory、Build/Update 等 mode | profile compiler按 static/dynamic/skinned/streamed 用途生成 build flags，非法组合 compile-time 拒绝 |
| HRT-P1-018 | vertex/index buffer 没有 AS input usage、format/stride/offset/transform 语义 | mesh allocator 显式分配 AS-input-capable slices，并验证 topology、format、alignment 与 lifetime |
| HRT-P1-019 | 没有 prebuild size、scratch/result alignment 或 scratch allocator | backend size query -> budgeted scratch arena -> command receipt；禁止每次 build 临时无上限分配 |
| HRT-P1-020 | 没有 build priority、frame budget、async compute eligibility 或 backlog | 建 build scheduler，按可见性/距离/LOD/stream priority 排序并报告 queued/built/deferred work |
| HRT-P1-021 | 没有动态 mesh refit/rebuild，skinning/deformation/particles 无策略 | 每类 geometry 声明 update mode、source generation 与 max stale frames；超阈值重建而非盲目 refit |
| HRT-P1-022 | 没有 compaction、query readiness、swap 与旧 BLAS retire | 实现异步 compaction state machine、per-frame budget、fence-safe handle swap 与节省 bytes receipt |
| HRT-P1-023 | 没有 BLAS serialization/cache 或 streaming/residency | 与 Runtime09D artifact/residency 集成，校验 adapter/driver/build flags/source hash 后才能复用 |
| HRT-P1-024 | 没有 TLAS create/build/update 与 per-world/per-view owner | 建 `RtSceneSnapshot -> TlasBuildPlan -> TlasGeneration`，明确共享 world TLAS 与 view-specific filtering 边界 |
| HRT-P1-025 | 没有 instance id、custom index、mask、SBT offset、cull/opaque/front-face flags | 建稳定 `RtInstanceRecord` schema，CPU/GPU/shader 共享生成且有 ABI/hash 验证 |
| HRT-P1-026 | 没有 current/previous transform、motion instance 或 teleport/rebase 处理 | 消费 Runtime23/24 transform generation；camera cut、origin rebase、teleport 显式 invalidate/refit/rebuild |
| HRT-P1-027 | scene extraction 不追踪 added/changed/removed RT instance | 建增量 extract/change journal；删除必须在 TLAS build 前失效且在 fence 后回收引用 |
| HRT-P1-028 | 没有 geometry/material/texture/light binding arrays 与稳定索引 | 从 GPU Scene/material system 派生 typed tables，处理 bindless limits、fallback slots 与 generation |
| HRT-P1-029 | 没有 alpha-masked、two-sided、transparent、procedural AABB eligibility | MVP 明确只支持哪些 surface；any-hit/procedural 未完成时 fail-close，不得把它们强制 OPAQUE |
| HRT-P1-030 | 没有 multi-view、split-screen、reflection capture、shadow/GI consumer 的 TLAS lifetime | scene compiler以 consumer/view key复用或派生 TLAS，并记录instance filter、valid rect与last-use fence |

## 9. P1：Ray Query/Pipeline、SBT、Material 与 Effect Gateway

| ID | 当前差距 | 重构要求 |
|---|---|---|
| HRT-P1-031 | shader compiler没有 ray-stage source/artifact/permutation | Runtime09C 增加 ray library compile、reflection、backend binary、BuildSet/currentness 与 last-good |
| HRT-P1-032 | 没有 ray pipeline descriptor、hit groups、miss table 或 recursion | 定义 canonical `RtPipelineRecipe`，所有 entry/payload/attribute/local-root compatibility 在创建前验证 |
| HRT-P1-033 | 没有 SBT handle、record layout、alignment、local data、generation | 建 `SbtLayout/SbtRecord/SbtArtifact`，与 pipeline/material/geometry generation绑定并由 fence 管理 |
| HRT-P1-034 | Inline Query 只有 capability bool，没有 shader API或 commit/proceed/candidate 语义 | 建 backend-neutral query shader library与 validation；unsupported stage/flag在compile时拒绝 |
| HRT-P1-035 | 没有 payload/attribute size、recursion/depth 与 stack budget | capability/profile/pipeline compiler联合计算并记录实际值，超限给typed mismatch |
| HRT-P1-036 | 没有 hit material、barycentrics、vertex fetch、normal/tangent/UV 与 texture sampling | 建共享 geometry fetch/material evaluation ABI，并覆盖LOD、index offset、instance transform与non-uniform scale |
| HRT-P1-037 | 没有 RT light cluster、visibility channel、shadow mask 或 emissive sampling | 光照 consumer通过 typed scene/light snapshot读取，不能在每个 effect 私建不一致灯光表 |
| HRT-P1-038 | Solari provider始终 Unavailable，唯一消息直接承认 pass executor 未实现 | 保留 unavailable 直到scene/query/lighting/denoise/evidence全链可用；不得先改 Ready 再逐步补实现 |
| HRT-P1-039 | 内建 ray_tracing feature声明 view/geometry/visibility但零 workload/pass | 删除占位 feature或让 compiler明确产出 DisabledPlaceholder；Ready 必须关联实际 passes与outputs |
| HRT-P1-040 | Hybrid GI hardware route只写bitmask，WGSL无AS binding，继续执行软件 sources | 硬件 route必须绑定Runtime28 executor并有hardware dispatch receipt；否则selection返回typed fallback而非硬件位 |
| HRT-P1-041 | Sound `RayTraced` 仅接收外部samples并按字段刷新状态 | graphics RT 与 acoustics geometry provider分域；只有真实 acoustics trace executor才能发布其自己的executed receipt |
| HRT-P1-042 | 没有统一 effect request/output，阴影、反射、AO、GI将各自猜能力 | 建 `RtEffectRequest/CompiledRtRoute/RtEffectOutputs`，每个effect声明query/pipeline、scene subset、ray budget、fallback |
| HRT-P1-043 | 没有 ray pipeline cache、SBT/material warmup 或 artifact兼容 | 集成Runtime09C cache，按device/driver/compiler/payload/layout/scene ABI键控并提供last-good与prewarm receipt |

## 10. P1：Denoising、Scalability、Editor、Diagnostics 与 Product Qualification

| ID | 当前差距 | 重构要求 |
|---|---|---|
| HRT-P1-044 | 没有 RT shadow/reflection/AO/GI/path-tracing 任一真实画面输出 | 首个vertical slice只选一个effect，但必须从scene到最终composition全链闭环，不以多个空feature扩大表面 |
| HRT-P1-045 | 没有 diffuse/specular/shadow 专属 spatial/temporal denoiser | 定义 noisy signal、hit distance、moments/variance、normal/depth/motion/confidence输入与effect-specific filter |
| HRT-P1-046 | 没有 RT history key、reprojection、disocclusion、camera cut/reset | history携view/scene/TLAS/pipeline/profile/extent generations，使用motion/depth/normal/material资格并输出reject mask |
| HRT-P1-047 | QualityProfile只有Solari bool与experimental bool，无ray count/bounce/resolution/distance | 建 typed `RtQualityTier/EffectQuality`，编译ray budget、bounce、checkerboard、denoise、distance、instance LOD 与memory上限 |
| HRT-P1-048 | 没有 hardware -> software/SDF/screen-space/raster/baked 的per-effect fallback | fallback hierarchy按语义与质量排序，返回chosen route/reason/cost；不能统一降到“Disabled”或静默换算法 |
| HRT-P1-049 | 没有 BLAS/TLAS bytes、scratch、build/refit/compact时间、rays/hits/divergence统计 | 统一stats/telemetry记录queued/build/trace/denoise GPU时间、memory、instances、triangles、rays与fallback |
| HRT-P1-050 | 没有 AS bounds/instance mask/LOD/SBT/hit-distance/reject 可视化与capture | 接Editor22/frame capture，所有debug view带scene/TLAS/pipeline generation，避免观察旧资源 |
| HRT-P1-051 | Editor ray-tracing-policy只有名称、crate、capability字符串 | 提供project/camera/effect settings、platform diagnostics、memory/budget、fallback与live receipt；算法authority仍在Runtime |
| HRT-P1-052 | Solari产品测试只验证NotRequested/Missing/Unavailable状态 | 增加真实adapter或conformance backend lane，证明AS build、TLAS update、query/dispatch、denoise与output确实发生 |
| HRT-P1-053 | `solari_capabilities()`手工把AS/InlineQuery设true，不实现任何RHI方法 | mock/backend测试必须实现完整contract；布尔注入仅可测framework mapping，不能命名为product-ready |
| HRT-P1-054 | 没有 capability coherence test，supported可与API缺失/limit 0并存 | 建backend conformance suite：每个reported operation执行最小合法/非法case并核对错误、fence、generation与leak |
| HRT-P1-055 | 没有device loss、adapter switch、shader reload、mesh hot reload下的AS/SBT恢复 | 定义Recovering/UsingLastGood/Failed状态，重建顺序为device -> artifacts -> BLAS -> TLAS -> SBT -> histories |
| HRT-P1-056 | 没有几何正确性、材质正确性、降噪画质、性能/显存与跨GPU矩阵 | 建analytic rays、CPU/software reference、masked/two-sided/skinned corpus、temporal sequence及vendor/backend p50/p95预算 |

## 11. P2：完成 P1 后再进入的能力

| ID | 扩展项 | 进入条件 |
|---|---|---|
| HRT-P2-001 | 跨运行/构建机 BLAS serialization与prebuild | P1 cache key、driver/device兼容与corruption fallback稳定 |
| HRT-P2-002 | GPU-driven TLAS instance generation/indirect build | CPU reference、bounds/mask/overflow与readback validation通过 |
| HRT-P2-003 | Opacity Micromap/Displacement Micromap | alpha/procedural baseline、asset pipeline与capability矩阵完整 |
| HRT-P2-004 | curves、hair、particles、custom procedural intersections | triangle与AABB procedural ABI、any-hit和material contract稳定 |
| HRT-P2-005 | motion BLAS/TLAS 与 ray-time sampling | current/previous transform、deformation与temporal oracle稳定 |
| HRT-P2-006 | ray sorting、wave compaction、Shader Execution Reordering | 先有portable baseline、divergence指标与vendor-neutral fallback |
| HRT-P2-007 | ReSTIR DI/GI、reservoir temporal/spatial reuse | 基础effect、visibility、history rejection与bias tests稳定 |
| HRT-P2-008 | NRD/DLSS-RR/其它 neural denoiser provider | typed denoiser IO、licensing/package、fallback与currentness完整 |
| HRT-P2-009 | reference path tracer成为CI/offline quality oracle | material/light transport ABI与deterministic seed/capture已冻结 |
| HRT-P2-010 | Editor AS inspector、ray picking与单像素路径回放 | debug/capture generation、privacy与性能隔离通过 |
| HRT-P2-011 | cook/build farm 预构建与增量分发 | artifact provenance、platform matrix与cache invalidation稳定 |
| HRT-P2-012 | GPU crash dump关联AS/SBT/pipeline records | debug names、stable IDs、artifact manifest与符号化链完整 |
| HRT-P2-013 | adapter/vendor自动调优profile | 固定corpus与telemetry样本足够，仍允许确定性project override |
| HRT-P2-014 | 统一CPU/Compute/Hardware reference backend | P1 effect语义稳定后再追求跨backend一致性，不阻塞首个硬件vertical slice |

## 12. 目标架构与数据流

```text
Adapter facts + enabled features + limits
  -> RtCapabilityProfile
Project/Camera/Effect settings + budgets
  -> CompiledRtProfile
World/GPU Scene/Mesh/Material change journal
  -> RtSceneCompiler
  -> BlasRecipeStore -> Build/Refit/Compact/Residency Scheduler -> BlasStore
  -> RtInstanceTable -> TLAS BuildPlan -> TlasStore
Shader libraries + hit groups + payload ABI
  -> RtPipelineArtifact -> SbtArtifact
CompiledRtProfile + Tlas + Pipeline/SBT
  -> RayQueryExecutor | RayPipelineExecutor | SoftwareTraceExecutor
  -> RtEffectNoisyOutputs + ExecutionReceipt
Depth/Normal/Motion/HitDistance/Moments
  -> Effect Denoiser + History Qualification
  -> Typed Effect Outputs -> Lighting/Composition
  -> Stats/Capture/Editor/CI Evidence
```

关键不变量：capability只证明device已启用且API可执行；BLAS/TLAS/SBT均有device与source generation；consumer不能绕过compiled route直接读bool；fallback与hardware执行使用同一effect output语义；Ready/Executed必须引用实际command、resource与artifact receipt。

## 13. 分层实施顺序与 42 个验收门

### Phase A：Truth 与 Canonical Owner

1. **HRT-GATE-001**：删除内建/插件双重feature authority，只保留canonical service identity。
2. **HRT-GATE-002**：`RtCapabilityProfile`覆盖operation与limit，不丢失max-instance等数值。
3. **HRT-GATE-003**：query-only、pipeline-only、both、none四种设备组合编译结果正确。
4. **HRT-GATE-004**：reported capability与真实RenderDevice方法由conformance probe绑定。
5. **HRT-GATE-005**：所有unsupported平台保持typed fail-close与fallback reason。
6. **HRT-GATE-006**：Solari仍Unavailable，直到后续Ready gate全部关闭。

### Phase B：RHI 最小闭环

7. **HRT-GATE-007**：BLAS/TLAS generational handles跨device/stale使用必然失败。
8. **HRT-GATE-008**：geometry/instance/build descriptors合法与非法矩阵覆盖。
9. **HRT-GATE-009**：prebuild size、scratch/result alignment与OOM路径可验证。
10. **HRT-GATE-010**：build/update/compact commands有queue/barrier/fence语义。
11. **HRT-GATE-011**：AS binding reflection/layout验证覆盖错误类型。
12. **HRT-GATE-012**：Ray Query最小analytic triangle hit/miss与CPU reference一致。
13. **HRT-GATE-013**：Ray Pipeline最小raygen/miss/hit dispatch与payload一致。
14. **HRT-GATE-014**：pipeline/SBT alignment、record、stale generation失败测试通过。

### Phase C：Scene 与资源生命周期

15. **HRT-GATE-015**：static mesh add/change/remove正确创建、替换、retire BLAS。
16. **HRT-GATE-016**：dynamic/skinned mesh refit/rebuild策略与source generation一致。
17. **HRT-GATE-017**：compaction受每帧预算限制并报告before/after bytes。
18. **HRT-GATE-018**：LOD/stream-in/out/residency不会让TLAS引用已释放BLAS。
19. **HRT-GATE-019**：TLAS instance id/mask/flags/SBT offset ABI round-trip通过。
20. **HRT-GATE-020**：previous transform、teleport、origin rebase与device loss重建通过。
21. **HRT-GATE-021**：multi-view共享/派生TLAS的过滤与lifetime可证明。
22. **HRT-GATE-022**：masked/two-sided/unsupported surface按明确eligibility处理。

### Phase D：Material、Effect 与 Fallback

23. **HRT-GATE-023**：geometry fetch在vertex/index offset、LOD、non-uniform transform下正确。
24. **HRT-GATE-024**：material/texture/light表索引有generation与fallback slots。
25. **HRT-GATE-025**：首个effect从scene到composition有真实GPU output。
26. **HRT-GATE-026**：hardware/query/pipeline/software/raster fallback输出语义一致。
27. **HRT-GATE-027**：Hybrid GI hardware route只在真实hardware receipt存在时选择。
28. **HRT-GATE-028**：Sound与Graphics RT metrics/capability完全分域。
29. **HRT-GATE-029**：artifact/cache key覆盖device/driver/compiler/payload/SBT/scene ABI。
30. **HRT-GATE-030**：shader/material hot reload使用last-good或typed failed，不读旧SBT。

### Phase E：Denoising、Scalability 与恢复

31. **HRT-GATE-031**：noisy/raw、spatial、temporal、final outputs可分别capture。
32. **HRT-GATE-032**：motion/depth/normal/material/disocclusion reject corpus通过。
33. **HRT-GATE-033**：camera cut、extent/profile/TLAS/pipeline generation变化重置history。
34. **HRT-GATE-034**：quality tier编译到ray/bounce/resolution/distance/denoise成本。
35. **HRT-GATE-035**：degrade ladder逐级返回chosen route、reason与实际budget。
36. **HRT-GATE-036**：AS/scratch/SBT/history显存峰值与steady-state受预算约束。
37. **HRT-GATE-037**：device loss恢复顺序和终态receipt覆盖。

### Phase F：Product Qualification

38. **HRT-GATE-038**：backend conformance suite逐项证明每个reported operation。
39. **HRT-GATE-039**：analytic/CPU reference、masked、dynamic、LOD与temporal画质corpus通过。
40. **HRT-GATE-040**：至少两类adapter/backend记录build/trace/denoise p50/p95与memory基线。
41. **HRT-GATE-041**：Editor显示实际adapter、route、fallback、AS/SBT generations与GPU evidence。
42. **HRT-GATE-042**：只有GATE-001..041全绿，Solari/首个RT effect才可从Partial/Unavailable升级Ready；Complete仍需P2产品范围单独裁决。

## 14. 明确禁止的临时实现

1. 禁止新增 `supports_ray_tracing: bool` 代替 capability profile与conformance。
2. 禁止让测试override bool成为产品可用性的唯一证据。
3. 禁止用storage buffer中的自建BVH冒充hardware AS；软件backend必须明确命名与计量。
4. 禁止以Compute pipeline包装所有路径后宣称Ray Pipeline已支持。
5. 禁止每帧全量重建全部BLAS/TLAS且没有预算、generation与fence。
6. 禁止把所有geometry强制OPAQUE来绕过alpha/two-sided资格。
7. 禁止在每个effect复制scene extraction、material table、TLAS与fallback逻辑。
8. 禁止只实现ray hit shader而没有miss、payload、SBT/material generation验证。
9. 禁止先把Solari provider改成Ready，再用TODO/no-op pass占位。
10. 禁止用Hybrid GI bitmask、sound `rays_traced`计数或feature compile成功冒充GPU rays executed。

## 15. Closeout 与后续依赖

本篇是 review/architecture record，不授权 production 修改。推荐第一实现里程碑不是“完成Solari”，而是 Phase A+B+C 的最小 backend conformance：一个真实支持的adapter上完成 static triangle BLAS、TLAS、inline query hit/miss、generation/fence、capability/limit receipt，再接一个受控shadow或validation output。这样可以先证明基础设施正确，再进入material、effect与denoiser；反向从Solari UI或Realtime GI效果开工会继续积累无法验证的枚举、bool与零 pass 壳。

Runtime09A/09C/09D、Runtime23/24提供通用依赖；Runtime28负责把这些依赖收敛为光追专属contract。Plugins04只有在HRT-GATE-042达成后才能升级Solari capability status，Editor22只消费共享settings/debug/evidence schema。任何跨owner失败应写入对应failure handoff，不得在本篇通过复制接口临时绕过。
