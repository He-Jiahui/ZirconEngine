---
related_code:
  - zircon_runtime/src/graphics/mod.rs
  - zircon_runtime/src/graphics/runtime_provider/mod.rs
  - zircon_runtime/src/graphics/runtime_provider/registration.rs
  - zircon_runtime/src/graphics/runtime_provider/update.rs
  - zircon_runtime/src/graphics/runtime_provider/feedback.rs
  - zircon_runtime/src/graphics/runtime_provider/prepare_input.rs
  - zircon_runtime/src/prelude.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/prelude.rs
  - zircon_runtime/src/scene/mod.rs
  - zircon_runtime/src/scene/prelude.rs
  - zircon_runtime/src/ui/mod.rs
  - zircon_runtime/src/ui/prelude.rs
  - zircon_runtime/src/ui/public_runtime_frame.rs
  - zircon_runtime/src/ui/tests/runtime_ui_support
  - zircon_runtime/src/graphics/types/viewport_render_frame_from_public_runtime.rs
  - zircon_runtime/src/graphics/mod.rs
  - zircon_runtime/src/graphics/prelude.rs
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs
  - zircon_runtime/src/core/runtime/state/module_entry.rs
  - zircon_runtime/src/core/runtime/diagnostics/devtools.rs
  - zircon_runtime/src/script/vm/host/builtin_host_modules.rs
  - zircon_runtime/src/graphics/backend/render_backend/offscreen_target.rs
  - zircon_runtime/src/graphics/backend/render_backend/offscreen_target_new/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_frame_graph_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/output_access.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/take_last_particle_gpu_readback_outputs.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/collect_runtime_feedback.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_material_uniform/gpu_material_uniform_resource.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs
  - zircon_runtime/src/graphics/scene/resources/post_process_lut_texture/post_process_lut_texture_resource.rs
  - zircon_runtime/src/graphics/scene/resources/output_target_texture/output_target_texture_resource.rs
  - zircon_runtime/src/graphics/scene/resources/prepared/prepared_output_target_texture.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_execute_output_target_writeback.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_mesh/gpu_mesh_resource.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_mesh/gpu_mesh_resource_from_asset.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_model/gpu_model_resource.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_model/gpu_model_resource_from_asset.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs
  - zircon_runtime/src/graphics/hybrid_gi_runtime_provider/provider_registration.rs
  - zircon_runtime/src/graphics/hybrid_gi_runtime_provider/prepare_input.rs
  - zircon_runtime/src/graphics/hybrid_gi_runtime_provider/runtime_update.rs
  - zircon_runtime/src/graphics/hybrid_gi_runtime_provider/runtime_feedback.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/provider_registration.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/prepare_input.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/runtime_update.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/runtime_feedback.rs
  - zircon_runtime/src/graphics/solari_runtime_provider/provider_registration.rs
  - docs/zircon_runtime/script/vm/host/function_ledger.md
  - docs/zircon_runtime/graphics/runtime_provider/registration.md
  - docs/zircon_runtime/graphics/runtime_provider/update.md
  - docs/zircon_runtime/graphics/runtime_provider/feedback.md
  - docs/zircon_runtime/graphics/runtime_provider/prepare_input.md
  - docs/zircon_runtime/graphics/render-product-submit.md
  - zircon_runtime/src/tests/prelude.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/provider_boilerplate.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/facade_surface.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/runtime_dead_code.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/diagnostics_surface.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/graphics_dead_code/mod.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/graphics_dead_code/module_layout.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/graphics_dead_code/renderer_output_accessors.rs
implementation_files:
  - docs/zircon_runtime/structure/module-convention.md
  - zircon_runtime/src/graphics/mod.rs
  - zircon_runtime/src/graphics/runtime_provider/mod.rs
  - zircon_runtime/src/graphics/runtime_provider/registration.rs
  - zircon_runtime/src/graphics/runtime_provider/update.rs
  - zircon_runtime/src/graphics/runtime_provider/feedback.rs
  - zircon_runtime/src/graphics/runtime_provider/prepare_input.rs
  - zircon_runtime/src/prelude.rs
  - zircon_runtime/src/asset/prelude.rs
  - zircon_runtime/src/scene/prelude.rs
  - zircon_runtime/src/ui/prelude.rs
  - zircon_runtime/src/graphics/prelude.rs
  - zircon_runtime/src/ui/public_runtime_frame.rs
  - zircon_runtime/src/ui/tests/runtime_ui_support
  - zircon_runtime/src/graphics/types/viewport_render_frame_from_public_runtime.rs
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs
  - zircon_runtime/src/core/runtime/state/module_entry.rs
  - zircon_runtime/src/core/runtime/diagnostics/devtools.rs
  - zircon_runtime/src/script/vm/host/builtin_host_modules.rs
  - zircon_runtime/src/graphics/backend/render_backend/offscreen_target.rs
  - zircon_runtime/src/graphics/backend/render_backend/offscreen_target_new/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_frame_graph_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/output_access.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/take_last_particle_gpu_readback_outputs.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/collect_runtime_feedback.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_material_uniform/gpu_material_uniform_resource.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs
  - zircon_runtime/src/graphics/scene/resources/post_process_lut_texture/post_process_lut_texture_resource.rs
  - zircon_runtime/src/graphics/scene/resources/output_target_texture/output_target_texture_resource.rs
  - zircon_runtime/src/graphics/scene/resources/prepared/prepared_output_target_texture.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_execute_output_target_writeback.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_mesh/gpu_mesh_resource.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_mesh/gpu_mesh_resource_from_asset.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_model/gpu_model_resource.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_model/gpu_model_resource_from_asset.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs
  - zircon_runtime/src/graphics/hybrid_gi_runtime_provider/provider_registration.rs
  - zircon_runtime/src/graphics/hybrid_gi_runtime_provider/prepare_input.rs
  - zircon_runtime/src/graphics/hybrid_gi_runtime_provider/runtime_update.rs
  - zircon_runtime/src/graphics/hybrid_gi_runtime_provider/runtime_feedback.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/provider_registration.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/prepare_input.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/runtime_update.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/runtime_feedback.rs
  - zircon_runtime/src/graphics/solari_runtime_provider/provider_registration.rs
  - docs/zircon_runtime/script/vm/host/function_ledger.md
  - docs/zircon_runtime/graphics/runtime_provider/registration.md
  - docs/zircon_runtime/graphics/runtime_provider/update.md
  - docs/zircon_runtime/graphics/runtime_provider/feedback.md
  - docs/zircon_runtime/graphics/runtime_provider/prepare_input.md
  - docs/zircon_runtime/graphics/render-product-submit.md
  - zircon_runtime/src/tests/prelude.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/provider_boilerplate.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/graphics_dead_code/mod.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/graphics_dead_code/module_layout.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/graphics_dead_code/renderer_output_accessors.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date.rs
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
tests:
  - cargo test -p zircon_runtime --lib runtime_15_mixed_visibility_has_facade_note --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib prelude --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_prelude_covers_required_types --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_facade_surface_guard_is_folder_backed --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_runtime_dead_code_guard_is_folder_backed --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_diagnostics_guard_is_folder_backed --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_runtime_ui_dead_code_surface_is_test_support --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_diagnostics_use_frame_trait_without_world_wrapper --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_provider_registration_uses_shared_owner --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_provider_update_uses_shared_stats_owner --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_provider_feedback_uses_shared_payload_owner --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_provider_prepare_input_uses_shared_extract_generation_owner --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_runtime_owned_dead_code_suppression_cleanup --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_script_host_value_descriptors_do_not_suppress_dead_code --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_offscreen_target_texture_owner_cleanup --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_gpu_material_uniform_owner_cleanup --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_gpu_mesh_order_signature_cleanup --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_gpu_model_identity_cleanup --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_post_process_lut_texture_owner_cleanup --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_output_target_texture_owner_cleanup --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_resource_streamer_diagnostics_accessor_cleanup --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_resource_streamer_resolve_texture_id_cleanup --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_particle_gpu_readback_output_accessor_cleanup --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_advanced_plugin_output_test_accessor_cleanup --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_graphics_dead_code_guard_is_folder_backed --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_provider_boilerplate_guard_is_folder_backed --no-default-features --features core-min --locked
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked
doc_type: module-detail
---

# Runtime 模块结构规范镜像文档

> 本文是 [Runtime 15](../../plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md) 的镜像文档，固定 `module_convention_gate` 的结构审计事实，由 `runtime_15_module_convention_mirror_docs_match_structure_audit_counts` 守卫锁定计数。上游规范：[`engine-code-structure-convention.md`](../../plans/engine-code-structure-convention.md)。
>
> 状态：in_progress（Runtime 15 F9 runtime prelude required type coverage、Runtime 15 graphics facade visibility note、Runtime 15 runtime UI dead-code support split、Runtime 15 F12 runtime-owned dead-code suppression cleanup、Runtime 15 F12 script host value descriptor dead-code cleanup、Runtime 15 F12 offscreen target texture owner cleanup、Runtime 15 F12 render backend state owner cleanup、Runtime 15 F12 gpu texture resource owner cleanup、Runtime 15 F12 gpu material uniform owner cleanup、Runtime 15 F12 gpu mesh order signature cleanup、Runtime 15 F12 gpu model identity cleanup、Runtime 15 F12 post-process LUT texture owner cleanup、Runtime 15 F12 output target texture owner cleanup、Runtime 15 F12 material runtime capture seed cleanup、Runtime 15 F12 resource streamer diagnostics accessor cleanup、Runtime 15 F12 resource streamer resolve texture id cleanup、Runtime 15 F12 particle GPU readback output accessor cleanup、Runtime 15 F12 advanced plugin output test accessor cleanup、Runtime 15 M3 graphics dead-code guard module split、Runtime 15 M3 provider boilerplate guard module split、Runtime 15 F14 diagnostics normalization、Runtime 15 F13 provider registration shared owner、Runtime 15 F13 provider update shared stats owner、Runtime 15 F13 provider feedback shared payload owner、Runtime 15 F13 provider prepare input shared frame owner 与 Runtime 15 F13 full provider boilerplate audit 已落地；完整 `module_convention_boundary.py` 审计计数、全量 dead-code sweep 与测试组织拆分仍 pending）。
>
> 最新完成：Runtime 15 M3 facade surface guard module split（`runtime_15_facade_surface_guard_module_split_static_passed_cargo_lock_blocked`）已把 façade/prelude 结构守卫迁入 `structure_convention/facade_surface.rs`；完整测试组织拆分仍 pending。
>
> 最新完成：Runtime 15 M3 runtime dead-code guard module split（`runtime_15_runtime_dead_code_guard_module_split_static_passed_cargo_lock_blocked`）已把 runtime dead-code 结构守卫迁入 `structure_convention/runtime_dead_code.rs`；完整测试组织拆分仍 pending。
>
> 最新完成：Runtime 15 M3 diagnostics guard module split（`runtime_15_diagnostics_guard_module_split_static_passed_cargo_lock_blocked`）已把 diagnostics 结构守卫迁入 `structure_convention/diagnostics_surface.rs`；完整测试组织拆分仍 pending。

## 治理范围

`zircon_runtime/src` 全模块的：façade 友好度（R3.1/R3.3）、可见性纪律（R3.4）、命名（R2.1–R2.4）、`mod.rs`/`module.rs` 判据（R1.2）、行为 owner 化（R1.3）、行数预算（R1.4）、测试组织（R4.1–R4.4）。

## `module_convention_gate` 字段（待 M1 实测填充）

| 字段 | 含义 | 目标 |
|---|---|---|
| `oversized_facade_files` | 超符号 / 行预算的 façade | → 0 |
| `mixed_visibility_mod_files` | 无 façade 注释的 `pub`/`pub(crate)` 混排 | → 0 |
| `prefix_vocabulary_violations` | 越界前缀（`runtime_` 滥用等） | → 0 |
| `plural_singular_violations` | 复数 / 单数误用目录 | → 0 |
| `banned_name_modules` | `_inner`/`_impl`/`_helper`/`util` 等 | → 0 |
| `module_rs_without_descriptor` | 非注册子系统却有 `module.rs` | → 0 |
| `oversized_test_files` | > 800 行测试 | → 0 |
| `duplicate_test_trees` | 重复测试树 | → 0 |
| `module_convention_gate.m1_gate_status` | 门状态 | `classified-and-clear` |
| `migration_debt_count` | 迁移债 | → 0 |
| `exempt` | 登记豁免 | 仅 vendored / fixture / `@generated` |

## 联动

与 `large-file-ownership-m1.md` 共享 hotspot 清单；render 子计划 graphics 热点纳入本治理。

## Runtime 15 graphics facade visibility note

状态：`runtime_15_graphics_facade_visibility_note_static_passed_cargo_blocked_graphics_drift`。

R3.4 的当前已落地部分是 `graphics/mod.rs` 的混合可见性边界说明。该 root façade 仍保留同一导出集合，但源码已经明确分出 crate-private implementation owners、public module entries、public façade exports、crate-visible bridge 和 test-only access。公共 module entries 只包括 feature extract source 合同、`graphics::prelude` 和 graphics module descriptor surface；`backend`、`scene`、`types` 等实现 owner 保持 `pub(crate)`，不会作为稳定模块入口泄漏。

守卫：`runtime_15_mixed_visibility_has_facade_note` 验证 `graphics/mod.rs` 保留上述分区注释、公共入口和实现模块私有性，并验证 Runtime 15 计划、runtime index、结构规范和本文档都记录同一状态锚。scoped rustfmt with `skip_children=true`、standalone guard 和状态锚静态检查已通过；Cargo 聚焦验证当前被既有 graphics 编译漂移阻塞（`FrameSubmissionContext::new` 参数数不匹配、`AdvancedProfileRuntimePlan: Default` 缺失），因此本切片只记录静态守卫闭合，core-min Cargo gate 继续 pending。

## Runtime 15 F9 runtime prelude required type coverage

状态：`runtime_15_prelude_required_types_coremin_check_passed`。

R3.3 的当前已落地部分是 prelude 分层：`asset/prelude.rs`、`scene/prelude.rs`、`ui/prelude.rs`、`graphics/prelude.rs` 分别维护子系统高频类型，crate 级 `prelude.rs` 只聚合这些子系统 prelude，不再直接列 asset/scene/ECS/UI/graphics 符号。该形态让 gameplay/authoring 用户能通过 `zircon_runtime::prelude::*` 获取资产句柄与 descriptor、ECS world/query/resource、UI surface/template/v2、graphics module/render pipeline 等常用入口，同时保持完整公共面仍归各子系统 `mod.rs`。

守卫：`runtime_prelude_exports_asset_scene_ui_and_graphics_contracts` 验证行为面可用；`runtime_15_prelude_covers_required_types` 验证 crate 聚合、四个子系统 `pub mod prelude;`、必含类型清单、Runtime 15 计划、runtime index、审查发现、结构规范和本文档状态锚同步。

## Runtime 15 runtime UI dead-code support split

状态：`runtime_15_runtime_ui_dead_code_support_split_coremin_check_passed`。

E6/S10/F10/F12 的当前已落地部分是把 runtime UI 的生产 dead-code surface 与测试支持拆开。`PublicRuntimeFrame` 现在由生产 owner `ui/public_runtime_frame.rs` 持有，`graphics/types/viewport_render_frame_from_public_runtime.rs` 继续通过 `crate::ui::PublicRuntimeFrame` 构造 `ViewportRenderFrame` 并把 `frame.extract` 包装为 `Arc<RenderFrameExtract>`。`RuntimeUiManager`、`RuntimeUiFixture`、input router、manager error 与 window-event helpers 全部移入 `ui/tests/runtime_ui_support`，由 `ui/mod.rs` 通过 `#[cfg(test)]` 和 `#[path = "tests/runtime_ui_support/mod.rs"]` 挂载给测试使用。

旧生产 `ui/runtime_ui/` 目录已删除，`ui/mod.rs` 不再声明 `#[allow(dead_code)] mod runtime_ui;`，也不保留兼容 re-export 或 shim。守卫：`runtime_15_runtime_ui_dead_code_surface_is_test_support` 验证生产 frame owner、test-only support owner、旧目录删除、graphics conversion anchor，以及 Runtime 15 计划、runtime index、审查发现、结构规范和本文档状态锚同步。验证：scoped rustfmt --check 通过；standalone structure guard 1/1、ui_architecture 3/3、status-output 2/2 通过；direct ui_architecture_boundary_audit risks=[]（ui entries 18/18、taffy hits/files 175/175 与 10/10）；core-min focused cargo test `runtime_15_runtime_ui_dead_code_surface_is_test_support` 1/1 通过；core-min `cargo check` 通过（既有 warnings）。 该切片只关闭 `runtime_ui` 子面；F12 全量 `#[allow(dead_code)]` sweep 仍由 Runtime 15 M5/T2 后续执行。

## Runtime 15 F12 runtime-owned dead-code suppression cleanup

状态：`runtime_15_runtime_owned_dead_code_suppression_cleanup_coremin_check_passed`。

E6/S10/F12 的当前新增落地部分是两个 runtime-owned suppression 点清理，避开 active render/provider、plugin、editor 会话区域。`asset/pipeline/worker_pool.rs` 中的 test-only `request_rx_guard` 不再依赖 `#[allow(dead_code)]`，而是通过 `request_channel_guard_is_alive_for_test()` 暴露给 worker-pool 行为测试，测试显式断言 bounded overflow 无 worker 场景下 receiver guard 仍然存活。这样保留通道连接语义，同时让 test-only 支撑代码有真实读取点。

`core/runtime/state/module_entry.rs` 的 descriptor 字段不再压制 dead-code lint；`ModuleEntry::descriptor()` 现在是明确 accessor，`core/runtime/diagnostics/devtools.rs` 通过该 accessor 读取 module name、description、driver/manager/plugin counts。守卫：`runtime_15_runtime_owned_dead_code_suppression_cleanup` 验证 asset worker 和 module entry 两个文件不再包含 `#[allow(dead_code)]`，并验证 Runtime 15 计划、runtime index、审查发现、结构规范和本文档状态锚同步。该切片只关闭 runtime-owned 两处 suppression 子面；script host value descriptor 子面已由 `runtime_15_script_host_value_descriptors_coremin_check_passed` 关闭，OffscreenTarget 固定帧 texture owner 子面已由 `runtime_15_offscreen_target_texture_owner_cleanup_static_passed_cargo_timeout_no_result` 关闭，更宽 graphics resources 与全量 F12 sweep 仍 pending。

## Runtime 15 F12 script host value descriptor dead-code cleanup

状态：`runtime_15_script_host_value_descriptors_coremin_check_passed`。

E6/S10/F12 的当前新增落地部分是脚本宿主 math 值描述器清理。`script/vm/host/builtin_host_modules.rs` 的 `Vec3` 与 `ColorRgba` 只作为 `ZirconScriptType` 反射描述器进入 `zircon_host_module!`，因此原先用 `#[allow(dead_code)]` 避开字段未读告警。本轮移除这两个 suppression，并新增字段布局哨兵，构造并读取 `Vec3 { x, y, z }` 与 `ColorRgba { r, g, b, a }`，让 descriptor-only 类型的字段形状保持编译器可见。

该哨兵不新增 VM host call、不改变 `zr.zircon.math` 的 `vec3_length` / `vec3_dot` 函数面，也不改变 Runtime 13 host ledger：`docs/zircon_runtime/script/vm/host/function_ledger.md` 仍记录 6 个固定 host module、52 个固定 host function、2 个固定 script type descriptor。守卫：`runtime_15_script_host_value_descriptors_do_not_suppress_dead_code` 验证 `builtin_host_modules.rs` 不再包含 `#[allow(dead_code)]`、布局哨兵读取所有字段、ledger 计数稳定，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档和脚本宿主清册的状态锚同步。该切片只关闭 script host value descriptor 子面；OffscreenTarget 固定帧 texture owner 子面已由 `runtime_15_offscreen_target_texture_owner_cleanup_static_passed_cargo_timeout_no_result` 关闭，更宽 graphics resources 与全量 F12 sweep 仍 pending。

## Runtime 15 F12 offscreen target texture owner cleanup

状态：`runtime_15_offscreen_target_texture_owner_cleanup_static_passed_cargo_timeout_no_result`。

E6/S10/F12 的当前新增落地部分是渲染后端固定帧 texture owner 清理。`graphics/backend/render_backend/offscreen_target.rs` 里的 `global_illumination`、`scene_color`、`bloom`、G-buffer、`normal`、`depth` 等 WGPU texture 字段原本只通过对应 `TextureView` 间接服务帧图资源导入，因此用 `#[allow(dead_code)]` 避开未读告警。本轮移除这些 suppression，并新增 `OffscreenTarget::RETAINED_FRAME_TEXTURE_COUNT` 与 `retained_frame_texture_count()`，显式读取 final color、GI、scene color、bloom、G-buffer、normal、AO 与 depth 的 9 个 texture owner。

`scene_renderer_core_render_compiled_scene/render/bind_frame_graph_resources.rs` 在生产绑定入口通过 debug assertion 消费该 owner 计数，说明这些字段负责保活 graph-imported `TextureView` 背后的 WGPU resources，而不是未接线脚手架。守卫：`runtime_15_offscreen_target_texture_owner_cleanup` 验证 OffscreenTarget 不再包含 `#[allow(dead_code)]`、构造路径仍 materialize 9 个 owner、compiled-scene binder 消费保活契约，并验证 Runtime 15 计划、runtime index、render index、审查发现、结构规范、本文档与 render-product 文档的状态锚同步。该切片只关闭 OffscreenTarget 固定帧 texture owner 子面；更宽 graphics resources 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt --check 通过；standalone structure guard 1/1、status-output all-subplans guard 1/1 通过；core-min Cargo check 在独立 target 目录 10 分钟超时无编译结果，残留本切片 cargo/rustc 进程已停止，不计通过。

## Runtime 15 F12 render backend state owner cleanup

状态：`runtime_15_render_backend_state_owner_cleanup_coremin_check_passed`。

E6/S10/F12 的当前新增落地部分是渲染后端 WGPU state owner 清理。`graphics/backend/render_backend/render_backend.rs` 里的 `instance`、`adapter` 与 `config` 字段原本作为 backend lifetime owner 保留，但除 capability projection 间接使用外没有显式读取，因此用 `#[allow(dead_code)]` 避开未读告警。本轮移除这些 suppression，并新增 `RenderBackend::RETAINED_STATE_OWNER_COUNT` 与 `retained_state_owner_count()`，显式读取 instance、adapter 与 config 3 个 backend state owner。

`RenderBackend::caps()` 在生产 capability projection 路径通过 debug assertion 消费该 owner 计数，说明这些字段负责保活 WGPU backend state 与 backend config，而不是未接线脚手架。守卫：`runtime_15_render_backend_state_owner_cleanup` 验证 `RenderBackend` 不再包含 `#[allow(dead_code)]`、owner 计数契约读取三项 state、`caps()` 消费保活契约，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档与 render-product 文档的状态锚同步。该切片只关闭 RenderBackend state owner 子面；更宽 graphics resources 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt --check 通过；standalone structure guard 1/1、status-output 2/2 通过；core-min `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime15-render-backend-state-owner-0622` 通过（既有 warnings）。

## Runtime 15 F12 gpu texture resource owner cleanup

状态：`runtime_15_gpu_texture_resource_owner_cleanup_coremin_check_passed`。

E6/S10/F12 的当前新增落地部分是材质纹理 GPU resource owner 清理。`graphics/scene/resources/gpu_texture/gpu_texture_resource.rs` 里的 `id`、`texture`、`view` 与 `sampler` 字段原本作为 material texture binding 的身份和 WGPU resource owner 保留，但用 `#[allow(dead_code)]` 避开未读告警。本轮移除这些 suppression，并新增 `GpuTextureResource::RETAINED_TEXTURE_BINDING_OWNER_COUNT` 与 `retained_texture_binding_owner_count()`，显式读取 texture identity、WGPU texture、view 与 sampler 4 个 binding owner。

`GpuTextureResource::view()` 和 `GpuTextureResource::sampler()` 在材质绑定入口通过 debug assertion 消费该 owner 计数，说明这些字段负责保活 material bind group 背后的 WGPU resources，而不是未接线脚手架。守卫：`runtime_15_gpu_texture_resource_owner_cleanup` 验证 `GpuTextureResource` 不再包含 `#[allow(dead_code)]`、owner 计数契约读取四项 state、view/sampler 绑定 accessor 消费保活契约，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档与 render-product 文档的状态锚同步。该切片只关闭 GpuTextureResource owner 子面；更宽 graphics resources 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt --check 通过；standalone structure guard 1/1、status-output 2/2 通过；core-min `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime15-gpu-texture-owner-0622` 通过（既有 warnings）。

## Runtime 15 F12 gpu material uniform owner cleanup

状态：`runtime_15_gpu_material_uniform_owner_cleanup_coremin_check_passed`。

E6/S10/F12 的当前新增落地部分是材质 uniform GPU resource owner 清理。`graphics/scene/resources/gpu_material_uniform/gpu_material_uniform_resource.rs` 里的 `buffer`、`payload_byte_len` 与 `buffer_byte_len` 字段原本作为 material uniform binding 的 WGPU buffer owner 与 byte-length diagnostics 保留，但用 `#[allow(dead_code)]` 避开未读告警。本轮移除这些 suppression，并新增 `GpuMaterialUniformResource::RETAINED_MATERIAL_UNIFORM_OWNER_COUNT` 与 `retained_material_uniform_owner_count()`，显式读取 WGPU buffer、payload byte length 与 padded buffer byte length 3 个 binding/diagnostics owner。

`GpuMaterialUniformResource::binding_resource()` 在材质 uniform 绑定入口通过 debug assertion 消费该 owner 计数，说明 buffer 字段负责保活 bind group 背后的 WGPU resource。`resource_streamer_accessors.rs` 现在通过 `payload_byte_len()` / `buffer_byte_len()` owner accessor 暴露诊断长度，不再直接读取字段。守卫：`runtime_15_gpu_material_uniform_owner_cleanup` 验证 `GpuMaterialUniformResource` 不再包含 `#[allow(dead_code)]`、owner 计数契约读取三项 state、binding accessor 消费保活契约、resource streamer 通过 owner accessor 读取诊断长度，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档与 render-product 文档的状态锚同步。该切片只关闭 GpuMaterialUniformResource owner 子面；更宽 graphics resources 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt --check 通过；standalone structure guard 1/1、status-output 2/2 通过；core-min `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime15-gpu-material-uniform-owner-0622` 通过（既有 144 warnings）。

## Runtime 15 F12 gpu mesh order signature cleanup

状态：`runtime_15_gpu_mesh_order_signature_cleanup_coremin_check_passed`。

E6/S10/F12 的当前新增落地部分是 mesh order-signature dead-code suppression 清理。`graphics/scene/resources/gpu_mesh/gpu_mesh_resource.rs` 里的 `indirect_order_signature` 字段原本服务 Virtual Geometry / indirect submission 顺序契约，但用 `#[allow(dead_code)]` 避开未读告警。本轮移除 suppression，将字段收窄为 `gpu_mesh` owner 内部字段，并通过 `GpuMeshResource::indirect_order_signature()` 暴露只读契约。

`graphics/scene/resources/gpu_mesh/gpu_mesh_resource_from_asset.rs` 继续从 position、normal、uv、joint indices/weights、tangent、color 与 index payload 派生完整 order signature。`graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs` 的 prepared mesh draw 路径通过 `mesh_order_command_sort_tie_breaker(...)` 把该签名混入稳定排序 tie-breaker，说明该字段是 draw ordering live input，而不是未接线脚手架。守卫：`runtime_15_gpu_mesh_order_signature_cleanup` 验证资源字段、签名派生、draw builder 接线，以及 Runtime 15 计划、runtime index、审查发现、结构规范、本文档与 render-product 文档的状态锚同步。该切片只关闭 GpuMeshResource order-signature 子面；更宽 graphics resources 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt --check 通过；standalone structure guard 1/1、status-output 2/2 通过；core-min `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime15-gpu-mesh-order-0622` 通过（既有 warnings）。

## Runtime 15 F12 gpu model identity cleanup

状态：`runtime_15_gpu_model_identity_cleanup_coremin_check_passed`。

E6/S10/F12 的当前新增落地部分是 GPU model identity dead-code suppression 清理。`graphics/scene/resources/gpu_model/gpu_model_resource.rs` 里的 `id` 字段原本用于记录资源身份，但用 `#[allow(dead_code)]` 避开未读告警。本轮移除 suppression，将字段收窄为 `gpu_model` owner 内部字段，并通过 `GpuModelResource::id()` 暴露只读契约。

`graphics/scene/resources/gpu_model/gpu_model_resource_from_asset.rs` 继续在构造 GPU model 时记录 `ResourceId`。`graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs::model(...)` 在返回缓存资源前通过 debug assertion 校验 `prepared.resource.id()` 与 streamer key 一致，说明该字段是 ResourceStreamer model cache identity 的 live contract，而不是未接线脚手架。守卫：`runtime_15_gpu_model_identity_cleanup` 验证资源字段、构造记录、streamer 查询接线，以及 Runtime 15 计划、runtime index、审查发现、结构规范、本文档与 render-product 文档的状态锚同步。该切片只关闭 GpuModelResource identity 子面；更宽 graphics resources 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt --check 通过；standalone structure guard 1/1、status-output 2/2 通过；core-min `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime15-gpu-model-identity-0622` 通过（既有 warnings）。

## Runtime 15 F12 post-process LUT texture owner cleanup

状态：`runtime_15_post_process_lut_texture_owner_cleanup_coremin_check_passed`。

E6/S10/F12 的当前新增落地部分是 post-process LUT texture owner dead-code suppression 清理。`graphics/scene/resources/post_process_lut_texture/post_process_lut_texture_resource.rs` 里的 `texture` 字段原本作为 3D LUT `TextureView` 背后的 WGPU owner 保留，但用 `#[allow(dead_code)]` 避开未读告警。本轮移除 suppression，并新增 `PostProcessLutTextureResource::RETAINED_LUT_TEXTURE_OWNER_COUNT` 与 `retained_lut_texture_owner_count()`，显式读取 texture/view 两个 LUT binding owner。

`PostProcessLutTextureResource::view()` 在 3D LUT 绑定入口通过 debug assertion 消费该 owner 计数。`graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs::prepared_post_process_lut_3d_view(...)` 保持 `RenderColorLookupTextureLayout::matches_texture_3d(...)` descriptor 匹配，并改为通过 `prepared.resource.view()` 暴露 binding view，说明 texture 字段是 ResourceStreamer post-process LUT cache 的 live owner，而不是未接线脚手架。守卫：`runtime_15_post_process_lut_texture_owner_cleanup` 验证资源字段、owner 计数、streamer 3D LUT accessor，以及 Runtime 15 计划、runtime index、审查发现、结构规范、本文档与 render-product 文档的状态锚同步。该切片只关闭 PostProcessLutTextureResource owner 子面；更宽 graphics resources 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt --check 通过；standalone structure guard 1/1、status-output 2/2 通过；core-min `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime15-post-process-lut-owner-0622` 通过（既有 warnings）。

## Runtime 15 F12 output target texture owner cleanup

状态：`runtime_15_output_target_texture_owner_cleanup_coremin_check_passed`。

E6/S10/F12 的当前新增落地部分是 output target texture owner dead-code suppression 清理。`graphics/scene/resources/output_target_texture/output_target_texture_resource.rs` 里的 descriptor、texture、view 与 sampler 字段原本作为 camera texture-target 写回、graph import 和材质采样的 WGPU resource owner 保留，但用 `#[allow(dead_code)]` 避开未读告警。本轮移除 suppression，并新增 `OutputTargetTextureResource::RETAINED_OUTPUT_TARGET_TEXTURE_OWNER_COUNT` 与 `retained_output_target_texture_owner_count()`，显式读取 output target descriptor、WGPU texture、view 与 sampler 4 个 owner。

`OutputTargetTextureResource::descriptor()`、`size()`、`texture()`、`view()` 与 `sampler()` 在 writeback、compiled-scene graph import 和 material sampling 路径通过 debug assertion 消费该 owner 计数。`graphics/scene/resources/prepared/prepared_output_target_texture.rs` 同步移除 prepared `resource` 字段 suppression，新增 `PreparedOutputTargetTexture::RETAINED_OUTPUT_TARGET_CACHE_OWNER_COUNT`、`retained_output_target_cache_owner_count()` 与 `resource()` accessor。`ResourceStreamer` 的 output-target graph import readiness、writeback clone 与 public output target resource accessor 都通过 prepared accessor 读取 cached Arc，说明这些字段是 output target cache 的 live owner，而不是未接线脚手架。守卫：`runtime_15_output_target_texture_owner_cleanup` 验证资源字段、cache owner 计数、streamer graph-import/writeback accessor，以及 Runtime 15 计划、runtime index、审查发现、结构规范、本文档与 render-product 文档的状态锚同步。该切片只关闭 OutputTargetTextureResource / PreparedOutputTargetTexture owner 子面；更宽 graphics resources 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt --check 通过；standalone structure guard 1/1、status-output 2/2 通过；core-min `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime15-output-target-owner-0622` 通过（既有 warnings）。

## Runtime 15 F12 material runtime capture seed cleanup

状态：`runtime_15_material_runtime_capture_seed_cleanup_coremin_check_passed`。

E6/S10/F12 的当前新增落地部分是 material runtime capture seed dead-code suppression 清理。`graphics/scene/resources/runtime/material_runtime.rs` 不再用 `#[allow(dead_code)]` 遮盖 `MaterialCaptureSeed`、`MaterialRuntime` 或 `MaterialRuntime::capture_seed()`。`MaterialRuntime` 仍保留为生产材质运行态 DTO，因为 material preparation、uniform upload、mesh draw construction 和 readiness reporting 都读取它；`MaterialCaptureSeed` 与 `capture_seed()` 则收进 `#[cfg(test)]`，只服务 render product streamer 测试对材质捕获种子的回归断言。

`graphics/scene/resources/runtime/mod.rs` 与 `graphics/scene/resources/mod.rs` 的 `MaterialCaptureSeed` re-export 同步收进 test cfg。`graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs` 的 `material_capture_seed(...)`、`sample_texture_rgba(...)`、`shading_model_id_for_lighting_model(...)`、`sample_texture_asset_rgba(...)` 与 `wrap01(...)` 也收进 test cfg，避免历史 Hybrid GI/material capture helper 继续作为生产 dead-code surface 暴露。守卫：`runtime_15_material_runtime_capture_seed_cleanup` 验证 material runtime、runtime/resources façade、resource streamer capture accessors，以及 Runtime 15 计划、runtime index、审查发现、结构规范、本文档与 render-product 文档的状态锚同步。该切片只关闭 MaterialRuntime capture seed/test texture sampling 子面；`resource_streamer_accessors.rs` 中其余 diagnostics accessor suppression 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt --check 通过；standalone structure guard 1/1、status-output 2/2 通过；core-min `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime15-material-capture-0622` 通过（既有 warnings）。

## Runtime 15 F12 resource streamer diagnostics accessor cleanup

状态：`runtime_15_resource_streamer_diagnostics_accessor_cleanup_static_passed_cargo_lock_blocked`。

E6/S10/F12 的当前新增落地部分是 ResourceStreamer diagnostics accessor suppression 清理。`graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs` 不再包含 `#[allow(dead_code)]`。只由 render product / asset flow 测试读取的资产管理快照、材质管理查询、uniform/property/texture-slot 诊断和 prepared-material state helper 统一收进 `#[cfg(test)]`，避免测试诊断 surface 继续留在生产构建里伪装为未接线生产代码。

生产仍使用的 material readiness bridge 不收进 test cfg：`material_readiness_report(...)` 与 `material_readiness_summary(...)` 保持正常构建，`resource_streamer_ensure_scene_resources.rs` 继续通过 `self.material_readiness_summary(&material_id)` 汇总材质 readiness stats。守卫：`runtime_15_resource_streamer_diagnostics_accessor_cleanup` 验证 accessors 文件没有 dead-code suppression、代表性测试诊断入口是 test-only、生产 readiness summary 仍由 ensure path 消费，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档和 render-product 文档的状态锚同步。该切片只关闭 ResourceStreamer diagnostics accessor 子面；全量 F12 sweep 仍 pending。

验证：scoped rustfmt --check 通过；静态扫描确认 `resource_streamer_accessors.rs` 无 `#[allow(dead_code)]`；带锁 standalone structure guard/status-output/core-min cargo check 均在进入测试前被当前工作区 `Cargo.lock` / `Cargo.toml` 不一致阻塞（Cargo 需要补齐 `zircon_plugin_sdk` 相关锁文件条目），不计通过。

## Runtime 15 F12 resource streamer resolve texture id cleanup

状态：`runtime_15_resource_streamer_resolve_texture_id_cleanup_static_passed_cargo_lock_blocked`。

E6/S10/F12 的当前新增落地部分是 ResourceStreamer texture-reference helper 僵尸清理。`graphics/scene/resources/resource_streamer/resource_streamer_resolve_texture_id.rs` 不再包含 `#[allow(dead_code)]`，并删除未使用的 `ResourceStreamer::resolve_texture_id(...)`。全仓库调用面没有该 helper 的生产消费者，因此本切片采用硬删除而不是 test-only 保留。

生产贴图解析入口保持不变：`resolve_texture_reference(...)` 与 `resolve_texture_reference_with_support(...)` 继续返回 `ResolvedTextureReference`，`ResolvedTextureReference::id()` 仍供当前材质准备路径读取成功解析的 `ResourceId`。未解析 locator 和未满足 upload support 的纹理仍走 `RenderMaterialValidationError`、`RenderMaterialFallbackUsage` 与 `RenderMaterialTextureSlotFallback` 报告路径。守卫：`runtime_15_resource_streamer_resolve_texture_id_cleanup` 验证旧 helper 和 dead-code suppression 不复活、生产解析入口仍存在，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档和 render-product 文档的状态锚同步。该切片只关闭 `resolve_texture_id` 僵尸 helper 子面；全量 F12 sweep 仍 pending。

验证：scoped rustfmt --check 通过；静态扫描确认该文件无 `#[allow(dead_code)]` 且 `resolve_texture_id(` 只剩状态守卫字符串；带锁 standalone structure guard/status-output/core-min cargo check 均在进入测试前被当前工作区 `Cargo.lock` / `Cargo.toml` 不一致阻塞（Cargo 需要补齐 `zircon_plugin_sdk` 相关锁文件条目），不计通过。

## Runtime 15 F12 particle GPU readback output accessor cleanup

状态：`runtime_15_particle_gpu_readback_output_accessor_cleanup_static_passed_cargo_lock_blocked`。

E6/S10/F12 的当前新增落地部分是 renderer runtime-output accessor 的 dead-code suppression 清理。`graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/take_last_particle_gpu_readback_outputs.rs` 中的 `SceneRenderer::take_last_particle_gpu_readback_outputs(...)` 不再包含 `#[allow(dead_code)]`，因为它已经由生产 runtime feedback 收集路径消费。

`graphics/runtime/render_framework/submit_frame_extract/submit/collect_runtime_feedback.rs::collect_particle_feedback(...)` 从 renderer output drain 调用 `renderer.take_last_particle_gpu_readback_outputs()`，再和 `RenderPreparedRuntimeSidebands::take_particle_readback_outputs()` 合并，最后在非空时投递到 `ParticleGpuFeedback::new(...)`。这说明该 accessor 是 particle runtime feedback bridge 的 live 输入，而不是未接线脚手架。守卫：`runtime_15_particle_gpu_readback_output_accessor_cleanup` 验证 accessor 文件无 dead-code suppression、feedback collector 仍消费该 accessor，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档和 render-product 文档的状态锚同步。该切片只关闭 particle GPU readback accessor 子面；全量 F12 sweep 仍 pending。

验证：scoped rustfmt --check 通过；静态扫描确认该文件无 `#[allow(dead_code)]` 且 runtime feedback 路径消费 `renderer.take_last_particle_gpu_readback_outputs()`；带锁 standalone structure guard/status-output/core-min cargo check 均在进入测试前被当前工作区 `Cargo.lock` / `Cargo.toml` 不一致阻塞，不计通过。

## Runtime 15 F12 advanced plugin output test accessor cleanup

状态：`runtime_15_advanced_plugin_output_test_accessor_cleanup_static_passed_cargo_lock_blocked`。

E6/S10/F12 的当前新增落地部分是 renderer advanced plugin output mailbox 的测试观察 helper 清理。`graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/output_access.rs` 不再用 `#[allow(dead_code)]` 保存 `has_virtual_geometry_gpu_readback(...)`、`plugin_renderer_outputs(...)` 与 `has_particle_gpu_readback(...)`。这三个 helper 只被同目录 inline tests 和 readback collection tests 用来观察 mailbox 内容，因此现在均由 `#[cfg(test)]` 收进测试编译面。

生产路径不通过这些 observation helper 决策。`SceneRendererAdvancedPluginOutputs` 仍保留 `take_hybrid_gi_readback_outputs(...)`、`take_particle_gpu_readback_outputs(...)` 与 `take_virtual_geometry_readback_outputs(...)`，供 runtime feedback/render product drain 各自插件输出槽。守卫：`runtime_15_advanced_plugin_output_test_accessor_cleanup` 验证 `output_access.rs` 无 dead-code suppression、三个 observation helper 均 test-only、生产 take/drain 方法仍存在，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、render-product 文档和 particles runtime 文档的状态锚同步。该切片只关闭 advanced plugin output test accessor 子面；全量 F12 sweep 仍 pending。

验证：scoped rustfmt --check 通过；静态扫描确认 `output_access.rs` 无 `#[allow(dead_code)]`、三个 observation helper 均收进 `#[cfg(test)]`，且生产 take/drain 方法仍存在；带锁 standalone structure guard/status-output/core-min cargo check 均在进入测试前被当前工作区 `Cargo.lock` / `Cargo.toml` 不一致阻塞，不计通过。

## Runtime 15 M3 graphics dead-code guard module split

状态：`runtime_15_graphics_dead_code_guard_module_split_static_passed_cargo_lock_blocked`。

R4.1/M3 的当前新增落地部分是 graphics dead-code 结构守卫测试组织拆分。903 行的 graphics dead-code 单文件 guard 硬切为 folder-backed `structure_convention/graphics_dead_code/mod.rs`；layout 守卫迁入 `structure_convention/graphics_dead_code/module_layout.rs`，renderer output accessor 相关守卫迁入 `structure_convention/graphics_dead_code/renderer_output_accessors.rs`。

父模块继续持有共享 `read_repo` / `read_runtime_src` helper 和其余 graphics F12 dead-code 守卫；子模块只承接 `runtime_15_particle_gpu_readback_output_accessor_cleanup` 与 `runtime_15_advanced_plugin_output_test_accessor_cleanup`。守卫：`runtime_15_graphics_dead_code_guard_is_folder_backed` 验证旧单文件路径不存在、新 parent/child 模块存在、父模块挂载 `mod renderer_output_accessors;`、子模块包含两个迁出的 renderer output accessor 守卫、父模块行数低于近大文件阈值，并验证 Runtime 15 计划、runtime index、审查发现、结构规范和本文档状态锚同步。该切片只关闭 graphics dead-code guard 测试组织子面；完整 `runtime_15_no_oversized_test_files` 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt/static checks 通过；带锁 focused structure guard/status-output/core-min cargo check 均在进入测试前被当前工作区 `Cargo.lock` / `Cargo.toml` 不一致阻塞，不计通过。

## Runtime 15 M3 provider boilerplate guard module split

状态：`runtime_15_provider_boilerplate_guard_module_split_static_passed_cargo_lock_blocked`。

R4.1/M3 的当前新增落地部分是 provider boilerplate 结构守卫测试组织拆分。provider registration、update stats、feedback shared payload 三个守卫已从顶层 `structure_convention.rs` 迁入 `structure_convention/provider_boilerplate.rs`，与 prepare-input shared frame owner 守卫和 full provider boilerplate audit 总守卫同 owner 管理。

守卫：`runtime_15_provider_boilerplate_guard_is_folder_backed` 验证顶层聚合文件挂载 `structure_convention/provider_boilerplate.rs`，不再直接持有 `runtime_15_provider_registration_uses_shared_owner`、`runtime_15_provider_update_uses_shared_stats_owner`、`runtime_15_provider_feedback_uses_shared_payload_owner`；同时要求 `structure_convention.rs` 保持 700 行以下、`provider_boilerplate.rs` 保持 900 行以下，并验证 Runtime 15 计划、runtime index、审查发现、结构规范和本文档状态锚同步。该切片只关闭 provider boilerplate guard 测试组织子面；完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt/static checks 通过；带锁 focused structure guard/status-output/core-min cargo check 均在进入测试前被当前工作区 `Cargo.lock` / `Cargo.toml` 不一致阻塞，不计通过。

## Runtime 15 M3 facade surface guard module split

状态：`runtime_15_facade_surface_guard_module_split_static_passed_cargo_lock_blocked`。

R4.1/M3 的当前新增落地部分是 façade/prelude 结构守卫测试组织拆分。`runtime_15_prelude_covers_required_types` 与 `runtime_15_mixed_visibility_has_facade_note` 已从顶层 `structure_convention.rs` 迁入 `structure_convention/facade_surface.rs`，让 crate/subsystem prelude coverage 与 graphics façade visibility note 的结构守卫同 owner 管理。

守卫：`runtime_15_facade_surface_guard_is_folder_backed` 验证顶层聚合文件挂载 `structure_convention/facade_surface.rs`，不再直接持有 `runtime_15_prelude_covers_required_types` 与 `runtime_15_mixed_visibility_has_facade_note`；同时要求 `structure_convention.rs` 保持 500 行以下、`facade_surface.rs` 保持 700 行以下，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档和 status-output expectations 都包含本切片锚。该切片只关闭 façade/prelude guard 测试组织子面；完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt/static checks 通过；带锁 focused structure guard/status-output/core-min cargo check 均在进入测试前被当前工作区 `Cargo.lock` / `Cargo.toml` 不一致阻塞，不计通过。

## Runtime 15 M3 runtime dead-code guard module split

状态：`runtime_15_runtime_dead_code_guard_module_split_static_passed_cargo_lock_blocked`。

R4.1/M3 的当前新增落地部分是 runtime dead-code 结构守卫测试组织拆分。`runtime_15_runtime_ui_dead_code_surface_is_test_support`、`runtime_15_runtime_owned_dead_code_suppression_cleanup` 与 `runtime_15_script_host_value_descriptors_do_not_suppress_dead_code` 已从顶层 `structure_convention.rs` 迁入 `structure_convention/runtime_dead_code.rs`，让 F10/F12 runtime-owned dead-code surface 的结构守卫同 owner 管理。

守卫：`runtime_15_runtime_dead_code_guard_is_folder_backed` 验证顶层聚合文件挂载 `structure_convention/runtime_dead_code.rs`，不再直接持有 runtime UI、runtime-owned cleanup 和 script host descriptor 三段 dead-code guard；同时要求 `structure_convention.rs` 保持 180 行以下、`runtime_dead_code.rs` 保持 700 行以下，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档和 status-output expectations 都包含本切片锚。该切片只关闭 runtime dead-code guard 测试组织子面；完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt/static checks 通过；带锁 focused structure guard/status-output/core-min cargo check 均在进入测试前被当前工作区 `Cargo.lock` / `Cargo.toml` 不一致阻塞，不计通过。

## Runtime 15 M3 diagnostics guard module split

状态：`runtime_15_diagnostics_guard_module_split_static_passed_cargo_lock_blocked`。

R4.1/M3 的当前新增落地部分是 diagnostics 结构守卫测试组织拆分。`runtime_15_diagnostics_use_frame_trait_without_world_wrapper` 已从顶层 `structure_convention.rs` 迁入 `structure_convention/diagnostics_surface.rs`，让 F14 diagnostics normalization 的结构守卫和 diagnostics 文档锚同 owner 管理。

守卫：`runtime_15_diagnostics_guard_is_folder_backed` 验证顶层聚合文件挂载 `structure_convention/diagnostics_surface.rs`，不再直接持有 diagnostics guard；同时要求 `structure_convention.rs` 保持 80 行以下、`diagnostics_surface.rs` 保持 500 行以下，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档和 status-output expectations 都包含本切片锚。该切片只关闭 diagnostics guard 测试组织子面；完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt/static checks 通过；带锁 focused structure guard/status-output/core-min cargo check 均在进入测试前被当前工作区 `Cargo.lock` / `Cargo.toml` 不一致阻塞，不计通过。

## Runtime 15 F14 diagnostics normalization

状态：`runtime_15_diagnostics_frame_trait_wrapper_removed_coremin_check_passed`。

E5/S11/F14 的当前已落地部分是 diagnostics 命名和纯包装层收束。`core/runtime/diagnostics/frame_diagnostics.rs` 现在拥有 `FrameDiagnostics` / `FrameDiagnosticsStatus`，render、physics、animation diagnostics 和 `EcsFramePerformanceDiagnostics` 均通过同一 trait 暴露 domain、available 和 error 状态。`RuntimeDiagnosticsSnapshot::frame_diagnostics_statuses()` 只组合 render/physics/animation 的状态，不改动既有 `DiagnosticStore` metric paths，避免影响诊断面板和日志消费者。

`World` 现在直接持有 `EcsFramePerformanceDiagnostics`，`scene/world/performance_diagnostics.rs` 不再定义 `WorldEcsFramePerformanceDiagnostics`，也不再通过 `.0` 做纯转发。守卫：`runtime_15_diagnostics_use_frame_trait_without_world_wrapper` 验证 trait owner、runtime 子域组合、ECS `scene.ecs` domain、World 直接字段和相关计划/文档状态锚同步。行为锚：`runtime_snapshot_frame_diagnostics_statuses_preserve_subdomains` 和 `ecs_frame_performance_diagnostics_uses_scene_ecs_frame_domain`。F13 registration、update stats、feedback shared payload、prepare-input shared frame owner 样板与 full provider boilerplate audit 已由 shared-owner 子切片和总守卫收束。

## Runtime 15 F13 provider registration shared owner

状态：`runtime_15_provider_registration_shared_owner_coremin_check_passed`。

E5/S11/F13 的当前新增落地部分是 runtime provider registration 存储与 debug 样板收束。`graphics/runtime_provider/registration.rs` 现在拥有 `RuntimeProviderRegistration<P: ?Sized>`，统一保存 provider ID、priority、provider trait object 和 provider-specific debug name；`define_runtime_provider_registration!` 生成 HGI、Virtual Geometry、Solari 三个 public registration wrapper 的 `new`、`provider_id`、`priority`、`with_priority`、`provider` 和 `Debug` 实现。

这保持外部 API 名称不变，`RuntimeExtensionRegistry`、`GraphicsModule` 和 `WgpuRenderFramework` 仍消费原来的 provider-specific registration 类型；变化仅限三套 provider registration 不再各自复制字段、priority builder 和 debug 实现。守卫：`runtime_15_provider_registration_uses_shared_owner` 验证共享 owner、宏生成入口、三套 provider-specific registration 不再持有重复字段，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档和 `docs/zircon_runtime/graphics/runtime_provider/registration.md` 状态锚同步。本切片只关闭 registration 样板；update、feedback 与 prepare-input shared-owner 子切片由后续记录覆盖。

## Runtime 15 F13 provider update shared stats owner

状态：`runtime_15_provider_update_shared_stats_owner_coremin_check_passed`。

E5/S11/F13 的当前新增落地部分是 runtime provider update stats 样板收束。`graphics/runtime_provider/update.rs` 现在拥有 `RuntimeProviderUpdate<S>`，统一保存 update stats payload；`define_runtime_provider_update!` 生成 HGI 与 Virtual Geometry 两个 provider-specific update wrapper。`HybridGiRuntimeUpdate::stats()` 继续按旧 API 返回 `HybridGiRuntimeStats` by value，`VirtualGeometryRuntimeUpdate::stats()` 继续返回 `&VirtualGeometryRuntimeStats`，因此 record-submission 与测试 fixture 调用点不需要迁移。

守卫：`runtime_15_provider_update_uses_shared_stats_owner` 验证共享 owner、宏生成入口、两套 provider-specific update 不再声明自己的 `stats` 字段，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档和 `docs/zircon_runtime/graphics/runtime_provider/update.md` 状态锚同步。验证边界：scoped rustfmt 与 core-min `cargo check` 已通过；standalone guard/status-output binary 启动被 Windows `ResourceUnavailable` / 用户取消状态阻断，focused Cargo test 超时无结果，不计为通过。本切片只关闭 update stats 样板；feedback 与 prepare-input shared-owner 子切片由后续记录覆盖。

## Runtime 15 F13 provider feedback shared payload owner

状态：`runtime_15_provider_feedback_shared_payload_owner_coremin_check_passed`。

E5/S11/F13 的当前新增落地部分是 runtime provider feedback 共同 payload 样板收束。`graphics/runtime_provider/feedback.rs` 现在拥有 `RuntimeProviderFeedback<G, V>`，统一保存 `gpu_completion` 与 `visibility_feedback` 两个 provider feedback 共同字段；HGI 与 Virtual Geometry 的 public feedback wrapper 继续保留原类型名、constructor 和 getter surface。

该切片刻意不合并 feature-specific 字段：HGI 的 `evictable_probe_ids` 仍由 `HybridGiRuntimeFeedback` 拥有；Virtual Geometry 的 `node_and_cluster_cull_page_requests`、`evictable_page_ids` 与 `generation` 仍由 `VirtualGeometryRuntimeFeedback` 拥有。守卫：`runtime_15_provider_feedback_uses_shared_payload_owner` 验证共享 owner、runtime_provider 挂载、两套 provider-specific feedback 不再声明共同 payload 字段，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档和 `docs/zircon_runtime/graphics/runtime_provider/feedback.md` 状态锚同步。验证：scoped rustfmt、standalone structure guard、standalone status-output guards 与 core-min `cargo check` 已通过（既有 warnings）。本切片只关闭 feedback 共同 payload 样板；prepare-input shared frame owner 由后续记录覆盖。

## Runtime 15 F13 provider prepare input shared frame owner

状态：`runtime_15_provider_prepare_input_shared_frame_owner_coremin_check_passed`。

E5/S11/F13 的当前新增落地部分是 runtime provider prepare input 共同帧字段收束。`graphics/runtime_provider/prepare_input.rs` 现在拥有 `RuntimeProviderPrepareInput<'a, E>`，统一保存 provider prepare 阶段共同的 optional extract 与 frame generation。HGI 与 Virtual Geometry 的 public prepare input wrapper 继续保留原类型名、constructor 参数和 getter surface。

该切片不合并 feature-specific 输入：HGI 的 mesh snapshots、三类 light snapshots 与 `VisibilityHybridGiUpdatePlan` 仍由 `HybridGiRuntimePrepareInput` 拥有；Virtual Geometry 的 page upload plan、visible clusters 与 draw segments 仍由 `VirtualGeometryRuntimePrepareInput` 拥有。守卫：`runtime_15_provider_prepare_input_uses_shared_extract_generation_owner` 验证共享 owner、runtime_provider 挂载、两套 provider-specific prepare input 不再声明共同 `extract` / `generation` 字段，并验证 Runtime 15 计划、runtime index、render index、审查发现、结构规范、本文档和 `docs/zircon_runtime/graphics/runtime_provider/prepare_input.md` 状态锚同步。验证：scoped rustfmt、standalone structure guard 1/1、standalone status-output all-subplans guard 1/1 与 core-min `cargo check` 已通过（既有 warnings）；broader `status_output` 批次仍有非本切片 Runtime 06 F8 row-drift 失败。

## Runtime 15 F13 full provider boilerplate audit

状态：`runtime_15_provider_boilerplate_full_audit_coremin_check_passed`。

E5/S11/F13 的当前总验收是 provider boilerplate 总守卫。`structure_convention/provider_boilerplate.rs` 现在包含 `runtime_15_no_duplicated_provider_boilerplate`，把 registration、update、feedback、prepare input 四个 shared-owner 子切片作为一个整体审计。

守卫要求 `graphics/runtime_provider/{registration,update,feedback,prepare_input}.rs` 均挂载共享 owner；HGI、Virtual Geometry、Solari registration 文件只使用 `define_runtime_provider_registration!`，不再复制 provider id / priority / trait-object / debug 样板；HGI/VG update 文件只使用 `define_runtime_provider_update!`，不再手写 constructor / stats getter；HGI/VG feedback 文件委托 `RuntimeProviderFeedback<G, V>`，不再复制共同 GPU completion / visibility feedback 字段；HGI/VG prepare-input 文件委托 `RuntimeProviderPrepareInput<'a, E>`，不再复制共同 optional extract / generation 字段。Particle feedback 只有 `ParticleGpuFeedback` 且没有 visibility feedback payload，因此作为 feature-specific 单 payload 例外记录，不强行套入双 payload owner。

状态输出期望行同步到 `expected_status_row_data.rs` 和 `expected_slices/{status,date}.rs`。验证：scoped rustfmt --check 通过；standalone full provider boilerplate guard 1/1 通过；standalone status-output all-subplans guard 1/1 通过；core-min `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime15-provider-boilerplate-full-coremin-0622` 通过（既有 141 warnings）。完整 `module_convention_gate`、全量 dead-code sweep 与测试组织拆分仍 pending。
