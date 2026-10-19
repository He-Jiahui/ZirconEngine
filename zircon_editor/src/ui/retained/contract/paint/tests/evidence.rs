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
fn owner_scope_is_idle_without_capture_and_shares_identity_during_capture() {
    let owner = std::rc::Rc::new(owner());
    {
        let _scope = super::PaintNodeOwnerScope::enter(Some(&owner));
        assert!(super::active_owner().is_none());
    }
    let capture = PaintEvidenceScope::begin(&std::env::current_dir().unwrap(), 1.0).unwrap();
    {
        let _scope = super::PaintNodeOwnerScope::enter(Some(&owner));
        assert!(std::rc::Rc::ptr_eq(&owner, &super::active_owner().unwrap()));
    }
    assert!(super::active_owner().is_none());
    capture.finish().unwrap();
    assert!(!super::capture_active());
}

#[test]
fn shadow_commands_keep_a_separate_dpi_normalized_receipt() {
    let repo = std::env::current_dir().unwrap();
    let scope = PaintEvidenceScope::begin(&repo, 2.0).unwrap();
    let owner = std::rc::Rc::new(owner());
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

#[test]
fn normal_paint_evidence_freezes_the_theme_at_capture_begin() {
    use crate::ui::retained_host::host_contract::paint_theme::{
        enter_host_paint_theme_scope, host_paint_theme_snapshot_from_tokens_for_test,
    };
    use zircon_runtime_interface::ui::design_tokens::EditorDesignTokens;

    let mut first = EditorDesignTokens::workbench_dark();
    first.id = "paint-input-at-begin".into();
    let first = host_paint_theme_snapshot_from_tokens_for_test(&first);
    let _first = enter_host_paint_theme_scope(first);
    let repo = std::env::current_dir().unwrap();
    let evidence = PaintEvidenceScope::begin_for_normal_present(&repo).unwrap();
    let mut second = EditorDesignTokens::workbench_dark();
    second.id = "later-theme".into();
    let second = host_paint_theme_snapshot_from_tokens_for_test(&second);
    let _second = enter_host_paint_theme_scope(second);
    let observed = evidence.finish().unwrap();
    assert_eq!(
        observed["designTokenInput"]["tokens"]["id"],
        "paint-input-at-begin"
    );
    assert!(observed["designTokenInput"]["settingsGeneration"].is_null());
}

#[test]
fn normal_paint_evidence_without_a_frame_theme_keeps_token_input_unobserved() {
    let repo = std::env::current_dir().unwrap();
    let evidence = PaintEvidenceScope::begin_for_normal_present(&repo).unwrap();
    assert!(evidence.finish().unwrap()["designTokenInput"].is_null());
}
