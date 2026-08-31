use std::collections::HashSet;
use std::sync::Arc;

use crate::core::framework::animation::{AnimationPoseOutput, AnimationPoseSource};
use crate::core::framework::render::{MaterialOverrideSet, RenderMeshBounds};
use crate::core::resource::{
    MaterialMarker, MeshMarker, ModelMarker, ResourceHandle, ResourceId, ResourceKind,
};

use super::super::{
    RenderSceneDelta, RenderSceneMeshBinding, RenderSceneMeshLod, RenderSceneMeshSource,
    RenderSceneMeshSourceLevel, RenderScenePrimitive, RenderScenePrimitiveLocalBounds,
    RenderSceneSkeletalPose,
};
use super::fixtures::{
    stable_key, test_descriptor, test_mesh_source_level_with_labels, test_primitive,
    test_primitive_with_revisions, test_revisions, test_scene,
};

#[test]
fn render_scene_resource_reference_delta_add_and_remove_are_inverse() {
    let mut scene = test_scene();
    let added = scene
        .apply_delta(RenderSceneDelta::new(vec![test_primitive(91)], Vec::new()))
        .expect("initial dependency add");

    assert_eq!(added.resource_reference_deltas().len(), 3);
    assert!(
        added
            .resource_reference_deltas()
            .iter()
            .all(|delta| delta.acquired_count() == 1 && delta.released_count() == 0)
    );
    assert_eq!(
        dependency_stats(&added),
        (1, 3, 3, 3),
        "one primitive payload must project three unique direct resources"
    );

    let removed = scene
        .apply_delta(RenderSceneDelta::new(Vec::new(), vec![stable_key(91)]))
        .expect("dependency removal");

    assert_eq!(removed.resource_reference_deltas().len(), 3);
    assert!(
        removed
            .resource_reference_deltas()
            .iter()
            .all(|delta| delta.acquired_count() == 0 && delta.released_count() == 1)
    );
    assert_eq!(
        resources(&added),
        resources(&removed),
        "removal must release the exact resources acquired by the primitive"
    );
    assert_eq!(dependency_stats(&removed), (1, 3, 3, 3));
}

#[test]
fn render_scene_resource_reference_delta_cancels_unchanged_model_and_mesh() {
    let mut scene = test_scene();
    scene
        .apply_delta(RenderSceneDelta::new(vec![test_primitive(92)], Vec::new()))
        .expect("initial dependency add");
    let changed = test_primitive_with_revisions(92, test_revisions(1, 1, 2, 1, 1), |descriptor| {
        descriptor.mesh_source = RenderSceneMeshSource::new(
            test_mesh_source_level_with_labels("base", "base", "replacement"),
            Vec::new(),
        );
    });

    let journal = scene
        .apply_delta(RenderSceneDelta::new(vec![changed], Vec::new()))
        .expect("material dependency update");
    let deltas = journal.resource_reference_deltas();

    assert_eq!(deltas.len(), 2);
    assert!(
        deltas
            .iter()
            .all(|delta| delta.resource().kind() == ResourceKind::Material)
    );
    assert_eq!(
        deltas
            .iter()
            .map(|delta| delta.acquired_count())
            .sum::<usize>(),
        1
    );
    assert_eq!(
        deltas
            .iter()
            .map(|delta| delta.released_count())
            .sum::<usize>(),
        1
    );
    assert_eq!(dependency_stats(&journal), (2, 6, 2, 2));
}

#[test]
fn render_scene_update_retains_previous_and_current_material_dependencies() {
    let mut scene = test_scene();
    let initial = scene
        .apply_delta(RenderSceneDelta::new(vec![test_primitive(13)], Vec::new()))
        .expect("initial add");
    let previous = Arc::clone(initial.additions()[0].primitive());
    let changed = test_primitive_with_revisions(13, test_revisions(1, 1, 2, 1, 1), |descriptor| {
        descriptor.mesh_source = RenderSceneMeshSource::new(
            test_mesh_source_level_with_labels("base", "base", "replacement"),
            Vec::new(),
        );
    });

    let journal = scene
        .apply_delta(RenderSceneDelta::new(vec![changed], Vec::new()))
        .expect("material dependency update");
    let update = &journal.updates()[0];

    assert_eq!(
        update.dirty(),
        super::super::RenderScenePrimitiveDirtyFlags::MATERIAL
    );
    assert!(Arc::ptr_eq(update.previous_primitive(), &previous));
    assert_eq!(
        update
            .previous_primitive()
            .descriptor()
            .mesh_source
            .base()
            .material,
        previous.descriptor().mesh_source.base().material
    );
    assert_ne!(
        update
            .previous_primitive()
            .descriptor()
            .mesh_source
            .base()
            .material,
        update.primitive().descriptor().mesh_source.base().material
    );
    assert_eq!(
        update.previous_primitive().stable_instance_key(),
        update.primitive().stable_instance_key()
    );
}

#[test]
fn render_scene_resource_reference_delta_cancels_cross_primitive_replacement() {
    let mut scene = test_scene();
    scene
        .apply_delta(RenderSceneDelta::new(vec![test_primitive(94)], Vec::new()))
        .expect("initial dependency add");

    let journal = scene
        .apply_delta(RenderSceneDelta::new(
            vec![test_primitive(95)],
            vec![stable_key(94)],
        ))
        .expect("same-resource primitive replacement");

    assert_eq!(journal.removals().len(), 1);
    assert_eq!(journal.additions().len(), 1);
    assert!(journal.resource_reference_deltas().is_empty());
    assert_eq!(dependency_stats(&journal), (2, 6, 6, 0));
}

#[test]
fn render_scene_resource_reference_delta_covers_complete_camera_neutral_source() {
    let mut descriptor = test_descriptor(93, stable_key(93));
    let base_model = model("complete/base-model");
    let lod_model = model("complete/lod-model");
    let shared_mesh = mesh("complete/shared-mesh");
    let base_primitive_mesh = mesh("complete/base-primitive-mesh");
    let lod_primitive_mesh = mesh("complete/lod-primitive-mesh");
    let shared_material = material("complete/shared-material");
    let base_primitive_material = material("complete/base-primitive-material");
    let lod_primitive_material = material("complete/lod-primitive-material");
    let override_material = material("complete/override-material");
    let skeleton = ResourceId::from_stable_label("tests/render-scene/complete/skeleton");
    descriptor.mesh_source = RenderSceneMeshSource::new(
        RenderSceneMeshSourceLevel::new(
            base_model,
            Some(shared_mesh),
            shared_material,
            vec![
                RenderSceneMeshBinding {
                    mesh: shared_mesh,
                    material: shared_material,
                },
                RenderSceneMeshBinding {
                    mesh: base_primitive_mesh,
                    material: base_primitive_material,
                },
            ],
        ),
        vec![RenderSceneMeshLod::new(
            20.0,
            RenderSceneMeshSourceLevel::new(
                lod_model,
                Some(shared_mesh),
                shared_material,
                vec![RenderSceneMeshBinding {
                    mesh: lod_primitive_mesh,
                    material: lod_primitive_material,
                }],
            ),
        )],
    );
    descriptor.common.material_overrides =
        MaterialOverrideSet::from_slots([(0, shared_material), (1, override_material)]);
    descriptor.skeletal_pose = Some(RenderSceneSkeletalPose::new(
        skeleton,
        Arc::new(AnimationPoseOutput {
            source: AnimationPoseSource::Clip,
            active_state: None,
            bones: Vec::new(),
        }),
    ));
    let primitive = RenderScenePrimitive::new(
        descriptor,
        RenderScenePrimitiveLocalBounds::new(
            RenderMeshBounds::from_min_max([-1.0; 3], [1.0; 3]),
            vec![RenderMeshBounds::from_min_max([-2.0; 3], [2.0; 3])],
        ),
        test_revisions(1, 1, 1, 1, 1),
    )
    .expect("complete dependency primitive");
    let mut scene = test_scene();

    let journal = scene
        .apply_delta(RenderSceneDelta::new(vec![primitive], Vec::new()))
        .expect("complete dependency add");
    let deltas = journal.resource_reference_deltas();
    let expected = HashSet::from([
        (ResourceKind::Model, base_model.id()),
        (ResourceKind::Model, lod_model.id()),
        (ResourceKind::Mesh, shared_mesh.id()),
        (ResourceKind::Mesh, base_primitive_mesh.id()),
        (ResourceKind::Mesh, lod_primitive_mesh.id()),
        (ResourceKind::Material, shared_material.id()),
        (ResourceKind::Material, base_primitive_material.id()),
        (ResourceKind::Material, lod_primitive_material.id()),
        (ResourceKind::Material, override_material.id()),
        (ResourceKind::AnimationSkeleton, skeleton),
    ]);

    assert_eq!(resources(&journal), expected);
    assert_eq!(deltas.len(), expected.len());
    assert!(
        deltas
            .iter()
            .all(|delta| delta.acquired_count() == 1 && delta.released_count() == 0)
    );
    assert!(
        deltas
            .windows(2)
            .all(|pair| dependency_sort_key(&pair[0]) < dependency_sort_key(&pair[1]))
    );
    assert_eq!(dependency_stats(&journal), (1, 10, 10, 10));
}

fn dependency_stats(
    journal: &super::super::RenderSceneChangeJournal,
) -> (usize, usize, usize, usize) {
    let stats = journal.resource_reference_stats();
    (
        stats.projected_primitive_payload_count(),
        stats.unique_dependency_key_visit_count(),
        stats.gross_observation_count(),
        stats.net_delta_count(),
    )
}

fn resources(
    journal: &super::super::RenderSceneChangeJournal,
) -> HashSet<(ResourceKind, ResourceId)> {
    journal
        .resource_reference_deltas()
        .iter()
        .map(|delta| (delta.resource().kind(), delta.resource().id()))
        .collect()
}

fn dependency_sort_key(
    delta: &super::super::RenderSceneResourceReferenceDelta,
) -> (u8, ResourceId) {
    (
        resource_kind_tag(delta.resource().kind()),
        delta.resource().id(),
    )
}

const fn resource_kind_tag(kind: ResourceKind) -> u8 {
    match kind {
        ResourceKind::Model => 0,
        ResourceKind::Mesh => 1,
        ResourceKind::Material => 2,
        ResourceKind::AnimationSkeleton => 3,
        _ => u8::MAX,
    }
}

fn model(label: &str) -> ResourceHandle<ModelMarker> {
    ResourceHandle::new(ResourceId::from_stable_label(&format!(
        "tests/render-scene/{label}"
    )))
}

fn mesh(label: &str) -> ResourceHandle<MeshMarker> {
    ResourceHandle::new(ResourceId::from_stable_label(&format!(
        "tests/render-scene/{label}"
    )))
}

fn material(label: &str) -> ResourceHandle<MaterialMarker> {
    ResourceHandle::new(ResourceId::from_stable_label(&format!(
        "tests/render-scene/{label}"
    )))
}
