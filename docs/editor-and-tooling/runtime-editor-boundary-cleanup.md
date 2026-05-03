---
related_code:
  - Cargo.toml
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/project/meta.rs
  - zircon_runtime/src/tests/scene_boundary/mod.rs
  - zircon_editor/src/ui/host/editor_asset_manager/mod.rs
  - zircon_editor/src/ui/host/editor_asset_manager/catalog.rs
  - zircon_editor/src/ui/host/editor_asset_manager/editor_meta.rs
  - zircon_editor/src/ui/host/editor_asset_manager/preview.rs
  - zircon_editor/src/ui/host/editor_asset_manager/reference_graph.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/project_sync/sync_from_project.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/default_editor_asset_manager/asset_details.rs
  - zircon_editor/src/tests/host/asset_metadata/runtime_sidecar_isolation.rs
  - zircon_editor/src/tests/host/asset_metadata/precedence.rs
  - zircon_editor/src/tests/host/asset_manager_boundary.rs
  - zircon_editor/src/tests/host/asset_metadata.rs
  - zircon_editor/src/lib.rs
  - zircon_editor/src/ui/host/mod.rs
  - zircon_editor/src/core/host/mod.rs
  - zircon_editor/src/ui/slint_host/app.rs
  - zircon_editor/src/scene/viewport/mod.rs
  - zircon_editor/src/scene/viewport/settings.rs
  - zircon_editor/src/scene/viewport/render_packet.rs
  - zircon_runtime/tests/m1_runtime_editor_boundary_contract.rs
  - zircon_editor/src/scene/viewport/handles/handle_tool.rs
  - zircon_editor/src/scene/viewport/handles/move_handle_tool_impl.rs
  - zircon_editor/src/scene/viewport/handles/rotate_handle_tool_impl.rs
  - zircon_editor/src/scene/viewport/handles/scale_handle_tool_impl.rs
  - zircon_editor/src/tests/editing/viewport.rs
  - zircon_editor/src/tests/host/render_framework_boundary.rs
  - zircon_editor/ui/workbench.slint
  - zircon_editor/ui/workbench/host_surface.slint
  - zircon_editor/ui/workbench/host_surface_contract.slint
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/overlay.rs
  - zircon_runtime/src/core/framework/scene/mod.rs
  - zircon_runtime/src/core/framework/scene/property_value.rs
  - zircon_runtime/src/graphics/mod.rs
  - zircon_runtime/src/graphics/types/mod.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
  - zircon_runtime/src/graphics/scene/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/icon_source/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/icon_source/viewport_icon_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_new/new_with_icon_source.rs
  - zircon_runtime/src/animation/mod.rs
  - zircon_runtime/src/animation/runtime/mod.rs
  - zircon_runtime/src/animation/sequence/mod.rs
  - zircon_runtime/src/physics/mod.rs
  - zircon_runtime/src/physics/runtime/mod.rs
  - zircon_runtime/src/scene/world/property_access.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/pending_completion/ordered_evictable_pages_for_target.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/foundation/runtime/config_path.rs
  - zircon_runtime/src/foundation/runtime/config_manager.rs
  - zircon_runtime/src/foundation/tests.rs
  - zircon_editor/src/tests/support.rs
  - zircon_editor/src/tests/editor_event/support.rs
  - zircon_editor/src/tests/host/manager/mod.rs
  - zircon_editor/src/tests/host/slint_drawer_resize.rs
  - zircon_editor/src/tests/host/ui_asset_editor_theme_tooling/mod.rs
  - zircon_editor/src/ui/slint_host/app/tests.rs
implementation_files:
  - Cargo.toml
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/project/meta.rs
  - zircon_editor/src/ui/host/editor_asset_manager/mod.rs
  - zircon_editor/src/ui/host/editor_asset_manager/catalog.rs
  - zircon_editor/src/ui/host/editor_asset_manager/editor_meta.rs
  - zircon_editor/src/ui/host/editor_asset_manager/preview.rs
  - zircon_editor/src/ui/host/editor_asset_manager/reference_graph.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/project_sync/sync_from_project.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/default_editor_asset_manager/asset_details.rs
  - zircon_editor/src/tests/host/asset_metadata/runtime_sidecar_isolation.rs
  - zircon_editor/src/tests/host/asset_metadata/precedence.rs
  - zircon_editor/src/lib.rs
  - zircon_editor/src/ui/host/mod.rs
  - zircon_editor/src/core/host/mod.rs
  - zircon_editor/src/ui/slint_host/app.rs
  - zircon_editor/src/scene/viewport/mod.rs
  - zircon_editor/src/scene/viewport/settings.rs
  - zircon_editor/src/scene/viewport/render_packet.rs
  - zircon_runtime/tests/m1_runtime_editor_boundary_contract.rs
  - zircon_editor/src/scene/viewport/handles/handle_tool.rs
  - zircon_editor/src/scene/viewport/handles/move_handle_tool_impl.rs
  - zircon_editor/src/scene/viewport/handles/rotate_handle_tool_impl.rs
  - zircon_editor/src/scene/viewport/handles/scale_handle_tool_impl.rs
  - zircon_editor/ui/workbench.slint
  - zircon_editor/ui/workbench/host_surface.slint
  - zircon_editor/ui/workbench/host_surface_contract.slint
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/overlay.rs
  - zircon_runtime/src/core/framework/scene/mod.rs
  - zircon_runtime/src/core/framework/scene/property_value.rs
  - zircon_runtime/src/graphics/mod.rs
  - zircon_runtime/src/graphics/types/mod.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
  - zircon_runtime/src/graphics/scene/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/icon_source/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/icon_source/viewport_icon_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_new/new_with_icon_source.rs
  - zircon_runtime/src/animation/mod.rs
  - zircon_runtime/src/animation/runtime/mod.rs
  - zircon_runtime/src/animation/sequence/mod.rs
  - zircon_runtime/src/physics/mod.rs
  - zircon_runtime/src/physics/runtime/mod.rs
  - zircon_runtime/src/scene/world/property_access.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/pending_completion/ordered_evictable_pages_for_target.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/foundation/runtime/config_path.rs
  - zircon_runtime/src/foundation/runtime/config_manager.rs
  - zircon_editor/src/tests/support.rs
  - zircon_editor/src/ui/slint_host/app/tests.rs
plan_sources:
  - user: 2026-04-20 继续，把 runtime 层仍然存在的 editor only 实现迁回 editor
  - user: 2026-04-20 继续
  - user: 2026-04-20 继续，runtime asset metadata / preview surface 里的 editor-only 实现也迁回 editor
  - user: 2026-04-20 你的侧重点是先恢复图形渲染相关的
  - .codex/plans/Runtime Core Fold-In And Compile Recovery.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
tests:
  - cargo check -p zircon_runtime --lib --offline --message-format short --target-dir target/codex-editor-software-renderer-check
  - cargo check -p zircon_editor --lib --offline --message-format short --target-dir target/codex-editor-software-renderer-check
  - cargo check -p zircon_editor --tests --offline --message-format short --target-dir target/ta
  - cargo test -p zircon_editor --lib editor_slint_host_prefers_winit_software_renderer_over_skia_wgpu_selection --offline --target-dir target/ta -j1 -- --nocapture
  - cargo test -p zircon_app entry_uses_runtime_owned_builtin_module_list_without_manual_graphics_insertion --offline --target-dir target/ta -j1 -- --nocapture
  - cargo test -p zircon_runtime --offline --target-dir target/codex-graphics-recovery render_framework_bridge -- --nocapture
  - cargo test -p zircon_runtime --offline --target-dir target/codex-graphics-recovery pipeline_compile -- --nocapture
  - cargo test -p zircon_runtime --offline --target-dir target/codex-graphics-recovery project_render -- --nocapture
  - cargo test -p zircon_runtime --offline --target-dir target/codex-graphics-recovery visibility -- --nocapture
  - cargo test -p zircon_runtime --offline --target-dir target/codex-graphics-recovery virtual_geometry_frontier_runtime -- --nocapture
  - cargo test -p zircon_runtime --offline --target-dir target/codex-graphics-recovery --no-run
  - cargo check -p zircon_runtime --lib --offline --target-dir target/codex-graphics-recovery
  - cargo test -p zircon_editor --offline --target-dir target/codex-graphics-recovery viewport -- --nocapture
  - cargo check -p zircon_editor --lib --message-format short --target-dir target/codex-editor-asset-meta-check
  - cargo check -p zircon_runtime --lib --message-format short --target-dir target/codex-runtime-asset-meta-check
  - cargo check -p zircon_editor --tests --message-format short --target-dir target/codex-editor-asset-meta-tests-check
  - cargo check -p zircon_runtime --tests --message-format short --target-dir target/codex-runtime-asset-meta-tests-check
  - cargo test -p zircon_runtime runtime_asset_surface_keeps_project_and_watch_under_namespaces --target-dir target/codex-runtime-asset-meta-tests-check -- --nocapture
  - cargo check --locked -p zircon_runtime --lib --message-format short --target-dir target/codex-runtime-graphics-surface-check
  - cargo check --locked -p zircon_runtime --tests --message-format short --target-dir target/codex-runtime-graphics-surface-check
  - cargo test --locked --message-format short -p zircon_runtime --lib graphics_surface_keeps_viewport_frame_and_icon_source_internal --target-dir target/codex-runtime-graphics-surface-test -- --nocapture
  - cargo check --locked -p zircon_editor --lib --message-format short --target-dir target/codex-editor-lib-check-isolated
  - cargo check --locked -p zircon_runtime --lib --message-format short --target-dir target/codex-runtime-scene-reflection-check
  - cargo check --locked -p zircon_runtime --tests --message-format short --target-dir target/codex-runtime-scene-reflection-check
  - cargo test --locked --message-format short -p zircon_runtime --lib runtime_scene_property_reflection_stays_internal --target-dir target/codex-runtime-scene-reflection-test -- --nocapture
  - cargo check --locked -p zircon_editor --lib --message-format short --target-dir target/codex-editor-lib-check-scene-proof
  - cargo test -p zircon_editor --lib asset_metadata --target-dir target/codex-editor-asset-meta-tests-check -- --nocapture
  - cargo check -p zircon_runtime --locked --lib --target-dir F:\cargo-targets\zircon-codex-a
  - cargo test -p zircon_runtime --test m1_runtime_editor_boundary_contract --locked --target-dir F:\cargo-targets\zircon-codex-a
  - Select-String scan over zircon_runtime/src, zircon_editor/src, zircon_app/src for legacy viewport authoring imports and config-path names
  - cargo test -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-workspace-followup-opencode-build --message-format short --color never -- --test-threads=1
  - cargo test -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-workspace-followup-opencode-build --message-format short --color never
  - cargo test --workspace --locked --verbose --jobs 1 --target-dir E:\cargo-targets\zircon-workspace-followup-opencode-build --message-format short --color never
doc_type: module-detail
---

# Runtime/Editor Boundary Cleanup

## Purpose

这份文档记录 2026-04-20 这一轮继续收 runtime/editor 边界时落下的三刀：

- 把 viewport authoring contract 从 runtime render contract 迁回 `zircon_editor`
- 把 runtime foundation 里残留的 editor-only config path 命名改成中性 runtime host config
- 把 graphics root surface 上伪装成公共 API 的内部 frame/icon seam 收回到 `zircon_runtime::graphics` 内部

目标不是再造一层兼容桥，而是直接删掉 runtime 里对 editor 作者态的默认假设。

## Graphics Compile Recovery

### Software Renderer Baseline

这轮 graphics-first recovery 明确把 editor host 的 Slint 运行时固定到了 `winit + software`：

- [Cargo.toml](/E:/Git/ZirconEngine/Cargo.toml) 里的 workspace `slint` features 已切成 `backend-winit + renderer-software + unstable-winit-030`
- [app.rs](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app.rs) 在 `run_editor()` 中显式调用 `.backend_name("winit".into())` 和 `.renderer_name("software".into())`
- [render_framework_boundary.rs](/E:/Git/ZirconEngine/zircon_editor/src/tests/host/render_framework_boundary.rs) 新增源码边界断言，持续拦截 `renderer-skia`、`unstable-wgpu-27` 和任何 `wgpu::` 直接回流到 editor host

这样做的目标不是长期放弃 GPU 渲染，而是在当前吸收后的目录结构里先恢复 editor/runtime 的图形编译闭环。运行时图形能力仍然留在 `zircon_runtime::graphics` 的 RenderFramework/SRP 主链里；Slint host 只是切回了一个更稳的测试与 authoring baseline。

### Slint Host Binding Recovery

[workbench.slint](/E:/Git/ZirconEngine/zircon_editor/ui/workbench.slint) 现在把 `UiHostWindow.host_presentation` 只作为 root 输入，再单向传给内部 `WorkbenchHostScaffold`：

- `UiHostWindow` 保留 `in property <HostWindowPresentationData> host_presentation`
- `host := WorkbenchHostScaffold { host_presentation: root.host_presentation; }`
- 不再把 `host_presentation` 通过 `<=>` 双向别名回自身

这消除了此前测试构建里出现的 `host_presentation` 自绑定环。当前 [host_surface.slint](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_surface.slint) 和 [host_surface_contract.slint](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_surface_contract.slint) 也已经收敛成单纯的 presentation-to-surface 数据投影，不再把临时 surface 输出属性当作 host 自身状态来源。

### Current Host Ownership Shape

这轮恢复还顺手把文档里的 owner 认知追上了当前源码现实：

- [zircon_editor/src/ui/host/mod.rs](/E:/Git/ZirconEngine/zircon_editor/src/ui/host/mod.rs) 才是 editor host manager internals 的真实 owner
- 这里集中声明了 `editor_manager`、`project_access`、`layout_hosts`、`window_host_manager`、`asset_editor_sessions` 等 orchestration 子域
- [zircon_editor/src/core/host/mod.rs](/E:/Git/ZirconEngine/zircon_editor/src/core/host/mod.rs) 现在只保留更窄的 shared host surface：`asset_editor`、`module`、`resource_access`
- 旧的 `zircon_editor/src/core/host/manager.rs` 路径已经不再是实现 owner，相关 boundary tests 也已改成跟随新 owner 读取 [project_access.rs](/E:/Git/ZirconEngine/zircon_editor/src/ui/host/project_access.rs) 这类实际文件

这点很关键，因为 compile recovery 不能靠“恢复旧路径”达成；当前 absorbed 目录结构下，正确做法是沿着 `ui::host` 的新 owner 继续收边界，而不是把实现搬回历史位置。

### Validation Evidence

当前这条 compile recovery 链已经有一组可重复的本地证据：

- `cargo check -p zircon_runtime --lib --offline --message-format short --target-dir target/codex-editor-software-renderer-check`
- `cargo check -p zircon_editor --lib --offline --message-format short --target-dir target/codex-editor-software-renderer-check`
- `cargo check -p zircon_editor --tests --offline --message-format short --target-dir target/ta`
- `cargo test -p zircon_editor --lib editor_slint_host_prefers_winit_software_renderer_over_skia_wgpu_selection --offline --target-dir target/ta -j1 -- --nocapture`
- `cargo test -p zircon_app entry_uses_runtime_owned_builtin_module_list_without_manual_graphics_insertion --offline --target-dir target/ta -j1 -- --nocapture`

这些检查证明了三件事：

- editor host 可以在当前吸收后的 crate/目录布局上重新编译
- editor host 的源码边界仍然没有回退到直接依赖 `wgpu`
- app 入口仍然使用 runtime-owned builtin module 清单，而不是人工把旧 graphics surface 塞回入口层

## Viewport Authoring Contract

### Editor Ownership

`zircon_editor::scene::viewport` 现在是 viewport authoring 类型的唯一 owner：

- `SceneViewportTool`
- `TransformSpace`
- `ViewOrientation`
- `GridMode`
- `SceneViewportSettings`

这些定义集中在 [settings.rs](/E:/Git/ZirconEngine/zircon_editor/src/scene/viewport/settings.rs)，并由 [mod.rs](/E:/Git/ZirconEngine/zircon_editor/src/scene/viewport/mod.rs) 作为 editor 内部统一入口导出。`zircon_editor/src/lib.rs` 也把这组 editor-owned 类型重新挂回 editor crate root，避免外部代码继续从 runtime contract 取作者态枚举。

### Runtime Contract After Cut

`zircon_runtime::core::framework::render` 现在只保留中性渲染 contract：

- `ProjectionMode`
- `DisplayMode`
- `ViewportCameraSnapshot`
- `ViewportRenderSettings`
- `SceneViewportExtractRequest`

runtime world 只消费 `ViewportRenderSettings` 投影出来的四个渲染相关字段：

- projection mode
- display mode
- preview lighting
- preview skybox

tool、transform space、view orientation、grid mode、gizmo 开关和 snap 步长不再属于 runtime render contract。

### Overlay Narrowing

[overlay.rs](/E:/Git/ZirconEngine/zircon_runtime/src/core/framework/render/overlay.rs) 里的 `HandleOverlayExtract` 也进一步收窄了：

- 删除 `tool`
- 删除 `space`

graphics 只需要 handle 的 origin 和 element 列表来渲染，不需要继续背着 editor authoring 元信息。真正需要 `tool/space` 的逻辑现在留在 editor handle drag session 和 viewport state 内部。

### Editor Import Contraction

`zircon_editor` 内部不再到处直接引用 `zircon_runtime::core::framework::render::*` 来拿 viewport 作者态类型。当前收束方式是：

- editor authoring 类型从 `crate::scene::viewport::*` 进入
- editor 仍然消费的 runtime render DTO，也通过同一个本地 viewport façade 进入

这样 editor 内部的调用面不再直接耦合 runtime render module 的具体路径，后续继续收边界时也更容易替换。

### Compile-Recovery Follow-Through

这一轮 graphics-first compile recovery 又把两个容易回流的边界点钉死了：

- `SceneViewportController` 侧的 render packet 现在统一走 `SceneViewportSettings::render_settings()`，而不是再假设 `ViewportRenderSettings::from(&SceneViewportSettings)` 这种“runtime contract 持有 editor authoring settings”的隐式转换。
- move / rotate / scale handle builders 现在都只构造 `HandleOverlayExtract { owner, origin, elements }`。`tool` 和 `space` 已经完全留在 editor 的 viewport state / drag session 内，不再残留为 shared render DTO 的半废字段。
- 对应地，`HandleTool` trait 里那个只为旧 DTO 服务的 `tool()` hook 也删掉了，避免 editor handle implementation 继续背一个没有读者的 contract。
- `zircon_editor/src/lib.rs` 的 viewport re-export 也拆成了显式的 interaction boundary 行：`GizmoAxis / ViewportFeedback / ViewportInput / ViewportState` 单独公开，确保 `editor_viewport_interaction_boundary_lives_in_editor_crate` 这条 boundary test 会继续拦住后续错误回流。
- `VirtualGeometryRuntimeState` 里上一轮 frontier merge 修复遗留下来的 `page_is_frontier_hot()` 也已经删掉，避免 runtime graphics recovery 的真实 warning 再被历史 helper 噪声淹没；相应的 frontier 行为则继续由 `page_or_lineage_is_hot()` / `frontier_hot_resident_pages()` 这对当前 helper 提供。

这几刀的共同目标不是“把测试喂绿”，而是让当前三层职责更清楚：

- editor 负责 tool / transform space / interaction state
- runtime framework render 负责 neutral viewport render DTO
- graphics 只消费 DTO 和 scene extract，不再拥有 editor authoring 元信息

### M1 Boundary Regression Guards

这一轮又补了一条更贴近 `M1` 的 contract 回归：[m1_runtime_editor_boundary_contract.rs](/E:/Git/ZirconEngine/zircon_runtime/tests/m1_runtime_editor_boundary_contract.rs) 直接锁了五件事：

- [render.rs](/E:/Git/ZirconEngine/zircon_runtime/src/scene/world/render.rs) 里的 runtime world render extract 只能生成空作者态 overlay 默认值，继续只把 `display_mode` 当作宿主可控渲染设置透传。
- runtime world 的默认 extract 入口继续把 `active_camera_override` 和 `camera` 置空，行为级回归也会证明 request 里真正允许生效的只有 `projection_mode`、`display_mode`、`preview_lighting`、`preview_skybox` 这些中性字段。
- [render_packet.rs](/E:/Git/ZirconEngine/zircon_editor/src/scene/viewport/render_packet.rs) 仍然是 selection / selection anchor / grid / handle / scene gizmo 这些作者态 overlay 的唯一默认装配点。
- editor viewport packet 继续独占 `camera: Some(camera.clone())` 以及 `preview_lighting` / `preview_skybox` 的作者态入口，runtime world 不会默认注入这类 editor-owned 状态。
- [`viewport_authoring_commands_do_not_mutate_runtime_world_or_default_extract`](/E:/Git/ZirconEngine/zircon_editor/src/tests/editing/state.rs) 现在又把 state 层边界钉住了一层：切换 projection / display / grid / preview lighting / preview skybox / gizmos 只会改变 editor packet，不会回写 runtime world，也不会改变 runtime world 自己的默认 render extract。
- runtime 生产源码里，`SelectionHighlightExtract`、`SelectionAnchorExtract`、`GridOverlayExtract`、`HandleOverlayExtract` 的构造不允许重新回流；对应构造点继续锁在 editor viewport 和 handle tool 实现里。
- `SceneGizmoOverlayExtract` 在 runtime 侧只保留一个明确例外：用于 runtime virtual-geometry 调试提交的 [build_runtime_frame.rs](/E:/Git/ZirconEngine/zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs)。这不是 editor authoring 回流，而是 runtime debug DTO 生产。
- neutral overlay DTO 现在有了一条实际渲染回归：即使没有 runtime world authoring 上下文，graphics 也必须能直接消费 `RenderSceneSnapshot + RenderOverlayExtract` 并把场景 gizmo 画出来。

为了让这条集成测试链保持可跑，这轮还顺手修了一个与 `M1` 无关但会挡住验证的 compile drift：[render_frame_with_pipeline.rs](/E:/Git/ZirconEngine/zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs) 重新把 `visbuffer64` source 参数按现有 `store_last_runtime_outputs(...)` 签名传回去，避免 runtime library 在 unrelated virtual-geometry 统计路径上卡死。

## Foundation Config Path Neutralization

### Problem

[config_path.rs](/E:/Git/ZirconEngine/zircon_runtime/src/foundation/runtime/config_path.rs) 之前把 runtime foundation 的默认配置落盘路径固定成 editor 语义：

- 环境变量：`ZIRCON_EDITOR_CONFIG_PATH`
- 默认文件：`editor-config.json`
- 当前目录回退：`.zircon-editor-config.json`

这让 runtime 自己决定了一个 editor-only 文件名，而 `ConfigManager` 实际上是 foundation 级公共服务，不只服务 editor。

### Current Behavior

runtime foundation 现在统一使用中性命名：

- 环境变量：`ZIRCON_CONFIG_PATH`
- 平台默认文件：`config.json`
- 当前目录回退：`.zircon-config.json`

`DefaultConfigManager` 的行为没有变：

- 仍在启动时从磁盘加载 key/value config
- 仍在每次 `set_value` 后持久化整个快照

变的是 runtime 不再在文件命名上内建 `editor` 语义。

### Test Injection Update

对应的 runtime/editor 测试注入也同步切到 `ZIRCON_CONFIG_PATH`。这样 editor host 测试仍然可以用临时文件隔离配置状态，但基础设施层不再把这件事写成 editor 专属能力。

所有会写 `ZIRCON_CONFIG_PATH` 的 editor crate tests 必须通过 [zircon_editor/src/tests/support.rs](/E:/Git/ZirconEngine/zircon_editor/src/tests/support.rs) 暴露的共享 `env_lock()` 串行化。这个锁是进程级环境变量的唯一测试保护层；子模块不能自建另一个 `Mutex` 来保护同一个变量，否则 Rust test harness 的默认并行执行仍会让两个测试同时切换 config path。

2026-05-03 的 workspace acceptance 复现了这个边界问题：`tests::workbench::reflection::action_dispatch::workbench_reflection_call_action_dispatches_docking_inspector_and_viewport_actions` 单测独立运行通过，`zircon_editor --lib -- --test-threads=1` 也通过，但默认并行 `zircon_editor --lib` 会偶发拿到错误的 config path。最低共享层不是 reflection route，而是 [zircon_editor/src/ui/slint_host/app/tests.rs](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/tests.rs) 里独立的本地 env lock。该 test module 现在改为调用 crate-wide `crate::tests::support::env_lock()`，并用默认并行 editor-lib 与 workspace test 重新验证。

## Graphics Root-Surface Contraction

### Internal Frame Carrier Only

`ViewportRenderFrame` 现在仍然存在，但只作为 `zircon_runtime::graphics` 内部 frame carrier：

- `zircon_runtime::graphics::types::ViewportRenderFrame` 改成 crate-private re-export
- `zircon_runtime::graphics` 与 `zircon_runtime/src/lib.rs` 都不再把它挂到公开 root-surface
- runtime UI、hybrid GI、virtual geometry 继续把 payload 叠在同一个内部 carrier 上，但这些准备态不再伪装成对外 graphics API

这意味着 graphics 对外暴露的仍然是 `SceneRenderer`、`WgpuRenderFramework`、`ViewportFrame` 这类真正的运行期入口，而不是 editor/runtime 共用的中间帧拼装结构。

### Internal Icon Source Seam

`ViewportIconSource` 也同步收回 graphics 内部：

- trait 定义改成 crate-private
- `zircon_runtime::graphics`、`zircon_runtime::graphics::scene`、`scene_renderer` 这几层已经不再 re-export 它；只剩 `overlay` / `icon_source` 子域保留 crate-private handoff
- `SceneRenderer::new_with_icon_source(...)` 也改成 crate-private，避免 runtime root 继续暴露一个只有 graphics 自己使用的 icon seam

这样 editor 与 runtime 对 scene gizmo icon 的交互边界就只剩 render packet DTO 里的 `ViewportIconId`。如果将来真的需要 editor host 提供图标字节源，也应该通过明确的 editor-owned attach/host seam 落地，而不是重新把 `ViewportIconSource` 挂回 runtime public API。

### Validation

这一刀当前有三层证据：

- `cargo check --locked -p zircon_runtime --lib --message-format short --target-dir target/codex-runtime-graphics-surface-check`
- `cargo check --locked -p zircon_runtime --tests --message-format short --target-dir target/codex-runtime-graphics-surface-check`
- `cargo test --locked --message-format short -p zircon_runtime --lib graphics_surface_keeps_viewport_frame_and_icon_source_internal --target-dir target/codex-runtime-graphics-surface-test -- --nocapture`

另外，`cargo check --locked -p zircon_editor --lib --message-format short --target-dir target/codex-editor-lib-check-isolated` 也已经通过，说明这轮 runtime graphics public-surface 收窄没有把 editor 下游编译面打断。

## Asset Editor Metadata And Preview Surface

### Runtime Meta Contraction

`zircon_runtime::asset::project::AssetMetaDocument` 现在只保留 runtime 真正拥有的 project/import/preview 字段：

- `asset_uuid`
- `primary_locator`
- `kind`
- `import_settings`
- `source_mtime_unix_ms`
- `source_hash`
- `preview_state`

原先混在这里的 `editor_adapter` 已删除。runtime 继续负责 asset registry、import pipeline 和 preview state 持久化，但不再定义“这个 asset 该用哪个 editor adapter 打开”。

### Editor-Owned Sidecar

`zircon_editor::ui::host::editor_asset_manager` 新增了 editor-owned metadata sidecar：

- 类型 owner: `EditorAssetMetaDocument`
- 文件路径: `foo.ext.editor.meta.toml`

editor asset host 在 `sync_from_project()` 里现在同时读取两份文档：

- runtime meta: `foo.ext.meta.toml`
- editor meta: `foo.ext.editor.meta.toml`

`foo.ext.editor.meta.toml` 是 editor adapter 选择的唯一持久化来源。若 editor sidecar 不存在，editor 返回默认 editor metadata；不会再从 runtime meta 导入 `editor_adapter`，也不会自动生成 sidecar。这样运行时 `.meta.toml` 只承载 runtime/import/preview 字段，编辑器发行和运行时发行之间不会通过历史字段互相污染。

### Runtime Root-Surface Cut

这轮同时把 runtime asset root 上残留的 editor-only preview/catalog surface 一起切掉了：

- 删除 `zircon_runtime::asset::editor`
- 删除 runtime root 对这些 editor 类型的 re-export：
  - `AssetCatalogRecord`
  - `PreviewArtifactKey`
  - `PreviewCache`
  - `PreviewPalette`
  - `PreviewScheduler`
  - `ReferenceGraph`

这些类型现在只由 `zircon_editor::ui::host::editor_asset_manager` 持有。runtime 侧只保留 asset namespace、pipeline、project 和 watch。

### Validation Notes

这一刀的 compile/test 证据分成两层：

- 代码与测试编译面已通过：
  - `cargo check -p zircon_editor --lib --message-format short --target-dir target/codex-editor-asset-meta-check`
  - `cargo check -p zircon_runtime --lib --message-format short --target-dir target/codex-runtime-asset-meta-check`
  - `cargo check -p zircon_editor --tests --message-format short --target-dir target/codex-editor-asset-meta-tests-check`
  - `cargo check -p zircon_runtime --tests --message-format short --target-dir target/codex-runtime-asset-meta-tests-check`
  - `cargo test -p zircon_runtime runtime_asset_surface_keeps_project_and_watch_under_namespaces --target-dir target/codex-runtime-asset-meta-tests-check -- --nocapture`
- `cargo test -p zircon_editor --lib asset_metadata --target-dir target/codex-editor-asset-meta-tests-check -- --nocapture` 没能执行到测试体本身：
  - `skia-bindings` 在 test profile 下尝试下载预编译包
  - 当前环境无法解析 `release-assets.githubusercontent.com`
  - 本机也没有可供 fallback full build 使用的 LLVM / `clang-cl`

因此这一轮可以确认 boundary 和 test compile 已正确，但 editor test run 还受本机图形依赖环境限制。

## Scene Property Reflection Contraction

### Problem

`zircon_runtime::scene::world::property_access` 里原先还挂着一组明显偏 authoring/reflection 的公共面：

- `ScenePropertyEntry`
- `World::property_entries()`

这组能力会把“枚举一个节点可编辑属性列表”的职责留在 runtime shared surface 上，但当前 tracked 生产代码里并没有 runtime/app/editor 的直接调用点。真正还在被 runtime 生产逻辑使用的只有 animation property track 需要的最小集合：

- `entity_path()`
- `resolve_entity_path()`
- `property()`
- `set_property()`
- `ScenePropertyValue`

### Current Behavior

这轮把 authoring reflection 残留从 runtime public API 收回了：

- `core::framework::scene::mod.rs` 不再公开 re-export `ScenePropertyEntry`
- `ScenePropertyEntry` 改成 crate-private
- `World::property_entries()` 改成 world 内部 helper，不再作为 runtime public method 暴露

这样 runtime 继续保留 animation/runtime 必需的 property mutation contract，但不再把“列出所有可编辑属性”的 editor 视角结构写进 shared framework public surface。

### Validation

- `cargo check --locked -p zircon_runtime --lib --message-format short --target-dir target/codex-runtime-scene-reflection-check`
- `cargo check --locked -p zircon_runtime --tests --message-format short --target-dir target/codex-runtime-scene-reflection-check`
- `cargo test --locked --message-format short -p zircon_runtime --lib runtime_scene_property_reflection_stays_internal --target-dir target/codex-runtime-scene-reflection-test -- --nocapture`

其中 [`zircon_runtime/src/tests/scene_boundary/mod.rs`](/E:/Git/ZirconEngine/zircon_runtime/src/tests/scene_boundary/mod.rs) 新增了源码边界断言，专门拦住 `ScenePropertyEntry` 和 `property_entries()` 再次回流成 runtime public surface。

另外下游 editor 证明编译也做了额外尝试：

- `cargo check --locked -p zircon_editor --lib --message-format short --target-dir target/codex-editor-lib-check-scene-proof`

这次失败不是 boundary 符号回流，而是本机环境噪声：

- `link.exe` 返回 `1180`
- `link.exe` 返回 `1140`
- 随后出现 `os error 112`，提示磁盘空间不足

对应日志保存在 `target/codex-editor-lib-check-scene-proof/editor-check.log`。因此当前可以确认 runtime 侧 public-surface 收缩已经通过自身 compile/test gate；editor 下游额外验证暂时被本机链接器/磁盘环境阻塞，而不是这次 scene property contract 改动本身打断。

## Current Boundary Result

这一轮之后，runtime 非测试生产代码里已经看不到这些旧残留：

- `SceneViewportTool`
- `TransformSpace`
- `ViewOrientation`
- `GridMode`
- `SceneViewportSettings`
- `ZIRCON_EDITOR_CONFIG_PATH`
- `editor-config.json`
- `editor_adapter` in `AssetMetaDocument`
- `zircon_runtime::asset::editor`
- runtime root re-exports of editor asset preview/catalog helpers
- runtime public re-exports of `ViewportRenderFrame`
- runtime public re-exports of `ViewportIconSource`
- public `SceneRenderer::new_with_icon_source(...)`
- public `ScenePropertyEntry`
- public `World::property_entries()`

当前继续留在 runtime 里的 overlay/gizmo/selection 相关结构，被视为中性 render packet DTO：

- runtime world 只生成空 overlay 或基础 display/preview extract
- editor 基于 runtime world + editor state 组装 selection/grid/gizmo/handle overlay
- graphics 只消费 DTO，不生成 editor authoring state

## Remaining Watchlist

这一轮扫描后，`zircon_runtime` 生产代码里新的强证据 editor-only 实现点已经继续减少。剩余需要继续盯的主要是：

- unit tests 的 key 名或事件 topic
- fixture 资源 id / display name
- 文档里对旧 boundary 的历史描述
- editor crate 自己对 `PreviewCache` / `ReferenceGraph` 的 root re-export 是否还可以继续内收

这些大多已经不是 runtime 继续持有 editor-only 生产实现的问题，而是清扫余波；后续可以继续按 docs/tests/root-surface fanout 收窄。
