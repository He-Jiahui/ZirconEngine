use zircon_runtime::core::framework::animation::AnimationSkeletonAsset;
use zircon_runtime::core::math::{Mat4, Quat, Real, Vec3};
use zircon_runtime::core::resource::ResourceManager;

use super::super::{SkeletonReferenceSpace, SkeletonReferenceTransformError};
use super::{bone, install, record, SkeletonReference};

fn assert_point(matrix: Mat4, point: Vec3, expected: Vec3) {
    let actual = matrix.transform_point3(point);
    assert!(
        actual.abs_diff_eq(expected, 0.0001),
        "{actual:?} != {expected:?}"
    );
}

#[test]
fn reference_matrices_compose_parent_trs_without_changing_raw_source_values() {
    let resources = ResourceManager::new();
    let mut child = bone("Head", Some(2));
    child.local_translation = [1.0, 2.0, 3.0];
    child.local_rotation = [0.0, 0.0, 0.0, 2.0];
    child.local_scale = [-1.0, 0.0, 2.0];
    let mut unrelated = bone("Head", None);
    unrelated.local_translation[0] = Real::NAN;
    let mut parent = bone("Root", None);
    parent.local_translation = [10.0, -4.0, 2.0];
    parent.local_rotation = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2).to_array();
    parent.local_scale = [2.0, 3.0, 4.0];
    let version = install(
        &resources,
        record("space-trs"),
        vec![child, unrelated, parent],
    );
    let reference = SkeletonReference::read(&resources, &version).unwrap();
    let key = reference.key(0).unwrap();
    let local = reference
        .reference_matrix(&key, SkeletonReferenceSpace::Local)
        .unwrap()
        .unwrap();
    assert_point(local, Vec3::ONE, Vec3::new(0.0, 2.0, 5.0));
    let root = reference
        .reference_matrix(&key, SkeletonReferenceSpace::SkeletonRoot)
        .unwrap()
        .unwrap();
    assert_point(root, Vec3::ZERO, Vec3::new(4.0, -2.0, 14.0));
    assert_point(root, Vec3::ONE, Vec3::new(4.0, -4.0, 22.0));
    assert_eq!(
        *reference.details(&key).unwrap().reference_local_rotation,
        [0.0, 0.0, 0.0, 2.0]
    );
    assert_eq!(
        *reference.details(&key).unwrap().reference_local_scale,
        [-1.0, 0.0, 2.0]
    );
}

#[test]
fn reference_root_matrices_preserve_inherited_shear() {
    let resources = ResourceManager::new();
    let mut root = bone("Root", None);
    root.local_translation = [0.0; 3];
    root.local_scale = [2.0, 1.0, 1.0];
    let mut child = bone("Child", Some(0));
    child.local_translation = [0.0; 3];
    child.local_rotation = Quat::from_rotation_z(std::f32::consts::FRAC_PI_4).to_array();
    let version = install(&resources, record("space-shear"), vec![root, child]);
    let reference = SkeletonReference::read(&resources, &version).unwrap();
    let matrix = reference
        .reference_matrix(
            &reference.key(1).unwrap(),
            SkeletonReferenceSpace::SkeletonRoot,
        )
        .unwrap()
        .unwrap();
    assert_point(
        matrix,
        Vec3::X,
        Vec3::new(
            std::f32::consts::SQRT_2,
            std::f32::consts::FRAC_1_SQRT_2,
            0.0,
        ),
    );
    assert!((matrix.x_axis.truncate().dot(matrix.y_axis.truncate()) + 1.5).abs() < 0.0001);
}

#[test]
fn reference_matrices_report_invalid_ancestor_values_without_identity_fallback() {
    for variant in 0..4 {
        let resources = ResourceManager::new();
        let mut invalid = bone("Parent", None);
        match variant {
            0 => invalid.local_translation[0] = Real::NAN,
            1 => invalid.local_rotation = [0.0; 4],
            2 => invalid.local_scale[2] = Real::INFINITY,
            _ => invalid.local_rotation = [Real::MAX; 4],
        }
        let version = install(
            &resources,
            record("space-invalid"),
            vec![bone("Child", Some(1)), invalid],
        );
        let reference = SkeletonReference::read(&resources, &version).unwrap();
        let key = reference.key(0).unwrap();
        assert!(reference
            .reference_matrix(&key, SkeletonReferenceSpace::Local)
            .unwrap()
            .is_some());
        assert!(matches!(
            reference.reference_matrix(&key, SkeletonReferenceSpace::SkeletonRoot),
            Err(SkeletonReferenceTransformError::InvalidLocal { joint_index: 1, .. })
        ));
        assert_eq!(reference.details(&key).unwrap().name, "Child");
    }
}

#[test]
fn reference_matrices_reject_non_finite_composition() {
    let resources = ResourceManager::new();
    let mut root = bone("Root", None);
    root.local_scale = [Real::MAX / 4.0, 1.0, 1.0];
    let mut child = bone("Child", Some(0));
    child.local_scale = [16.0, 1.0, 1.0];
    let version = install(&resources, record("space-overflow"), vec![root, child]);
    let reference = SkeletonReference::read(&resources, &version).unwrap();
    let key = reference.key(1).unwrap();
    assert!(reference
        .reference_matrix(&key, SkeletonReferenceSpace::Local)
        .unwrap()
        .is_some());
    assert!(matches!(
        reference.reference_matrix(&key, SkeletonReferenceSpace::SkeletonRoot),
        Err(SkeletonReferenceTransformError::NonFiniteMatrix { joint_index: 1 })
    ));
}

#[test]
fn reference_matrices_reject_foreign_replaced_and_recreated_keys() {
    let resources = ResourceManager::new();
    let original_record = record("space-identity");
    let version = install(
        &resources,
        original_record.clone(),
        vec![bone("Head", None)],
    );
    let original = SkeletonReference::read(&resources, &version).unwrap();
    let old_key = original.key(0).unwrap();
    let foreign_version = install(&resources, record("space-other"), vec![bone("Head", None)]);
    let foreign = SkeletonReference::read(&resources, &foreign_version).unwrap();
    resources
        .store_payload(
            original_record.id,
            original.record().revision,
            AnimationSkeletonAsset {
                name: None,
                bones: vec![bone("Head", None)],
            },
        )
        .unwrap();
    let replacement_version = resources
        .projection_snapshot()
        .active()
        .key(original_record.id)
        .unwrap();
    let replacement = SkeletonReference::read(&resources, &replacement_version).unwrap();
    resources
        .remove_by_locator(&original_record.primary_locator)
        .unwrap()
        .unwrap();
    let recreated_version = install(&resources, original_record, vec![bone("Head", None)]);
    let recreated = SkeletonReference::read(&resources, &recreated_version).unwrap();
    assert_eq!(recreated.record().revision, original.record().revision);
    for space in [
        SkeletonReferenceSpace::Local,
        SkeletonReferenceSpace::SkeletonRoot,
    ] {
        assert!(foreign.reference_matrix(&old_key, space).unwrap().is_none());
        assert!(replacement
            .reference_matrix(&old_key, space)
            .unwrap()
            .is_none());
        assert!(recreated
            .reference_matrix(&old_key, space)
            .unwrap()
            .is_none());
        assert!(recreated
            .reference_matrix(&replacement.key(0).unwrap(), space)
            .unwrap()
            .is_none());
        assert!(original
            .reference_matrix(&old_key, space)
            .unwrap()
            .is_some());
    }
}

#[test]
fn reference_root_matrix_walk_is_iterative_for_deep_skeletons() {
    let resources = ResourceManager::new();
    let count = 20_000_usize;
    let bones = (0..count)
        .map(|index| {
            let mut item = bone("Joint", index.checked_sub(1).map(|parent| parent as u32));
            item.local_translation = [1.0, 0.0, 0.0];
            item
        })
        .collect();
    let version = install(&resources, record("space-deep"), bones);
    let reference = SkeletonReference::read(&resources, &version).unwrap();
    let matrix = reference
        .reference_matrix(
            &reference.key((count - 1) as u32).unwrap(),
            SkeletonReferenceSpace::SkeletonRoot,
        )
        .unwrap()
        .unwrap();
    assert_point(matrix, Vec3::ZERO, Vec3::new(count as Real, 0.0, 0.0));
}
