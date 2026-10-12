use super::*;
use crate::ui::workbench::snapshot::SceneEntry;

fn scene_entries(values: &[(NodeId, &str, bool)]) -> SceneEntries {
    let selected = values
        .iter()
        .filter(|(_, _, selected)| *selected)
        .map(|(id, _, _)| *id)
        .collect::<Vec<_>>();
    SceneEntries::from_entries(
        values
            .iter()
            .map(|(id, name, _)| SceneEntry {
                id: *id,
                name: (*name).into(),
                depth: 0,
            })
            .collect::<Vec<_>>(),
        selected,
    )
}

#[test]
fn hierarchy_rename_target_requires_exactly_one_selected_entry() {
    let entries = scene_entries(&[(4, "Camera", true), (7, "Light", false)]);

    assert_eq!(
        single_selected_hierarchy_rename_target(&entries),
        Some((4, "Camera".into()))
    );
    assert_eq!(
        single_selected_hierarchy_rename_target(&scene_entries(&[(4, "Camera", false)])),
        None
    );
    assert_eq!(
        single_selected_hierarchy_rename_target(&scene_entries(&[
            (4, "Camera", true),
            (7, "Light", true),
        ])),
        None
    );
}

#[test]
fn double_click_name_lookup_uses_the_current_entity_projection() {
    let entries = scene_entries(&[(4, "Old", false), (7, "Current", true)]);

    assert_eq!(hierarchy_name_for_entity(&entries, 7), Some("Current"));
    assert_eq!(hierarchy_name_for_entity(&entries, 9), None);
}

#[test]
fn keyboard_rename_uses_the_authoritative_scene_snapshot() {
    let source = include_str!("../hierarchy_rename.rs");
    let production = source.split("#[cfg(test)]").next().unwrap_or(source);

    assert!(production.contains("self.runtime.editor_snapshot().scene_entries"));
    assert!(!production
        .contains("single_selected_hierarchy_rename_target(&self.hierarchy_scene_entries)"));
}

#[test]
fn hierarchy_rename_dispatch_kind_carries_the_exact_node_id() {
    assert_eq!(
        hierarchy_inline_rename_target_id("hierarchy_inline_rename:42"),
        Some("42")
    );
    assert_eq!(hierarchy_inline_rename_target_id("hierarchy"), None);
}

#[test]
fn hierarchy_rename_target_comes_only_from_the_active_inline_focus() {
    let focus = HostTextInputFocusData {
        control_id: HIERARCHY_INLINE_RENAME_CONTROL_ID.into(),
        dispatch_kind: "hierarchy_inline_rename:42".into(),
        ..HostTextInputFocusData::default()
    };

    assert_eq!(hierarchy_rename_target_from_focus(&focus), Some(42));
    assert_eq!(
        hierarchy_rename_target_from_focus(&HostTextInputFocusData {
            control_id: "OtherControl".into(),
            dispatch_kind: "hierarchy_inline_rename:42".into(),
            ..HostTextInputFocusData::default()
        }),
        None
    );
    assert_eq!(
        hierarchy_rename_target_from_focus(&HostTextInputFocusData {
            control_id: HIERARCHY_INLINE_RENAME_CONTROL_ID.into(),
            dispatch_kind: "hierarchy_inline_rename:not-a-node".into(),
            ..HostTextInputFocusData::default()
        }),
        None
    );
}

#[test]
fn hierarchy_rename_does_not_replace_an_existing_text_input_focus() {
    assert!(can_begin_hierarchy_rename_from_focus(
        &HostTextInputFocusData::default()
    ));
    assert!(!can_begin_hierarchy_rename_from_focus(
        &HostTextInputFocusData {
            control_id: "InspectorField".into(),
            ..HostTextInputFocusData::default()
        }
    ));
}

#[test]
fn hierarchy_rename_double_click_requires_the_same_node_within_the_window() {
    let now = Instant::now();
    let previous = HierarchyRenameClick {
        node_id: 4,
        at: now,
    };

    assert!(is_hierarchy_rename_double_click(Some(&previous), 4, now));
    assert!(!is_hierarchy_rename_double_click(Some(&previous), 5, now));
    assert!(!is_hierarchy_rename_double_click(
        Some(&previous),
        4,
        now + HIERARCHY_RENAME_DOUBLE_CLICK_WINDOW + Duration::from_millis(1),
    ));
}
