use super::super::options::{segmented_options, selected_segment_value};
use super::support::segmented_node;
use crate::ui::retained_host::primitives::SharedString;

#[test]
fn segmented_options_prefer_declared_option_cells() {
    let node = segmented_node();
    let options: Vec<SharedString> = segmented_options(&node);

    assert_eq!(
        options,
        vec![
            SharedString::from("left"),
            SharedString::from("center"),
            SharedString::from("right")
        ]
    );
    let selected: Option<&str> = selected_segment_value(&node);
    assert_eq!(selected, Some("center"));
}
