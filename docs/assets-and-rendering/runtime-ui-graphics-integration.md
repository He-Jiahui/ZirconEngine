---
related_code:
  - zircon_framework/src/render/backend_types.rs
  - zircon_graphics/Cargo.toml
  - zircon_graphics/src/lib.rs
  - zircon_graphics/src/runtime/mod.rs
  - zircon_graphics/src/runtime/ui/mod.rs
  - zircon_graphics/src/runtime/ui/runtime_ui_fixture.rs
  - zircon_graphics/src/runtime/ui/runtime_ui_manager.rs
  - zircon_graphics/src/runtime/ui/runtime_ui_manager_error.rs
  - zircon_graphics/src/runtime/ui/fixtures/hud_overlay.ui.toml
  - zircon_graphics/src/runtime/ui/fixtures/pause_menu.ui.toml
  - zircon_graphics/src/runtime/ui/fixtures/settings_dialog.ui.toml
  - zircon_graphics/src/runtime/ui/fixtures/inventory_list.ui.toml
  - zircon_graphics/src/runtime/render_framework/mod.rs
  - zircon_graphics/src/runtime/render_framework/submit_runtime_frame.rs
  - zircon_graphics/src/runtime/render_framework/submit_frame_extract/mod.rs
  - zircon_graphics/src/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_graphics/src/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_graphics/src/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_graphics/src/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs
  - zircon_graphics/src/types/editor_or_runtime_frame.rs
  - zircon_graphics/src/types/editor_or_runtime_frame_with_ui.rs
  - zircon_graphics/src/scene/scene_renderer/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core/scene_renderer_core.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_new/construct/construct.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_graphics/src/scene/scene_renderer/ui/mod.rs
  - zircon_graphics/src/scene/scene_renderer/ui/screen_space_ui_renderer.rs
  - zircon_graphics/src/scene/scene_renderer/ui/new.rs
  - zircon_graphics/src/scene/scene_renderer/ui/render.rs
  - zircon_graphics/src/scene/scene_renderer/ui/shaders/screen_space_ui.wgsl
  - zircon_graphics/src/tests/runtime_ui_integration.rs
implementation_files:
  - zircon_framework/src/render/backend_types.rs
  - zircon_graphics/Cargo.toml
  - zircon_graphics/src/lib.rs
  - zircon_graphics/src/runtime/mod.rs
  - zircon_graphics/src/runtime/ui/mod.rs
  - zircon_graphics/src/runtime/ui/runtime_ui_fixture.rs
  - zircon_graphics/src/runtime/ui/runtime_ui_manager.rs
  - zircon_graphics/src/runtime/ui/runtime_ui_manager_error.rs
  - zircon_graphics/src/runtime/ui/fixtures/hud_overlay.ui.toml
  - zircon_graphics/src/runtime/ui/fixtures/pause_menu.ui.toml
  - zircon_graphics/src/runtime/ui/fixtures/settings_dialog.ui.toml
  - zircon_graphics/src/runtime/ui/fixtures/inventory_list.ui.toml
  - zircon_graphics/src/runtime/render_framework/mod.rs
  - zircon_graphics/src/runtime/render_framework/submit_runtime_frame.rs
  - zircon_graphics/src/runtime/render_framework/submit_frame_extract/mod.rs
  - zircon_graphics/src/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_graphics/src/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_graphics/src/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_graphics/src/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs
  - zircon_graphics/src/types/editor_or_runtime_frame.rs
  - zircon_graphics/src/types/editor_or_runtime_frame_with_ui.rs
  - zircon_graphics/src/scene/scene_renderer/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core/scene_renderer_core.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_new/construct/construct.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_graphics/src/scene/scene_renderer/ui/mod.rs
  - zircon_graphics/src/scene/scene_renderer/ui/screen_space_ui_renderer.rs
  - zircon_graphics/src/scene/scene_renderer/ui/new.rs
  - zircon_graphics/src/scene/scene_renderer/ui/render.rs
  - zircon_graphics/src/scene/scene_renderer/ui/shaders/screen_space_ui.wgsl
plan_sources:
  - user: 2026-04-18 下一步可以直接进入 Graphics/runtime integration
  - .codex/plans/编辑器 .slint 去真源 Runtime UI 可用 Cutover 路线图.md
tests:
  - zircon_graphics/src/tests/runtime_ui_integration.rs
  - cargo test -p zircon_graphics --lib --locked --features runtime-ui-integration-tests runtime_ui_manager_builds_all_builtin_fixtures_into_shared_surfaces -- --nocapture
  - cargo test -p zircon_graphics --lib --locked --features runtime-ui-integration-tests render_framework_submits_runtime_ui_frames_and_renders_pause_menu_panels -- --nocapture
  - cargo test -p zircon_graphics --lib --locked --features runtime-ui-integration-tests render_framework_reports_clipped_ui_commands_for_inventory_fixture -- --nocapture
  - cargo test -p zircon_graphics --lib --locked --features runtime-ui-integration-tests
  - cargo test -p zircon_framework --lib --locked
doc_type: module-detail
---

# Runtime UI Graphics Integration

## Purpose

这一步把 `Runtime visual contract` 真正接到了 `zircon_graphics` 的运行时提交流水里，不再停留在 `zircon_ui::UiRenderExtract` 只存在于 shared core 的状态。完成后，运行时 UI 的最短闭环固定为：

1. `.ui.toml` 资产定义结构、布局和视觉字段。
2. `RuntimeUiManager` 通过 `UiAssetLoader -> UiDocumentCompiler -> UiTemplateSurfaceBuilder` 构建 shared `UiSurface`。
3. `RuntimeUiManager::build_frame()` 生成带 `UiRenderExtract` 的 `EditorOrRuntimeFrame`。
4. `WgpuRenderFramework::submit_runtime_frame(...)` 复用既有 render framework 状态机，把 UI 和 scene/GI/VG payload 一起送进 `SceneRenderer`。
5. `ScreenSpaceUiRenderer` 在 scene overlays 之后追加 screen-space UI pass，把 shared draw list 画到最终颜色目标。

这使得 runtime fixture 不再依赖 Slint preview 或 editor host 才能“看到 UI”。

## Runtime Fixture Assets

运行时内建 fixture 现在都落成真实的 `.ui.toml` 文件，路径在 `zircon_graphics/src/runtime/ui/fixtures/`：

- `hud_overlay.ui.toml`
- `pause_menu.ui.toml`
- `settings_dialog.ui.toml`
- `inventory_list.ui.toml`

这里故意没有把 fixture 写成 Rust 手工树，而是保留成 editor/runtime 共识的文本资产，原因有两个：

- 它们可以继续被现有 UI asset editor 读取和可视化编辑，不会因为 graphics 接入又引入一套只给测试看的私有构造 DSL。
- 这四个 fixture 正好对应 roadmap 里的 runtime 验收样本，后续做 screenshot golden、route golden、输入回放时可以直接复用同一份资产。

`RuntimeUiFixture` 只负责把 fixture 名称映射到稳定 `asset_id` 和源码文本；真正的解析、编译和 surface 构建全部仍交给 `zircon_ui` 的 shared template/runtime 链路。

## Runtime UI Manager

`RuntimeUiManager` 的职责被刻意压小，只做 runtime-host 最基础的三件事：

- 持有 `viewport_size`
- 加载一个 builtin fixture 并构建 `UiSurface`
- 把当前 `UiSurface.render_extract` 打包成 `EditorOrRuntimeFrame`

它目前不承担字体 atlas、图片流送、文本输入、焦点策略或输入分发；这些仍属于后续 runtime host service 的继续扩展点。当前 manager 的作用是把 graphics/runtime integration 需要的最小 host boundary 先钉死：

- runtime UI 的结构和视觉数据必须来自 shared `UiSurface`
- graphics 只吃 `UiRenderExtract`
- scene extract 仍保持为空场景快照，screen-space UI pass 不理解 editor-only docking/page/window 语义

`build_frame()` 使用空 `RenderSceneSnapshot` 加默认 camera/preview clear color，目的是让 HUD、暂停菜单、设置对话框这类 screen-space fixture 能在无 scene 内容时也稳定出图。

## Frame Carrier And Render Framework Boundary

`EditorOrRuntimeFrame` 新增了公开字段 `ui: Option<UiRenderExtract>`，并配套了 `with_ui(...)` builder。这样运行时 UI 不需要再修改 `RenderFramework` trait 或把 `zircon_ui` 依赖推回 `zircon_framework`；UI payload 只在 `zircon_graphics` 内部的 runtime frame carrier 上存在。

在 render framework 内部：

- `WgpuRenderFramework` 新增 inherent 方法 `submit_runtime_frame(viewport, frame)`
- 既有 `submit_frame_extract(...)` 保持不变，继续服务纯 scene extract 提交
- `submit_runtime_frame(...)` 复用 `build_frame_submission_context(...)`、`prepare_runtime_submission(...)`、history 轮换、GPU completion 收集和 `update_stats(...)`

差异只在一处：runtime frame 路径会保留调用者已经放进来的 `frame.ui`，再把 hybrid GI / virtual geometry 的 runtime prepare payload 叠回同一个 `EditorOrRuntimeFrame`，然后交给 `SceneRenderer::render_frame_with_pipeline(...)`。

这条边界让 graphics/runtime integration 先局部闭环，而不需要在当前阶段扩张 `RenderFramework` trait 或引入一条新的跨 crate 公共 render command 协议。

## UI Stats

`RenderStats` 新增了 runtime UI 相关计数：

- `last_ui_command_count`
- `last_ui_quad_count`
- `last_ui_text_payload_count`
- `last_ui_image_payload_count`
- `last_ui_clipped_command_count`

这些值在 `build_frame_submission_context(...)` 阶段直接从 `UiRenderExtract` 统计，随后由 `update_base_stats(...)` 写回最终 stats。这样做有两个好处：

- stats 不依赖 GPU readback 或 screenshot，对 headless 验证非常稳定。
- 统计口径固定绑定 shared draw list，而不是 renderer 内部的临时分解结果。比如 border 可能在 GPU 侧拆成多个矩形，但 `last_ui_quad_count` 仍然反映 shared contract 里的 quad command 数量。

## Screen-Space UI Pass

`SceneRendererCore` 现在拥有一个新的 `ScreenSpaceUiRenderer`，并在两条渲染路径里都接入：

- `render_scene(...)`
- `render_compiled_scene(...)`

record 顺序固定为：

1. scene/deferred/post-process
2. existing overlay renderer
3. screen-space UI renderer

这保证 runtime UI 在最终颜色目标上显示为真正的后置屏幕空间层，不参与 scene depth，也不会被 editor overlay pass 的内部顺序反向覆盖。

当前 pass 采用一个最小但真实的 GPU 实现：

- WGSL shader 只吃 position/color 顶点
- CPU 侧把 shared `UiRenderCommand` 转成三角形顶点
- quad/background/border 直接映射成实矩形
- text/image 暂时走占位可见渲染，而不是完整字体 atlas / 纹理采样
- `clip_frame` 通过 `set_scissor_rect(...)` 变成真实 scissor

这不是最终 UI renderer 的终态，但它已经满足 roadmap 当前阶段要求：`zircon_graphics` 真正消费 shared visual draw list，而不是继续停留在“仅统计 frame geometry”。

## Current Deliberate Limitations

这一步刻意没有把下面这些内容一并做完：

- 字体测量和真实 glyph atlas
- 图片/图标纹理资产装载与采样
- world-space UI
- 复杂主题动画和富文本排版

原因不是它们不重要，而是当前 slice 的完成门槛是先把 shared runtime visual contract 接上真实 graphics pass。为了不把里程碑又拖回大而散的“全功能 UI renderer”，这一版的 text/image 先以稳定可见占位图形落地，同时通过 stats 报告 text/image payload 数量，给下一步 atlas/texture 接入保留明确的技术边界。

## Acceptance Fixtures

四个 builtin fixture 的角色已经固定：

- `HudOverlay`
  - 验证常驻 HUD、角标、文本和 icon/image payload 能进入 runtime frame
- `PauseMenu`
  - 验证中心对话框、scrim、按钮面板和可见 screen-space footprint
- `SettingsDialog`
  - 验证多面板配置 UI 能以纯 TOML 资产落成 shared surface
- `InventoryList`
  - 验证 scrollable/virtualized list 的 clip/scissor 统计链路

其中 `InventoryList` 的意义不是“目前已经完成完整虚拟化 renderer”，而是 runtime integration 这一步至少要证明 clipped draw command 会被保留并进入真实 graphics pass 的统计和 record 流程。

## Validation Evidence

本轮已经跑过并通过以下验证：

- `cargo test -p zircon_graphics --lib --locked --features runtime-ui-integration-tests runtime_ui_manager_builds_all_builtin_fixtures_into_shared_surfaces -- --nocapture`
- `cargo test -p zircon_graphics --lib --locked --features runtime-ui-integration-tests render_framework_submits_runtime_ui_frames_and_renders_pause_menu_panels -- --nocapture`
- `cargo test -p zircon_graphics --lib --locked --features runtime-ui-integration-tests render_framework_reports_clipped_ui_commands_for_inventory_fixture -- --nocapture`
- `cargo test -p zircon_graphics --lib --locked --features runtime-ui-integration-tests`
- `cargo test -p zircon_framework --lib --locked`

这里的 focused assertions 不只检查“函数可调用”，还检查了三类真实结果：

- manager 能把四个 `.ui.toml` fixture 构造成 non-trivial shared `UiSurface`
- render framework stats 会报告 UI command/quad/clip 数量
- headless capture 在暂停菜单场景下会出现可见中心对话框亮区，而不是只有背景清屏

这意味着 `Graphics/runtime integration` 已经从“API 存在”推进到了“shared runtime UI 能通过 graphics pass 真正出图并可验证”。
