---
related_code:
  - zircon_runtime_interface/src/ui/surface/render/command_kind.rs
  - zircon_runtime_interface/src/ui/surface/render/command.rs
  - zircon_runtime_interface/src/ui/surface/render/list.rs
  - zircon_runtime_interface/src/ui/surface/render/paint.rs
  - zircon_runtime_interface/src/ui/surface/render/brush.rs
  - zircon_runtime_interface/src/ui/surface/render/batch.rs
  - zircon_runtime_interface/src/ui/surface/render/cache.rs
  - zircon_runtime_interface/src/ui/surface/render/debug.rs
  - zircon_runtime_interface/src/ui/surface/render/parity.rs
  - zircon_runtime_interface/src/ui/surface/render/visualizer.rs
  - zircon_runtime_interface/src/ui/surface/render/text_shape.rs
  - zircon_runtime_interface/src/ui/surface/render/extract.rs
  - zircon_runtime_interface/src/ui/surface/render/resolved_style.rs
  - zircon_runtime_interface/src/ui/surface/render/text_layout.rs
  - zircon_runtime_interface/src/ui/surface/render/editable_text.rs
  - zircon_runtime_interface/src/ui/surface/render/typography.rs
  - zircon_runtime_interface/src/ui/surface/render/visual_asset_ref.rs
  - zircon_runtime/src/ui/surface/render/extract.rs
  - zircon_runtime/src/ui/surface/render/resolve.rs
  - zircon_runtime/src/ui/text/mod.rs
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_runtime/src/ui/text/rich_text.rs
  - zircon_runtime/src/ui/text/edit_state.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/screen_space_ui_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_font_bake.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/shaders/sdf_text.wgsl
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/render_commands.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/primitives.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/visual_assets.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/text.rs
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Rendering/DrawElementTypes.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Rendering/DrawElements.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Rendering/DrawElementPayloads.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Rendering/ElementBatcher.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Rendering/ElementBatcher.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Rendering/SlateRenderBatch.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Rendering/SlateRenderBatch.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Styling/SlateBrush.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Styling/SlateBrush.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateRHIRenderer/Private/SlateRHIRenderer.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateRHIRenderer/Private/SlateRHIRenderingPolicy.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/SlateFontRenderer.cpp
implementation_files:
  - zircon_runtime_interface/src/ui/surface/render/command_kind.rs
  - zircon_runtime_interface/src/ui/surface/render/command.rs
  - zircon_runtime_interface/src/ui/surface/render/list.rs
  - zircon_runtime_interface/src/ui/surface/render/paint.rs
  - zircon_runtime_interface/src/ui/surface/render/brush.rs
  - zircon_runtime_interface/src/ui/surface/render/batch.rs
  - zircon_runtime_interface/src/ui/surface/render/cache.rs
  - zircon_runtime_interface/src/ui/surface/render/debug.rs
  - zircon_runtime_interface/src/ui/surface/render/parity.rs
  - zircon_runtime_interface/src/ui/surface/render/visualizer.rs
  - zircon_runtime_interface/src/ui/surface/render/text_shape.rs
  - zircon_runtime_interface/src/ui/surface/render/extract.rs
  - zircon_runtime_interface/src/ui/surface/render/resolved_style.rs
  - zircon_runtime_interface/src/ui/surface/render/text_layout.rs
  - zircon_runtime_interface/src/ui/surface/render/editable_text.rs
  - zircon_runtime_interface/src/ui/surface/render/typography.rs
  - zircon_runtime_interface/src/ui/surface/render/visual_asset_ref.rs
  - zircon_runtime/src/ui/surface/render/extract.rs
  - zircon_runtime/src/ui/surface/render/resolve.rs
  - zircon_runtime/src/ui/text/mod.rs
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_runtime/src/ui/text/rich_text.rs
  - zircon_runtime/src/ui/text/edit_state.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/screen_space_ui_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_font_bake.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/shaders/sdf_text.wgsl
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/render_commands.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/primitives.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/visual_assets.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/text.rs
plan_sources:
  - user: 2026-05-06 完善渲染方面内容，参照 dev 下虚幻源码
  - user: 2026-05-06 Zircon UI 与 Unreal Slate 差异审计及后续里程碑
  - docs/ui-and-layout/zircon-ui-unreal-slate-layout-gap-audit.md
  - docs/ui-and-layout/slate-style-ui-surface-frame.md
  - docs/assets-and-rendering/runtime-ui-graphics-integration.md
  - .codex/plans/Material UI + .ui.toml 全链路 UI 系统推进计划.md
  - .codex/plans/UI SDF 字体真实 Bake 收束计划.md
tests:
  - cargo check -p zircon_runtime_interface
  - cargo check -p zircon_runtime_interface --tests
  - cargo test -p zircon_runtime_interface --lib render_contracts --locked --jobs 1 --message-format short --color never -- --nocapture
  - rustfmt --edition 2021 --check "zircon_runtime_interface/src/ui/surface/render/debug.rs" "zircon_runtime_interface/src/ui/surface/render/parity.rs" "zircon_runtime_interface/src/ui/surface/render/visualizer.rs" "zircon_runtime_interface/src/ui/surface/render/mod.rs" "zircon_runtime_interface/src/ui/surface/mod.rs" "zircon_runtime_interface/src/tests/render_contracts.rs"
  - rustfmt --edition 2021 --check "zircon_runtime_interface/src/ui/layout/slot.rs" "zircon_runtime_interface/src/ui/layout/linear_sizing.rs" "zircon_runtime_interface/src/ui/surface/render/debug.rs" "zircon_runtime_interface/src/ui/surface/render/parity.rs" "zircon_runtime_interface/src/ui/surface/render/mod.rs" "zircon_runtime_interface/src/ui/surface/mod.rs" "zircon_runtime_interface/src/tests/render_contracts.rs" (R8 final closeout: passed with no output after scoped target clean)
  - CARGO_TARGET_DIR=E:\zircon-build\targets\ui-render-r4-interface cargo check -p zircon_runtime_interface --tests --locked --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=E:\zircon-build\targets\ui-render-r4-interface cargo test -p zircon_runtime_interface --lib render_contracts --locked --jobs 1 --message-format short --color never -- --nocapture (R7 review fixes: 19 passed, 0 failed)
  - CARGO_TARGET_DIR=E:\zircon-build\targets\ui-render-r4-interface cargo check -p zircon_runtime_interface --tests --locked --jobs 1 --message-format short --color never (R8 shared parity seam: passed)
  - CARGO_TARGET_DIR=E:\zircon-build\targets\ui-render-r4-interface cargo test -p zircon_runtime_interface --lib render_contracts --locked --jobs 1 --message-format short --color never -- --nocapture (R8 shared parity seam: 21 passed, 0 failed, 31 filtered out)
  - cargo check -p zircon_runtime_interface --tests --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-r8-closeout --message-format short --color never (R8 shared parity seam: passed)
  - cargo test -p zircon_runtime_interface --lib render_contracts --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-r8-closeout --message-format short --color never -- --nocapture (R8 shared parity seam: 21 passed, 0 failed, 31 filtered out)
  - rustfmt --edition 2021 --check "zircon_runtime_interface/src/tests/render_contracts.rs" (R9 interface parity harness: passed with no output)
  - cargo check -p zircon_runtime_interface --tests --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-r9-offscreen-parity --message-format short --color never (R9 interface parity harness: passed)
  - cargo test -p zircon_runtime_interface --lib render_contracts --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-r9-offscreen-parity --message-format short --color never -- --nocapture (R9 interface parity harness: 23 passed, 0 failed, 32 filtered out)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never (passed with existing warnings)
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never (attempted; timed out during dependency compilation and background cargo/rustc continued)
  - cargo test -p zircon_editor --lib rust_owned_host_painter_draws_runtime_render_commands --locked --jobs 1 --message-format short --color never -- --nocapture (attempted; timed out during dependency compilation)
doc_type: milestone-detail
---

# Zircon UI 与 Unreal Slate 渲染差异审计

## Scope

本文补齐 `Zircon UI 与 Unreal Slate 差异审计及后续里程碑` 中“渲染”部分的细化内容。重点不是把 Unreal `FSlateDrawElement` API 逐字搬进 Zircon，而是把 Slate 的 paint element、brush payload、batch key、cached element、RHI submit 和 debug visualizer 责任拆成 Zircon 可实现的 `.ui.toml -> UiSurfaceFrame -> UiRenderExtract -> renderer/editor painter` 合同。

当前 Zircon 渲染链路已经有 retained UI 的起点：`UiRenderCommandKind::{Group, Quad, Text, Image}`、`UiRenderCommand`、`UiRenderExtract`、runtime screen-space WGPU pass、glyphon/SDF 文本路径、editor host painter 对同一 render command 的投影，以及 render framework 对 runtime UI extract 的提交入口。与 Slate 相比，缺口集中在命令语义太窄、brush/material payload 不足、批处理键不可观测、cached element/invalidation 还未和渲染缓存闭环、debug visualizer 只停在初级统计、文本测量/绘制仍未共享 shaping truth。

## Unreal Slate 渲染源码基线

| Slate 源码 | 渲染职责 | 对 Zircon 的约束 |
|---|---|---|
| `DrawElementTypes.h` | `FSlateDrawElement` 是 Slate paint 输出的基本单元，覆盖 `MakeBox`、`MakeText`、`MakeShapedText`、`MakeGradient`、`MakeLines`、`MakeViewport`、`MakeCustomVerts`、post-process blur 等，并在 element 上保存 layer、position、local size、scale、render transform、clip handle、draw effects、batch flags 和 cached 标记。 | `UiRenderCommand` 不能长期只靠 `Group/Quad/Text/Image` 表达所有视觉；需要拆出几何、paint effect、clip handle、resource/material key、primitive kind、debug/cached 标记和可扩展 command payload。 |
| `DrawElementPayloads.h` / `DrawElementTypes.h` element structs | Box/RoundedBox/Text/ShapedText/Gradient/Spline/Line/CustomVerts 等 payload 保存 brush margin、UV、tiling、mirroring、draw type、font info、shaped glyph sequence、outline、overflow、points、thickness 和 instance data。 | Zircon 需要 `UiBrush` / `UiPaintPayload` 分层，而不是把所有样式塞进 `UiResolvedStyle`；圆角、9-slice、border、gradient、line/vector、自定义顶点、viewport texture 和文本 shaping 都需要类型化 payload。 |
| `SlateBrush.h` / `SlateBrush.cpp` | `FSlateBrush` 把 draw type 分为 `NoDrawType`、`Box`、`Border`、`Image`、`RoundedBox`，并持有 tiling、mirroring、image type、rounding、outline、margin、UV region、tint 和 rendering resource handle。 | Zircon 的 `.ui.toml` style/resource 需要区分 solid/color brush、image brush、box/9-slice brush、border brush、rounded box brush、vector/SVG brush 和 material brush，并把资源解析结果作为批处理键的一部分。 |
| `ElementBatcher.h` / `ElementBatcher.cpp` | `FSlateElementBatch` 用 shader resource 作为主 key，用 shader params、draw flags、shader type、primitive type、draw effects、clip state、custom drawer、instance data、scene index 作为 batch key；`FSlateBatchData::MergeRenderBatches()` 稳定按 layer 排序并合并可批处理 batch，同时导出 final vertex/index buffers、layer count、batch count、stencil/virtual texture feedback 需求。 | Zircon screen-space UI 现在逐 command 规划 quad vertices，缺显式 `UiBatchKey`、layer-stable merge、final vertex/index buffer 统计、scissor/stencil 需求、texture/material feedback 和 debug dump。 |
| `SlateRenderBatch.h` | `FSlateRenderBatch` 保存 layer、shader params、resource、primitive、shader type、draw effects、draw flags、scene index、clip state、vertex/index offsets/count、instance data、custom drawer 和 `IsBatchableWith` 规则。 | Zircon 需要把 batchable 条件变成可测试合同：相同 layer、clip、shader/material、resource、blend/effect、primitive、text backend 和 atlas page 才能合并；不同条件必须保留分批原因。 |
| `DrawElements.h` | `FSlateCachedElementList` / `FSlateCachedElementData` 把 widget-owned draw elements、cached clip states、cached render batches 和 invalidated lists 绑定到 invalidation root；未失效时可复用 cached batches。 | Zircon dirty flags 不能只触发整帧 rebuild；需要把 `UiSurfaceFrame` 的 arranged nodes、render commands、batch plan、clip states 和 invalidation reason 连起来，支持局部 recache 与 debug 解释。 |
| `SlateRHIRenderer.cpp` / `SlateRHIRenderingPolicy.h` | RHI renderer 创建 projection matrix、viewport/backbuffer/HDR composite inputs、RDG elements buffers，并把 `FSlateBatchData` 转成 draw elements pass；同时提供 `Slate.ShowOverdraw`、`Slate.ShowBatching`、`Slate.ShowWireFrame`、GPU drawcall stat、HDR UI luminance/composite 等调试/平台配置。 | Zircon runtime UI pass 需要 drawcall/batch/overdraw/wireframe/scissor/material stats，HDR/linear/sRGB/viewport scale 策略，以及在 render framework stats 中暴露 UI pass 成本。 |
| `SlateFontRenderer.cpp` / `ElementBatcher.cpp` text path | Slate 通过 font cache、FreeType flags、font anti-aliasing、outline method、`FSlateTextShaper`、SDF text debug visual 和 shaped glyph payload 连接测量、shaping 与绘制。 | Zircon 已有 glyphon/SDF 绘制能力，但 shared text measure 仍需使用同一 font/shaping source；text command 需要携带 shaped lines/glyph runs/cache key，而不是 editor painter 继续估算字符宽度。 |

## 当前 Zircon 渲染状态

`zircon_runtime_interface::ui::surface::render` 当前定义了中立 render DTO。`UiRenderCommandKind` 只有 `Group`、`Quad`、`Text`、`Image` 四类，`UiRenderCommand` 保存 `node_id`、`kind`、`UiFrame`、可选 `clip_frame`、`z_index`、`UiResolvedStyle`、可选 `UiResolvedTextLayout`、文本、图片引用和 opacity。这个 DTO 已经足够让 editor host painter 和 runtime renderer 共用基础视觉输出，但不足以描述 Slate 级 brush、material、vector、custom verts、batch key 和 debug/cached 状态。

`zircon_runtime::ui::surface::render::extract_ui_render_tree_from_arranged()` 已经从同一 `UiArrangedTree` 生成 `UiRenderExtract`，说明 render 与 hit/layout 共用 arranged geometry 的方向正确。`zircon_runtime::graphics::scene::scene_renderer::ui::render` 会把 visible UI extract 转成 screen-space quad vertices、scissor 和 text batches，并分流 auto/native/SDF 文本。`zircon_editor::ui::slint_host::host_contract::painter::render_commands` 也从同一 `UiRenderCommand` 生成 host paint commands。

当前关键限制是：runtime renderer 的批处理以 command 为单位规划 draw，缺少跨 command 的 batch key 合并；quad 只能表达实心背景和简单 border；image 只有 `UiVisualAssetRef`，尚未区分 image/box/border/vector/material resource；text layout 与 renderer text backend 尚未共享 shaped glyph truth；debug 数据还不能复盘为什么某个节点拆成某个 batch、某个 clip 或某次 drawcall。

## 渲染差异细化

| 领域 | Slate 参考行为 | Zircon 当前状态 | 缺口 |
|---|---|---|---|
| Paint element 类型 | `FSlateDrawElement` 覆盖 box、rotated box、text、shaped text、gradient、spline、line、dashed line、viewport、custom、custom verts、post-process blur。 | `UiRenderCommandKind` 只有 Group/Quad/Text/Image。 | 需要扩展为 `Brush`、`Border`、`RoundedRect`、`Line`、`Gradient`、`Vector`、`CustomVerts`、`ViewportTexture`、`Material` 等 payload；V1 可只实现 DTO + fallback 绘制，不一次完成全部 GPU shader。 |
| Brush / style payload | `FSlateBrush` 明确 draw type、margin、tiling、mirroring、UV、image type、outline、rounding、resource handle。 | `UiResolvedStyle` 保存背景、前景、border、font 等通用 style；image 另有 `UiVisualAssetRef`。 | 需要 `UiBrushRef`/`UiBrushPayload`，把 solid、image、box/9-slice、border、rounded、vector、material 与 resource handle 分开；`.ui.toml` 的 Material 组件应输出 brush，而不是只输出颜色字符串。 |
| Geometry / transform | Draw element 保存 position、local size、scale、render transform、pixel snapping 和 clip handle。 | Command 只保存绝对 axis-aligned `UiFrame` 与可选 `clip_frame`。 | 需要接入布局审计中的 `UiGeometry`、render transform、pixel snapping、clip zone；render command 应保留 unsnapped layout geometry 与 snapped paint geometry。 |
| Clip | Slate batch key 使用 clip state handle，RHI pass 可判定是否需要 stencil clipping。 | Runtime UI pass 用 axis-aligned `set_scissor_rect`；editor painter 用 `FrameRect` clip。 | V1 可继续 scissor，但 DTO 需要 `UiClipId`/clip chain/debug reason；后续支持 transformed/stencil clip 时不能重写所有 command。 |
| Batch key | Slate 以 resource + shader params + draw flags + shader type + primitive + draw effects + clip + instance data + scene index 作为 batch key，并稳定按 layer 合并。 | `plan_screen_space_ui_batches()` 逐 command 输出 vertex range 和 scissor；text 按 auto/native/SDF 分流。 | 需要显式 `UiBatchKey`、`UiBatchPlan`、stable layer/order 合并、batch split reason 和 final vertex/index/resource stats；text atlas page、image atlas page、material id 都必须进入 key。 |
| Cached element / invalidation | Slate cached element list 绑定 widget owner、cached clip states、cached batches 和 invalidation root；未失效时复用。 | Zircon 有 dirty flags、render extract rebuild、editor redraw 起点。 | 需要按 surface/node generation 建立 render command cache、batch plan cache、clip state cache；debug snapshot 应显示 recached node、cache hit/miss、invalidated reason。 |
| Resource / atlas | Slate brush 通过 resource handle/proxy 取渲染资源；batch 不能跨多个 shader resource 合并。 | Image/SVG/icon 已进入 shared UI asset 路径，runtime renderer 与 editor painter都有占位/加载路径。 | 需要 UI image/icon/SVG atlas contract、resource revision、UV rect、fallback texture 和 material resource key；atlas page 变化必须触发 batch/cache invalidation。 |
| Material / shader | Slate batch key 区分 shader type、shader params、draw flags、material shader；RHI renderer 接 HDR/VT feedback。 | UI pass 当前主要是 color quad + text；Material UI 主要还是组件视觉描述。 | 需要 UI material descriptor、shader variant key、blend/effect、sRGB/HDR policy、VT/feedback 占位；Material component 不能只停在编辑器 Slint 风格。 |
| Text render | Slate 区分 text/shaped text payload，font cache/shaper 与 draw element 共享 glyph sequence，支持 outline/overflow。 | `UiResolvedTextLayout` 已有行框、方向、overflow、range、line metrics、rich runs、editable caret/selection/composition DTO；runtime 使用 glyphon/SDF，editor fallback 仍有 estimated width。 | 需要把 shared layout foundation 继续接到真实 shaping/font fallback/BiDi visual order，并让 editor painter 和 runtime renderer消费同一 shaped text artifact。 |
| Debug visualizer | Slate 有 overdraw、batching、wireframe、drawcall stat、SDF text debug visual 和 Slate debugging stats。 | `UiHitTestDebugDump`、surface debug snapshot、drawcall/overdraw/material batch 统计只有起点。 | 需要 `UiRenderDebugSnapshot`，覆盖 command count、batch count、drawcall count、merged/split reason、overdraw heat、scissor/clip overlay、atlas/material stats、text backend stats。 |

## Zircon 渲染目标契约

渲染真源仍然是 `.ui.toml` 描述、`zircon_runtime_interface::ui` DTO 和 `UiSurfaceFrame`，不是 Unreal live widget `OnPaint` override。参考 Slate 时只采用职责划分：widget/tree 产出 paint element，element 携带 paint payload 与 geometry，batcher 根据 batch key 合并，renderer 只消费 batch plan 和资源句柄，debug 工具能解释每一步。

目标数据流固定为：

1. `.ui.toml` 编译出 node、style、brush/resource/material/text descriptors。
2. Layout pass 生成 `UiArrangedTree`，保留 layout geometry、render geometry policy、clip chain、z/layer/order、visibility。
3. Render extract 从 arranged tree 生成 `UiRenderCommand` 或下一代 `UiPaintElement`，并附带 brush/text/image/material payload、resource revision 和 invalidation generation。
4. Batcher 把 commands 转成 `UiBatchPlan`：batch key、draw ranges、clip/scissor/stencil need、resource handles、merge/split reason、debug counters。
5. Runtime WGPU UI renderer、editor host painter 和未来 offscreen/world-space UI 都消费同一 command/batch/debug 合同；host 不重新解释 `.ui.toml` 视觉。

## R1-R8 Contract Slice Status

本轮已经把 R1-R8 的 neutral DTO 底座落到 `zircon_runtime_interface::ui::surface::render`：

- `UiPaintElement` / `UiPaintPayload` 拆出 geometry、clip、effect、brush/text payload 和 cache/debug 预留字段。
- `UiBrushPayload` / `UiRenderResourceKey` 覆盖 solid、image、box、border、rounded、gradient、vector、material，以及 revision/atlas page/UV rect/fallback resource 这些 batch/cache key 必需字段。
- `UiRenderResourceState` 在 brush payload 中保留 resolved revision、atlas page、UV rect、pixel size 和 fallback resource，先把 image/icon/SVG/font atlas 与 material variant 的可观测状态纳入 shared contract。
- `UiShapedText` / `UiTextPaint` 先从 `UiResolvedTextLayout` 派生，保留 source/visual ranges、run kind、line frame、baseline、direction、overflow 和 text render mode，并扩展 font/atlas resource、ellipsis range、per-glyph id/advance/atlas UV、selection/caret/composition/decorator payload，作为后续真实 glyph shaping 的 shared artifact 槽位。
- `UiBatchKey` / `UiBatchPlan` / `UiBatchSplitReason` 提供 layer、clip、primitive、shader、resource、text backend、opacity 的稳定合并合同。
- `UiRenderCachePlan` / `UiRenderCacheInvalidationReason` / `UiRenderCacheStatus` 提供 paint/batch cache hit、rebuild、resource revision、clip、text shape、dirty node 等可复盘合同，但不在本切片抢占 runtime invalidation/performance 实现。
- `UiRenderDebugSnapshot` 可从 `UiRenderExtract` 导出 paint/batch/cache replay 数据，并接入 `UiSurfaceDebugSnapshot.render_batches`。
- `UiRenderVisualizerSnapshot` / `UiRenderVisualizerOverlay` / `UiRenderVisualizerStats` 提供 R7 Widget Reflector 类 render 面板所需的中立导出合同：paint element 表、batch group 表、wireframe/clip/batch/overdraw/resource/text overlay、overdraw regions、resource bindings、text backend/glyph/decorator stats 和 cache reuse/rebuild counters。
- `UiRendererParitySnapshot` / `UiRendererParityPaintRow` / `UiRendererParityBatchRow` 提供 R8 shared seam：runtime renderer、editor painter、offscreen capture 可以比较同一份 paint order、batch order、clip key、payload kind、resource key、text render mode 和 opacity identity，再把像素差异归因到明确 backend divergence。

当前仍保留 `UiRenderCommand` 作为迁移输入，没有给旧 struct literal 增加必填字段。R1-R6 已经完成 runtime diagnostics 的 `render_batches` 挂载与 editor Rust-owned painter 的派生 `UiPaintElement`/`UiPaintPayload` 消费；R7 只新增 shared visualizer export/replay DTO，不新增 live editor panel 或 runtime overlay 行为；R8 当前只新增 shared parity packet，不删除 editor painter/runtime renderer 的旧 consumption 分支。R4-R8 目前是合同层接入：真实 atlas 分配、资源 revision 驱动的局部 cache invalidation、runtime image/material shader 采样、GPU overdraw pass、live editor visualizer 面板、HarfBuzz 级 shaping、平台 IME 绘制以及 runtime/editor/offscreen parity golden tests 仍在后续 runtime renderer/text/editor 实现中完成。

R8 focused validation ran on 2026-05-06 with no runtime/editor painter edits. The final closeout rerun saw E: below the 50 GB cleanup threshold, so `cargo clean --target-dir "E:\zircon-build\targets\ui-render-r4-interface"` removed 755.6 MiB before validation. `rustfmt --edition 2021 --check "zircon_runtime_interface/src/ui/layout/slot.rs" "zircon_runtime_interface/src/ui/layout/linear_sizing.rs" "zircon_runtime_interface/src/ui/surface/render/debug.rs" "zircon_runtime_interface/src/ui/surface/render/parity.rs" "zircon_runtime_interface/src/ui/surface/render/mod.rs" "zircon_runtime_interface/src/ui/surface/mod.rs" "zircon_runtime_interface/src/tests/render_contracts.rs"` passed with no output. With `CARGO_TARGET_DIR=E:\zircon-build\targets\ui-render-r4-interface`, `cargo check -p zircon_runtime_interface --tests --locked --jobs 1 --message-format short --color never` finished successfully and `cargo test -p zircon_runtime_interface --lib render_contracts --locked --jobs 1 --message-format short --color never -- --nocapture` passed with `21 passed; 0 failed; 31 filtered out`.

R9 starts the parity/golden harness path without declaring renderer cleanup complete. The first slice remains interface-owned and uses hand-authored `UiRenderExtract` fixtures to prove `UiRendererParitySnapshot` semantic equality before any backend pixel output is considered. Runtime WGPU adapter output, editor painter adapter output, and offscreen RGBA golden comparison remain later implementation milestones; the acceptance order is shared paint/batch/clip/resource/text identity first, optional backend pixels second.

R9 focused validation ran on 2026-05-06 with no runtime/editor painter edits. D: had 125,702,815,744 bytes free, above the 50 GB cleanup threshold, so no target clean was needed. `rustfmt --edition 2021 --check "zircon_runtime_interface/src/tests/render_contracts.rs"` passed with no output. `cargo check -p zircon_runtime_interface --tests --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-render-r9-offscreen-parity" --message-format short --color never` finished successfully, and `cargo test -p zircon_runtime_interface --lib render_contracts --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-render-r9-offscreen-parity" --message-format short --color never -- --nocapture` passed with `23 passed; 0 failed; 32 filtered out`.

## Rendering Milestones

1. **R0 Slate 渲染证据冻结**：保留本文列出的 Unreal 源码路径，补充当前 Zircon render DTO/renderer/painter source map；后续 UI render PR 必须声明参考的 Slate element/batcher 行为和有意差异。
2. **R1 Paint DTO 扩展**：新增 `UiPaintElement` 或扩展 `UiRenderCommand`，拆出 geometry、clip id、paint payload、brush ref、resource/material key、draw effect、pixel snapping、debug/cached fields；保持旧 `Group/Quad/Text/Image` 作为迁移前的最小命令子集直到硬切完成。
3. **R2 Brush / Material Contract**：引入 `UiBrushPayload`，覆盖 solid、image、box/9-slice、border、rounded、gradient、vector/SVG、material；`.ui.toml` Material 原子组件输出 brush payload 和 resource key，而不是 editor-only 颜色约定。
4. **R3 Batch Plan**：实现 `UiBatchKey`、`UiBatchPlan`、stable layer/order merge、vertex/index buffers、drawcall count、batch split reason；runtime UI pass 与 editor painter 都能导出同一批处理统计。
5. **R4 Atlas / Resource Integration**：把 image/icon/SVG/font atlas 页、UV rect、resource revision、fallback resource、material variant 接入 batch key；资源 revision 改变时只 invalidate 对应 render cache。
6. **R5 Cached Render Elements**：基于 `UiSurfaceFrame` generation、node dirty flags、clip states、resource revisions 建立 render command/batch cache；debug snapshot 显示 recached nodes、cache hit/miss 和 invalidation reason。
7. **R6 Text Draw Truth**：把 text measure、line layout、glyph shaping、glyphon/SDF draw plan、editor painter text artifact 收敛到同一 shaped text DTO；补齐 outline、overflow/ellipsis、BiDi、rich text、selection/caret 和 IME composition 的 paint payload。
8. **R7 Debug Visualizer**：实现 Widget Reflector 类 render 面板，输出 live/snapshot tree、paint elements、batch groups、drawcall/overdraw/material stats、clip/scissor overlay、wireframe、SDF text debug 和 export/replay artifact。
9. **R8 Renderer Parity And Cleanup**：runtime WGPU UI renderer、editor painter、offscreen capture 使用同一 command/batch contract；删除 editor-only visual special cases 和 command-kind 兼容分支；screen-space golden/pixel tests 覆盖 core paint payload。

## Acceptance Tests

渲染验收应从 DTO 到 renderer 逐层推进，避免只靠 editor 截图或单一 runtime fixture：

- `zircon_runtime_interface` contract：paint element 能序列化 brush/material/text/clip/batch key 所需字段，并保留 old command migration snapshot。
- Brush tests：solid、image、box/9-slice、border、rounded、gradient、vector/SVG、material payload 都能从 `.ui.toml` 编译到 render DTO；缺失资源产生显式 fallback payload。
- Batch tests：相同 layer/resource/material/clip/effect/primitive 的 commands 合并；不同 clip、atlas page、material variant、text backend 和 draw effect 保留 split reason。
- Runtime renderer tests：quad、rounded rect、border、image atlas、SVG/vector fallback、material fallback pass、line/gradient 的 vertex/index ranges 与 drawcall count 可断言。
- Text tests：shared shaped layout 同时驱动 measure、glyphon/SDF draw plan 和 editor painter；wrap、ellipsis、BiDi、outline、selection/caret/IME payload 有 golden 或 contract coverage。
- Cache tests：style-only、layout-only、text-only、resource-revision、clip-chain 改动只 invalidates 对应 command/batch/cache；unchanged subtree 复用 batch plan。
- Debug tests：snapshot 导出 command count、batch count、drawcall count、overdraw heat bins、clip/scissor overlays、merged/split reason、resource/material stats，并可 replay 到最小 UI fixture。
- Editor/runtime parity：同一 `.ui.toml` 在 editor painter 与 runtime WGPU renderer 中输出一致 paint element order、clip chain、batch stats 和 text layout artifact；像素差异必须能追溯到明确 backend divergence。

## Assumptions

- V1 继续以 screen-space UI 为主；world-space/3D UI 预留 viewport texture、scene index 和 transform 字段，但不阻塞 R1-R3 的 DTO/batch 收敛。
- `UiRenderCommandKind::{Group, Quad, Text, Image}` 是当前实现事实，不是长期最终形态；扩展时应硬切到类型化 paint payload，避免无限添加兼容字段。
- Unreal `FSlateBrush` 和 `FSlateDrawElement` 只作为职责和验收参考；Zircon 不引入 C++ live widget subclass 或 `OnPaint` override 模型。
- Runtime renderer 是最终渲染能力真源；editor host painter 只能作为同一 DTO 的预览/宿主投影，不能拥有独立视觉语义。
- 本文是审计和里程碑文档，没有声明任何测试已运行或通过。
