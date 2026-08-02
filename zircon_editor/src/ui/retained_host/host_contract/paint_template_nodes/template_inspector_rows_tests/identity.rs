use super::super::super::template_inspector_row_kind::{
    InspectorResourceKind, InspectorRowKind, inspector_row_kind,
};
use super::support::inspector_node;

#[test]
fn inspector_row_kind_only_promotes_known_resource_and_shadow_rows() {
    assert_eq!(
        inspector_row_kind(&inspector_node("WorkbenchMeshRow", "Mesh", "Box_01")),
        Some(InspectorRowKind::Resource(InspectorResourceKind::Mesh))
    );
    assert_eq!(
        inspector_row_kind(&inspector_node(
            "WorkbenchMaterialRow",
            "Cast Shadows",
            "false"
        )),
        Some(InspectorRowKind::ShadowSelect)
    );
    assert_eq!(
        inspector_row_kind(&inspector_node("WorkbenchMeshRow", "Visible", "true")),
        None
    );
}
