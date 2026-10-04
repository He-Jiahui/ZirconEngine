use std::collections::HashMap;

use crate::core::math::Mat4;
use crate::graphics::scene::scene_renderer::SkinnedMeshJointPaletteStorage;

use super::{grow_palette_capacity, stage_palette_span, SkinnedPaletteSpan};

#[test]
fn palette_span_projects_current_and_previous_instance_indirection() {
    let current = SkinnedPaletteSpan {
        matrix_base: 7,
        joint_count: 64,
    };
    let previous = SkinnedPaletteSpan {
        matrix_base: 11,
        joint_count: 32,
    };

    assert_eq!(current.params(Some(previous)), [7, 64, 11, 32]);
    assert_eq!(current.params(None), [7, 64, 0, 0]);
}

#[test]
fn palette_capacity_is_power_of_two_and_grow_only_at_the_owner() {
    assert_eq!(grow_palette_capacity(1), 1);
    assert_eq!(grow_palette_capacity(4_096), 4_096);
    assert_eq!(grow_palette_capacity(4_097), 8_192);
}

#[test]
fn palette_packing_is_contiguous_and_deduplicates_stable_instances() {
    let mut spans = HashMap::new();
    let mut matrices = Vec::new();
    let first = SkinnedMeshJointPaletteStorage::from_matrices(&[Mat4::IDENTITY; 2])
        .expect("test palette fits CPU snapshot");
    let second = SkinnedMeshJointPaletteStorage::from_matrices(&[Mat4::IDENTITY])
        .expect("test palette fits CPU snapshot");

    let first_span = stage_palette_span(&mut spans, &mut matrices, 11, &first);
    let second_span = stage_palette_span(&mut spans, &mut matrices, 22, &second);
    let repeated_first_span = stage_palette_span(&mut spans, &mut matrices, 11, &first);

    assert_eq!(
        first_span,
        SkinnedPaletteSpan {
            matrix_base: 0,
            joint_count: 2
        }
    );
    assert_eq!(
        second_span,
        SkinnedPaletteSpan {
            matrix_base: 2,
            joint_count: 1
        }
    );
    assert_eq!(repeated_first_span, first_span);
    assert_eq!(matrices.len(), 3);
    assert_eq!(spans.len(), 2);
}
