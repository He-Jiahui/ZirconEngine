---
handoff_kind: fixed
status: fixed
created_at: 2026-07-15
summary_slug: mesh-template-pipeline-layout-binding-drift
origin_plan: docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
fixing_plan: docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
origin_child_dir: docs/plans/zircon_runtime/runtime/15
fixing_child_dir: docs/plans/zircon_runtime/render/18
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_gbuffer_mesh_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/prewarm_pipeline_validation.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/forward_shadow_receiver.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_construct/construct/construct.rs
  - zircon_runtime/src/graphics/shader/wgsl/zr_light_cookie.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_shading_standard_pbr.wgsl
tests:
  - cargo test -p zircon_runtime --lib graphics::scene::scene_renderer::mesh::mesh_pipeline::create_gbuffer_mesh_pipeline::tests::gbuffer_mesh_pipeline_creates_on_wgpu_device_with_template_shader --locked -- --exact --nocapture
  - cargo test -p zircon_runtime --lib graphics::scene::scene_renderer::mesh::mesh_pipeline_cache::prewarm_pipeline_validation::tests::mesh_prewarm_pipeline_validation_creates_all_builtin_pass_pipelines --locked -- --exact --nocapture
  - cargo test -p zircon_runtime --lib scene:: --locked
resolved_at: 2026-07-15
---


# Render18：mesh template source 与 pipeline layout binding 漂移

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md`
- 来源执行切片：Runtime15 depth-prepass source-guard Failure 的 Editor02 `scene::` 上行门
- 修复责任计划：`docs/plans/zircon_runtime/render/18-advanced-lighting-features.md`
- 交接原因：失败来自 Render18 新增 light-cookie / irradiance-volume shader binding 后，mesh template 与 WGPU validation pipeline layout 没有由同一 feature/source contract 装配；Runtime15 不拥有 advanced-lighting shader ABI 或 mesh material bind-group layout。

## 失败现象与复现证据

Windows 受管 Runtime15 job `eac5c8f0d83448aaa65034ff5ee9f9bf` 已实际运行
1685 个 `scene::` 匹配测试，结果 `1677 passed / 2 failed / 6 ignored / 6458
filtered`。仅有两个 WGPU validation 失败：

- `gbuffer_mesh_pipeline_creates_on_wgpu_device_with_template_shader`：fragment shader 要求
  group 1 binding 35，但 G-buffer validation pipeline layout 缺失该 binding；
- `mesh_prewarm_pipeline_validation_creates_all_builtin_pass_pipelines`：forward prewarm
  fragment shader 要求 group 1 binding 33，但 validation layout 缺失该 binding。

同一当前源码的 Runtime15 depth-prepass exact job
`aa6beee8c42c455e94547d1dc41c1cd2` 已为 `1 passed / 0 failed / 8142
filtered`，因此本 failure 不属于 source-guard owner 修复。

## 最低共享层根因

Render18 的 standard-PBR template/include 已能声明 light-cookie 与 irradiance-volume 资源，
但 G-buffer fixture 和 prewarm validation 使用的 scene/material pipeline layout 仍按旧 binding
集合构造。shader specialization 与 layout specialization 不再共享同一个 `PipelineKey` / feature
truth，导致真实 WGPU pipeline 创建阶段拒绝 binding 33/35。

## 架构修复验收

- mesh template source 与 bind-group layout 必须由同一 feature/source specialization contract
  决定；启用的 light-cookie / irradiance-volume module 对应真实资源 binding，未启用的 module
  不得通过测试专用 dummy layout 掩盖漂移。
- 上述两个 focused WGPU pipeline creation tests 在 current source 上均通过。
- 原始 Runtime15 `scene::` 上行门重新运行且无 Render18 pipeline validation 失败，之后才能
  回传 Runtime15 的 depth-prepass Failure。

## 禁止临时方案

- 禁止在测试内追加不可达的 dummy binding、屏蔽 WGPU validation 或删除两个 device tests。
- 禁止把 binding 声明复制到 Runtime15/source-guard owner，或恢复父文件兼容 wrapper。
- 禁止复用旧 pipeline 成功日志代替当前 shader source 与 layout 的重新创建。

## 修复结果与回传

- 根因：G-buffer and prewarm WGPU validators copied stale group-1 layouts after Render18 extended the production shadow-receiver ABI with bindings 33/35.
- 架构修复：Reused the canonical production create_forward_shadow_receiver_layout owner in both validators and deleted duplicate local layout truth.
- 验证：Managed GPU job a84e42ccf57e402b8f5679a104734de7 passed exact G-buffer and prewarm WGPU creation 1/1 each; upward job 2d3753edcc2149799ffb0e88eb31b6d3 passed scene:: 1667/0 with 6 ignored.
- 回传：Canonical mesh layout convergence is complete; Runtime15 scene upward gate is green and the failure is returned to the origin child record.
