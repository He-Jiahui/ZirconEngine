---
related_code:
  - zircon_runtime_interface/src/ui/surface/render/command.rs
  - zircon_runtime_interface/src/ui/surface/render/batch.rs
  - zircon_runtime/src/ui/surface/render/extract.rs
  - zircon_runtime/src/rhi
  - zircon_runtime/src/rhi_wgpu
plan_sources:
  - docs/plans/zircon_editor/editor_layout/10-real-rendering-pipeline-and-contract.md
  - docs/plans/zircon_editor/editor_layout/09-incremental-message-bus-and-refresh.md
  - docs/plans/zircon_editor/editor_layout/16-relative-layout-and-resolution-adaptation.md
  - docs/plans/zircon_editor/editor_layout/17-text-rendering-and-typography.md
status: planned
---
# 21 GPU 提交与绘制管线(批次合并 / 裁剪栈 / 图集 / 顶点 / 上屏)

## 1. 目标

把"绘制命令如何变成最少的 GPU draw call 上屏"沉淀为一份**GPU 提交规范**。`10` 已定 extract→command→**batch**→上屏的契约,但把底层批次合并、裁剪栈、图集、顶点装配、render-thread 提交**整体推给运行时渲染框架,未细化**(`10` §1 明确不重写 wgpu)。这是"GPU 提交绘制不成熟"的根因:`UiBatch`/`UiBatchKey`/`UiBatchPlan` DTO 存在,但**批次键的合并语义、裁剪栈(scissor vs stencil)、动态图集、layer/z 排序、render-thread 拆分**无权威规范。本计划补这层,深化 `10` 的 batch→上屏段,运行时 wgpu 实现细节仍归渲染框架。

> 工程化硬目标(接 `index` §4.0):UI 上屏必须是**批次化、可裁剪、增量**的,不是每节点一个 draw call、每帧全量重画。

## 2. 现状(按代码核实)

- `iface .../render/batch.rs` 有 `UiBatch`/`UiBatchPlan`/`UiBatchKey`;`render/command.rs` 有 `UiRenderCommand`(矩形/边框/文本/图像/裁剪);`extract.rs` 产 `UiRenderExtract`(`draw_order`)。
- 缺:`UiBatchKey` **合并语义**(哪些命令可并入同批)、**裁剪栈**(push/pop、scissor vs stencil 选择)、**图集**(字形/图标 atlas、合批的纹理一致性约束)、**layer/z 与批次边界**、**render-thread 提交拆分**、**顶点/索引装配规范**。
- `10` §2.2 已点"脏区→重提取增量"缺口;本计划补"批次/上屏侧的增量(dirty region 局部重绘)"。

## 3. 设计

### 3.1 批次合并键(治"每节点一 draw call")

规范:相邻绘制命令在**批次键一致**且**裁剪状态一致**时合并为一个 `UiBatch`,对标 UE `FSlateRenderBatch::IsBatchableWith`(`SlateRenderBatch.h:145`,键含 `ShaderResource`/`ShaderType`/`ClippingState` 等 `:148/24/28/40`)、Bevy `UiBatch` 按 image 合批(`bevy_ui_render/lib.rs:1416`):

`UiBatchKey` = (shader/材质类型, 纹理/图集句柄, 裁剪状态句柄, 绘制基元类型, 混合/draw flags)。**裁剪状态必须进合并键**(裁剪不同不能并批)。layer/z **不进合并键**但决定排序与相邻性(对标 UE:layer 不在键中,但 `MergeRenderBatches` 按 layer 排序后合并相邻可并批者,`ElementBatcher.h:192`)。

### 3.2 layer / z 排序与批次边界

规范:每命令带 `layer_id`(z 序),提取按 `draw_order` + layer 排序;合批只在**排序后相邻且 key 一致**的命令间发生(保持视觉叠放正确),对标 UE `FSlateBatchData::MergeRenderBatches`(`ElementBatcher.h:192`)。上层(浮层/弹窗/焦点环)用更高 layer,自然压在内容之上,与 18 命中 z 序一致。

### 3.3 裁剪栈(scissor vs stencil)

规范:容器(`overflow:hidden/scroll`、圆角裁剪)push 裁剪区、子绘制完 pop,栈式管理,对标 UE `FSlateClippingManager` push/pop + `FSlateClippingState`/`FSlateClippingZone`(`Clipping.h:60`):

- **轴对齐矩形裁剪 → 硬件 scissor**(廉价);**旋转/非矩形/圆角 → stencil**(较贵)。规范默认轴对齐走 scissor,仅必要时 stencil(对标 UE 按 `bIsAxisAligned` 选 scissor/stencil)。
- 裁剪状态去重缓存,命令持裁剪句柄(对标 UE `FClipStateHandle`);裁剪句柄进批次键(§3.1)。
- 嵌套裁剪取交集(对标 UE `FSlateClippingZone::Intersect` `Clipping.h:128`)。

### 3.4 纹理图集(字形 / 图标 / 9-slice)

规范:字形(接 17)、图标 SVG 栅格、小图共享**动态图集**,使同图集元素可合批,对标 UE `FSlateTextureAtlas`(grayscale/color/MSDF 字体 atlas)、Unity UI Toolkit **dynamic atlas**:

- 字形按 `font_size_logical × scale_factor` 栅格进图集(接 17 §3.2 / 16 DPI);atlas key 含 scale(治 17 G2 同源缺陷)。
- 同图集 + 同 shader + 同裁剪 → 单批(§3.1)。图集满则开新页(新纹理 = 新批边界)。

### 3.5 顶点 / 索引装配

规范:每命令展开为顶点(位置/UV/色/SDF 像素尺寸)+ 索引,装入共享顶点/索引缓冲,对标 UE `FSlateVertex`(`RenderingCommon.h`)。**像素吸附归这层或合成层,不归 Taffy**(Taffy 已 `disable_rounding`,接 13/16):顶点位置在装配时按 `scale_factor` 决定是否吸附整像素(文本/1px 边框吸附,自由内容不吸附),对标 UE `ESlateVertexRounding`。

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
| 运行时 | `zircon_runtime/src/rhi(_wgpu)` | 消费批次计划做 wgpu 提交(实现细节,不在本契约) |

## 6. 里程碑切片化

| # | 切片 | 验证命令 |
| -- | --- | --- |
| S1 | 批次键合并语义 + layer 排序 + 裁剪栈(scissor/stencil)契约 | `cargo test -p zircon_runtime_interface --lib ui_batch --locked` |
| S2 | 字形/图标动态图集合批(接 17 scale) + 顶点装配 + 像素吸附 | `cargo test -p zircon_runtime --lib ui_atlas_batch --locked` |
| S3 | dirty region 局部重绘(接 09/10),复用未脏批次 | `cargo test -p zircon_runtime --lib ui_partial_render --locked` |

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

planned。后续项:S1 批次键合并语义 + layer 排序 + 裁剪栈契约。
