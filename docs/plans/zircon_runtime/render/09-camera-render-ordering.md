---
related_code:
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_queue.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_sort.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/pipeline_kind.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
  - zircon_runtime/src/graphics/types/viewport_render_output_target.rs
  - zircon_runtime/src/graphics/types/viewport_render_region.rs
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

- `camera.rs` 有 `ViewportCameraSnapshot`/`RenderCameraTarget`/`RenderLayer` 基础;2026-06-18 已新增 `CameraRenderDescriptor` 与 `RenderViewExtract.cameras`,让计划 04 custom-target visibility、submit preflight、runtime-frame accessors 和 temporal history validation 改读 descriptor target/layer/viewport/order/clear;`ViewportCameraSnapshot` 上 `target`/`order`/`viewport`/`clear_color`/`render_layers` 五字段已删除,offscreen submit 已有 Base/Overlay descriptor loop scaffold,但 editor 面板字段、WGPU target 合成/output ownership 与 Base/Overlay attachment/load-op 仍未完成。
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
| `zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop.rs` | 帧内多相机循环:相机序列 → 逐 Base 相机 graph 执行 → 逐 Overlay 相机 graph 执行(复用 Base attachment) |

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
| 2026-06-15 | CO-M1 camera contract and multi-camera loop | 部分完成: camera snapshot/layer 基础存在,2026-06-18 visibility payload bridge、neutral descriptor/sequence contract、extract-side `RenderViewExtract.cameras`、descriptor-driven visibility consumer、selected descriptor submit preflight、descriptor-owned snapshot hard cutover 与 offscreen submit loop scaffold 已接入;完整 Base/Overlay 合成仍未完成 | `ViewportCameraSnapshot`、`RenderCameraTarget`、`RenderLayer`、temporal jitter 与 view/projection matrix pair 已被计划 06 使用;`CameraRenderDescriptor`/`resolve_camera_sequence` 已提供 Base/Overlay 序列契约;scene-backed `RenderViewExtract` 现在携带 active scene camera descriptor 列表,scene layer candidate union、custom-target visibility、submit target resolution/output-target/present preflight 已从 selected descriptor 读取 target/layer;offscreen submit 已通过 `submit/camera_loop.rs` 将 Base/Overlay descriptors 投影为 selected-camera child extracts。 | 计划 06 TP-M2 状态表记录 `temporal_jitter`、Halton、matrix pair 与 scene uniform ABI;计划 04 状态表记录 per-view visibility 和 2026-06-18 custom-target visibility payload bridge;core-min `cargo check`、lib-test `--no-run` 与 direct binary camera/extract/visibility exact tests 通过;本表新增 2026-06-18 descriptor/sequence、extract descriptor、descriptor visibility consumer、selected descriptor submit preflight、descriptor-owned hard cutover 与 M1-S2 loop scaffold focused evidence。 | 完成 Base/Overlay attachment reuse/load-op、custom target WGPU composite/output ownership、per-camera post/history/light ownership、surface present/direct runtime-frame 多相机路径和 editor authoring 面板。 |
| 2026-06-18 | CO-M1 M1-S1 camera descriptor/sequence contract | 部分完成: neutral `CameraRenderDescriptor` 与 Base/Overlay sequence resolver 落地;hard cutover 和 submit loop 未开始 | 新增 `core/framework/render/camera_stack.rs`,定义 `CameraRenderType`、`RenderCameraClear`、`CameraRenderDescriptor`、`CameraSequenceReport`、`CameraSequenceEntry` 与 violation reason;`resolve_camera_sequence(...)` 只让 Base 相机成序,按 Base stack 顺序挂接 Overlay,继承 Base target/viewport,并把缺失、非 Overlay、target 不匹配、Overlay 带 stack 记入 report;`camera_ordering.rs` 的 `RenderCameraOrderInput` 改为 descriptor-backed,保留 `ViewportCameraSnapshot` transitional constructor;`mod.rs` 导出新契约。 | `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-stack-contract-0618 --message-format short --color never` 通过(既有 warning set);filtered `cargo test -p zircon_runtime --lib render_camera_ ...` 在 lib-test binary 编译完成后因工具 timeout/BrokenPipe 未计为通过;直接运行 hot binary exact tests 通过: `render_camera_sequence_sorts_by_render_order`、`render_camera_stack_overlay_follows_base_and_inherits_target_viewport`、`render_camera_stack_rejects_invalid_members`、`render_camera_ordering_sorts_by_order_then_target_and_tracks_target_hdr_index`。 | 继续 M1-S1 hard cutover: `RenderViewExtract.cameras`、scene/editor projection fields、删除 snapshot 上 target/order 直接所有权;随后协调 dirty submit_frame_extract worktree 后接 `camera_loop.rs`、per-camera post/history/light ownership 和 Base/Overlay WGPU 输出。 |
| 2026-06-18 | CO-M1 M1-S1 extract-side camera descriptor cutover | 部分完成: `RenderViewExtract.cameras` 已落地并由 scene extract 填充 active scene camera descriptors;single-camera submit 仍读 `view.camera` | `RenderViewExtract` 新增 `cameras: Vec<CameraRenderDescriptor>`, synthetic/explicit camera 路径自动生成单元素 descriptor;scene-backed extract 以 deterministic camera schedule order 写入 active scene camera descriptors,并把 selected scene camera descriptor 对齐到 request override 后的 effective `view.camera`;`apply_target_size(...)` 只同步 selected/synthetic descriptor,避免污染其他 Texture/Headless camera descriptors;`SortedRenderCamera` 增 `render_type`。 | `rustfmt --edition 2021` 已跑过 touched Rust files;`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-extract-descriptors-0618 --message-format short --color never` 通过(既有 warning set);三次 `cargo test --exact` 因未给完整路径只匹配 0 tests,不计为通过;随后直接运行 lib-test binary exact tests 通过: `scene::tests::render_extract::render_frame_extract_carries_scene_camera_order_report_for_scene_camera`、`scene::tests::render_extract::explicit_camera_render_frame_extract_has_no_scene_camera_order_report`、`core::framework::render::camera_ordering::tests::render_camera_order_report_carries_descriptor_render_type`。 | 继续 M1-S1: 把 scene/editor 相机字段投影迁到 descriptor 所有权,再删除 `ViewportCameraSnapshot` 上 `target`/`order`/`viewport`/`clear_color`/`render_layers` 残留直接所有权;之后协调 dirty submit path 接 `camera_loop.rs`。 |
| 2026-06-18 | CO-M1 M1-S1 descriptor-driven visibility consumer cutover | 部分完成: scene layer union 与 custom-target visibility 已消费 `RenderViewExtract.cameras`;single-camera submit 仍读 `view.camera` | `RenderViewExtract::selected_camera_descriptor()` 提供 selected scene/synthetic descriptor read path;`World::render_extract_layers_for_view(...)` 改用 selected descriptor layers 加 Texture/Headless descriptor layers 构造 mesh/sprite candidate layer union;`FrameVisibility::from_frame_views(...)` 改接收 `&[CameraRenderDescriptor]`, custom-target views 从非 PrimarySurface descriptors 构建并以 descriptor `culling_mask` 过滤 primitive relevance;`RenderCameraOrderReport` 保留 ordering diagnostics,不再作为 visibility custom-target payload source。 | `rustfmt --edition 2021` 已跑过 touched Rust files;`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-descriptor-visibility-0618 --message-format short --color never` 通过(既有 warning set);首次并行 `cargo test` 因同 target dir Cargo lock contention 超时且未产出 binary,不计为通过;`cargo test -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-descriptor-visibility-0618 --no-run --message-format short --color never` 通过;直接运行 lib-test binary exact tests 通过: `graphics::visibility::context::from_extract_with_history::construct::tests::visibility_context_builds_custom_target_view_from_camera_descriptors`、`scene::tests::render_extract::render_frame_extract_keeps_custom_target_layer_geometry_for_visibility_views`、`scene::tests::render_extract::render_frame_extract_carries_scene_camera_order_report_for_scene_camera`。 | 继续 M1-S1: 把剩余 scene/editor 相机字段读写迁到 descriptor 所有权,删除 `ViewportCameraSnapshot` 上 `target`/`order`/`viewport`/`clear_color`/`render_layers` 直接所有权;随后协调 dirty submit path 接 `camera_loop.rs`。 |
| 2026-06-18 | CO-M1 M1-S1 selected descriptor projection and submit preflight cutover | 已被后续 hard cutover 取代: selected camera target/layer/viewport sizing consumers started reading descriptor;`camera_loop.rs` 未开始 | `CameraRenderDescriptor::as_effective_camera()` 与 `RenderViewExtract` selected descriptor helpers 让 submit target resolution/output-target/target diagnostics、surface present preflight、post-process volume layer resolution 与 main-view visibility relevance 改读 selected descriptor;当时仍保留 snapshot 过渡投影,随后同日 hard cutover 删除了 snapshot 五字段。 | `rustfmt --edition 2021` 已跑过 touched Rust files;`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-selected-descriptor-0618 --message-format short --color never` 通过(既有 warning set);新增 `render_view_apply_target_size_preserves_descriptor_target_and_layers` 覆盖 descriptor target/layer/viewport sizing projection;focused `cargo test -p zircon_runtime --lib render_view_apply_target_size_preserves_descriptor_target_and_layers --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-selected-descriptor-0618 --message-format short --color never -- --nocapture` 在 shared lib-test 编译阶段 904s timeout,无测试结果,已清理该 target-dir Cargo/rustc 进程,不计为通过。 | 已由后续 descriptor-owned snapshot field hard cutover 收束 snapshot 字段删除;继续 `camera_loop.rs`、per-camera history/light/post ownership 与 custom target WGPU 输出。 |
| 2026-06-18 | CO-M1 M1-S1 runtime-frame camera descriptor accessor cutover | 部分完成: renderer-bound `ViewportRenderFrame::camera()` 已返回 selected `CameraRenderDescriptor`;snapshot payload 仍是 transitional projection,previous motion-vector camera 仍为 snapshot,`camera_loop.rs` 未开始 | `ViewportRenderFrame::camera()` 改为 descriptor-backed accessor,新增 `effective_camera()` 作为矩阵/阴影/粒子/post-process/temporal history 等 legacy math consumers 的 selected descriptor projection;`RenderViewExtract::sync_selected_descriptor_camera_payload()` 在 frame 构造与 submit jitter 投影时同步 transform/projection/dynamic-resolution/MSAA/temporal jitter payload,同时保留 target/order/viewport/clear/layer 的 descriptor 所有权;scene uniform、shadow cascades、selection/handle/gizmo billboard axes、particle quads、DoF prepare、velocity-camera pass、temporal/particle history 改读 `effective_camera()`。 | `rustfmt --edition 2021` 已跑过 touched Rust files;stale scan 确认 `frame.camera()` snapshot-style consumers 已迁移,仅保留 `frame.camera().camera.temporal_jitter` descriptor payload regression assertion;`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-runtime-frame-descriptor-0618 --message-format short --color never` 通过(既有 141 warning set,6m32s);focused unit/product tests 未在本实现切片执行,留到 milestone testing stage。 | 继续 M1-S1: 迁移剩余 scene/editor 相机字段写入并删除 `ViewportCameraSnapshot` 上 `target`/`order`/`viewport`/`clear_color`/`render_layers` 直接所有权;随后接 `camera_loop.rs`、per-camera history/light/post ownership 与 custom target WGPU 输出。 |
| 2026-06-18 | CO-M1 M1-S1 descriptor-owned snapshot field hard cutover | 部分完成: `ViewportCameraSnapshot` 五个 descriptor-owned 字段已删除;single-effective-camera submit 仍存在,`camera_loop.rs` 未开始 | `ViewportCameraSnapshot` 现在只保留 transform/projection/aspect/active/HDR/exposure/MSAA/dynamic-resolution/temporal-jitter payload;`CameraRenderDescriptor` 独占 target、viewport_rect、render_order、clear、culling_mask、volume_mask;`SceneViewportExtractRequest::camera` 改为 `Option<CameraRenderDescriptor>`;scene extract、ordering、visibility、submit preflight、runtime-frame accessors、pipeline/test fixtures 和 `FrameHistoryValidationKey` 均改读 selected descriptor,避免 history/target/layer state 因 snapshot 收缩丢失。 | `cargo fmt --package zircon_runtime -- --check` 通过;`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-descriptor-hard-cutover-0618 --message-format short --color never` 通过(既有 warning set);`cargo test -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-descriptor-hard-cutover-0618 --no-run --message-format short --color never` 通过(既有 warning set);stale field scans、conflict marker scan、scoped `git diff --check` 通过,仅有仓库 LF/CRLF 提示。 | 下一步转 M1-S2 前先协调 editor 面板/authoring 字段与 dirty submit worktree;随后实现 `camera_loop.rs`、per-camera post/history/light ownership、Base/Overlay load-op 翻译和 custom-target WGPU 输出。 |
| 2026-06-18 | CO-M1 M1-S2 submit camera loop scaffold | 部分完成: generated offscreen submit 已按 resolved Base/Overlay descriptors 循环执行 selected-camera child extracts;Base/Overlay load-op、target reuse/composite、present/direct runtime-frame 与 per-camera ownership 未完成 | 新增 `graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop.rs`;`submit_frame_extract_with_ui(...)` 在一个 operation lock 与 `submit_frame_extract` profile scope 内调用 `submit_camera_loop(...)`;loop 使用 `resolve_camera_sequence(...)` 扁平化 Base→Overlay 顺序,并用 `RenderFrameExtract::with_selected_camera_descriptor(...)` 生成只含当前 selected descriptor 的 child extract 后复用原 single-camera submit body。 | `cargo fmt --package zircon_runtime` 通过;`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-loop-0618 --message-format short --color never` 通过(既有 warning set);`cargo test -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-loop-0618 --no-run --message-format short --color never` 通过;focused tests 通过: `camera_loop` 2 tests 与 `render_frame_extract_selected_camera_descriptor_replaces_active_selection_only` 1 test。 | 继续 M1-S2/M2: Base/Overlay attachment load/store 与 clear/clear_depth 翻译,自定义 target 输出 ownership/composite,per-camera post/history/light state,screen-space UI final composite ownership,surface present/direct runtime-frame 多相机路径,以及双相机 RT/product 像素证据。 |
| 2026-06-18 | CO-M1 M1-S2 screen-space UI terminal routing | 部分完成: `camera_loop` 已把共享 UI extract 路由到 terminal Base stack child;full Base/Overlay composite/attachment ownership 未完成 | `camera_loop_submissions(...)` 现在为每个 selected-camera child 标记 `receives_terminal_ui`;`terminal_screen_space_ui_camera_position(...)` 选择最后一个 `PrimarySurface` Base stack,没有 primary 时回退到最后一个 Base stack 以保留 texture/headless-only 单目标 UI 行为;terminal child 是该 stack 的最后一个 Overlay,没有 Overlay 时为 Base;非 terminal child 传入 `None` UI。 | `cargo fmt --package zircon_runtime` 通过;`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-ui-terminal-0618 --message-format short --color never` 通过(既有 warning set);`cargo test -p zircon_runtime --lib camera_loop --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-ui-terminal-0618 --message-format short --color never -- --nocapture` 通过 4 tests。 | 继续 M1-S2/M2: Base/Overlay attachment load/store 与 clear/clear_depth 翻译,final target composite ownership,per-camera post/history/light state,surface present/direct runtime-frame 多相机路径,以及双相机 RT/product 像素证据。 |
| 2026-06-18 | CO-M1 M1-S2 camera target compile fingerprint | 部分完成: compiled graph cache key 已包含 selected camera target/render type/viewport-rect presence;texture target format 尚未进入 key,完整 composite ownership 未完成 | `CompiledGraphCacheKey` 的 `RenderGraphCompileFrameFingerprint` 新增 `RenderGraphCompileCameraTargetFingerprint`、`camera_render_type` 与 `viewport_rect_present`;target 指纹区分 PrimarySurface、Texture `ResourceId` 与 Headless size;`extract_compile_fingerprint(...)` 改读 selected `CameraRenderDescriptor` 的 camera payload,避免 descriptor hard cutover 后与 transitional `view.camera` 不一致;`CameraRenderType` 补 `Hash`。 | `cargo fmt --package zircon_runtime -- --check` 通过;`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-target-fingerprint-0618 --message-format short --color never` 通过(既有 warning set);首次 `cargo test -p zircon_runtime --lib compiled_graph_cache ...` 暴露 test-only 导入/move 与旧测试直接改 `view.camera` 的契约漂移,修正后同命令通过 8 tests。 | 后续若 final target format 成为编译期 graph shape 输入,把 prepared texture descriptor/format class 传入 key;继续 CO-M2 physical attachment reuse、final output owner、per-camera history/post/light 与像素证据。 |
| 2026-06-18 | CO-M1/M2 camera clear-load attachment policy | 部分完成: selected-camera child frame 已把 Base/Overlay clear 语义转成 scene color/depth 首写 load ops;物理 attachment reuse/composite 未完成 | 新增 `graphics/types/viewport_camera_stack_attachment_policy.rs`;`ViewportRenderFrame` 构造时从 selected `CameraRenderDescriptor` 派生策略;`RenderPassGpuExecutionContext` 暴露 frame policy;`RenderPassExecutionContext::attachment_ops_for_write(...)` 在 GPU frame 存在时只改 `scene-color`/`scene-depth` graph-declared 首写 `Clear`,保留 store op 与后续 `Load` 写入。Base `Skybox`/`Color` 清 color+depth,`DepthOnly` load color/clear depth,`None` load+load,`None`+MSAA>1 清 color/load depth;Overlay color 恒 load,depth 按 `clear_depth`。 | `cargo fmt --package zircon_runtime` 已运行;`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-attachment-policy-0618 --message-format short --color never` 通过(既有 warning set);新增 4 个策略源码合同测试,但 `cargo test -p zircon_runtime --lib viewport_camera_stack_attachment_policy ...` 在 shared lib-test 编译阶段 904s timeout,无测试通过结果,残留 Cargo/rustc 进程已清理。 | 继续 CO-M2: Overlay 复用 Base 物理 color/depth attachment/import,final Base-stack composite owner,per-camera post/history/light state,custom target 输出、split-screen product evidence 和像素/RenderDoc 对拍。 |
| 2026-06-18 | CO-M1/M2 camera stack output ownership policy | 部分完成: camera loop 已标记 stack/viewport terminal child,非 stack-terminal child 不再写 texture output target;generated offscreen 非 viewport-terminal child 不再消费 pending capture、分配 shared history、写 viewport record/history/stats;物理 composite 与独立 per-camera state 未完成 | 新增 `graphics/types/viewport_camera_stack_output_policy.rs`;`camera_loop` 为每个 child 计算 `stack_terminal` 与 `viewport_terminal`,并把策略经 `build_runtime_frame(...)` 写入 `ViewportRenderFrame`。`direct_imported_final_target(...)` 对非 stack-terminal child 返回 `None`,`render_frame_with_pipeline_to_target(...)` 对这些 child 调用 `suppress_output_target_writeback(...)`,`output_target_capture_resource(...)` 不再从 prepared texture target 捕获,避免中间 Base/Overlay child 过早改写最终 texture 输出。`submit_selected_camera_frame(...)` 现在用 `owns_viewport_submission()` gate graphics-debugger capture、`resolve_history_handle(...)`、`record_submission(...)`、temporal/particle previous-state、history release、virtual-geometry debug snapshot 和 `update_stats(...)`;非 owner 使用 inactive history 并只 drain renderer feedback。Present/direct runtime-frame 路径保持默认 terminal 策略。 | `cargo fmt --package zircon_runtime -- --check` 通过;`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-output-policy-0618 --message-format short --color never` 通过(既有 warning set);首次 focused tests 因并行同 target-dir 与 cold lib-test compile timeout,不计为通过;热 rerun 通过 `cargo test -p zircon_runtime --lib camera_loop ...` 5 tests 和 `cargo test -p zircon_runtime --lib build_runtime_frame ...` 1 test。viewport-owner follow-up `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-record-owner-0618 --message-format short --color never` 通过(既有 warning set),hot rerun 通过 `cargo test -p zircon_runtime --lib camera_loop ...` 5 tests 与 `cargo test -p zircon_runtime --lib resolve_history_handle ...` 1 test。 | 继续 CO-M2: Overlay 复用 Base 物理 color/depth attachment/import,独立 per-camera post/history/light state,custom target composite 输出、split-screen product evidence 和像素/RenderDoc 对拍。 |
| 2026-06-18 | CO-M2 fixed scene target physical reuse contract | 部分完成: Base/Overlay child graph submits 共享 fixed `OffscreenTarget` scene color/depth backing 的底层契约已受测;final composite 与像素/product 验收未完成 | `bind_frame_graph_resources(...)` 对 live `SCENE_COLOR`/`SCENE_DEPTH` 绑定 renderer-owned `OffscreenTarget` views,不让它们进入 graph-owned transient materialization;`ViewportCameraStackAttachmentPolicy` 决定 Base/Overlay 首写 clear/load,因此 Overlay load 可以保留 Base child 的物理 color/depth 内容。新增 `frame_binder_reuses_fixed_scene_color_and_depth_targets` 断言 scene targets 是 external texture views 且没有 owned transient backing。 | `cargo fmt --package zircon_runtime -- --check` 通过;首次 fresh target-dir focused test 超时且无产物,不计为通过;热 target-dir `cargo test -p zircon_runtime --lib frame_binder_reuses_fixed_scene_color_and_depth_targets --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-record-owner-0618 --message-format short --color never -- --nocapture` 通过 1 test(既有 warning set)。 | 继续 CO-M2: final custom-target composite owner、独立 per-camera post/history/light/HGI/VG/particle state、surface present/direct runtime-frame 多相机路径和 product/RenderDoc 对拍。 |
| 2026-06-19 | CO-M2 selected-camera viewport/scissor render region | 部分完成: graph-raster camera passes now apply selected `CameraRenderDescriptor.viewport_rect` as clamped WGPU viewport+scissor;split-screen raster isolation contract exists,但 final composite/product evidence 仍未完成 | 新增 `ViewportRenderRegion` 并挂到 `ViewportRenderFrame`;frame constructors derive it from selected camera plus target size;normal prepass、base mesh、deferred G-buffer/lighting、sprite、CPU particle billboard、particle velocity、TAA reactive-mask mesh、preview sky、grid/wireframe/selection/gizmo/handle overlays 与 particles plugin GPU transparent draw 都应用该 region;plugin direct tests use `ViewportRenderRegion::full_target(...)` for old full-target behavior。 | `cargo fmt --package zircon_runtime -- --check` 通过;`cargo fmt -p zircon_plugin_particles_runtime -- --check` 通过;`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-viewport-region-0619 --message-format short --color never` 通过(既有 warning set);`cargo test -p zircon_runtime --lib viewport_render_region --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-viewport-region-0619 --message-format short --color never -- --nocapture` 通过 3 tests;`cargo check -p zircon_plugin_particles_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-viewport-region-0619-particles --message-format short --color never` 通过;`cargo test -p zircon_plugin_particles_runtime particle_gpu_runtime_owner_ --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-viewport-region-0619-particles --message-format short --color never -- --nocapture` 通过 4 tests。 | 继续 CO-M2: final custom-target composite owner、独立 per-camera post/history/light/HGI/VG/particle state、surface present/direct runtime-frame 多相机路径和 product/RenderDoc 对拍。 |
| 2026-06-19 | CO-M2 post-process viewport origin and terminal writeback | 部分完成: fullscreen post-process now distinguishes local graph-owned viewport textures from fixed full-frame source/history textures and writes terminal output only into selected-camera region;final composite/product evidence 仍未完成 | `PostProcessParams` packs local viewport size, physical viewport origin, cluster dimensions, and scene source origin separately; `post_process.wgsl` / SSR WGSL read graph intermediates locally and fixed frame/history/G-buffer/AO inputs through physical origin; split bloom/DoF/motion-blur/scene-composite/blur/SSR executors receive source origins; `BLOOM` and `GLOBAL_ILLUMINATION` are graph-owned post-process textures; Hybrid GI history copies graph-owned GI into the selected camera physical history region; output-transfer/FXAA/final SMAA resolve apply `ViewportRenderRegion` viewport/scissor and subtract a terminal-origin uniform while SMAA edge/blend remain local. | `cargo fmt --package zircon_runtime` 通过;`cargo test -p zircon_runtime --lib post_process_params_pack_viewport_and_scene_source_origins_separately --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-postprocess-region-tests-0619 --message-format short --color never -- --nocapture` 通过 1 test;`cargo test -p zircon_runtime --lib viewport_region_maps_local_postprocess_coords_to_physical_target_coords --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-postprocess-region-tests-0619 --message-format short --color never -- --nocapture` 通过 1 test;`cargo test -p zircon_runtime --lib history_region_copy --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-postprocess-region-tests-0619 --message-format short --color never -- --nocapture` 通过 2 tests;`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-postprocess-region-tests-0619 --message-format short --color never` 通过(既有 warning set)。 | 继续 CO-M2: final custom-target composite owner、UI/scene overlay 顺序、per-camera light/VG/particle state、TAA/SSR/history product rules、surface present/direct runtime-frame 多相机路径和 product/RenderDoc 对拍。 |
| 2026-06-15 | CO-M2 camera stack and overlay composition | 部分完成: neutral Base/Overlay sequence contract、scene color/depth 首写 clear/load 策略、fixed scene target physical reuse 契约、texture-output stack-terminal gate、generated offscreen viewport-terminal record/history/stats owner gate、selected-camera viewport/scissor graph-raster region 与 fullscreen post-process viewport-origin/terminal writeback policy 已存在,最终 composite 未完成 | `resolve_camera_sequence(...)` 已表达 Overlay 不独立成序、Base stack 顺序、target/viewport 继承和违规 report;`ViewportCameraStackAttachmentPolicy` 已把 Base clear/Overlay `clear_depth` 翻译到 graph-declared first clear 写入;`bind_frame_graph_resources(...)` 已把 live `SCENE_COLOR`/`SCENE_DEPTH` 导向 fixed `OffscreenTarget` views;`ViewportCameraStackOutputPolicy` 已阻止非 stack-terminal child 直接 import/writeback/capture texture output target,并阻止非 viewport-terminal child 写 shared viewport record/history/stats;`ViewportRenderRegion` 已把 selected `viewport_rect` 应用为 graph-raster 与 terminal post-process WGPU viewport/scissor;post-process graph-owned intermediates localize selected-camera execution,固定 frame/history inputs 通过 origin 采样。当前仍没有最终 custom-target composite、完整 per-camera light/VG/particle state、present/direct runtime-frame 多相机路径或 UI/scene overlay product 验收。 | 2026-06-18 descriptor/sequence focused tests 覆盖排序、继承与违规剔除;camera attachment/output policy `core-min` check 通过;output policy focused tests 覆盖 stack/viewport terminal flags、frame policy carry 和 inactive history gate;fixed scene target binding focused test 通过;2026-06-19 viewport region tests、particles transparent plugin tests 与 post-process origin/history focused tests 通过;尚无像素级 overlay/product 证据。 | 继续实现 final composite owner、UI/scene overlay 顺序、per-camera state 产品规则、surface present/direct runtime-frame 多相机路径和 product/RenderDoc 对拍。 |
| 2026-06-15 | CO-M3 RenderQueue and unified sort key | 部分完成: phase sort 已有,统一 queue override 未完成 | `phase_queue.rs`/`phase_sort.rs` 有 Opaque/AlphaMask/Transparent/Sprite2d 等 phase 排序;计划 03/04/05 已依赖 source entity、view visibility 和 shadow sorting,但材质 queue 数值覆写、sprite/UI/3D 统一 sort key 尚未落地。 | 本文件 `现状与差距` 记录 phase 硬编码;计划 05/06 状态表记录 shadow/TAA pass order 依赖现有 phase。 | 实施统一 sort key 位段、材质 queue override 和 world-space UI/sprite 混排规则。 |
| 2026-06-15 | CO-M4 layer filtering across the stack | 部分完成: visibility/shadow layer 使用已接入,custom-target visibility payload 已接入,多相机全线贯通未完成 | 计划 04 的 FrameVisibility 与计划 05 shadow view 已按 view key/layer mask 消费可见集;2026-06-18 起 scene extract 会为 Texture/Headless scene cameras 合并 mesh/sprite 候选层,custom target visibility 使用各自 camera layer mask;但 UI/2D/overlay layer、post/history/light ownership 和 WGPU target loop 仍未统一。 | 计划 04 VC-M1 状态表记录 directional cascade/point face/spot shadow view 与 custom-target visibility payload bridge;计划 05 LS-M3 状态表记录 shadow atlas view key 消费。 | 等 CO-M1 多相机 loop/descriptor 完成后贯通 render queue、UI、2D、shadow、post history 和 target 输出。 |

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
