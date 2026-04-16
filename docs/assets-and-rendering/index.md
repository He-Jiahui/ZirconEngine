---
related_code:
  - zircon_resource/src/lib.rs
  - zircon_resource/src/locator.rs
  - zircon_resource/src/handle.rs
  - zircon_resource/src/record.rs
  - zircon_resource/src/manager.rs
  - zircon_asset/src/project/manifest.rs
  - zircon_asset/src/project/paths.rs
  - zircon_asset/src/project/manager.rs
  - zircon_asset/src/pipeline/manager.rs
  - zircon_asset/src/watch.rs
  - zircon_manager/src/lib.rs
  - zircon_scene/src/components.rs
  - zircon_scene/src/world/bootstrap.rs
  - zircon_scene/src/world/project_io.rs
  - zircon_scene/src/module.rs
  - zircon_graphics/src/scene/resources/mod.rs
  - zircon_graphics/src/scene/scene_renderer/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/mod.rs
  - zircon_graphics/src/scene/scene_renderer/deferred/mod.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/mod.rs
  - zircon_graphics/src/scene/scene_renderer/particle/mod.rs
  - zircon_graphics/src/scene/scene_renderer/overlay.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/viewport_overlay_renderer/mod.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/passes/base_scene_pass.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/passes/mod.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/icons/viewport_icon_atlas.rs
  - zircon_graphics/src/scene/scene_renderer/primitives/mod.rs
  - zircon_graphics/src/service/mod.rs
  - zircon_graphics/src/types.rs
  - zircon_graphics/src/runtime/mod.rs
  - zircon_graphics/src/runtime/offline_bake.rs
  - zircon_graphics/src/runtime/hybrid_gi.rs
  - zircon_graphics/src/runtime/virtual_geometry.rs
  - zircon_graphics/src/backend/render_backend/read_buffer_u32s.rs
  - zircon_graphics/src/runtime/server/mod.rs
  - zircon_graphics/src/runtime/server/create_viewport.rs
  - zircon_graphics/src/runtime/server/viewport_record.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/mod.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_readback.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/mod.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_readback.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_resources.rs
  - zircon_graphics/src/pipeline/mod.rs
  - zircon_graphics/src/feature/mod.rs
  - zircon_graphics/src/material/mod.rs
  - zircon_graphics/src/shader/mod.rs
  - zircon_graphics/src/visibility/mod.rs
  - zircon_graphics/src/visibility/context/from_extract_with_history.rs
  - zircon_graphics/src/visibility/declarations/visibility_hybrid_gi_probe.rs
  - zircon_graphics/src/visibility/declarations/visibility_hybrid_gi_feedback.rs
  - zircon_graphics/src/visibility/declarations/visibility_hybrid_gi_update_plan.rs
  - zircon_graphics/src/visibility/planning/build_hybrid_gi_plan.rs
  - zircon_graphics/src/extract/mod.rs
  - zircon_rhi/src/lib.rs
  - zircon_rhi_wgpu/src/lib.rs
  - zircon_render_graph/src/lib.rs
  - zircon_render_server/src/lib.rs
  - zircon_scene/src/render_extract.rs
  - zircon_editor/src/workbench/project/mod.rs
  - zircon_editor/src/host/app.rs
  - zircon_editor/src/host/slint_host/app.rs
  - zircon_editor/src/editing/command.rs
  - zircon_editor/src/editing/state/mod.rs
implementation_files:
  - zircon_resource/src/lib.rs
  - zircon_resource/src/locator.rs
  - zircon_resource/src/handle.rs
  - zircon_resource/src/record.rs
  - zircon_resource/src/manager.rs
  - zircon_asset/src/project/manifest.rs
  - zircon_asset/src/project/paths.rs
  - zircon_asset/src/project/manager.rs
  - zircon_asset/src/pipeline/manager.rs
  - zircon_asset/src/watch.rs
  - zircon_manager/src/lib.rs
  - zircon_scene/src/components.rs
  - zircon_scene/src/world/bootstrap.rs
  - zircon_scene/src/world/project_io.rs
  - zircon_scene/src/module.rs
  - zircon_graphics/src/scene/resources/mod.rs
  - zircon_graphics/src/scene/scene_renderer/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/mod.rs
  - zircon_graphics/src/scene/scene_renderer/deferred/mod.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/mod.rs
  - zircon_graphics/src/scene/scene_renderer/particle/mod.rs
  - zircon_graphics/src/scene/scene_renderer/overlay.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/viewport_overlay_renderer/mod.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/passes/base_scene_pass.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/passes/mod.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/icons/viewport_icon_atlas.rs
  - zircon_graphics/src/scene/scene_renderer/primitives/mod.rs
  - zircon_graphics/src/types.rs
  - zircon_graphics/src/runtime/mod.rs
  - zircon_graphics/src/runtime/offline_bake.rs
  - zircon_graphics/src/runtime/hybrid_gi.rs
  - zircon_graphics/src/runtime/virtual_geometry.rs
  - zircon_graphics/src/backend/render_backend/read_buffer_u32s.rs
  - zircon_graphics/src/runtime/server/mod.rs
  - zircon_graphics/src/runtime/server/create_viewport.rs
  - zircon_graphics/src/runtime/server/viewport_record.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/mod.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_readback.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/mod.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_readback.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_resources.rs
  - zircon_graphics/src/pipeline/mod.rs
  - zircon_graphics/src/feature/mod.rs
  - zircon_graphics/src/material/mod.rs
  - zircon_graphics/src/shader/mod.rs
  - zircon_graphics/src/visibility/mod.rs
  - zircon_graphics/src/visibility/context/from_extract_with_history.rs
  - zircon_graphics/src/visibility/declarations/visibility_hybrid_gi_probe.rs
  - zircon_graphics/src/visibility/declarations/visibility_hybrid_gi_feedback.rs
  - zircon_graphics/src/visibility/declarations/visibility_hybrid_gi_update_plan.rs
  - zircon_graphics/src/visibility/planning/build_hybrid_gi_plan.rs
  - zircon_graphics/src/extract/mod.rs
  - zircon_rhi/src/lib.rs
  - zircon_rhi_wgpu/src/lib.rs
  - zircon_render_graph/src/lib.rs
  - zircon_render_server/src/lib.rs
  - zircon_scene/src/render_extract.rs
  - zircon_editor/src/workbench/project/mod.rs
  - zircon_editor/src/host/app.rs
  - zircon_editor/src/host/slint_host/app.rs
  - zircon_editor/src/editing/command.rs
  - zircon_editor/src/editing/state/mod.rs
plan_sources:
  - user: 2026-04-13 实现目录式 Project 资源抽象优先全链路替换计划
  - user: 2026-04-16 implement Zircon SRP/RHI Rendering Architecture Roadmap
  - .codex/plans/全系统重构方案.md
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-16-m5-virtual-geometry-runtime-host.md
  - docs/superpowers/plans/2026-04-16-m5-virtual-geometry-prepare-consumption.md
  - docs/superpowers/plans/2026-04-16-m5-virtual-geometry-feedback-streaming.md
  - docs/superpowers/plans/2026-04-16-m5-hybrid-gi-runtime-host.md
  - docs/superpowers/plans/2026-04-16-m5-hybrid-gi-feedback-streaming.md
  - docs/superpowers/plans/2026-04-17-m5-virtual-geometry-gpu-uploader-readback.md
  - docs/superpowers/plans/2026-04-17-m5-hybrid-gi-gpu-completion-source.md
  - docs/superpowers/plans/2026-04-17-m5-hybrid-gi-radiance-cache-lighting-resolve.md
tests:
  - zircon_resource/src/tests.rs
  - zircon_asset/src/tests/pipeline/manager.rs
  - zircon_scene/src/lib.rs
  - zircon_graphics/src/tests/project_render.rs
  - zircon_rhi/src/tests/capabilities.rs
  - zircon_rhi/src/tests/descriptors.rs
  - zircon_rhi_wgpu/src/tests.rs
  - zircon_render_graph/src/tests/ordering.rs
  - zircon_render_graph/src/tests/cycles.rs
  - zircon_render_server/src/tests.rs
  - zircon_scene/tests/render_frame_extract.rs
  - zircon_graphics/src/tests/render_server_bridge.rs
  - zircon_graphics/src/tests/virtual_geometry_gpu.rs
  - zircon_graphics/src/tests/hybrid_gi_visibility.rs
  - zircon_graphics/src/tests/hybrid_gi_runtime.rs
  - zircon_graphics/src/tests/hybrid_gi_gpu.rs
  - zircon_graphics/src/tests/hybrid_gi_resolve_render.rs
  - zircon_editor/src/lib.rs
  - cargo test -p zircon_resource -p zircon_asset -p zircon_scene -p zircon_graphics -p zircon_editor
  - cargo test -p zircon_rhi --lib --tests
  - cargo test -p zircon_rhi_wgpu --lib --tests
  - cargo test -p zircon_render_graph --lib --tests
  - cargo test -p zircon_render_server --lib --tests
  - cargo test -p zircon_scene render_frame_extract_adapter_preserves_legacy_snapshot_content
  - cargo test -p zircon_graphics render_server_tracks_viewports_and_accepts_frame_extract_submission
  - cargo test --workspace --locked
doc_type: category-index
---

# Assets And Rendering

## Purpose

本目录记录目录式项目、资源抽象层、导入/监听、场景实例化、prepare/cache 渲染链路以及 editor 视口联动这条主链的实现约束。

## Documents

- [Directory Project Asset Rendering](./directory-project-asset-rendering.md): `zircon_resource` locator/handle/state 契约，`Project/assets` 与 `Project/library` 的职责，`res://`/`lib://`/`builtin://`/`mem://` 统一来源，`AssetManager`/`ResourceManager`/`EditorAssetManager`、`SceneAssetSerializer`、`LevelManager -> LevelSystem -> World` 与 graphics revision cache 的自动刷新路径。
- [SRP RHI Render Server Architecture](./srp-rhi-render-server-architecture.md): `zircon_rhi`、`zircon_rhi_wgpu`、`zircon_render_graph`、`zircon_render_server` 的基础边界，`RenderFrameExtract` 的新公共面，`zircon_graphics` 当前的 render server 兼容桥，以及 M4 已经真正落地的 deferred、clustered lighting、SSAO、history、bloom、color grading、reflection probe、baked lighting、particle 与 offline bake baseline；同时记录 M5 的 `Virtual Geometry / Hybrid GI` capability-slot 边界，以及 Virtual Geometry 从 preprocess 到 GPU completion/hierarchy refine、Hybrid GI 从 preprocess 到 GPU completion/radiance-cache lighting resolve 的基线。
- [M5 Virtual Geometry Prepare Consumption Plan](../superpowers/plans/2026-04-16-m5-virtual-geometry-prepare-consumption.md): `VirtualGeometryRuntimeState` 如何生成 frame-local prepare snapshot，`submit_frame_extract(...)` 如何在 render 前挂接它，以及当前 mesh fallback 如何在 feature 开启时 honor 这个 prepare 结果。
- [M5 Virtual Geometry Feedback Streaming Plan](../superpowers/plans/2026-04-16-m5-virtual-geometry-feedback-streaming.md): `VisibilityVirtualGeometryFeedback` 如何在帧后驱动 runtime host 消费 pending request、回收 evictable page，并把 residency 推进到下一帧 prepare snapshot。
- [M5 Hybrid GI Runtime Host Plan](../superpowers/plans/2026-04-16-m5-hybrid-gi-runtime-host.md): `Hybrid GI` 在 `RenderFrameExtract -> VisibilityContext -> WgpuRenderServer` 这条主链上的 probe/trace extract、dirty-request history、viewport probe-cache runtime host，以及 façade stats 边界。
- [M5 Hybrid GI Feedback Streaming Plan](../superpowers/plans/2026-04-16-m5-hybrid-gi-feedback-streaming.md): `VisibilityHybridGiFeedback` 如何在帧后驱动 runtime host 消费 pending probe update、回收 evictable probe，并把 trace schedule 与 probe residency 推进到下一帧 runtime snapshot。
- [M5 Virtual Geometry GPU Uploader Readback](../superpowers/plans/2026-04-17-m5-virtual-geometry-gpu-uploader-readback.md): `Virtual Geometry` 如何把 resident page table / pending request 送入真实 `wgpu` compute/readback 路径，并用 GPU completion source 推进 runtime host，而不是继续只靠 CPU feedback。
- [M5 Virtual Geometry Cluster Refine](../superpowers/plans/2026-04-17-m5-virtual-geometry-cluster-refine.md): `Virtual Geometry` 如何在统一 visibility 规划层引入 budget-aware parent-child refine frontier，让 children 只在预算允许时替换 parent。
- [M5 Hybrid GI GPU Completion Source](../superpowers/plans/2026-04-17-m5-hybrid-gi-gpu-completion-source.md): `Hybrid GI` 如何生成 renderer-local prepare snapshot、执行 probe update / trace completion compute pass，并把 GPU readback 回写到 viewport runtime host。
- [M5 Hybrid GI Radiance Cache Lighting Resolve](../superpowers/plans/2026-04-17-m5-hybrid-gi-radiance-cache-lighting-resolve.md): `Hybrid GI` 如何把 resident probe 的 irradiance 数据接进 `runtime prepare -> post-process resource -> shader resolve`，并让最终帧出现可测量的间接光颜色变化。

## Current Scope

当前文档覆盖的交付边界是：

- `zircon_resource` 作为跨 crate 资源基础层，统一 locator、typed handle、state、record、event、manager 契约
- `zircon-project.toml` + `assets/` + `library/` 的目录式项目根
- `res://` / `lib://` / `builtin://` / `mem://` 的统一资源来源模型
- PNG/JPEG、WGSL、TOML material、TOML scene、OBJ、glTF/GLB 的导入与 library artifact 持久化
- `SceneAssetSerializer` 驱动的 `SceneAsset <-> World` 转换，以及 `LevelSystem` 对运行中 world 的托管
- `MeshRenderer`/`RenderExtract` 基于 `ResourceHandle<ModelMarker/MaterialMarker>` 的渲染输入
- `zircon_graphics` 基于 `ResourceId + revision` 的 prepare/cache 与 WGSL shader / pipeline 选择，并把 `scene/resources/`、`scene/scene_renderer/core/`、`scene/scene_renderer/post_process/` 收口成 root-only wiring + folder-backed 子模块
- `zircon_rhi` / `zircon_rhi_wgpu` / `zircon_render_graph` / `zircon_render_server` 的基础渲染边界，以及 `RenderFrameExtract` 与旧 `RenderSceneSnapshot` 的过渡桥
- `zircon_graphics` 通过 `RenderServer` 路径执行真实的 `final color / scene color / bloom / gbuffer albedo / normal / AO / history / clustered-light buffer / reflection-probe buffer` 中间资源链，而不是只停在 compile skeleton；built-in deferred 已经会把 opaque geometry 改走固定材质解码和 deferred lighting，剩余的 `bloom / color grading / baked lighting / reflection probes / particle rendering / offline bake` 也已经沿同一条 runtime path 接通
- `Virtual Geometry / Hybrid GI` 的第一段 capability-slot 边界：extract placeholder、quality/profile toggle、history slot、feature descriptor、explicit compile opt-in、render-server capability gate 与 façade stats 可观测性
- `Virtual Geometry` 的 preprocess + runtime host + GPU completion + hierarchy refine baseline：cluster/page 数据合同、cluster-level frustum filtering、budgeted page request planning、跨帧 dirty request 历史、viewport 级 page table/residency/request sink host、frame-local prepare snapshot、prepare-driven mesh fallback filtering、feedback-driven residency progression、renderer-local `wgpu` uploader/readback completion source，以及 budget-aware parent-child refine frontier
- `Hybrid GI` 的 preprocess + runtime host + GPU completion + radiance-cache lighting resolve baseline：probe/trace region 数据合同、probe frustum filtering、budgeted probe request 与 trace schedule、跨帧 dirty probe request 历史、viewport 级 probe cache/residency/update host、feedback-driven probe residency / trace schedule progression、renderer-local probe update / trace completion readback source，以及 resident probe irradiance 驱动的 post-process lighting resolve
- editor 打开目录项目、导入模型、保存默认 level、通过 `ResourceManager` 读取资源树并在 watcher 变化后重建 viewport extract，并通过 `RenderServer` 驱动 editor/runtime viewport 输出

尚未覆盖的高阶内容仍包括完整 metallic-roughness 扩展材质模型、FBX/ASTC/PVRTexTool 真正导入链、`RenderingManager` 向纯兼容桥的最终收束，以及 GPU-driven visibility 的真实 indirect/occlusion/BVH 执行层、`Virtual Geometry` 的 cluster-streaming/indirect raster/深层 split-merge 路径、`Hybrid GI` 的 radiance-cache/screen-probe/RT 混合照明路径。

