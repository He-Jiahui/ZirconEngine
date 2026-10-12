use super::current_pointer_route;
use zircon_runtime_interface::ui::{event_ui::UiNodeId, tree::UiTreeError};

#[test]
fn current_pointer_route_preserves_the_ui_tree_error() {
    let expected = UiTreeError::MissingNode(UiNodeId::new(901));

    let result = current_pointer_route(Err(expected.clone()));

    assert_eq!(result.unwrap_err(), expected);
}
