use super::data::{FrameRect, HostAssetDeletionBlockerData, HostWindowPresentationData};
use super::native_pointer::asset_deletion_blocker_action_at;
use crate::ui::retained_host::primitives::ModelRc;

#[test]
fn blocker_data_keeps_every_referencer_while_bounding_visible_rows() {
    let referencers = (0..32)
        .map(|index| format!("project://assets/referencer-{index:02}.scene"))
        .collect::<Vec<_>>();

    let data = HostAssetDeletionBlockerData::for_window(
        1280.0,
        720.0,
        "project://assets/target.material".to_owned(),
        ModelRc::with_metadata(referencers.clone(), ()),
    );

    assert_eq!(data.referencers.row_count(), referencers.len());
    assert!(data.visible_referencer_rows < referencers.len());
    assert_eq!(data.overflow_label, "23 more referencers");
    assert_eq!(
        data.referencers.iter().cloned().collect::<Vec<_>>(),
        referencers
    );
}

#[test]
fn blocker_layout_remains_inside_a_compact_window() {
    let data = HostAssetDeletionBlockerData::for_window(
        280.0,
        220.0,
        "project://assets/target.material".to_owned(),
        ModelRc::with_metadata(vec!["project://assets/referencer.scene".to_owned()], ()),
    );

    assert!(contains_frame(&data.overlay_frame, &data.dialog_frame));
    assert!(contains_frame(
        &data.dialog_frame,
        &data.referencer_list_frame
    ));
    assert!(contains_frame(&data.dialog_frame, &data.close_button_frame));
}

#[test]
fn blocker_hit_route_only_accepts_the_close_button() {
    let mut presentation = HostWindowPresentationData::default();
    presentation.asset_deletion_blocker = HostAssetDeletionBlockerData::for_window(
        800.0,
        600.0,
        "project://assets/target.material".to_owned(),
        ModelRc::with_metadata(vec!["project://assets/referencer.scene".to_owned()], ()),
    );
    let button = &presentation.asset_deletion_blocker.close_button_frame;

    assert!(asset_deletion_blocker_action_at(
        &presentation,
        button.x + button.width * 0.5,
        button.y + button.height * 0.5,
    ));
    assert!(!asset_deletion_blocker_action_at(
        &presentation,
        presentation.asset_deletion_blocker.dialog_frame.x + 12.0,
        presentation.asset_deletion_blocker.dialog_frame.y + 12.0,
    ));
}

fn contains_frame(outer: &FrameRect, inner: &FrameRect) -> bool {
    inner.x >= outer.x
        && inner.y >= outer.y
        && inner.x + inner.width <= outer.x + outer.width
        && inner.y + inner.height <= outer.y + outer.height
}
