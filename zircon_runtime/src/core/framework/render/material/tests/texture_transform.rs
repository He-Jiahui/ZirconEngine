use super::RenderMaterialTextureTransform;

#[test]
fn texture_transform_precomputes_rotation_sin_cos_with_finite_fallback() {
    assert_eq!(
        RenderMaterialTextureTransform::IDENTITY.as_uniform_rotation_sin_cos(),
        [1.0, 0.0]
    );

    let quarter_turn = RenderMaterialTextureTransform {
        rotation: std::f32::consts::FRAC_PI_2,
        ..RenderMaterialTextureTransform::IDENTITY
    };
    let [cos, sin] = quarter_turn.as_uniform_rotation_sin_cos();
    assert!(cos.abs() <= 0.000_001);
    assert!((sin - 1.0).abs() <= 0.000_001);

    let non_finite = RenderMaterialTextureTransform {
        rotation: f32::NAN,
        ..RenderMaterialTextureTransform::IDENTITY
    };
    assert_eq!(non_finite.as_uniform_rotation_sin_cos(), [1.0, 0.0]);
}
