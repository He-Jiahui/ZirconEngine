use zircon_runtime_interface::ui::{
    dispatch::{UiPointerId, UiPointerSource},
    event_ui::UiNodeId,
    layout::UiPoint,
    surface::UiPointerButton,
};

use super::UiActivePointerTable;

#[test]
fn active_pointer_table_keeps_hover_press_and_capture_per_pointer() {
    let first_pointer = UiPointerId::new(1);
    let second_pointer = UiPointerId::new(2);
    let mut table = UiActivePointerTable::default();

    table.upsert(first_pointer, UiPointerSource::Touch, true);
    table.record_point(first_pointer, UiPoint::new(10.0, 12.0));
    table.set_hovered_path(first_pointer, vec![UiNodeId::new(3), UiNodeId::new(1)]);
    table.press_button(
        first_pointer,
        Some(UiPointerButton::Primary),
        Some(UiNodeId::new(3)),
    );
    table.set_capture_target(first_pointer, Some(UiNodeId::new(3)));

    table.upsert(second_pointer, UiPointerSource::Touch, false);
    table.record_point(second_pointer, UiPoint::new(80.0, 18.0));
    table.set_hovered_path(second_pointer, vec![UiNodeId::new(4), UiNodeId::new(1)]);
    table.press_button(
        second_pointer,
        Some(UiPointerButton::Primary),
        Some(UiNodeId::new(4)),
    );
    table.set_capture_target(second_pointer, Some(UiNodeId::new(4)));

    let first = table.entry(first_pointer).unwrap();
    assert_eq!(first.last_point, Some(UiPoint::new(10.0, 12.0)));
    assert_eq!(first.hovered, vec![UiNodeId::new(3), UiNodeId::new(1)]);
    assert_eq!(first.pressed_buttons, 0b001);
    assert_eq!(first.pressed_target, Some(UiNodeId::new(3)));
    assert_eq!(first.capture_target, Some(UiNodeId::new(3)));
    assert!(first.is_primary);

    let second = table.entry(second_pointer).unwrap();
    assert_eq!(second.last_point, Some(UiPoint::new(80.0, 18.0)));
    assert_eq!(second.hovered, vec![UiNodeId::new(4), UiNodeId::new(1)]);
    assert_eq!(second.pressed_buttons, 0b001);
    assert_eq!(second.pressed_target, Some(UiNodeId::new(4)));
    assert_eq!(second.capture_target, Some(UiNodeId::new(4)));
    assert!(!second.is_primary);
}

#[test]
fn active_pointer_table_release_clears_only_matching_pointer_button_state() {
    let first_pointer = UiPointerId::new(1);
    let second_pointer = UiPointerId::new(2);
    let mut table = UiActivePointerTable::default();

    table.upsert(first_pointer, UiPointerSource::Mouse, true);
    table.upsert(second_pointer, UiPointerSource::Mouse, false);
    table.press_button(
        first_pointer,
        Some(UiPointerButton::Primary),
        Some(UiNodeId::new(3)),
    );
    table.press_button(
        second_pointer,
        Some(UiPointerButton::Primary),
        Some(UiNodeId::new(4)),
    );

    table.release_button(first_pointer, Some(UiPointerButton::Primary));

    assert_eq!(table.entry(first_pointer).unwrap().pressed_buttons, 0);
    assert_eq!(table.entry(first_pointer).unwrap().pressed_target, None);
    assert_eq!(table.entry(second_pointer).unwrap().pressed_buttons, 0b001);
    assert_eq!(
        table.entry(second_pointer).unwrap().pressed_target,
        Some(UiNodeId::new(4))
    );
}

#[test]
fn active_pointer_table_tracks_primary_touch_and_pen_membership_through_lifecycle() {
    let touch = UiPointerId::new(1);
    let pen = UiPointerId::new(2);
    let second_touch = UiPointerId::new(3);
    let mut table = UiActivePointerTable::default();

    assert_primary_membership_matches_legacy_scan(&table);
    assert!(!table.has_primary_for_source(UiPointerSource::Touch));
    table.upsert(touch, UiPointerSource::Touch, true);
    assert_primary_membership_matches_legacy_scan(&table);
    assert!(table.has_primary_for_source(UiPointerSource::Touch));

    table.upsert(second_touch, UiPointerSource::Touch, true);
    assert_primary_membership_matches_legacy_scan(&table);
    table.upsert(touch, UiPointerSource::Touch, false);
    assert_primary_membership_matches_legacy_scan(&table);
    assert!(table.has_primary_for_source(UiPointerSource::Touch));
    table.upsert(second_touch, UiPointerSource::Touch, false);
    assert_primary_membership_matches_legacy_scan(&table);
    assert!(!table.has_primary_for_source(UiPointerSource::Touch));

    table.upsert(touch, UiPointerSource::Touch, false);
    assert_primary_membership_matches_legacy_scan(&table);
    assert!(!table.has_primary_for_source(UiPointerSource::Touch));

    table.upsert(touch, UiPointerSource::Pen, true);
    assert_primary_membership_matches_legacy_scan(&table);
    assert!(!table.has_primary_for_source(UiPointerSource::Touch));
    assert!(table.has_primary_for_source(UiPointerSource::Pen));

    table.upsert(pen, UiPointerSource::Touch, true);
    assert_primary_membership_matches_legacy_scan(&table);
    assert!(table.has_primary_for_source(UiPointerSource::Touch));
    table.remove(pen);
    assert_primary_membership_matches_legacy_scan(&table);
    assert!(!table.has_primary_for_source(UiPointerSource::Touch));

    table.clear();
    assert_primary_membership_matches_legacy_scan(&table);
    assert!(!table.has_primary_for_source(UiPointerSource::Pen));
}

fn assert_primary_membership_matches_legacy_scan(table: &UiActivePointerTable) {
    for source in [
        UiPointerSource::Touch,
        UiPointerSource::Pen,
        UiPointerSource::Mouse,
        UiPointerSource::Unknown,
    ] {
        let legacy = source.is_touch_like()
            && table
                .entries()
                .iter()
                .any(|entry| entry.source == source && entry.is_primary);
        assert_eq!(table.has_primary_for_source(source), legacy, "{source:?}");
    }
}
