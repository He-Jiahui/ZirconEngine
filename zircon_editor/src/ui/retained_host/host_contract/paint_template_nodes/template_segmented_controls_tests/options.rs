use super::super::options::{segmented_options, selected_segment_value};
use super::support::segmented_node;

#[test]
fn segmented_options_prefer_declared_option_cells() {
    let node = segmented_node();

    assert_eq!(
        segmented_options(&node),
        vec![
            "left".to_string(),
            "center".to_string(),
            "right".to_string()
        ]
    );
    assert_eq!(selected_segment_value(&node).as_deref(), Some("center"));
}
