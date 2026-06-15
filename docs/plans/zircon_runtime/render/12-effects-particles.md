---
related_code:
  - zircon_runtime/src/graphics/particle_runtime_provider/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/mod.rs
  - zircon_plugins/rendering/plugin.toml
  - dev/UnrealEngine/Engine/Plugins/FX/Niagara/Source/Niagara/Private/NiagaraSystemSimulation.cpp
  - dev/UnrealEngine/Engine/Plugins/FX/Niagara/Source/Niagara/Private/NiagaraSystemGpuComputeProxy.cpp
  - dev/UnrealEngine/Engine/Plugins/FX/Niagara/Source/Niagara/Private/NiagaraSystemRenderData.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/PostProcessing/LensFlareCommonSRP.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/PostProcessing/LensFlareDataSRP.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/Passes/PostProcess/LensFlareDataDrivenPostProcessPass.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/Passes/PostProcess/LensFlareScreenSpacePostProcessPass.cs
  - dev/Graphics/Packages/com.unity.visualeffectgraph
  - dev/Fyrox/fyrox-impl/src/scene/particle_system/mod.rs
  - dev/Fyrox/fyrox-impl/src/scene/particle_system/draw.rs
  - dev/Fyrox/fyrox-impl/src/scene/particle_system/emitter/base.rs
  - dev/Fyrox/fyrox-impl/src/scene/sprite.rs
  - dev/Fyrox/fyrox-impl/src/scene/decal.rs
  - dev/bevy/crates/bevy_pbr/src/decal/forward.rs
  - dev/bevy/crates/bevy_pbr/src/decal/clustered.rs
  - dev/godot/servers/rendering/renderer_rd/storage_rd/particles_storage.cpp
  - dev/godot/scene/resources/3d/primitive_meshes.cpp
plan_sources:
  - .codex/plans/ZirconEngine Particles 插件完善计划.md
  - .codex/plans/Rendering 插件选项补齐计划.md
---

# 计划 12:特效渲染器与粒子(billboard / trail / projector / halo / lens flare)

## 目标

以计划 10 的自定义渲染器注册表为底座,补齐特效渲染器族,并把粒子渲染面与 CPU/GPU 模拟面的契约定稿:

1. `BillboardRenderer`:视面/视轴/自定义轴对齐,3D 中的公告板四边形(粒子与独立组件共用顶点构造)。
2. `TrailRenderer`:运动轨迹条带(顶点环形缓冲、宽度/颜色随生命周期曲线、对齐相机)。
3. `ProjectorRenderer`:投影贴花(视锥投影材质到场景),与 rendering.decals feature 收敛为同一实现的两种授权面。
4. `Halo` 与 `LensFlare`:光晕(光源处可见性衰减的公告板)与镜头光斑(数据驱动元素链 + 屏幕空间遮挡测试)。
5. 粒子:CPU 模拟(小规模/逻辑可读)与 GPU 模拟(compute,大规模)双档,渲染统一走 billboard/mesh/trail 三种粒子渲染模式;与既有 particles 插件、vfx_graph feature 收敛。

## 现状与差距

- `particle_runtime_provider` 与 particles 插件已有 emitter extract 与异步模拟雏形、vfx_graph feature 占位,但渲染模式单一,无 trail/mesh 粒子,GPU 模拟反馈(存活数 → draw args)未接 indirect。
- billboard/trail/projector/halo/lens flare 全部缺失。
- decals feature 与"projector"语义重复:需要收敛而不是两套。

## 参考代码

| 文件 | 应重点阅读 |
|------|-----------|
| `dev/UnrealEngine/.../Niagara/Private/NiagaraSystemSimulation.cpp` | CPU 模拟的批组织与生命周期(spawn/update 阶段切分) |
| `dev/UnrealEngine/.../Niagara/Private/NiagaraSystemGpuComputeProxy.cpp` | GPU 模拟代理:粒子状态 buffer 双缓冲、存活计数回读避免、间接 draw args 由 compute 写出 —— GPU 粒子与计划 03 indirect 的衔接样板 |
| `dev/UnrealEngine/.../Niagara/Private/NiagaraSystemRenderData.cpp` | 模拟与渲染解耦:renderer 按模拟输出构建 sprite/mesh/ribbon 绘制 |
| `dev/Graphics/.../core/Runtime/PostProcessing/LensFlareCommonSRP.cs` + `LensFlareDataSRP.cs` | 数据驱动 lens flare:元素链资产(位置比例/旋转/调制)、遮挡测试(深度采样计数)与绘制合成 |
| `dev/Graphics/.../universal/.../LensFlareScreenSpacePostProcessPass.cs` | 屏幕空间 flare(从 bloom 链派生)作为第二种实现档 |

次参考:`dev/bevy/crates/bevy_sprite`(billboard 顶点构造);`dev/Graphics/Packages/com.unity.visualeffectgraph`(GPU VFX 图的属性 buffer 布局,vfx_graph feature 远期)。

**Rust/wgpu 落地参照(防凭空实现)**:

| 文件 | 对应本计划机制 | 应重点阅读 |
|------|---------------|-----------|
| `dev/Fyrox/fyrox-impl/src/scene/particle_system/mod.rs` | 粒子系统组件契约(CPU 档) | Rust 引擎完整 CPU 粒子实现:粒子存储、emitter 列表、生命周期更新与 trail-free 的简洁组件面 —— `ParticleSimSource::Cpu` 的母本,重点 |
| `dev/Fyrox/fyrox-impl/src/scene/particle_system/draw.rs` | billboard 粒子顶点 ABI | `Vertex`(position/tex_coord/size/rotation/color 的 `#[repr(C)]` Pod)与 `VertexTrait::layout()` 声明 —— `BillboardInstanceData` 逐字段对拍的同类 |
| `dev/Fyrox/fyrox-impl/src/scene/particle_system/emitter/base.rs` | 发射器参数面 | `BaseEmitter`(:34,spawn_rate/max_particles/寿命与范围参数)的 Rust 契约与 spawn 逻辑 |
| `dev/Fyrox/fyrox-impl/src/scene/sprite.rs` | `BillboardRenderer` 独立组件 | 始终面向相机的 billboard 场景节点(:111)字段与 bounds 处理 |
| `dev/Fyrox/fyrox-impl/src/scene/decal.rs` | `ProjectorExtract` 组件面 | box 投影 decal 节点:`diffuse_texture`/`normal_texture`(:117/:120)+ layer 掩码(:63)的组件契约 |
| `dev/bevy/crates/bevy_pbr/src/decal/forward.rs` | decal forward(screen-space)回落档 | forward decal 的深度重建 + box clip Rust/wgpu 实现(配 `forward_decal.wgsl`)—— `decals.screen_space_composite` 同型 |
| `dev/bevy/crates/bevy_pbr/src/decal/clustered.rs` | decal clustered/deferred 档 | clustered decal 的 GPU 数据组织与剔除(配 `clustered.wgsl`)—— 两档 executor 共用投影函数的对照 |
| `dev/godot/servers/rendering/renderer_rd/storage_rd/particles_storage.cpp` | GPU 粒子模拟 + trail 数据流 | compute 粒子处理管线(`copy_pipelines`/`SortEffects`)、`trail_bind_pose_buffer`/`ParticlesFrameParams` 的 trail 缓冲组织(非 Rust,但为最完整的 GPU 粒子 + trail 落地;结构定义见同目录 `particles_storage.h`) |
| `dev/godot/scene/resources/3d/primitive_meshes.cpp` | `RibbonBuilder` 双排展开 | `RibbonTrailMesh::_create_mesh_array`(:2884)/`TubeTrailMesh::_create_mesh_array`(:2508):条带双排顶点 + 索引构造的可读样板 |

`LensFlareAsset`/halo(数据驱动元素链 + 屏幕空间遮挡测试)无 Rust 同类参照,实现时以 URP `LensFlareCommonSRP` 为唯一样板,按 index §8 第 8 条配对拍测试先行。

## 目标架构

归属:渲染器组件契约经计划 10 注册表;粒子模拟保持在 zircon_plugins/particles;halo/lens flare/projector 作为 rendering 插件 feature;billboard/trail 为内建渲染器(`scene_renderer/effects/`)。

核心设计:

- `BillboardRenderer`/`TrailRenderer`:`RendererTypeDescriptor` 注册;transparent queue 默认;trail 顶点在 prepare 期按历史位置环形缓冲展开,billboard 顶点 shader 侧展开(GeometrySource 之一,计划 08 模板)。
- `ProjectorRenderer` 与 decals 收敛:同一 deferred 投影实现(G-buffer 混合)+ forward 回落(逐对象重绘);组件面叫 Projector,feature 面叫 decals,文档明示同源。
- `LensFlareAsset`:元素链(纹理/沿轴位置/比例/颜色调制/旋转);遮挡测试用 HZB 点采样计数(计划 04 产物),无 readback;halo 是单元素 flare 的退化配置,不另做系统。
- 粒子契约:`ParticleSimOutput`(状态 buffer 句柄 + 存活计数 buffer + 渲染模式)→ 渲染端三模式(billboard 粒子 = instanced billboard;mesh 粒子 = 计划 03 instancing;trail 粒子 = ribbon 展开);GPU 档由 compute 写 indirect args(计划 03 batcher 的 args 槽)。

## 里程碑

### FX-M1 billboard 与 trail 渲染器

实施切片:
1. 两渲染器注册 + 顶点构造(billboard GeometrySource;trail 环形缓冲);生命周期曲线参数。

测试阶段:
- `cargo check -p zircon_runtime --lib --locked`;`cargo test -p zircon_runtime effects --locked` + `render_product`
- 验收证据:billboard 始终面向相机(多角度抓帧);trail 随运动生成条带且数量封顶。

### FX-M2 粒子渲染三模式与 GPU indirect

实施切片:
1. `ParticleSimOutput` 契约定稿;particles 插件对接;billboard/mesh/trail 三渲染模式。
2. GPU 模拟 compute 写 indirect args(依赖计划 03 GS-M4);CPU 档走常规提交。

测试阶段:
- `cargo test --manifest-path zircon_plugins/Cargo.toml -p <particles runtime crate> --locked` 与 runtime effects 范围
- 验收证据:万级 GPU 粒子无逐帧 readback(计数断言);CPU/GPU 档渲染产物一致(小规模对拍)。

### FX-M3 projector/decals 收敛

实施切片:
1. 统一投影实现;Projector 组件面;decals feature 改为引用同一实现;删除重复路径。

测试阶段:
- `cargo test -p zircon_runtime --lib --locked` 范围 + 插件测试
- 验收证据:deferred 与 forward 回落产物一致;feature 关闭时 graph 无投影 pass。

### FX-M4 lens flare 与 halo

实施切片:
1. `LensFlareAsset` + 数据驱动绘制 pass(transparent 后);HZB 遮挡衰减;halo 预设。

测试阶段:
- `cargo test -p zircon_runtime effects --locked`
- 验收证据:光源被遮挡时 flare 平滑消隐(遮挡比例 readback 断言);元素链沿屏幕轴分布正确。

## 工程落地细化

本章是计划 12 的实施权威(index.md §8 第 7 条)。bind group 槽位、std430、`zr_` include、queue 数值段、测试命名等全局约定直接引用 index.md §8,本章不重定义。跨计划契约原样消费:计划 01 `RgTextureHandle`;计划 03 `GpuScene`/`IndirectDrawBatcher`(`draw_indexed_indirect` 5 词 args,`instance_count` 在 word1);计划 04 `HzbBuilder`(furthest 链,R32Float);计划 09 `RenderQueueValue`/`RenderLayer` 与 `sort_key`;计划 10 `RendererCommon`/`RendererTypeDescriptor`;计划 16 `GpuReadbackQueue`(仅测试档)。

### 模块与文件落点

新增文件(`zircon_runtime` 侧,facade 固定 `zircon_runtime::core::framework::render`,不新增 crate):

| 路径 | 内容 |
|------|------|
| `zircon_runtime/src/core/framework/render/effects/mod.rs` | 契约模块声明 + curated re-export(`BillboardExtract`、`TrailExtract`、`ProjectorExtract`、`LensFlareAsset`、`ParticleSimOutput`),保持 thin |
| `zircon_runtime/src/core/framework/render/effects/billboard.rs` | `BillboardExtract`、`BillboardAlignMode` |
| `zircon_runtime/src/core/framework/render/effects/trail.rs` | `TrailExtract`、`TrailPoint`、`TrailUvMode`、宽度/颜色曲线采样 |
| `zircon_runtime/src/core/framework/render/effects/projector.rs` | `ProjectorExtract`(box decal 组件面契约,decals feature 同源消费) |
| `zircon_runtime/src/core/framework/render/effects/lens_flare.rs` | `LensFlareAsset`、`LensFlareElement`、`halo_preset` 构造器 |
| `zircon_runtime/src/core/framework/render/effects/particle_sim_output.rs` | `ParticleSimOutput`、`ParticleRenderMode`、`ParticleSimSource`、state buffer stride/offset 常量 |
| `zircon_runtime/src/graphics/scene/scene_renderer/effects/mod.rs` | 模块声明 + `effects_renderer_type_descriptors()`(billboard/trail 经计划 10 注册表注册的统一入口) |
| `zircon_runtime/src/graphics/scene/scene_renderer/effects/billboard/mod.rs` | 模块声明 |
| `zircon_runtime/src/graphics/scene/scene_renderer/effects/billboard/billboard_renderer.rs` | `BillboardRenderer`:prepare(实例打包)+ pass processor 工厂;transparent queue 默认 |
| `zircon_runtime/src/graphics/scene/scene_renderer/effects/billboard/instance_buffer.rs` | `BillboardInstanceData` `#[repr(C)]` Pod 镜像 + storage buffer 增长策略(与 WGSL 节逐字段对拍) |
| `zircon_runtime/src/graphics/scene/scene_renderer/effects/trail/mod.rs` | 模块声明 |
| `zircon_runtime/src/graphics/scene/scene_renderer/effects/trail/trail_renderer.rs` | `TrailRenderer`:prepare 期消费环带快照、提交双排顶点 |
| `zircon_runtime/src/graphics/scene/scene_renderer/effects/trail/ribbon_builder.rs` | `RibbonBuilder`:点环带 → 双排顶点 + 索引,V1 纯 CPU,封顶与稀疏采样 |
| `zircon_runtime/src/graphics/scene/scene_renderer/effects/shaders/zr_billboard.wgsl` | 共享 include:`zr_billboard_corner` / `zr_billboard_expand`,只含函数与 struct,无 entry point(§8 第 3 条) |
| `zircon_runtime/src/graphics/scene/scene_renderer/effects/shaders/billboard.wgsl` | billboard 顶点/片元 entry(顶点阶段 expand,见 WGSL 节) |
| `zircon_runtime/src/graphics/scene/scene_renderer/effects/shaders/trail.wgsl` | trail 双排顶点着色(UV 滚动、宽度已在 CPU 烘进顶点) |

新增文件(`zircon_plugins` 侧):

| 路径 | 内容 |
|------|------|
| `zircon_plugins/rendering/features/lens_flare/runtime/Cargo.toml` | crate `zircon_plugin_rendering_lens_flare_runtime`(对齐 decals feature crate 形态) |
| `zircon_plugins/rendering/features/lens_flare/runtime/src/lib.rs` | `FEATURE_ID = "rendering.lens_flare"`、`RuntimePluginFeature` 实现、`render_feature_descriptor()`、executor 注册(对齐 `zircon_plugin_rendering_decals_runtime` 的 `register` 三连) |
| `zircon_plugins/rendering/features/lens_flare/runtime/src/executors.rs` | `lens_flare.occlusion`(compute)与 `lens_flare.draw`(graphics)两个 pass executor 契约 + 资源 IO 声明 |
| `zircon_plugins/rendering/features/lens_flare/runtime/src/shaders/lens_flare.wgsl` | 遮挡 compute entry + element 绘制 entry |
| `zircon_plugins/rendering/features/lens_flare/editor/` | crate `zircon_plugin_rendering_lens_flare_editor`:`LensFlareAsset` 面板与 halo 预设 |
| `zircon_plugins/particles/runtime/src/render/sim_output.rs` | 由 CPU/GPU 模拟产出构建 `ParticleSimOutput`(双档同一出口) |

修改文件:

| 路径 | 改动要点 |
|------|---------|
| `zircon_runtime/src/core/framework/render/mod.rs` | 挂载 `effects` 契约子模块 |
| `zircon_runtime/src/core/framework/render/frame_extract.rs` | `ParticleExtract` 增加 `sim_outputs: Vec<ParticleSimOutput>`;新增 effects extract 段(billboards/trails/projectors/lens_flares),schema 经计划 10 `RendererTypeDescriptor` 声明 |
| `zircon_runtime/src/core/framework/render/scene_extract.rs` | `BillboardExtract`/`TrailExtract`/`ProjectorExtract` 复合 `RendererCommon`(计划 10 字段不重复定义) |
| `zircon_runtime/src/core/framework/render/backend_types.rs` | `RenderStats` 增加 `last_trail_vertices`、`last_billboard_instances`、`last_flare_occlusion_dispatches`、`last_particle_indirect_writes` |
| `zircon_runtime/src/graphics/scene/scene_renderer/mod.rs` | 挂载 `effects` 子模块 |
| `zircon_runtime/src/graphics/particle_runtime_provider/gpu_feedback.rs`、`runtime_feedback.rs` | `ParticleGpuFeedback` 改为只承载诊断档(`GpuReadbackQueue` 路径),稳态帧不再携带 readback 输出 |
| `zircon_plugins/particles/runtime/src/render/extract.rs` | `build_particle_extract` 改为输出 `ParticleSimOutput`(渲染模式、buffer 资源名、indirect 槽) |
| `zircon_plugins/particles/runtime/src/render/executors.rs` | `particles.gpu.indirect-draw-args` 写入改为计划 03 `IndirectDrawBatcher` 分配的 args 槽(word1);`particles.gpu.debug-readback` 从常驻资源降级为测试档可选 |
| `zircon_plugins/particles/runtime/src/render/gpu/readback.rs` | `ParticleGpuCounterReadback` / `ParticleGpuReadbackRequest` 收编进计划 16 `GpuReadbackQueue`,仅 CPU/GPU 对拍测试使用 |
| `zircon_plugins/particles/runtime/src/simulation/cpu.rs` | CPU 档输出从 `ParticleSpriteSnapshot` 专用路径改为同一 `ParticleSimOutput`(`ParticleSimSource::Cpu`) |
| `zircon_plugins/rendering/features/decals/runtime/src/lib.rs` | `DecalProjectorDescriptor` 对齐 `ProjectorExtract` 字段;`EXECUTOR_ID = "decals.projector-composite"` 拆分为 deferred / screen-space 两 executor(同一 WGSL 投影函数) |
| `zircon_plugins/rendering/plugin.toml` | 新增 `[[optional_features]] id = "rendering.lens_flare"` 与对应 `[[modules]]`(runtime/editor 两条) |

### 核心类型与接口

契约层(`core/framework/render/effects/`,不得 import wgpu;GPU buffer 以 graph external 资源名引用,沿用既有 `particles.gpu.*` 命名,计划 01 的 buffer 句柄族落地后切句柄):

```rust
// billboard.rs
pub enum BillboardAlignMode {
    /// 面向视平面(用 view 矩阵 right/up,所有 billboard 平行)
    ViewPlane,
    /// 面向相机点(逐实例朝向相机位置)
    ViewPoint,
    /// 轴锁定:绕 axis 旋转面向相机(火焰柱、树 imposter)
    AxisLocked { axis: [f32; 3] },
}

pub struct BillboardExtract {
    pub common: RendererCommon,        // 计划 10:layer_mask/queue override/...
    pub position: [f32; 3],
    pub size: [f32; 2],
    pub rotation_radians: f32,
    pub color: [f32; 4],
    pub align: BillboardAlignMode,
    pub material: RenderMaterialKey,   // 既有材质键,默认走 Transparent=3000 队列
}

// trail.rs
pub struct TrailPoint {
    pub position: [f32; 3],
    pub spawn_time_seconds: f32,       // 用于生命周期曲线与 UV 滚动
}

pub enum TrailUvMode {
    /// V1 必做:U 沿全长 0..1 拉伸
    Stretch,
    /// U 按世界距离平铺
    TilePerMeter { tiles_per_meter: f32 },
}

pub struct TrailExtract {
    pub common: RendererCommon,
    /// 环形缓冲快照:runtime 侧维护 ring buffer of points,extract 时按时间序展平
    pub points: Vec<TrailPoint>,
    pub max_points: u32,               // 默认 64,硬上限 256(风险节封顶策略)
    pub min_vertex_distance: f32,      // 距离稀疏采样阈值,小于则不记新点
    pub lifetime_seconds: f32,
    pub width_curve: [f32; 8],         // 归一化生命周期 8 段折线
    pub color_curve: [[f32; 4]; 4],    // 4 键颜色渐变
    pub uv_mode: TrailUvMode,
    pub uv_scroll_per_second: f32,
    pub material: RenderMaterialKey,
}

// projector.rs(组件面;decals feature 是同一实现的授权面)
pub struct ProjectorExtract {
    pub common: RendererCommon,
    /// 世界 → decal box 单位空间(box 内为 [-0.5, 0.5]^3)
    pub world_to_box: [[f32; 4]; 4],
    pub opacity: f32,
    pub normal_blend: f32,             // 与 DecalProjectorDescriptor::normal_blend 同义
    pub angle_fade_degrees: f32,       // 表面法线与投影方向夹角衰减
    pub sort_order: i32,               // 同位置多 decal 叠序
    pub material: RenderMaterialKey,
}

// lens_flare.rs
pub struct LensFlareElement {
    pub texture: RenderTextureKey,
    /// 沿"光源屏幕位置 → 屏幕中心镜像"轴:0=光源处,1=中心,负值/大于 1 允许(对齐 Unity position)
    pub axis_position: f32,
    pub position_offset: [f32; 2],     // 屏幕空间附加偏移
    pub uniform_scale: f32,
    pub size: [f32; 2],
    pub rotation_degrees: f32,
    pub auto_rotate: bool,             // 沿轴自动转向
    pub tint: [f32; 4],
    pub intensity: f32,
    pub modulate_by_light_color: bool,
}

pub struct LensFlareAsset {
    pub elements: Vec<LensFlareElement>,
    pub occlusion_radius_world: f32,   // 遮挡测试圆盘半径(世界空间,投到屏幕)
    pub occlusion_sample_count: u32,   // HZB 圆盘采样数,默认 8,上限 32
    pub fade_speed: f32,               // 可见因子时域平滑速率(1/秒)
    pub allow_off_screen: bool,
}

impl LensFlareAsset {
    /// halo = 单元素退化配置:axis_position=0、auto_rotate=false、无链
    pub fn halo_preset(texture: RenderTextureKey, size: f32, tint: [f32; 4]) -> Self;
}

// particle_sim_output.rs —— 模拟面(插件)与渲染面(runtime)的唯一契约
pub enum ParticleRenderMode {
    Billboard { align: BillboardAlignMode },
    Mesh { mesh: RenderMeshKey },          // 计划 03 instancing 消费
    Trail { uv_mode: TrailUvMode, width_curve: [f32; 8] },
}

pub enum ParticleSimSource {
    /// CPU 档:已展平的实例数组,prepare 期直接打包进 billboard 实例 buffer
    Cpu { instances: Vec<CpuParticleInstance> },
    /// GPU 档:全部以 graph external buffer 资源名引用,渲染端零 readback
    Gpu {
        state_buffer: String,          // "particles.gpu.particles-b"(当帧输出侧)
        alive_index_buffer: String,    // "particles.gpu.alive-indices"
        counter_buffer: String,        // "particles.gpu.counters"
        /// 计划 03 IndirectDrawBatcher 分配的 args 槽(5 词布局),compact 后由
        /// particles.write_indirect 把 alive_count 写进该槽 word1
        indirect_slot: IndirectArgsSlot,
    },
}

pub struct ParticleSimOutput {
    pub emitter_id: String,
    pub capacity: u32,                 // <= PARTICLE_GPU_MAX_PARTICLES
    pub render_mode: ParticleRenderMode,
    pub source: ParticleSimSource,
    pub bounds: RenderBoundsSnapshot,  // 计划 04 视锥剔除用;GPU 档为保守包围盒
    pub material: RenderMaterialKey,
}
```

实现归属:`BillboardRenderer`/`TrailRenderer` 在 `scene_renderer/effects/`(runtime 内建);decal 两档 executor 在 `zircon_plugin_rendering_decals_runtime`;flare 两 executor 在 `zircon_plugin_rendering_lens_flare_runtime`;模拟与 `ParticleSimOutput` 的构建在 `zircon_plugin_particles_runtime`。四类渲染器一律经 `RendererTypeDescriptor` 注册,不在 scene_renderer 加硬编码枚举分支。

### GPU 数据布局与 WGSL 约定

**particle state buffer**(std430 storage,双缓冲 `particles.gpu.particles-a/b`,stride 64):

| 字段 | 类型 | 偏移 |
|------|------|------|
| `position` | `vec3<f32>` | 0 |
| `age_seconds` | `f32` | 12 |
| `velocity` | `vec3<f32>` | 16 |
| `lifetime_seconds` | `f32` | 28 |
| `color` | `vec4<f32>` | 32 |
| `size` | `vec2<f32>` | 48 |
| `rotation_radians` | `f32` | 56 |
| `flags_seed` | `u32`(低 8 位 flags,高 24 位 RNG seed) | 60 |

`counter_buffer`(`particles.gpu.counters`):word0 = `alive_count`、word1 = `dead_count`、word2 = `spawn_request`、word3 = padding。`alive_index_buffer` 为 `array<u32, capacity>`,compact pass 压实写入。

**indirect args**:布局唯一归计划 03(`draw_indexed_indirect` 5 词:`[index_count, instance_count, first_index, base_vertex, first_instance]`)。`particles.write_indirect` compute 只做一件事:`args[slot + 1] = counters[0]`(alive_count → word1);word0 由 CPU 在槽分配时一次写死(billboard=6,mesh=该 mesh index_count),word2..4 同理。本计划不自定义 args 布局。

**flare 可见因子 buffer**(`effects.flare.visibility`,storage,RW,stride 16,`ZR_MAX_FLARE_SLOTS = 64`,共 1 KiB):

| 字段 | 类型 | 偏移 |
|------|------|------|
| `visible_factor` | `f32`(0..1,本帧 HZB 采样可见率) | 0 |
| `smoothed_factor` | `f32`(按 `fade_speed` 指数平滑后的消费值) | 4 |
| `last_frame_index` | `u32` | 8 |
| `_pad` | `u32` | 12 |

遮挡 compute(每 flare 一线程,1 workgroup 即可):把光源世界位投到屏幕 → 在 `occlusion_radius` 屏幕圆盘上取 `occlusion_sample_count` 个 HZB furthest 链样本(R32Float,mip 按圆盘像素尺寸选,`HzbBuilder` 产物经 group1 绑定)→ 通过比例写 `visible_factor` 并就地更新 `smoothed_factor`。flare 绘制顶点着色器读同一 buffer 按 slot 调制 alpha —— 全程无 readback;`GpuReadbackQueue`(计划 16)仅在验收测试中读 `smoothed_factor` 断言。

**decal pass 绑定**(两档共用投影函数 `zr_decal_project`,放 `zr_decal.wgsl` include):

- group0:frame/view(§8 固定,binding 编号沿用既有 SRP 布局)。
- group1(pass 级):`b0` = scene depth(`texture_depth_2d`,screen-space 档采样重建世界坐标;deferred 档同号绑定,另以 attachment 写 G-buffer)、`b1` = depth sampler(non-filtering)。
- group2(material 级):`b0` = decal uniform(`world_to_box` 矩阵、`opacity`、`normal_blend`、`angle_fade`)、`b1` = decal albedo texture、`b2` = sampler。
- group3(instance 级):`b0` = decal 实例 storage buffer(多 decal 一次 instanced box 提交)。

片元逻辑:depth → 世界坐标 → `world_to_box` → 任一分量超 [-0.5,0.5] 则 `discard` → box XY 作投影 UV 采样。screen-space 档混到 scene color(SrcAlpha/OneMinusSrcAlpha);deferred 档写 G-buffer albedo/normal(normal 按 `normal_blend` 插值)。

**billboard 顶点展开:结论 = WGSL 顶点阶段 expand,不做 CPU quad 生成。** 每实例固定 6 顶点(`draw(6, instance_count)`),`vertex_index` 查 `zr_billboard_corner` 常量表得 [-0.5,0.5]² 角点,`zr_billboard_expand(center_ws, size, rotation, align_mode, view)` 在顶点阶段按对齐模式算世界偏移。实例数据走 group3 storage buffer(`BillboardInstanceData`,stride 48:position+rotation / size+align_axis_xy / color 三个 vec4)。GPU 粒子 billboard 复用同一 entry,实例数据改读 particle state buffer + alive index 间接寻址(`state[alive[instance_index]]`),由 shader 模板条件拼接(计划 08 GeometrySource)。与 sprite 批管线的关系:sprite(2D/屏幕空间)保持既有 prepared batch 路径不动,billboard 是 3D transparent queue 渲染器,仅共享 §8 槽位与计划 09 sort_key,不共享顶点构造 —— 两者合并留给计划 14 评估,本计划不动 sprite。

**trail**:V1 CPU 生成。`RibbonBuilder` 把 N 点环带展开为 2N 顶点(每点沿 `normalize(cross(tangent, view_dir))` ± 半宽偏移;宽度 = `width_curve` 按 `(now - spawn_time)/lifetime` 采样)与 `6(N-1)` 索引;UV.x 按 `TrailUvMode`,UV.y ∈ {0,1} 双排。顶点 buffer 每 trail 上限 `2 * max_points`,index buffer 静态可复用。GPU 展开(对照 Niagara `GenerateIndexBufferForView` 的 per-view 路径)明确留远期,不在 V1 范围。

### 帧时序与集成点

帧内顺序(全部为 graph 节点,经 RenderFeature descriptor / `RendererTypeDescriptor` 接入,无旁路):

1. **extract**:runtime 写 `RenderFrameExtract.particles.sim_outputs` 与 effects extract 段;trail 环带在 runtime 侧组件状态中维护,extract 仅快照(渲染模块不回访 ECS,§6 第 6 条)。
2. **compute lane(opaque 之前)**:`particles.spawn_update` → `particles.compact` → `particles.write_indirect`(写计划 03 args 槽 word1)。与 `gpu_scene_upload`(计划 03)无资源依赖,可同 lane 顺序排布。
3. **`hzb_build`(计划 04)之后**:`lens_flare.occlusion` compute(读 HZB furthest 链 + flare slot uniform,写 `effects.flare.visibility`)。
4. **decal**:deferred 路径节点 `decals.gbuffer_project` 位于 gbuffer base pass 之后、deferred lighting 之前;forward 路径节点 `decals.screen_space_composite` 位于 opaque 之后、transparent 之前。feature 关闭时 compiled graph 不含任一节点(§6 第 4 条)。
5. **transparent phase**:billboard / trail / 粒子三模式经各自 pass processor 入队,排序统一走计划 09 `sort_key`(queue 段 Transparent=3000);GPU 粒子提交 = `IndirectDrawBatcher` 槽的 `draw_indexed_indirect`,CPU 档常规 instanced 提交,同一 shader ABI。
6. **`effects.lens_flare_draw`**:transparent 之后、post-process 链入口之前(flare 进 bloom 输入,对齐 URP 数据驱动档的链位)。
7. **present 前**:`RenderStats` 写入本章新增四字段。

**硬切换删除项**(各自里程碑内完成,不留双路径):

- FX-M2:`zircon_runtime/src/graphics/scene/scene_renderer/particle/` 整目录删除(`ParticleRenderer`、`build_particle_vertices`、`particle_vertex` 的 CPU per-particle quad 路径),粒子渲染统一走 effects 三模式;其 mod.rs 测试中 world-HUD billboard 的 depth-read-only / transparent-blend 断言迁移为 `render_billboard_*` 测试。
- FX-M2:`particle_runtime_provider` 的逐帧 readback 反馈链(`ParticleGpuFeedback` 携带 `RenderParticleGpuReadbackOutputs` 的稳态路径)删除,改诊断档;`particles.gpu.debug-readback` 移出常驻 `INDIRECT_RESOURCES`。
- FX-M3:`zircon_plugin_rendering_decals_runtime` 中与 `ProjectorExtract` 重复的私有投影描述字段删除,`DecalProjectorDescriptor` 只保留 feature 面配置;旧单一 `decals.projector-composite` executor 由两档 executor 取代。

### 实施切片细化

**FX-M1 billboard 与 trail 渲染器**

| 切片 | 触碰文件 | 改动要点 | 完成判据 |
|------|---------|---------|---------|
| M1-S1 契约 | `effects/{mod,billboard,trail}.rs`、`frame_extract.rs`、`scene_extract.rs` | 三契约类型 + extract 段;复合 `RendererCommon` | `cargo check -p zircon_runtime --lib --locked` 过;契约单测过 |
| M1-S2 billboard | `effects/billboard/*`、`shaders/zr_billboard.wgsl`、`shaders/billboard.wgsl` | 顶点阶段 expand 三对齐模式;实例 storage buffer;`RendererTypeDescriptor` 注册 | 数学单测(角点/轴锁)过;graph 含 transparent 注入 |
| M1-S3 trail | `effects/trail/*`、`shaders/trail.wgsl` | `RibbonBuilder` 双排展开、宽度/颜色曲线、UV 两模式、封顶与 `min_vertex_distance` | 顶点数 = 2N、索引数 = 6(N-1) 断言;封顶断言 |
| M1-S4 验收 | — | `cargo test -p zircon_runtime effects --locked` + `render_product`;多角度抓帧 | billboard 面向相机;trail 条带正确且数量封顶 |

**FX-M2 粒子三模式与 GPU indirect**

| 切片 | 触碰文件 | 改动要点 | 完成判据 |
|------|---------|---------|---------|
| M2-S1 契约定稿 | `effects/particle_sim_output.rs`、`particles/runtime/src/render/{extract,sim_output}.rs`、`simulation/cpu.rs` | `ParticleSimOutput` 双档同一出口;CPU 档弃 sprite 专用路径 | 插件单测过;CPU 档经新契约出图 |
| M2-S2 GPU indirect | `particles/runtime/src/render/executors.rs`、`render/gpu/*` | state stride 64 对齐本章布局;`write_indirect` 写计划 03 槽 word1;readback 收编 `GpuReadbackQueue` 测试档 | 稳态帧 readback 次数 = 0(统计断言) |
| M2-S3 三模式消费 | `effects/billboard/*`(粒子间接寻址变体)、`effects/trail/*`、mesh 走计划 03 instancing 接口 | billboard=instanced 间接寻址;mesh=GpuScene 实例;trail=每粒子环带(上限收紧) | 三模式 `render_product` 各一场景 |
| M2-S4 硬切换 | `scene_renderer/particle/` 删除、`particle_runtime_provider` 改造 | 见删除项清单 | 全仓无旧符号引用;`cargo test -p zircon_runtime --lib --locked` 过 |

**FX-M3 projector/decals 收敛**

| 切片 | 触碰文件 | 改动要点 | 完成判据 |
|------|---------|---------|---------|
| M3-S1 契约与 WGSL | `effects/projector.rs`、decals crate `src/shaders/zr_decal.wgsl` | `ProjectorExtract`;`zr_decal_project` 共用函数 | box 内外判定单测(CPU 镜像)过 |
| M3-S2 两档 executor | decals crate `src/{lib,executors}.rs` | deferred G-buffer 写 + screen-space 合成;绑定按本章编号;删旧单 executor | 两档 `render_product` 一致(容差内) |
| M3-S3 feature gate | `rendering/plugin.toml`、graph 编译测试 | 组件面/feature 面同源;关 feature 无 pass | `render_decal_feature_off_removes_graph_pass` 过 |

**FX-M4 lens flare 与 halo**

| 切片 | 触碰文件 | 改动要点 | 完成判据 |
|------|---------|---------|---------|
| M4-S1 资产契约 | `effects/lens_flare.rs` | `LensFlareAsset`/`LensFlareElement`/`halo_preset` | 元素轴分布纯函数单测过 |
| M4-S2 遮挡 compute | lens_flare crate `src/{executors,shaders/lens_flare.wgsl}` | HZB 圆盘采样 → 可见因子 buffer(stride 16);时域平滑 | buffer 布局断言;遮挡场景因子趋 0(`GpuReadbackQueue` 测试档) |
| M4-S3 绘制 pass | 同上 + graph 链位 | element 链沿轴展开绘制,读因子调制;halo 预设 | 抓帧:遮挡平滑消隐、链分布正确 |
| M4-S4 HZB 回落 | executors.rs | HZB 不可用档退中心单点深度采样 | 低端档 capability 测试过 |

### 测试与验收清单

单测(`render_<topic>_*`,§8 第 6 条;位置 = 对应模块 `#[cfg(test)]` 或插件 `src/tests/`):

| 测试函数 | 断言要点 | 位置 |
|---------|---------|------|
| `render_billboard_view_plane_uses_view_axes` | ViewPlane 模式角点偏移 = view right/up 线性组合 | `effects/billboard/billboard_renderer.rs` |
| `render_billboard_axis_locked_preserves_axis` | AxisLocked 展开后四角点轴向分量与锁轴一致 | 同上 |
| `render_billboard_stays_transparent_depth_read_only` | pipeline 状态:depth write off、LessEqual、SrcAlpha 混合(迁移自旧 particle/mod.rs 断言) | `effects/billboard/mod.rs` |
| `render_billboard_registers_via_renderer_type_descriptor` | 注册表含 billboard/trail 描述符,scene_renderer 无枚举分支 | `effects/mod.rs` |
| `render_trail_ribbon_double_row_counts` | N 点 → 2N 顶点、6(N-1) 索引 | `effects/trail/ribbon_builder.rs` |
| `render_trail_ring_buffer_caps_points` | 超 `max_points` 后丢最旧点;`min_vertex_distance` 内不记点 | 同上 |
| `render_trail_width_and_color_curve_sampling` | 曲线端点/中点采样值 | 同上 |
| `render_trail_uv_stretch_and_tile_modes` | Stretch 端点 0/1;TilePerMeter 按累计距离 | 同上 |
| `render_particle_sim_output_state_stride_is_64` | Rust Pod 镜像 `size_of`/offset 与本章表逐项相等 | `effects/particle_sim_output.rs` |
| `render_particle_indirect_writes_alive_count_to_word1` | write_indirect 后槽 word1 = counters word0,word0 不被触碰 | `zircon_plugin_particles_runtime` `src/tests/` |
| `render_particle_gpu_steady_state_has_zero_readback` | 稳态 100 帧 readback 提交计数 = 0 | 同上 |
| `render_particle_cpu_gpu_parity_small_scene` | 同种子小规模 CPU/GPU 输出实例对拍(经 `GpuReadbackQueue` 测试档) | 同上 |
| `render_decal_box_clip_rejects_outside_world_positions` | `zr_decal_project` CPU 镜像:box 外 discard、box 内 UV 正确 | decals crate `src/tests/` |
| `render_decal_two_tier_executors_share_projection` | 两 executor 声明引用同一 include;graph 链位正确 | 同上 |
| `render_decal_feature_off_removes_graph_pass` | feature 关闭 → compiled graph 无 `decals.*` 节点 | `zircon_runtime` graph 编译测试旁 |
| `render_flare_visibility_buffer_stride_is_16` | slot 布局断言 | lens_flare crate `src/tests/` |
| `render_flare_element_axis_distribution_matches_position` | axis_position 0/0.5/1 的屏幕坐标纯函数对拍 | 同上 |
| `render_flare_halo_preset_is_single_element_at_source` | `halo_preset` 元素数 = 1 且 axis_position = 0 | 同上 |
| `render_flare_occlusion_factor_drops_when_blocked` | 遮挡场景 `smoothed_factor` 单调下降至 < 0.05 | 同上(`GpuReadbackQueue` 测试档) |

产物对拍(`render_product_*` + `ZR_RENDERDOC_CAPTURE_NEXT=1` 人工抓帧):`render_product_effects_billboard_trail`(多角度 billboard + 运动 trail)、`render_product_particle_three_modes`(同发射器 billboard/mesh/trail)、`render_product_particle_cpu_gpu_match`(小规模双档同图)、`render_product_decal_box_forward_deferred`(两档一致)、`render_product_lens_flare_occlusion`(遮挡前后两帧)。

命令:切片期 `cargo check -p zircon_runtime --lib --locked`;里程碑期 `cargo test -p zircon_runtime effects --locked`、`cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_particles_runtime --locked`、`-p zircon_plugin_rendering_decals_runtime`、`-p zircon_plugin_rendering_lens_flare_runtime`。

## 状态与产出记录

| 日期 | 里程碑/切片 | 状态 | 产出 | 验证与证据 | 后续 |
|------|-------------|------|------|------------|------|
| 2026-06-15 | FX-M1 billboard and trail renderers | 部分完成: billboard/particle sprite 路径存在,trail renderer 未落地 | Particle transparent pass、billboard basis、particle previous-state/velocity writer 已由计划 06 大量使用;trail renderer、mesh particle renderer 和独立 effect renderer family 仍未实现。 | 计划 06 TP-M1-S9..S23 与 TP-M4-S4e 状态表记录 particle transparent pass、previous state、billboard basis、velocity writer 和产品过滤测试。 | 补 trail strip geometry、mesh particle mode 和 renderer family registry 对接。 |
| 2026-06-15 | FX-M2 particle modes and GPU indirect | 部分完成: CPU/renderer-owned 粒子状态与 velocity 已完成多项基线,GPU indirect 未完成 | 已有 particle transparent pass、stable sprite identity、renderer-owned previous rows、previous billboard basis、scene-velocity writer、readback/product tests;GPU simulation feedback 到 indirect args 仍未接入。 | 计划 06 状态表记录 `render_product_particle_velocity` 过滤产品测试、`render_product_taa_particle_transparent_pass_contributes_before_resolve` 通过。 | 接入 GPU simulation alive count、indirect draw args 与 per-view particle sorting。 |
| 2026-06-15 | FX-M3 projector/decals convergence | 未启动: 仍需语义合并 | decals feature 与 projector 语义仍重复,尚未统一到一个 renderer/effect contract。 | 本文件 `现状与差距` 明确 decals/projector 需要收敛。 | 定义 projector/decal 共同数据模型、receiver filtering 与 deferred/forward 消费路径。 |
| 2026-06-15 | FX-M4 lens flare and halo | 未启动: 仍为后续计划 | halo/lens flare 缺失,无 occlusion query、screen-space placement 或 asset contract。 | 本文件 `现状与差距` 明确 billboard/trail/projector/halo/lens flare 全部缺失。 | 等 camera ordering、light data 和 post chain 稳定后实施 screen-space flare pass。 |

### 参考实现精读笔记

**`LensFlareCommonSRP.cs`(Unity SRP core)**:遮挡走专用 `occlusionRT`(`RTHandle`,宽 = `maxLensFlareWithOcclusion` = 128,即每 flare 占 1 像素列;高 = `maxLensFlareWithOcclusionTemporalSample` = 8 + `mergeNeeded`,做跨帧"一致投票"合并)。`ComputeOcclusion(...)` 对每个 `LensFlareCompInfo` 设 `FLARE_COMPUTE_OCCLUSION` keyword、打包 `_FlareData0/2/3` 全局量后,把 viewport 设为 `(x = info.index, w = 1, h = 1)` 用材质 pass `"LensFlareOcclusion"` 画 1×1 —— 即"GPU 写可见因子、绘制时 GPU 读"零 readback 方案的纹理版。元素放置由 `GetFlareData0(screenPos, translationScale, rayOff0, vLocalScreenRatio, angleDeg, position, angularOffset, positionOffset, autoRotate)` 打包;光源形状衰减按灯型分函数(`ShapeAttenuationDirLight` / `ShapeAttenuationSpotConeLight` / `ShapeAttenuationAreaTubeLight` 等)。**Zircon 对应**:可见因子用 storage buffer(stride 16 槽)替代 1×N RT —— 我们无 XR slice 与材质 pass 约束,compute 直采 `HzbBuilder` furthest 链比 Unity 的深度比较绘制更直接;时域用 `fade_speed` 指数平滑替代 8 帧投票纹理行。**取舍**:不做 panini 投影修正、cloud/sun occlusion 纹理输入;灯型衰减 V1 只做方向光/点光两档。

**`LensFlareDataSRP.cs`(`LensFlareDataElementSRP`)**:元素字段全集 = `position`/`positionOffset`/`angularOffset`/`translationScale`/`lensFlareTexture`/`uniformScale`/`sizeXY`/`rotation`/`autoRotate`/`tint`/`modulateByLightColor`/`count` + `lengthSpread` + `positionCurve`/`scaleCurve`(多元素散布)/`seed` 与变体(`intensityVariation`、`positionVariation`)/procedural SDF 形状(`sideCount`、`sdfRoundness`、`inverseSDF`)/noise(`noiseAmplitude` 等)。**Zircon 对应**:`LensFlareElement` 取其纹理元素核心子集(本章字段表);**取舍**:不做 procedural SDF 形状、noise、distortion 曲线与 per-element count 散布 —— 多元素重复由资产里显式多 element 表达,曲线类字段后续按需追加。

**`NiagaraRendererSprites.cpp`(UE Niagara)**:模拟与渲染的契约 = `FNiagaraDynamicDataSprites`(`FNiagaraDynamicDataBase` 派生)持 `GetParticleDataToRender()` 返回的 `FNiagaraDataBuffer`;`PrepareParticleSpriteRenderData(...)` 只读该 buffer + `SourceMode`(`ENiagaraRendererSourceDataMode::Particles`)决定属性取自粒子流还是参数库(`ENiagaraSpriteVFLayout::Alignment` / `Facing` 的 `GetGPUOffset()`)。GPU 档关键:渲染器从不回读计数 —— `GPUCountBufferOffset != INDEX_NONE` 时经 `FNiagaraGPUInstanceCountManager::AddDrawIndirect(...)` 拿 `FIndirectArgSlot`,把 `IndirectDraw.SRV`/`Offset` 喂给 vertex factory loose params,instance 数始终留在 GPU。排序决策 `bNeedsSort = SortMode != ENiagaraSortMode::None && (translucent || !bSortOnlyWhenTranslucent)`,大计数时移交 GPU sort。**Zircon 对应**:`ParticleSimOutput` ≈ `FNiagaraDynamicDataBase` 契约,`IndirectArgsSlot` ≈ `FIndirectArgSlot`(由计划 03 batcher 充当 `FNiagaraGPUInstanceCountManager`);`ParticleSimSource::Gpu` 的三 buffer + 槽即该形态的 wgpu 翻译。**取舍**:不做 per-view GPU 粒子排序与 custom sort 变量,V1 透明排序停在 emitter 粒度(计划 09 sort_key);不做 `SourceMode::Emitter`(单粒子取参数库)档。

**`NiagaraRendererRibbons.cpp`**:CPU 路径 `GenerateVertexBufferCPU` → `GenerateVertexBufferForMultiRibbon` → `GenerateVertexBufferForRibbonPart`(产出 `SegmentData`、`TangentAndDistances`,UV 由 `UV0Settings` 决定拉伸/平铺);buffer 落地经 `FNiagaraRibbonVertexBuffers::InitializeOrUpdateBuffers`;索引按视角逐 view 生成(`GenerateIndexBufferForView`),并有 `TessellationConfig` 曲率细分。**Zircon 对应**:`RibbonBuilder` = `GenerateVertexBufferForRibbonPart` 的单 ribbon 简化版,`TangentAndDistances` 对应我们的切线 + 累计距离(TilePerMeter 用);**取舍**:V1 不做 multi-ribbon(`RibbonLinkOrder` 分组)、不做 tessellation、不做 per-view 索引重建 —— 朝向相机的侧偏移放进顶点着色器(读 view),索引 buffer 视角无关可复用,这是 wgpu 下更省的等价形。

## 风险与回退

- trail 顶点 CPU 展开在大量拖尾时成本高:每 trail 顶点数封顶 + 距离稀疏采样;GPU 展开留远期。
- lens flare 遮挡测试在 HZB 不可用(低端档)时:回落中心单点深度采样,接受精度损失。
- vfx_graph 真实编译不在本计划:粒子契约保证 vfx_graph 后续以同一 `ParticleSimOutput` 接入。
