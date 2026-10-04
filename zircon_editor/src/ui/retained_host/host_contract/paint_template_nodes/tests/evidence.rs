use serde_json::json;

use super::{
    record_box_shadow_command, record_drawn_command, record_media_not_ready,
    record_owner_commands_complete, PaintEvidenceScope,
};
use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::{
    HostPaintCommand, PaintNodeIdentity,
};

fn owner() -> PaintNodeIdentity {
    PaintNodeIdentity {
        node_id: "generated-card".into(),
        parent_node_id: Some("generated-root".into()),
        source_path: "zircon_editor/assets/ui/editor/workbench.zui".into(),
        source_node_id: "card".into(),
        instance_path: "[]".into(),
        parent_source_path: Some("zircon_editor/assets/ui/editor/workbench.zui".into()),
        parent_source_node_id: Some("root".into()),
        parent_instance_path: Some("[]".into()),
        control_id: Some("card".into()),
    }
}

#[test]
fn shadow_commands_keep_a_separate_dpi_normalized_receipt() {
    let repo = std::env::current_dir().unwrap();
    let scope = PaintEvidenceScope::begin(&repo, 2.0).unwrap();
    let owner = owner();
    record_owner_commands_complete(&owner);

    let mut shadow = HostPaintCommand::quad(
        FrameRect {
            x: 14.0,
            y: 24.0,
            width: 100.0,
            height: 60.0,
        },
        None,
        0,
        Some([1, 2, 3, 200]),
        None,
        0.0,
        8.0,
        0.75,
    )
    .with_box_shadow(2.0, 4.0, 0.0, 2.0, 8.0, false);
    shadow.set_owner(owner.clone());

    let mut surface = HostPaintCommand::quad(
        FrameRect {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 60.0,
        },
        None,
        1,
        Some([10, 20, 30, 255]),
        Some([40, 50, 60, 255]),
        2.0,
        8.0,
        0.75,
    );
    surface.set_owner(owner);

    record_box_shadow_command(&shadow);
    record_drawn_command(&shadow);
    record_drawn_command(&surface);
    let evidence = scope.finish().unwrap();
    let style = &evidence["styles"][0];
    assert_eq!(
        style["effectiveStyle"]["properties"]["backgroundColor"],
        "rgba(10,20,30,1)"
    );
    assert_eq!(style["effectiveStyle"]["properties"]["opacity"], 0.75);
    assert_eq!(
        style["styleInventory"]["properties"]["boxShadow"]["layers"][0]["color"],
        "rgba(1,2,3,1)"
    );
    assert_eq!(
        style["styleInventory"]["properties"]["boxShadow"]["layers"][0]["offsetY"],
        2.0
    );
    let opacity = style["styleInventory"]["properties"]["boxShadow"]["layers"][0]["opacity"]
        .as_f64()
        .unwrap();
    assert!((opacity - ((200.0 / 255.0) * 0.75)).abs() < 0.00001);
    assert_eq!(
        style["styleInventory"]["properties"]["boxShadow"]["layers"][0]["inset"],
        json!(false)
    );
}

#[test]
fn painted_placeholder_without_file_backed_pixels_keeps_media_audit_pending() {
    let repo = std::env::current_dir().unwrap();
    let scope = PaintEvidenceScope::begin(&repo, 1.0).unwrap();
    record_media_not_ready(Some("deferred-image"));
    let evidence = scope.finish().unwrap();

    assert_eq!(evidence["assetAudit"]["complete"], false);
    assert_eq!(
        evidence["issues"],
        json!(["image resource deferred-image was painted without ready file-backed pixels"])
    );
}
