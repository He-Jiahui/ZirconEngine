use super::super::identity::is_workbench_section_title;
use super::support::title_node;

#[test]
fn workbench_section_title_matches_titles_without_row_labels() {
    assert!(is_workbench_section_title(&title_node(
        "WorkbenchButtonsTitle",
        "Buttons"
    )));
    assert!(is_workbench_section_title(&title_node(
        "WorkbenchTransformLabel",
        "Transform"
    )));
    assert!(!is_workbench_section_title(&title_node(
        "WorkbenchTransformPositionLabel",
        "Position"
    )));
}
