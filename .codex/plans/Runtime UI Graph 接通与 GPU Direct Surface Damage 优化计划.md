# Runtime UI Graph 接通与 GPU Direct Surface Damage 优化计划

## Summary
- 目标一：完成 M7A `Runtime UI Placement In Product Pipeline`，让 `render_product_ui` 通过，并确保 runtime UI 由 render graph 的 `ui.screen-space` executor 执行，不再走 renderer 末尾的 legacy UI side path。
- 目标二：在 retained editor GPU UI surface 上恢复局部 repaint/damage 优化，但不依赖 swapchain 旧内容，也不回退到旧的“offscreen 主渲染目标 + final blit”流程。
- 当前实现已有部分脏改：`RenderStats` 已出现 UI graph stats 字段，`RenderPassGpuExecutionContext` 已开始携带 `ScreenSpaceUiRenderer`，`render_compiled_scene` 已开始移除末尾 legacy UI record。实施时以清理、补齐、验证这些已有改动为主。

## Status 2026-05-18 Closeout
- 已完成 M7A runtime UI graph 接通：`RenderPassExecutorRegistry` 注册 `ui.screen-space`，executor 通过 GPU context 的 `record_screen_space_ui_to_resource(PostProcessGraphResourceNames::FINAL_COLOR)` 调用共享 `ScreenSpaceUiRenderer`，`render_product_ui` 已通过。
- 已完成 GPU direct surface damage cache：`WgpuRetainedSurfaceCache` 作为 direct-present 内部 retained cache；full frame 直接绘制到 swapchain 并同步 cache，damage frame 先 restore cache 到当前 swapchain，再 patch damage 相交 draw ops 到 swapchain/cache。
- 已确认旧 offscreen/blit 主路径没有回到生产源码；`WgpuOffscreenTarget`、`WgpuBlitResources`、`blit_offscreen_to_surface`、`zircon-ui-offscreen` 只保留在计划/历史说明里作为禁止或归档语义。
- 已更新 `docs/assets-and-rendering/render-framework-architecture.md`、`docs/zircon_runtime/rhi/ui_surface.md`、`docs/zircon_editor/ui/retained_host/performance.md`，并归档 `.codex/sessions/20260518-0210-runtime-ui-graph-and-direct-damage.md` 到 `.codex/sessions/archive/`。
- 已完成 scoped validation，使用 `CARGO_TARGET_DIR=target/codex-native-material-painter`：runtime/editor `cargo check --lib --locked`，`render_product_ui`、`runtime_ui`、runtime `ui_surface`、editor `gpu_presenter`、runtime `material_button_style`、editor `native_material_painter` 均通过；追加 source guards `builtin_registry_covers_runtime_ui_executor_id` 与 `native_ui_surface_source_uses_direct_surface_without_offscreen_blit` 也通过。`git diff --check` 通过，仅有 LF/CRLF 提示。
- 未运行 workspace-wide validation；当前 checkout 仍有其它 active sessions 的无关 dirty files 与独立阻塞，不能把本计划 scoped green 解释为 workspace green。

## Key Changes

### 1. Runtime UI Product Graph
- 保留 `RenderPassGpuExecutionContext` 持有 `ScreenSpaceUiRenderer` 的方向，但把直接字段访问收敛成 GPU context 方法，例如 `record_screen_space_ui_to_resource(resource_name)`。
- `ui.screen-space` executor 只做一件事：通过 GPU context 读取 `final-color` texture view，并调用 `ScreenSpaceUiRenderer::record(...)` 录制 UI draw。
- `execute_graph_stage(...)` 继续把同一个 `ScreenSpaceUiRenderer` 传给 graph pass context，保证 graph executor 与 renderer core 共用同一套 UI pipeline/text atlas/cache。
- `render_compiled_scene(...)` 中保留 `LATE_GRAPH_STAGES = [Ui, Overlay, Debug]`，并删除末尾独立的 `screen_space_ui_renderer.record(...)` legacy 调用，避免 UI 被绘制两次。
- `RenderGraphExecutionResources` 必须在 frame target import 阶段绑定 `final-color`，供 `ui.screen-space` 使用；UI pass 不创建自己的目标，也不绕过 product graph。
- `RenderStats` 继续暴露：
  - `last_ui_graph_executed_pass_count`
  - `last_ui_target_size`
  - `last_ui_graph_pass_order`
- `last_ui_graph_pass_order` 必须基于实际 pass index 计算，只有确认 `post-process < runtime-ui < overlay-gizmo` 时才返回 `Some("postprocess-ui-overlay")`，不能只检查三个 pass 是否存在。

### 2. Direct Surface Damage Cache
- 在 `WgpuUiSurfaceRenderer` 内部新增 retained cache，例如 `WgpuRetainedSurfaceCache`：
  - texture format 使用当前 non-sRGB swapchain format。
  - usage 至少包含 `RENDER_ATTACHMENT | TEXTURE_BINDING`。
  - 若实现 copy restore 快路径，则额外包含 `COPY_SRC`，surface usage 仅在 capabilities 支持时加入 `COPY_DST`。
  - resize 后重建 cache，并标记 `initialized = false`。
- 全量帧流程：
  - 获取当前 surface texture。
  - 使用 damage 清空后的 full draw list，直接绘制到 swapchain surface view。
  - 同一批 draw ops 再绘制到 retained cache view，保持 cache 与屏幕一致。
  - 标记 cache initialized。
- Damage 帧流程：
  - 前置条件：`draw_list.damage.is_some()` 且 cache initialized 且 size/format 匹配。
  - 先把 retained cache 恢复到当前 swapchain surface view。
  - 再用原始 damage draw list 生成 patch draw ops，只绘制与 damage 相交的 commands。
  - patch draw ops 同时绘制到 swapchain surface view 和 retained cache view。
- cache restore 允许使用内部 fullscreen restore pipeline 或 supported usage 下的 copy path，但命名上不要恢复旧 offscreen/blit 概念：
  - 使用 `restore_cache_to_surface` / `WgpuRetainedSurfaceCache`。
  - 不恢复 `WgpuOffscreenTarget`、`WgpuBlitResources`、`blit_offscreen_to_surface`。
- 把现有 `render_draw_list_to_surface(...)` 拆成可复用 recording helper：
  - 获取/管理 surface texture 的逻辑单独保留。
  - solid/image/text vertex buffer 构建单独保留。
  - `record_draw_ops_to_view(view, load_op, draw_ops)` 可对 surface 和 cache 顺序调用。
  - text batches 使用同一次 `WgpuUiTextRenderer::prepare(...)` 后顺序录制到两个 target；若编译暴露 glyphon replay 限制，则改为每个 target 前重新 prepare，但行为保持一致。

### 3. Stats And Tests
- `UiSurfaceDrawList::stats()` 保持 damage-aware：patch frame 的 visible command/item 只统计 damage 内实际参与 patch 的命令。
- `WgpuUiSurfacePresenter` 的 batch stats：
  - full frame 使用 full draw list batch plan。
  - damage frame 使用 damage draw list batch plan。
  - `draw_calls` 记录 surface presentation 相关 draw/restore 工作；cache maintenance 不改变 public command vocabulary。
- 更新/新增 source guards：
  - 保证旧 offscreen/blit 主路径没有回来。
  - 允许并要求 retained cache 只作为 damage restore/cache maintenance 存在。
- 更新 editor GPU presenter 测试：
  - direct surface 支持 damage 后，damage present 不再强制构建 full command stream。
  - 但当 cache 未初始化、resize 后、或 damage 缺失时仍必须 full redraw。
- 更新 runtime UI graph 测试：
  - `render_product_ui` 断言 `runtime-ui` graph pass 执行一次。
  - 断言 executor ids 包含 `ui.screen-space`。
  - 断言 UI target size 为 viewport size。
  - 断言 pass order 为 `postprocess-ui-overlay`。

## Test Plan
- 先运行轻量编译：
  - `cargo check -p zircon_runtime --lib --locked`
  - `cargo check -p zircon_editor --lib --locked`
- M7A focused gate：
  - `cargo test -p zircon_runtime --locked render_product_ui --jobs 1 --message-format short --color never`
  - `cargo test -p zircon_runtime --locked runtime_ui --jobs 1 --message-format short --color never`
- RHI UI surface focused gate：
  - `cargo test -p zircon_runtime --lib ui_surface --locked --jobs 1 --message-format short --color never`
  - `cargo test -p zircon_editor --lib gpu_presenter --locked --jobs 1 --message-format short --color never`
- Regression guard：
  - `cargo test -p zircon_runtime --lib material_button_style --locked --jobs 1 --message-format short --color never`
  - `cargo test -p zircon_editor --lib native_material_painter --locked --jobs 1 --message-format short --color never`
- Closeout：
  - `git diff --check`
  - 更新 `docs/assets-and-rendering/render-framework-architecture.md`
  - 更新 `docs/zircon_runtime/rhi/ui_surface.md`
  - 归档当前 `.codex/sessions/20260518-0210-runtime-ui-graph-and-direct-damage.md`

## Assumptions
- 只改 retained editor GPU UI surface 的 damage/cache 行为，不改 scene/viewport offscreen renderer。
- Runtime UI graph placement 属于 M7A 正常路径，不为 `render_product_ui` 写测试专用 shortcut。
- Direct surface 仍是最终 present 路径；retained cache 只是 damage frame 的 restore/patch 基底。
- 保持 non-sRGB byte-space parity：swapchain format、cache texture format、image upload texture format都不使用 sRGB。
- 不触碰 `.zmaterial`、material editor UI、runtime UI showcase/data-source、MUI 组件计划或无关 dirty files。
