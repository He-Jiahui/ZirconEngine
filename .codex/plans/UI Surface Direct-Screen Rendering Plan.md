# UI Surface Direct-Screen Rendering Plan

## Summary
- 现有 UI offscreen 主要是为了保留上一帧做 damage patch，再 blit 到窗口；但这会多一次纹理采样/格式转换路径，和你要的简单高效、颜色一致方向冲突。
- 决策改为：`zircon_runtime::rhi_wgpu::ui_surface` 的 native GPU UI 直接渲染到 swapchain surface。typed button style 只产生命令数据，不引入任何 offscreen 风格层。
- 范围只覆盖 editor chrome / Material UI surface；scene/viewport 的 offscreen 继续保留，因为它承担后处理、截图、视口纹理等职责。
- 2026-05-18 后续收口已恢复 GPU direct surface 的局部 repaint/damage：不依赖 swapchain 旧内容，也不回退旧 offscreen 主渲染目标；runtime 内部 retained cache 只作为 damage frame 的 restore/patch 基底。
- 同一轮也完成 active render M7A：runtime UI 已通过 product render graph 的 `ui.screen-space` executor 录制到 `final-color`，`render_product_ui` 不再卡在 executor registration blocker。

## Key Changes
- 在 `zircon_runtime/src/rhi_wgpu/ui_surface.rs` 硬切掉 UI offscreen/blit：删除 `WgpuOffscreenTarget`、`WgpuBlitResources`、`offscreen_initialized`、offscreen render pass 和 blit pass。
- native present 流程改为：获取当前 surface texture，按 batch plan 将 solid/image/text 直接画进 surface view，首个 pass clear black，同一帧后续 pass load，最后 `present()`。
- surface format 只接受非 sRGB：优先 `Bgra8Unorm` / `Rgba8Unorm`；如果平台只给 sRGB，GPU UI presenter 初始化失败并走现有 fallback，不再静默引入颜色空间差异。
- direct swapchain 不依赖上一帧内容：`WgpuRetainedSurfaceCache` 持有内部非 sRGB cache texture。全量帧直接画到 swapchain 并同步更新 cache；cache initialized 且 draw list 带 damage 时，先把 cache restore 到当前 swapchain，再只把 damage 相交 commands patch 到 swapchain 和 cache。
- button style binding 继续使用 `ResolvedButtonStyle` / `ButtonStyleFields` / weak `StyleSheetScope`，editor painter 消费 resolved style 后输出普通 UI commands，不增加专用离屏按钮路径。
- runtime UI product graph 使用 shared `ScreenSpaceUiRenderer`：`RenderPassGpuExecutionContext::record_screen_space_ui_to_resource("final-color")` 封装 final-color lookup，`ui.screen-space` executor 只负责从 graph pass 调用该入口。
- `render_compiled_scene(...)` 不再在末尾额外调用 legacy UI record；late graph stages 继续覆盖 `Ui`、`Overlay`、`Debug`，统计以实际 pass index 证明 `post-process < runtime-ui < overlay-gizmo`。

## Public Interfaces / Types
- `UiSurfacePresenter`、`UiSurfaceDrawList`、`ChromeCommandStream` 对外语义不新增公开 command kind。
- WGPU UI 内部可把 `OFFSCREEN_FORMAT` 改名为 image upload texture format；它只用于上传图片纹理，不再代表 render target。
- `ResolvedButtonStyle` 保持 runtime/editor 共享 DTO，用于按钮样式解析和 painter 输入。

## Test Plan
- `cargo test -p zircon_runtime --lib wgpu_ui_surface_uses_non_srgb_formats_for_byte_exact_editor_parity --locked --jobs 1`
- 新增/更新 runtime 测试：确认 native UI source 不再包含 `zircon-ui-offscreen`、`blit_offscreen_to_surface`、`WgpuOffscreenTarget`；确认 sRGB-only surface formats 不被选为 GPU UI target。
- 新增/更新 editor GPU presenter 测试：cache 未初始化、resize 后或缺少 damage 时 full redraw；cache ready 后 damage present 保留 patch stream，不再强制构建完整 command stream。
- 保留并运行 button style focused tests：`cargo test -p zircon_runtime --lib material_button_style --locked --jobs 1`、`cargo test -p zircon_editor --lib native_material_painter --locked --jobs 1`。
- M7A focused tests：`cargo test -p zircon_runtime --locked render_product_ui --jobs 1 --message-format short --color never`、`cargo test -p zircon_runtime --locked runtime_ui --jobs 1 --message-format short --color never`。
- live evidence 阶段重跑 `startup` / `viewport_image` focused profile，验收无 software fallback、颜色 parity 不回退、draw-call batching 仍有效。

## Assumptions
- “统一流程”锁定为 UI surface native GPU 路径统一直接到屏幕；不改 scene renderer 的 offscreen。
- direct UI present 允许 retained cache damage restore/patch，但 cache 不是主 offscreen render target，也不恢复 `WgpuOffscreenTarget`、`WgpuBlitResources`、`blit_offscreen_to_surface`。
- 不触碰 Material Lab 布局/截图脚本所有权，也不改变默认 focus/a11y 交互语义。

## Closeout 2026-05-18
- 已完成 direct-screen cutover：native WGPU UI presenter 不再保留 UI offscreen target，也不再执行 final blit；swapchain target 限定为非 sRGB。
- 已完成 retained-cache damage follow-up：full frame 直接绘制到 swapchain 并更新 cache；damage frame 从 retained cache restore 当前 swapchain，再 patch damage 相交 commands 到 swapchain/cache。source guard 保持旧 offscreen/blit 主路径不回归。
- 已完成 active render M7A：`render_product_ui` 通过，`runtime-ui` graph pass 通过 `ui.screen-space` executor 读取 `final-color` 并调用共享 `ScreenSpaceUiRenderer`；renderer tail legacy UI record 已移除。
- 已补验证：`cargo check -p zircon_runtime --lib --locked`、`cargo check -p zircon_editor --lib --locked`、runtime `render_product_ui`、runtime `runtime_ui`、runtime `ui_surface`、editor `gpu_presenter`、runtime `material_button_style`、editor `native_material_painter` 均通过。`native_material_painter` 最终使用 `target/codex-native-material-painter` 避开无关并行 Cargo lock contention，结果为 6 passed。
