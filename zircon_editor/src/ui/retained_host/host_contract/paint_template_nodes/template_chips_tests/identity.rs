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
