---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/core/framework/render/ui_submission.rs
  - zircon_runtime/crates/zr_rhi/src/ui_surface.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/ui
  - zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface
  - zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface.rs
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - docs/assets-and-rendering/runtime-ui-graphics-integration.md
  - docs/assets-and-rendering/runtime-ui-slate-rendering-gap-audit.md
tests:
  - zircon_runtime/src/graphics/tests/render_product_ui.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/tests.rs
  - zircon_runtime/tests/runtime_ui_text_render_contract.rs
doc_type: module-detail
---

# UI 渲染交界

## 两条 UI 产品路径

ZirconEngine 有两条相关但独立的 UI 渲染路径：

1. **Viewport Screen-Space UI**：`UiRenderSubmission` 随场景帧提交，作为 Render Graph 的 UI Pass 绘制到 viewport 最终目标。游戏 HUD、viewport overlay 使用它。
2. **RHI Retained UI Surface**：`UiSurfaceDrawList` 交给 `UiSurfacePresenter`，直接呈现编辑器/工具宿主窗口；它有独立的 retained geometry、damage、image registry 和 WGPU Surface 生命周期。

不要把第二条当作第一条的低级实现，也不要把 `UiRenderExtract` 直接传给 `UiSurfacePresenter`。两者共享部分视觉语义和 GPU context，但拥有不同提交、缓存和坐标合同。

## Viewport UI

`UiRenderSubmission` 可包含多个 segment，每段拥有 route tree ID 和 node ID projection，防止来自不同 UI tree 的节点身份碰撞。它与 `RenderFrameExtract` 一起通过 `submit_frame_extract_with_ui` 或 `present_frame_extract_with_ui` 提交。

规划器将 `UiRenderCommand` 转换为：

- 背景、边框、圆角与装饰 vertex draw；
- image batch；
- Auto、Native、SDF 三类 text batch；
- clip/scissor；
- rich text/paint element 和 post-text decoration。

UI 在 3D 默认管线中位于 post process、overlay/debug 之后，确保 screen-space UI 不被场景 tone/effect 改写；Core2D 的 phase mapping 不完全相同，定制管线应显式检查 terminal ordering。

## 文本

`ScreenSpaceUiTextSystem` 解析 Auto 路由，使用字体资产和显式 style 选择 native 或 SDF。Text layout 在 UI extract 层已生成 resolved lines/glyph 信息时，graphics 层应消费结果而不是重新排版。

SDF 路径维护 glyph atlas、slot cache、bake/upload report 和 dirty 计划；native path 使用对应文本 backend。空白 glyph 保留 advance 但不要求可见 atlas slot。字体、raster scale、size、route identity 或 glyph artifact revision 变化会使相关缓存失效。

## 图像与共享 GPU 产品

Viewport UI 的 image asset 解析为 runtime resource ID，由 UI image system 建立 texture/bind group。Retained UI Surface 还支持 `UiSurfaceImageResourceTable` 和 `WgpuUiSharedImageRegistry`：同设备场景 viewport product 可作为 external image 被 editor UI 采样，避免 CPU capture/readback 再上传。

共享图像以 resource key + generation 标识；registry、surface cache 和 in-flight present 都会持有 pin。只有最后一个 registry/surface/GPU pin 释放后才能销毁物理 allocation。

## Retained UI Surface

`UiSurfaceDrawList` 包含 surface size、projection size、damage、ordered commands、可选 generation、compact styles 和 image resources。Presenter 支持 resize、resident 查询、borrowed/owned present 和 last stats。

```rust
use zircon_runtime::rhi::{
    UiSurfaceDescriptor, UiSurfaceDrawList, UiSurfacePresenter,
};

fn present_ui(
    presenter: &mut dyn UiSurfacePresenter,
    draw_list: UiSurfaceDrawList,
) -> Result<(), zircon_runtime::rhi::RhiError> {
    let stats = presenter.present_owned(draw_list)?;
    if !stats.outcome.is_submitted() {
        // Surface 暂不可呈现；等待 resize/redraw 事件后重试。
    }
    Ok(())
}
```

创建方式：独立 UI 可调用 `zircon_runtime::rhi::create_default_ui_surface_presenter`；需要采样同设备 viewport product 时，应调用 `RenderFramework::create_ui_surface_presenter`，让框架提供共享 WGPU context。

## Damage、Retained Cache 与 Resize

Damage rectangle 只提交受影响区域。WGPU presenter 会缓存编译后的 batch、vertex、text 和 image dependencies；generation 未变且 projection 不变时可复用。

`retarget_surface_size_preserving_projection` 是 native resize transaction 的内部接口：改变实际 target，不改变该 producer generation 的坐标空间，从而保留几何缓存。普通调用者不应手工使用 `#[doc(hidden)]` 方法。

## 命令与样式

`UiSurfaceCommandKind` 支持 Quad、Border、Text、Image、Styled 和 Clip。Compact styles 将重复样式 intern 为 `UiSurfaceStyleHandle`，降低命令 payload。`resolved_kind()` 会验证 style handle 与 payload，非法行不应进入 GPU draw。

`UiSurfacePresentStats` 提供编译/实际 draw 数、damage scan、batch merge、overlap、vertex/image/text cache、上传、GPU time、共享 image residency/pin 等指标。结构为 `#[non_exhaustive]`，外部代码构造或匹配时必须允许新增字段。

## 状态与限制

- Viewport UI、Native/SDF 文本、image、clip 和 WGPU retained Surface 为**默认可用**（需 `ui` + `graphics` feature）。
- 同设备 shared viewport image 为**默认可用的 WGPU 路径**，不是后端中立共享句柄标准。
- Damage/retained cache 优化不能改变绘制顺序、clip 或 alpha 合成结果。
- Accessibility、input/focus 属于 UI 系统，不由 graphics presenter 负责。
