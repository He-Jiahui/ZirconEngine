use super::*;
use crate::graphics::scene::render_scene::{RenderSceneMeshLod, RenderSceneMeshSource};

#[test]
fn render_scene_primitive_descriptor_excludes_view_selected_mesh_lod() {
    let source = include_str!("../primitive.rs");
    let mesh_source = include_str!("../mesh_source.rs");

    assert!(!source.contains("RenderMeshSnapshot"));
    assert!(!source.contains("mesh_lod"));
    assert!(!mesh_source.contains("RenderMeshSnapshot"));
    assert!(!mesh_source.contains("mesh_lod"));
}

#[test]
fn render_scene_mesh_source_keeps_lods_camera_neutral_and_selects_by_view_distance() {
    let mut descriptor = test_descriptor(3, stable_key(3));
    descriptor.mesh_source = RenderSceneMeshSource::new(
        test_mesh_source_level("base"),
        vec![
            RenderSceneMeshLod::new(30.0, test_mesh_source_level("far")),
            RenderSceneMeshLod::new(10.0, test_mesh_source_level("near")),
        ],
    );
    let primitive = RenderScenePrimitive::new(
        descriptor,
        RenderScenePrimitiveLocalBounds::new(
            RenderMeshBounds::from_min_max([-1.0; 3], [1.0; 3]),
            vec![
                RenderMeshBounds::from_min_max([-2.0; 3], [2.0; 3]),
                RenderMeshBounds::from_min_max([-3.0; 3], [3.0; 3]),
            ],
        ),
        test_revisions(1, 1, 1, 1, 1),
    )
    .expect("valid unordered LOD source");
    let mut scene = test_scene();
    scene
        .apply_delta(RenderSceneDelta::new(vec![primitive], Vec::new()))
        .expect("persistent primitive add");
    let generation = scene.read().generation();
    let handle = scene
        .read()
        .handle_for_stable_key(stable_key(3))
        .expect("persistent primitive handle");
    let read = scene.read();
    let source = &read
        .get(handle)
        .expect("persistent primitive")
        .descriptor()
        .mesh_source;

    assert_eq!(source.lods()[0].min_distance, 10.0);
    assert_eq!(source.lods()[1].min_distance, 30.0);
    let primitive = read.get(handle).expect("persistent primitive");
    assert_eq!(primitive.local_bounds_source().lods()[0].min, [-3.0; 3]);
    assert_eq!(primitive.local_bounds_source().lods()[1].min, [-2.0; 3]);
    assert_eq!(source.select_for_distance(5.0).lod_index(), None);
    assert_eq!(source.select_for_distance(10.0).lod_index(), Some(0));
    assert_eq!(source.select_for_distance(29.0).min_distance(), Some(10.0));
    assert_eq!(source.select_for_distance(30.0).lod_index(), Some(1));
    assert_eq!(source.select_for_distance(f32::NAN).source(), source.base());
    assert_eq!(scene.read().generation(), generation);
}

#[test]
fn render_scene_primitive_rejects_ambiguous_lod_thresholds() {
    let mut descriptor = test_descriptor(4, stable_key(4));
    descriptor.mesh_source = RenderSceneMeshSource::new(
        test_mesh_source_level("base"),
        vec![
            RenderSceneMeshLod::new(10.0, test_mesh_source_level("first")),
            RenderSceneMeshLod::new(10.0, test_mesh_source_level("second")),
        ],
    );

    let error = RenderScenePrimitive::new(
        descriptor,
        RenderScenePrimitiveLocalBounds::new(
            RenderMeshBounds::from_min_max([-1.0; 3], [1.0; 3]),
            vec![
                RenderMeshBounds::from_min_max([-1.0; 3], [1.0; 3]),
                RenderMeshBounds::from_min_max([-1.0; 3], [1.0; 3]),
            ],
        ),
        test_revisions(1, 1, 1, 1, 1),
    )
    .expect_err("duplicate LOD threshold must be rejected");

    assert_eq!(error.stable_instance_key(), stable_key(4));
    assert_eq!(
        error.field(),
        RenderScenePrimitiveField::LodMinDistanceOrder
    );
}

#[test]
fn render_scene_primitive_unions_base_and_all_lod_local_bounds() {
    let mut descriptor = test_descriptor(5, stable_key(5));
    descriptor.mesh_source = RenderSceneMeshSource::new(
        test_mesh_source_level("base"),
        vec![
            RenderSceneMeshLod::new(10.0, test_mesh_source_level("near")),
            RenderSceneMeshLod::new(30.0, test_mesh_source_level("far")),
        ],
    );

    let primitive = RenderScenePrimitive::new(
        descriptor,
        RenderScenePrimitiveLocalBounds::new(
            RenderMeshBounds::from_min_max([-1.0, -2.0, -3.0], [1.0, 2.0, 3.0]),
            vec![
                RenderMeshBounds::from_min_max([-4.0, -1.0, -2.0], [2.0, 5.0, 1.0]),
                RenderMeshBounds::from_min_max([-2.0, -6.0, -1.0], [7.0, 3.0, 8.0]),
            ],
        ),
        test_revisions(1, 1, 1, 1, 1),
    )
    .expect("base and all LOD bounds");

    assert_eq!(primitive.local_bounds().min, [-4.0, -6.0, -3.0]);
    assert_eq!(primitive.local_bounds().max, [7.0, 5.0, 8.0]);
}

#[test]
fn render_scene_primitive_rejects_lod_bounds_count_mismatch() {
    let mut descriptor = test_descriptor(6, stable_key(6));
    descriptor.mesh_source = RenderSceneMeshSource::new(
        test_mesh_source_level("base"),
        vec![RenderSceneMeshLod::new(10.0, test_mesh_source_level("lod"))],
    );

    let error = RenderScenePrimitive::new(
        descriptor,
        RenderScenePrimitiveLocalBounds::base_only(RenderMeshBounds::from_min_max(
            [-1.0; 3], [1.0; 3],
        )),
        test_revisions(1, 1, 1, 1, 1),
    )
    .expect_err("every LOD source requires conservative local bounds");

    assert_eq!(error.stable_instance_key(), stable_key(6));
    assert_eq!(
        error.field(),
        RenderScenePrimitiveField::LodLocalBoundsCount
    );
}

#[test]
fn render_scene_lod_bounds_change_invalidates_bounds_when_union_is_stable() {
    let mut descriptor = test_descriptor(7, stable_key(7));
    descriptor.mesh_source = RenderSceneMeshSource::new(
        test_mesh_source_level("base"),
        vec![RenderSceneMeshLod::new(10.0, test_mesh_source_level("lod"))],
    );
    let initial = RenderScenePrimitive::new(
        descriptor.clone(),
        RenderScenePrimitiveLocalBounds::new(
            RenderMeshBounds::from_min_max([-3.0; 3], [3.0; 3]),
            vec![RenderMeshBounds::from_min_max([-1.0; 3], [1.0; 3])],
        ),
        test_revisions(1, 1, 1, 1, 1),
    )
    .expect("initial LOD bounds");
    let changed = RenderScenePrimitive::new(
        descriptor,
        RenderScenePrimitiveLocalBounds::new(
            RenderMeshBounds::from_min_max([-3.0; 3], [3.0; 3]),
            vec![RenderMeshBounds::from_min_max([-2.0; 3], [2.0; 3])],
        ),
        test_revisions(1, 1, 1, 1, 1),
    )
    .expect("changed LOD bounds with stable union");
    let mut scene = test_scene();
    scene
        .apply_delta(RenderSceneDelta::new(vec![initial], Vec::new()))
        .expect("initial primitive");

    let journal = scene
        .apply_delta(RenderSceneDelta::new(vec![changed], Vec::new()))
        .expect("LOD bounds update");

    assert_eq!(
        journal.updates()[0].dirty(),
        RenderScenePrimitiveDirtyFlags::BOUNDS | RenderScenePrimitiveDirtyFlags::LOCAL_BOUNDS
    );
}

#[test]
fn render_scene_mesh_source_change_invalidates_geometry_and_view_projection() {
    let mut scene = test_scene();
    scene
        .apply_delta(RenderSceneDelta::new(vec![test_primitive(13)], Vec::new()))
        .expect("initial add");
    let changed = test_primitive_with(13, |descriptor| {
        descriptor.mesh_source = RenderSceneMeshSource::new(
            test_mesh_source_level_with_labels("base", "changed", "base"),
            Vec::<RenderSceneMeshLod>::new(),
        );
    });

    let journal = scene
        .apply_delta(RenderSceneDelta::new(vec![changed], Vec::new()))
        .expect("mesh source update");

    assert_eq!(
        journal.updates()[0].dirty(),
        RenderScenePrimitiveDirtyFlags::GEOMETRY | RenderScenePrimitiveDirtyFlags::VISIBILITY
    );
}

#[test]
fn render_scene_mesh_source_material_change_stays_in_material_domain() {
    let mut scene = test_scene();
    scene
        .apply_delta(RenderSceneDelta::new(vec![test_primitive(14)], Vec::new()))
        .expect("initial add");
    let changed = test_primitive_with(14, |descriptor| {
        descriptor.mesh_source = RenderSceneMeshSource::new(
            test_mesh_source_level_with_labels("base", "base", "changed"),
            Vec::<RenderSceneMeshLod>::new(),
        );
    });

    let journal = scene
        .apply_delta(RenderSceneDelta::new(vec![changed], Vec::new()))
        .expect("material source update");

    assert_eq!(
        journal.updates()[0].dirty(),
        RenderScenePrimitiveDirtyFlags::MATERIAL
    );
}

#[test]
fn render_scene_lod_threshold_change_only_invalidates_view_selection() {
    let mut scene = test_scene();
    let initial = test_primitive_with(18, |descriptor| {
        descriptor.mesh_source = RenderSceneMeshSource::new(
            test_mesh_source_level("base"),
            vec![RenderSceneMeshLod::new(10.0, test_mesh_source_level("lod"))],
        );
    });
    scene
        .apply_delta(RenderSceneDelta::new(vec![initial], Vec::new()))
        .expect("initial LOD policy");
    let changed = test_primitive_with(18, |descriptor| {
        descriptor.mesh_source = RenderSceneMeshSource::new(
            test_mesh_source_level("base"),
            vec![RenderSceneMeshLod::new(20.0, test_mesh_source_level("lod"))],
        );
    });

    let journal = scene
        .apply_delta(RenderSceneDelta::new(vec![changed], Vec::new()))
        .expect("LOD policy update");

    assert_eq!(
        journal.updates()[0].dirty(),
        RenderScenePrimitiveDirtyFlags::VISIBILITY
    );
    let counts = journal.stats().dirty_domain_counts();
    assert_eq!(counts.geometry_count(), 0);
    assert_eq!(counts.visibility_count(), 1);
}
