use crate::core::framework::render::{
    RenderHybridGiExtract, RenderHybridGiProbe, RenderHybridGiTraceRegion,
};
use crate::core::math::{Transform, Vec3};
use crate::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};
use crate::scene::world::World;

use crate::graphics::{
    VisibilityContext, VisibilityHybridGiFeedback, VisibilityHybridGiProbe,
    VisibilityHybridGiUpdatePlan,
};

#[test]
fn visibility_context_builds_hybrid_gi_probe_and_trace_plan() {
    let mut world = World::new();
    remove_default_meshes(&mut world);

    let mesh = world.spawn_mesh_node(
        model_handle("res://models/hybrid_gi.obj"),
        material_handle("res://materials/hybrid_gi.material.toml"),
    );
    world
        .update_transform(mesh, Transform::from_translation(Vec3::ZERO))
        .expect("mesh transform should update");

    let mut extract = world.to_render_frame_extract();
    extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
        probe_budget: 1,
        tracing_budget: 1,
        probes: vec![
            hybrid_probe(mesh, 30, false, Vec3::ZERO, 128),
            hybrid_probe(mesh, 20, true, Vec3::new(0.1, 0.0, 0.0), 64),
            hybrid_probe(mesh, 10, false, Vec3::new(100.0, 0.0, 0.0), 32),
        ],
        trace_regions: vec![
            hybrid_trace_region(mesh, 40, Vec3::ZERO, 8.0),
            hybrid_trace_region(mesh, 50, Vec3::new(0.1, 0.0, 0.0), 5.0),
            hybrid_trace_region(mesh, 60, Vec3::new(100.0, 0.0, 0.0), 10.0),
        ],
    });

    let context = VisibilityContext::from(&extract);

    assert_eq!(
        context.hybrid_gi_active_probes,
        vec![
            VisibilityHybridGiProbe {
                entity: mesh,
                probe_id: 30,
                resident: false,
                ray_budget: 128,
            },
            VisibilityHybridGiProbe {
                entity: mesh,
                probe_id: 20,
                resident: true,
                ray_budget: 64,
            },
        ]
    );
    assert_eq!(
        context.hybrid_gi_update_plan,
        VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![20],
            requested_probe_ids: vec![30],
            dirty_requested_probe_ids: vec![30],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        }
    );
    assert_eq!(
        context.hybrid_gi_feedback,
        VisibilityHybridGiFeedback {
            active_probe_ids: vec![30, 20],
            requested_probe_ids: vec![30],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        }
    );
    assert_eq!(
        context.history_snapshot.hybrid_gi_requested_probes,
        vec![30]
    );
}

#[test]
fn visibility_context_with_history_tracks_hybrid_gi_requested_probes() {
    let mut world = World::new();
    remove_default_meshes(&mut world);

    let mesh = world.spawn_mesh_node(
        model_handle("res://models/hybrid_gi.obj"),
        material_handle("res://materials/hybrid_gi.material.toml"),
    );
    world
        .update_transform(mesh, Transform::from_translation(Vec3::ZERO))
        .expect("mesh transform should update");

    let mut previous_extract = world.to_render_frame_extract();
    previous_extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
        probe_budget: 2,
        tracing_budget: 2,
        probes: vec![
            hybrid_probe(mesh, 300, false, Vec3::ZERO, 96),
            hybrid_probe(mesh, 200, true, Vec3::new(0.1, 0.0, 0.0), 64),
        ],
        trace_regions: vec![
            hybrid_trace_region(mesh, 40, Vec3::ZERO, 7.0),
            hybrid_trace_region(mesh, 20, Vec3::new(0.1, 0.0, 0.0), 4.0),
        ],
    });
    let previous_context = VisibilityContext::from(&previous_extract);

    let mut current_extract = world.to_render_frame_extract();
    current_extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
        probe_budget: 2,
        tracing_budget: 2,
        probes: vec![
            hybrid_probe(mesh, 600, false, Vec3::ZERO, 128),
            hybrid_probe(mesh, 300, false, Vec3::new(0.1, 0.0, 0.0), 96),
            hybrid_probe(mesh, 200, true, Vec3::new(0.2, 0.0, 0.0), 64),
            hybrid_probe(mesh, 700, true, Vec3::new(100.0, 0.0, 0.0), 32),
        ],
        trace_regions: vec![
            hybrid_trace_region(mesh, 80, Vec3::ZERO, 10.0),
            hybrid_trace_region(mesh, 40, Vec3::new(0.1, 0.0, 0.0), 7.0),
            hybrid_trace_region(mesh, 20, Vec3::new(0.2, 0.0, 0.0), 4.0),
        ],
    });

    let context = VisibilityContext::from_extract_with_history(
        &current_extract,
        Some(&previous_context.history_snapshot),
    );

    assert_eq!(
        context.hybrid_gi_update_plan,
        VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![200, 700],
            requested_probe_ids: vec![600, 300],
            dirty_requested_probe_ids: vec![600],
            scheduled_trace_region_ids: vec![80, 40],
            evictable_probe_ids: vec![700],
        }
    );
    assert_eq!(
        context.hybrid_gi_feedback,
        VisibilityHybridGiFeedback {
            active_probe_ids: vec![600, 300, 200],
            requested_probe_ids: vec![600, 300],
            scheduled_trace_region_ids: vec![80, 40],
            evictable_probe_ids: vec![700],
        }
    );
    assert_eq!(
        context.history_snapshot.hybrid_gi_requested_probes,
        vec![600, 300]
    );
}

#[test]
fn visibility_context_prioritizes_hybrid_gi_probe_requests_supported_by_scheduled_trace_regions() {
    let mut world = World::new();
    remove_default_meshes(&mut world);

    let mesh = world.spawn_mesh_node(
        model_handle("res://models/hybrid_gi.obj"),
        material_handle("res://materials/hybrid_gi.material.toml"),
    );
    world
        .update_transform(mesh, Transform::from_translation(Vec3::ZERO))
        .expect("mesh transform should update");

    let mut extract = world.to_render_frame_extract();
    extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
        probe_budget: 1,
        tracing_budget: 1,
        probes: vec![
            hybrid_probe(mesh, 30, false, Vec3::new(1.4, 0.0, 0.0), 128),
            hybrid_probe(mesh, 20, false, Vec3::new(0.05, 0.0, 0.0), 64),
            hybrid_probe(mesh, 10, true, Vec3::new(-0.25, 0.0, 0.0), 32),
        ],
        trace_regions: vec![hybrid_trace_region(mesh, 40, Vec3::ZERO, 8.0)],
    });

    let context = VisibilityContext::from(&extract);

    assert_eq!(
        context.hybrid_gi_update_plan,
        VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![10],
            requested_probe_ids: vec![20],
            dirty_requested_probe_ids: vec![20],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        }
    );
    assert_eq!(
        context.hybrid_gi_feedback,
        VisibilityHybridGiFeedback {
            active_probe_ids: vec![30, 20, 10],
            requested_probe_ids: vec![20],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        }
    );
}

#[test]
fn visibility_context_prefers_previously_requested_hybrid_gi_lineage_when_trace_schedule_clears() {
    let mut world = World::new();
    remove_default_meshes(&mut world);

    let mesh = world.spawn_mesh_node(
        model_handle("res://models/hybrid_gi.obj"),
        material_handle("res://materials/hybrid_gi.material.toml"),
    );
    world
        .update_transform(mesh, Transform::from_translation(Vec3::ZERO))
        .expect("mesh transform should update");

    let mut previous_extract = world.to_render_frame_extract();
    previous_extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
        probe_budget: 1,
        tracing_budget: 1,
        probes: vec![
            hybrid_probe(mesh, 10, true, Vec3::new(-2.0, 0.0, 0.0), 96),
            hybrid_probe_with_parent(mesh, 30, false, Vec3::new(-1.8, 0.0, 0.0), 96, 10),
            hybrid_probe(mesh, 40, true, Vec3::new(2.0, 0.0, 0.0), 96),
            hybrid_probe_with_parent(mesh, 60, false, Vec3::new(2.2, 0.0, 0.0), 96, 40),
        ],
        trace_regions: vec![hybrid_trace_region(mesh, 80, Vec3::new(2.1, 0.0, 0.0), 8.0)],
    });
    let previous_context = VisibilityContext::from(&previous_extract);
    assert_eq!(
        previous_context.history_snapshot.hybrid_gi_requested_probes,
        vec![60],
        "expected the warm frame to pick the right-hand lineage before testing history-driven request continuation"
    );

    let mut current_extract = world.to_render_frame_extract();
    current_extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
        probe_budget: 1,
        tracing_budget: 0,
        probes: vec![
            hybrid_probe(mesh, 10, true, Vec3::new(-2.0, 0.0, 0.0), 96),
            hybrid_probe_with_parent(mesh, 30, false, Vec3::new(-1.8, 0.0, 0.0), 96, 10),
            hybrid_probe(mesh, 40, true, Vec3::new(2.0, 0.0, 0.0), 96),
            hybrid_probe_with_parent(mesh, 60, false, Vec3::new(2.2, 0.0, 0.0), 96, 40),
        ],
        trace_regions: Vec::new(),
    });

    let without_history = VisibilityContext::from(&current_extract);
    assert_eq!(
        without_history.hybrid_gi_update_plan.requested_probe_ids,
        vec![30],
        "expected the no-history fallback to collapse back onto the lower-id lineage once the current trace schedule disappears"
    );

    let context = VisibilityContext::from_extract_with_history(
        &current_extract,
        Some(&previous_context.history_snapshot),
    );

    assert_eq!(
        context.hybrid_gi_update_plan,
        VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![10, 40],
            requested_probe_ids: vec![60],
            dirty_requested_probe_ids: Vec::new(),
            scheduled_trace_region_ids: Vec::new(),
            evictable_probe_ids: Vec::new(),
        },
        "expected a previously requested screen-probe lineage to keep ownership for one more frame after the current trace schedule clears, instead of immediately flipping to an unrelated lineage on probe-id tie-breaks"
    );
    assert_eq!(
        context.hybrid_gi_feedback,
        VisibilityHybridGiFeedback {
            active_probe_ids: vec![10, 40],
            requested_probe_ids: vec![60],
            scheduled_trace_region_ids: Vec::new(),
            evictable_probe_ids: Vec::new(),
        },
        "expected feedback/history to publish the continued requested lineage so runtime-host gather/request remains scene-driven even through a temporary trace-schedule gap"
    );
}

#[test]
fn visibility_context_holds_newly_resident_hybrid_gi_probe_out_of_evictable_list_for_one_frame() {
    let mut world = World::new();
    remove_default_meshes(&mut world);

    let mesh = world.spawn_mesh_node(
        model_handle("res://models/hybrid_gi.obj"),
        material_handle("res://materials/hybrid_gi.material.toml"),
    );
    world
        .update_transform(mesh, Transform::from_translation(Vec3::ZERO))
        .expect("mesh transform should update");

    let mut previous_extract = world.to_render_frame_extract();
    previous_extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
        probe_budget: 2,
        tracing_budget: 1,
        probes: vec![
            hybrid_probe(mesh, 300, false, Vec3::ZERO, 96),
            hybrid_probe(mesh, 200, true, Vec3::new(0.1, 0.0, 0.0), 64),
        ],
        trace_regions: vec![hybrid_trace_region(mesh, 40, Vec3::ZERO, 7.0)],
    });
    let previous_context = VisibilityContext::from(&previous_extract);

    let mut current_extract = world.to_render_frame_extract();
    current_extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
        probe_budget: 2,
        tracing_budget: 1,
        probes: vec![
            hybrid_probe(mesh, 300, true, Vec3::new(100.0, 0.0, 0.0), 96),
            hybrid_probe(mesh, 200, true, Vec3::ZERO, 64),
        ],
        trace_regions: vec![hybrid_trace_region(mesh, 40, Vec3::ZERO, 7.0)],
    });

    let held_context = VisibilityContext::from_extract_with_history(
        &current_extract,
        Some(&previous_context.history_snapshot),
    );

    assert_eq!(
        held_context.hybrid_gi_update_plan,
        VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![300, 200],
            requested_probe_ids: Vec::new(),
            dirty_requested_probe_ids: Vec::new(),
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        }
    );
    assert_eq!(
        held_context.hybrid_gi_feedback,
        VisibilityHybridGiFeedback {
            active_probe_ids: vec![200],
            requested_probe_ids: Vec::new(),
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        }
    );

    let settled_context = VisibilityContext::from_extract_with_history(
        &current_extract,
        Some(&held_context.history_snapshot),
    );

    assert_eq!(
        settled_context.hybrid_gi_update_plan,
        VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![300, 200],
            requested_probe_ids: Vec::new(),
            dirty_requested_probe_ids: Vec::new(),
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: vec![300],
        }
    );
    assert_eq!(
        settled_context.hybrid_gi_feedback,
        VisibilityHybridGiFeedback {
            active_probe_ids: vec![200],
            requested_probe_ids: Vec::new(),
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: vec![300],
        }
    );
}

#[test]
fn visibility_context_keeps_hybrid_gi_parent_probe_visible_while_requesting_nonresident_children() {
    let mut world = World::new();
    remove_default_meshes(&mut world);

    let mesh = world.spawn_mesh_node(
        model_handle("res://models/hybrid_gi.obj"),
        material_handle("res://materials/hybrid_gi.material.toml"),
    );
    world
        .update_transform(mesh, Transform::from_translation(Vec3::ZERO))
        .expect("mesh transform should update");

    let mut extract = world.to_render_frame_extract();
    extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
        probe_budget: 2,
        tracing_budget: 1,
        probes: vec![
            hybrid_probe(mesh, 10, true, Vec3::ZERO, 128),
            hybrid_probe_with_parent(mesh, 20, false, Vec3::new(0.08, 0.0, 0.0), 96, 10),
            hybrid_probe_with_parent(mesh, 30, false, Vec3::new(-0.08, 0.0, 0.0), 80, 10),
        ],
        trace_regions: vec![hybrid_trace_region(mesh, 40, Vec3::ZERO, 8.0)],
    });

    let context = VisibilityContext::from(&extract);

    assert_eq!(
        context.hybrid_gi_active_probes,
        vec![VisibilityHybridGiProbe {
            entity: mesh,
            probe_id: 10,
            resident: true,
            ray_budget: 128,
        }]
    );
    assert_eq!(
        context.hybrid_gi_update_plan,
        VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![10],
            requested_probe_ids: vec![20, 30],
            dirty_requested_probe_ids: vec![20, 30],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        }
    );
    assert_eq!(
        context.hybrid_gi_feedback,
        VisibilityHybridGiFeedback {
            active_probe_ids: vec![10],
            requested_probe_ids: vec![20, 30],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        }
    );
}

#[test]
fn visibility_context_requests_nonresident_hybrid_gi_descendant_supported_by_trace_regions() {
    let mut world = World::new();
    remove_default_meshes(&mut world);

    let mesh = world.spawn_mesh_node(
        model_handle("res://models/hybrid_gi.obj"),
        material_handle("res://materials/hybrid_gi.material.toml"),
    );
    world
        .update_transform(mesh, Transform::from_translation(Vec3::ZERO))
        .expect("mesh transform should update");

    let mut extract = world.to_render_frame_extract();
    extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
        probe_budget: 1,
        tracing_budget: 1,
        probes: vec![
            hybrid_probe(mesh, 10, true, Vec3::new(0.3, 0.0, 0.0), 96),
            hybrid_probe_with_parent(mesh, 20, false, Vec3::new(0.12, 0.0, 0.0), 80, 10),
            hybrid_probe_with_parent(mesh, 30, false, Vec3::new(0.02, 0.0, 0.0), 128, 20),
        ],
        trace_regions: vec![hybrid_trace_region(mesh, 40, Vec3::ZERO, 8.0)],
    });

    let context = VisibilityContext::from(&extract);

    assert_eq!(
        context.hybrid_gi_active_probes,
        vec![VisibilityHybridGiProbe {
            entity: mesh,
            probe_id: 10,
            resident: true,
            ray_budget: 96,
        }]
    );
    assert_eq!(
        context.hybrid_gi_update_plan,
        VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![10],
            requested_probe_ids: vec![30],
            dirty_requested_probe_ids: vec![30],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        }
    );
    assert_eq!(
        context.hybrid_gi_feedback,
        VisibilityHybridGiFeedback {
            active_probe_ids: vec![10],
            requested_probe_ids: vec![30],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        }
    );
}

#[test]
fn visibility_context_prefers_deeper_nonresident_hybrid_gi_descendant_when_trace_support_ties() {
    let mut world = World::new();
    remove_default_meshes(&mut world);

    let mesh = world.spawn_mesh_node(
        model_handle("res://models/hybrid_gi.obj"),
        material_handle("res://materials/hybrid_gi.material.toml"),
    );
    world
        .update_transform(mesh, Transform::from_translation(Vec3::ZERO))
        .expect("mesh transform should update");

    let mut extract = world.to_render_frame_extract();
    extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
        probe_budget: 1,
        tracing_budget: 1,
        probes: vec![
            hybrid_probe(mesh, 10, true, Vec3::new(0.3, 0.0, 0.0), 96),
            hybrid_probe_with_parent(mesh, 20, false, Vec3::new(0.05, 0.0, 0.0), 128, 10),
            hybrid_probe_with_parent(mesh, 30, false, Vec3::new(0.05, 0.0, 0.0), 64, 20),
        ],
        trace_regions: vec![hybrid_trace_region(mesh, 40, Vec3::ZERO, 8.0)],
    });

    let context = VisibilityContext::from(&extract);

    assert_eq!(
        context.hybrid_gi_update_plan,
        VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![10],
            requested_probe_ids: vec![30],
            dirty_requested_probe_ids: vec![30],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        }
    );
    assert_eq!(
        context.hybrid_gi_feedback,
        VisibilityHybridGiFeedback {
            active_probe_ids: vec![10],
            requested_probe_ids: vec![30],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        }
    );
}

#[test]
fn visibility_context_prefers_nonresident_hybrid_gi_descendant_supported_by_ancestor_trace_lineage()
{
    let mut world = World::new();
    remove_default_meshes(&mut world);

    let mesh = world.spawn_mesh_node(
        model_handle("res://models/hybrid_gi.obj"),
        material_handle("res://materials/hybrid_gi.material.toml"),
    );
    world
        .update_transform(mesh, Transform::from_translation(Vec3::ZERO))
        .expect("mesh transform should update");

    let mut extract = world.to_render_frame_extract();
    extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
        probe_budget: 1,
        tracing_budget: 1,
        probes: vec![
            hybrid_probe(mesh, 10, true, Vec3::new(0.32, 0.0, 0.0), 96),
            hybrid_probe_with_parent(mesh, 20, false, Vec3::new(0.03, 0.0, 0.0), 96, 10),
            hybrid_probe_with_parent(mesh, 30, false, Vec3::new(0.22, 0.0, 0.0), 112, 20),
            hybrid_probe(mesh, 40, false, Vec3::new(0.1, 0.0, 0.0), 128),
        ],
        trace_regions: vec![hybrid_trace_region(mesh, 40, Vec3::ZERO, 8.0)],
    });

    let context = VisibilityContext::from(&extract);

    assert_eq!(
        context.hybrid_gi_update_plan,
        VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![10],
            requested_probe_ids: vec![30],
            dirty_requested_probe_ids: vec![30],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        }
    );
    assert_eq!(
        context.hybrid_gi_feedback,
        VisibilityHybridGiFeedback {
            active_probe_ids: vec![40, 10],
            requested_probe_ids: vec![30],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        }
    );
}

#[test]
fn visibility_context_spreads_hybrid_gi_probe_budget_across_active_lineages_before_second_descendant_request(
) {
    let mut world = World::new();
    remove_default_meshes(&mut world);

    let mesh = world.spawn_mesh_node(
        model_handle("res://models/hybrid_gi.obj"),
        material_handle("res://materials/hybrid_gi.material.toml"),
    );
    world
        .update_transform(mesh, Transform::from_translation(Vec3::ZERO))
        .expect("mesh transform should update");

    let mut extract = world.to_render_frame_extract();
    extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
        probe_budget: 2,
        tracing_budget: 1,
        probes: vec![
            hybrid_probe(mesh, 10, true, Vec3::new(0.32, 0.0, 0.0), 96),
            hybrid_probe_with_parent(mesh, 20, false, Vec3::new(0.08, 0.0, 0.0), 112, 10),
            hybrid_probe_with_parent(mesh, 30, false, Vec3::new(0.02, 0.0, 0.0), 128, 20),
            hybrid_probe(mesh, 40, true, Vec3::new(-0.32, 0.0, 0.0), 96),
            hybrid_probe_with_parent(mesh, 50, false, Vec3::new(-0.08, 0.0, 0.0), 104, 40),
        ],
        trace_regions: vec![hybrid_trace_region(mesh, 60, Vec3::ZERO, 8.0)],
    });

    let context = VisibilityContext::from(&extract);

    assert_eq!(
        context.hybrid_gi_update_plan,
        VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![10, 40],
            requested_probe_ids: vec![30, 50],
            dirty_requested_probe_ids: vec![30, 50],
            scheduled_trace_region_ids: vec![60],
            evictable_probe_ids: Vec::new(),
        }
    );
    assert_eq!(
        context.hybrid_gi_feedback,
        VisibilityHybridGiFeedback {
            active_probe_ids: vec![10, 40],
            requested_probe_ids: vec![30, 50],
            scheduled_trace_region_ids: vec![60],
            evictable_probe_ids: Vec::new(),
        }
    );
}

#[test]
fn visibility_context_holds_resident_hybrid_gi_child_probe_one_frame_when_frontier_merges_back_to_parent(
) {
    let mut world = World::new();
    remove_default_meshes(&mut world);

    let mesh = world.spawn_mesh_node(
        model_handle("res://models/hybrid_gi.obj"),
        material_handle("res://materials/hybrid_gi.material.toml"),
    );
    world
        .update_transform(mesh, Transform::from_translation(Vec3::ZERO))
        .expect("mesh transform should update");

    let mut previous_extract = world.to_render_frame_extract();
    previous_extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
        probe_budget: 3,
        tracing_budget: 1,
        probes: vec![
            hybrid_probe(mesh, 10, true, Vec3::ZERO, 128),
            hybrid_probe_with_parent(mesh, 20, true, Vec3::new(0.08, 0.0, 0.0), 96, 10),
            hybrid_probe_with_parent(mesh, 30, true, Vec3::new(-0.08, 0.0, 0.0), 80, 10),
        ],
        trace_regions: vec![hybrid_trace_region(mesh, 40, Vec3::ZERO, 8.0)],
    });
    let previous_context = VisibilityContext::from(&previous_extract);

    assert_eq!(
        previous_context.hybrid_gi_feedback.active_probe_ids,
        vec![20, 30],
        "expected the fully resident frame to refine onto the child probe frontier before testing merge-back hysteresis"
    );

    let mut current_extract = world.to_render_frame_extract();
    current_extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
        probe_budget: 3,
        tracing_budget: 1,
        probes: vec![
            hybrid_probe(mesh, 10, true, Vec3::ZERO, 128),
            hybrid_probe_with_parent(mesh, 20, true, Vec3::new(0.08, 0.0, 0.0), 96, 10),
            hybrid_probe_with_parent(mesh, 30, false, Vec3::new(-0.08, 0.0, 0.0), 80, 10),
        ],
        trace_regions: vec![hybrid_trace_region(mesh, 40, Vec3::ZERO, 8.0)],
    });

    let held_context = VisibilityContext::from_extract_with_history(
        &current_extract,
        Some(&previous_context.history_snapshot),
    );

    assert_eq!(
        held_context.hybrid_gi_feedback,
        VisibilityHybridGiFeedback {
            active_probe_ids: vec![10],
            requested_probe_ids: vec![30],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        }
    );
    assert_eq!(
        held_context.hybrid_gi_update_plan,
        VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![10, 20],
            requested_probe_ids: vec![30],
            dirty_requested_probe_ids: vec![30],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        }
    );

    let settled_context = VisibilityContext::from_extract_with_history(
        &current_extract,
        Some(&held_context.history_snapshot),
    );

    assert_eq!(
        settled_context.hybrid_gi_feedback,
        VisibilityHybridGiFeedback {
            active_probe_ids: vec![10],
            requested_probe_ids: vec![30],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        }
    );
    assert_eq!(
        settled_context.hybrid_gi_update_plan,
        VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![10, 20],
            requested_probe_ids: vec![30],
            dirty_requested_probe_ids: Vec::new(),
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        }
    );
}

#[test]
fn visibility_context_keeps_resident_hybrid_gi_descendant_probe_hot_while_ancestor_request_remains_pending(
) {
    let mut world = World::new();
    remove_default_meshes(&mut world);

    let mesh = world.spawn_mesh_node(
        model_handle("res://models/hybrid_gi.obj"),
        material_handle("res://materials/hybrid_gi.material.toml"),
    );
    world
        .update_transform(mesh, Transform::from_translation(Vec3::ZERO))
        .expect("mesh transform should update");

    let mut previous_extract = world.to_render_frame_extract();
    previous_extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
        probe_budget: 3,
        tracing_budget: 1,
        probes: vec![
            hybrid_probe(mesh, 10, true, Vec3::ZERO, 128),
            hybrid_probe_with_parent(mesh, 20, true, Vec3::new(0.08, 0.0, 0.0), 96, 10),
            hybrid_probe_with_parent(mesh, 30, true, Vec3::new(0.14, 0.0, 0.0), 80, 20),
        ],
        trace_regions: vec![hybrid_trace_region(mesh, 40, Vec3::ZERO, 8.0)],
    });
    let previous_context = VisibilityContext::from(&previous_extract);

    assert_eq!(
        previous_context.hybrid_gi_feedback.active_probe_ids,
        vec![30],
        "expected the fully resident frame to refine onto the deepest probe frontier before testing pending-ancestor hysteresis"
    );

    let mut current_extract = world.to_render_frame_extract();
    current_extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
        probe_budget: 3,
        tracing_budget: 1,
        probes: vec![
            hybrid_probe(mesh, 10, true, Vec3::ZERO, 128),
            hybrid_probe_with_parent(mesh, 20, false, Vec3::new(0.08, 0.0, 0.0), 96, 10),
            hybrid_probe_with_parent(mesh, 30, true, Vec3::new(0.14, 0.0, 0.0), 80, 20),
        ],
        trace_regions: vec![hybrid_trace_region(mesh, 40, Vec3::ZERO, 8.0)],
    });

    let held_context = VisibilityContext::from_extract_with_history(
        &current_extract,
        Some(&previous_context.history_snapshot),
    );
    let settled_context = VisibilityContext::from_extract_with_history(
        &current_extract,
        Some(&held_context.history_snapshot),
    );

    assert_eq!(
        settled_context.hybrid_gi_feedback,
        VisibilityHybridGiFeedback {
            active_probe_ids: vec![10],
            requested_probe_ids: vec![20],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        },
        "expected the still-resident descendant probe to stay hot while its nonresident ancestor request remains pending, instead of becoming evictable on the second collapsed frame"
    );
    assert_eq!(
        settled_context.hybrid_gi_update_plan,
        VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![10, 30],
            requested_probe_ids: vec![20],
            dirty_requested_probe_ids: Vec::new(),
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        }
    );
}

fn remove_default_meshes(world: &mut World) {
    let mesh_entities = world
        .nodes()
        .iter()
        .filter(|node| node.mesh.is_some())
        .map(|node| node.id)
        .collect::<Vec<_>>();
    for entity in mesh_entities {
        assert!(world.remove_entity(entity));
    }
}

fn model_handle(label: &str) -> ResourceHandle<ModelMarker> {
    ResourceHandle::new(ResourceId::from_stable_label(label))
}

fn material_handle(label: &str) -> ResourceHandle<MaterialMarker> {
    ResourceHandle::new(ResourceId::from_stable_label(label))
}

fn hybrid_probe(
    entity: u64,
    probe_id: u32,
    resident: bool,
    position: Vec3,
    ray_budget: u32,
) -> RenderHybridGiProbe {
    RenderHybridGiProbe {
        entity,
        probe_id,
        position,
        radius: 0.5,
        parent_probe_id: None,
        resident,
        ray_budget,
    }
}

fn hybrid_probe_with_parent(
    entity: u64,
    probe_id: u32,
    resident: bool,
    position: Vec3,
    ray_budget: u32,
    parent_probe_id: u32,
) -> RenderHybridGiProbe {
    RenderHybridGiProbe {
        parent_probe_id: Some(parent_probe_id),
        ..hybrid_probe(entity, probe_id, resident, position, ray_budget)
    }
}

fn hybrid_trace_region(
    entity: u64,
    region_id: u32,
    bounds_center: Vec3,
    screen_coverage: f32,
) -> RenderHybridGiTraceRegion {
    RenderHybridGiTraceRegion {
        entity,
        region_id,
        bounds_center,
        bounds_radius: 0.5,
        screen_coverage,
        rt_lighting_rgb: [0, 0, 0],
    }
}
