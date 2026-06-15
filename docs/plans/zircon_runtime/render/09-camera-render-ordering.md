---
related_code:
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_queue.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_sort.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/pipeline_kind.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
  - zircon_runtime/src/graphics/types/viewport_render_output_target.rs
  - zircon_runtime/src/graphics/runtime/render_framework/create_viewport/mod.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/UniversalRenderPipeline.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/UniversalAdditionalCameraData.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/ScriptableRenderer.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/UniversalRenderPipelineCore.cs
  - dev/bevy/crates/bevy_camera/src/camera.rs
  - dev/bevy/crates/bevy_camera/src/clear_color.rs
  - dev/bevy/crates/bevy_camera/src/visibility/render_layers.rs
  - dev/bevy/crates/bevy_render/src/camera.rs
  - dev/bevy/crates/bevy_render/src/render_phase/mod.rs
  - dev/bevy/crates/bevy_render/src/render_phase/rangefinder.rs
  - dev/bevy/crates/bevy_core_pipeline/src/core_3d/mod.rs
  - dev/bevy/crates/bevy_core_pipeline/src/core_2d/mod.rs
plan_sources:
  - .codex/plans/ZirconEngine Bevy-Level Rendering Completion Plan.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
---

# 计划 09:相机管理与渲染顺序体系(Unity 对齐)

## 目标

建立 Unity 语义的相机与排序体系,作为所有渲染器(3D/2D/UI/特效)共享的顺序权威:

1. 多相机:相机栈(Base + Overlay)、per-camera 渲染目标(屏幕 / RT)、viewport rect、depth(相机间先后)、clear 策略。
2. RenderLayer:相机 culling mask × 渲染器 layer 的可见性矩阵;灯光/阴影/volume 也按 layer 过滤(计划 05/07 消费)。
3. RenderQueue 数值体系(Unity 对齐):Background(1000) / Geometry(2000) / AlphaTest(2450) / Transparent(3000) / Overlay(4000);材质可覆写 queue 值;queue 段映射到 RenderPhase。
4. 2D sorting layer + order in layer、UI z-index、3D 深度排序统一收敛进计划 02 的 sort_key 打包规则,各渲染域共享一个排序定义而不是各自为政。

## 现状与差距

- `camera.rs` 有 `ViewportCameraSnapshot`/`RenderCameraTarget`/`RenderLayer` 基础,但无相机栈与相机间 depth 顺序概念;多相机即多 viewport,无 Base/Overlay 合成语义。
- `phase_queue.rs`/`phase_sort.rs` 的 phase 划分(Opaque3d/AlphaMask3d/Transparent3d/Sprite2d)是硬编码枚举,材质无 queue 数值可覆写;sprite 与 UI 的排序规则与 3D 不在同一定义里。
- UI z-index 在 UI 模块内部自洽(已闭环),但与场景内 3D/2D 内容混排(world-space UI、sprite 在 3D 中)无统一权威。

## 参考代码

| 文件 | 应重点阅读 |
|------|-----------|
| `dev/Graphics/.../Runtime/UniversalRenderPipeline.cs` | 相机栈排序与逐相机渲染循环(`RenderCameraStack`):Base/Overlay 的目标合成、相机 depth 排序 |
| `dev/Graphics/.../Runtime/UniversalAdditionalCameraData.cs` | 相机扩展数据面:render type、culling mask、volume layer mask、renderer 选择 —— 本计划相机契约字段的清单样板 |
| `dev/Graphics/.../Runtime/ScriptableRenderer.cs` | renderer 与相机解耦:相机选择 renderer 实例(对应本引擎 pipeline asset 选择) |
| `dev/Graphics/.../Runtime/UniversalRenderPipelineCore.cs` | `CameraData`/`SortingCriteria` 的组织:排序准则位组合(深度、queue、材质 id) |

次参考:`dev/bevy/crates/bevy_render/src/camera/`(`RenderTarget`/`Viewport`/`ClearColorConfig` 的 Rust 契约表达)。

**Rust/wgpu 落地参照(防凭空实现)**:

| 文件 | 对应本计划机制 | 应重点阅读 |
|------|---------------|-----------|
| `dev/bevy/crates/bevy_camera/src/camera.rs` | `CameraRenderDescriptor`(order/target/viewport/clear) | `Camera` 组件的 `order: isize`(:355)/`viewport: Option<Viewport>`/`clear_color` 字段与 `RenderTarget`(窗口 / Image / TextureView)枚举 —— 相机契约字段的 Rust 形态 |
| `dev/bevy/crates/bevy_camera/src/clear_color.rs` | `RenderCameraClear` | `ClearColorConfig`(Default/Custom/None)三态枚举的契约表达 |
| `dev/bevy/crates/bevy_camera/src/visibility/render_layers.rs` | `RenderLayerSet` 四方共用 mask | `RenderLayers` 以 SmallVec 位集支持 ≥32 layer、`intersects` 过滤 —— 与既有 `RenderLayerSet` 同构,定稿时对照其 API 面 |
| `dev/bevy/crates/bevy_render/src/camera.rs` | `resolve_camera_sequence` + camera_loop | `sort_cameras`(:674):按 order→target 排序、同 order 同 target 发 ambiguity 警告;`SortedCameras` 驱动逐相机执行的 Rust 同型 |
| `dev/bevy/crates/bevy_render/src/render_phase/mod.rs` | `phase_queue.rs`/`phase_sort.rs` 改造 | `PhaseItem::sort_key()`(:2091)+ `sort_phase_system`(:2173);Sorted/Binned 双形态 —— 域 A"聚簇优先"即 bin key 思路的单键化 |
| `dev/bevy/crates/bevy_render/src/render_phase/rangefinder.rs` | 域 A/B 的 effective_depth 来源 | `ViewRangefinder3d::distance`:view 矩阵 row2 点积取 view-space z 作距离键(深度量化的输入定义) |
| `dev/bevy/crates/bevy_core_pipeline/src/core_3d/mod.rs` | 域 A 不透明聚簇 / 域 B 透明深度 | `Opaque3dBatchSetKey`/`Opaque3dBinKey`(:186/:221,pipeline+material 聚簇键)与 `Transparent3d` 的 `FloatOrd` 距离键(:422)对照 |
| `dev/bevy/crates/bevy_core_pipeline/src/core_2d/mod.rs` | 域 C 2D 排序 | `Transparent2d.sort_key: FloatOrd`(:312)与 2D phase 划分 —— 单 f32 键与本计划三级字典序(sorting_layer/order/y)的差距即扩展点 |

`RenderQueueValue` 材质 queue 数值段/覆写与相机栈 Base/Overlay 显式契约(bevy 仅有 order + `ClearColorConfig::None` 的隐式叠加拼法,无栈结构)无 Rust 同类参照,实现时以 UE/URP 为唯一样板,按 index §8 第 8 条配对拍测试先行。

## 目标架构

归属:契约全部在 `core/framework/render/camera.rs` 与 `core_pipeline/`;执行在 render_framework 的 viewport/相机循环;不新增 crate。

核心设计:

- `CameraRenderDescriptor` 扩展:`render_order: i32`(相机间顺序,Unity camera depth)、`render_type: Base | Overlay`、`stack: Vec<CameraId>`(Base 持有 Overlay 列表)、`target: Screen | Texture(handle)`、`viewport_rect`、`clear: Color | Depth | Skybox | None`、`culling_mask: RenderLayerMask`、`volume_mask`。
- 帧循环改造:按 render_order 排序的相机序列 → 每相机一次 graph 执行(共享 compiled graph 缓存,按相机配置键控);Overlay 相机复用 Base 的 color/depth attachment(load 而非 clear),计划 01 的资源声明天然表达。
- `RenderQueueValue(u16)`:材质资产新增字段,默认按材质域映射(opaque→2000、alpha-test→2450、transparent→3000);queue 段→phase 映射表固化在 `core_pipeline`;段内值参与 sort_key。
- 统一排序定义(写入计划 02 的 sort_key 打包):
  `[相机 render_order] > [queue 段/phase] > [2D: sorting_layer → order_in_layer → y/深度] / [3D opaque: pipeline 聚簇 → 深度前向] / [3D transparent: 深度后向 → queue 内序]`;
  world-space UI 作为 transparent queue 的普通成员参与 3D 排序,screen-space UI 维持既有闭环链路在 graph 末端。
- RenderLayer 定稿为 32 位 mask;相机/灯光/volume/渲染器四方共用同一类型(已有 `RenderLayer` 扩展,不新造)。

## 里程碑

### CO-M1 相机契约与多相机循环

实施切片:
1. 相机契约扩展(order/type/stack/target/viewport/clear/masks);extract 与编辑器相机面板字段对接。
2. 帧循环按 render_order 多相机执行;RT 相机(离屏)路径与既有 offline/viewport 渲染收敛。

测试阶段:
- `cargo check -p zircon_runtime --lib --locked`;`cargo test -p zircon_runtime camera --locked` + `render_product` 回归
- 验收证据:双相机(主 + RT)各自产物正确;render_order 交换后合成顺序改变(对拍断言)。

### CO-M2 相机栈与 Overlay 合成

实施切片:
1. Base/Overlay 栈:Overlay 不 clear color、深度策略可选;栈内顺序生效。
2. viewport rect 与多相机分屏。

测试阶段:
- `cargo test -p zircon_runtime camera --locked`(栈合成:overlay 内容叠于 base 之上的像素断言)
- 验收证据:武器手/HUD 相机叠加用例;分屏用例。

### CO-M3 RenderQueue 与统一排序键

实施切片:
1. 材质 queue 字段与段→phase 映射;计划 02 sort_key 打包按本计划定义重排。
2. sprite/world-space UI 并入统一排序;sorting layer/order in layer 契约进 `core_pipeline`(2D 细则由计划 14 消费)。

测试阶段:
- `cargo test -p zircon_runtime phase --locked`(queue 覆写改变绘制顺序的断言;透明排序正确性)
- 验收证据:自定义 queue=2900 的 opaque 材质绘于 transparent 之前最后;sprite 与透明 mesh 混排顺序正确。

### CO-M4 layer 过滤全线贯通

实施切片:
1. culling mask 进计划 04 可见性(per-view 过滤);灯光 layer(计划 05)、volume layer(计划 07)对接同一 mask 类型。

测试阶段:
- `cargo test -p zircon_runtime visibility --locked` 与 light/volume 范围回归
- 验收证据:同场景两相机不同 mask 看到不同对象集合;灯光 mask 只照亮指定 layer。

## 工程落地细化

本章是本计划的实施权威(index.md §8 第 7 条)。`RenderQueueValue` 数值段与 `sort_key: u64` 位段的全局权威即本章(index.md §8 第 4/5 条);计划 02(`MeshDrawCommand` 命令排序)、计划 10(合批切分)、计划 14(2D sorting layer/y-sort)只消费本章布局,不得另造位段。术语:**域(domain)** = sort_key 中段的按 phase 族多态解释;**相机序列(camera sequence)** = 帧内按 `render_order` 解析出的 Base 相机列表及其各自的 Overlay 子列表。

### 模块与文件落点

新增文件:

| 文件 | 内容 |
|---|---|
| `zircon_runtime/src/core/framework/render/core_pipeline/render_queue.rs` | `RenderQueueValue(u16)` 常量段、材质覆写 clamp(±100)、段→`RenderPhase` 映射,含 `#[cfg(test)] mod tests` |
| `zircon_runtime/src/core/framework/render/core_pipeline/packed_sort_key.rs` | 位段常量(shift/width)、`packed_sort_key_u64` 唯一打包入口、各域编码 helper、与 breakdown 共用的解码函数,含 `#[cfg(test)] mod tests` |
| `zircon_runtime/src/core/framework/render/camera_stack.rs` | `CameraRenderType` / `RenderCameraClear` / `CameraRenderDescriptor` / `resolve_camera_sequence`(栈校验与 Base/Overlay 合成规则),含 `#[cfg(test)] mod tests` |
| `zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/camera_loop.rs` | 帧内多相机循环:相机序列 → 逐 Base 相机 graph 执行 → 逐 Overlay 相机 graph 执行(复用 Base attachment) |

修改文件:

| 文件 | 改动 |
|---|---|
| `core/framework/render/camera.rs` | `ViewportCameraSnapshot` 收缩为投影/曝光/MSAA/动态分辨率载荷:`order`/`target`/`viewport`/`clear_color`/`render_layers` 五字段迁出至 `CameraRenderDescriptor`(硬切换,同变更迁移调用方);删除对应 serde default 函数 |
| `core/framework/render/camera_ordering.rs` | `RenderCameraOrderInput` 改持 `CameraRenderDescriptor`;`SortedRenderCamera` 增 `render_type`;Overlay 相机不再独立成序(只随 Base 出现) |
| `core/framework/render/core_pipeline/mod.rs` | 声明 `render_queue`/`packed_sort_key` 模块与 re-export(仅 wiring) |
| `core/framework/render/core_pipeline/phase_sort.rs` | `RenderPhaseSortKey` 改为 `u64` newtype 薄层(`raw() -> u64`);删除 i128 打包、`signed_15/23/35`、`render_queue_sort_key` 等全部 i128 helper;`RenderPhaseSortComponents` 字段按本章定稿(见下) |
| `core_pipeline/phase_sort_key_breakdown.rs`、`phase_sort_decision.rs`、`phase_sort_decision_field.rs` | breakdown/decision 字段按新位段(camera_order/queue/domain/tie)重写,解码函数从 `packed_sort_key.rs` 复用 |
| `core_pipeline/phase_queue.rs` | `MeshPhaseInput`/`SpritePhaseInput` 的 `render_queue: i32`+`material_queue: i32` 合并为 `queue: RenderQueueValue`;增 `camera_order`/`sorting_layer`/`y_sort` 输入;`into_phase_item` 改调 `packed_sort_key_u64` |
| `core_pipeline/phase_queue_ordering_key.rs` | `raw_sort_key()` 返回 `u64`;entity 全量比较保留为最终决胜 |
| `core_pipeline/render_phase.rs` | `RenderPhase::mesh_phase(pipeline, alpha_mask, transparent)` 布尔签名删除,改 `RenderQueueValue::phase(pipeline)` 驱动 |
| `core/framework/render/frame_extract.rs` | `RenderViewExtract` 增 `cameras: Vec<CameraRenderDescriptor>`;`from_camera` 内部构造单元素序列(editor viewport 单相机路径不变) |
| `core/framework/render/material/`(材质快照模块) | 材质快照增 `render_queue: Option<RenderQueueValue>` 覆写字段 + 导入期 queue×混合态矛盾校验(对应风险条目) |
| `core/framework/render/scene_extract.rs` | `RendererCommon.render_queue_override: Option<RenderQueueValue>`(计划 10 契约对齐本类型);灯光/volume/渲染器 mask 字段统一 `RenderLayerSet` |
| `graphics/types/viewport_render_frame.rs` | `camera()` 访问器改返回主相机 `CameraRenderDescriptor`;`previous_motion_vector_camera` 同步换型 |
| `graphics/runtime/render_framework/create_viewport/create.rs` | viewport 创建按 descriptor 的 `target`/`viewport_rect` |
| `graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs` | 单相机提交改为调 `camera_loop.rs` 序列循环 |
| `graphics/pipeline/compiled_graph_cache.rs`(计划 01 所有) | `CompiledGraphCacheKey` 增 camera target 成分(见"帧时序与集成点") |

### 核心类型与接口

契约归属:类型全部在 `core/framework/render`(camera.rs / camera_stack.rs / core_pipeline),无 `wgpu`;执行归属:render_framework 的 camera_loop 与 viewport 路径。

```rust
// core/framework/render/camera_stack.rs —— 相机契约(对齐 URP UniversalAdditionalCameraData)
pub enum CameraRenderType { Base, Overlay }

pub enum RenderCameraClear {
    Skybox,            // 清 depth,天空盒覆盖 color(MSAA>1 时额外清 color,见精读笔记)
    Color(Vec4),       // 清 color + depth
    DepthOnly,         // 仅清 depth
    None,              // 都不清(load 上一相机/上一帧内容)
}

pub struct CameraRenderDescriptor {
    pub entity: Option<EntityId>,           // 场景相机实体;editor 合成相机为 None
    pub render_order: i32,                  // 相机间先后(Unity camera depth 语义),升序执行
    pub render_type: CameraRenderType,
    pub stack: Vec<EntityId>,               // 仅 Base 有效;Overlay 必须为空(resolve 校验)
    pub target: RenderCameraTarget,         // 复用既有 PrimarySurface | Texture(handle) | Headless
    pub viewport_rect: Option<RenderViewportRect>, // 复用既有类型;None = 全目标
    pub clear: RenderCameraClear,           // Base 相机消费;Overlay 忽略 color 部分
    pub clear_depth: bool,                  // 仅 Overlay 消费(URP clearDepth 语义)
    pub culling_mask: RenderLayerSet,       // 计划 04 per-view 过滤消费
    pub volume_mask: RenderLayerSet,        // 计划 07 VolumeEvaluator 消费
    pub camera: ViewportCameraSnapshot,     // 投影/曝光/HDR/MSAA/动态分辨率载荷(收缩后)
}

/// 校验并合成相机序列:Base 按 render_order 升序;每个 Base 携带其 stack 解析出的
/// Overlay 列表(保持 stack 内顺序)。违规(Overlay 带 stack、stack 引用非 Overlay、
/// stack 成员 target 与 Base 不一致)记入 report 并剔除该成员,不 panic。
pub fn resolve_camera_sequence(
    cameras: &[CameraRenderDescriptor],
) -> CameraSequenceReport; // { sequence: Vec<CameraSequenceEntry>, violations: Vec<...> }

pub struct CameraSequenceEntry {
    pub base: CameraRenderDescriptor,
    pub overlays: Vec<CameraRenderDescriptor>,
}
```

```rust
// core/framework/render/core_pipeline/render_queue.rs —— queue 数值权威(index.md §8 第 4 条)
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RenderQueueValue(pub u16);

impl RenderQueueValue {
    pub const BACKGROUND: Self = Self(1000);
    pub const GEOMETRY: Self = Self(2000);
    pub const ALPHA_TEST: Self = Self(2450);
    pub const GEOMETRY_LAST: Self = Self(2500);   // 段边界:≤2500 走不透明族
    pub const TRANSPARENT: Self = Self(3000);
    pub const OVERLAY: Self = Self(4000);
    pub const MAX: Self = Self(5000);

    /// 材质默认映射:Opaque→GEOMETRY、Mask→ALPHA_TEST、Blend→TRANSPARENT。
    pub fn from_alpha_mode(mode: &RenderMaterialAlphaMode) -> Self;
    /// 材质覆写偏移,clamp 到 ±100(index.md §8 第 4 条)。
    pub fn with_material_offset(self, offset: i16) -> Self;
    /// 段→phase 映射(唯一权威):
    ///   0..=2449  → Opaque{2d,3d};2450..=2500 → AlphaMask{2d,3d};
    ///   2501..=3999 → Transparent{2d,3d};4000.. → Overlay。
    pub fn phase(self, pipeline: CorePipelineKind) -> RenderPhase;
}
```

`RenderLayer` 定稿:`RenderLayer = u32` 保持**layer 索引**语义(0 起);掩码类型统一为既有 `RenderLayerSet`(支持 ≥32 layer)。四处同一语义与 extract 传递:

| 消费方 | 字段 | 传递路径 |
|---|---|---|
| 相机 | `CameraRenderDescriptor.culling_mask` | extract view → 计划 04 per-view 可见性过滤 |
| 灯光 | 灯光 extract 快照 `layer_mask: RenderLayerSet` | `frame_extract.lighting` → 计划 05 light/shadow 过滤 |
| volume | volume extract `volume_mask: RenderLayerSet` × 相机 `volume_mask` 相交 | 计划 07 `VolumeEvaluator` |
| 渲染器 | `RendererCommon.layer_mask: RenderLayerSet` | 计划 10 extract 基座 → 可见性矩阵行 |

GPU 侧或 ABI 边界需要 u32 位掩码时,经既有 `RenderLayerSet::to_legacy_mask_lossy()` 降级;layer ≥32 仅参与 CPU 侧过滤。注:计划 07 细化章节的 `volume_mask`/`camera_volume_mask` 已统一为 `RenderLayerSet`,与本定稿一致。

### 排序键位段定稿(本计划核心交付)

`sort_key: u64` 全局位段权威表,从高到低:

| 位区间 | 宽度 | 字段 | 编码 |
|---|---|---|---|
| 63–56 | 8 | `camera_order` | `(render_order.clamp(-128, 127) + 128) as u64`。URP camera depth 实践范围 [-100,100];同一相机的命令列表内本段恒定(不影响相机内排序),保留它使跨相机合并诊断与全局重放有全序;同 order 歧义由 `camera_ordering.rs` ambiguity 报告兜底 |
| 55–43 | 13 | `queue` | `RenderQueueValue.0.min(8191) as u64`。覆盖 0..5000 实用域;queue 段同时决定 phase(`RenderQueueValue::phase`),段内数值直接参与排序 → "queue=2900 的材质绘于 transparent 段内最前"自然成立 |
| 42–10 | 33 | `domain`(按 phase 族多态) | 见下三张子表 |
| 9–0 | 10 | `tie_breaker` | `entity_tie_breaker & 0x3ff`。配合 Rust `sort_by_key` 稳定排序 + extract 顺序确定性;`RenderPhaseQueueOrderingKey` 保留 entity 全量比较作诊断路径最终决胜 |

**域 A:3D 不透明族**(`Opaque3d`/`AlphaMask3d`/`Prepass`/`Shadow`/`Deferred`)——pipeline/material 聚类优先,深度前到后:

| 位区间 | 宽度 | 字段 | 编码 |
|---|---|---|---|
| 42–33 | 10 | `pipeline_cluster` | `pipeline_variant & 0x3ff`(mesh_pipeline_cache 变体索引低 10 位) |
| 32–25 | 8 | `material_cluster` | `((material_discriminant >> 8) ^ material_discriminant) & 0xff`(u16 折叠 8 位) |
| 24–10 | 15 | `depth_f2b` | `clamp(round(effective_depth * 8.0), 0, 32767)`,1/8 m 步长、4096 m 量程,前到后升序 |

**域 B:3D 透明族**(`Transparent3d`,world-space UI 作为 transparent queue 普通成员走此域)——深度后到前主导:

| 位区间 | 宽度 | 字段 | 编码 |
|---|---|---|---|
| 42–20 | 23 | `depth_b2f` | `8_388_607 - clamp(round(effective_depth * 1000.0), 0, 8_388_607)`,mm 步长、8388 m 量程,后到前 |
| 19–10 | 10 | `pipeline_cluster` | 同域 A;仅在等深桶内聚簇,不破坏混合正确性 |

**域 C:2D 族**(`Opaque2d`/`AlphaMask2d`/`Transparent2d`)——sorting_layer → order_in_layer → y-sort(计划 14 消费):

| 位区间 | 宽度 | 字段 | 编码 |
|---|---|---|---|
| 42–35 | 8 | `sorting_layer` | `(sorting_layer.clamp(-128, 127) + 128) as u64` |
| 34–20 | 15 | `order_in_layer` | `(order_in_layer.clamp(-16_384, 16_383) + 16_384) as u64` |
| 19–10 | 10 | `y_sort` | 启用时 `(round(y * Y_SORT_UNITS).clamp(-512, 511) + 512) as u64`(y 大者键大→后绘,Godot 语义;`Y_SORT_UNITS = 16.0` 常量由计划 14 调整);未启用填中值 512 |

**域 D:UI/Overlay 族**(`Ui`/`Overlay` phase;screen-space UI 既有闭环不经此键,本域服务 queue≥4000 的材质与 overlay 内容)——ui z-index 映射进 Overlay 域的规则:queue 段固定为 `OVERLAY`(4000)基段(材质可 ±100 覆写),z-index 全量进 domain:

| 位区间 | 宽度 | 字段 | 编码 |
|---|---|---|---|
| 42–20 | 23 | `z_index` | `(ui_z_index.clamp(-4_194_304, 4_194_303) + 4_194_304) as u64`(与旧 `signed_23` 同宽,无损迁移) |
| 19–10 | 10 | 保留 | 填 0 |

域选择函数:`Transparent3d → B`;`*2d → C`;`Ui | Overlay → D`;其余(含 `PostProcess`/`Debug` 的少量命令)→ A。

**编码函数(签名与计划 02 已锁定的一致,不改)**:

```rust
// core/framework/render/core_pipeline/packed_sort_key.rs —— 唯一 u64 打包入口
pub const SORT_KEY_CAMERA_ORDER_SHIFT: u32 = 56;
pub const SORT_KEY_QUEUE_SHIFT: u32 = 43;
pub const SORT_KEY_DOMAIN_SHIFT: u32 = 10;

pub fn packed_sort_key_u64(
    phase: RenderPhase,
    components: RenderPhaseSortComponents,
    pipeline_variant: u32,      // 域 A/B 聚簇;域 C/D 忽略
    material_discriminant: u16, // 域 A 聚簇;其余域忽略
) -> u64;
```

`RenderPhaseSortComponents` 定稿(CO-M3 与唯一生产方 `phase_queue.rs` 同变更切换):

```rust
pub struct RenderPhaseSortComponents {
    pub camera_order: i32,        // 新增:CameraRenderDescriptor.render_order
    pub queue: RenderQueueValue,  // 合并替换 render_queue/material_queue 两个 i32
    pub sorting_layer: i32,       // 新增:域 C
    pub order_in_layer: i32,
    pub y_sort: Option<f32>,      // 新增:域 C
    pub depth: f32,
    pub depth_bias: f32,          // effective_depth = depth + depth_bias(保留)
    pub ui_z_index: i32,
    pub entity_tie_breaker: u64,
}
```

**从 i128 旧键的保序量化迁移方案**:

1. **queue 合并**:旧键 `render_queue`(15 bit,bit 112 起)高于 `material_queue`(15 bit)。新键单一 `queue` = 材质基础段 `from_alpha_mode` + `with_material_offset`。现有唯一生产方 `phase_queue.rs` 以"段值 + 段内偏移"填充两字段,合并是单调映射 → 保序。
2. **透明深度 23 bit**:沿用旧 `depth_sort_key` 的 `round(depth * 1000)` 量化(mm),仅把旧 signed_35 量程裁到 8388 m;默认 `z_far = 200 m` 场景内逐项保序,无回归。
3. **不透明深度降为 15 bit 粗量化(1/8 m)**:**有意非保序**。依据:opaque 深度序只是 early-z/overdraw 性能提示,正确性由 depth buffer 保证(URP `canSkipFrontToBackSorting` 在 HSR GPU 上整个放弃距离排序,见精读笔记);把高位让给 pipeline/material 聚簇,状态切换收益 > 精确深度序。等深桶内由 tie + 稳定排序定序。
4. **聚类位宽取舍**:`pipeline_cluster` 10 bit = 1024 变体/相机/phase,对照 mesh_pipeline_cache 现存变体数(双位数量级)余量 >10×;`material_cluster` 8 bit 分桶。两者折叠冲突只降低聚簇率(多一次状态切换),不影响绘制正确性 —— 这是 33 bit 域预算下牺牲冲突率换深度位的依据。
5. **为什么是绝对量化而非按 z_near/z_far 归一**:归一需要 per-view 参数进函数签名,破坏与计划 02 锁定的签名;且计划 02 MD-M3 缓存的静态 opaque 命令带陈旧深度位,绝对量化下陈旧度有界、归一量化在相机 z 范围变化时整体失真。
6. **`order_in_layer` 23→15 bit**:clamp ±16383(Unity order in layer 实践为 i16 域);extract 侧越界 `debug_assert` + clamp。`ui_z_index` 23→23 无损。`tie` 16→10 bit,由稳定排序兜底。
7. **对拍迁移测试**:`render_sort_key_matches_legacy_i128_order_on_representative_set` 用覆盖四域的样本分别按旧 i128 与新 u64 排序,断言除第 1/3 条声明的有意差异外顺序逐项一致;该测试随 i128 路径删除而转为固定快照断言。

若计划 02 的过渡实现(i128 分段压缩 u64)已先落地,CO-M3 只替换 `packed_sort_key_u64` 函数内部为本位段,消费方零改动;若本计划先行,则直接以本位段实现,02 落地时直接消费。

### 帧时序与集成点

宿主链路:`WgpuRenderFramework::submit_frame_extract`(`submit/submit.rs`)→ 新 `camera_loop.rs` → 逐相机 `SceneRendererCore` graph 执行。帧内顺序:

1. **extract**:scene 侧投影全部活动相机为 `RenderViewExtract.cameras: Vec<CameraRenderDescriptor>`;editor viewport 路径经 `from_camera` 退化为单 Base 相机,行为不变。
2. **序列解析**:`resolve_camera_sequence` 产出 Base 序列(render_order 升序;同 order 沿用 `camera_ordering.rs` 的 target 维度次序 + ambiguity 报告)。RT 相机(`target = Texture | Headless`)是普通 Base 序列项;其输出 texture 由消费方 graph import——若 RT 相机 render_order 晚于消费者,消费者读到上一帧内容(与 Unity 同 depth 行为一致,文档约定 RT 相机应取更小 render_order)。
3. **每 Base 相机一次 graph 执行**:`CompiledGraphCache::get_or_compile`(计划 01),**缓存键扩展点**:键成分增加 camera target 描述 = `RenderCameraTargetOrderKey`(kind + 尺寸/格式)+ `viewport_rect` 有无 + `render_type`。`render_order`/`culling_mask`/`volume_mask`/clear 颜色值是运行时参数,不进键(对应风险条目"缓存膨胀")。clear 策略翻译为首个 color/depth pass 的 load op:`Skybox→depth clear + sky pass`、`Color→clear(color)+clear(depth)`、`DepthOnly→load(color)+clear(depth)`、`None→load+load`。
4. **Overlay 子循环**:每个 Overlay 相机独立一次 graph 执行,color/depth attachment 以 import 方式复用 Base 的目标资源;color 恒 `load`(Overlay 永不清 color),depth 按 `clear_depth` 选 `clear|load`(URP `GetCameraClearFlag` Overlay 分支语义)。Overlay 继承 Base 的 target 与 viewport_rect(URP 以 baseCamera 构造 overlayCameraData 的对应物),自身仅贡献 view 矩阵、culling/volume mask 与深度策略。
5. **帧末**:screen-space UI 既有闭环 graph 末端节点只挂在"最后一个 PrimarySurface Base 相机"的执行上(URP `isLastBaseCamera` 语义);present/writeback 路径不动。

**硬切换删除项**(各切片落地同变更执行):

- `ViewportCameraSnapshot` 的 `order`/`target`/`viewport`/`clear_color`/`render_layers` 字段与其 serde default 函数(场景资产经 importer 一次性升级映射,不留双读路径)。
- `RenderPhase::mesh_phase(pipeline, alpha_mask, transparent)` 布尔签名。
- `phase_sort.rs` 的 i128 打包路径、`signed_15/23/35`、`RenderPhaseSortKey::new(raw: i64)`/`raw() -> i128`。
- `camera_ordering.rs` 中以 `ViewportCameraSnapshot` 为输入的 `RenderCameraOrderInput` 旧形态。
- `submit/submit.rs` 内单相机直通提交分支(由 camera_loop 单元素序列覆盖)。

### 实施切片细化

**CO-M1 相机契约与多相机循环**

| 切片 | 触碰文件 | 改动要点 | 完成判据 |
|---|---|---|---|
| M1-S1 契约迁移 | `camera_stack.rs`(新)、`camera.rs`、`camera_ordering.rs`、`frame_extract.rs`、scene 相机投影与编辑器相机面板字段 | `CameraRenderDescriptor` 三类型落地;snapshot 五字段迁出并删除;extract 增 `cameras` | `cargo check -p zircon_runtime --lib --locked` 通过;grep 无 `ViewportCameraSnapshot` 上的 `target`/`order` 残留访问 |
| M1-S2 多相机循环 | `camera_loop.rs`(新)、`submit/submit.rs`、`create_viewport/create.rs`、`compiled_graph_cache.rs` | 序列循环替换单相机提交;缓存键加 target 成分;RT 相机走普通序列项 | 双相机(主 + RT)帧各自产物正确;单相机路径回归不变 |

测试阶段:`cargo test -p zircon_runtime camera --locked` + `render_product` 回归;render_order 交换对拍断言。

**CO-M2 相机栈与 Overlay 合成**

| 切片 | 触碰文件 | 改动要点 | 完成判据 |
|---|---|---|---|
| M2-S1 栈合成 | `camera_stack.rs`、`camera_loop.rs` | `resolve_camera_sequence` 校验三规则;Overlay attachment import + load 语义 + `clear_depth` 分支 | overlay 内容叠于 base 之上像素断言;违规栈进 report 不 panic |
| M2-S2 分屏 | `camera_loop.rs`、`create_viewport/create.rs` | `viewport_rect.clamped_to_size` 应用到 attachment viewport/scissor | 双 Base 相机各占半屏像素断言 |

测试阶段:`cargo test -p zircon_runtime camera --locked`。

**CO-M3 RenderQueue 与统一排序键**

| 切片 | 触碰文件 | 改动要点 | 完成判据 |
|---|---|---|---|
| M3-S1 queue 契约 | `render_queue.rs`(新)、材质快照模块、`render_phase.rs`、`phase_queue.rs` | `RenderQueueValue` + 材质覆写字段 + 导入期矛盾校验;`mesh_phase` 改 queue 驱动并删旧签名 | check 通过;queue=2900 材质落 Transparent 段最前 |
| M3-S2 u64 位段定稿 | `packed_sort_key.rs`(新)、`phase_sort.rs`、`phase_sort_key_breakdown.rs`、`phase_sort_decision*.rs`、`phase_queue_ordering_key.rs` | 本章位段落地;i128 路径全删;breakdown 按新段重写;(计划 02 已落地时仅改函数内部) | check 通过;对拍迁移测试通过;repo 内 `i128` 在 core_pipeline 下无残留 |

测试阶段:`cargo test -p zircon_runtime phase --locked` + `cargo test -p zircon_runtime render_sort_key --locked`。

**CO-M4 layer 过滤全线贯通**

| 切片 | 触碰文件 | 改动要点 | 完成判据 |
|---|---|---|---|
| M4-S1 mask 贯通 | `scene_extract.rs`、计划 04 可见性入口、计划 05 灯光 extract、计划 07 volume extract | 四处字段统一 `RenderLayerSet`;per-view 过滤用 `culling_mask.intersects` | 两相机不同 mask 对象集合断言;灯光 mask 过滤断言 |

测试阶段:`cargo test -p zircon_runtime visibility --locked` + light/volume 范围回归。

### 测试与验收清单

单测(`render_<topic>_*` 命名,index.md §8 第 6 条):

| 测试函数 | 断言 | 位置 |
|---|---|---|
| `render_sort_key_camera_order_dominates_queue` | 不同 camera_order 的项排序先于一切 queue/深度差异 | `core_pipeline/packed_sort_key.rs` |
| `render_sort_key_queue_segment_maps_to_phase` | 1000/2000/2450/2500/2501/3000/4000 各值经 `phase()` 落入预期 `RenderPhase` | `core_pipeline/render_queue.rs` |
| `render_queue_material_offset_clamped_to_100` | offset=±150 被 clamp 到 ±100 | `core_pipeline/render_queue.rs` |
| `render_sort_key_opaque_clusters_pipeline_before_depth` | 两 pipeline 三深度样本先按 cluster 聚簇、簇内深度升序 | `core_pipeline/packed_sort_key.rs` |
| `render_sort_key_transparent_depth_back_to_front_ignores_cluster` | 透明排序深度降序主导,cluster 仅等深决胜 | 同上 |
| `render_sort_key_2d_sorting_layer_then_order_then_y` | 域 C 三级字典序成立;y 大者后绘 | 同上 |
| `render_sort_key_ui_z_index_maps_into_overlay_segment` | queue=OVERLAY 时 z_index 全序保持且不越出 domain 位 | 同上 |
| `render_sort_key_matches_legacy_i128_order_on_representative_set` | 迁移保序(声明差异除外),见迁移方案第 7 条 | 同上 |
| `render_sort_key_breakdown_roundtrip` | 打包→breakdown 解码逐字段还原 | `core_pipeline/phase_sort_key_breakdown.rs` |
| `render_camera_sequence_sorts_by_render_order` | 序列按 render_order 升序;同 order 报 ambiguity | `core/framework/render/camera_stack.rs` |
| `render_camera_stack_overlay_follows_base` | Overlay 紧随其 Base,保持 stack 内顺序 | 同上 |
| `render_camera_stack_rejects_invalid_members` | Overlay 带 stack / stack 引用 Base / target 不一致 → 进 violations 且被剔除 | 同上 |
| `render_camera_clear_overlay_never_clears_color` | Overlay 翻译出的 load op:color 恒 load;depth 按 clear_depth | `camera_loop.rs` |
| `render_camera_viewport_rect_clamped` | 越界 viewport_rect 被 clamp 进目标尺寸 | `camera.rs`(既有 `clamped_to_size` 扩展) |

产物对拍(`render_product_*` + `ZR_RENDERDOC_CAPTURE_NEXT=1` 人工比对):

| 场景 | 验收 |
|---|---|
| `render_product_dual_camera_rt_then_main` | RT 相机(order=-1)产物被主相机材质采样,内容正确 |
| `render_product_camera_render_order_swap_changes_composite` | 交换两 Base 相机 render_order 后合成层次反转 |
| `render_product_overlay_stack_composites_over_base` | 武器手/HUD overlay 叠加于 base,depth 按 clear_depth 两种策略各一例 |
| `render_product_split_screen_viewports` | 双相机各占半屏,边界无串扰 |
| `render_product_queue_override_reorders_draws` | queue=2900 不透明材质绘于 Geometry 之后、Transparent 之前(抓帧事件序) |

## 状态与产出记录

| 日期 | 里程碑/切片 | 状态 | 产出 | 验证与证据 | 后续 |
|------|-------------|------|------|------------|------|
| 2026-06-15 | CO-M1 camera contract and multi-camera loop | 部分完成: camera snapshot/layer 基础存在,Base/Overlay 语义未落地 | `ViewportCameraSnapshot`、`RenderCameraTarget`、`RenderLayer`、temporal jitter 与 view/projection matrix pair 已被计划 06 使用;计划 04 shadow view 也有独立 view key,但多 camera render loop/target 合成仍未统一。 | 计划 06 TP-M2 状态表记录 `temporal_jitter`、Halton、matrix pair 与 scene uniform ABI;计划 04 状态表记录 per-view visibility。 | 实施 `RenderFrameExtract` 多相机快照、相机 depth 顺序和 custom target 输出链。 |
| 2026-06-15 | CO-M2 camera stack and overlay composition | 未启动: 仍为计划设计 | 当前多 viewport 不等于 Base/Overlay camera stack,无 overlay 合成语义。 | 本文件 `现状与差距` 明确无相机栈和相机间 depth 顺序概念。 | 定稿 Base/Overlay target reuse、clear flags、UI/scene overlay 顺序和 history 共享规则。 |
| 2026-06-15 | CO-M3 RenderQueue and unified sort key | 部分完成: phase sort 已有,统一 queue override 未完成 | `phase_queue.rs`/`phase_sort.rs` 有 Opaque/AlphaMask/Transparent/Sprite2d 等 phase 排序;计划 03/04/05 已依赖 source entity、view visibility 和 shadow sorting,但材质 queue 数值覆写、sprite/UI/3D 统一 sort key 尚未落地。 | 本文件 `现状与差距` 记录 phase 硬编码;计划 05/06 状态表记录 shadow/TAA pass order 依赖现有 phase。 | 实施统一 sort key 位段、材质 queue override 和 world-space UI/sprite 混排规则。 |
| 2026-06-15 | CO-M4 layer filtering across the stack | 部分完成: visibility/shadow layer 使用已接入,多相机全线贯通未完成 | 计划 04 的 FrameVisibility 与计划 05 shadow view 已按 view key/layer mask 消费可见集;但 custom render target camera 与 UI/2D/overlay layer 仍未统一。 | 计划 04 VC-M1 状态表记录 directional cascade/point face/spot shadow view;计划 05 LS-M3 状态表记录 shadow atlas view key 消费。 | 等 CO-M1 多相机快照完成后贯通 render queue、UI、2D、shadow 和 post history。 |

### 参考实现精读笔记

| 真实符号 | 要点 | Zircon 对应物与取舍 |
|---|---|---|
| `UniversalRenderPipelineCore.cs` `cameraComparison = (camera1, camera2) => (int)camera1.depth - (int)camera2.depth`、`SortCameras` | 相机间只按 float depth 截断 int 比较,无次级键,同 depth 顺序未定义 | `render_order: i32` + `camera_ordering.rs` 既有 ambiguity 报告:同 order 同 target 显式诊断而非未定义——比 URP 更严,代价是多一份 report 结构,接受 |
| `UniversalRenderPipeline.cs` `RenderCameraStack`(L1038):`renderType == CameraRenderType.Overlay` 即 return;`lastActiveOverlayCameraIndex`;overlay 循环内 `CreateCameraData(overlayFrameData, baseCamera, baseCameraAdditionalData)` 后 `InitializeAdditionalCameraData(overlayCamera, ...)` 覆写 | Overlay 不独立成帧;overlay 的 CameraData 先以 **base** 相机构造再逐相机覆写 → target/viewport 继承 Base 是构造性事实;"最后活跃相机"决定 resolve-to-screen 时机 | `resolve_camera_sequence` 把 Overlay 收进 `CameraSequenceEntry.overlays`;继承规则写死为 target/viewport_rect 来自 Base。Zircon 用 graph import 表达 resolve 时机,不需要 lastActive 标志位,简化接受 |
| `UniversalAdditionalCameraData.cs`:`CameraRenderType { Base, Overlay }`、`cameraStack`(非 Base 访问报 warning 返 null)、`clearDepth`、`volumeLayerMask`、`renderPostProcessing` | 相机扩展字段清单样板;stack 仅 Base 暴露的访问器级防御 | `CameraRenderDescriptor` 字段一一对应(`renderPostProcessing` 归计划 07,不进本契约);防御移到 `resolve_camera_sequence` 的数据校验,契约结构保持 plain data |
| `ScriptableRenderer.cs` `GetCameraClearFlag`(L1031):Overlay → `clearDepth ? ClearFlag.DepthStencil : ClearFlag.None`;Skybox/Nothing 且 `msaaSamples > 1` → 额外清 color(防 alpha-to-coverage 混上一帧) | clear 决策集中一处、按 render_type 分叉;MSAA 边角案例显式处理 | `RenderCameraClear` → load op 翻译表集中在 `camera_loop.rs` 单函数;MSAA>1 时 Skybox/None 同样强制清 color,采纳该边角 |
| `UniversalRenderPipeline.cs` L1667:`commonOpaqueFlags = SortingCriteria.CommonOpaque`、`noFrontToBackOpaqueFlags = SortingCriteria.SortingLayer \| RenderQueue \| OptimizeStateChanges \| CanvasOrder`、`canSkipFrontToBackSorting`(HSR GPU 或 `OpaqueSortMode.NoDistanceSort`) | opaque 的距离排序是**可整体放弃**的优化项,SortingLayer/RenderQueue/状态切换优化才是常驻准则 | 直接支撑位段决策:域 A 深度只占 15 bit 粗量化、聚簇位在深度之上;Zircon 不做运行时开关(位段静态),以聚簇优先的单一布局覆盖两种模式,取舍接受 |

## 风险与回退

- 多相机 × compiled graph 缓存膨胀:缓存键只含影响图结构的相机字段(目标格式/尺寸/feature),order/mask 类字段走运行时参数,避免每相机一份图。
- queue 自由覆写破坏 phase 假设(如 queue<2500 却半透明混合):材质验证器在资产导入期诊断 queue 与混合态矛盾。
- screen-space UI 已闭环路径不动:本计划只接管 world-space UI 的排序归属,越界改动按硬切换原则需先在 UI 计划侧立项。
