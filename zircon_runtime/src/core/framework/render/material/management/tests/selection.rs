use super::*;
use crate::core::framework::render::material::management::RenderMaterialManagementSnapshot;

#[test]
fn selection_owns_records_after_source_changes() {
    let material_id = ResourceId::from_stable_label("material:owned-selection");
    let mut records = vec![RenderMaterialManagementRecord {
        material_id,
        material_name: Some("Original".to_string()),
        snapshot: RenderMaterialManagementSnapshot::default(),
    }];

    let selection = RenderMaterialManagementSelection::from_records(&records, [material_id]);
    records[0].material_name = Some("Changed after selection".to_string());

    assert_eq!(selection.records.len(), 1);
    assert_eq!(
        selection.records[0].material_name.as_deref(),
        Some("Original")
    );
}
