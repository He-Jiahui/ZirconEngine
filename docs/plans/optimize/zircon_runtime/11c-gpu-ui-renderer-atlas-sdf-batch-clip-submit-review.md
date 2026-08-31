---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/ui
  - zircon_runtime/src/ui/icon_atlas
  - zircon_runtime/src/ui/surface/render
  - zircon_runtime_interface/src/ui/surface/render
  - zircon_runtime/crates/zr_rhi/src/ui_surface.rs
  - zircon_runtime/crates/zr_rhi/src/ui_surface
  - zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream
  - zircon_editor/src/ui/retained_host/host_contract/presenter/gpu
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/zircon_runtime/text/04-glyph-atlas-and-rasterization.md
  - docs/plans/zircon_runtime/text/05-sdf-msdf-pipeline.md
  - docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
  - docs/plans/zircon_editor/editor_layout/21-gpu-submission-and-draw-pipeline.md
  - docs/plans/performance/01/2026-07-18-graphics-ui-render-root-static-review.md
  - docs/plans/performance/01/2026-07-18-rhi-ui-surface-batching-static-review.md
  - docs/plans/performance/01/2026-07-18-rhi-wgpu-ui-rendering-static-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Rendering
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Rendering
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Layout/Clipping.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Textures
  - dev/bevy/crates/bevy_ui_render
  - dev/Fyrox/fyrox-ui/src/draw.rs
  - dev/Fyrox/fyrox-ui/src/brush.rs
  - dev/godot/servers/rendering
  - dev/godot/scene/main/canvas_item.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Textures/Texture2DAtlas.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Textures/PowerOfTwoTextureAtlas.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 11C · GPU UI Renderer、Atlas、SDF、Batch、Clip 与 Submit 工程化差距

## 1. 结论

Zircon 当前不是完全没有 GPU UI 基础。游戏/runtime scene 路径已经有真实 WGPU pipeline、普通形状与图片、Glyphon、native bitmap/color glyph atlas、SDF/MSDF/MTSDF、dirty page upload、稳定 slot 统计和 fallback report。Editor native window 路径也已经有基于重叠依赖的安全重排、solid instance arena、image vertex arena、generation-keyed compiled plan、bounded image cache、GPU timestamp、retained surface texture 和 damage scissor。这些实现中有相当一部分值得保留，尤其不能退回“每个节点即时创建 GPU 对象”的实现。

问题在于这些能力没有组成一个产品级 UI renderer，而是形成两套互不兼容的终端：游戏视口由 `ScreenSpaceUiRenderer` 消费 `UiRenderExtract`，Editor native window 由 `WgpuUiSurfaceRenderer` 消费另一套 `UiSurfaceDrawList`。两者不共享 ordered draw-op、brush/material、clip、font owner、icon atlas、image residency、cache generation、color space、device lifetime 或统计合同。`zircon_runtime_interface` 中更完整的 `UiBatchPlan`、`UiRenderCachePlan`、rounded/gradient/vector/material brush、stencil clip 和 draw effect 主要停留在 DTO、测试、debug visualizer；产品 scene renderer 仍从每个 legacy command 临时展开 paint element，再按形状、图片、三类文字、文字后装饰分桶提交。

2026-08-29 text-owner correction：screen-space product renderer 与 dynamic Runtime UI surface 现共享 Core
`TextModule` 提供的 `FontCollectionService`；动态 UI 在首次布局前完成 font asset admission，renderer plan
不再调用进程级 text layout service，raw batch 仅在 prepare 阶段按 collection revision fallback shape。
这只关闭了 font-owner/首帧时序的结构缺口；Editor native window、PIE 注入、统一 ordered draw-op 以及
受管 WGPU/PNG/power 对拍仍开放，不能把该切片表述为完整产品 renderer 收敛。

本轮确认三项 P0。第一，游戏/runtime UI 的 painter order 会被按资源类别重新分组：全部 solid 先画、全部 image 后画、文字又按 Glyphon -> bitmap atlas -> SDF 固定顺序画，最后统一画 caret/underline 等装饰；原本按 `(z_index, paint_order, node_id)` 排好的命令顺序因此失效。第二，runtime component 广泛发出的 `UiVisualAssetRef::Icon` 没有生产 renderer，实际被画成居中的实心矩形；已有 `UiIconAtlasBuilder` 只生成 CPU 计划，既不栅格化也不上传，且没有 production consumer。第三，Editor damage patch 在 retained texture 上使用 `Load` 和 premultiplied alpha 直接重放脏区，却没有先恢复背景或清除目标区域；重复半透明 patch 会累积变深，删除/移动内容可留下旧像素，而当前 GPU 测试只覆盖不暴露问题的 opaque patch。

性能方面也不能用“已有 batch/cache”笼统宣称完成。Editor RHI 的 dependency batching 是真实基础，但日常 Editor `present()` 故意生成无 producer generation 的 live full/damage draw list，现有 compiled-plan/text cache 因而不命中；generation 主要用于 native resize snapshot 等特殊路径。游戏 UI 则每帧重建七类 `Vec`、逐 command 构造 paint payload、以 JSON 序列化计算 generation、重建 CPU quad、对全部 vertex bytes 做 BLAKE3，再按 command/scissor 或 image 单独 draw。4K Editor retained path 还会在每次无效化后把整张约 31.64 MiB RGBA texture 复制到 swapchain，damage 只减少重画，不减少最终 copy 带宽。

本轮登记 3 项 P0、30 项 P1、8 项 P2。重构必须先建立统一、有序、可缓存的 GPU UI presentation artifact，并用像素测试关闭 painter order、icon 和 damage correctness；随后收敛 brush/clip/effect、font/icon/image atlas、颜色与 device generation；最后再做 bindless/indirect、复杂 clip、HDR 和更激进的性能优化。若没有相同视觉质量、相同分辨率和相同交互负载下的 CPU/GPU/带宽/VRAM 基线，不得宣称性能已经达到或超过 Unreal Slate。

## 2. 审查边界与证据

### 2.1 当前源码范围

| 集合 | 全部文件 / 物理行 | production 文件 / 物理行 | 本轮证据 |
|---|---:|---:|---|
| game `scene_renderer/ui` | 108 / 25,835 | 68 / 14,096 | E3：plan、geometry、image、Glyphon、bitmap atlas、SDF、upload、record 与 WGSL |
| runtime `ui/icon_atlas` | 3 / 357 | 3 / 357 | E3：SVG parse 与 atlas plan；无 production consumer |
| runtime `ui/surface/render` | 46 / 17,635 | 39 / 16,489 | E3：command extraction、cache/damage、widget paint producer |
| interface `ui/surface/render` | 35 / 5,857 | 34 / 5,697 | E3：paint、brush、batch、clip、cache、debug、parity、stats DTO |
| `zr_rhi` UI surface | 4 / 1,795 | 3 / 1,255 | E3：native draw-list、command、image resource、stats 与 presenter contract |
| `zr_rhi_wgpu` UI surface | 20 / 8,936 | 15 / 6,031 | E3：batch、geometry、pipeline、text、image cache、retained present 与 surface setup |
| combined focused set | 216 / 60,415 | 162 / 43,925 | E2-E3：495 个 test attributes、0 ignored；production fingerprint `b628534653d8f2b332019ad31b0379d89fda0c278735868630daafb0bf87ba05` |

production classifier 排除路径段 `tests` 与叶文件 `tests.rs`，但保留 WGSL；fingerprint 算法为路径排序、逐文件 SHA-256，再对 `path<TAB>hash<LF>` 清单取 SHA-256。本轮开始与成文前，这 162 个 production 文件均未出现在 `git status` 修改列表；跨域 Editor/test 文件存在其他 Session 修改，因此实施前仍需重取指纹并复核 producer。

owner chain 按两条产品路径分别追踪：

1. `UiTree`/arranged order -> `UiRenderExtract` -> `ScreenSpaceUiRenderer` -> shape/image/Glyphon/bitmap/SDF pipeline -> final color attachment。
2. Editor `ChromeCommandStream` -> `UiSurfaceDrawList` -> `WgpuUiSurfaceRenderer` -> dependency batch/text/image cache -> retained texture/damage patch -> native surface。

然后反查 interface 的 paint/batch/cache/clip contract、runtime icon atlas、Editor icon atlas，以及旧性能 handoff 和既有计划状态。只存在于测试、debug visualizer 或未被产品调用的类型，不计为产品能力。

### 2.2 参考源码边界

- Unreal Slate `ElementBatcher.cpp`/`SlateRenderBatch` 将 shader resource/type、primitive、draw effect、batch flag、clip handle、scene 等纳入 batch identity；merge 前按 layer 稳定排序，同 layer 保留原始顺序，只有 `IsBatchableWith` 才合并。`Clipping.h` 和 texture atlas owner 又把 scissor/stencil 与 page/resource 生命周期显式化。这里把 Unreal 当工程上限，不照搬其宏、线程或 RHI 类型。
- Bevy `bevy_ui_render` 从 extracted node 形成 `TransparentUi`/stack index phase，已有 image slicing、gradient、box shadow、material、clip 与 batch。它证明 Rust ECS/render-world 下可以保持 order 和 typed feature route，但不是复杂 UI 的全部上限。
- Godot `RendererCanvasCull`/canvas renderer 与 `CanvasItem` command 覆盖 material、clip/group、SDF、shadow 和 canvas ordering，用于判断 Zircon 是否具备完整 2D canvas contract。
- Fyrox `fyrox-ui/src/draw.rs` 与 `brush.rs` 提供 drawing context、command、clipping、texture/gradient 的较小 Rust baseline；若 Zircon 连这一能力面都未闭合，不能以“架构更复杂”掩盖产品缺口。
- 仓内 Unity Graphics 不含 UI Toolkit renderer 源码，本文不推断 Unity 闭源 UI 行为。只引用 `Texture2DAtlas.cs` 与 `PowerOfTwoTextureAtlas.cs` 中可直接证明的 allocation、update count/hash、mip、padding 和 blit/resource generation 设计。

### 2.3 明确未做

本轮没有运行 Cargo、Editor、WGPU、RenderDoc、真实窗口、HDR display 或 device-loss 注入；没有重新生成 framebuffer artifact，也没有做 Unreal/Slate 同场景 benchmark。495 个 test attribute 只是结构证据，不代替 GPU pixel、pass、bandwidth 和 native compositor 验收。本文可静态证明提交顺序、未接 consumer、cache key 条件和 load/blend 行为；视觉差异大小、具体 GPU 时间和驱动差异必须由后续动态 gate 给出。

## 3. 两套产品 renderer 的实际边界

| 维度 | 游戏/runtime scene UI | Editor native window UI |
|---|---|---|
| 输入 | `UiRenderExtract` / `UiRenderCommand` | `UiSurfaceDrawList` / `UiSurfaceCommand` |
| order | 输入已排序，backend 按 solid/image/text/decor 分桶后破坏 | dependency depth 只重排互不重叠项，基础正确 |
| shape | CPU rect/border triangles | solid instance，含 analytic rounded geometry |
| text | Zircon text + Glyphon + bitmap/SDF 三 route | 独立 Glyphon `FontSystem`，plain nowrap text |
| image | ResourceStreamer full-UV texture，逐 image draw | generation resource table、bounded local/shared cache、atlas UV |
| icon | `Icon` 变实心矩形 | process-global Editor icon atlas |
| clip | 单矩形 scissor | command/effective rect + damage scissor；无 stencil/mask |
| cache | atlas 有局部 cache；frame plan 每帧重建 | generation cache 存在；普通 live frame 无 generation |
| partial redraw | 无 retained UI presentation | retained texture + damage patch，但 patch 合成错误且全图 copy |
| color | CSS-like byte值直接写 sRGB attachment | 只选 non-sRGB UNORM，以 byte-space premultiplied blend |
| stats | 主要是 text prepare report | batch/image/text/pass/copy/GPU timing 较完整 |

正确的收敛方向不是让两套 renderer 永久“功能对齐”，也不是把 Editor command 强行塞进 scene UI，而是建立一个 backend-neutral、generation-owned、严格有序的 `UiGpuPresentation`：上层 producer 可不同，最终必须编译到同一 draw-op/clip/resource/color/device contract。游戏与 Editor 可以使用不同 surface integration，但不可继续拥有两套 painter、字体、图标和缓存真值。

## 4. 可保留的真实基础

### 4.1 Arranged tree 和 interface batch plan 已定义稳定 painter order

`ui/surface/arranged.rs` 会按 `(z_index, paint_order, node_id)` 排列节点；`UiBatchPlan::from_paint_elements` 也按 `(z_index, paint_order, source index)` 稳定地形成 ordered indices，只合并同 layer、相邻且 key 相等的元素。P0 不是 producer 完全无序，而是 game backend 没有消费这份 ordered plan。

### 4.2 Editor dependency batching 是安全重排基础

`zr_rhi_wgpu/ui_surface/batching.rs` 建立重叠依赖 depth，只在互不影响 painter correctness 的 item 间重排；solid 可走 compact instance，image/text 分别保留资源和 batch identity。其 generation cache、persistent buffer、cache hit/build counter 和 overlap/dependency stats 都有实际实现。后续应将这套算法提升为共享 compiler 的一个 backend 策略，而不是删除后重写。

### 4.3 Glyph atlas 与 SDF 不是单页全量占位实现

native bitmap/color glyph 路径有 page generation、face invalidation、staging failure/requeue、dirty upload 与 typed report；SDF 路径有 slot retention、relocation/eviction 统计、dirty page、page limit/oversized/generation failure 和 native fallback。问题是产品预算、统一 owner、ordered composition 和稳定 presentation 未闭合，不应把已有 atlas 工作归零。

### 4.4 Editor image residency 与统计已具备有界基础

per-presenter image cache 和 device-shared registry 都有 256 entries / 64 MiB admission、LRU eviction、generation identity、CPU/GPU resident byte 和 failure stats。per-presenter cache 持有 view/bind group/外部 texture `Arc`，不等于无条件复制第二份 GPU texture。后续要统一 budget owner 与 generation，而不是错误地将现状描述为“双份纹理必然常驻”。

### 4.5 Retained surface 和 damage producer 可以保留，但必须修正合成语义

Editor 已能从 dirty region 生成 damage stream，在 retained texture 上限定 scissor，并把 copy bytes/pass counts 暴露到 stats。这个结构可成为 partial-present 基础；当前 P0 是 patch 没有定义 replace/backdrop 语义，不是“完全没有局部重绘”。

## 5. P0：先关闭视觉错误和产品占位

### P0-1：game renderer 按资源类别重排，破坏 painter order

`render.rs` 遍历已经排序的 `extract.list.commands`，却把输出写入 `draws`、`images`、`auto_texts`、`native_texts`、`sdf_texts`、`post_text_draws` 七组数组。`record.rs` 固定先提交全部 `draws`，再提交全部 image，再提交全部 text，最后全部 post-text decoration；`text.rs` 内部又固定按 Glyphon native -> bitmap atlas -> SDF 顺序提交。backend 没有消费 `z_index`，也没有消费 interface 已生成的 `UiBatchPlan::ordered_element_indices`。

因此，一个较低 z 的 image 总会盖住所有较高 z solid；较低 z 的 SDF text 会盖住较高 z 的 Glyphon text；任意较低节点的 caret/underline 会在所有文字之后覆盖不相关的更高层内容。即使 node list 和命令 list 完全排序正确，这个分类 fanout 仍会制造稳定错误。

必须改为单一 ordered draw-op stream。每个 op 携带 stable painter token、pipeline/material key、resource generation、clip handle 和 geometry range；compiler 只能在证明不改变重叠关系时合批或重排。验收至少覆盖 solid/image/Glyphon/bitmap/SDF/selection/composition/caret 两两重叠矩阵，以及同 layer 相邻合批、非重叠安全重排和 popup/tooltip 跨层顺序。

### P0-2：runtime icon 没有 production renderer，语义图标被画成矩形

runtime button、tree row、status 等 producer 会发出 `UiVisualAssetRef::Icon`。game `render.rs` 只为 `UiVisualAssetRef::Image` 建 `ScreenSpaceUiImageBatch`；任何 `Icon`、无法解析的 image 或 `UiRenderCommandKind::Image` fallback 都会在目标 frame 中心画一个边长约 68%、最小 8 像素的前景色实心矩形。这不是 loading placeholder 的短暂状态，而是当前 icon 的稳定产品输出。

`ui/icon_atlas` 的 `UiIconAtlasBuilder` 只对请求去重、计算正方网格 slot 和 UV，并把 SVG path 字符串放进 plan。它没有 path tessellation/raster、texture page、upload、generation、eviction、sampling padding 或 renderer consumer；仓内引用除自身和测试外为零。Editor 同时另有完全独立的 RGBA icon atlas，不能替 runtime 路径完成产品能力。

必须建立 cooked icon/vector artifact 与 runtime GPU owner：内容 hash、semantic/theme variant、DPI bucket、page generation、UV/gutter、color/tint policy、pending/missing/error state、budget/eviction 和 device generation 必须贯通。真正缺失的 icon 可显示明确 engine-owned missing glyph，但正常 `Icon` 不得静默变矩形。验收需要对当前所有 semantic icon 做 asset-to-pixel inventory，并在 1.0/1.25/1.5/2.0 raster scale 下做清晰度和 atlas batching 验证。

### P0-3：Editor damage patch 在旧像素上重复 alpha blend，删除和半透明更新不正确

`presentation.rs` 的 `SurfaceRenderMode::DamagePatch` 对 retained texture 使用 `TargetLoad::Load`，只用 damage scissor 重放相交 draw-op。所有 UI pipeline 使用 premultiplied alpha blending；脏区开始前没有 clear、background restore、source replacement 或 isolated layer composition。若 patch 中的半透明内容重复提交，它会叠加在上一帧同一内容上；若节点被删除/移动而 patch 没有一个完全不透明背景覆盖旧区域，旧像素继续留在 retained texture。

当前 GPU patch test 先写 opaque seed，再用 opaque patch 覆盖，无法发现该错误。`UiSurfaceDrawList` 是通用 RHI contract，没有“每个 damage rect 必有 opaque root background”的前置条件，因此不能把正确性寄托在当前 Editor theme 恰好通常不透明。

damage contract 必须二选一并显式化：要么 producer 为每个 damage rect 提供完整 backdrop replay，renderer 先 clear/restore 到确定背景后按完整 painter stack 重画；要么把 patch 画入隔离 texture，再以 replace/composite 语义写回 retained cache。验收必须证明 full redraw 与 damage patch 对 opaque、50% alpha、删除、移动、圆角边缘、文字抗锯齿和连续 100 次相同 patch 逐像素一致。

## 6. P1：统一 presentation、brush、clip 与 effect

### P1-1：两套 GPU UI renderer 没有共同 authority

scene UI 和 native surface UI 各自拥有 command、pipeline、text、image、clip、cache 与 stats。任何新能力都要实现两次，且现在已经出现 painter、font、icon、color 和 cache 行为漂移。必须引入唯一的 `UiGpuPresentationCompiler` 与 backend-neutral artifact；surface adapter 只负责 target/acquire/present，不能再定义绘制语义。

### P1-2：interface 的 batch/cache/paint contract 没有进入产品提交

`UiBatchPlan`、`UiRenderCachePlan`、parity、visualizer 和 debug report 能在 CPU 测试中工作，但 game renderer 每 command 调用 `to_paint_elements(0)`，Editor RHI 又消费另一套 `UiSurfaceDrawList`。这些 DTO 目前是“设计表面”，不是产品 spine。必须硬切决定哪些字段成为 canonical presentation，删除或适配其余重复 contract。

### P1-3：brush 类型比 backend 能力宽得多

interface 声明 Solid、Image、Box、Border、Rounded、Gradient、Vector、Material 等 payload；game backend 实际只有 axis-aligned solid rect、四条 rect border、full-UV image 和 text，`corner_radius` 只在 paint DTO 中出现。nine-slice、tile、gradient、vector/material、analytic round 和 border join 均未进入 scene GPU path。

### P1-4：clip 只有轴对齐单矩形 scissor

game `clipped_scissor` 只对 frame、`clip_frame` 和 viewport 求矩形交集；interface `UiClipMode::Stencil` 只有 key/test 使用。没有 nested clip handle 生命周期、rounded clip、path mask、transform clip、stencil depth overflow 或 mask cache。应按 axis-aligned scissor -> analytic round/mask -> stencil/path 的成本阶梯实现，并把 clip identity 纳入 batch key。

### P1-5：draw effect 与 offscreen layer 没有产品 owner

`UiDrawEffect`、opacity group、shadow、blur/backdrop、blend mode、filter、subtree transform 和 material isolation 没有统一执行路径。逐 command alpha 不能正确表示有重叠子元素的 group opacity。需要 typed layer/effect graph、临时 target budget、composite order 和 cache invalidation，禁止把所有 effect 都继续压成 vertex color。

## 7. P1：game scene renderer 的热路径与资源模型

### P1-6：普通形状按 command 生成 CPU 三角形并逐 scissor draw

`push_rect` 每矩形生成 6 个 vertex，border 最多拆成 4 个 rect/24 vertex；`record.rs` 对每个 `ScreenSpaceUiDraw` 单独 `set_scissor_rect` + `draw`。没有 static quad + instance arena、index buffer、相邻 key merge、multi-draw 或 indirect route。先做 ordered instancing/batching，再评估 bindless/indirect；不能为追求 draw 数先破坏 painter correctness。

### P1-7：每帧重建七组计划和大量 owned payload

`prepare_screen_space_ui` 每帧新建形状、图片、三类文字和装饰数组；每 command 重新投影 paint element，text batch 拥有 `String`、glyph advance、shaped glyph、style 和 artifact handle 的多组数据。稳定 UI 仍承担遍历、分配、clone 和比较。extract 应发布 generation-owned immutable segments，dirty producer 只替换受影响 range。

### P1-8：cache generation 通过逐 command JSON 序列化计算

`UiRenderCommand::cache_generation()` 调用 `serde_json::to_writer` 把完整 command 写入 FNV writer。序列化失败时统一返回 `FNV_OFFSET`，不同失败 command 会碰成同一 generation；即使成功，每帧仍支付字段遍历、字符串和 serde 分派。generation 必须由 producer mutation/version 与依赖 content generation 组合产生，不应在 render prepare 热路径重新序列化对象。

### P1-9：GPU upload avoidance 仍需先重建并 hash 全部 vertex bytes

shape 和 image vertex buffer 都先生成完整 CPU payload，再对全部 bytes 做 BLAKE3 判断是否 `queue.write_buffer`。这可避免相同 payload 重传，却不能避免计划构建、几何生成和 O(N) hash。稳定 generation 应直接命中 persistent range；dirty segment 只更新局部 staging range，并统计 hashed/uploaded bytes。

### P1-10：image 是 full-UV、逐图片 draw 和逐纹理 binding 切换

scene image path固定 `[0,1]` UV，每个 image 独立 draw；同 texture 可复用 bind group，但不能把相邻 image 合成 instance batch。没有 atlas UV、nine-slice、tile、sampler/mip policy、density bucket 或 texture array/bindless table。应由 image handle 携带 view/sampler/subresource/generation/UV，而不是从 source 字符串临时解析 `ResourceId`。

### P1-11：image readiness 被 generic streamer fallback 吞平

prepare 通过 `resolve_ui_texture_id` 获取 fallback/placeholder，但 command 和 stats 不区分 ready、pending、missing、decode failed、evicted、generation stale。渲染必须有 typed readiness 和 last-good policy，frame stats 可按原因计数；Editor/diagnostics 才能定位“资源没到”而不是只看到一张通用图。

### P1-12：scene UI image 与普通形状缺统一 residency/budget

Editor RHI image cache有 256/64 MiB 局部和共享预算，game UI image binding cache只保留当前 prepare epoch出现的 binding，资源实际 residency 归 generic streamer，UI 没有 page/bind-group/vertex/cache budget 或 pressure policy。应接 09D 的 render asset residency owner，并把 UI priority、pin、eviction 和 fallback 记录进同一账本。

## 8. P1：glyph atlas、SDF 与文字 GPU route

### P1-13：SDF quality 和 page budget 是私有固定默认值

production `SdfAtlasQuality` 固定 64 px slot、最小 8x8 grid，即默认 512x512 page，cached slot 目标 256；page cap又来自 glyph atlas 默认常量。测试可传自定义 quality，产品没有 device capability、DPI、project quality、locale、VRAM pressure 或 platform profile入口。应形成显式 quality tier 和预算 owner，并记录 page/slot/eviction/fallback/quality downgrade。

### P1-14：SDF 内容变化会重新规划 atlas，稳定 slot 仍可能搬迁

内容变化后会从 cached slot key 列表构造新的 `GlyphAtlasSet`/allocator；代码也明确统计 retained-but-relocated slot。dirty page 和 partial upload基础存在，但 dynamic text churn 仍可能造成 slot relocation、vertex invalidation 和 page rebuild。应使用长期 page allocator、generational slot handle、空洞回收/compaction policy，并只在显式维护点搬迁。

### P1-15：三类 text backend 没有统一 ordered material route

同一 painter stream 被拆成 Glyphon native、bitmap atlas 和 SDF 三类，最终固定顺序提交。即使修复 P0，还需要统一 text draw-op：每个 glyph run 可选择 backend/material，但必须保留原 painter token，并让 fallback span 插回原位置。selection/background/caret 也应是同一 ordered layer 的 typed primitive。

### P1-16：game native text 每帧仍构造 transient buffer/area

Glyphon state可复用 atlas/cache，但 native route仍按 batch准备 transient text buffer、area和相关 vector；stable UI 缺 paragraph/glyph-run generation cache。11B 的 resolved glyph artifact 应成为 GPU compiler输入，native fallback只处理明确不能走 Zircon atlas的 span，不能再对整个稳定文本重复 shape。

### P1-17：Editor RHI text contract 退化为 plain、nowrap 字符串

`UiSurfaceCommandKind::Text` 只携字符串、family、weight、size、line height和极小 style；backend硬编码 `Wrap::None`。没有 direction、language、alignment、resolved glyph/layout artifact、font asset generation、rich run、selection/composition、SDF mode或text effect。Editor chrome因此绕开 11B 已有的 shaping/layout contract。

### P1-18：Editor RHI 创建独立系统字体 `FontSystem`

`WgpuUiTextRenderer::new` 直接 `FontSystem::new()`，拥有另一套 system-font discovery、Swash cache和Glyphon atlas，不消费 Zircon `TextRuntimeService`、project cooked font、fallback collection或font generation。相同字符串在 game UI 与Editor可能选择不同字体、宽度和像素。必须统一 font byte/face/glyph identity与device atlas owner。

### P1-19：Editor text draw-op 会碎片化 render pass

每个 text `DrawOp` 构造一个 `TextRenderer`；`record_draw_ops_to_view` 遇到 text 就单开 render pass，连续 non-text 才合并。solid/text/image交错越多，pass数越高。应让 ordered compiler形成可合并的连续 material run，Glyphon或自有glyph pipeline共享 pass/atlas，只有真实 attachment/effect boundary 才断 pass。

## 9. P1：Editor generation cache、retained present 与 icon atlas

### P1-20：compiled generation cache没有覆盖普通 live Editor frame

batch/text cache只有 `draw_list.generation()` 为 `Some` 才可命中；normal `ui_surface_draw_list_from_owned_stream` 明确传 `None`。`gpu_presenter_keeps_live_full_and_damage_streams_unversioned` 测试也锁定 live full/damage 无 generation，versioned path主要服务 native resize snapshot。因此普通 redraw仍重建 batch topology、geometry、text renderer和upload source。producer必须为每个 immutable chrome generation提供稳定 ID，damage只携 changed segment/generation delta。

### P1-21：retained texture每次更新后仍全图 copy到 surface

无论 full redraw 还是 damage patch，`record_copy_to_surface` 都按 retained/surface交集复制整张 texture。3840x2160 RGBA8为 33,177,600 bytes，即约 31.64 MiB/次，60 Hz约 1.85 GiB/s仅计算单向像素字节，尚不含渲染和协议开销。需要调查 direct render、partial copy capability、swapchain preserve限制和compositor策略；至少以 stats/gate证明 damage场景确实降低总带宽，而不只是降低draw数。

### P1-22：Editor icon atlas 是 process-global mutable singleton

`EDITOR_ICON_ATLAS: OnceLock<Mutex<EditorIconAtlas>>` 跨project、session、window和device共享可变slot/page，预算固定64页/64 MiB。没有session teardown、theme generation、device owner、project isolation或content provenance。应由共享只读icon artifact + device atlas service + session lease组成，不能让进程全局mutex成为真值。

### P1-23：Editor icon page在一次 discovery wave后立即 sealed

同一 `pack` 调用中的pending icon可填满新页，但调用结束即把changed page标成 `sealed=true`；后续frame发现的新icon即使旧页有空位也会跳过它，只能开新页或驱逐整页。动态插件、延迟打开panel和主题切换会按“发现批次”碎片化页面。应支持可追加page generation、dirty subrect upload和slot稳定性，或在明确prewarm/cook阶段冻结page。

### P1-24：icon、glyph、image atlas没有统一page/resource contract

runtime icon plan、Editor icon RGBA page、native glyph atlas、SDF atlas、scene image和RHI image registry各自定义key、generation、budget和upload。缺少统一的 page handle、format/color space、padding/mip、dirty rect、lease、device generation、residency stats和cook schema。可以保留不同allocator/format，但外部资源合同必须收敛。

## 10. P1：颜色、输出、设备生命周期与观测

### P1-25：game UI 的 CSS-like 颜色没有线性化却写入 sRGB attachment

`parse_color` 将 `#RRGGBB` byte直接除以255；shape shader原样返回该值，而 final color format为 `Rgba8UnormSrgb`。若颜色字符串语义与CSS/sRGB一致，`#808080` 被当作约0.502线性值，再由attachment编码成约0.737 sRGB，显示明显偏亮。必须定义authoring color space，进入blend前转换到linear；图片texture view、tint、premultiply和output transfer也要遵循同一合同。

### P1-26：Editor选择non-sRGB surface并在byte space做premultiplied blend

surface setup只接受 `Bgra8Unorm`/`Rgba8Unorm`，不选sRGB格式；pipeline使用premultiplied alpha。这样可以保持某些UI byte值“看起来接近”，但alpha blending发生在非线性byte space，没有明确的linear-light或legacy兼容模式，也拒绝sRGB-only/HDR surface。需建立可测试的working/output color contract，而不是依赖format选择暗含语义。

### P1-27：UI没有HDR、wide-gamut、ICC和terminal composition policy

game UI在09H2 terminal LDR之后直接写final target，Editor走独立SDR native surface。没有HDR paper white、PQ/scRGB、wide gamut、display profile、OS compositor alpha或截图/capture color metadata。至少应定义SDR UI如何合成到HDR scene、Editor HDR monitor如何选择surface以及像素测试使用哪个reference space。

### P1-28：GPU资源没有统一device generation和恢复协议

atlas texture、pipeline、bind group、retained texture、Glyphon state和image registry都由具体WGPU对象owner持有，但跨两套renderer没有统一device-lost/reset generation、recreate order、in-flight lease或last-good策略。surface `Lost/Outdated` reconfigure不能等价于device loss。应接09A的device generation/retirement协议，所有presentation handle必须可判stale并可重建。

### P1-29：game UI观测远弱于Editor RHI

game `ScreenSpaceUiRenderer`主要暴露text prepare report，缺少总command/paint element、ordered op、batch/draw/pass、pipeline switch、clip split、image ready/fallback、vertex/hash/upload bytes、atlas VRAM、cache hit、painter reorder和GPU time。Editor已有较丰富stats，应抽成共同schema，按surface/backend维度报告并接RenderFrameProfile。

### P1-30：`UiRenderExtractKind` 和产品capability没有实际选择作用

interface枚举 `LegacyCommandList`、`PaintElements`、`BatchedPaint`、`TextSelectionCursor`、`DebugOverlay`，但 `UiRenderExtract` 本身不携该kind，production也没有据此选择consumer。类似的brush/clip/cache DTO容易让文档误判产品已切换。应删除无效mode，或让canonical artifact以schema/capability version显式协商并由backend拒绝不支持特性。

## 11. P2：证据、工具与长期质量缺口

### P2-1：runtime SVG parser是临时字符串扫描器

parser用 `find("<svg")`、字符串attribute搜索，只收集 `<path d>`及fill/stroke字符串；不解析XML namespace、CSS/class、group/transform、clip、gradient、use、stroke语义，也没有path验证或raster结果。它当前无consumer，优先级低于P0接线；接线时必须替换为成熟、受预算约束的SVG解析/栅格库或cooked vector pipeline。

### P2-2：CPU parity/debug visualizer没有对拍真实GPU backend

interface visualizer能展示batch/cache计划，但game renderer不消费该计划，Editor又用另一套batch算法。需要把debug输出绑定到实际compiled presentation，并能从RenderDoc marker反查draw-op、source node、clip、resource generation和cache reason。

### P2-3：没有跨primitive painter-order像素矩阵

现有测试大量验证结构、计数和单pipeline framebuffer，未发现覆盖solid/image/三text route/decoration的全组合重叠。应生成小尺寸确定性golden和property-based overlap scene，任何batch优化都先通过顺序门禁。

### P2-4：没有真实damage等价性长序列测试

需要随机insert/remove/move/opacity/theme/text edit序列，每一步同时做full redraw和damage redraw并逐像素比较；统计damage area、replayed ops、copy bytes和retained VRAM。1000步无漂移才可接受partial redraw。

### P2-5：没有atlas长期churn/fragmentation基准

应覆盖CJK/emoji/variable font、动态plugin icon、DPI/theme切换、locale切换和VRAM pressure，记录page occupancy、relocation、dirty bytes、eviction、fallback、frame hitch和重建时间，而不只测一次分配成功。

### P2-6：没有UI cold/warm prewarm artifact

pipeline、font face、常用glyph、semantic icon和Editor chrome image缺统一prewarm manifest。应由cook产出常用resource set，启动按预算异步预热，并区分cold first-use和warm steady-state基线。

### P2-7：缺少同画质、多分辨率和多窗口性能基线

至少建立1080p/1440p/4K、100/1k/10k visible element、1/4 native window、1.0/2.0 DPI的CPU extract/compile、GPU pass、draw、upload、copy、VRAM和input-to-present数据；与Unreal Slate在等价字体、透明度、clip和image负载下比较。无等价画质数据不得用单一draw数宣传“更快”。

### P2-8：旧计划和failure记录存在局部完成误导

旧记录正确描述了dependency batching、generation cache和partial upload已落地，但没有强调live frame无generation、game painter分类重排、runtime icon占位和damage alpha错误。应保留历史记录并由本文重开边界，禁止删除failure来制造完成状态。

## 12. 与参考引擎的直接差异

| 工程能力 | 参考源码直接证明的基线 | Zircon current source | 必须收敛的差异 |
|---|---|---|---|
| painter/batch | Unreal stable layer order + `IsBatchableWith`；Bevy stack phase | game按primitive全局分桶；Editor有安全dependency batch | 单一ordered op与可证明安全的merge/reorder |
| clip | Unreal scissor/stencil clip handle；Godot canvas clip/group | 两路径主要是矩形scissor | nested handle、rounded/path/stencil、overflow与budget |
| brush/material | Bevy gradient/slice/shadow/material；Fyrox texture/gradient brush | interface有类型，game backend不消费 | canonical brush compiler、effect/layer与fallback |
| atlas | Unreal texture atlas；Unity Graphics有update hash/mip/padding | glyph有基础，icon/image多套割裂 | page/generation/dirty/mip/padding/lease统一contract |
| text | Unreal Slate font cache/HarfBuzz与renderer同owner | game三route；Editor另建system `FontSystem` | 11B artifact到GPU的唯一font/glyph owner |
| invalidation | Unreal invalidation/retained paint；Godot canvas update | Editor retained patch有错误且全图copy；game无presentation cache | full/damage像素等价、segment cache、带宽实测 |
| color/output | 参考renderer明确working/output pipeline | game sRGB输入未线性化；Editor byte-space UNORM | linear/premultiply/HDR/display contract |
| observability | Unreal/modernrenderers有batch/resource/pass stats与markers | Editor较好，game不足 | 统一stats、GPU timer、source marker、基线gate |

## 13. 目标架构

### 13.1 唯一 canonical artifact

建议硬切引入以下分层，名称可调整但责任不可再次拆散：

```text
UiTree / Editor Chrome producers
        -> UiPaintScene generation
        -> UiGpuPresentationCompiler
             ordered draw ops
             clip/effect graph
             immutable geometry/text/resource segments
             batch/dependency plan
        -> UiGpuResourceService(device generation)
             font/glyph/SDF/icon/image pages
             residency/budget/upload/prewarm
        -> Surface adapters
             scene target / native swapchain / retained target
        -> stats + capture + pixel gates
```

`UiPaintScene`必须只有一个painter token序列；任何backend分类只能成为material key，不能变成新的全局顺序。Editor和game可拥有不同producer schema adapter，但compiler之后必须共用clip、resource、color和generation语义。

### 13.2 Artifact最小字段

- presentation generation、producer generation、device generation、viewport/raster scale、working/output color space；
- ordered op：painter token、source node/diagnostic ID、geometry range、material/pipeline key、resource handle/generation、clip/effect handle；
- immutable segment generation与dirty ranges，支持insert/remove/move而不复制全表；
- resource readiness：ready/pending/missing/failed/evicted/stale，并带last-good/fallback policy；
- typed compile diagnostics：unsupported brush/clip/effect、budget downgrade、atlas allocation、shader/pipeline readiness；
- stats identity：extract/compile/batch/upload/record/present各阶段可对拍。

### 13.3 性能原则

1. painter correctness 是所有batch优化的前置门；只允许合并相邻同key，或依据精确overlap dependency证明可重排。
2. stable generation的extract/serde/hash/shape/tessellate/upload必须为0；只有visibility/projection变化时允许轻量replay。
3. 普通rect/image/glyph使用persistent static geometry + instance/indirect range；复杂vector/path按generation缓存tessellation。
4. atlas/resource由device service长期拥有，page/slot用generational handle；帧只持lease，不持跨device裸WGPU对象。
5. damage redraw必须与full redraw逐像素等价；若平台partial present无收益，允许按capability回退full present，但stats必须诚实。

## 14. 依赖顺序重构里程碑

### UI-GPU-M0：正确性止血

1. 建立跨primitive painter-order pixel matrix，先写出能稳定复现P0-1的失败测试。
2. 把game提交改为单一ordered draw-op，禁止七类全局分桶；text fallback span按原token插回。
3. runtime `Icon`接真实cooked/raster artifact与GPU page；临时missing图标必须显式可诊断。
4. 修复damage replace/backdrop语义，加入半透明、删除、移动和100次重复patch等价测试。

Gate：三项P0的current-source GPU测试在至少一个软件/CI adapter和一个真实WGPU adapter通过；artifact包含PNG、stats和source fingerprint。

### UI-GPU-M1：canonical presentation硬切

1. 定义`UiPaintScene`/`UiGpuPresentation` schema和generation；让game与Editor adapter都编译到同一ordered model。
2. 吸收`UiBatchPlan`有价值字段和Editor dependency batching，删除无consumer的`UiRenderExtractKind`与重复cache真值。
3. stable generation实现segment-level compile/cache；normal Editor live full/damage都必须携producer generation。
4. 统一shape/image/text op、clip handle和stats，不再允许surface backend重新解释painter语义。

Gate：稳定1k-element UI连续600帧，presentation rebuild、serde hash、text shape、geometry upload均为0；交互修改单节点只更新有界segment。

### UI-GPU-M2：brush、clip、effect与color

1. 实现rounded/border/gradient/image slice/vector/material的typed compiler与unsupported诊断。
2. 实现scissor、analytic rounded mask、stencil/path clip成本阶梯和嵌套clip handle。
3. 建立offscreen effect/layer、group opacity、shadow/filter和temporary target预算。
4. 定义sRGB authoring -> linear blend -> SDR/HDR output contract，两套surface使用相同pixel reference。

Gate：brush/clip/effect conformance scene与参考CPU raster/golden逐像素在明确容差内；HDR/SDR各有reference artifact。

### UI-GPU-M3：统一font/icon/image atlas与device lifetime

1. 让11B resolved glyph/font artifact成为game和Editor唯一字体输入，移除Editor独立system `FontSystem`真值。
2. 建立统一page/resource handle、generation、dirty rect、padding/mip、residency、budget和prewarm manifest。
3. 将runtime/Editor icon atlas收敛到device resource service；解决sealed-page碎片和动态theme/DPI/locale generation。
4. 接09A device-loss generation、in-flight lease和重建顺序。

Gate：device loss、theme/DPI/locale/font hot reload和VRAM pressure下无stale handle/错页；atlas occupancy/churn有长期报告。

### UI-GPU-M4：性能超过目标的证据

1. persistent instance/index/indirect arena，按adapter能力选择bindless/texture array/bind-group route。
2. 减少text pass split，调查direct-to-surface/partial copy并关闭4K全图copy热点。
3. 接RenderFrameProfile/GPU timestamp/RenderDoc marker，建立cold/warm、多窗口、多DPI基线。
4. 在相同画质、字体、clip、透明度和资源负载下与Unreal Slate对比CPU/GPU/带宽/VRAM/input-to-present。

Gate：先达到正确性和功能parity，再以P50/P95/P99及最坏帧证明目标平台性能；回归门禁使用确定性计数和宽松硬上限，时间型数据保留趋势与人工release gate。

## 15. 必须重开的既有计划

| 既有记录 | current source判定 | 重开内容 |
|---|---|---|
| Text04 glyph atlas | 局部基础真实，产品未闭合 | 统一device owner、Editor/game共享font/glyph identity、budget、device loss、ordered submission |
| Text05 SDF/MSDF | 算法/dirty page/fallback基础真实 | 产品quality tier、长期allocator、route order、跨DPI/HDR质量和性能gate |
| Text09 cache/performance | stable cache只覆盖局部 | generation-ownedpresentation、segment invalidation、stable-frame zero-work与多window pressure |
| Render17 performance | 已登记UI逐帧重建风险 | 把game/Editor共同stats、GPU timer、copy bandwidth和同画质benchmark纳入PF里程碑 |
| EditorLayout21 GPU pipeline | `in_progress`判断正确 | 明确normal live frame无generation，补P0顺序/damage/icon/color与canonical artifact硬切 |
| 2026-07-18 UI performance reviews | 证据大体仍有效 | 保留历史，不把后续局部cache/batching落地扩大成产品完成 |
| UI surface per-present failure | generation cache已部分落地 | 只关闭versioned snapshot子项；live full/damage和text pass仍保持open |
| pairwise overlap batching failure | dependency-depth实现可保留 | 关闭算法子项前补GPU painter matrix与规模/最坏case动态验证 |

## 16. 验收矩阵

| Gate | 必测场景 | 必须记录 |
|---|---|---|
| painter | 7类primitive两两重叠、同层/跨层、popup/tooltip | pixel hash、ordered op、batch/reorder reason |
| icon | 全semantic inventory、theme、1.0-2.0 DPI、missing/pending | asset/page generation、fallback reason、清晰度artifact |
| damage | alpha、删除、移动、AA文字、圆角、100/1000步序列 | full/damage diff、damage area、replayed ops、copy bytes |
| stable cache | 100/1k/10k元素静止600帧 | rebuild/hash/shape/upload=0、resident bytes稳定 |
| churn | CJK/emoji、plugin icon、locale/theme/DPI切换 | occupancy、relocation、eviction、dirty upload、P99 hitch |
| clip/effect | nested scissor/round/path、group opacity、shadow/blur | pass/target/stencil数、pixel diff、temporary VRAM |
| color | SDR sRGB、HDR、透明叠加、截图/capture | reference-space像素、surface format、metadata |
| lifetime | resize、multiwindow、surface lost、device lost、hot reload | generation transition、stale reject、rebuild time、leak |
| performance | 1080p/1440p/4K，1/4 window，1.0/2.0 DPI | CPU阶段、GPU pass、draw/pass、upload/copy、VRAM、latency |

## 17. 完成定义

11C只能在以下条件同时满足后从`implementation_status: pending`转为完成：

1. game与Editor产品路径消费同一canonical ordered presentation，旧的七类全局分桶和重复绘制真值已硬切移除；
2. runtime icon不再以矩形占位，所有正常semantic icon有cooked artifact、generation、atlas/residency和typed failure；
3. full redraw与damage patch在规定序列逐像素等价，半透明和删除场景通过；
4. rounded/gradient/image slice/vector/material、scissor/rounded/stencil clip与group effect有明确支持矩阵和unsupported诊断；
5. game与Editor共享font/glyph/icon/image resource contract、device generation和统一stats；
6. stable generation达到zero rebuild/shape/hash/upload，4K damage path的copy带宽有可接受证据；
7. SDR/HDR颜色合同、device loss、multiwindow、DPI/theme/locale hot reload和VRAM pressure通过动态gate；
8. 同画质Unreal Slate对照记录CPU/GPU/带宽/VRAM和P50/P95/P99，结论由数据支持。

本轮只完成current-source静态审查和重构规划，没有修改生产代码，也没有生成GPU动态验收artifact。当前状态必须保持`review_complete / implementation pending / source_recheck_required`。

2026-08-28 current-source correction：SDF atlas retained frame assembly 已拥有 iterator-based glyph-key
collector，但 cache discard 与 standalone plan 曾继续调用变为 `cfg(test)` 的旧 slice wrapper，导致 default
Runtime production compile missing symbol。两个入口现直接传 `texts.iter()` 到唯一 iterator owner；未恢复
production facade，未创建 flattened text-batch vector，也未改变 key、slot、page、eviction 或 draw-order
算法。该修正只关闭 cutover compile path，不能改变本报告的 GPU 产品结论；managed Runtime、WGPU、
RenderDoc、功耗与 PNG 仍开放。状态：
`sdf_atlas_iterator_owner_cutover_static / managed_gpu_product_validation_pending`。

2026-08-29 text-owner lifecycle correction：screen-space renderer now holds a non-Clone
`RuntimeFontAssetClaimScope` from Text Core, reconciles dependency membership before refreshing its explicit
collection, and removes released identities from its local ready/missing/error admission cache. Text Core batches
last-scope owner retirement into one database publication, preserving shared HUD/menu owners. This is a static
ownership and retry correction only; it does not implement GPU atlas residency, upload, device-loss recovery,
batch/clip submission, or any timing/power claim. Managed Runtime/WGPU/PNG, profile and Unreal same-load evidence
remain open. Changed/new font admissions are coalesced with release into one collection publication; GPU and
managed evidence remain open. The
report stays `review_complete / implementation pending / source_recheck_required`.
