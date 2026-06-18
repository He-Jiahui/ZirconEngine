---
related_code:
  - zircon_runtime/src/render_graph/builder.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/backend/render_backend/mod.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - dev/UnrealEngine/Engine/Plugins/Experimental/NNERuntimeRDG/Source/NNEHlslShaders/Internal/NNEHlslShadersOperator.h
  - dev/UnrealEngine/Engine/Plugins/Experimental/NNERuntimeRDG/Shaders/Private/NNEHlslShaders/NNEHlslShadersConv.usf
  - dev/UnrealEngine/Engine/Plugins/Experimental/NNERuntimeRDG/Shaders/Private/NNEHlslShaders/NNEHlslShadersGemm.usf
  - dev/UnrealEngine/Engine/Plugins/Experimental/NNERuntimeBasicCpu/Source/NNERuntimeBasicCpu/Private/NNERuntimeBasicCpuModel.h
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/MipGen/MipGenerator.cs
  - dev/bevy/crates/bevy_render/src/gpu_readback.rs
  - dev/bevy/crates/bevy_render/src/render_resource/pipeline_cache.rs
  - dev/bevy/crates/bevy_core_pipeline/src/mip_generation/mod.rs
  - dev/learn-wgpu-zh/code/utils/src/node/compute_node.rs
  - dev/learn-wgpu-zh/code/intermediate/compute-pipeline/src/blur_node.rs
plan_sources:
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - .codex/plans/多插件组合可选功能规则设计.md
---

# 计划 16:compute shader 框架与神经网络支持

## 目标

1. 面向用户/插件的 compute 框架:`ComputePassDescriptor`(WGSL/zshader compute 入口 + 绑定 schema + dispatch 来源)经 RenderFeature 进 graph,支持 dispatch_indirect、跨帧持久 buffer、异步 readback(非阻塞回传);内建系统(HZB、light grid、SVT、粒子)逐步迁到同一框架,消除各自手写 compute 样板。
2. 神经网络推理底座(NN Runtime,插件):算子集 compute 化(GEMM/Conv/激活/归一化/上下采样等,对照 UE NNERuntimeRDG 算子清单的核心子集),模型资产(算子图 + 权重 buffer),图执行器把算子序列编译为 graph compute pass 链;首个应用场景:NN 后处理(风格化/降噪/上采样占位)挂进计划 07 链尾的 upscale 槽位。
3. CPU 推理回落档(对照 NNERuntimeBasicCpu):无 compute 能力或调试时同模型可在 CPU 跑,产物一致性可对拍。

## 现状与差距

- compute 已在引擎内多处使用(后处理/SSAO/粒子),但都是 executor 内手写:无统一描述符、无 dispatch_indirect 封装、readback 各自为政;插件想加 compute pass 没有正门。
- 神经网络:完全空白;用户需求(NN 后处理、未来 ML 驱动动画/上采样)需要一个与渲染帧同步的 GPU 推理底座,而不是外挂 ONNX 进程。

## 参考代码

| 文件 | 应重点阅读 |
|------|-----------|
| `dev/UnrealEngine/.../NNERuntimeRDG/Source/NNEHlslShaders/Internal/NNEHlslShadersOperator.h` | 算子枚举与注册:UE 把 NN 算子做成 RDG compute pass 的清单与分类 —— 本计划算子集裁剪的依据 |
| `dev/UnrealEngine/.../NNERuntimeRDG/Shaders/Private/NNEHlslShaders/NNEHlslShadersGemm.usf` + `NNEHlslShadersConv.usf` | GEMM/Conv 的 compute 实现形态(tile 化、共享内存),WGSL 移植参照 |
| `dev/UnrealEngine/.../NNERuntimeBasicCpu/Private/NNERuntimeBasicCpuModel.h` | CPU 推理回落档的模型表示(扁平算子序列 + buffer 视图) |
| `dev/Graphics/.../Runtime/MipGen/MipGenerator.cs` | 通用 compute pass 封装的最小形态(绑定/dispatch/链式 reduce) |

次参考:`dev/learn-wgpu-zh` compute 章节(wgpu compute 基础范式);wgpu `Limits`(workgroup 尺寸/storage 大小约束进 capability gate)。

**Rust/wgpu 落地参照(防凭空实现)**:

| 文件 | 对应本计划机制 | 应重点阅读 |
|------|---------------|-----------|
| `dev/bevy/crates/bevy_render/src/gpu_readback.rs` | `GpuReadbackQueue` 直接对应:buffer/texture 异步回读的完整 Rust/wgpu 实绩(staging 池、`map_async` 收口、跨帧回调) | `GpuReadbackBufferPool`(按帧计数复用/`max_unused_frames` 过期)、readback 生命周期 prepare→copy→map→事件派发;对照差异:bevy 按 ECS 组件驱动,Zircon 走 ticket + 回调 |
| `dev/bevy/crates/bevy_render/src/render_resource/pipeline_cache.rs` | `compute_pipeline_cache.rs` 对应:compute pipeline 的缓存键控与异步创建状态机 | `Pipeline::ComputePipeline` 分支、`CachedPipelineState::{Queued,Creating,Ok}`、`create_pipeline_task` 与同步编译回落开关 |
| `dev/bevy/crates/bevy_core_pipeline/src/mip_generation/mod.rs` | CN-M1 迁移示范同型:内建 reduce 系统经统一 compute 组织(job 注册 → pipeline 特化 → dispatch) | `MipGenerationJobs`/`MipGenerationPhaseId`(phase 锚点)、`SpecializedComputePipelines` 特化键控、WGSL 模板文本替换(与 `shader_templates.rs` 同手法) |
| `dev/learn-wgpu-zh/code/utils/src/node/compute_node.rs` | 通用 compute pass 最小封装:`ComputeNode`(pipeline + bind groups + dispatch 四元组) | `wgpu::ComputePipelineDescriptor` 创建、`dispatch_by_offsets` 的 dynamic uniform offset 与 workgroup 计数 —— `"compute.generic"` executor 录制路径的最小对照 |
| `dev/learn-wgpu-zh/code/intermediate/compute-pipeline/src/blur_node.rs` | compute pass 链(横/纵两 dispatch 交替读写)的教学级实绩 | ping-pong 绑定组织与 `dispatch_workgroups` 的 div_ceil 计算(对照 `PerPixel` 变体) |

`NN 算子库(GEMM/Conv2d/池化/归一化的 compute 化)` 无 Rust 渲染内嵌同类参照,实现时以 UE NNERuntimeRDG 为唯一样板,按 index §8 第 8 条配对拍测试先行(CPU 解释档即对拍基准)。

## 目标架构

归属:compute 框架是 runtime 基础设施(`render_graph/` + `graphics/` 内建);NN Runtime 为 `zircon_plugins/` 新插件包(算子 WGSL + 图执行器 + 模型资产),经 compute 框架接入,runtime 零 NN 专有代码。

核心设计:

- `ComputePassDescriptor`:入口(shader 资产引用 + entry point)、绑定 schema(storage/uniform/texture,经计划 01 句柄)、dispatch(Fixed(x,y,z) | FromBuffer(indirect) | PerPixel(target, local_size))、执行阶段(phase 锚点);经 RenderFeature descriptor 注册,在 graph 中与 render pass 同等公民(culling/资源生命周期全适用)。
- `GpuReadbackQueue`:统一异步回读(staging ring buffer + N 帧延迟回调),SVT feedback/粒子计数/NN 输出共用;禁止 executor 私自 map_async。
- NN 插件:
  - `NnModelAsset`:算子序列(拓扑排序后的扁平列表)+ tensor 描述(shape/dtype,V1 仅 f32/f16)+ 权重 blob;导入器从 ONNX 子集转换(离线工具,插件 editor crate)。
  - 算子库:V1 集 = GEMM、Conv2d、DepthwiseConv、Add/Mul/激活(ReLU/Sigmoid/Tanh/SiLU)、BatchNorm/LayerNorm、MaxPool/AvgPool、Upsample、Concat/Slice/Reshape(view 级零拷贝);每算子一个 WGSL 模板 + 尺寸特化变体(走计划 08 变体缓存)。
  - `NnGraphExecutor`:模型 → compute pass 链(中间 tensor 走计划 01 瞬态池);`NnPostProcessEffect` 作为计划 07 的 volume 组件接入链尾。
  - CPU 档:同模型解释执行(rayon 并行),用于无能力回落与正确性对拍。

## 里程碑

### CN-M1 compute 框架

实施切片:
1. `ComputePassDescriptor` + graph 接入 + dispatch_indirect 封装;`GpuReadbackQueue`。
2. 一个内建系统迁移示范(MipGen/HZB reduce 迁到框架),验证框架够用。

测试阶段:
- `cargo check -p zircon_runtime --lib --locked`;`cargo test -p zircon_runtime compute --locked` + `render_graph` 回归
- 验收证据:迁移后 HZB 产物不变;readback 队列 N 帧延迟语义单测;插件注册 compute pass 的集成测试。

### CN-M2 NN 插件骨架与算子库 V1

实施切片:
1. 插件包族 + `NnModelAsset` + ONNX 子集离线转换器。
2. 算子 WGSL V1 集 + 每算子 CPU 参考实现 + 单算子对拍测试框架。

测试阶段:
- `cargo test --manifest-path zircon_plugins/Cargo.toml -p <nn runtime crate> --locked`(每算子 GPU vs CPU 数值对拍,容差 1e-3)
- 验收证据:算子对拍全绿;转换器处理一个真实小模型(如小型 CNN)。

### CN-M3 图执行器与端到端推理

实施切片:
1. `NnGraphExecutor`(瞬态 tensor 池、pass 链生成);权重上传与常驻管理。
2. 端到端:小模型图像分类/风格化在引擎内推理,输出经 readback 验证。

测试阶段:
- 插件范围测试(端到端输出与 CPU 档对拍;与离线参考输出对拍)
- 验收证据:同输入 GPU/CPU/离线三方一致(容差内);帧内推理耗时进 RenderStats。

### CN-M4 NN 后处理接入

实施切片:
1. `NnPostProcessEffect` volume 组件(模型引用 + 输入输出约定:scene color in/out);挂计划 07 链尾 upscale 槽或自定义槽。

测试阶段:
- 插件 + runtime post_process 范围回归
- 验收证据:风格化模型作用于画面(抓帧);关闭 feature 时 graph 无 NN pass;性能档位(分辨率缩放推理)生效。

## 工程落地细化

实施权威章节(index.md §8 第 7 条)。跨计划契约名原样引用:计划 01 `RgTextureHandle`/`RgBufferHandle`/`RgResourceResolver`/`TransientResourcePool`/`mark_persistent`/`mark_readback`;计划 04 `HzbBuilder`(迁移示范对象);计划 07 `VolumeComponentDescriptor`;计划 13 `TextureMetadata`(SVT feedback 是 `GpuReadbackQueue` 第二消费方)。本计划新增契约名:`ComputePassDescriptor`、`GpuReadbackQueue`、`NnModelAsset`、`NnGraphExecutor`、`NnPostProcessEffect`。

现状基础(写前已核):`render_graph/types.rs` 已有 `RenderGraphComputeWorkload { pipeline_label, workgroup_size, dispatch_extent }` 与 `RenderGraphComputeDispatchExtent::{Viewport, ClusterGrid, Fixed}`、`QueueLane::{Graphics, AsyncCompute, AsyncCopy}`;`RenderFeaturePassDescriptor` 已有 `compute_workload: Option<RenderGraphComputeWorkload>` 字段与 `.with_compute_workload(...)`(`feature_descriptors/clustered_lighting.rs` 为既有用例);手写 compute 样板现存两处:`execute_ssao.rs`(pipeline `OnceCell::get_or_init` + 临时 bind group + `dispatch_workgroups(div_ceil(SSAO_WORKGROUP_SIZE))`)与 `execute_clustered_lighting.rs`;阻塞 readback 现存一处 runtime(`read_texture_rgba.rs` 的 `map_async` + `poll(wait_indefinitely)`)加三处插件(见迁移禁令清单);`backend_types.rs` 已有 `supports_async_compute`/`supports_neural_compute` capability 位与 `RenderStats::last_graph_compute_dispatch_count` 系列字段。本计划的增量是:描述符定稿 + 通用 executor(消样板)、dispatch 来源扩展(indirect/per-pixel)、统一异步 readback、NN 插件包族。

### 模块与文件落点

runtime 框架侧(compute 框架 + readback):

| 文件 | 内容 | 性质 |
|------|------|------|
| `zircon_runtime/src/render_graph/types.rs` | `RenderGraphComputeDispatchExtent` 增 `FromBuffer { buffer: String, offset: u64 }` 与 `PerPixel { target: String, local_size: [u32; 2] }`;删 `Viewport`(语义被 `PerPixel` 覆盖,调用方同变更迁移,硬切换);新增 `BindingSchemaEntry`/`ComputeBindingKind`(纯名字层,无 wgpu) | 修改 |
| `zircon_runtime/src/render_graph/builder.rs` | `set_compute_workload` 校验:`FromBuffer` 的 buffer 必须已被该 pass `read_buffer` 声明;`PerPixel` 的 target 必须已声明读或写 | 修改 |
| `zircon_runtime/src/graphics/feature/compute_pass_descriptor/mod.rs` + `compute_pass_descriptor.rs` | `ComputePassDescriptor`/`ComputeShaderSource`,以及向 `RenderFeaturePassDescriptor` 的 lowering(自动展开 IO 声明 + compute_workload + executor id) | 新增 |
| `zircon_runtime/src/graphics/feature/render_feature_pass_descriptor/render_feature_pass_descriptor.rs` | 增 `compute_pass: Option<ComputePassDescriptor>` 字段与 `.with_compute_pass(...)` builder | 修改 |
| `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/generic_compute_executor.rs` | `"compute.generic"` executor:按 schema 组 bind group(经 `RgResourceResolver` 解析句柄/名字)、三种 dispatch 录制、pipeline 缓存查询;含 `#[cfg(test)]` | 新增 |
| `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/compute_pipeline_cache.rs` | shader module + `wgpu::ComputePipeline` 缓存,键 = (shader source 哈希, entry_point);走计划 08 变体缓存同款键控策略 | 新增 |
| `zircon_runtime/src/graphics/backend/render_backend/gpu_readback_queue/mod.rs` + `staging_ring.rs` + `ticket.rs` | `GpuReadbackQueue`/`ReadbackTicket`/staging ring;wgpu 类型只在此层 | 新增 |
| `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/scene_renderer_core.rs` | `SceneRendererCore` 增 `readback_queue: GpuReadbackQueue` 字段 | 修改 |
| `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs` | `with_builtin_noop_executors` 注册 `"compute.generic"`;迁移对象的手写 executor 注册行删除 | 修改 |
| `zircon_runtime/src/core/framework/render/backend_types.rs` | `RenderStats` 增 `last_readback_in_flight_count: usize`、`last_readback_bytes: u64`、`last_named_compute_pass_micros: Vec<(String, u64)>`(纯数据,NN pass 以 pass 名出现,runtime 不识别 NN) | 修改 |
| `zircon_runtime/src/graphics/scene/scene_renderer/hzb/mod.rs` + `feature_descriptors/hzb.rs`(计划 04 落点) | 迁移示范:HZB reduce 的手写 pipeline/bind/dispatch 样板删除,改 `with_compute_pass`;若实施时计划 04 未落地,迁移示范回退为 `execute_ssao.rs`(删本体,`ao.ssao-evaluate` 改挂 `compute.generic`,disabled 时的 clear 路径留在 feature 开关侧) | 修改 |
| `zircon_runtime/src/graphics/tests/render_compute.rs`、`graphics/backend/render_backend/gpu_readback_queue/tests.rs` | 测试(注册进各自 `mod.rs`) | 新增 |

NN 插件包族(`zircon_plugins/`,对照 `rendering/`、`particles/` 既有包形态;`zircon_plugins/Cargo.toml` workspace members 增三 crate):

| 文件 | 内容 |
|------|------|
| `zircon_plugins/neural/plugin.toml` | id `"neural"`,modules `neural.runtime`(crate `zircon_plugin_neural_runtime`)/`neural.editor`(crate `zircon_plugin_neural_editor`),optional_features 含 `post_process` |
| `zircon_plugins/neural/runtime/src/lib.rs` | crate 根:模块声明 + 注册入口(薄) |
| `zircon_plugins/neural/runtime/src/model/{asset.rs,format.rs,validate.rs}` | `NnModelAsset`、`.znn` 二进制读写、加载校验 |
| `zircon_plugins/neural/runtime/src/ops/{op_code.rs,attrs.rs}` | `NnOpCode` 枚举、各算子属性结构 |
| `zircon_plugins/neural/runtime/src/gpu/{graph_executor.rs,tensor_layout.rs,weight_upload.rs,shader_templates.rs}` | `NnGraphExecutor`、tensor→buffer 布局、权重上传常驻、WGSL 模板特化 |
| `zircon_plugins/neural/runtime/src/gpu/shaders/{nn_gemm.wgsl,nn_conv2d.wgsl,nn_elementwise.wgsl,nn_pool.wgsl,nn_normalize.wgsl,nn_upsample.wgsl,nn_tensor_image.wgsl}` | 算子 WGSL 模板(`zr_` include 不适用:模板含 entry point,自成一族) |
| `zircon_plugins/neural/runtime/src/cpu/{interpreter.rs,ops/}` | CPU 解释执行档(rayon) |
| `zircon_plugins/neural/runtime/src/tests/` | 对拍与资产测试 |
| `zircon_plugins/neural/features/post_process/runtime/src/lib.rs` | crate `zircon_plugin_neural_post_process_runtime`:`NnPostProcessEffect` + `VolumeComponentDescriptor` 注册 + RenderFeature descriptor(对照 `rendering/features/*` 形态) |
| `zircon_plugins/neural/editor/src/onnx/{reader.rs,convert.rs,diagnostics.rs}` + `src/bin/zr_onnx_convert.rs` | ONNX 子集离线转换器(库 + CLI bin) |

### 核心类型与接口

层归属:`ComputePassDescriptor` 在 graphics 层(引用 `RenderPassStage`,graphics 依赖 render_graph,反向不可);`BindingSchemaEntry` 在 render_graph 层(纯名字);`GpuReadbackQueue` 携带 wgpu,固定 backend 层不出 graphics;framework 契约层(`core::framework::render`)只收 `RenderStats` 纯数据。NN 全部类型在插件 crate,runtime 零 NN 专有代码。

```rust
// render_graph/types.rs(规划层,无 wgpu)
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ComputeBindingKind {
    UniformBuffer,          // 只读小参数块
    StorageBufferRead,
    StorageBufferReadWrite,
    SampledTexture,
    StorageTextureWrite,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BindingSchemaEntry {
    pub binding: u32,            // group1 内编号(§8 槽位:compute pass 全部资源属 pass 级 = group1)
    pub resource: String,        // graph 资源名;执行期经 RgResourceResolver 解析为 RgTextureHandle/RgBufferHandle 物理资源
    pub kind: ComputeBindingKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RenderGraphComputeDispatchExtent {
    Fixed([u32; 3]),                                      // 既有
    ClusterGrid,                                          // 既有(clustered lighting 专用,保留)
    FromBuffer { buffer: String, offset: u64 },           // 新:dispatch_workgroups_indirect
    PerPixel { target: String, local_size: [u32; 2] },    // 新:groups = ceil(target_extent / local_size);替代并删除 Viewport
}
```

```rust
// graphics/feature/compute_pass_descriptor/compute_pass_descriptor.rs(graphics 层)
#[derive(Clone, Debug)]
pub enum ComputeShaderSource {
    BuiltinWgsl { label: &'static str, source: &'static str },  // include_str! 内建
    Asset { asset: crate::asset::AssetId },                     // zshader/wgsl 资产(经 shader_wgsl_importer 路径)
    InlineWgsl { label: String, source: String },               // 插件运行期模板特化产物(NN 用)
}

#[derive(Clone, Debug)]
pub struct ComputePassDescriptor {
    pub pass_name: String,                  // graph pass 名,也是 RenderStats 计时键
    pub stage: RenderPassStage,             // phase 锚点:复用既有枚举,不新设(见帧时序节锚点表)
    pub queue: QueueLane,
    pub shader: ComputeShaderSource,
    pub entry_point: String,                // 默认 "cs_main"
    pub workgroup_size: [u32; 3],           // 必须与 WGSL @workgroup_size 一致,executor 断言
    pub bindings: Vec<BindingSchemaEntry>,
    pub dispatch: RenderGraphComputeDispatchExtent,
    pub flags: PassFlags,
}

impl ComputePassDescriptor {
    /// lowering:展开为 RenderFeaturePassDescriptor ——
    /// executor_id 固定 "compute.generic";compute_workload 由 (pass_name, workgroup_size, dispatch) 构成;
    /// bindings 逐条翻译 IO 声明(SampledTexture→read_texture、StorageTextureWrite→write_storage_texture、
    /// StorageBufferRead→read_buffer、StorageBufferReadWrite→write_buffer、FromBuffer.buffer→read_buffer);
    /// 外部资源名(import_external_resource 注册的)走 read_external/write_external 对应变体。
    pub fn into_feature_pass(self) -> RenderFeaturePassDescriptor;
}
```

插件注册 compute pass 的完整形态(对照 `particles/runtime/src/render/feature.rs` 既有写法):

```rust
// zircon_plugins/<plugin>/runtime — RenderFeature 注册一个 compute pass
use zircon_runtime::graphics::{ComputePassDescriptor, ComputeShaderSource,
    RenderFeatureDescriptor, RenderPassStage};
use zircon_runtime::render_graph::{BindingSchemaEntry, ComputeBindingKind,
    QueueLane, RenderGraphComputeDispatchExtent};

pub fn render_feature_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "my_feature",
        vec!["view".to_string()],
        Vec::new(),
        vec![ComputePassDescriptor {
            pass_name: "my-feature-reduce".into(),
            stage: RenderPassStage::PostProcess,
            queue: QueueLane::AsyncCompute,
            shader: ComputeShaderSource::BuiltinWgsl {
                label: "my-feature-reduce", source: include_str!("shaders/reduce.wgsl") },
            entry_point: "cs_main".into(),
            workgroup_size: [8, 8, 1],
            bindings: vec![
                BindingSchemaEntry { binding: 0, resource: "my.params".into(),
                    kind: ComputeBindingKind::UniformBuffer },
                BindingSchemaEntry { binding: 1, resource: "scene-color".into(),
                    kind: ComputeBindingKind::SampledTexture },
                BindingSchemaEntry { binding: 2, resource: "my.output".into(),
                    kind: ComputeBindingKind::StorageBufferReadWrite },
            ],
            dispatch: RenderGraphComputeDispatchExtent::PerPixel {
                target: "scene-color".into(), local_size: [8, 8] },
            flags: Default::default(),
        }.into_feature_pass()],
    )
}
```

```rust
// graphics/backend/render_backend/gpu_readback_queue/mod.rs(backend 层,wgpu 允许)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ReadbackTicket(u64);   // 单调递增,跨帧唯一

pub type ReadbackCallback = Box<dyn FnOnce(Result<&[u8], ReadbackError>) + Send + 'static>;

pub struct GpuReadbackQueue {
    slots: [StagingSlot; Self::FRAME_SLOTS],   // FRAME_SLOTS = 3
    next_ticket: u64,
    pending: Vec<PendingRequest>,              // 本帧已登记、待 encode 的请求
}

impl GpuReadbackQueue {
    pub const FRAME_SLOTS: usize = 3;          // in-flight 帧数,槽位 = frame_index % 3
    /// graph 资源回读:buffer 须已 mark_readback(计划 01),否则 Err。
    pub fn request_readback(&mut self, buffer: RgBufferHandle, range: std::ops::Range<u64>,
        callback: ReadbackCallback) -> Result<ReadbackTicket, ReadbackError>;
    /// 外部常驻 buffer 回读(SVT feedback、粒子计数、NN 输出):计划 13 第二消费方走此入口。
    pub fn request_readback_external(&mut self, name: &str, buffer: &wgpu::Buffer,
        range: std::ops::Range<u64>, callback: ReadbackCallback) -> ReadbackTicket;
    /// 帧 N 录制末尾(graph 执行后、submit 前):copy_buffer_to_buffer 到本帧槽位 staging。
    pub fn encode_copies(&mut self, encoder: &mut wgpu::CommandEncoder,
        resolver: &RgResourceResolver<'_>) -> Result<(), ReadbackError>;
    /// submit 后立刻调用:对本帧槽位 staging 发起 map_async(整槽一次)。
    pub fn begin_map(&mut self, frame_index: u64);
    /// 帧首调用:device.poll(PollType::Poll) 非阻塞;已完成槽位逐请求切片回调、unmap、槽位归还。
    pub fn poll_completed(&mut self, device: &wgpu::Device) -> ReadbackPollStats;
    pub fn cancel(&mut self, ticket: ReadbackTicket);
}
```

staging ring 尺寸策略与 fence 语义:

- 槽容量:初始 256 KiB;帧内请求总量(各请求按 256 字节对齐后求和)超容时按 2 的幂增长重建该槽;连续 240 帧使用率 < 25% 时减半(下限 256 KiB)。
- 请求偏移对齐 256(同时满足 `COPY_BUFFER_ALIGNMENT`、map 切片与潜在 storage 复用)。
- fence 语义:wgpu 无显式 fence,以 `map_async` 完成回调即完成信号;`poll_completed` 用非阻塞 poll,回调最早在帧 N+1、典型帧 N+2 送达(**N 帧延迟语义:调用方不得假设当帧可得**)。轮转回到同一槽位(帧 N+3)时若 map 仍未完成,仅对该槽阻塞等待(背压),次数计入 `RenderStats::last_readback_in_flight_count` 旁的诊断。
- **私自 map_async 迁移禁令清单**(本计划落地后,以下调用点全部改走 `GpuReadbackQueue`,源码扫描测试封死):
  1. `zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/gpu_readback/decode/read_buffer_u32s.rs:16`
  2. `zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/gpu_readback/decode/read_buffer_u32s.rs:16`
  3. `zircon_plugins/particles/runtime/src/render/gpu/backend.rs:543`
  4. `zircon_runtime/src/graphics/backend/render_backend/read_texture_rgba.rs:49` —— **唯一白名单**:同步抓帧/测试产物路径(`finish_viewport_frame.rs` 消费),保留阻塞语义,文件头注释声明豁免;其余任何新增 `map_async` 由 `readback_no_private_map_async_source_scan` 测试拒绝。

NN 插件侧核心类型(全部在 `zircon_plugin_neural_runtime`):

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NnDtype { F32, F16 }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NnTensorKind { Input, Output, Intermediate, Weight }

#[derive(Clone, Debug)]
pub struct NnTensorDesc {
    pub dtype: NnDtype,
    pub kind: NnTensorKind,
    pub shape: [u32; 4],          // NCHW,不足 4 维左补 1;V1 固定 rank<=4
    pub weight_offset: u64,       // kind==Weight 时为权重 blob 内偏移(256 对齐),否则 0
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum NnOpCode {
    Gemm = 1, Conv2d = 2, DepthwiseConv2d = 3,
    Add = 16, Mul = 17, Sub = 18, Div = 19,
    Relu = 32, Sigmoid = 33, Tanh = 34, Silu = 35,
    BatchNorm = 48, LayerNorm = 49,
    MaxPool2d = 64, AvgPool2d = 65, Upsample2d = 66,
    Concat = 80, Slice = 81, Reshape = 82,   // view 级:只改 NnTensorDesc/偏移,不发 pass
}

#[derive(Clone, Debug)]
pub struct NnOp {
    pub code: NnOpCode,
    pub inputs: Vec<u16>,         // tensor id
    pub outputs: Vec<u16>,
    pub attrs: NnOpAttrs,         // 每算子定型结构(Conv2dAttrs{stride,padding,dilation,groups} 等)
}

pub struct NnModelAsset {
    pub tensors: Vec<NnTensorDesc>,
    pub ops: Vec<NnOp>,           // 已拓扑排序的扁平列表(转换器保证)
    pub weights: Vec<u8>,         // 对齐见字节格式表
}

pub struct NnGraphExecutor { /* 模型句柄 + pipeline 特化缓存 + 权重 GPU 常驻表 */ }
impl NnGraphExecutor {
    /// 模型 → ComputePassDescriptor 链:每非 view 算子一个 pass;中间 tensor 经 builder.create_buffer
    /// 得 RgBufferHandle(生命周期区间由 graph 编译推导,TransientResourcePool 复用,计划 01);
    /// 权重 buffer 上传一次常驻,经 import_external_resource 进图。
    pub fn build_passes(&self, model: &NnModelAsset, io: &NnGraphIo)
        -> Result<Vec<ComputePassDescriptor>, NnError>;
}

pub fn run_cpu(model: &NnModelAsset, inputs: &[(u16, &[f32])]) -> Result<Vec<Vec<f32>>, NnError>; // rayon 解释执行
```

### GPU 数据布局与 WGSL 约定

**`NnModelAsset` 二进制格式(`.znn`,全部小端)**:

| 偏移 | 字段 | 类型 | 说明 |
|------|------|------|------|
| 0 | magic | `[u8; 4]` | `b"ZRNN"` |
| 4 | version | u32 | 1;不识别即拒载,无静默兼容 |
| 8 | flags | u32 | bit0 = 权重为 f16(否则 f32);其余保留 0 |
| 12 | tensor_count | u32 | |
| 16 | op_count | u32 | |
| 20 | op_table_size | u32 | 字节数 |
| 24 | weight_blob_offset | u64 | 文件内偏移,256 对齐 |
| 32 | weight_blob_size | u64 | |
| 40 | tensor 表 | 32 B × tensor_count | 每条:dtype u8、kind u8、rank u8、pad u8、shape `[u32;4]`、weight_offset u64、pad `[u8;8]` |
| — | op 表 | 变长 | 每条:op_code u16、input_count u8、output_count u8、attr_size u16、pad u16、tensor id u16 × (in+out)(4 字节对齐)、attr 块 |
| weight_blob_offset | 权重 blob | | 每个 Weight tensor 的起始 256 对齐(对齐 wgpu `min_storage_buffer_offset_alignment` 默认值,允许单 buffer 多权重 offset 绑定) |

**tensor GPU 布局**:每个非 view tensor 一个 storage buffer 区段,NCHW 行主连续,f32(f16 路径走能力检测,`supports_neural_compute` 为 false 时整 feature 关闭);元素数 = N·C·H·W,无 padding(std430 标量数组);Reshape/Slice/Concat 不分配新 buffer:转换器把 Concat(沿 N 或 C 外维)预解析为各输入直接写入输出 buffer 的偏移区段,内维 Concat 在转换期插入显式 Copy 算子,不做运行期判断。

**NN 算子 binding 编号**(§8 槽位:compute pass 资源全属 pass 级 = group1;group0/2/3 空缺):

```
@group(1) @binding(0) var<uniform> params: NnOpParams;        // 每算子定型参数(shape/stride/alpha 等)
@group(1) @binding(1..=k) var<storage, read> input_i;          // 按 NnOp.inputs 序
@group(1) @binding(k+1..) var<storage, read_write> output_j;   // 按 NnOp.outputs 序
```

通用 compute pass(非 NN)同规:`BindingSchemaEntry.binding` 即 group1 内编号,executor 按列表序组装,断言无重号。

**GEMM tile 常量**(对照 `NNEHlslShadersGemm.usf` ALGORITHM 5,`GROUP_SIZE 16` 共享内存档):

```wgsl
// nn_gemm.wgsl 模板要点
const TILE: u32 = 16u;                                  // = usf GROUP_SIZE 16(8/32 档 V1 不做,见精读笔记取舍)
var<workgroup> tile_a: array<f32, 256>;                 // = SharedMemoryA[GROUP_SIZE*GROUP_SIZE]
var<workgroup> tile_b: array<f32, 256>;
@compute @workgroup_size(16, 16, 1)
// k 维步数 = ceil(K / TILE)(= usf NumGroupSteps);每步:双 tile 协同加载(越界线程补 0 参与加载,
// 对齐 usf "overflow elements set to zero"),workgroupBarrier() 后每线程沿 tile 行×列累积
// acc += tile_a[ty*TILE+kk] * tile_b[kk*TILE+tx];仅在最终写回前判 (m<M && n<N) 提前返回
// (usf 注释:out-of-bound 线程必须活到共享内存加载完);写回 y = alpha*acc + beta*c(Gemm attrs)。
```

**Conv2d(direct)**:每线程一个输出像素,`@workgroup_size(8, 8, 1)`,dispatch = `(ceil(W_out/8), ceil(H_out/8), C_out·N)`;权重 OIHW 连续;内层沿 (C_in, kh, kw) 累积;DepthwiseConv2d 同映射但 z = C·N 且只取本通道权重。V1 不做 im2col+GEMM 路径(留 V2,接口不锁死)。

**激活/逐元素**:单模板 `nn_elementwise.wgsl`,线性展开 `@workgroup_size(64, 1, 1)`,dispatch x = `ceil(elem_count/64)`,越界 guard;算子函数体经 `shader_templates.rs` 文本替换 `//ZR_NN_OP_BODY` 注入(Relu/Sigmoid/Tanh/Silu/Add/Mul/Sub/Div 共 1 个模板 8 个特化),特化产物作 `ComputeShaderSource::InlineWgsl` 进 pipeline 缓存(键含特化名)。

### 帧时序与集成点

**compute pass 的 phase 锚点**:不新设枚举,锚点 = `RenderPassStage`(`graphics/pipeline/declarations/render_pass_stage.rs` 既有 18 变体)× `QueueLane`。compute 常用锚点对照:

| 锚点 stage | 典型 compute 负载 | 既有/将有用例 |
|-----------|------------------|--------------|
| `DepthPrepass`(后) | HZB reduce(计划 04 `HzbBuilder`) | 迁移示范 |
| `AmbientOcclusion` | SSAO evaluate | `ao.ssao-evaluate` |
| `Lighting` | clustered light culling | `lighting.clustered-cull`(既有 `ClusterGrid`) |
| `Transparent3d` | 粒子 spawn/update/compact(`AsyncCompute`) | particles 插件既有 |
| `PostProcess` | exposure histogram(计划 07)、NN 推理链 | 本计划 CN-M4 |

**单帧顺序**(锚定 `scene_renderer_core_render_compiled_scene/render/`):

1. 帧首:`readback_queue.poll_completed(device)` —— 派发已完成回调(SVT feedback 消费、粒子计数、NN 输出),更新 `RenderStats` readback 字段。
2. graph build/编译:RenderFeature 解析期,`ComputePassDescriptor::into_feature_pass` 产物与 render pass 同表进 builder;feature 关闭则 descriptor 不注册,compiled graph 无对应 pass(§6.4);pass culling、瞬态区间推导对 compute pass 全适用(计划 01)。
3. pass 循环:`"compute.generic"` executor 经 `RgResourceResolver` 解析 schema → bind group(group1),pipeline 缓存命中或编译,按 dispatch 变体录制(`Fixed`→`dispatch_workgroups`;`FromBuffer`→`dispatch_workgroups_indirect(buffer, offset)`;`PerPixel`→按 target 实际 extent `div_ceil(local_size)`)。
4. 录制末尾、submit 前:`readback_queue.encode_copies(encoder, resolver)`(本帧全部 readback 请求的 `copy_buffer_to_buffer` 进同一 encoder 尾部)。
5. submit 后:`readback_queue.begin_map(frame_index)`;回调于帧 N+1..N+2 的步骤 1 送达(三槽 ring 保证不覆写)。

**NN 推理在链尾的位置**(计划 07 链定稿表衔接):`NnPostProcessEffect` 注册 `post.nn-effect` 槽,两种挂法由模型 IO 形状决定 —— 超分模型:占据 upscale 槽(替换 bilinear),输入 anti-aliased(render-scale 尺寸)、输出 `UPSCALED`(全尺寸);等尺寸模型(风格化/降噪):挂 upscale 槽之后,`UPSCALED` in/out。pass 链 = `nn_tensor_image.wgsl` image→tensor 前导 pass + 算子 pass 链 + tensor→image 收尾 pass,全部经 `into_feature_pass` 进 graph。推理档:`inference_scale`(1.0/0.75/0.5)对输入先 bilinear 降采样、输出回放大(两 pass 由 effect 在 scale<1 时追加)。Volume 参数经计划 07 `PostProcessVolumeExtract` 求值,extract 只带纯数据(模型 AssetId + 标量),遵守 §6.6。

### 实施切片细化

**CN-M1 compute 框架**

- 切片 1.1 dispatch 来源与 schema 类型(render_graph 层)
  - 触碰:`render_graph/types.rs`、`builder.rs`、`render_graph/tests/`。
  - 要点:`FromBuffer`/`PerPixel` 变体 + `BindingSchemaEntry`;删 `Viewport` 并迁移其唯一类调用方(`RenderGraphComputeWorkload::viewport` 构造器改产 `PerPixel`,SSAO/计划 04 描述符同变更更新);builder 校验规则。
  - 判据:`cargo check -p zircon_runtime --lib --locked` 过;旧 `Viewport` 引用编译期不可达。
- 切片 1.2 `ComputePassDescriptor` + lowering + 通用 executor
  - 触碰:`graphics/feature/compute_pass_descriptor/`(新)、`render_feature_pass_descriptor.rs`、`graph_execution/generic_compute_executor.rs`(新)、`compute_pipeline_cache.rs`(新)、`render_pass_executor_registry.rs`。
  - 要点:lowering 的 IO 自动声明;executor 的 schema→bind group 组装与三种 dispatch;pipeline 缓存。
  - 判据:check 过;插件示例 descriptor(测试夹具)能进 compiled graph 并被 noop 设备路径执行。
- 切片 1.3 `GpuReadbackQueue`
  - 触碰:`gpu_readback_queue/`(新)、`scene_renderer_core.rs`、`render/render.rs`(帧首 poll + 录制尾 encode 两个挂点)、`backend_types.rs`(RenderStats 字段)。
  - 要点:三槽 ring、尺寸策略、`mark_readback` 校验、external 入口(为计划 13 留好签名)。
  - 判据:check 过;poll/encode 挂点就位且空队列零开销(无请求时不创建 staging)。
- 切片 1.4 迁移示范(HZB reduce;计划 04 未落地则 SSAO)
  - 触碰:`hzb/mod.rs` + `feature_descriptors/hzb.rs`(或 `execute_ssao.rs` + `feature_descriptors/` ssao 项 + `builtin_postprocess_executors.rs` 的 `ssao_executor`)。
  - 要点:删手写 pipeline `OnceCell`/bind group/dispatch 样板与专属 executor 注册行,改 `.with_compute_pass(...)`;WGSL binding 编号迁到 group1 规约(硬切换,shader 同变更改)。
  - 判据:迁移前后产物 readback 一致(进 M1 测试阶段验收);样板文件行数净减。
- M1 测试阶段:`cargo test -p zircon_runtime compute --locked` + `render_graph` + `readback` 过滤词;验收证据同正文。
- 切片 1.5 插件 readback 迁移(M1 测试绿后)
  - 触碰:迁移禁令清单 1–3 的三个插件文件 + `readback_no_private_map_async_source_scan` 测试。
  - 判据:`cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_hybrid_gi_runtime -p zircon_plugin_virtual_geometry_runtime -p zircon_plugin_particles_runtime --locked` 过(各包真实 crate 名以 Cargo.toml 为准)。

**CN-M2 NN 插件骨架与算子库 V1**

- 切片 2.1 包族 + `NnModelAsset`:`neural/plugin.toml`、runtime crate 的 `model/`+`ops/`、`zircon_plugins/Cargo.toml` members → 判据:二进制 roundtrip 单测过(check 级)。
- 切片 2.2 ONNX 转换器:`neural/editor` 的 `onnx/` + `zr_onnx_convert` bin;支持算子映射表(`Gemm|MatMul→Gemm`、`Conv→Conv2d/DepthwiseConv2d(group==C_in)`、`Relu/Sigmoid/Tanh→同名`、`Mul(Sigmoid(x),x) 图样→Silu 融合`、`BatchNormalization→BatchNorm(可烘进相邻 Conv 权重)`、`MaxPool/AveragePool→*Pool2d`、`Resize(nearest|linear, scale=2)→Upsample2d`、`Concat/Slice/Reshape/Flatten→view 级`、`Add/Mul/Sub/Div→同名`);不支持算子诊断:stderr 表格 + JSON(`{"node": 名, "op_type": 类型, "reason": 文本, "input_shapes": [...]}` 行式)+ 退出码 2,不静默近似 → 判据:小 CNN 转换通过、构造的含 `LSTM` 模型给出诊断。
- 切片 2.3 WGSL 模板 + CPU 参考 + 对拍框架:`gpu/shaders/` 全集、`shader_templates.rs`、`cpu/`;单算子对拍夹具(随机权重/输入,固定种子)。
- M2 测试阶段:`cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_neural_runtime --locked`;对拍全绿(容差见测试节)。

**CN-M3 图执行器与端到端**

- 切片 3.1 `NnGraphExecutor`:拓扑序(资产已序,执行器只校验)→ 每算子 `ComputePassDescriptor`;中间 tensor `create_buffer` 区间交瞬态池;view 算子折叠为偏移重写;权重 `weight_upload.rs` 一次上传 + `import_external_resource` 常驻(不进瞬态池,等价 `mark_persistent` 语义的 buffer 侧)。
- 切片 3.2 端到端:小模型推理样例 + 输出经 `request_readback` 验证;pass 计时进 `last_named_compute_pass_micros`。
- M3 测试阶段:GPU/CPU/离线(转换器自带 reference 输出)三方对拍;瞬态复用断言。

**CN-M4 NN 后处理接入**

- 切片 4.1 `neural/features/post_process/runtime`:`NnPostProcessEffect` + `VolumeComponentDescriptor`(参数:model AssetId、intensity f32 [0,1]、inference_scale 枚举、enabled)随 RenderFeature descriptor 注册(计划 07 既定挂法);tensor↔image 前后导 pass;feature 关闭零 pass。
- M4 测试阶段:插件 + `cargo test -p zircon_runtime post_process --locked` 回归;抓帧验收(`ZR_RENDERDOC_CAPTURE_NEXT=1`)。

### 测试与验收清单

runtime 侧(`zircon_runtime/src/graphics/tests/render_compute.rs`、`gpu_readback_queue/tests.rs`、`render_graph/tests/`):

| 测试函数 | 断言 |
|---------|------|
| `render_compute_descriptor_lowering_declares_schema_io` | `into_feature_pass` 后每个 `BindingSchemaEntry` 在 pass IO 声明中可见,kind↔access 对应 |
| `render_compute_dispatch_from_buffer_requires_read_declaration` | `FromBuffer` 引用未声明 buffer → builder 校验 Err |
| `render_compute_per_pixel_groups_match_target_extent` | 1920×1080 + local 8×8 → groups (240,135,1);非整除尺寸 div_ceil |
| `render_compute_generic_executor_rejects_binding_collision` | schema 重号 → executor Err,不录制 |
| `render_compute_pipeline_cache_reuses_same_source_entry` | 同 (source 哈希, entry) 二次取 → 同 Arc,无重编译 |
| `render_compute_plugin_pass_absent_when_feature_disabled` | feature 关闭 → compiled graph 无 pass(§6.4) |
| `render_product_hzb_unchanged_after_compute_migration` | 迁移前基准 mip 链 readback == 迁移后(逐 texel;基准在切片 1.4 前录制) |
| `readback_callback_fires_after_n_frame_delay` | 帧 N 请求,模拟 poll:帧 N 不回调,帧 N+2 前回调且数据正确 |
| `readback_slot_reuse_blocks_until_map_complete` | 第 4 帧复用槽位且 map 未完 → 阻塞等待路径走通,计数上报 |
| `readback_ring_grows_to_fit_frame_requests` | 单帧 300 KiB 请求 → 槽容量 512 KiB;空闲 240 帧后回落 |
| `readback_external_buffer_request_supported` | external 入口回调送达(SVT feedback / 粒子计数兼容面,计划 13) |
| `readback_unmarked_buffer_rejected` | 未 `mark_readback` 的 `RgBufferHandle` → Err |
| `readback_no_private_map_async_source_scan` | 源码扫描:`map_async` 仅出现于 `gpu_readback_queue/` 与 `read_texture_rgba.rs` 白名单(对照 `surface_targets.rs:536` 既有手法) |

插件侧(`zircon_plugins/neural/runtime/src/tests/`、`features/post_process`):

| 测试函数 | 断言 |
|---------|------|
| `nn_model_asset_binary_roundtrip` | save→load 逐字段相等;权重 blob 256 对齐 |
| `nn_model_asset_rejects_bad_magic_or_version` | magic/version 错 → 拒载诊断 |
| `nn_onnx_convert_small_cnn_succeeds` | 2 层 conv+relu+pool+gemm 模型 → op 序列与期望逐条相等 |
| `nn_onnx_convert_unsupported_op_diagnostic` | 含 LSTM → 退出码 2,JSON 行含 node/op_type/reason |
| `nn_op_gemm_gpu_matches_cpu` | 含非 tile 整除尺寸 (M,K,N)=(17,33,5)、(64,64,64)、转置组合;容差见下 |
| `nn_op_conv2d_gpu_matches_cpu` / `nn_op_depthwise_conv2d_gpu_matches_cpu` | stride/padding/dilation 组合各一 |
| `nn_op_elementwise_parity` | Relu/Sigmoid/Tanh/Silu/Add/Mul/Sub/Div 共参数化 8 例 |
| `nn_op_pool_norm_upsample_parity` | MaxPool/AvgPool/BatchNorm/LayerNorm/Upsample2d |
| `nn_graph_executor_intermediate_tensors_are_transient` | pass 链中间 tensor 区间不重叠者复用同池条目(经计划 01 池统计) |
| `nn_graph_executor_view_ops_emit_no_pass` | Reshape/Slice/外维 Concat → pass 数不含之 |
| `nn_e2e_small_cnn_three_way_parity` | GPU == CPU == 离线 reference(容差内) |
| `nn_post_process_volume_descriptor_registered` | `VolumeComponentDescriptor` 出现在注册表,参数 schema 含 4 字段 |
| `nn_post_process_feature_off_emits_no_passes` | 关闭 → graph 无 `post.nn-effect` 及前后导 pass |

**对拍容差**:`|gpu - cpu| <= 1e-3 + 1e-3 * |cpu|`(逐元素,绝对+相对混合;f32 路径)。f16 权重路径以 f32 烘焙版为基准、同容差;超差时测试输出首个超差元素的 (索引, gpu, cpu, diff) 便于定位。GPU 测试在无适配器环境 skip(对照插件既有 GPU 测试的跳过手法),CI 至少跑 CPU↔离线一侧。

## 状态与产出记录

| 日期 | 里程碑/切片 | 状态 | 产出 | 验证与证据 | 后续 |
|------|-------------|------|------|------------|------|
| 2026-06-15 | CN-M1 compute framework | 部分完成: compute executor 分散存在,统一框架未落地 | HZB、SSAO、postprocess exposure、contact shadow、particle/other passes 已各自手写 compute dispatch;但无统一 descriptor、dispatch indirect、readback 和 resource validation 框架。 | 计划 04 VC-M3、计划 05 LS-M4、计划 07 PP-M3-S1b 状态表记录多个 compute executor 已接入;本文件 `现状与差距` 指出仍是 executor 内手写。 | 抽象 compute pipeline descriptor、bind layout contract、dispatch/readback helper 和 diagnostics。 |
| 2026-06-15 | CN-M2 NN operators / NN plugin skeleton and operator V1 | 未启动: 神经网络支持空白 | 无 NN graph、tensor resource、operator registry 或 model import。 | 本文件 `现状与差距` 明确神经网络完全空白。 | 建立 NN plugin crate、tensor buffer ABI、NN operators/basic ops 与 CPU reference tests。 |
| 2026-06-15 | CN-M3 graph executor and end-to-end inference | 未启动: 依赖 CN-M2 | 无图执行器、schedule、barrier 或 frame-synced inference。 | 当前无相关实现或状态表证据。 | 实施 graph compiler、resource planner、dispatch chain 和 readback/e2e tests。 |
| 2026-06-15 | CN-M4 NN postprocess integration | 未启动: 等待 CN-M1-M3 与计划 07 | 无 NN upscaler/denoiser/postprocess pass。 | 计划 07 后处理状态表显示 LUT/uber 后续仍未完成,NN postprocess 未进入实现。 | 在 postprocess graph 稳定后接 NN pass、history/input resource contract。 |

### 参考实现精读笔记

`dev/UnrealEngine/.../NNEHlslShaders/Internal/NNEHlslShadersOperator.h`:

- `EElementWiseUnaryOperatorType`(Abs/Acos/…/Relu/Sigmoid/Tanh/HardSigmoid/HardSwish/LeakyRelu/Selu/Softplus/Softsign/Sqrt 等 ~30 项)、`EElementWiseBinaryOperatorType`(Add/Div/Mod/Mul/Prelu/Pow/Sub)、`EElementWiseVariadicOperatorType`(Max/Min/Mean/Sum):UE 按"输入元数"而非语义分类,逐元素族共用一个 shader 框架。Zircon 对应:`nn_elementwise.wgsl` 单模板 + 函数体注入,同思路;V1 只取 Relu/Sigmoid/Tanh/Silu + 四则,远小于 UE 清单(裁剪依据:首场景为后处理 CNN)。
- 头内注释 `//Not, //And … need boolean tensors`、`//BitShift … need integer tensors`:UE 把非浮点 dtype 算子整体注释排除 —— 佐证本计划 V1 仅 f32/f16 的 dtype 收口是同款取舍。

`dev/UnrealEngine/.../NNEHlslShaders/NNEHlslShadersGemm.usf`:

- `#define WORK_TYPE float` + `READ(x)/WRITE(x)` 宏:精度与存取抽象一层,fp16 切换不改算法体。Zircon:WGSL 无预处理,由 `shader_templates.rs` 文本替换承担同职。
- ALGORITHM 0–3 朴素档(`[numthreads(8,8,1)/(16,16,1)/(32,32,1)/(256,1,1)]`)、4–6 共享内存档(`GROUP_SIZE 8/16/32`,`groupshared WORK_TYPE SharedMemoryA/B[GROUP_SIZE*GROUP_SIZE]`)、7+ 多载入档(`GROUP_SIZE_X 16/32` × `LOAD_PER_THREAD 16/8`):UE 按尺寸选档。Zircon 取舍:V1 只移植 ALGORITHM 5(TILE=16)单档 —— 多档选择是纯性能优化,等 RenderStats 计时数据再决定,接口(模板特化键)不锁死。
- 共享内存档核心:`NumGroupSteps = ceil(K / GROUP_SIZE)`;越界线程置 0 仍参与加载(`Temp = 0; if (… < M && … < K) Temp = READ(A[AIdx])`),`GroupMemoryBarrierWithGroupSync()` 两道(覆写前/使用前),最终 `if (DispatchThreadID.y >= M || x >= N) return;` 才出界返回,写回 `Alpha * Result + GetBetaTimesC(...)` —— `nn_gemm.wgsl` 逐条对应(见 GPU 布局节)。
- `StackShapeA_StackShapeB_StackStrideA_StackStrideB[MAX_NUM_STACK_DIMENSIONS]` + `GetMatrixStackOffsets(GroupID)`:batch/广播 GEMM 经 GroupID.z 索引堆叠矩阵。Zircon V1 不支持 batch GEMM(后处理 CNN 的 GEMM 在尾部全连接,batch=1),`NnOpParams` 预留 batch 字段,模板不实现。

`dev/UnrealEngine/.../NNERuntimeBasicCpu/Private/NNERuntimeBasicCpuModel.h`:

- `FModelCPU`:静态 `ModelMagicNumber`/`ModelVersionNumber` + `SerializationLoad(uint64& InOutOffset, TConstArrayView<uint8>)`/`SerializationSave`/`SerializationSize` 三段式游标序列化 —— `.znn` 的 magic/version 拒载与"格式即权威"取自此;Zircon 用定长 header + 表偏移代替游标递归(算子已扁平,无嵌套 Layer 树)。
- `FModelCPU::Layer: TSharedPtr<Private::ILayer>` 与 `FModelInstanceCPU::Instance: TSharedPtr<ILayerInstance>`:模型(权重,共享)与实例(中间缓冲,每实例)分离;`RunSync(TConstArrayView<FTensorBindingCPU>...)` + `SetInputTensorShapes`。Zircon 对应:`NnModelAsset`(共享)/ `run_cpu` 调用期临时 tensor 表(每调用),V1 不做实例池 —— CPU 档定位是回落与对拍,不追吞吐。
- `WeakThis: TWeakPtr<FModelCPU>` 保活手法:Rust `Arc` 语义天然覆盖,无对应物。

`dev/Graphics/.../universal/Runtime/MipGen/MipGenerator.cs`:

- `m_SupportCompute = SystemInfo.supportsComputeShaders` + compute/raster 双路径(`MipChainRasterBlurExecutePass` 用 `DrawProcedural` 兜底):Unity 为低端设备保留 raster 回落。Zircon 取舍:不做双路径 —— compute 能力缺失时 feature 经 capability gate 整体关闭(`backend_types.rs` 既有 `supports_async_compute`/`supports_neural_compute` 位),与风险节"能力检测"一致。
- `ComputePackedMipChainInfo`(注释 "We pack all MIP levels into the top MIP level to avoid the Pow2 MIP chain restriction"):mip 打包进单层回避尺寸限制。Zircon 不需要:计划 04 HZB 尺寸恒为 2 的幂(`next_pow2(view)/2`),无此约束。
- 每 mip 循环 `cmd.SetComputeTextureParam(data.cs, kernel, _Source, …, srcMipLevel)` + `cmd.DispatchCompute(cs, kernel, DivRoundUp(dstSize.x, 8), DivRoundUp(dstSize.y, 8), volumeDepth)`:最小 compute 封装形态 = (cs, kernel, 绑定表, DivRoundUp dispatch)四元组 —— `ComputePassDescriptor` 的 (shader, entry_point, bindings, dispatch) 四字段即其声明式等价,`PerPixel` 的 `div_ceil` 对应 `DivRoundUp`;区别:Unity 在 pass 回调里命令式逐 mip 重绑,Zircon 经 graph 声明每 mip 一个 pass(`HzbBuilder` 的 mip 链由计划 04 按 4 级一批展开),culling/生命周期可见性换少量 pass 开销。

## 风险与回退

- wgpu 无 fp16 storage 时算力受限:能力检测选 f32 路径,模型转换器可烘 f32;性能不达标先限低分辨率输入。
- 算子覆盖不足导致模型转换失败:转换器给出"不支持算子"清单诊断,V1 明确只支持声明的子集,不做静默近似。
- 推理与渲染同队列竞争帧预算:V1 同 encoder 串行 + 耗时统计;异步 compute 队列依赖 wgpu 多队列演进,接口不锁死。
