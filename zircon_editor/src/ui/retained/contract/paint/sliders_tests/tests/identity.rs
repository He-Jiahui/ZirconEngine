use super::super::is_workbench_slider;
use super::support::slider_node;

#[test]
fn workbench_slider_recognizes_slider_ids_and_roles() {
    assert!(is_workbench_slider(&slider_node(
        "WorkbenchInputSlider",
        0.75
    )));
    assert!(is_workbench_slider(&slider_node(
        "WorkbenchInputRangeSlider",
        0.8
    )));
    assert!(is_workbench_slider(&slider_node(
        "WorkbenchInputStepsSlider",
        0.86
    )));
    assert!(is_workbench_slider(&slider_node(
        "WorkbenchSliderRoot",
        0.5
    )));
    assert!(!is_workbench_slider(&slider_node("PlainSlider", 0.5)));
}

#[test]
fn declared_workbench_slider_variant_is_recognized_without_workbench_id() {
    let mut node = slider_node("PlainSlider", 0.5);
    node.component_variant = "workbench-slider".into();

    assert!(is_workbench_slider(&node));
}
