use crate::{try_to_render_scalar, RenderNarrowingError};

#[test]
fn checked_render_narrowing_preserves_current_f32_values_and_emits_a_receipt() {
    let receipt = try_to_render_scalar(3.5).expect("current f32 scalar is renderable");

    assert_eq!(receipt.source(), 3.5);
    assert_eq!(receipt.rendered(), 3.5);
    assert_eq!(receipt.absolute_error(), 0.0);
    assert!(receipt.is_exact());
}

#[test]
fn checked_render_narrowing_rejects_non_finite_source() {
    assert!(matches!(
        try_to_render_scalar(f32::NAN),
        Err(RenderNarrowingError::NonFiniteSource)
    ));
}
