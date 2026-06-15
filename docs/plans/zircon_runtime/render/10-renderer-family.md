---
related_code:
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/core/framework/render/mesh/descriptor.rs
  - zircon_runtime/src/core/framework/render/sprite/sprite.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/create_mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/sprite_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/mesh.rs
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/PrimitiveSceneProxy.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/Passes/DrawObjectsPass.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/Passes/RenderObjectsPass.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/ScriptableRendererFeature.cs
  - dev/bevy/crates/bevy_sprite/src/sprite.rs
  - dev/bevy/crates/bevy_render/src/batching/mod.rs
  - dev/bevy/crates/bevy_render/src/batching/gpu_preprocessing.rs
  - dev/bevy/crates/bevy_camera/src/visibility/range.rs
  - dev/bevy/crates/bevy_render/src/view/visibility/range.rs
  - dev/bevy/crates/bevy_pbr/src/render/mesh.rs
  - dev/bevy/crates/bevy_sprite_render/src/render/mod.rs
  - dev/Fyrox/fyrox-impl/src/scene/base.rs
  - dev/Fyrox/fyrox-impl/src/renderer/bundle.rs
plan_sources:
  - .codex/plans/ZirconEngine Bevy-Level Rendering Completion Plan.md
  - .codex/plans/ZirconEngine ECS 到渲染链路完善里程碑计划.md
---

# 计划 10:渲染器组件族(定制、裁剪、优化开关)

## 目标

把"渲染器"确立为面向用户的组件层(Unity Renderer 组件族语义),与底层 draw 管线(计划 02/03)解耦:

1. 组件族定稿:`MeshRenderer`、`SkinnedMeshRenderer`、`SpriteRenderer`、world-space `UiRenderer`,以及计划 12 的特效渲染器(billboard/trail 等)共用同一 renderer 基座契约。
2. 每渲染器的通用面板字段:cast/receive shadows、motion vector 开关、render layer、queue 覆写、per-renderer 材质覆写、static 标记(参与静态合批/烘焙)、LOD 归属。
3. 优化路径选择权威化:静态合批(static batching)、动态合批(dynamic batching)、GPU instancing 三种策略按渲染器标记 + 自动判定,统计可观测 —— 把现在 `prepared_queue.rs` 的"候选统计"兑现为真实策略。
4. LOD Group:多级 mesh + 屏占比阈值 + 过渡(dither cross-fade),骨骼/材质跨级共享。
5. 自定义渲染器扩展点:插件以 descriptor 注册新渲染器类型(extract 段 + pass 参与声明),对齐 URP RendererFeature 的可扩展语义。

## 现状与差距

- extract 中 mesh/sprite 实例字段齐,但无统一 renderer 基座:cast shadow 等开关散落,sprite 与 mesh 的公共语义(layer/排序/材质覆写)各写一份。
- LOD 仅"距离阈值选 mesh"的描述符,无屏占比、无过渡、无 LOD Group 资产语义。
- 合批:静态/动态/instancing 候选只有统计,无策略执行(计划 03 解决机制,本计划定策略与组件面)。
- 自定义渲染器无注册位:新增渲染器类型(如 trail)需要改内建枚举。

## 参考代码

| 文件 | 应重点阅读 |
|------|-----------|
| `dev/UnrealEngine/.../Engine/Private/PrimitiveSceneProxy.cpp` | proxy 基类承载的通用渲染开关(CastShadow/bRenderInMainPass/MotionVector 相关)与派生类定制点 —— renderer 基座契约的字段清单 |
| `dev/Graphics/.../Runtime/Passes/DrawObjectsPass.cs` | URP 按 queue 段 + layer mask 过滤绘制的 filtering/rendering settings 模型 |
| `dev/Graphics/.../Runtime/Passes/RenderObjectsPass.cs` + `ScriptableRendererFeature.cs` | 用户自定义绘制注入:override material、自定义过滤 —— 自定义渲染器扩展点样板 |
| `dev/bevy/crates/bevy_sprite/src/sprite.rs` | sprite 组件字段的 Rust 表达(flip/anchor/custom_size/atlas 引用) |

**Rust/wgpu 落地参照(防凭空实现)**:

| 文件 | 对应本计划机制 | 应重点阅读 |
|------|---------------|-----------|
| `dev/bevy/crates/bevy_render/src/batching/mod.rs` | 合批决策入口与组键 | `NoAutomaticBatching`(:25)显式退出位、`GetBatchData`/`GetFullBatchData`(:77/:119,batch 兼容键 + per-instance 数据切分)trait 切面 —— `MeshDrawBatchKey` 维度设计的对照 |
| `dev/bevy/crates/bevy_render/src/batching/gpu_preprocessing.rs` | GPU instancing 路径(衔接计划 03) | `BatchedInstanceBuffers`/`PhaseBatchedInstanceBuffers`(:167/:217)的实例 buffer 双层组织、`GpuPreprocessingSupport`(:104)能力降级 |
| `dev/bevy/crates/bevy_camera/src/visibility/range.rs` | LodGroup dither cross-fade 契约 | `VisibilityRange` 的 margin 重叠与 dithering crossfade 语义(:52/:76 注释)—— 与 `LodSelection.fade_from_level/fade_factor` 直接对应,重点 |
| `dev/bevy/crates/bevy_render/src/view/visibility/range.rs` | LOD 选择结果→GPU 下发 | `RenderVisibilityRanges`(:62)、`lod_index_for_entity`/`entity_has_crossfading_visibility_ranges`(:145/:152)与 dither 因子 buffer 写出 |
| `dev/bevy/crates/bevy_pbr/src/render/mesh.rs` | mesh 渲染器 extract→prepare→queue 全链 | batching/gpu_preprocessing 在 mesh 域的接入、`no_automatic_skin_batching`/`no_automatic_morph_batching`(:178)退出条件、`VisibilityRange` dither 消费 |
| `dev/bevy/crates/bevy_sprite_render/src/render/mod.rs` | sprite 渲染器的批组织 | `extract_sprites`(:345)/`queue_sprites`(:484)/`prepare_sprite_image_bind_groups`(:621)与 `SpriteBatch`(:474)按 image 切批的三段式 |
| `dev/Fyrox/fyrox-impl/src/scene/base.rs` | RendererCommon 字段母本(Rust 版) | `LodGroup`(:131)挂在 Base 节点的资产语义、`cast_shadows`(:411)等通用渲染开关的组件面表达 |
| `dev/Fyrox/fyrox-impl/src/renderer/bundle.rs` | 同 key 实例归并(instancing 策略) | `RenderDataBundle`(同 vertex/index + 共享材质,:176)与 `SurfaceInstanceData`(:188)的逐实例字段切分 |

静态合批(顶点预变换合并)与动态合批(CPU 顶点搬运)无 Rust 同类参照,实现时以 Unity 为唯一样板,按 index §8 第 8 条配对拍测试先行。

## 目标架构

归属:renderer 基座契约进 `core/framework/render/scene_extract.rs`(`RendererCommon` 结构);各渲染器 extract 段复合它;策略判定在 `scene_renderer/mesh/` prepare 阶段。

核心设计:

- `RendererCommon`:`layer_mask`、`render_queue_override`、`cast_shadows: Off|On|TwoSided|ShadowsOnly`、`receive_shadows`、`motion_vectors: Auto|Force|Off`、`material_overrides`、`is_static`、`lod_group: Option<LodGroupId>`、`enabled`。relevance(计划 04)从这里派生。
- 合批策略判定(prepare 期,每 draw 一次):
  - static 标记 + 不动 + 同材质 → 静态合批组(顶点预变换合并,资产期或加载期执行,运行时零成本);
  - 小顶点数 + 同材质 + 动态 → 动态合批(CPU 顶点搬运,阈值默认 ≤300 顶点,Unity 经验值);
  - 同 mesh + 同材质 → GPU instancing(计划 03 indirect batcher);
  - 互斥优先级:instancing > 静态合批 > 动态合批;判定结果与原因进 RenderStats(可解释性)。
- `LodGroup`:级列表(mesh + 屏占比阈值 + 可选材质)+ cross-fade 过渡(dither,变体 flag 走计划 08);skinned LOD 共享骨骼。
- 自定义渲染器注册:`RendererTypeDescriptor`(extract 段 schema + 参与 phase + 对应 pass processor 工厂),内建四渲染器与计划 12 特效渲染器同走此注册表,删除硬编码枚举分支。

## 里程碑

### RF-M1 RendererCommon 基座

实施切片:
1. `RendererCommon` 契约;mesh/sprite extract 复合并迁移散落字段;ECS 组件与编辑器面板对接。
2. cast_shadows/receive/motion_vectors 接通计划 04 relevance 与计划 05/06 消费端。

测试阶段:
- `cargo check -p zircon_runtime --lib --locked`;`cargo test -p zircon_runtime scene_extract --locked` + `render_product` 回归
- 验收证据:关 cast_shadows 后该对象不入 shadow pass(命令数断言);ShadowsOnly 模式主 pass 不可见。

### RF-M2 合批策略执行

实施切片:
1. 静态合批:加载期顶点预变换合并 + 合并 draw;动态合批:阈值判定 + CPU 顶点搬运路径。
2. 三策略互斥判定与 stats;`prepared_queue.rs` 候选统计改为策略结果。

测试阶段:
- `cargo test -p zircon_runtime mesh --locked`(同材质静态 100 物合为少量 draw;超阈值不动态合批)
- 验收证据:三策略场景 draw call 统计对比记入文档;产物对拍不变。

### RF-M3 LOD Group

实施切片:
1. `LodGroup` 资产/组件、屏占比计算(由计划 04 view 数据)、级选择。
2. dither cross-fade 变体与过渡窗口。

测试阶段:
- `cargo test -p zircon_runtime lod --locked`(屏占比阈值切级断言)
- 验收证据:拉远连续切级无 pop(抓帧序列);过渡期两级 dither 共存。

### RF-M4 自定义渲染器注册表

实施切片:
1. `RendererTypeDescriptor` 注册表;内建渲染器迁入;插件注册路径(计划 12 的 trail/billboard 首个消费)。

测试阶段:
- `cargo test -p zircon_runtime renderer --locked` 与插件 workspace 对应测试
- 验收证据:示例插件注册自定义渲染器并出现在指定 phase(集成测试)。

## 工程落地细化

本章是计划 10 的实施权威(见 index.md §8 第 7 条)。bind group 槽位、std430 布局、`zr_` include、`RenderQueueValue` 数值段、`sort_key` 位段权威等全局约定直接引用 index.md §8,本章不重定义。跨计划契约按既定名原样引用:计划 02 的 `MeshDrawCommand`/`MeshPassProcessor`/`CachedMeshDrawCommands`(静态命令缓存失效代际由本计划 `RendererCommon` 的 static 性变化驱动)、计划 03 的 `GpuScene`/`IndirectDrawBatcher`(per-draw first_instance ABI 已定,本计划只做策略与 stats)、计划 04 的 `ViewVisibilityContext`/`PrimitiveRelevance`(位表已定,含 `needs_velocity` 位)、计划 08 的 `ShaderVariantKey`(dither cross-fade 变体位)、计划 09 的 `RenderQueueValue`/`RenderLayer` 与 `sort_key` 位段。

### 模块与文件落点

**新增文件(7 处):**

| 路径 | 内容 | 层 |
|------|------|----|
| `zircon_runtime/src/core/framework/render/renderer_common.rs` | `RendererCommon`、`CastShadowsMode`、`MotionVectorMode`、`MaterialOverrideSet`、`LodGroupId` | framework 契约(无 wgpu) |
| `zircon_runtime/src/core/framework/render/lod_group.rs` | `LodGroup`、`LodLevel`、`LodSelection`、屏占比/滞回纯函数 | framework 契约(无 wgpu) |
| `zircon_runtime/src/core/framework/render/renderer_type.rs` | `RendererTypeId`、`RendererTypeDescriptor`(extract 段 schema + phase 参与声明) | framework 契约(无 wgpu) |
| `zircon_runtime/src/graphics/scene/scene_renderer/renderer_registry/mod.rs` + `registry.rs` + `builtin.rs` | `RendererTypeRegistry`、三阶段钩子 trait、内建渲染器注册 | graphics 实现 |
| `zircon_runtime/src/graphics/scene/scene_renderer/mesh/batching/mod.rs` + `decision.rs` + `stats.rs` | `BatchStrategyDecision`、`BatchRejectReason`、决策函数与统计 | graphics 实现 |
| `zircon_runtime/src/graphics/scene/scene_renderer/mesh/batching/static_batching.rs` + `dynamic_batching.rs` | 静态合并几何缓存、动态 CPU 顶点搬运路径 | graphics 实现 |
| `zircon_runtime/src/graphics/scene/scene_renderer/mesh/lod_select.rs` | 每帧 LOD 档位选择 + cross-fade 状态机(消费 `ViewVisibilityContext` 视图数据) | graphics 实现 |

**修改文件(9 处):**

| 路径 | 改动 |
|------|------|
| `zircon_runtime/src/core/framework/render/scene_extract.rs` | `RenderMeshSnapshot` 增加 `common: RendererCommon`;删除散落的 `render_layer_mask: u32` 字段(硬切换);`RenderMeshLodSelection` 由距离阈值语义改为 `LodGroup` 选择结果投影 |
| `zircon_runtime/src/core/framework/render/sprite/sprite.rs` | `RenderSpriteSnapshot` 增加 `common: RendererCommon`,删除 `render_layer_mask: u32`;`z_order` 保留(归 09 sort_key 消费) |
| `zircon_runtime/src/core/framework/render/frame_extract.rs` | `RenderFrameExtract` 增加 `lod_groups: Vec<LodGroup>` 字段(资产/组件快照,按 `LodGroupId` 索引) |
| `zircon_runtime/src/core/framework/render/mod.rs` | 三个新契约模块的 `mod` 声明与 facade 再导出(`zircon_runtime::core::framework::render` 固定 facade,只加薄声明) |
| `zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/queue_profile.rs` | `static_batch_eligible`/`dynamic_batch_eligible`/`gpu_instancing_eligible` 三个候选谓词改为输入 `RendererCommon` 派生数据;`MeshDrawBatchKey` 增加 `layer_mask`/`queue_value` 维度(不同 layer/queue 不合批) |
| `zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/create_mesh_draw.rs` | 形参 `cast_shadows: bool, receive_shadows: bool` 替换为 `common: &RendererCommon`(硬切换,调用方同变更迁移) |
| `zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue.rs` | `PreparedMeshQueueStats` 的 `*_candidate_*` 六个候选字段删除,替换为决策结果字段(见下节表);`prepare_mesh_queue` 接入 `batching::decide` |
| `zircon_runtime/src/graphics/scene/scene_renderer/sprite/sprite_renderer.rs` | sprite 渲染器经 `RendererTypeRegistry` 注册(RF-M4),删除调用侧硬编码分支 |
| `zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/mesh.rs`(及 `sprite.rs`/`ui.rs`) | feature descriptor 增加 renderer type 声明,phase 参与从注册表派生 |

ECS 组件与编辑器面板字段(RF-M1 切片 1 的"对接"部分)由 extract 生产方所在的 scene authority 模块承接,渲染侧只消费 `RenderFrameExtract`,不直接访问 ECS World(index.md §6 第 6 条)。

### 核心类型与接口

**`RendererCommon`(framework 契约,`renderer_common.rs`):**

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct RendererCommon {
    pub enabled: bool,
    pub layer_mask: RenderLayer,                      // 计划 09 类型,相机/灯光/volume 共用
    pub queue_override: Option<RenderQueueValue>,     // None = 取材质默认;覆写受 ±100 偏移约束(index.md §8 第 4 条)
    pub cast_shadows: CastShadowsMode,
    pub receive_shadows: bool,
    pub motion_vectors: MotionVectorMode,
    pub material_overrides: MaterialOverrideSet,      // per-slot 句柄覆写,不换 shader 模板
    pub is_static: bool,                              // 参与静态合批/静态命令缓存;变化即推进 CachedMeshDrawCommands 代际
    pub lod_group: Option<LodGroupId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CastShadowsMode { Off, On, TwoSided, ShadowsOnly }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MotionVectorMode { Auto, ForceOn, ForceOff }

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MaterialOverrideSet {
    pub slots: Vec<(u32 /* material slot index */, ResourceHandle<MaterialMarker>)>,
}
```

消费方约定(本计划只产字段,语义落点在各消费计划):

- 计划 04:`PrimitiveRelevance` 从 `RendererCommon` 派生 —— `cast_shadows != Off` → relevance 进 shadow 视图;`ShadowsOnly` → 主视图 relevance 清零但 shadow relevance 保留(对应 UE `bShadowIndirectOnly` + 主 pass 不可见的组合);`motion_vectors` 解析为 `needs_velocity` 位:`Auto` = `Mobility::Dynamic` 或本帧 transform 变化,`ForceOn` 恒置位,`ForceOff` 恒清零。
- 计划 05:`TwoSided` 映射 shadow pass processor 的 rasterizer cull 覆写;`receive_shadows` 进 model uniform flags(现 `create_mesh_draw.rs` 的 `model_uniform_from_draw_state` 已有该位,字段来源切换即可)。
- 计划 06:velocity pass 只取 `needs_velocity` 位,不重读 `RendererCommon`。

**`LodGroup`(framework 契约,`lod_group.rs`):**

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct LodGroup {
    pub id: LodGroupId,
    pub levels: Vec<LodLevel>,            // 按 coverage_threshold 降序;levels[0] 为最精
    pub cross_fade: bool,                 // dither 过渡开关
    pub hysteresis: Real,                 // 默认 0.1(阈值回差比例)
}

#[derive(Clone, Debug, PartialEq)]
pub struct LodLevel {
    pub mesh: ResourceHandle<MeshMarker>,
    pub coverage_threshold: Real,         // 屏占比下限,低于则切到下一级
    pub material: Option<ResourceHandle<MaterialMarker>>, // None = 跨级共享材质
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LodSelection {
    pub level_index: u32,
    pub fade_from_level: Option<u32>,     // Some = cross-fade 过渡中,两级共存
    pub fade_factor: Real,                // 0..1,写入 dither 变体的 per-draw 参数
}
```

屏占比公式(纯函数,放 `lod_group.rs` 便于单测):`coverage = bounds_radius / (distance * tan(fov_y / 2))`。档位选择:取第一个 `coverage >= coverage_threshold` 的级;滞回:已在 `i` 级时,升级(更精)需 `coverage >= levels[i-1].threshold`,降级需 `coverage < levels[i].threshold * (1.0 - hysteresis)`,消除阈值附近抖动。cross-fade:切级时开 `fade_from_level`,`fade_factor` 按固定帧窗(默认 8 帧)推进,两级 draw 同帧提交且 `ShaderVariantKey` 置 dither 变体位(计划 08),`fade_factor` 与互补值分别下发;skinned LOD 各级共享同一 joint palette(palette 句柄在 LodGroup 级而非 LodLevel 级解析)。与计划 04 的集成点:LOD 选择在 visibility 之后、prepare 之前执行,输入 `ViewVisibilityContext` 的相机位置与 `fov_y`,只对 relevance 可见集计算;过渡期两级均参与 relevance(同 bounds)。

**`RendererTypeDescriptor` 与注册表(契约 + graphics 双层):**

```rust
// framework 契约层(renderer_type.rs):纯声明,无 wgpu
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RendererTypeId(pub &'static str);   // "mesh"/"skinned_mesh"/"sprite"/"text"/"tilemap"/"billboard"/"trail"/"terrain"

#[derive(Clone, Debug)]
pub struct RendererTypeDescriptor {
    pub id: RendererTypeId,
    pub extract_kind: RendererExtractKind,     // 指明消费 RenderFrameExtract 的哪个段(meshes/sprites/particles/插件 sideband)
    pub participates: Vec<RenderPassStage>,    // phase 参与声明,feature graph 编排消费
    pub supports_batching: bool,               // false 则 prepare 跳过合批决策(如 trail)
}

// graphics 实现层(renderer_registry/registry.rs):三阶段钩子,可触 GPU 资源
pub(crate) trait RendererTypeRuntime: Send + Sync {
    fn descriptor(&self) -> &RendererTypeDescriptor;
    /// extract 段解析:从 RenderFrameExtract 切出本类型实例并归一化为含 RendererCommon 的中间表
    fn extract(&self, frame: &RenderFrameExtract, out: &mut RendererExtractBuffer);
    /// prepare:GPU 资源 ensure、合批决策、LOD 投影;产出批次(计划 02 术语 MeshBatch)
    fn prepare(&self, ctx: &mut RendererPrepareContext<'_>) -> RendererPrepareOutput;
    /// queue:把批次交给各 phase 的 MeshPassProcessor 转为 MeshDrawCommand(计划 02)
    fn queue(&self, ctx: &RendererQueueContext<'_>, out: &mut MeshDrawCommandList);
}

pub(crate) struct RendererTypeRegistry { /* Vec<Box<dyn RendererTypeRuntime>>,按 id 去重 */ }
```

内建八类(mesh/skinned/sprite/text/tilemap/billboard/trail/terrain;text/tilemap 归计划 14、billboard/trail 归计划 12、terrain 归计划 15,但全部经本注册表注册,RF-M4 先迁 mesh/skinned/sprite/ui)。插件注册入口:插件的 `RenderFeatureDescriptor` 增加 `renderer_types()` 返回 `Vec<Box<dyn RendererTypeRuntime>>`,feature 启用时注入注册表、关闭时不注册(index.md §6 第 4 条);不提供注册表以外的第二入口。注册表建立后删除 `scene_renderer` 中按渲染器类型的硬编码 match 分支(硬切换)。

### 合批决策与统计

决策在 prepare 期每 draw 执行一次,输入为 `MeshDrawBatchKey`(现 `queue_profile.rs` 已有,补 layer/queue 维度)分组后的组视图。互斥优先级固定:**GPU instancing > static batching > dynamic batching**,不做运行时自动切换抖动。

决策树(每组每 draw):

```text
enabled == false ────────────────────────────→ 不进任何队列(extract 期已剔)
phase == Transparent ────────────────────────→ Unbatched(TransparentPhase)
uses_skinned_gpu_skinning / indirect 已占用 ──→ Unbatched(SkinnedGpuPath / IndirectAlreadyBatched)
material_overrides 非空且组内不一致 ──────────→ Unbatched(MaterialOverrideMismatch)
LOD cross-fade 进行中(fade_from_level=Some) ─→ Unbatched(LodCrossFadeActive)   // fade_factor 为 per-draw 数据
组内同 (mesh, material set, pipeline) 且组员数 ≥ 2
  ├─ 是 → GpuInstanced(交计划 03 IndirectDrawBatcher,first_instance ABI)
  └─ 否(同材质不同 mesh)
       ├─ is_static 且 Mobility::Static ────→ StaticBatched(进/查静态合并几何缓存)
       ├─ 非 static 且顶点数 ≤ 300 ─────────→ DynamicBatched
       │     但 motion_vectors 解析为 needs_velocity ─→ Unbatched(MotionVectorForced)
       │     // 合并顶点后 per-object prev transform 不可表达
       └─ 其余 ────────────────────────────→ Unbatched(UniqueBatchKey / VertexBudgetExceeded)
```

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BatchStrategyDecision {
    GpuInstanced { group: u32 },
    StaticBatched { group: u32 },
    DynamicBatched { group: u32 },
    Unbatched { reason: BatchRejectReason },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum BatchRejectReason {
    TransparentPhase,
    SkinnedGpuPath,
    IndirectAlreadyBatched,
    MaterialOverrideMismatch,
    LodCrossFadeActive,
    MotionVectorForced,
    VertexBudgetExceeded,   // 动态合批 > 300 顶点(Unity 经验阈值,常量 DYNAMIC_BATCH_VERTEX_BUDGET)
    UniqueBatchKey,         // 无同 key 同伴
    StrategyDisabled,       // quality profile 关闭了该策略
}
```

**static batching**:对 `StaticBatched` 组,在加载期(资产可用即触发,非每帧)把组员顶点经各自 `model_matrix` 预变换后拷入合并 vertex/index buffer,记录每员 `first_index/index_count` 区间;运行时该组提交为同 pipeline 下的少量 draw(连续区间可并段),model uniform 置单位矩阵。失效条件即 `is_static` 契约失效:组员 `common.is_static` 翻假、transform 实际变化(extract 对静态对象做防御性校验,变化即降级并计 stats)、material/mesh 句柄变化 —— 任一发生则整组代际 +1、合并缓冲重建,同时推进计划 02 `CachedMeshDrawCommands` 同源代际。
**dynamic batching**:对 `DynamicBatched` 组,prepare 期 CPU 把 ≤300 顶点的小 mesh 逐帧变换到世界空间写入帧环形 vertex buffer,合并为单 draw。不划算即退出的条件(决策前置判定,记 `StrategyDisabled` 或对应 reason):组员数 < 2、单帧搬运顶点总量超预算(默认 32k,quality profile 可调)、目标平台 CPU 受限 profile 直接关闭。
**stats 字段**(替换 `PreparedMeshQueueStats` 中 `static_batch_candidate_group_count` 等六个候选字段,其余字段保留):

| 新字段 | 含义 |
|--------|------|
| `gpu_instanced_group_count` / `gpu_instanced_draw_count` | 实际进 `IndirectDrawBatcher` 的组/draw 数 |
| `static_batched_group_count` / `static_batched_draw_count` | 静态合并组数与吸收的原始 draw 数 |
| `dynamic_batched_group_count` / `dynamic_batched_draw_count` / `dynamic_batched_vertex_count` | 动态合批组/draw/搬运顶点数 |
| `unbatched_draw_count` | 未合批 draw 总数 |
| `unbatched_reasons: [usize; BatchRejectReason::COUNT]` | 按 reason 枚举序的解释性计数(可观测性核心) |
| `static_batch_invalidation_count` | 本帧静态组失效重建次数(稳态应为 0) |

### 帧时序与集成点

入口仍为 `WgpuRenderFramework::submit_frame_extract`(index.md §1),本计划不改 graph 编排,只在既有 Extract → Prepare → Queue 内插钩子:

1. **extract 后**:`RendererTypeRegistry` 遍历注册项调 `extract()`,各类型从 `RenderFrameExtract` 切段(meshes/sprites/…)归一化出含 `RendererCommon` 的实例表;`enabled=false` 在此剔除。
2. **visibility(计划 04)**:`ViewVisibilityContext` 消费 `RendererCommon` 派生 `PrimitiveRelevance`(layer_mask 过滤、shadow/main pass 位、needs_velocity 位)。
3. **LOD 选择**:`lod_select.rs` 对可见集计算 `LodSelection`,把 `RenderMeshSnapshot.mesh` 解析为选中级 mesh 句柄;cross-fade 状态机在此推进。
4. **prepare 钩子**:各类型 `prepare()` 内执行 `batching::decide` → 静态合并缓存查询/动态搬运 → 产出批次;stats 写 `PreparedMeshQueueStats`。
5. **queue 钩子**:`queue()` 把批次交计划 02 各 `MeshPassProcessor` 生成 `MeshDrawCommand`;`queue_override` 在此改写 phase 归属与 `sort_key` 的 queue 位段(位段布局归 09,本计划只填值);early-z:depth prepass 收 Opaque(2000 段)与 AlphaTest(2450 段,prepass 用 alpha-discard 变体),`RenderQueueValue >= 2500` 不进 prepass —— 对应现 `early_z_eligible()` 谓词从 phase 派生改为 queue 值派生,主 pass 对已 prepass 对象用深度等值测试(URP depth priming 同型)。
6. **execute**:不新增 pass;所有提交仍经既有 graph 节点(index.md §6 第 3 条)。

硬切换删除项(与各切片同变更执行):`RenderMeshSnapshot.render_layer_mask`/`RenderSpriteSnapshot.render_layer_mask` 散字段;`create_mesh_draw` 的 `cast_shadows`/`receive_shadows` bool 形参;`PreparedMeshQueueStats` 六个 `*_candidate_*` 字段及其断言测试;`queue_profile.rs` 中基于 `Mobility` 直接判合批资格的旧谓词体;RF-M4 后 `scene_renderer` 内按渲染器类型的硬编码 match 分支与内建枚举。

### 实施切片细化

**RF-M1 RendererCommon 基座**

| 切片 | 触碰文件 | 改动要点 | 完成判据 |
|------|---------|---------|---------|
| 1a | `renderer_common.rs`(新)、`render/mod.rs` | 契约类型 + facade 导出;单测随文件 | `cargo check -p zircon_runtime --lib --locked` 过;类型可从 `core::framework::render` 引用 |
| 1b | `scene_extract.rs`、`sprite/sprite.rs`、`frame_extract.rs` | snapshot 复合 `common`,删散字段;extract 生产方同变更迁移 | 全仓无 `render_layer_mask` 散字段引用;`scene_extract` 测试过 |
| 1c | `create_mesh_draw.rs`、`build/`、`queue_profile.rs` | bool 形参 → `&RendererCommon`;`casts_shadow()` 改读 `CastShadowsMode` | `render_product` 回归不变 |
| 2 | `mesh_draw/queue_profile.rs`、计划 04/05/06 接缝文件 | relevance 派生表落地;`ShadowsOnly` 主 pass 剔除;`needs_velocity` 解析 | 关 cast_shadows 后 shadow 命令数为 0 的断言;ShadowsOnly 主 pass 命令数为 0 |

**RF-M2 合批策略执行**

| 切片 | 触碰文件 | 改动要点 | 完成判据 |
|------|---------|---------|---------|
| 1a | `batching/static_batching.rs`(新) | 加载期顶点预变换合并缓存、代际失效 | 同材质静态 100 物合并后 draw 数 ≤ 组数断言 |
| 1b | `batching/dynamic_batching.rs`(新) | ≤300 顶点判定、帧环形 buffer 搬运、预算退出 | 超阈值 mesh 不合批且 reason 为 `VertexBudgetExceeded` |
| 2 | `batching/decision.rs`、`stats.rs`(新)、`prepared_queue.rs` | 决策树落地;候选字段删除换决策字段;instancing 组交 `IndirectDrawBatcher` | stats 候选字段编译期消失;三策略互斥断言;`render_product` 不变 |

**RF-M3 LOD Group**

| 切片 | 触碰文件 | 改动要点 | 完成判据 |
|------|---------|---------|---------|
| 1 | `lod_group.rs`(新)、`frame_extract.rs`、`lod_select.rs`(新) | 资产/组件契约、coverage 公式、滞回选级 | 公式与滞回纯函数单测过;阈值切级断言 |
| 2 | `lod_select.rs`、计划 08 变体接缝、`create_mesh_draw.rs` | cross-fade 状态机、dither 变体位、`fade_factor` 下发 | 过渡期同帧两级命令共存且带 dither 变体断言 |

**RF-M4 自定义渲染器注册表**

| 切片 | 触碰文件 | 改动要点 | 完成判据 |
|------|---------|---------|---------|
| 1 | `renderer_type.rs`(新)、`renderer_registry/`(新)、`feature_descriptors/{mesh,sprite,ui}.rs`、`sprite_renderer.rs` | 注册表 + 三钩子 trait;内建 mesh/skinned/sprite/ui 迁入;插件 `renderer_types()` 入口;删硬编码分支 | 注册表去重断言;示例插件渲染器出现在声明 phase 的集成测试;全仓无被删分支引用 |

### 测试与验收清单

单测(命名遵循 index.md §8 第 6 条 `render_<topic>_*`;位置 = 类型所在文件的 `#[cfg(test)]` 块,沿用 `prepared_queue.rs` 现行风格):

| 测试函数 | 断言 | 位置 |
|---------|------|------|
| `render_renderer_family_cast_shadows_off_removes_shadow_commands` | `CastShadowsMode::Off` 时 shadow phase 命令数为 0,主 pass 不变 | `mesh/prepared_queue.rs` |
| `render_renderer_family_shadows_only_hidden_in_main_pass` | `ShadowsOnly` 主 pass 命令数 0、shadow 命令数 > 0 | `mesh/prepared_queue.rs` |
| `render_renderer_family_two_sided_shadow_overrides_cull_mode` | `TwoSided` 时 shadow processor 输出 cull=None | 计划 05 shadow processor 测试文件 |
| `render_renderer_family_motion_vector_mode_resolves_needs_velocity` | Auto/ForceOn/ForceOff × 静/动 mobility 六组合的 `needs_velocity` 位 | `renderer_common.rs` |
| `render_renderer_family_queue_override_moves_draw_between_phases` | `queue_override=Some(3000)` 的 opaque 材质进 Transparent phase | `mesh/prepared_queue.rs` |
| `render_renderer_family_registry_rejects_duplicate_type_id` | 重复 `RendererTypeId` 注册返回错误 | `renderer_registry/registry.rs` |
| `render_renderer_family_plugin_renderer_participates_declared_phase` | 测试插件注册类型出现在其 `participates` phase 命令流 | `renderer_registry/` 集成测试 |
| `render_batching_priority_prefers_instancing_over_static` | 同 mesh+材质+static 组判为 `GpuInstanced` 非 `StaticBatched` | `batching/decision.rs` |
| `render_batching_static_group_absorbs_same_material_static_draws` | 100 静态同材质异 mesh → `static_batched_draw_count==100` 且组数小 | `batching/static_batching.rs` |
| `render_batching_static_invalidation_bumps_generation_once` | 改一员材质 → 仅该组代际 +1、`static_batch_invalidation_count==1` | `batching/static_batching.rs` |
| `render_batching_dynamic_rejects_over_vertex_budget` | 301 顶点 mesh reason==`VertexBudgetExceeded` | `batching/dynamic_batching.rs` |
| `render_batching_dynamic_skips_forced_motion_vector_draws` | `ForceOn` 动态小 mesh reason==`MotionVectorForced` | `batching/decision.rs` |
| `render_batching_unbatched_reasons_sum_to_unbatched_count` | reason 计数和 == `unbatched_draw_count`(决策全覆盖) | `batching/stats.rs` |
| `render_lod_coverage_matches_reference_formula` | `coverage(r=1, d=10, fov=90°) == 0.1` 等参考点 | `lod_group.rs` |
| `render_lod_hysteresis_holds_level_inside_deadband` | 阈值 ±hysteresis 内往返不切级 | `lod_group.rs` |
| `render_lod_cross_fade_emits_both_levels_with_dither_variant` | 过渡帧两级命令共存、`ShaderVariantKey` dither 位置位、fade 因子互补 | `lod_select.rs` |
| `render_lod_skinned_levels_share_joint_palette` | 各级 palette 句柄相同 | `lod_select.rs` |

产物对拍(`render_product_*`,沿用既有 harness):`render_product_renderer_family_shadow_toggle_scene`(开关 cast_shadows 两帧对拍)、`render_product_batching_static_scene_pixel_identical`(合批前后逐像素一致,draw 数下降)、`render_product_lod_transition_sequence`(拉远序列无 pop)。里程碑测试命令沿用正文(`cargo test -p zircon_runtime mesh|lod|renderer --locked`);切片期只 `cargo check -p zircon_runtime --lib --locked`(milestone-first)。

## 状态与产出记录

| 日期 | 里程碑/切片 | 状态 | 产出 | 验证与证据 | 后续 |
|------|-------------|------|------|------------|------|
| 2026-06-15 | RF-M1 RendererCommon baseline | 部分完成: extract 字段和若干公共语义存在,统一 renderer 基座未落地 | Mesh/sprite/particle extract 已能提供 layer、source entity、material、shadow/velocity 相关字段;计划 03/04/05/06 已在 GPUScene、visibility、shadow 和 velocity 中消费这些字段。 | 计划 03/04/05/06 状态表分别记录 source entity、GPUScene instance source、shadow caster filtering、particle/mesh velocity 相关证据。 | 建立 `RendererCommon` 组件/DTO,把 layer、sorting、material override、shadow、motion flags 收束到单一基座。 |
| 2026-06-15 | RF-M2 batching policy execution | 部分完成: GPUScene/indirect 机制存在,renderer 级策略未定稿 | 计划 03 GS-M4 已有 indirect batcher、multi-draw replay 与 diagnostics;但本计划定义的 static/dynamic/instancing 策略组件面仍未落地。 | 计划 03 GS-M4 状态表记录 indirect stats、WGPU replay 与 `cargo check` 证据。 | 补 renderer batching policy、batching reason diagnostics 和 material/renderer override。 |
| 2026-06-15 | RF-M3 LOD Group | 未启动: 仍停留在计划描述 | 当前只具备距离阈值式 LOD 描述,无屏占比、过渡或 LOD Group 资产。 | 本文件 `现状与差距` 明确 LOD 仅为距离阈值选 mesh。 | 实施 LODGroup asset、screen-size evaluation、cross-fade 与 renderer stats。 |
| 2026-06-15 | RF-M4 custom renderer registry | 未启动: 仍缺注册入口 | 新增渲染器类型仍需改内建枚举,无插件/运行时 renderer 注册位。 | 本文件 `现状与差距` 明确 custom renderer 无注册位。 | 与计划 12 trail/projector/terrain 等消费者联动建立 renderer family registry。 |

### 参考实现精读笔记

**`dev/UnrealEngine/.../Engine/Private/PrimitiveSceneProxy.cpp`**

- `FPrimitiveSceneProxyDesc::InitializeFromPrimitiveComponent`(约 :285 起)一次性拷贝组件开关全集:`CastShadow`、`bCastDynamicShadow`、`bCastHiddenShadow`、`bCastShadowAsTwoSided`、`bRenderInMainPass`、`bRenderInDepthPass`、`Mobility`、`TranslucencySortPriority`、`MinDrawDistance`/`CachedMaxDrawDistance`、`ShadowCacheInvalidationBehavior` —— "组件字段 → 渲染快照"单向拷贝即 Zircon 的 extract 复合 `RendererCommon`,字段清单是 `RendererCommon` 的母本(`bCastShadowAsTwoSided` → `TwoSided`;`bShadowIndirectOnly` + `bRenderInMainPass=false` 的组合 → `ShadowsOnly`)。取舍:UE 拷 50+ 个 bool,Zircon 只收敛 9 字段,editor-only/raytracing/foliage 类不进契约。
- proxy 构造列表里 `bCastDynamicShadow(InProxyDesc.bCastDynamicShadow && InProxyDesc.CastShadow && !InProxyDesc.GetShadowIndirectOnly())`(:474)—— 派生开关在构造期一次解析、运行期只读;Zircon 对应:`needs_velocity`/shadow relevance 在 extract→relevance 派生一次,prepare/queue 不再看原始枚举。
- `FPrimitiveSceneProxy::GetViewRelevance(const FSceneView*)`(:816)默认返回空 `FPrimitiveViewRelevance`,派生类按视图填位 —— 印证 relevance 是"每视图位表"而非全局 bool,Zircon 由计划 04 `PrimitiveRelevance` 承接,本计划只供字段。
- 双路径:`DrawStaticElements(FStaticPrimitiveDrawInterface* PDI)`(头文件 :391)静态路径进缓存命令,`GetDynamicMeshElements(..., FMeshElementCollector&)`(:448)每帧动态收集 —— Zircon 对应 `is_static` → 计划 02 `CachedMeshDrawCommands` 与每帧 dynamic 列表的分流;本计划的 `is_static` 失效即 UE `ShadowCacheInvalidationBehavior` 同型的代际驱动。取舍:Zircon 不做虚函数双路径,extract 数据驱动单路径 + 缓存命中。

**`dev/Graphics/.../Runtime/Passes/DrawObjectsPass.cs`**

- `m_FilteringSettings = new FilteringSettings(renderQueueRange, layerMask)`(:92)—— pass 级过滤只有 queue 段 + layer mask 两维,Zircon 等价物即 `RenderQueueValue` 段 + `RenderLayer`(归 09),本计划保证两字段在 `RendererCommon`/`MeshDrawBatchKey` 中可用。
- sort 选择(:204–206):opaque 用 `cameraData.defaultOpaqueSortFlags`、透明用 `SortingCriteria.CommonTransparent`、2D 用 `SortingLayer | RenderQueue | OptimizeStateChanges | CanvasOrder` 组合 —— 排序准则按 phase 配置而非全局唯一;Zircon 由 09 的 `sort_key` 位段一次表达,本计划不造第二套。
- depth priming 时 `m_RenderStateBlock.depthState = new DepthState(false, CompareFunction.Equal)`(:221)—— early-z 后主 pass 关深度写、用等值测试;本计划"帧时序与集成点"第 5 条直接采用该型。

**`dev/Graphics/.../Runtime/Passes/RenderObjectsPass.cs` + `ScriptableRendererFeature.cs`**

- `overrideMaterial`/`overrideMaterialPassIndex`/`overrideShader`(:21–31)与 `SetDepthState(bool, CompareFunction)`(:57)—— URP 的覆写是整材质/整 shader 替换;Zircon 取舍:`MaterialOverrideSet` 只做 per-slot 句柄覆写、不换 shader 模板(正文风险节既定),避免变体爆炸。
- `ScriptableRendererFeature` 的 `public abstract void Create()`(:50)+ `AddRenderPasses(ScriptableRenderer, ref RenderingData)`(:64)双钩子 —— "一次构造 + 每帧注入"的扩展模型;Zircon 映射为 `RendererTypeRuntime` 注册(一次)+ `prepare/queue` 钩子(每帧),且注入只经 `RenderFeatureDescriptor`,无运行时动态加 pass 的旁路。

**`dev/bevy/crates/bevy_sprite/src/sprite.rs`**

- `struct Sprite`(:19)字段 `image: Handle<Image>`、`texture_atlas: Option<TextureAtlas>`、`color`、`flip_x/flip_y`、`custom_size: Option<Vec2>`、`rect: Option<Rect>`、`image_mode: SpriteImageMode` —— 与现 `RenderSpriteSnapshot`(`core/framework/render/sprite/sprite.rs`)逐字段同构,确认 sprite 契约无需重设计,RF-M1 仅追加 `common: RendererCommon` 并删 `render_layer_mask` 散字段。

## 风险与回退

- 动态合批的 CPU 搬运可能负优化:严格阈值 + stats 暴露收益,默认保守,可被 quality profile 关闭。
- 静态合批与 GPU instancing 在同组对象上竞争:互斥优先级固定且可解释,不做运行时自动切换抖动。
- 材质覆写引爆变体数:覆写只换绑定不换 shader 模板,超出参数集的覆写在导入期诊断。
