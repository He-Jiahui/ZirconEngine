#[test]
fn runtime_07_render_product_diagnostics_owner_split_keeps_families_folder_backed() {
    let product_root =
        include_str!("../../../../core/runtime/diagnostics/render_stats_store/product.rs");
    let camera =
        include_str!("../../../../core/runtime/diagnostics/render_stats_store/product/camera.rs");
    let visibility = include_str!(
        "../../../../core/runtime/diagnostics/render_stats_store/product/visibility.rs"
    );
    let hzb =
        include_str!("../../../../core/runtime/diagnostics/render_stats_store/product/hzb.rs");
    let light_grid = include_str!(
        "../../../../core/runtime/diagnostics/render_stats_store/product/light_grid.rs"
    );
    let effect_stack = include_str!(
        "../../../../core/runtime/diagnostics/render_stats_store/product/effect_stack.rs"
    );
    let material =
        include_str!("../../../../core/runtime/diagnostics/render_stats_store/product/material.rs");
    let light =
        include_str!("../../../../core/runtime/diagnostics/render_stats_store/product/light.rs");
    let mesh_queue = include_str!(
        "../../../../core/runtime/diagnostics/render_stats_store/product/mesh_queue.rs"
    );
    let gpu_scene = include_str!(
        "../../../../core/runtime/diagnostics/render_stats_store/product/gpu_scene.rs"
    );
    let sprite =
        include_str!("../../../../core/runtime/diagnostics/render_stats_store/product/sprite.rs");
    let ui = include_str!("../../../../core/runtime/diagnostics/render_stats_store/product/ui.rs");
    let diagnostics_doc = include_str!("../../../../../../docs/zircon_runtime/core/diagnostics.md");
    let runtime_07_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let hotspot_doc =
        include_str!("../../../../../../docs/zircon_runtime/performance/hotspot_inventory.md");
    let large_file_doc =
        include_str!("../../../../../../docs/engine-architecture/large-file-ownership-m1.md");

    for module_decl in [
        "mod camera;",
        "mod visibility;",
        "mod hzb;",
        "mod light_grid;",
        "mod effect_stack;",
        "mod material;",
        "mod light;",
        "mod mesh_queue;",
        "mod gpu_scene;",
        "mod sprite;",
        "mod ui;",
    ] {
        assert!(
            product_root.contains(module_decl),
            "product.rs should keep render product diagnostic child declaration `{module_decl}`"
        );
    }

    for root_dispatch_anchor in [
        "camera::record(store, stats);",
        "visibility::record(store, stats);",
        "hzb::record(store, stats);",
        "light_grid::record(store, stats);",
        "material::record(store, stats);",
        "light::record(store, stats);",
        "mesh_queue::record(store, stats);",
        "gpu_scene::record(store, stats);",
        "sprite::record(store, stats);",
        "effect_stack::record(store, stats);",
        "ui::record(store, stats);",
    ] {
        assert!(
            product_root.contains(root_dispatch_anchor),
            "product.rs should keep only render product dispatch anchor `{root_dispatch_anchor}`"
        );
    }

    for moved_owner_anchor in [
        "fn record_camera",
        "fn record_visibility",
        "fn record_hzb",
        "fn record_light_grid",
        "fn record_light_family",
        "fn record_mesh_queue",
        "fn record_gpu_scene",
        "fn record_sprite",
        "fn record_ui",
    ] {
        assert!(
            !product_root.contains(moved_owner_anchor),
            "product.rs should not reclaim moved render product owner `{moved_owner_anchor}`"
        );
    }

    for (module_name, module_source, expected_anchor) in [
        (
            "camera",
            camera,
            "render.camera.target.graph_import.ready_for_direct_import",
        ),
        (
            "visibility",
            visibility,
            "render.visibility.static_index.main_view_static_candidate_count",
        ),
        ("hzb", hzb, "render.hzb.occlusion.remaining_instance_count"),
        ("light_grid", light_grid, "render.light_grid.tile_count"),
        (
            "effect_stack",
            effect_stack,
            "render.post_process.motion_vector.camera.ready",
        ),
        ("material", material, "render.material.ready_count"),
        ("light", light, "render.light.directional.ready_count"),
        (
            "mesh_queue",
            mesh_queue,
            "render.mesh.queue.cached_command_hit_count",
        ),
        ("gpu_scene", gpu_scene, "render.gpu_scene.uploaded_bytes"),
        (
            "sprite",
            sprite,
            "render.sprite.queue.transparent_draw_batch_count",
        ),
        ("ui", ui, "render.ui.graph_executed_pass_count"),
    ] {
        assert!(
            module_source.contains("pub(super) fn record")
                && module_source.contains(expected_anchor),
            "render_stats_store/product/{module_name}.rs should own `{expected_anchor}`"
        );
    }

    for doc_anchor in [
        "render_product_diagnostics_owner_split_static_passed_cargo_deferred",
        "render_stats_store/product/{camera,mesh_queue,gpu_scene}.rs",
        "render_stats_store/product/{camera,visibility,hzb,light_grid,effect_stack,material,light,mesh_queue,gpu_scene,sprite,ui}.rs",
        "large_file_hotspot_count = 39",
        "runtime-other = 15",
        "hotspot_guard_anchor_count = 25",
        "expected_source_file_count = 38",
        "runtime_07_render_product_diagnostics_owner_split_keeps_families_folder_backed",
    ] {
        assert!(
            diagnostics_doc.contains(doc_anchor)
                || runtime_07_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || hotspot_doc.contains(doc_anchor)
                || large_file_doc.contains(doc_anchor),
            "render product diagnostics owner split docs should retain `{doc_anchor}`"
        );
    }
}
