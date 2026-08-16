---
handoff_kind: fixed
status: fixed
created_at: 2026-08-16
resolved_at: 2026-08-16
summary_slug: app01-editor-host-hgi-compile-drift
origin_plan: docs/plans/optimize/zircon_app/01-product-host-bootstrap-loop-dynamic-runtime-shutdown-review.md
fixing_plan: docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
origin_child_dir: docs/plans/optimize/zircon_app/01
fixing_child_dir: docs/plans/zircon_runtime/render/18
failure_scope: cross_plan
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/gpu_resources/execute_prepare/execute/card_capture_shading.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/gpu_resources/execute_prepare/execute/collect_inputs.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/gpu_resources/execute_prepare/execute/dispatch_probe_trace_tiles.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/gpu_resources/execute_prepare/execute/dispatch_probe_trace_tiles/tests.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/gpu_resources/execute_prepare/execute/dispatch_probe_trace_tiles/tests/global_sdf.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/gpu_resources/execute_prepare/execute/dispatch_radiance_cache/dispatch.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/gpu_resources/execute_prepare/execute/dispatch_radiance_cache/mod.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/gpu_resources/execute_prepare/execute/dispatch_radiance_cache/tests.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/gpu_resources/execute_prepare/execute/voxel_clipmap_debug.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/gpu_resources/global_sdf/dispatch.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/gpu_resources/global_sdf/packing.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/gpu_resources/global_sdf/packing/tests.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/gpu_resources/global_sdf/pending.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/gpu_resources/global_sdf/trace_bindings.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/gpu_resources/mod.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/mod.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/root_output_sources/runtime_prepare_collector.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/root_output_sources/runtime_prepare_collector/mesh_projection.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/scene_representation/global_sdf_scene_state/tests.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/scene_representation/mesh_sdf_scene_state/tests.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/scene_representation/mod.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/scene_representation/radiance_cache_state.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/scene_representation/radiance_cache_state/tests.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/scene_representation/surface_cache_state.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/scene_representation/trace_capability_graph/mod.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/scene_representation/voxel_scene_state.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/test_sources/hybrid_gi_render_framework_stats.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/test_sources/hybrid_gi_scene_prepare_material_fixtures.rs
  - zircon_plugins/hybrid_gi/runtime/src/provider/tests.rs
  - zircon_plugins/hybrid_gi/runtime/src/render_pass_executors/resolve_trace_handoff/tests/spatial_filter.rs
  - zircon_runtime/src/graphics/runtime_prepare_collector.rs
  - zircon_runtime/src/graphics/runtime_prepare_collector/tests.rs
tests:
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_plugin_hybrid_gi_runtime -SkipTest
  - RUST_TEST_NOCAPTURE=1 ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_plugin_hybrid_gi_runtime -LibTests
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_plugin_hybrid_gi_runtime -LibTests -TestFilter render_framework_stats_expose_scene_representation_screen_probe_and_radiance_cache_counts
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_app -NoDefaultFeatures -Features target-editor-host -Bin zircon_editor -SkipTest
---

# Render18: App01 editor-host Hybrid GI compile drift fixed

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_app/01-product-host-bootstrap-loop-dynamic-runtime-shutdown-review.md`
- 来源执行切片：App01 M2 shared foreign-output owner target-editor-host product gate
- 修复责任计划：`docs/plans/zircon_runtime/render/18-advanced-lighting-features.md`
- 交接原因：失败位于 Render18 自有的 neutral render DTO、Hybrid GI scene representation、
  GPU resources 与 Global SDF owner 边界；App01 不应在产品 host 层屏蔽插件。

## 失败现象与复现证据

App01 的 `target-editor-host` 产品构建在 Hybrid GI 包报告 42 个编译错误。Render18 的前向
实现跨过多个 owner 模块，但产品图无法完成 Hybrid GI 编译。修复后同一 App01 产品构建
成功越过 Hybrid GI，只在 Editor16 的 `with_hub_handshake` 可见性边界报告 `E0624`；该新
blocker 已单独交接给 Editor16。

## 最低共享层根因

导出可见性、函数签名、`Arc<[T]>` 调用形态、Global SDF completion byte count 与临时上传
slice 没有同步收敛。最低修复层是恢复各 DTO、常量和函数在真实 owner 上所需的最窄
可见性，并让所有调用点使用当前单一 API；App 和 Editor 不持有任何 Hybrid GI 例外。

## 架构修复验收

- neutral render root 导出 probe-trace diagnostic/cost/fallback DTO，Hybrid GI 内部保持最窄
  `pub(in ...)` 导出，不增加兼容别名或重复类型。
- runtime-prepare 投影缓存统一保存 canonical `Arc<[RenderMeshSnapshot]>` 与同次遍历生成的
  world bounds；稳定 generation 复用同一投影，不再从 Mesh SDF object 反向拼装第二份列表。
- Global SDF 空 object/payload 上传使用具名栈内 fallback slice，completion readback 按实际
  request 数计算字节数；readback map 错误向上返回，不再被 collector 静默丢弃。
- RenderFramework WGPU 回归使用有界帧数与 wall-clock deadline 等待真实 depth/probe readback，
  超时诊断同时输出 in-flight、completion、slot-reuse 和 Global SDF page counters。

## 验证与量化结果

- Hybrid GI 生产包构建通过：编译错误从 `42` 降至 `0`。
- 完整 `zircon_plugin_hybrid_gi_runtime` lib-test 图注册 `229` 项测试；协调器在
  `RUST_TEST_NOCAPTURE=1` 下 `build/test 2/2` 阶段通过，执行结果为 `209 passed / 20 ignored /
  0 failed`；focused RenderFramework/WGPU 证据 `5/5` 通过。
- Windows 默认 captured-output 矩阵两次在未报告任何 named failed test 或 Rust panic 时以
  process exit 101 结束；同一二进制默认直接执行与协调器 nocapture 完整矩阵均通过，因此
  该捕获通道异常不作为代码通过证据，也不被伪装为测试断言通过。
- runtime-prepare mesh projection 在 cache miss 时由两次列表构造收敛为一次 scene-mesh 遍历；
  cache hit 保持零重建。GPU 回读等待受 frame limit 与 wall-clock deadline 双重约束。
- App01 原始 editor-host 产品构建已越过 Hybrid GI，随后停在 Editor16 自有的
  `with_hub_handshake` 可见性错误；该新 blocker 已单独交接，不属于本 failure。

## 禁止临时方案

- 不在 App、Editor、Cargo feature 或测试上屏蔽 Hybrid GI。
- 不增加兼容别名、重复 DTO、测试专用导出或静默 fallback。
- 不吞掉 GPU readback 错误，不以源码检索替代包级构建和测试。
- 不把 Render18 仍开放的动态 RenderDoc/WGPU 产品验收宣称为已完成。

## 修复结果与回传

- 状态：`fixed`。
- 根因：Render18 DTO/constant/function owner exports and callers drifted apart across neutral render, Hybrid GI scene representation, GPU resources, and Global SDF.
- 架构修复：Restore the narrowest owner visibility, converge callers on one API, cache one canonical mesh/world-bounds projection, and propagate bounded GPU readback errors.
- 验证：Hybrid GI production build passes with 42 to 0 compile errors; focused default-capture tests pass; the nocapture full matrix reports 209 passed, 20 ignored, 0 failed; App01 editor-host reaches the separate Editor16 E0624 blocker.
- 回传：`docs/plans/zircon_runtime/render/18/2026-08-16-app01-editor-host-hgi-compile-drift-return.md` records the fixed status for Render18, and Editor16 owns the remaining host configuration visibility blocker.
