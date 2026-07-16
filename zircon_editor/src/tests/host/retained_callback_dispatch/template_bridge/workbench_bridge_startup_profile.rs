use std::time::Instant;

use super::super::support::*;
use crate::ui::control::EditorUiControlService;
use crate::ui::retained_host::callback_dispatch::load_startup_builtin_template_runtime;
use crate::ui::template_runtime::{RetainedUiNodeProjection, WORKBENCH_WINDOW_DOCUMENT_ID};
use crate::ui::workbench::reference::EditorWorkbenchReferenceMetrics;

fn retained_node_count(node: &RetainedUiNodeProjection) -> usize {
    1 + node.children.iter().map(retained_node_count).sum::<usize>()
}

#[test]
#[ignore = "diagnostic profile: run manually while repairing the shared Workbench V2 bridge path"]
fn profiles_startup_workbench_v2_surface_boundaries() {
    let _guard = env_lock()
        .lock()
        .expect("test environment lock should be available");
    let started = Instant::now();

    let runtime = load_startup_builtin_template_runtime()
        .expect("startup template runtime should load for the V2 bridge profile");
    eprintln!(
        "workbench-v2-profile stage=startup-runtime elapsed_ms={}",
        started.elapsed().as_millis()
    );

    let mut projection = runtime
        .project_document(WORKBENCH_WINDOW_DOCUMENT_ID)
        .expect("workbench document projection should be available");
    let mut route_service = EditorUiControlService::default();
    runtime
        .register_projection_routes(&mut route_service, &mut projection)
        .expect("workbench projection routes should register");
    eprintln!(
        "workbench-v2-profile stage=document-projection elapsed_ms={} nodes={} bindings={}",
        started.elapsed().as_millis(),
        retained_node_count(&projection.root),
        projection.bindings.len()
    );

    let mut surface = runtime
        .build_shared_surface(WORKBENCH_WINDOW_DOCUMENT_ID)
        .expect("workbench V2 surface should build");
    eprintln!(
        "workbench-v2-profile stage=surface-build elapsed_ms={} nodes={}",
        started.elapsed().as_millis(),
        surface.tree.nodes.len()
    );

    surface
        .compute_layout(EditorWorkbenchReferenceMetrics::default().target_size())
        .expect("workbench V2 surface should lay out");
    let rebuild = surface.last_rebuild_report;
    eprintln!(
        "workbench-v2-profile stage=surface-layout elapsed_ms={} render_commands={} layout_ms={:.3} arranged_ms={:.3} hit_grid_ms={:.3} render_extract_ms={:.3}",
        started.elapsed().as_millis(),
        surface.render_extract.list.commands.len(),
        rebuild.layout_elapsed_micros as f64 / 1_000.0,
        rebuild.arranged_elapsed_micros as f64 / 1_000.0,
        rebuild.hit_grid_elapsed_micros as f64 / 1_000.0,
        rebuild.render_elapsed_micros as f64 / 1_000.0,
    );

    let host_projection = runtime
        .build_retained_host_projection_with_surface(&projection, &surface)
        .expect("workbench host projection should be generated from the surface");
    eprintln!(
        "workbench-v2-profile stage=host-projection elapsed_ms={} nodes={}",
        started.elapsed().as_millis(),
        host_projection.nodes.len()
    );

    assert!(
        surface.tree.nodes.len() > 1_000,
        "profile must exercise the componentized Workbench surface rather than a reduced fixture"
    );
    assert!(
        host_projection
            .node_by_control_id("WorkbenchWindowRoot")
            .is_some(),
        "profile must retain the authored Workbench root"
    );
}
