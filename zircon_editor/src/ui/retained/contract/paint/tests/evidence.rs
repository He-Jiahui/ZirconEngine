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
        item_key: None,
        property_field_id: None,
        parent_source_path: Some("zircon_editor/assets/ui/editor/workbench.zui".into()),
        parent_source_node_id: Some("root".into()),
        parent_instance_path: Some("[]".into()),
        control_id: Some("card".into()),
    }
}

fn repeated_owner(node: &str, parent: &str, item: &str, field: &str) -> PaintNodeIdentity {
    PaintNodeIdentity {
        node_id: node.into(),
        parent_source_node_id: Some(parent.into()),
        item_key: Some(item.into()),
        property_field_id: Some(field.into()),
        ..owner()
    }
}

fn record_surface(owner: &std::rc::Rc<PaintNodeIdentity>, color: [u8; 4]) {
    record_owner_commands_complete(owner);
    let mut command = HostPaintCommand::quad(
        FrameRect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 24.0,
        },
        None,
        0,
        Some(color),
        None,
        0.0,
        0.0,
        1.0,
    );
    command.set_owner(owner.clone());
    record_drawn_command(&command);
}

#[test]
fn repeat_items_keep_independent_styles_and_media_under_one_authored_prototype() {
    let repo = std::env::current_dir().unwrap();
    let scope = PaintEvidenceScope::begin(&repo, 1.0).unwrap();
    super::register_media_source("test:repeat-media", &repo.join("Cargo.toml"));
    for (node, item, field, color) in [
        (
            "row-near",
            "near-key",
            "camera.near_clip",
            [10, 20, 30, 255],
        ),
        ("row-far", "far-key", "camera.far_clip", [40, 50, 60, 255]),
    ] {
        let owner = std::rc::Rc::new(repeated_owner(node, "properties", item, field));
        record_surface(&owner, color);
        let json = super::owner_json(Some(&owner));
        assert_eq!(json["itemKey"], item);
        assert_eq!(json["propertyFieldId"], field);
        let _owner = super::PaintNodeOwnerScope::enter(Some(&owner));
        super::record_media_use("test:repeat-media");
    }
    let evidence = scope.finish().unwrap();
    let styles = evidence["styles"].as_array().unwrap();
    assert_eq!(styles.len(), 2);
    for (item, field, color) in [
        ("near-key", "camera.near_clip", "rgba(10,20,30,1)"),
        ("far-key", "camera.far_clip", "rgba(40,50,60,1)"),
    ] {
        let style = styles
            .iter()
            .find(|style| style["itemKey"] == item)
            .unwrap();
        assert_eq!(style["sourcePath"], owner().source_path);
        assert_eq!(style["sourceNodeId"], owner().source_node_id);
        assert_eq!(style["instancePath"], "[]");
        assert_eq!(style["parentSourceNodeId"], "properties");
        assert_eq!(style["propertyFieldId"], field);
        assert_eq!(
            style["effectiveStyle"]["properties"]["backgroundColor"],
            color
        );
        assert_eq!(style["styleInventory"]["complete"], true);
        let media = evidence["assetAudit"]["resources"]
            .as_array()
            .unwrap()
            .iter()
            .find(|resource| resource["itemKey"] == item)
            .unwrap();
        assert_eq!(media["sourceNodeId"], owner().source_node_id);
        assert_eq!(media["parentSourceNodeId"], "properties");
        assert_eq!(media["propertyFieldId"], field);
    }
    assert_eq!(
        evidence["assetAudit"]["resources"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(evidence["assetAudit"]["complete"], true);
    assert_eq!(evidence["issues"], json!([]));
}

#[test]
fn repeat_items_with_the_same_key_under_distinct_authored_owners_do_not_merge() {
    let repo = std::env::current_dir().unwrap();
    let scope = PaintEvidenceScope::begin(&repo, 1.0).unwrap();
    super::register_media_source("test:repeat-owner-media", &repo.join("Cargo.toml"));
    for (index, part) in [
        "parentSourcePath",
        "parentSourceNodeId",
        "parentInstancePath",
    ]
    .into_iter()
    .enumerate()
    {
        let mut owner = repeated_owner(
            &format!("row-{index}"),
            "properties",
            "shared-key",
            "camera.fov",
        );
        match part {
            "parentSourcePath" => owner.parent_source_path = Some("another-panel.zui".into()),
            "parentSourceNodeId" => owner.parent_source_node_id = Some("another-properties".into()),
            _ => {
                owner.parent_instance_path =
                    Some("[{\"sourcePath\":\"panel.zui\",\"sourceNodeId\":\"other\"}]".into())
            }
        }
        let owner = std::rc::Rc::new(owner);
        record_surface(&owner, [10, 20, 30, 255]);
        let _owner = super::PaintNodeOwnerScope::enter(Some(&owner));
        super::record_media_use("test:repeat-owner-media");
    }
    let evidence = scope.finish().unwrap();
    assert_eq!(evidence["styles"].as_array().unwrap().len(), 3);
    assert!(evidence["styles"]
        .as_array()
        .unwrap()
        .iter()
        .all(|style| style["styleInventory"]["complete"] == true));
    assert_eq!(
        evidence["assetAudit"]["resources"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    assert_eq!(evidence["assetAudit"]["complete"], true);
    assert_eq!(evidence["issues"], json!([]));
}

#[test]
fn repeat_item_conflicting_physical_nodes_or_fields_keep_evidence_incomplete() {
    for change_field in [false, true] {
        let repo = std::env::current_dir().unwrap();
        let scope = PaintEvidenceScope::begin(&repo, 1.0).unwrap();
        let first = repeated_owner("row", "properties", "same-key", "camera.near_clip");
        let mut conflict = first.clone();
        if change_field {
            conflict.property_field_id = Some("camera.far_clip".into());
        } else {
            conflict.node_id = "other-row".into();
        }
        record_surface(&std::rc::Rc::new(first), [10, 20, 30, 255]);
        record_surface(&std::rc::Rc::new(conflict), [10, 20, 30, 255]);
        let evidence = scope.finish().unwrap();
        assert_eq!(evidence["styles"].as_array().unwrap().len(), 1);
        assert_eq!(evidence["styles"][0]["propertyFieldId"], "camera.near_clip");
        assert_eq!(evidence["styles"][0]["effectiveStyle"]["complete"], false);
        assert_eq!(evidence["styles"][0]["styleInventory"]["complete"], false);
        assert_eq!(evidence["assetAudit"]["complete"], false);
        assert_eq!(evidence["issues"].as_array().unwrap().len(), 1);
    }
}

#[test]
fn partial_repeat_identity_and_missing_authored_owner_are_not_complete() {
    for missing in [
        "itemKey",
        "propertyFieldId",
        "parentSourcePath",
        "parentSourceNodeId",
        "parentInstancePath",
        "emptyItemKey",
    ] {
        let repo = std::env::current_dir().unwrap();
        let scope = PaintEvidenceScope::begin(&repo, 1.0).unwrap();
        let mut owner = repeated_owner("row", "properties", "real-key", "camera.fov");
        match missing {
            "itemKey" => owner.item_key = None,
            "propertyFieldId" => owner.property_field_id = None,
            "parentSourcePath" => owner.parent_source_path = None,
            "parentSourceNodeId" => owner.parent_source_node_id = None,
            "parentInstancePath" => owner.parent_instance_path = None,
            _ => owner.item_key = Some(String::new()),
        }
        record_surface(&std::rc::Rc::new(owner), [10, 20, 30, 255]);
        let evidence = scope.finish().unwrap();
        assert_eq!(evidence["styles"], json!([]), "{missing}");
        assert_eq!(evidence["assetAudit"]["complete"], false, "{missing}");
        assert!(
            !evidence["issues"].as_array().unwrap().is_empty(),
            "{missing}"
        );
    }
}

#[test]
fn legacy_authored_owner_collision_is_not_disambiguated_by_physical_node() {
    let repo = std::env::current_dir().unwrap();
    let scope = PaintEvidenceScope::begin(&repo, 1.0).unwrap();
    let first = owner();
    let mut other = first.clone();
    other.node_id = "another-physical-row".into();
    record_surface(&std::rc::Rc::new(first), [10, 20, 30, 255]);
    record_surface(&std::rc::Rc::new(other), [10, 20, 30, 255]);
    let evidence = scope.finish().unwrap();
    assert_eq!(evidence["styles"].as_array().unwrap().len(), 1);
    assert_eq!(evidence["styles"][0]["styleInventory"]["complete"], false);
    assert_eq!(evidence["assetAudit"]["complete"], false);
    assert!(!evidence["issues"].as_array().unwrap().is_empty());
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

#[test]
fn media_receipts_exist_only_during_capture_and_reject_source_conflicts() {
    let repo = std::env::current_dir().unwrap();
    let source = repo.join("Cargo.toml");
    super::register_media_source("test:media", &source);
    assert!(super::MEDIA_FILES.with_borrow(|files| files.is_empty()));
    let scope = PaintEvidenceScope::begin(&repo, 1.0).unwrap();
    super::register_media_source("test:media", &source);
    super::register_media_source("test:media", &source);
    assert!(super::MEDIA_FILES.with_borrow(|files| files["test:media"].is_some()));
    super::register_media_source("test:media", &repo.join("missing-source"));
    assert!(super::MEDIA_FILES.with_borrow(|files| files["test:media"].is_none()));
    drop(scope);
    assert!(super::MEDIA_FILES.with_borrow(|files| files.is_empty()));
}
