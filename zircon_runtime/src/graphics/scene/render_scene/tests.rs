use std::sync::Arc;

use crate::core::framework::render::{
    CastShadowsMode, RenderMaterialAlphaMode, RenderMaterialPropertyValue, RenderMeshBounds,
    RenderWorldSnapshotHandle,
};
use crate::core::framework::scene::Mobility;
use crate::core::math::{Mat4, Vec3, Vec4};

use super::{
    RenderScene, RenderSceneApplyError, RenderSceneDelta, RenderSceneGeneration,
    RenderScenePrimitive, RenderScenePrimitiveDirtyFlags, RenderScenePrimitiveField,
    RenderScenePrimitiveLocalBounds,
};

mod deformation;
mod fixtures;
mod mesh_source;
mod resource_dependencies;

use fixtures::*;

#[test]
fn render_scene_delta_assigns_deterministic_handles_and_journal_order() {
    let mut scene = test_scene();
    let second = test_primitive(2);
    let first = test_primitive(1);

    let journal = scene
        .apply_delta(RenderSceneDelta::new(vec![second, first], Vec::new()))
        .expect("valid scene delta");

    assert_eq!(journal.from_generation(), RenderSceneGeneration::INITIAL);
    assert_eq!(journal.to_generation(), RenderSceneGeneration::new(1));
    assert_eq!(journal.additions().len(), 2);
    assert_eq!(
        journal.additions()[0].primitive().stable_instance_key(),
        stable_key(1)
    );
    assert_eq!(
        journal.additions()[1].primitive().stable_instance_key(),
        stable_key(2)
    );
    assert_eq!(journal.additions()[0].handle().slot(), 0);
    assert_eq!(journal.additions()[1].handle().slot(), 1);
    assert_eq!(journal.additions()[0].dense_index(), 0);
    assert_eq!(journal.additions()[1].dense_index(), 1);
    assert_eq!(scene.read().len(), 2);
    assert_eq!(scene.read().generation().get(), 1);
    assert_eq!(scene.read().iter().count(), 2);
    assert_eq!(
        scene.read().handle_for_stable_key(stable_key(1)),
        Some(journal.additions()[0].handle())
    );
    assert_eq!(journal.stats().input_upsert_count(), 2);
    assert_eq!(journal.stats().stable_key_lookup_count(), 2);
    assert_eq!(journal.stats().primitive_comparison_count(), 0);
    assert_eq!(journal.stats().appended_handle_slot_count(), 2);
}

#[test]
fn render_scene_read_view_and_journal_preserve_world_identity() {
    let world = RenderWorldSnapshotHandle::new(73);
    let mut scene = RenderScene::new(world);

    let journal = scene
        .apply_delta(RenderSceneDelta::new(vec![test_primitive(73)], Vec::new()))
        .expect("world-bound scene delta");

    assert_eq!(scene.read().world(), world);
    assert_eq!(journal.world(), world);
}

#[test]
fn render_scene_no_op_upsert_keeps_generation_and_empty_journal() {
    let mut scene = test_scene();
    let primitive = test_primitive(7);
    scene
        .apply_delta(RenderSceneDelta::new(vec![primitive.clone()], Vec::new()))
        .expect("initial add");

    let journal = scene
        .apply_delta(RenderSceneDelta::new(vec![primitive], Vec::new()))
        .expect("idempotent upsert");

    assert!(journal.is_empty());
    assert_eq!(journal.from_generation(), RenderSceneGeneration::new(1));
    assert_eq!(journal.to_generation(), RenderSceneGeneration::new(1));
    assert_eq!(scene.read().generation(), RenderSceneGeneration::new(1));
    assert_eq!(journal.stats().input_upsert_count(), 1);
    assert_eq!(journal.stats().input_removal_count(), 0);
    assert_eq!(journal.stats().primitive_comparison_count(), 1);
    assert_eq!(journal.stats().dirty_domain_counts().total_count(), 0);
    assert!(journal.resource_reference_deltas().is_empty());
    assert_eq!(
        journal.resource_reference_stats(),
        super::RenderSceneResourceReferenceDeltaStats::default()
    );
}

#[test]
fn render_scene_update_reports_precise_dirty_domains() {
    let mut scene = test_scene();
    let initial = scene
        .apply_delta(RenderSceneDelta::new(vec![test_primitive(11)], Vec::new()))
        .expect("initial add");
    let previous = Arc::clone(initial.additions()[0].primitive());
    let transformed =
        test_primitive_with_revisions(11, test_revisions(2, 1, 1, 1, 1), |descriptor| {
            descriptor.world_from_local = Mat4::from_translation(Vec3::new(3.0, 4.0, 5.0));
        });

    let journal = scene
        .apply_delta(RenderSceneDelta::new(vec![transformed], Vec::new()))
        .expect("transform update");

    let dirty = journal.updates()[0].dirty();
    assert!(Arc::ptr_eq(
        journal.updates()[0].previous_primitive(),
        &previous
    ));
    assert!(!Arc::ptr_eq(
        journal.updates()[0].previous_primitive(),
        journal.updates()[0].primitive()
    ));
    assert_eq!(journal.updates()[0].handle().slot(), 0);
    assert_eq!(journal.updates()[0].dense_index(), 0);
    assert_eq!(journal.updates()[0].primitive().revisions().transform, 2);
    assert_eq!(
        journal.updates()[0].primitive().world_bounds().center,
        [3.0, 4.0, 5.0]
    );
    assert!(dirty.contains(RenderScenePrimitiveDirtyFlags::TRANSFORM));
    assert!(dirty.contains(RenderScenePrimitiveDirtyFlags::BOUNDS));
    assert!(!dirty.contains(RenderScenePrimitiveDirtyFlags::LOCAL_BOUNDS));
    assert!(!dirty.contains(RenderScenePrimitiveDirtyFlags::GEOMETRY));
    assert!(!dirty.contains(RenderScenePrimitiveDirtyFlags::MATERIAL));
    assert!(!dirty.contains(RenderScenePrimitiveDirtyFlags::DEFORMATION));
    assert!(journal.resource_reference_deltas().is_empty());
    assert_eq!(
        journal
            .resource_reference_stats()
            .projected_primitive_payload_count(),
        0
    );
}

#[test]
fn render_scene_bounds_change_does_not_invalidate_geometry() {
    let mut scene = test_scene();
    scene
        .apply_delta(RenderSceneDelta::new(vec![test_primitive(12)], Vec::new()))
        .expect("initial add");
    let changed = RenderScenePrimitive::new(
        test_descriptor(12, stable_key(12)),
        RenderScenePrimitiveLocalBounds::base_only(RenderMeshBounds::from_min_max(
            [-2.0, -2.0, -3.0],
            [2.0, 2.0, 3.0],
        )),
        test_revisions(1, 1, 1, 2, 1),
    )
    .expect("changed bounds");

    let journal = scene
        .apply_delta(RenderSceneDelta::new(vec![changed], Vec::new()))
        .expect("bounds update");

    assert_eq!(
        journal.updates()[0].dirty(),
        RenderScenePrimitiveDirtyFlags::BOUNDS | RenderScenePrimitiveDirtyFlags::LOCAL_BOUNDS
    );
}

#[test]
fn render_scene_primitive_publishes_shear_conservative_world_bounds_once() {
    let mut descriptor = test_descriptor(14, stable_key(14));
    descriptor.world_from_local = Mat4::from_cols(
        Vec4::new(2.0, 0.0, 0.0, 0.0),
        Vec4::new(1.0, 1.0, 0.0, 0.0),
        Vec4::new(0.0, 0.0, 3.0, 0.0),
        Vec4::new(10.0, 20.0, 30.0, 1.0),
    );
    let primitive = RenderScenePrimitive::new(
        descriptor,
        RenderScenePrimitiveLocalBounds::base_only(RenderMeshBounds::from_min_max(
            [-1.0, -2.0, -0.5],
            [3.0, 2.0, 0.5],
        )),
        test_revisions(1, 1, 1, 1, 1),
    )
    .expect("valid sheared primitive");

    assert_eq!(primitive.world_bounds().center, [12.0, 20.0, 30.0]);
    assert_eq!(primitive.world_bounds().min, [6.0, 18.0, 28.5]);
    assert_eq!(primitive.world_bounds().max, [18.0, 22.0, 31.5]);
    assert_eq!(primitive.world_bounds().radius, 6.5);
}

#[test]
fn render_scene_dynamic_primitive_uses_explicit_material_revision() {
    let mut scene = test_scene();
    let dynamic = test_primitive_with(12, |descriptor| {
        descriptor.mobility = Mobility::Dynamic;
        descriptor.transform_static = false;
        descriptor.common.is_static = false;
    });
    scene
        .apply_delta(RenderSceneDelta::new(vec![dynamic], Vec::new()))
        .expect("initial dynamic add");
    let changed = test_primitive_with_revisions(12, test_revisions(1, 1, 2, 1, 1), |descriptor| {
        descriptor.mobility = Mobility::Dynamic;
        descriptor.transform_static = false;
        descriptor.common.is_static = false;
    });

    let journal = scene
        .apply_delta(RenderSceneDelta::new(vec![changed], Vec::new()))
        .expect("dynamic material revision");
    let dirty = journal.updates()[0].dirty();

    assert_eq!(dirty, RenderScenePrimitiveDirtyFlags::MATERIAL);
}

#[test]
fn render_scene_mobility_change_invalidates_render_state_and_view_relevance() {
    let mut scene = test_scene();
    scene
        .apply_delta(RenderSceneDelta::new(vec![test_primitive(15)], Vec::new()))
        .expect("initial add");
    let changed = test_primitive_with(15, |descriptor| {
        descriptor.mobility = Mobility::Dynamic;
    });

    let journal = scene
        .apply_delta(RenderSceneDelta::new(vec![changed], Vec::new()))
        .expect("mobility update");

    assert_eq!(
        journal.updates()[0].dirty(),
        RenderScenePrimitiveDirtyFlags::RENDER_STATE | RenderScenePrimitiveDirtyFlags::VISIBILITY
    );
}

#[test]
fn render_scene_cast_shadow_change_invalidates_shadow_relevance() {
    let mut scene = test_scene();
    scene
        .apply_delta(RenderSceneDelta::new(vec![test_primitive(16)], Vec::new()))
        .expect("initial add");
    let changed = test_primitive_with(16, |descriptor| {
        descriptor.common.cast_shadows = CastShadowsMode::Off;
    });

    let journal = scene
        .apply_delta(RenderSceneDelta::new(vec![changed], Vec::new()))
        .expect("cast-shadow update");

    assert_eq!(
        journal.updates()[0].dirty(),
        RenderScenePrimitiveDirtyFlags::RENDER_STATE | RenderScenePrimitiveDirtyFlags::VISIBILITY
    );
}

#[test]
fn render_scene_alpha_cutoff_stays_material_only_until_phase_changes() {
    let mut scene = test_scene();
    let masked = test_primitive_with(17, |descriptor| {
        descriptor.material_alpha_mode = RenderMaterialAlphaMode::Mask { cutoff: 0.25 };
    });
    scene
        .apply_delta(RenderSceneDelta::new(vec![masked], Vec::new()))
        .expect("initial masked add");
    let changed_cutoff = test_primitive_with(17, |descriptor| {
        descriptor.material_alpha_mode = RenderMaterialAlphaMode::Mask { cutoff: 0.5 };
    });

    let cutoff_journal = scene
        .apply_delta(RenderSceneDelta::new(vec![changed_cutoff], Vec::new()))
        .expect("cutoff update");

    assert_eq!(
        cutoff_journal.updates()[0].dirty(),
        RenderScenePrimitiveDirtyFlags::MATERIAL
    );

    let transparent = test_primitive_with(17, |descriptor| {
        descriptor.material_alpha_mode = RenderMaterialAlphaMode::Blend;
    });
    let phase_journal = scene
        .apply_delta(RenderSceneDelta::new(vec![transparent], Vec::new()))
        .expect("alpha phase update");

    assert_eq!(
        phase_journal.updates()[0].dirty(),
        RenderScenePrimitiveDirtyFlags::MATERIAL
            | RenderScenePrimitiveDirtyFlags::RENDER_STATE
            | RenderScenePrimitiveDirtyFlags::VISIBILITY
    );
}

#[test]
fn render_scene_remove_invalidates_stale_handle_and_reuses_slot_generation() {
    let mut scene = test_scene();
    let added = scene
        .apply_delta(RenderSceneDelta::new(vec![test_primitive(21)], Vec::new()))
        .expect("initial add");
    let stale = added.additions()[0].handle();

    let removed = scene
        .apply_delta(RenderSceneDelta::new(Vec::new(), vec![stable_key(21)]))
        .expect("remove");
    assert_eq!(removed.removals()[0].handle(), stale);
    assert_eq!(
        removed.removals()[0].primitive().stable_instance_key(),
        stable_key(21)
    );
    assert!(scene.read().get(stale).is_none());

    let replacement = scene
        .apply_delta(RenderSceneDelta::new(vec![test_primitive(22)], Vec::new()))
        .expect("replacement add");
    let current = replacement.additions()[0].handle();

    assert_eq!(current.slot(), stale.slot());
    assert_ne!(current.slot_generation(), stale.slot_generation());
    assert!(scene.read().get(stale).is_none());
    assert_eq!(
        scene
            .read()
            .get(current)
            .expect("current handle")
            .stable_instance_key(),
        stable_key(22)
    );
    assert_eq!(replacement.stats().reused_handle_slot_count(), 1);
    assert_eq!(replacement.stats().appended_handle_slot_count(), 0);
}

#[test]
fn render_scene_storage_stats_track_high_water_holes_and_reuse_in_constant_time() {
    let mut scene = test_scene();
    assert_eq!(scene.read().storage_stats().live_primitive_count(), 0);
    assert_eq!(scene.read().storage_stats().handle_slot_high_water(), 0);

    let added = scene
        .apply_delta(RenderSceneDelta::new(
            vec![test_primitive(23), test_primitive(24), test_primitive(25)],
            Vec::new(),
        ))
        .expect("three initial primitives");
    let full = scene.read().storage_stats();
    assert_eq!(full.live_primitive_count(), 3);
    assert_eq!(full.handle_slot_high_water(), 3);
    assert_eq!(full.reusable_handle_hole_count(), 0);
    assert_eq!(full.generation_exhausted_handle_slot_count(), 0);
    assert_eq!(full.fragmented_handle_slot_count(), 0);
    assert_eq!(added.stats().storage_stats(), full);

    let removed = scene
        .apply_delta(RenderSceneDelta::new(Vec::new(), vec![stable_key(24)]))
        .expect("remove middle primitive");
    let fragmented = scene.read().storage_stats();
    assert_eq!(fragmented.live_primitive_count(), 2);
    assert_eq!(fragmented.handle_slot_high_water(), 3);
    assert_eq!(fragmented.reusable_handle_hole_count(), 1);
    assert_eq!(fragmented.fragmented_handle_slot_count(), 1);
    assert_eq!(removed.stats().storage_stats(), fragmented);

    let replacement = scene
        .apply_delta(RenderSceneDelta::new(vec![test_primitive(26)], Vec::new()))
        .expect("reuse handle hole");
    let reused = scene.read().storage_stats();
    assert_eq!(reused.live_primitive_count(), 3);
    assert_eq!(reused.handle_slot_high_water(), 3);
    assert_eq!(reused.reusable_handle_hole_count(), 0);
    assert_eq!(replacement.stats().storage_stats(), reused);
}

#[test]
fn render_scene_removal_reports_dense_swap_relocation() {
    let mut scene = test_scene();
    let added = scene
        .apply_delta(RenderSceneDelta::new(
            vec![test_primitive(31), test_primitive(32), test_primitive(33)],
            Vec::new(),
        ))
        .expect("initial adds");
    let moved_handle = added.additions()[2].handle();

    let journal = scene
        .apply_delta(RenderSceneDelta::new(Vec::new(), vec![stable_key(31)]))
        .expect("remove first dense item");

    let removal = &journal.removals()[0];
    assert_eq!(removal.dense_index(), 0);
    let relocation = removal.relocation().expect("last dense item must move");
    assert_eq!(relocation.handle(), moved_handle);
    assert_eq!(relocation.from_dense_index(), 2);
    assert_eq!(relocation.to_dense_index(), 0);
    assert_eq!(scene.read().dense_index(moved_handle), Some(0));
    assert_eq!(journal.stats().dense_relocation_count(), 1);
}

#[test]
fn render_scene_rejects_duplicate_delta_without_mutation() {
    let mut scene = test_scene();
    let primitive = test_primitive(41);

    let error = scene
        .apply_delta(RenderSceneDelta::new(
            vec![primitive.clone(), primitive],
            Vec::new(),
        ))
        .expect_err("duplicate stable key must fail");

    assert_eq!(
        error,
        RenderSceneApplyError::DuplicateUpsert {
            stable_instance_key: stable_key(41),
        }
    );
    assert!(scene.read().is_empty());
    assert_eq!(scene.read().generation(), RenderSceneGeneration::INITIAL);
}

#[test]
fn render_scene_rejects_remove_upsert_conflict_without_mutation() {
    let mut scene = test_scene();
    let key = stable_key(51);

    let error = scene
        .apply_delta(RenderSceneDelta::new(vec![test_primitive(51)], vec![key]))
        .expect_err("one key cannot be removed and upserted together");

    assert_eq!(
        error,
        RenderSceneApplyError::ConflictingMutation {
            stable_instance_key: key,
        }
    );
    assert!(scene.read().is_empty());
}

#[test]
fn render_scene_rejects_stable_key_entity_reassignment() {
    let mut scene = test_scene();
    scene
        .apply_delta(RenderSceneDelta::new(vec![test_primitive(61)], Vec::new()))
        .expect("initial add");
    let reassigned = test_primitive_with_key(62, stable_key(61));

    let error = scene
        .apply_delta(RenderSceneDelta::new(vec![reassigned], Vec::new()))
        .expect_err("stable identity cannot change entity owner");

    assert_eq!(
        error,
        RenderSceneApplyError::StableKeyOwnerChanged {
            stable_instance_key: stable_key(61),
            previous_node_id: 61,
            incoming_node_id: 62,
        }
    );
    assert_eq!(scene.read().generation(), RenderSceneGeneration::new(1));
}

#[test]
fn render_scene_journal_keeps_immutable_payload_after_later_update() {
    let mut scene = test_scene();
    let added = scene
        .apply_delta(RenderSceneDelta::new(vec![test_primitive(71)], Vec::new()))
        .expect("initial add");
    let original = added.additions()[0].primitive().clone();
    let changed = test_primitive_with(71, |descriptor| {
        descriptor.tint = Vec4::new(0.2, 0.3, 0.4, 1.0);
    });

    scene
        .apply_delta(RenderSceneDelta::new(vec![changed], Vec::new()))
        .expect("material update");

    assert_eq!(original.descriptor().tint, Vec4::ONE);
    assert_eq!(
        added.additions()[0].primitive().descriptor().tint,
        Vec4::ONE
    );
}

#[test]
fn render_scene_update_combines_material_deformation_and_visibility_domains() {
    let mut scene = test_scene();
    scene
        .apply_delta(RenderSceneDelta::new(vec![test_primitive(81)], Vec::new()))
        .expect("initial add");
    let mut descriptor = test_descriptor(81, stable_key(81));
    descriptor.tint = Vec4::new(0.5, 0.6, 0.7, 1.0);
    descriptor.morph_weights = Arc::from([0.25]);
    descriptor.common.enabled = false;
    let changed = RenderScenePrimitive::new(
        descriptor,
        RenderScenePrimitiveLocalBounds::base_only(RenderMeshBounds::from_min_max(
            [-1.0, -2.0, -3.0],
            [1.0, 2.0, 3.0],
        )),
        test_revisions(1, 1, 1, 1, 2),
    )
    .expect("finite changed primitive");

    let journal = scene
        .apply_delta(RenderSceneDelta::new(vec![changed], Vec::new()))
        .expect("combined update");
    let dirty = journal.updates()[0].dirty();

    assert!(dirty.contains(RenderScenePrimitiveDirtyFlags::MATERIAL));
    assert!(dirty.contains(RenderScenePrimitiveDirtyFlags::DEFORMATION));
    assert!(dirty.contains(RenderScenePrimitiveDirtyFlags::VISIBILITY));
    assert!(dirty.contains(RenderScenePrimitiveDirtyFlags::BOUNDS));
    assert!(dirty.contains(RenderScenePrimitiveDirtyFlags::LOCAL_BOUNDS));
    assert!(!dirty.contains(RenderScenePrimitiveDirtyFlags::TRANSFORM));
    assert!(!dirty.contains(RenderScenePrimitiveDirtyFlags::GEOMETRY));
    let counts = journal.stats().dirty_domain_counts();
    assert_eq!(counts.transform_count(), 0);
    assert_eq!(counts.geometry_count(), 0);
    assert_eq!(counts.material_count(), 1);
    assert_eq!(counts.deformation_count(), 1);
    assert_eq!(counts.render_state_count(), 0);
    assert_eq!(counts.visibility_count(), 1);
    assert_eq!(counts.bounds_count(), 1);
    assert_eq!(counts.total_count(), 4);
}

#[test]
fn render_scene_primitive_rejects_non_finite_or_non_affine_world_matrix() {
    let mut non_finite = Mat4::IDENTITY.to_cols_array();
    non_finite[4] = f32::NAN;
    let mut non_affine = Mat4::IDENTITY.to_cols_array();
    non_affine[3] = 0.25;

    for world_from_local in [non_finite, non_affine] {
        let mut descriptor = test_descriptor(90, stable_key(90));
        descriptor.world_from_local = Mat4::from_cols_array(&world_from_local);
        let error = RenderScenePrimitive::new(
            descriptor,
            RenderScenePrimitiveLocalBounds::base_only(RenderMeshBounds::from_min_max(
                [-1.0; 3], [1.0; 3],
            )),
            test_revisions(1, 1, 1, 1, 1),
        )
        .expect_err("invalid world matrix must fail");

        assert_eq!(error.stable_instance_key(), stable_key(90));
        assert_eq!(error.field(), RenderScenePrimitiveField::WorldFromLocal);
    }
}

#[test]
fn render_scene_primitive_rejects_world_bounds_projection_overflow() {
    let mut descriptor = test_descriptor(95, stable_key(95));
    descriptor.world_from_local = Mat4::from_scale(Vec3::new(f32::MAX, 1.0, 1.0));

    let error = RenderScenePrimitive::new(
        descriptor,
        RenderScenePrimitiveLocalBounds::base_only(RenderMeshBounds::from_min_max(
            [-2.0; 3], [2.0; 3],
        )),
        test_revisions(1, 1, 1, 1, 1),
    )
    .expect_err("overflowed world bounds must fail");

    assert_eq!(error.stable_instance_key(), stable_key(95));
    assert_eq!(error.field(), RenderScenePrimitiveField::WorldBoundsMin);
}

#[test]
fn render_scene_primitive_rejects_non_finite_bounds_before_scene_mutation() {
    let error = RenderScenePrimitive::new(
        test_descriptor(91, stable_key(91)),
        RenderScenePrimitiveLocalBounds::base_only(RenderMeshBounds::from_min_max(
            [f32::NAN, -1.0, -1.0],
            [1.0; 3],
        )),
        test_revisions(1, 1, 1, 1, 1),
    )
    .expect_err("non-finite local bounds must fail");

    assert_eq!(error.stable_instance_key(), stable_key(91));
    assert_eq!(error.field(), RenderScenePrimitiveField::LocalBoundsMin);
}

#[test]
fn render_scene_primitive_rejects_alpha_mask_cutoff_outside_unit_interval() {
    for cutoff in [-0.01, 1.01] {
        let mut descriptor = test_descriptor(92, stable_key(92));
        descriptor.material_alpha_mode = RenderMaterialAlphaMode::Mask { cutoff };

        let error = RenderScenePrimitive::new(
            descriptor,
            RenderScenePrimitiveLocalBounds::base_only(RenderMeshBounds::from_min_max(
                [-1.0; 3], [1.0; 3],
            )),
            test_revisions(1, 1, 1, 1, 1),
        )
        .expect_err("alpha mask cutoff outside [0, 1] must fail");

        assert_eq!(error.stable_instance_key(), stable_key(92));
        assert_eq!(
            error.field(),
            RenderScenePrimitiveField::MaterialAlphaCutoff
        );
    }
}

#[test]
fn render_scene_primitive_rejects_non_finite_material_property_override() {
    for value in [
        RenderMaterialPropertyValue::Float { value: f32::NAN },
        RenderMaterialPropertyValue::Vec4 {
            value: [0.0, 1.0, f32::INFINITY, 1.0],
        },
    ] {
        let mut descriptor = test_descriptor(93, stable_key(93));
        descriptor
            .material_property_overrides
            .insert("invalid", value);

        let error = RenderScenePrimitive::new(
            descriptor,
            RenderScenePrimitiveLocalBounds::base_only(RenderMeshBounds::from_min_max(
                [-1.0; 3], [1.0; 3],
            )),
            test_revisions(1, 1, 1, 1, 1),
        )
        .expect_err("non-finite property override must fail");

        assert_eq!(error.stable_instance_key(), stable_key(93));
        assert_eq!(
            error.field(),
            RenderScenePrimitiveField::MaterialPropertyOverride
        );
    }
}

#[test]
fn render_scene_primitive_canonicalizes_bounds_metadata_once() {
    let primitive = RenderScenePrimitive::new(
        test_descriptor(92, stable_key(92)),
        RenderScenePrimitiveLocalBounds::base_only(RenderMeshBounds {
            min: [-1.0; 3],
            max: [1.0; 3],
            center: [100.0; 3],
            radius: 0.0,
        }),
        test_revisions(1, 2, 3, 7, 9),
    )
    .expect("finite local bounds");

    assert_eq!(primitive.local_bounds().center, [0.0; 3]);
    assert!((primitive.local_bounds().radius - 3.0_f32.sqrt()).abs() <= 1.0e-6);
    assert_eq!(primitive.revisions().bounds, 7);
    assert_eq!(primitive.revisions().deformation, 9);
}

#[test]
fn render_scene_large_stable_generation_publishes_only_one_changed_entry() {
    const PRIMITIVE_COUNT: u64 = 10_000;
    let mut scene = test_scene();
    let initial = (1..=PRIMITIVE_COUNT)
        .map(test_primitive)
        .collect::<Vec<_>>();
    scene
        .apply_delta(RenderSceneDelta::new(initial, Vec::new()))
        .expect("large initial generation");
    let changed = test_primitive_with_revisions(
        PRIMITIVE_COUNT / 2,
        test_revisions(2, 1, 1, 1, 1),
        |descriptor| {
            descriptor.world_from_local = Mat4::from_translation(Vec3::X);
        },
    );

    let journal = scene
        .apply_delta(RenderSceneDelta::new(vec![changed], Vec::new()))
        .expect("single-entry delta");

    assert_eq!(scene.read().len(), PRIMITIVE_COUNT as usize);
    assert_eq!(journal.updates().len(), 1);
    assert!(journal.additions().is_empty());
    assert!(journal.removals().is_empty());
    assert_eq!(journal.stats().stable_key_lookup_count(), 1);
    assert_eq!(journal.stats().primitive_comparison_count(), 1);
    assert_eq!(journal.stats().reused_handle_slot_count(), 0);
    assert_eq!(
        journal.resource_reference_stats(),
        super::RenderSceneResourceReferenceDeltaStats::default()
    );
}
