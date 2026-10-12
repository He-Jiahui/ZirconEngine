use super::super::identity::is_workbench_chip;
use super::support::chip_node;

#[test]
fn workbench_chip_matches_viewport_chips_but_not_status_chips() {
    assert!(is_workbench_chip(&chip_node(
        "WorkbenchViewportMode",
        "Perspective",
    )));
    assert!(is_workbench_chip(&chip_node("WorkbenchChipRoot", "Chip")));
    assert!(!is_workbench_chip(&chip_node(
        "WorkbenchStatusGrid",
        "Grid: 10 cm",
    )));
}

#[test]
fn workbench_chip_matches_the_semantic_variant_after_component_expansion() {
    let mut node = chip_node("WorkbenchExtensionBlendSpacePreviewCamera", "Perspective");
    node.component_variant = "chip".into();

    assert!(
        is_workbench_chip(&node),
        "component roots expand to Label nodes, so their semantic chip variant must preserve the shared painter route"
    );
}
