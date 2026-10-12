use super::*;
use zircon_runtime::core::framework::render::cubemap_direction_from_scaled_uv;

#[test]
fn six_capture_views_transform_to_cmft_face_axes() {
    for view in REFLECTION_PROBE_CAPTURE_FACE_VIEWS {
        let camera = view.camera([0.0; 3], 0.1, 100.0);
        let raw_right = camera.transform.right().to_array();
        let raw_down = (-camera.transform.up()).to_array();
        let (stored_right, stored_down) = match view.storage_transform {
            ReflectionProbeCaptureStorageTransform::FlipHorizontal => {
                (negate(raw_right), raw_down)
            }
            ReflectionProbeCaptureStorageTransform::FlipVertical => {
                (raw_right, negate(raw_down))
            }
        };
        let face = view.face.cubemap_face();
        let center = cubemap_direction_from_scaled_uv(face, [0.0, 0.0]);
        let right = cubemap_direction_from_scaled_uv(face, [0.01, 0.0]);
        let down = cubemap_direction_from_scaled_uv(face, [0.0, 0.01]);

        assert_axis_close(stored_right, subtract(right, center));
        assert_axis_close(stored_down, subtract(down, center));
    }
}

#[test]
fn storage_transform_flips_expected_axis_without_transposing() {
    let mut horizontal = vec![[0.0; 4], [1.0; 4], [2.0; 4], [3.0; 4]];
    REFLECTION_PROBE_CAPTURE_FACE_VIEWS[0].transform_to_cmft_layout(2, &mut horizontal);
    assert_eq!(horizontal[0][0], 1.0);
    assert_eq!(horizontal[1][0], 0.0);
    assert_eq!(horizontal[2][0], 3.0);
    assert_eq!(horizontal[3][0], 2.0);

    let mut vertical = vec![[0.0; 4], [1.0; 4], [2.0; 4], [3.0; 4]];
    REFLECTION_PROBE_CAPTURE_FACE_VIEWS[2].transform_to_cmft_layout(2, &mut vertical);
    assert_eq!(vertical[0][0], 2.0);
    assert_eq!(vertical[1][0], 3.0);
    assert_eq!(vertical[2][0], 0.0);
    assert_eq!(vertical[3][0], 1.0);
}

fn negate(value: [f32; 3]) -> [f32; 3] {
    [-value[0], -value[1], -value[2]]
}

fn subtract(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [left[0] - right[0], left[1] - right[1], left[2] - right[2]]
}

fn assert_axis_close(actual: [f32; 3], expected: [f32; 3]) {
    let actual = Vec3::from_array(actual).normalize();
    let expected = Vec3::from_array(expected).normalize();
    assert!(
        actual.dot(expected) > 0.9999,
        "actual={actual:?} expected={expected:?}"
    );
}
