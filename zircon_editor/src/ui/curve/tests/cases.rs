use super::{
    CurveBounds, CurveCanvasTransform, CurveElementKind, CurveElementRef, CurveInterpolation,
    CurveKey, CurvePoint, CurveSelection, CurveView,
};

#[test]
fn canvas_transform_round_trips_curve_coordinates_with_an_inverted_value_axis() {
    let transform = CurveCanvasTransform::new(
        CurveBounds::new(0.0, 4.0, -2.0, 2.0),
        CurvePoint::new(400.0, 200.0),
    );

    let screen = transform.curve_to_screen(CurvePoint::new(1.0, 1.0));
    let round_trip = transform.screen_to_curve(screen);

    assert_eq!(screen, CurvePoint::new(100.0, 50.0));
    assert!((round_trip.time - 1.0).abs() < 1.0e-6);
    assert!((round_trip.value - 1.0).abs() < 1.0e-6);
}

#[test]
fn selection_keeps_key_and_tangent_handles_distinct() {
    let mut selection = CurveSelection::default();
    let key = "Root/Hero:Transform.translation.x@00000000".to_string();

    let changed = selection.replace([
        CurveElementRef::new("translation.x", key.clone(), CurveElementKind::Key),
        CurveElementRef::new("translation.x", key.clone(), CurveElementKind::OutTangent),
    ]);

    assert!(changed);
    assert_eq!(selection.elements().len(), 2);
    assert!(selection.contains(&CurveElementRef::new(
        "translation.x",
        key,
        CurveElementKind::OutTangent,
    )));
}

#[test]
fn curve_view_limits_visible_keys_without_copying_the_curve_model() {
    let curve = CurveView {
        id: "translation.x".to_string(),
        display_name: "Translation X".to_string(),
        interpolation: CurveInterpolation::Hermite,
        keys: vec![
            CurveKey::new("first", CurvePoint::new(0.0, 0.0)),
            CurveKey::new("second", CurvePoint::new(2.0, 1.0)),
            CurveKey::new("third", CurvePoint::new(4.0, 3.0)),
        ],
    };

    let visible = curve.keys_in_bounds(CurveBounds::new(1.0, 3.0, -1.0, 2.0));

    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].id, "second");
}
