---
related_code:
  - zircon_runtime_interface/src/ui/surface/render/command.rs
  - zircon_runtime_interface/src/ui/surface/render/batch.rs
  - zircon_runtime_interface/src/ui/surface/render/batch/key.rs
  - zircon_runtime_interface/src/ui/surface/render/batch/plan.rs
  - zircon_runtime_interface/src/ui/surface/render/batch/clip.rs
  - zircon_runtime/src/ui/surface/render/extract.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/batching.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/geometry.rs
  - zircon_runtime/crates/zr_rhi
  - zircon_runtime/crates/zr_rhi_wgpu
plan_sources:
  - docs/plans/zircon_editor/editor_layout/10-real-rendering-pipeline-and-contract.md
  - docs/plans/zircon_editor/editor_layout/09-incremental-message-bus-and-refresh.md
  - docs/plans/zircon_editor/editor_layout/16-relative-layout-and-resolution-adaptation.md
  - docs/plans/zircon_editor/editor_layout/17-text-rendering-and-typography.md
  - docs/plans/zircon_runtime/render/14-2d-stack.md  # 2D 栈勾稽(2026-07-02 评审收口)
  - docs/plans/zircon_runtime/text/04-glyph-atlas-and-rasterization.md  # 文本图集共享服务(2026-07-02 评审收口)
status: in_progress
---
# 21 GPU 提交与绘制管线(批次合并 / 裁剪栈 / 图集 / 顶点 / 上屏)

## 1. 目标

把"绘制命令如何变成最少的 GPU draw call 上屏"沉淀为一份**GPU 提交规范**。当前 interface 已实现 `UiBatchKey`、排序后相邻合批、clip-state 去重与 scissor 栈；wgpu backend 已实现 dependency-layer draw plan、solid instance/image vertex arena、generation-keyed compiled-plan cache 和统计。本计划继续把这些已有实现收敛为稳定契约，补齐动态图集、stencil/复杂裁剪、backend 资源生命周期与局部重绘证据；运行时 wgpu 细节仍归渲染框架。

> 工程化硬目标(接 `index` §4.0):UI 上屏必须是**批次化、可裁剪、增量**的,不是每节点一个 draw call、每帧全量重画。

## 2. 现状(按代码核实)

- `batch/key.rs` 已把 clip、primitive、shader、resource、text backend、draw effects 与 opacity class 纳入 `UiBatchKey`；layer 明确不进 key。`batch/plan.rs` 按 `(z_index, paint_order, source index)` 排序，只合并同 layer 的相邻同 key 元素，并记录 range/source/split reason/stats。
- `batch/clip.rs` 已去重 clip state，并用 push/pop stack 对嵌套 scissor 求交；复杂 stencil 与后端 clip handle 生命周期仍需完成/验证。
- `zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/batching.rs` 已按 dependency depth 构建 solid/image/text draw ops，集中 solid vertices/instances 与 image vertices，并通过 `CompiledUiBatchPlanCache` 区分 generation cache build/hit。对应 stats initializer 的源码修复已可见，但 open failure 仍要求稳定 current-source managed gate 与来源回传。
- `10` §2.2 的脏区/重提取、动态图集、复杂裁剪和 backend 持久资源仍缺完整跨层验收；不能因 DTO/CPU plan 已存在而宣称 GPU 提交完成。

## 3. 设计

### 3.1 批次合并键(治"每节点一 draw call")

规范:相邻绘制命令在**批次键一致**且**裁剪状态一致**时合并为一个 `UiBatch`,对标 UE `FSlateRenderBatch::IsBatchableWith`(`SlateRenderBatch.h:145`,键含 `ShaderResource`/`ShaderType`/`ClippingState` 等 `:148/24/28/40`)、Bevy `UiBatch` 按 image 合批(`bevy_ui_render/lib.rs:1416`):

`UiBatchKey` = (shader/材质类型, 纹理/图集句柄, 裁剪状态句柄, 绘制基元类型, 混合/draw flags)。**裁剪状态必须进合并键**(裁剪不同不能并批)。layer/z **不进合并键**但决定排序与相邻性(对标 UE:layer 不在键中,但 `MergeRenderBatches` 按 layer 排序后合并相邻可并批者,`ElementBatcher.h:192`)。

### 3.2 layer / z 排序与批次边界

规范:每命令带 `layer_id`(z 序),提取按 `draw_order` + layer 排序;合批只在**排序后相邻且 key 一致**的命令间发生(保持视觉叠放正确),对标 UE `FSlateBatchData::MergeRenderBatches`(`ElementBatcher.h:192`)。上层(浮层/弹窗/焦点环)用更高 layer,自然压在内容之上,与 18 命中 z 序一致。

**layer_id 分配表(2026-07-02 评审收口)**:作者侧 V1 **不提供 z-index 词汇**;层级由"Overlay family + 提取序"决定,本表是 `layer_id` 的唯一权威分配(`13`/`18` 引用此表):

| 段位 | 内容 |
| --- | --- |
| 0 段 | 常规内容(dock 区、面板、控件) |
| 100 段 | dock 浮层(拖拽预览停靠指示等 dock chrome 浮层) |
| 200 段 | popup / menu |
| 300 段 | tooltip |
| 400 段 | 拖拽幽灵 / 焦点环 |
| 900 段 | 调试叠加 |

段内按**提取序递增**(后提取者压前者);跨段不因提取序穿越。

### 3.3 裁剪栈(scissor vs stencil)

规范:容器(`overflow:hidden/scroll`、圆角裁剪)push 裁剪区、子绘制完 pop,栈式管理,对标 UE `FSlateClippingManager` push/pop + `FSlateClippingState`/`FSlateClippingZone`(`Clipping.h:60`):

- **轴对齐矩形裁剪 → 硬件 scissor**(廉价);**旋转/非矩形/圆角 → stencil**(较贵)。规范默认轴对齐走 scissor,仅必要时 stencil(对标 UE 按 `bIsAxisAligned` 选 scissor/stencil)。
- 裁剪状态去重缓存,命令持裁剪句柄(对标 UE `FClipStateHandle`);裁剪句柄进批次键(§3.1)。
- 嵌套裁剪取交集(对标 UE `FSlateClippingZone::Intersect` `Clipping.h:128`)。
- **圆角廉价路径(2026-07-02 评审收口)**:低圆角 chrome 容器优先**矩形 scissor + 圆角仅作用于自身填充**(圆角画在填充画刷里,裁剪仍是轴对齐矩形);仅当内容**真正溢出圆角区**(子内容侵入圆角切角像素)时才升级 stencil。避免编辑器 chrome 大面积走 stencil。

### 3.4 纹理图集(字形 / 图标 / 9-slice)

规范:字形(接 17)、图标 SVG 栅格、小图共享**动态图集**,使同图集元素可合批,对标 UE `FSlateTextureAtlas`(grayscale/color/MSDF 字体 atlas)、Unity UI Toolkit **dynamic atlas**:

- 字形按 `font_size_logical × scale_factor` 栅格进图集(接 17 §3.2 / 16 DPI);atlas key 含 scale(治 17 G2 同源缺陷)。
- 同图集 + 同 shader + 同裁剪 → 单批(§3.1)。图集满则开新页(新纹理 = 新批边界)。
- **9-slice 原语规范(2026-07-02 评审收口)**:9-slice 命令按 UV 三段分片装配(横纵各三段共 9 区):**四角不缩放**(原尺寸贴),**四边单轴拉伸**(上下边横向拉伸、左右边纵向拉伸),**中心双轴拉伸**;边距(margin)以逻辑单位声明、顶点装配时乘 scale;9 区共享同一纹理页,合批不因 9-slice 断批。

**与 `zircon_runtime` render/14(2D 栈)的勾稽表(2026-07-02 评审收口)**:

| 职责 | 权威 |
| --- | --- |
| UI `UiBatchPlan` 顶点装配 / UI 批次(本文 §3.1-§3.5) | 本计划(21) |
| glyph quad → 场景 2D sprite 批(world-space 文本进场景 2D 管线) | `zircon_runtime/render/14-2d-stack.md` |
| 文本图集(字形栅格页)供给 | runtime `text/04` 共享服务(UI 批与场景 sprite 批共用) |

### 3.5 顶点 / 索引装配

规范:每命令展开为顶点(位置/UV/色/SDF 像素尺寸)+ 索引,装入共享顶点/索引缓冲,对标 UE `FSlateVertex`(`RenderingCommon.h`)。**像素吸附归这层或合成层,不归 Taffy**(Taffy 已 `disable_rounding`,接 13/16):顶点位置在装配时按 `scale_factor` 决定是否吸附整像素(文本/1px 边框吸附,自由内容不吸附),对标 UE `ESlateVertexRounding`。

**换算单点条款(2026-07-02 评审收口)**:全管线(SOURCE→STYLE→LAYOUT→COMMAND)一律逻辑坐标;逻辑→物理换算**单点发生在本节顶点装配阶段**(乘 `scale_factor` + 像素吸附),上游任何段不得预乘 scale。唯一例外:文本字形栅格按物理像素,由 runtime `text/04` `GlyphRasterKey { px_size_bucket }` 承担。与 `10` §3.1、`16` §3.4 同条款互为引用。

**组透明与多窗口条款(2026-07-02 评审收口,与 `10` §3.1 同款互引)**:透明度为逐命令 α 直乘(顶点色 alpha);子树组透明 V1 禁止(离屏合成登记 V2),弹层淡入淡出仅允许"整棵子树同一 α 且子元素不重叠"的受限形态。多窗口:批次计划以渲染根(窗口)为单位装配与提交,每窗口独立 `scale_factor` 与顶点缓冲,跨窗口不合批。

### 3.6 render-thread 提交与帧节奏

规范:提取/批次/顶点装配在逻辑侧产出**可序列化批次计划**(`UiBatchPlan`),提交在渲染侧消费(对标 UE game-thread batch → render-thread `DrawWindow_RenderThread` `SlateRHIRenderer.cpp`)。跨 ABI 边界传**批次计划 DTO + 顶点缓冲**,不传 wgpu 对象(遵 `runtime_interface` 边界:不传 wgpu/trait 对象)。wgpu pipeline/bind group/draw 调用归运行时渲染框架。

### 3.7 增量上屏(dirty region 局部重绘,接 09/10)

规范:仅脏视图/脏节点重提取重批,未脏区复用上帧批次/缓存,对标 Slint `PartialRenderer`/`DirtyRegion`(`partial_renderer.rs:215/378`)、UE `FSlateInvalidationRoot` 快/慢路径。脏集来自 09 `ViewDirtySet`;样式变更(20)/布局变更(13)/数据变更(11)各自标脏。可计算脏矩形并只重绘该区(scissor 限定),减少上屏带宽。

## 4. 接口与数据结构草案(Rust)

```rust
pub struct UiBatchKey { pub shader: UiShaderKind, pub texture: UiTextureHandle,
    pub clip: UiClipHandle, pub primitive: UiPrimitive, pub flags: UiDrawFlags } // 不含 layer
pub struct UiClipState { pub zone: UiClipZone, pub method: UiClipMethod /* Scissor|Stencil */ }
pub struct UiClipStack { /* push/pop, intersect 嵌套 */ }
pub struct UiVertex { pub pos: [f32;2], pub uv: [f32;4], pub color: u32, pub pixel_size: [u16;2] }
pub struct UiBatchPlan { pub batches: Vec<UiBatch>, pub vertices: Vec<UiVertex>, pub indices: Vec<u32> }

pub fn merge_batches(commands: &[UiRenderCommand], clips: &UiClipStack) -> UiBatchPlan; // 排序后相邻同键合并
pub fn dirty_region(dirty: &ViewDirtySet, arranged: &ArrangedTree) -> Vec<UiRect>;       // 局部重绘
```

## 5. 模块与文件落点

| 动作 | 文件 | 说明 |
| --- | --- | --- |
| 新增(契约) | `docs/ui-and-layout/gpu-submission-contract.md` | 批次键/裁剪栈/图集/顶点/layer/增量上屏 |
| DTO | `iface .../render/batch.rs` | `UiBatchKey` 合并语义 + 裁剪句柄 + 顶点 |
| 提取/批次 | `runtime .../render/extract.rs` + 新 batch owner | 排序合并 + 裁剪栈 + 图集合批 |
| 运行时 | `zircon_runtime/crates/zr_rhi(_wgpu)` | 消费批次计划做 wgpu 提交(实现细节,不在本契约) |

## 6. 里程碑切片化

| # | 切片 | 验证命令 |
| -- | --- | --- |
| S1 | 加固并受管验收现有批次键、layer 排序、相邻合批和 scissor clip stack；补 stencil 契约 | `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime_interface -SkipBuild -LibTests -TestFilter ui_batch` |
| S2 | 字形/图标动态图集合批(接 17 scale) + 复核现有顶点/instance arena + 像素吸附 | `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter ui_atlas_batch` |
| S3 | dirty region 局部重绘(接 09/10)，验证 generation cache 命中并复用未脏批次/持久 GPU 资源 | `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter ui_partial_render` |

## 7. 测试矩阵

- 相邻同键命令合并为一批;裁剪/纹理/shader 不同则断批。
- layer 高者排在上;合批不破坏叠放顺序。
- 轴对齐裁剪走 scissor、非矩形走 stencil;嵌套裁剪取交集。
- 同图集元素合一批;图集翻页断批。
- 文本/1px 边框顶点整像素吸附,自由内容不吸附(接 16)。
- 仅脏区重提取重批,未脏批次复用(接 09)。
- draw call 数随合批显著低于节点数(回归指标)。

## 8. 风险与对策

- 风险:裁剪句柄进批次键导致过度断批。对策:裁剪状态去重 + 轴对齐优先 scissor,减少独立裁剪态数量。
- 风险:增量重绘与批次缓存一致性。对策:脏集驱动失效 + 批次计划按视图分段缓存;先全量再分段。

## 9. 完成定义

批次合并键语义 + layer 排序 + 裁剪栈(scissor/stencil)+ 动态图集合批 + 顶点装配/像素吸附 + render-thread 提交边界 + dirty region 局部重绘成文;draw call 合批与增量上屏可回归。

## 10. 边界约束

extract→command 契约归 `10`;wgpu pipeline/bind group/draw 实现归运行时渲染框架(本计划只定批次/裁剪/图集/顶点/提交契约);字形度量/栅格归 17;DPI/缩放归 16;脏集归 09;computed style(画刷来源)归 20;ABI 边界不传 wgpu/trait 对象(遵 `runtime_interface`)。

## 11. 参考实现对照(dev/ 源码锚点,已核实)

- **Unreal(批次/裁剪/提交最直接对照)**:`SlateCore/.../Rendering/ElementBatcher.h:153`(FSlateBatchData)`:192`(MergeRenderBatches)`:245`(FSlateElementBatcher)`:257`(AddElements);`Rendering/SlateRenderBatch.h:145/148`(IsBatchableWith,键含 ShaderResource/ShaderType/ClippingState `:24/28/40`);`Layout/Clipping.h:60`(FSlateClippingZone)`:128`(Intersect);`Rendering/SlateRHIRenderer.cpp`(DrawWindow_RenderThread,game/render thread 拆分);`FSlateVertex`/`ESlateVertexRounding`(RenderingCommon.h);`FSlateTextureAtlas`(TextureAtlas.h)。
- **Unity UI Toolkit**(UIElements docs + 示例):UIR 渲染器、`MeshGenerationContext`/`Painter2D`(自定义网格,见 `radial-progress-vector-api`)、**dynamic atlas**(PanelSettings)、USS `overflow`/裁剪;`create-a-custom-swirl-filter` 示例(自定义渲染/材质)可作验证参照。
- **Bevy**:`bevy_ui_render/lib.rs:370-380`(ExtractedUiNodes)`:1416-1419`(UiBatch)`:1511-1905`(prepare_uinodes 合批);`CalculatedClip`(scissor)。
- **Slint**:`internal/core/item_rendering.rs`(渲染遍历/filter)、`software_renderer/partial_renderer.rs:215`(DirtyRegion)`:378`(compute_dirty_regions)——增量重绘对照。

## 12. 状态与产出记录

in_progress。interface batch key/ordered merge/clip stack，以及 wgpu dependency-layer draw plan、vertex/instance arena 与 generation cache 已有当前源码 owner；stencil/动态图集/backend 资源生命周期、局部重绘和稳定受管验收仍未完成，不据此宣称里程碑完成。

- applicable open failure（保持 open）：[batch-draw-plan-stats-initializer-drift](21/failure-2026-07-29-batch-draw-plan-stats-initializer-drift.md)。

- 2026-07-18 scene UI image性能交接：每可见image当前逐frame创建bind group、6-vertex GPU buffer并单draw，即使相同texture；stable draw list无generation命中。S1必须把image纳入ordered batch key(texture generation+clip/scissor+blend)，以static quad+instance arena和persistent binding handle提交；stable prepare/create/upload=0，见PERF-MVP-397及UI image静态证据。
- 2026-07-18 scene screen-space UI root交接：当前每frame从原始command重建七组plan Vec，并在`to_paint_elements`内逐command重建payload、serde hash和debug label；text line/advance/style再深clone。本轮只把decoration路径的重复paint投影2次降至1次。S1/S3必须让extract发布唯一generation-owned ordered plan、共享text/image handles与persistent geometry arena，stable generation projection/serde/hash/clone/plan rebuild/upload均为0；见PERF-MVP-398及UI render root静态证据。
- 2026-07-18 scene UI render子目录补充交接：17/17文件确认root/helper现共享一次paint projection，rich parse已从每inline降至每command，vertical advance与background blocker嵌套全扫已索引/线性化；但rich-run查找、inline前缀、七组Vec、CPU rect vertices与text/style clone仍逐frame。S1/S3的唯一prepared plan须同时提供dense rich-run range和共享line/report handles，见PERF-MVP-398及UI render子目录证据。
