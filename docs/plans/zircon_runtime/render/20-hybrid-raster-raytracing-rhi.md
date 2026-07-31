---
related_code:
  - zircon_runtime/src/rhi/capabilities.rs
  - zircon_runtime/src/rhi/descriptors/pipeline.rs
  - zircon_runtime/src/rhi/device.rs
  - zircon_runtime/src/rhi/device/handles.rs
  - zircon_runtime/src/rhi_wgpu/capabilities.rs
  - zircon_runtime/src/rhi_wgpu/device.rs
  - zircon_runtime/src/core/framework/render/backend_types/capability.rs
  - zircon_runtime/src/core/framework/render/solari/capability.rs
  - zircon_runtime/src/graphics/backend/render_backend/request_device.rs
  - zircon_runtime/src/graphics/feature/render_feature_capability_requirement.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/ray_tracing.rs
  - zircon_runtime/src/graphics/pipeline/declarations/render_pipeline_compile_options.rs
  - zircon_runtime/src/graphics/pipeline/compile_options/methods.rs
  - zircon_runtime/src/graphics/runtime/render_framework/capability_summary/capability_summary.rs
  - zircon_runtime/src/graphics/runtime/render_framework/capability_validation/mod.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/gpu_scene.rs
  - zircon_runtime/src/render_graph/types.rs
  - zircon_runtime/src/render_graph/builder.rs
  - zircon_runtime/src/render_graph/graph.rs
related_tests:
  - zircon_runtime/src/rhi/tests/capabilities.rs
  - zircon_runtime/src/rhi/tests/command_list.rs
  - zircon_runtime/src/rhi/tests/pipeline.rs
  - zircon_runtime/src/graphics/runtime/render_framework/capability_validation/mod.rs
  - zircon_runtime/src/graphics/tests/advanced_followup_slots.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile/validation_core.rs
  - zircon_runtime/src/graphics/tests/render_product_solari.rs
  - zircon_runtime/src/render_graph/tests/resources.rs
reference_sources:
  - dev/bevy/crates/bevy_solari/src/lib.rs
  - dev/bevy/crates/bevy_solari/src/scene/mod.rs
  - dev/bevy/crates/bevy_solari/src/scene/blas.rs
  - dev/bevy/crates/bevy_solari/src/scene/binder.rs
  - dev/bevy/crates/bevy_solari/src/realtime/node.rs
  - dev/UnrealEngine/Engine/Source/Runtime/RHI/Public/RHIResources.h
  - dev/UnrealEngine/Engine/Source/Runtime/RHI/Public/RHICommandList.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/RayTracing/RayTracingScene.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/RayTracing/RayTracingShaderBindingTable.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Tests/RayTracingTestbed.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/CommandBuffers/IComputeCommandBuffer.cs
plan_sources:
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - .codex/plans/Hybrid GI Lumen-Style V1 三阶段计划.md
  - .codex/plans/Hybrid GI 计算机图形学合集工程映射.md
---

# 计划 20:统一光栅/光追 RHI 与跨平台能力门控

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to implement this plan milestone by milestone. Track implementation slices with checkbox (`- [ ]`) syntax and complete each milestone's named testing stage before promotion.

**Goal:** 在保持 `wgpu` 为主后端的前提下,建立光栅与硬件光追共用的 RHI、场景资源、RenderGraph 和能力选择合同,使不同平台/API 能按实际能力启用、降级或拒绝功能,且不把后端名称泄漏到上层渲染逻辑。

**Architecture:** `zircon_runtime::rhi` 描述物理能力、资源和命令,`zircon_runtime::core::framework::render` 暴露中立能力摘要、项目策略和选择报告,graphics 的 RenderFeature 声明有序执行候选,统一 resolver 以 `设备能力 ∩ 项目策略 ∩ feature 需求` 产生唯一执行路径。当前只要求 `wgpu` 适配器落地;DX12/Vulkan/Metal 只保留可验证的未来映射,不得在本计划阶段创建原生后端或上层后端分支。

**Tech Stack:** Rust、wgpu 29、WGSL、Zircon RHI、RenderGraph、GPUScene、RenderFeature、RenderDoc;未来映射参考 DXR、Vulkan KHR ray tracing 与 Metal acceleration-structure 能力。

---

```zircon-workflow
{
  "schema": 1,
  "workflow_id": "render-20-hybrid-raster-raytracing-rhi",
  "goal": "建立 wgpu 主后端下统一光栅/光追 RHI、跨平台能力开关、加速结构生命周期与 RenderGraph 混合调度。",
  "milestones": [
    {"id": "RT-M1", "title": "能力模型、策略开关与路径解析", "depends_on": []},
    {"id": "RT-M2", "title": "RHI 加速结构、RayQuery 与光追管线合同", "depends_on": ["RT-M1"]},
    {"id": "RT-M3", "title": "共享网格资源与 RayTracingScene 生命周期", "depends_on": ["RT-M2"]},
    {"id": "RT-M4", "title": "RenderGraph 混合调度与同步", "depends_on": ["RT-M2", "RT-M3"]},
    {"id": "RT-M5", "title": "消费者接入、Solari 与确定性降级", "depends_on": ["RT-M4"]},
    {"id": "RT-M6", "title": "跨平台映射、工具与产品验收", "depends_on": ["RT-M5"]}
  ]
}
```

<!-- Workflow topology is maintained independently from milestone output records. -->

## 1. 已确认边界

1. `wgpu` 保持当前唯一必须实现的主后端。本计划不新增 `zircon_rhi_dx12`、`zircon_rhi_vulkan` 或 `zircon_rhi_metal`。
2. 硬件光追始终是能力门控的可选路径,不是 WebGPU/WGPU、Hybrid GI、Forward+ 或 Deferred 成立的前置条件。
3. DX12/Vulkan/Metal 只定义未来 adapter 如何填充同一能力结构、实现同一 RHI 命令;上层不得出现 `if backend == Vulkan` 一类分支。
4. `VK_EXT_descriptor_heap` 是可选 Vulkan 扩展策略,不是 Vulkan 1.4 或 Zircon RHI 基线。实现前必须以 Khronos 扩展规范和目标驱动实测为准。
5. 本计划拥有共享机制:能力解析、BLAS/TLAS、RayQuery、完整光追管线、shader record/SBT、场景加速结构和 graph 同步。阴影、AO、反射、GI 的算法与质量仍由计划 05/07/18、Hybrid GI 和 Solari 各自拥有。
6. 所有实际 pass 都经 RenderGraph 声明;任何 BLAS/TLAS build、trace dispatch、denoise 或 composite 都不得在 executor 外旁路 `queue.submit`。
7. 现有 `RenderCapabilitySummary`、`RenderFeatureCapabilityRequirement` 和 compile opt-in 是迁移起点,不是并存旧接口。每个里程碑完成时直接迁移调用方并删除被替代字段。

## 2. 当前实现基线与缺口

### 2.1 已有可复用基础

- `rhi/capabilities.rs` 已有 `AccelerationStructureCaps { supported, inline_ray_query, ray_tracing_pipeline, max_instance_count }`,证明 RHI 已预留光追能力槽。
- `core/framework/render/backend_types/capability.rs` 已把 `AccelerationStructures`、`InlineRayQuery`、`RayTracingPipeline` 拆成中立 `RenderCapabilityKind`,并能产出缺失能力诊断。
- `RenderPipelineCompileOptions` 已区分 feature enable 与 capability opt-in;`capability_validation` 会在 profile/pipeline 激活时拒绝缺失的严格能力。
- `PipelineKind::RayTracing` 已存在,且 WGPU 合同测试当前明确返回“不支持”,不会假成功。
- Solari 当前只要求 `InlineRayQuery + AccelerationStructures + binding arrays`,并以 unavailable/provider status 暴露未完成状态;这为后续渐进启用保留了真实边界。

### 2.2 必须补齐的缺口

- 三个布尔值无法表达 BLAS/TLAS build/update/refit/compaction、triangle/AABB 几何、实例上限、scratch 对齐、RayQuery shader stage、完整管线递归深度、payload/attribute 限制和 SBT 对齐。
- `rhi_wgpu/capabilities.rs` 无条件返回 `AccelerationStructureCaps::disabled()`,尚未把 adapter 支持、device 实际请求和最终返回 feature 汇聚为同一事实。
- `RenderPipelineCompileOptions.enabled_capabilities` 表示“用户允许”,`RenderCapabilitySummary` 表示“设备支持”,但 feature descriptor 只有扁平 all-of 要求,不能表达 `Pipeline -> InlineQuery -> Compute -> Raster -> Disabled` 的有序候选。
- `ShaderStage` 只有 Vertex/Fragment/Compute,`PipelineDesc` 没有 ray generation/miss/hit group/callable 或 shader table 描述,`RenderDevice`/command list 也没有 AS 创建、build、compact 和 trace 命令。
- GPU mesh buffer 创建时没有统一 BLAS input usage 规划;没有 per-asset BLAS cache、per-view TLAS、代际失效、延迟销毁或帧预算。
- RenderGraph 只认识 buffer/texture 与 raster/compute 工作,尚不能声明 acceleration-structure build-write/trace-read 依赖,也无法证明 GBuffer 与 RT pass 的同步无旁路。
- 当前 `ray_tracing` builtin descriptor 同时要求 AS 与完整 pipeline,而 Solari 只需要 inline query;二者不能共用一个粗粒度“ray tracing on/off”开关。

## 3. 所有权与分层

| 层 | 唯一职责 | 禁止事项 |
|---|---|---|
| `zircon_runtime::rhi` | 物理 capability/limit、AS 资源描述、稳定句柄、build/compact/trace 命令、结构化 RHI 错误 | 不出现 Mesh/Material/Light/Scene、产品 profile 或平台名称判断 |
| `zircon_runtime::rhi_wgpu` | 将 wgpu adapter/device 实际 features/limits 映射到 RHI,执行当前 wgpu 可表达的命令 | 不伪造 DXR/Vulkan/Metal 能力,不把实验 feature 当稳定基线 |
| `core::framework::render` | 中立能力摘要、项目策略、执行路径、选择报告和缺失诊断 DTO | 不 import `wgpu`,不持有 BLAS/TLAS 原生对象 |
| `graphics::feature` / pipeline compiler | feature 的共同必需能力、有序候选路径和 failure policy;编译出唯一选择 | 不读取 `backend_name`,不自行重复 capability 探测 |
| `graphics::scene::ray_tracing_scene` | 共享 mesh buffer 到 BLAS、GPUScene instance 到 TLAS 的生命周期与预算 | 不访问 ECS World,不拥有资产导入或材质编辑状态 |
| `render_graph` | AS 资源访问、build/trace 节点依赖、队列 lane 和同步 | 不实现后端 barrier,不允许 executor 外提交 |
| 计划 05/07/18、HGI、Solari | 声明各自候选路径并消费统一 scene/RHI 结果 | 不维护私有 BLAS/TLAS 或私有平台能力表 |

生命周期固定为:graphics 模块 `Immediate` 注册能力 resolver 与 RHI factory;昂贵的 BLAS/TLAS、SBT 和消费者 pipeline `Lazy` 创建。模块 shutdown 先停止 feature 提交,再等待 in-flight frame fence,最后按 TLAS -> BLAS/SBT -> shared mesh buffer 顺序延迟销毁。

## 4. 支持能力接口与开关

### 4.1 物理能力:后端实际支持什么

`rhi::RenderBackendCaps` 将现有 `acceleration_structures` 硬切换为嵌套 `ray_tracing: RayTracingCaps`;framework 的 `RenderCapabilitySummary` 对应使用中立 `RayTracingCapabilitySummary`。禁止继续平铺三个布尔值。

```rust
pub struct RayTracingCaps {
    pub acceleration_structure: AccelerationStructureCaps,
    pub inline_query: InlineRayQueryCaps,
    pub pipeline: RayTracingPipelineCaps,
    pub binding: RayTracingBindingCaps,
}

pub struct AccelerationStructureCaps {
    pub build: bool,
    pub update: bool,
    pub compaction: bool,
    pub indirect_build: bool,
    pub triangle_geometry: bool,
    pub aabb_geometry: bool,
    pub max_instances: Option<u32>,
    pub max_geometries: Option<u32>,
    pub max_primitives: Option<u64>,
}

pub struct InlineRayQueryCaps {
    pub supported: bool,
    pub shader_stages: RayQueryShaderStageMask,
    pub returns_committed_vertex_position: bool,
}

pub struct RayTracingPipelineCaps {
    pub supported: bool,
    pub callable_shaders: bool,
    pub max_recursion_depth: Option<u32>,
    pub max_payload_size: Option<u32>,
    pub max_attribute_size: Option<u32>,
    pub shader_group_handle_size: Option<u32>,
    pub shader_record_alignment: Option<u32>,
    pub shader_table_alignment: Option<u32>,
}

pub struct RayTracingBindingCaps {
    pub acceleration_structure_binding: bool,
    pub shader_record_local_data: bool,
    pub max_acceleration_structures_per_shader_stage: Option<u32>,
}
```

约束:

- `inline_query.supported` 或 `pipeline.supported` 为真时,`acceleration_structure.build` 与 `binding.acceleration_structure_binding` 必须为真;构造/映射阶段发现不可能组合即返回 `InvalidBackendCapabilities`,不得将矛盾状态传播到 profile。
- 未知 limit 使用 `Option::None`,不以 `0` 同时表示“不支持”和“未报告”。
- `backend_name` 只用于诊断与 telemetry;选择函数的输入比较测试必须证明改名不会改变结果。
- adapter 报告、device 请求和 device 最终启用 feature 三者分开记录;只有**最终启用**的能力能进入 `RenderBackendCaps`。

### 4.2 项目策略:产品允许使用什么

`RenderPipelineCompileOptions.enabled_capabilities` 被 `RenderCapabilityPolicy` 取代,避免把“项目允许”误当“设备支持”。profile/project 配置使用三态开关:

```rust
pub enum RenderCapabilityMode {
    Disabled,
    Auto,
    Required,
}

pub struct RayTracingPolicy {
    pub acceleration_structures: RenderCapabilityMode,
    pub inline_query: RenderCapabilityMode,
    pub pipeline: RenderCapabilityMode,
}

pub struct RenderCapabilityPolicy {
    pub ray_tracing: RayTracingPolicy,
    pub allow_experimental_wgpu_features: bool,
}
```

语义固定:

- 默认策略为三个光追开关全部 `Disabled`,且 `allow_experimental_wgpu_features=false`;默认启动路径不得因 adapter 偶然支持实验能力而改变。
- `Disabled`:即使设备支持也过滤该路径;compiled graph、pipeline cache 与资源分配均不得包含它。
- `Auto`:设备支持且 feature 候选允许时可选择;缺失时继续下一个候选,不报硬错误。
- `Required`:仅对当前激活 profile/feature 生效;缺失时在 profile 激活或 pipeline compile 阶段返回结构化 `CapabilityMismatch`,不得静默降级。
- `allow_experimental_wgpu_features=false`:所有 wgpu 实验光追 feature 在 device request 前关闭;不能仅在 graph 层假关闭后仍请求和初始化昂贵资源。
- device 创建时对 `Auto` 请求 adapter 已支持的允许 feature,以便运行期切换 profile;`Required` 缺失则返回明确 startup/device creation 错误。
- 已创建设备后从 `Disabled` 切到 `Auto/Required`,若目标 feature 当初未被 device 启用,resolver 返回 `DeviceRecreationRequired { capabilities }`;宿主决定重建设备后再切 profile。反向关闭可立即失效 selection 和 RT 资源,不要求重建设备。

### 4.3 Feature 需求:一个功能有哪些候选路径

保留 `RenderFeatureDescriptor.capability_requirements` 表达所有候选都需要的共同 all-of 能力,新增有序 `capability_plan`;不再用一个 `RayTracingPipeline` bool 代表所有光追功能。

```rust
pub enum RenderExecutionPath {
    Raster,
    Compute,
    InlineRayQuery,
    RayTracingPipeline,
    Disabled,
}

pub enum CapabilityFailurePolicy {
    UseNextCandidate,
    DisableFeature,
    RejectProfile,
}

pub struct RenderFeatureExecutionCandidate {
    pub path: RenderExecutionPath,
    pub requirements: Vec<RenderCapabilityKind>,
}

pub struct RenderFeatureCapabilityPlan {
    pub candidates: Vec<RenderFeatureExecutionCandidate>,
    pub failure_policy: CapabilityFailurePolicy,
}
```

候选顺序由消费者计划定义,但必须复用这些路径。例如:

| Feature | 有序候选 | 最低结果语义 |
|---|---|---|
| Solari realtime | InlineRayQuery -> Disabled | 实验 profile 可选择 `RejectProfile`;默认 profile 不请求 |
| ray-traced soft shadow | InlineRayQuery -> RayTracingPipeline -> Raster | 最终回到计划 05 shadow map,阴影功能不消失 |
| reflection | RayTracingPipeline -> InlineRayQuery -> Compute(SSR) -> Raster(probe) | 保持反射源 ledger,路径变化不重复计能量 |
| Hybrid GI world intersection | InlineRayQuery -> RayTracingPipeline -> Compute(SDF/Voxel) -> Disabled | WGPU V1 不依赖硬件 RT,source ledger 语义不变 |
| validation path tracer | RayTracingPipeline -> Disabled | 只用于测试/离线验证,不进入默认产品 profile |

### 4.4 唯一路径解析器

唯一公式固定为 `Capabilities + Policy + CapabilityPlan -> Selection`;四个对象分别可测试、可序列化诊断,不得互相覆盖字段。

```rust
pub fn resolve_render_feature_path(
    capabilities: &RenderCapabilitySummary,
    policy: &RenderCapabilityPolicy,
    plan: &RenderFeatureCapabilityPlan,
) -> Result<RenderCapabilitySelection, RenderCapabilitySelectionError>;

pub struct RenderCapabilitySelection {
    pub selected_path: RenderExecutionPath,
    pub considered: Vec<RenderCapabilityCandidateReport>,
    pub missing: Vec<RenderCapabilityMismatchDetail>,
    pub policy_blocked: Vec<RenderCapabilityKind>,
    pub reason: RenderCapabilitySelectionReason,
}
```

解析顺序固定为:

1. 验证 capability 结构内部一致性。
2. 按项目策略过滤 `Disabled` 与未允许的实验能力。
3. 按 feature candidates 原始顺序检查共同 all-of 与候选 requirements。
4. 选择第一个全部满足的候选并冻结到 `CompiledRenderPipeline`;同一帧 executor 不得重新选择。
5. 无候选时按 failure policy 选择 `Disabled` 或返回结构化错误。
6. 把候选、缺失、策略阻断与最终原因写入 stats/debug view;禁止只输出一个布尔值。

缓存键必须包含 `device_capability_generation + policy_revision + feature_descriptor_revision`;adapter/device 重建、profile 切换或插件热替换后旧 selection 自动失效。

## 5. RHI 合同

### 5.1 资源、句柄与描述符

新增 owner 文件,避免继续膨胀 `rhi/device.rs` 与 `descriptors/pipeline.rs`:

| 路径 | 计划内容 |
|---|---|
| `rhi/capabilities/ray_tracing.rs` | 上述物理 caps、合法性校验与 disabled/default 构造 |
| `rhi/descriptors/ray_tracing.rs` | triangle/AABB geometry、BLAS/TLAS、instance、build flags、ray pipeline/hit group/SBT 描述符 |
| `rhi/device/ray_tracing.rs` | 创建/销毁/查询大小与命令 trait 扩展 |
| `rhi/device/handles.rs` | `BlasHandle`、`TlasHandle`、`RayTracingPipelineHandle`、`ShaderBindingTableHandle` 代际句柄 |
| `rhi_wgpu/ray_tracing.rs` | wgpu 实验能力映射和可用命令;不支持的完整 pipeline 保持结构化错误 |

核心描述符必须覆盖:

- `BlasGeometryDesc::{Triangles, Aabbs}`;triangle 复用现有 vertex/index buffer handle、format、stride、offset、count,不复制业务 mesh 对象。
- `AccelerationStructureBuildFlags::{PreferFastBuild, PreferFastTrace, AllowUpdate, AllowCompaction}`;adapter 可拒绝不支持组合。
- `TlasInstanceDesc`:3x4 transform、instance custom index、mask、SBT record offset、instance flags 与 `BlasHandle`。
- `RayTracingPipelineDesc`:ray generation、miss、hit groups、callable groups、递归/payload/attribute 需求;创建时与 caps limits 比较。
- `ShaderBindingTableDesc`:pipeline handle、raygen/miss/hit/callable record slices;RHI 计算并验证 stride/alignment,上层不读取 D3D12/Vulkan 原生 handle size。
- buffer usage 增加 `ACCELERATION_STRUCTURE_BUILD_INPUT`、`ACCELERATION_STRUCTURE_STORAGE`、`SHADER_BINDING_TABLE`;同一 GPU mesh vertex/index buffer 同时服务 raster 与 BLAS build。

`ShaderStage` 增加 RayGeneration/Miss/ClosestHit/AnyHit/Intersection/Callable 可见性,或引入等价的 `RayTracingShaderStage`;二者只能有一个权威。inline query 仍是 Compute/Fragment 等普通 shader stage 的能力,不能伪装成 ray generation shader。

### 5.2 命令面与错误

RHI command list 新增:

- `build_blas` / `update_blas`
- `request_blas_compaction_size` / `compact_blas`
- `build_tlas` / `update_tlas`
- `dispatch_rays`

inline RayQuery 不增加独立 dispatch 命令;它通过普通 compute/raster shader 中的查询能力执行。`dispatch_rays` 只在 `pipeline.supported` 时可用。

所有命令必须验证 handle 代际、usage、scratch size/alignment、build/update flag、instance/geometry limit、pipeline/SBT 对应关系和资源状态。错误统一为 `RhiError::{UnsupportedCapability, InvalidAccelerationStructureDescriptor, InvalidRayTracingPipelineDescriptor, InvalidShaderBindingTable, ResourceStateConflict}`,不得 panic 或返回模糊字符串。

## 6. 共享网格与 RayTracingScene 生命周期

### 6.1 数据所有权

- 资产导入/烘焙侧只记录“可进入光追”的 geometry metadata 和稳定 revision;不保存平台 BLAS 二进制,也不在首帧同步生成全场景 BLAS。
- `GpuMeshResource` 创建 vertex/index buffer 时依据项目 policy 增加 BLAS input usage;光栅和光追引用同一 buffer handle、offset 与 layout revision。
- `RayTracingBlasCache` 键为 `(mesh_resource_id, geometry_revision, buffer_allocation_generation, build_flags)`;资源更新后旧 BLAS 进入 deferred destruction,不能被新 TLAS 引用。
- `RayTracingScene` 只消费 GPUScene instance/transform/material/visibility extract,不访问 ECS World。TLAS instance custom index 回指稳定 GPUScene instance id,材质和几何查表仍走统一 scene binding。
- TLAS 默认按 render family/view group 构建,复用 plan 09 的 layer mask 与 view visibility;不同消费者只读同一 TLAS,不得每个 effect 各建一份。

### 6.2 静态、动态与特殊几何

| 几何类型 | 首版策略 | 能力不足时 |
|---|---|---|
| 静态 mesh | build -> 异步查询 compact size -> 分帧 compaction -> 缓存 | 保留未压缩 BLAS;无 AS 时不创建 |
| transform-only instance | BLAS 不变,TLAS 增量更新或重建 | 受 TLAS frame budget 约束 |
| skinned/morph mesh | 有 update/refit 时更新动态 BLAS;否则按预算重建 | 超预算实例退出 RT 路径并记录 reason,光栅不受影响 |
| alpha-tested mesh | geometry 保持 triangle;opaque/any-hit 策略由材质 capability 与 consumer 决定 | 缺 any-hit 时走 conservative opaque 或退出该候选,不得透明穿帮而不报告 |
| Virtual Geometry | 首版只允许独立 coarse proxy BLAS | cluster 原生 RT 表示由 VG 计划另立里程碑,本计划不展开 |
| particles/UI/terrain procedural | 首版不进入共享 TLAS | 继续使用各自 raster/compute 路径 |

每帧维护 build vertices/primitive count、scratch bytes、compaction bytes 和 TLAS instances 四类预算;预算归计划 17 统一统计。删除/重建必须至少等待当前 frames-in-flight fence,压力测试覆盖反复导入、热重载、增删实例和 device lost 后全量重建。

## 7. RenderGraph 混合调度

RenderGraph 新增中立 `RgAccelerationStructureHandle` 或等价 typed external resource,并定义访问:

- `AccelerationStructureBuildWrite`
- `AccelerationStructureUpdateReadWrite`
- `AccelerationStructureTraceRead`
- `ShaderBindingTableRead`

`RayTracingPassDescriptor` 只表达 pipeline dispatch;inline query 继续使用 `ComputePassDescriptor`/raster pass,但显式声明 TLAS trace-read。AS build 是 compute 工作,不是独立“光追队列”;仍映射到现有 Graphics/Compute lane,由后端 caps 决定是否可异步。

固定帧序:

```text
RenderExtract / GPUScene sync
  -> shared mesh upload
  -> BLAS build or update -> optional compaction
  -> TLAS instance upload -> TLAS build or update
  -> raster depth/GBuffer/velocity
  -> inline-query compute or ray-pipeline dispatch
  -> denoise/history
  -> lighting/composite
  -> postprocess/UI/present
```

约束:

- GBuffer/depth/velocity 直接作为 RT/denoise graph 输入;只有 usage/format 明确不兼容时允许 graph 声明 copy,不能以“跨 API”名义默认复制。
- graph compiler 根据 AS access 产生 build-write -> trace-read 依赖;具体 D3D12/Vulkan/Metal barrier 只在未来 adapter 内实现。
- async compute 不可用时全部折回 Graphics lane,拓扑和资源语义保持一致。
- feature 关闭或 resolver 选择 Raster/Compute fallback 时,compiled graph 不含未选 RT pass、SBT 和消费者私有 RT resources。
- graph dump、debug marker、selection report 与 GPU timestamp 使用同一 pass name,便于 RenderDoc 对照。

## 8. 多平台/API 映射规则

| API/后端 | 首版状态 | 未来 adapter 映射边界 |
|---|---|---|
| wgpu 29 native | 唯一必须实现;默认禁用实验 RT | 只有 adapter 支持、项目允许且 device 实际请求成功的 `EXPERIMENTAL_RAY_QUERY` 才映射 AS + InlineQuery;支持 binding arrays 等附加条件分别报告;完整 RayTracingPipeline 继续为 false,直到 wgpu 提供并完成真实实现 |
| WebGPU 浏览器 | 必须可运行 fallback | 未报告 AS/RayQuery/Pipeline 即全部 false;HGI 保留 SDF/Voxel,阴影/反射保留 raster/compute 路径 |
| D3D12/DXR | 本轮不实现 | 未来读取 ray tracing tier 和实际 limits,映射 BLAS/TLAS、state object、shader table 与 DispatchRays;不向 framework 暴露 D3D12 类型 |
| Vulkan KHR | 本轮不实现 | 未来分别探测 acceleration structure、ray query、ray tracing pipeline 和依赖扩展;SBT 对齐取设备属性;descriptor set/buffer/heap 是 adapter 策略,不改变 feature 接口 |
| `VK_EXT_descriptor_heap` | 可选研究项 | 只有扩展存在且驱动验证通过才选择 heap 路径;否则继续 descriptor set/buffer;不得据文章标题推导为 Vulkan 1.4 核心能力 |
| Metal | 本轮不实现 | 未来只映射 Metal 实际提供的 AS/intersection/function-table 能力;若不能满足完整 pipeline 合同,只报告 InlineQuery/compute 等价路径,不得伪装支持 SBT pipeline |

任何未来 adapter 都必须先通过同一 capability truth-table、descriptor validation、lifetime 和 graph ordering suites,再允许消费者启用。平台差异通过 caps/limits/selection report 表达,不通过 feature 代码复制。

## 9. 文章与参考证据审查

指定文章[“自研渲染必看！混合光栅+光追管线”](https://mp.weixin.qq.com/s/yPkUrdYIwtWvgxH2cETMJA)及“计算机图形学”合集用于发现工程问题,不作为 API 或性能权威。采纳与纠偏如下:

| 线索 | 采纳 | 纠偏 |
|---|---|---|
| 文章 45:上层管线 / 统一 RHI / 多后端 / 工具分层 | 采纳统一 RHI、shared mesh buffer、BLAS/TLAS 生命周期、GBuffer 复用和混合调度 | 当前只实现 wgpu;不因目标架构提前创建三套原生后端 |
| 文章 41:DXR 与 Vulkan RT 的共同抽象 | 采纳 AS、RayQuery、pipeline/SBT 的中立合同 | 具体 stage、limit、descriptor 与同步必须回查官方规范和本地参考源码 |
| 文章 42:光栅与光追各自适用场景 | 采纳“光栅主可见性 + 可选 RT 效果 + compute denoise”的职责分工 | 不固定一条全平台相同的 RT 路径;每个 feature 有有序候选和确定性 fallback |
| 文章 44:`VK_EXT_descriptor_heap` | 作为未来 Vulkan binding adapter 研究项 | Khronos 将其定义为扩展能力,不是所有 Vulkan 1.4 设备必有的核心基线 |
| 文章 17/22:桌面 API 与 WebGPU 差异 | 采纳严格 capability/format/limit 验证 | 不采纳“WebGPU 普遍原生支持硬件光追”等绝对化主张 |

主参考证据:

- Bevy Solari `required_wgpu_features()` 证明 inline ray query 与 binding-array 能力必须分别请求;`scene/blas.rs` 证明 BLAS cache、共享 mesh allocator buffer、分帧 compaction 和不兼容 mesh 过滤应是独立生命周期。
- Unreal `RHIResources.h` / `RHICommandList.h` / `RayTracingScene.h` 证明 AS resource/command、持久 scene instance、per-view visibility 和 SBT 是不同边界;`RayTracingTestbed.cpp` 提供最小 BLAS -> TLAS -> trace -> readback 测试形态。
- Unity Graphics `IComputeCommandBuffer.cs` 证明 AS build 与将 AS 绑定给 compute/ray shader 可以在统一 command surface 上表达;Zircon 不复制其引擎对象或平台类。
- Khronos [`VK_EXT_descriptor_heap` proposal](https://docs.vulkan.org/features/latest/features/proposals/VK_EXT_descriptor_heap.html)与[guide](https://docs.vulkan.org/guide/latest/descriptor_heap.html)只用于验证扩展语义。

## 10. 里程碑

### RT-M1 能力模型、策略开关与路径解析

**目标:** 建立设备能力、项目策略和 feature 需求三层分离的唯一开关系统,在任何 GPU 资源创建前得到确定性 selection。

**依赖:** 计划 19 的 capability 基线和计划 17 的诊断字段;不依赖实际硬件 RT。

**实施切片:**

- [ ] 将 RHI/framework 三个平铺布尔值迁移为嵌套 `RayTracingCaps` / `RayTracingCapabilitySummary`,补齐 limits 与不可能组合校验。
- [ ] 将 compile options 的 capability opt-in 迁移为 `RenderCapabilityPolicy` 三态开关和实验 wgpu 总开关。
- [ ] 为 RenderFeature 增加共同 all-of + 有序 execution candidates + failure policy;迁移 builtin ray tracing 与 Solari requirements。
- [ ] 实现唯一 resolver、selection report、cache revision 和 backend-name-independent 诊断。
- [ ] 在 request_device 前生成 optional/required feature request,在 device 创建后只发布最终启用能力。

**RT-M1 测试阶段:** `render_capability_ray_tracing_*` 覆盖 caps 合法性、Disabled/Auto/Required 真值表、candidate 顺序、strict error、fallback、revision 失效和 backend name 不变性;`render_product_solari_*` 保持默认不请求、缺 capability、实验 gate 和 provider unavailable 的现有语义。批量运行 RHI capability、pipeline compile、render framework capability validation 与 Solari product suites,失败时先修正中立 capability foundation。

**退出证据:** 没有代码路径同时把“允许”和“支持”写入同一字段;所有 RT feature 都可解释最终选择,且默认/浏览器能力集不创建 RT graph 节点。

### RT-M2 RHI 加速结构、RayQuery 与光追管线合同

**目标:** 让 AS、inline query 和完整 pipeline 成为三个可独立实现/拒绝的 RHI 能力。

**依赖:** RT-M1 selection 冻结;计划 01 的资源生命周期和计划 08 的 shader source/variant 合同。

**实施切片:**

- [ ] 新增 typed handles、buffer usages、BLAS/TLAS geometry/instance/build descriptors 与 size/scratch query。
- [ ] 新增 shader stage/hit group、RayTracingPipelineDesc、SBT descriptor 和严格 limits/alignment 校验。
- [ ] 扩展 RenderDevice/command list 的 build/update/compact/dispatch 命令与结构化错误;inline query 继续走普通 shader dispatch。
- [ ] wgpu adapter 仅实现实际可用的实验 ray-query/AS 子集;完整 pipeline 无能力时保持 UnsupportedCapability。
- [ ] 为 resource generation、destroy order、invalid usage、update without flag、SBT mismatch 添加 contract tests。

**RT-M2 测试阶段:** 批量运行 `rhi_ray_tracing_*`、`rhi_resource_lifecycle_*`、`rhi_pipeline_*`、`rhi_command_list_*`;无 RT GPU 环境仍必须通过 disabled/unsupported 合同测试,有能力的 GPU test 才执行最小 triangle query。任何 test-only CPU RHI 不得被当作产品 GPU 支持证据。

**退出证据:** WGPU 不支持的命令稳定返回结构化错误;所有 AS/SBT handle 代际、usage 与对齐错误有负向测试。

### RT-M3 共享网格资源与 RayTracingScene 生命周期

**目标:** 光栅与光追复用同一 GPU geometry allocation,并以预算化 BLAS cache + GPUScene-driven TLAS 提供唯一场景加速结构。

**依赖:** RT-M2;计划 03 GPUScene 与计划 13 资源 revision/streaming 合同。

**实施切片:**

- [ ] 为 GpuMeshResource 加 BLAS input usage 与 layout/allocation generation,迁移创建方和 streamer rebuild。
- [ ] 新建 `graphics/scene/ray_tracing_scene/` owner,实现 BLAS cache key、build queue、compaction queue 与 deferred destruction。
- [ ] 从 GPUScene stable instance id/transform/layer/visibility 生成 TLAS instance buffer和 per-render-family TLAS。
- [ ] 实现 static、transform-only、skinned/morph rebuild/refit、alpha-test 和 coarse VG proxy 策略与 fallback reason。
- [ ] 接入计划 17 的 primitive/scratch/compaction/instance budgets 和 device-lost rebuild。

**RT-M3 测试阶段:** `render_ray_tracing_scene_*` 覆盖同 buffer 复用、cache hit/invalidation、静态 compaction、动态 update/rebuild、TLAS mask、删除延迟与 device lost;`render_perf_ray_tracing_scene_*` 只断言确定性计数和预算,时间只观测。压力 fixture 覆盖大量实例与反复 add/remove/update。

**退出证据:** 不存在消费者私有 BLAS/TLAS;资源更新不会让 TLAS 引用旧代际 BLAS;无硬件 RT 时 raster/GPUScene 产物不变。

### RT-M4 RenderGraph 混合调度与同步

**目标:** 把 AS build、raster GBuffer、inline query/ray dispatch、denoise 和 composite 编成一张有完整资源依赖的图。

**依赖:** RT-M2/RT-M3;计划 01 graph resource/access、计划 06 history、计划 17 pass profiling。

**实施切片:**

- [ ] 增加 typed AS graph resource、build/update/trace/SBT access 和 descriptor validation。
- [ ] 增加 RayTracingPassDescriptor;inline query 通过 compute/raster descriptor 声明 TLAS read。
- [ ] graph compiler 推导 AS write->read、queue foldback、GBuffer/depth/velocity 消费和 culling;executor registry 增加对应执行器。
- [ ] 将所有 build/compact/trace 提交迁入 graph executor,删除旁路 encoder/queue submit。
- [ ] graph dump/profile/marker 记录 selection path、AS resource generation 与 pass 名。

**RT-M4 测试阶段:** `render_graph_ray_tracing_*` 覆盖缺失 producer、read-before-build、cycle、queue foldback、feature culling 和同帧 generation mismatch;`render_product_hybrid_raster_rt_graph_*` 检查 Disabled/Compute/Inline/Pipeline 四种 compiled graph 形状。GPU 抓帧只在对应能力存在时要求 RT 节点,否则要求 fallback 节点和明确 reason。

**退出证据:** graph 外无 AS/trace queue submission;feature 关闭时没有隐藏资源分配;同一 GBuffer 输入不存在无理由复制。

### RT-M5 消费者接入、Solari 与确定性降级

**目标:** 用真实消费者证明 capability plan 可复用,而不把共享层变成某个效果的特例。

**依赖:** RT-M4;消费者计划各自最低 fallback 已可运行。

**实施切片:**

- [ ] 计划 05 soft shadow 声明 Inline/Pipeline/Raster 候选,共享 TLAS 和 selection report。
- [ ] 计划 07/18 reflection 声明 Pipeline/Inline/SSR/probe 候选,沿用反射 source ledger 与 denoise/history。
- [ ] Hybrid GI M5-S5 只把硬件 RT 注册为 world-intersection candidate,SDF/Voxel 保持 WGPU V1 基线。
- [ ] Solari provider 在 inline query、binding arrays、RayTracingScene、prepass/history 都 ready 后才报告 Ready;默认 profile 仍不请求。
- [ ] validation path tracer 只在完整 pipeline candidate 选中时注册,否则 Disabled,不影响产品完成度。

**RT-M5 测试阶段:** 各消费者运行 capability matrix:缺 AS、只有 AS、AS+Inline、AS+Pipeline、policy disabled、experimental disabled、strict required;断言 graph 形状、status/reason、source ledger 和无 double count。产品图像只比较效果语义、有效像素和容差,不要求跨 API bit-exact。

**退出证据:** 至少两个消费者使用同一 resolver 与 scene owner;更改 backend_name 不改变路径;WebGPU/WGPU 无 RT 环境仍通过核心场景。

### RT-M6 跨平台映射、工具与产品验收

**目标:** 固化未来 adapter 合同和当前 wgpu 产品证据,避免“接口存在”被误报成“多后端已完成”。

**依赖:** RT-M5;计划 17 profiling/RenderDoc 钩子。

**实施切片:**

- [ ] 为 wgpu adapter 建立 request/returned feature、limits、selection 与 graph 节点的端到端报告。
- [ ] 写 DX12/Vulkan/Metal 映射表和 adapter conformance fixture,只测试中立输入输出,不创建原生实现。
- [ ] 增加 RenderDoc capture marker、BLAS/TLAS build/trace pass stats、selection dump 和 device-lost diagnostics。
- [ ] 建立 fallback 产品矩阵与有能力 GPU 的最小 triangle、动态 instance、shadow/reflection/Solari 场景。
- [ ] 更新实际实现对应的 `docs/zircon_runtime/**` 模块文档;只记录真实落地能力,未实现 adapter 保持 future mapping。

**RT-M6 测试阶段:** 按 `milestone-validation-policy.md` 批量运行 RHI、render graph、pipeline compile、scene lifetime、消费者产品和 runtime diagnostics suites;有 RT GPU 时保存能力清单、graph dump、RenderDoc capture 与图像,无 RT GPU 时保存 disabled/fallback 选择报告。跨平台 conformance 只在真实 backend adapter 存在后升级为产品完成门。

**退出证据:** 文档、stats 与实际 device features 一致;没有任何原生后端完成声明;fallback 和 strict-required 两条路径均有端到端证据。

## 11. 验收矩阵

| 维度 | 必须覆盖 |
|---|---|
| capability truth table | physical/policy/requirement 全组合、impossible caps、limits、backend-name invariance |
| compile/graph | 候选顺序、strict error、disable culling、queue foldback、AS write/read、history/denoise 顺序 |
| RHI contract | descriptor/usage/alignment/generation、build/update/compact、pipeline/SBT mismatch、unsupported path |
| lifetime | cache hit、revision invalidation、frames-in-flight deferred destroy、device lost、重复加载卸载 |
| consumer | shadow/reflection/HGI/Solari 至少两个真实 consumer;能力不足不崩溃且结果可解释 |
| platform | wgpu request/returned 一致;Web fallback;未来 adapter 同一 conformance suite |
| stress | 大量实例、动态变形重建预算、反复 profile 切换、插件热替换、内存压力下清理 |
| tooling | graph dump、marker、stats、selection reason、RenderDoc 捕获路径一致 |

## 12. 非目标与拒绝项

- 不在本计划内实现完整 Lumen、路径追踪器、降噪算法或新材质系统。
- 不为了文章中的“统一”而把 raster pipeline 与 ray pipeline 塞进一个巨型 `PipelineDesc` 可选字段集合;共享 layout/handle 可复用,描述符保持按 pipeline kind 分型。
- 不保存驱动专属 BLAS blob 作为跨平台资产基线。
- 不以 GPU 型号、操作系统或 backend string 推断能力。
- 不把实验 wgpu feature 设为默认 Required,不让 CI 因无 RT GPU 失败;CI 必须验证正确 fallback,有能力 runner 才验证 RT product path。
- 不接受 first-frame 全场景同步 BLAS build、每 effect 私建 TLAS、AS build graph 外提交、无代际裸句柄或 silent fallback。

## 状态与产出记录

每个里程碑测试通过后记录一次;实现切片不单独写入产出记录。

| 里程碑 | 范围 | 状态 | 完成日期 | 验证批次 / 残余风险 |
|---|---|---|---|---|
