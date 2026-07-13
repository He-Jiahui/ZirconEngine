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
    let mut property_label = title_node("WorkbenchTransformPositionLabel", "Position");
    property_label.component_variant = "property-label".into();
    assert!(!is_workbench_section_title(&property_label));
}

#[test]
fn section_title_identity_uses_semantic_variant_instead_of_control_id_suffix() {
    let mut ordinary_label = title_node("WorkbenchOrdinaryTitle", "Ordinary");
    ordinary_label.component_variant = "ordinary-label".into();
    assert!(!is_workbench_section_title(&ordinary_label));

    let semantic_title = title_node("ArbitraryControl", "Semantic");
    assert!(is_workbench_section_title(&semantic_title));
}
