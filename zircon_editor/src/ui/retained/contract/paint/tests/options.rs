use super::super::options::{segmented_options, selected_segment_value};
use super::support::segmented_node;

#[test]
fn segmented_options_prefer_declared_option_cells() {
    let node = segmented_node();
    let options = segmented_options(&node).collect::<Vec<_>>();

    assert_eq!(options, vec!["left", "center", "right"]);
    let selected: Option<&str> = selected_segment_value(&node);
    assert_eq!(selected, Some("center"));
}
