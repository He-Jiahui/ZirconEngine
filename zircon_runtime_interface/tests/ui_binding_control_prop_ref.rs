//! Focused parser contract for cross-control property references.

use zircon_runtime_interface::ui::template::{UiBindingExpression, UiBindingExpressionParseError};

#[test]
fn parses_cross_control_property_reference() {
    assert_eq!(
        UiBindingExpression::parse("control.NavigationBakeSurfaceList.prop.selected_entity")
            .unwrap(),
        UiBindingExpression::ControlPropRef {
            control_id: "NavigationBakeSurfaceList".to_string(),
            property: "selected_entity".to_string(),
        }
    );
}

#[test]
fn rejects_incomplete_cross_control_property_reference() {
    assert!(matches!(
        UiBindingExpression::parse("control.NavigationBakeSurfaceList.prop"),
        Err(UiBindingExpressionParseError::UnexpectedToken(_))
    ));
}

#[test]
fn rejects_invalid_cross_control_property_segment() {
    assert!(matches!(
        UiBindingExpression::parse("control.NavigationBakeSurfaceList.state.selected_entity"),
        Err(UiBindingExpressionParseError::UnexpectedToken(_))
    ));
}
