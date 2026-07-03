#[path = "structure_convention/animation_manager.rs"]
mod animation_manager;
#[path = "structure_convention/diagnostics_surface.rs"]
mod diagnostics_surface;
#[path = "structure_convention/facade_surface.rs"]
mod facade_surface;
#[path = "structure_convention/graphics_dead_code/mod.rs"]
mod graphics_dead_code;
#[path = "structure_convention/lock_poison_policy.rs"]
mod lock_poison_policy;
#[path = "structure_convention/module_convention_gate.rs"]
mod module_convention_gate;
#[path = "structure_convention/native_live_host_lock_poison.rs"]
mod native_live_host_lock_poison;
#[path = "structure_convention/production_file_budget.rs"]
mod production_file_budget;
#[path = "structure_convention/provider_boilerplate.rs"]
mod provider_boilerplate;
#[path = "structure_convention/render_builtin_postprocess_executors.rs"]
mod render_builtin_postprocess_executors;
#[path = "structure_convention/render_graph_execution_record.rs"]
mod render_graph_execution_record;
#[path = "structure_convention/render_mesh_draw_command_list.rs"]
mod render_mesh_draw_command_list;
#[path = "structure_convention/render_mesh_pass_processors.rs"]
mod render_mesh_pass_processors;
#[path = "structure_convention/render_pending_command_cache_material_boundary.rs"]
mod render_pending_command_cache_material_boundary;
#[path = "structure_convention/render_pending_command_cache_plan.rs"]
mod render_pending_command_cache_plan;
#[path = "structure_convention/render_post_process_stack.rs"]
mod render_post_process_stack;
#[path = "structure_convention/render_post_process_volume_component.rs"]
mod render_post_process_volume_component;
#[path = "structure_convention/render_prepared_mesh_queue.rs"]
mod render_prepared_mesh_queue;
#[path = "structure_convention/rhi_wgpu_lock_poison.rs"]
mod rhi_wgpu_lock_poison;
#[path = "structure_convention/runtime_dead_code/mod.rs"]
mod runtime_dead_code;
#[path = "structure_convention/script_vm_lock_poison.rs"]
mod script_vm_lock_poison;
#[path = "structure_convention/test_file_budget/mod.rs"]
mod test_file_budget;

fn assert_contains_all(label: &str, source: &str, required: &[&str]) {
    let missing: Vec<_> = required
        .iter()
        .copied()
        .filter(|anchor| !source.contains(anchor))
        .collect();
    assert!(
        missing.is_empty(),
        "{label} missing required anchors: {missing:?}"
    );
}

fn runtime_src_path(relative: &str) -> std::path::PathBuf {
    if let Some(manifest_dir) = option_env!("CARGO_MANIFEST_DIR") {
        std::path::PathBuf::from(manifest_dir)
            .join("src")
            .join(relative)
    } else {
        std::path::PathBuf::from("zircon_runtime")
            .join("src")
            .join(relative)
    }
}

fn repo_path(relative: &str) -> std::path::PathBuf {
    if let Some(manifest_dir) = option_env!("CARGO_MANIFEST_DIR") {
        std::path::PathBuf::from(manifest_dir)
            .parent()
            .expect("zircon_runtime manifest should live under repository root")
            .join(relative)
    } else {
        std::path::PathBuf::from(relative)
    }
}
