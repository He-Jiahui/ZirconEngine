//! 运行时结构约束的测试入口；子模块分别检查所有权拆分、锁中毒策略和迁移证据。
#[path = "structure/animation_manager.rs"]
mod animation_manager;
#[path = "structure/diagnostics_surface.rs"]
mod diagnostics_surface;
#[path = "structure/evidence_ownership.rs"]
mod evidence_ownership;
#[path = "structure/facade_surface.rs"]
mod facade_surface;
#[path = "structure/ffi_panic_boundary.rs"]
mod ffi_panic_boundary;
#[path = "structure/graphics_dead_code/mod.rs"]
mod graphics_dead_code;
#[path = "structure/lock_poison_policy.rs"]
mod lock_poison_policy;
#[path = "structure/module_convention_gate.rs"]
mod module_convention_gate;
#[path = "structure/native_live_host_lock_poison.rs"]
mod native_live_host_lock_poison;
#[path = "structure/source_size.rs"]
mod production_file_budget;
#[path = "structure/provider_boilerplate.rs"]
mod provider_boilerplate;
#[path = "../structure/render_builtin_postprocess_executors.rs"]
mod render_builtin_postprocess_executors;
#[path = "structure/render_graph_execution_record.rs"]
mod render_graph_execution_record;
#[path = "structure/render_mesh_draw_command_list.rs"]
mod render_mesh_draw_command_list;
#[path = "structure/render_mesh_pass_processors.rs"]
mod render_mesh_pass_processors;
#[path = "../render_pending_command_cache_material_boundary.rs"]
mod render_pending_command_cache_material_boundary;
#[path = "../structure/render_pending_command_cache_plan.rs"]
mod render_pending_command_cache_plan;
#[path = "structure/render_post_process_stack.rs"]
mod render_post_process_stack;
#[path = "../structure/render_post_process_volume_component.rs"]
mod render_post_process_volume_component;
#[path = "structure/render_prepared_mesh_queue.rs"]
mod render_prepared_mesh_queue;
#[path = "structure/rhi_wgpu_lock_poison.rs"]
mod rhi_wgpu_lock_poison;
#[path = "structure/runtime_dead_code/mod.rs"]
mod runtime_dead_code;
#[path = "structure/rust_source_view.rs"]
mod rust_source_view;
#[path = "structure/script_vm_lock_poison.rs"]
mod script_vm_lock_poison;
#[path = "structure/shader_prewarm_asset_inventory.rs"]
mod shader_prewarm_asset_inventory;
#[path = "structure/support.rs"]
mod support;
#[path = "structure/test_size/mod.rs"]
mod test_file_budget;

pub(super) use support::priority_plan_doc_current_owner_archive_source;
use support::{assert_contains_all, assert_contains_all_exact, repo_path, runtime_src_path};
