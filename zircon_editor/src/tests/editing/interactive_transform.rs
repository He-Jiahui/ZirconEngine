use zircon_runtime::scene::components::{Mobility, NodeKind};
use zircon_runtime::scene::Scene;
use zircon_runtime_interface::math::{Quat, Transform, Vec3};

use crate::core::editing::interactive_transform::{
    InteractiveTransformAxis, InteractiveTransformError, InteractiveTransformKind,
    InteractiveTransformSession, InteractiveTransformSpace, InteractiveTransformSpec, PivotMode,
};
use crate::core::editor_message::DocumentId;

const DOCUMENT: DocumentId = DocumentId::new(301);

fn move_spec() -> InteractiveTransformSpec {
    InteractiveTransformSpec::new(
        InteractiveTransformKind::Move,
        InteractiveTransformAxis::X,
        InteractiveTransformSpace::Global,
        false,
    )
}

#[test]
fn interactive_transform_filters_selected_descendants_and_moves_roots_as_one_batch() {
    let mut scene = Scene::empty();
    let parent = scene.spawn_node(NodeKind::Empty).unwrap();
    let child = scene.spawn_node(NodeKind::Cube).unwrap();
    let sibling = scene.spawn_node(NodeKind::Cube).unwrap();
    scene.set_parent_checked(child, Some(parent)).unwrap();
    scene
        .update_transform(
            parent,
            Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
        )
        .unwrap();
    scene
        .update_transform(child, Transform::from_translation(Vec3::new(2.0, 0.0, 0.0)))
        .unwrap();
    scene
        .update_transform(
            sibling,
            Transform::from_translation(Vec3::new(0.0, 2.0, 0.0)),
        )
        .unwrap();

    let mut session = InteractiveTransformSession::begin(
        &scene,
        &[parent, child, sibling, parent, sibling],
        child,
        move_spec(),
        PivotMode::Centroid,
        DOCUMENT,
    )
    .unwrap();
    assert_eq!(session.primary_root(), parent);
    assert_eq!(
        session.target_entities().collect::<Vec<_>>(),
        vec![parent, sibling]
    );

    let mut target = session.pivot_transform();
    target.translation.x += 3.0;
    session
        .preview(&mut scene, Some(DOCUMENT), parent, target)
        .unwrap();

    assert_eq!(
        scene.world_transform(parent).unwrap().translation,
        Vec3::new(4.0, 0.0, 0.0)
    );
    assert_eq!(
        scene.world_transform(child).unwrap().translation,
        Vec3::new(6.0, 0.0, 0.0)
    );
    assert_eq!(
        scene.world_transform(sibling).unwrap().translation,
        Vec3::new(3.0, 2.0, 0.0)
    );

    let command = session.finish(&scene, Some(DOCUMENT)).unwrap().unwrap();
    assert_eq!(command.target_count(), 2);
    session.cancel(&mut scene, Some(DOCUMENT)).unwrap();
    assert_eq!(
        scene.world_transform(parent).unwrap().translation,
        Vec3::new(1.0, 0.0, 0.0)
    );
    assert_eq!(
        scene.world_transform(child).unwrap().translation,
        Vec3::new(3.0, 0.0, 0.0)
    );
    assert_eq!(
        scene.world_transform(sibling).unwrap().translation,
        Vec3::new(0.0, 2.0, 0.0)
    );
}

#[test]
fn interactive_transform_rejects_static_targets_before_preview() {
    let mut scene = Scene::empty();
    let dynamic = scene.spawn_node(NodeKind::Cube).unwrap();
    let static_node = scene.spawn_node(NodeKind::Cube).unwrap();
    scene.set_mobility(static_node, Mobility::Static).unwrap();
    let dynamic_before = scene.local_transform(dynamic).unwrap();

    assert!(matches!(
        InteractiveTransformSession::begin(
            &scene,
            &[dynamic, static_node],
            dynamic,
            move_spec(),
            PivotMode::Centroid,
            DOCUMENT,
        ),
        Err(InteractiveTransformError::TargetNotMutable { entity })
            if entity == static_node
    ));
    assert_eq!(scene.local_transform(dynamic), Some(dynamic_before));
}

#[test]
fn interactive_transform_rejects_stale_document_and_primary_requests() {
    let mut scene = Scene::empty();
    let primary = scene.spawn_node(NodeKind::Cube).unwrap();
    let other = scene.spawn_node(NodeKind::Cube).unwrap();
    let before = scene.local_transform(primary).unwrap();
    let mut target = scene.world_transform(primary).unwrap();
    target.translation.x += 2.0;
    let mut session = InteractiveTransformSession::begin(
        &scene,
        &[primary],
        primary,
        move_spec(),
        PivotMode::Centroid,
        DOCUMENT,
    )
    .unwrap();

    assert!(matches!(
        session.preview(&mut scene, Some(DOCUMENT), other, target),
        Err(InteractiveTransformError::PrimaryTargetMismatch { expected, actual })
            if expected == primary && actual == other
    ));
    assert!(matches!(
        session.preview(&mut scene, Some(DocumentId::new(302)), primary, target),
        Err(InteractiveTransformError::DocumentChanged { expected, actual })
            if expected == DOCUMENT && actual == Some(DocumentId::new(302))
    ));
    assert_eq!(scene.local_transform(primary), Some(before));
}

#[test]
fn interactive_transform_converts_world_translation_through_parent_inverse() {
    let mut scene = Scene::empty();
    let parent = scene.spawn_node(NodeKind::Empty).unwrap();
    let child = scene.spawn_node(NodeKind::Cube).unwrap();
    scene.set_parent_checked(child, Some(parent)).unwrap();
    scene
        .update_transform(
            parent,
            Transform::identity().with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
        )
        .unwrap();
    scene
        .update_transform(child, Transform::from_translation(Vec3::new(2.0, 0.0, 0.0)))
        .unwrap();

    let before_world = scene.world_transform(child).unwrap();
    let mut session = InteractiveTransformSession::begin(
        &scene,
        &[child],
        child,
        move_spec(),
        PivotMode::Centroid,
        DOCUMENT,
    )
    .unwrap();
    let mut target = before_world;
    target.translation.x += 1.0;
    session
        .preview(&mut scene, Some(DOCUMENT), child, target)
        .unwrap();

    let after_world = scene.world_transform(child).unwrap();
    assert!((after_world.translation - target.translation).length() < 1.0e-4);
    assert!((scene.local_transform(child).unwrap().translation.y + 1.0).abs() < 1.0e-4);
}

#[test]
fn interactive_transform_rejects_shear_without_partial_preview() {
    let mut scene = Scene::empty();
    let parent = scene.spawn_node(NodeKind::Empty).unwrap();
    let child = scene.spawn_node(NodeKind::Cube).unwrap();
    scene.set_parent_checked(child, Some(parent)).unwrap();
    scene
        .update_transform(
            parent,
            Transform::identity()
                .with_rotation(Quat::from_rotation_z(0.4))
                .with_scale(Vec3::new(2.0, 1.0, 1.0)),
        )
        .unwrap();

    let before = scene.local_transform(child).unwrap();
    let mut session = InteractiveTransformSession::begin(
        &scene,
        &[child],
        child,
        InteractiveTransformSpec::new(
            InteractiveTransformKind::Rotate,
            InteractiveTransformAxis::Z,
            InteractiveTransformSpace::Global,
            false,
        ),
        PivotMode::Centroid,
        DOCUMENT,
    )
    .unwrap();
    let mut target = scene.world_transform(child).unwrap();
    target.rotation = Quat::from_rotation_z(1.1) * target.rotation;

    assert!(matches!(
        session.preview(&mut scene, Some(DOCUMENT), child, target),
        Err(InteractiveTransformError::NonRepresentableTransform { entity, .. })
            if entity == child
    ));
    assert_eq!(scene.local_transform(child), Some(before));
}

#[test]
fn pivot_modes_distinguish_primary_origin_from_selection_centroid() {
    let mut scene = Scene::empty();
    let left = scene.spawn_node(NodeKind::Cube).unwrap();
    let right = scene.spawn_node(NodeKind::Cube).unwrap();
    scene
        .update_transform(left, Transform::from_translation(Vec3::new(-2.0, 0.0, 0.0)))
        .unwrap();
    scene
        .update_transform(right, Transform::from_translation(Vec3::new(2.0, 0.0, 0.0)))
        .unwrap();

    let primary = InteractiveTransformSession::begin(
        &scene,
        &[left, right],
        left,
        move_spec(),
        PivotMode::Primary,
        DOCUMENT,
    )
    .unwrap();
    let centroid = InteractiveTransformSession::begin(
        &scene,
        &[left, right],
        left,
        move_spec(),
        PivotMode::Centroid,
        DOCUMENT,
    )
    .unwrap();

    assert_eq!(primary.pivot_mode(), PivotMode::Primary);
    assert_eq!(primary.pivot_world(), Vec3::new(-2.0, 0.0, 0.0));
    assert_eq!(centroid.pivot_mode(), PivotMode::Centroid);
    assert_eq!(centroid.pivot_world(), Vec3::ZERO);
}

#[test]
fn multi_selection_rotation_uses_the_frozen_centroid_as_the_shared_pivot() {
    let mut scene = Scene::empty();
    let left = scene.spawn_node(NodeKind::Cube).unwrap();
    let right = scene.spawn_node(NodeKind::Cube).unwrap();
    scene
        .update_transform(left, Transform::from_translation(Vec3::new(-2.0, 0.0, 0.0)))
        .unwrap();
    scene
        .update_transform(right, Transform::from_translation(Vec3::new(2.0, 0.0, 0.0)))
        .unwrap();
    let rotate = InteractiveTransformSpec::new(
        InteractiveTransformKind::Rotate,
        InteractiveTransformAxis::Z,
        InteractiveTransformSpace::Global,
        false,
    );
    let mut session = InteractiveTransformSession::begin(
        &scene,
        &[left, right],
        left,
        rotate,
        PivotMode::Centroid,
        DOCUMENT,
    )
    .unwrap();

    assert_eq!(session.pivot_mode(), PivotMode::Centroid);
    assert_eq!(session.pivot_world(), Vec3::ZERO);
    let target_pivot =
        Transform::identity().with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2));
    session
        .preview(&mut scene, Some(DOCUMENT), left, target_pivot)
        .unwrap();

    let left_after = scene.world_transform(left).unwrap().translation;
    let right_after = scene.world_transform(right).unwrap().translation;
    assert!((left_after - Vec3::new(0.0, -2.0, 0.0)).length() < 1.0e-4);
    assert!((right_after - Vec3::new(0.0, 2.0, 0.0)).length() < 1.0e-4);
}

#[test]
fn global_scale_uses_world_axes_for_a_rotated_single_selection() {
    let mut scene = Scene::empty();
    let primary = scene.spawn_node(NodeKind::Cube).unwrap();
    scene
        .update_transform(
            primary,
            Transform::from_translation(Vec3::new(3.0, 2.0, 0.0))
                .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
        )
        .unwrap();
    let scale = InteractiveTransformSpec::new(
        InteractiveTransformKind::Scale,
        InteractiveTransformAxis::X,
        InteractiveTransformSpace::Global,
        false,
    );
    let mut session = InteractiveTransformSession::begin(
        &scene,
        &[primary],
        primary,
        scale,
        PivotMode::Primary,
        DOCUMENT,
    )
    .unwrap();
    let mut target_pivot = session.pivot_transform();
    target_pivot.scale.x *= 2.0;

    session
        .preview(&mut scene, Some(DOCUMENT), primary, target_pivot)
        .unwrap();

    let after = scene.world_matrix(primary).unwrap();
    assert!((after.transform_point3(Vec3::ZERO) - Vec3::new(3.0, 2.0, 0.0)).length() < 1.0e-4);
    assert!((after.transform_vector3(Vec3::X).length() - 1.0).abs() < 1.0e-4);
    assert!((after.transform_vector3(Vec3::Y).length() - 2.0).abs() < 1.0e-4);
}

#[test]
fn global_scale_moves_a_rotated_multi_selection_along_world_axes_about_the_centroid() {
    let mut scene = Scene::empty();
    let left = scene.spawn_node(NodeKind::Cube).unwrap();
    let right = scene.spawn_node(NodeKind::Cube).unwrap();
    scene
        .update_transform(
            left,
            Transform::from_translation(Vec3::new(-2.0, 0.0, 0.0))
                .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
        )
        .unwrap();
    scene
        .update_transform(right, Transform::from_translation(Vec3::new(2.0, 0.0, 0.0)))
        .unwrap();
    let scale = InteractiveTransformSpec::new(
        InteractiveTransformKind::Scale,
        InteractiveTransformAxis::X,
        InteractiveTransformSpace::Global,
        false,
    );
    let mut session = InteractiveTransformSession::begin(
        &scene,
        &[left, right],
        left,
        scale,
        PivotMode::Centroid,
        DOCUMENT,
    )
    .unwrap();
    let mut target_pivot = session.pivot_transform();
    target_pivot.scale.x *= 2.0;

    session
        .preview(&mut scene, Some(DOCUMENT), left, target_pivot)
        .unwrap();

    assert!(
        (scene.world_transform(left).unwrap().translation - Vec3::new(-4.0, 0.0, 0.0)).length()
            < 1.0e-4
    );
    assert!(
        (scene.world_transform(right).unwrap().translation - Vec3::new(4.0, 0.0, 0.0)).length()
            < 1.0e-4
    );
}
