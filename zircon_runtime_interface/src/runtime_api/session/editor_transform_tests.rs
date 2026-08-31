use std::mem::{align_of, size_of};

use crate::math::{Quat, Transform, Vec3};
use crate::{
    ZrRuntimeEditorTransformPhaseV1, ZrRuntimeEditorTransformWriteV1, ZIRCON_RUNTIME_ABI_VERSION_V1,
};

fn transform(x: f32) -> Transform {
    Transform {
        translation: Vec3::new(x, 2.0, 3.0),
        rotation: Quat::from_rotation_y(0.25),
        scale: Vec3::new(1.0, 2.0, 3.0),
    }
}

#[test]
fn editor_transform_write_is_a_fixed_128_byte_payload() {
    assert_eq!(size_of::<ZrRuntimeEditorTransformWriteV1>(), 128);
    assert_eq!(align_of::<ZrRuntimeEditorTransformWriteV1>(), 8);
}

#[test]
fn editor_transform_write_preserves_expected_and_target_values() {
    let expected = transform(1.0);
    let target = transform(4.0);
    let request = ZrRuntimeEditorTransformWriteV1::new(
        7,
        11,
        2,
        13,
        ZrRuntimeEditorTransformPhaseV1::Preview,
        expected,
        target,
    );

    assert_eq!(request.abi_version, ZIRCON_RUNTIME_ABI_VERSION_V1);
    assert_eq!(request.entity, 7);
    assert_eq!(request.interaction_id, 11);
    assert_eq!(request.sequence, 2);
    assert_eq!(request.world_replacement_epoch, 13);
    assert_eq!(
        request.phase(),
        Some(ZrRuntimeEditorTransformPhaseV1::Preview)
    );
    assert_eq!(request.expected_transform(), expected);
    assert_eq!(request.target_transform(), target);
    assert!(request.validate_editor_transform_write());
}

#[test]
fn editor_transform_write_rejects_invalid_phase_sequence_and_transform() {
    let value = transform(1.0);
    let mut begin = ZrRuntimeEditorTransformWriteV1::new(
        7,
        11,
        1,
        13,
        ZrRuntimeEditorTransformPhaseV1::Begin,
        value,
        value,
    );
    assert!(begin.validate_editor_transform_write());

    begin.sequence = 2;
    assert!(!begin.validate_editor_transform_write());

    let mut preview = ZrRuntimeEditorTransformWriteV1::new(
        7,
        11,
        2,
        13,
        ZrRuntimeEditorTransformPhaseV1::Preview,
        value,
        value,
    );
    preview.target.translation[0] = f32::NAN;
    assert!(!preview.validate_editor_transform_write());

    preview.target.translation[0] = 1.0;
    preview.target.scale[2] = 0.0;
    assert!(!preview.validate_editor_transform_write());

    preview.target.scale[2] = 3.0;
    preview.phase = u32::MAX;
    assert!(!preview.validate_editor_transform_write());
}
