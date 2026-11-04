use super::*;

#[test]
fn u09_command_palette_keyboard_uses_internal_rows_and_commit_identity() {
    use crate::ui::retained_host::host_contract::data::TemplatePaneOptionData;
    use crate::ui::retained_host::host_contract::template_popup_layout::{
        command_palette_metrics, command_palette_option_frame_within,
    };
    use crate::ui::retained_host::primitives::ModelRc;

    let metrics = command_palette_metrics();
    let mut node = TemplatePaneNodeData::default();
    node.control_id = "WorkbenchCommandPalette".into();
    node.role = "CommandPalette".into();
    node.edit_action_id = "CommandPalette/QueryChanged".into();
    node.commit_action_id = "CommandPalette/Commit".into();
    node.frame.x = 20.25;
    node.frame.y = 80.5;
    node.frame.width = 360.5;
    node.frame.height = metrics.list_top + metrics.row_height * 1.5;
    node.structured_options = ModelRc::with_metadata(
        (0..3)
            .map(|index| TemplatePaneOptionData {
                id: format!("candidate-{index}"),
                label: format!("Candidate {index}"),
                disabled: index == 0,
                selected: index == 1,
                ..Default::default()
            })
            .collect(),
        (),
    );
    let bounds = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 600.0,
    };
    let target =
        option_popup_keyboard_target(&node, &HostPaneInteractionStateData::default(), &bounds)
            .expect("one enabled candidate has visible coverage inside the palette");
    let panel = frame_from_template_node(&node);
    assert_eq!(target.popup_frame, panel);
    assert_eq!(target.rows.len(), 1);
    assert_eq!(target.rows[0].identity.as_str(), "candidate-1");
    assert_eq!(target.rows[0].action_id.as_str(), "CommandPalette/Commit");
    assert_eq!(
        target.rows[0].frame,
        command_palette_option_frame_within(&panel, 1, &bounds).unwrap()
    );
    assert_eq!(target.rows[0].frame.height, metrics.row_height * 0.5);
    assert_eq!(target.current_frame, target.rows[0].frame);
    node.commit_action_id = "".into();
    assert!(
        option_popup_keyboard_target(&node, &HostPaneInteractionStateData::default(), &bounds)
            .is_none()
    );
}

#[test]
fn u09_command_palette_keyboard_refuses_hidden_rows_and_clips_partial_candidates() {
    use crate::ui::retained_host::host_contract::data::{
        TemplateNodeFrameData, TemplatePaneOptionData,
    };
    use crate::ui::retained_host::host_contract::template_popup_layout::command_palette_metrics;
    use crate::ui::retained_host::primitives::ModelRc;

    let metrics = command_palette_metrics();
    let mut node = TemplatePaneNodeData::default();
    node.control_id = "WorkbenchCommandPalette".into();
    node.role = "CommandPalette".into();
    node.commit_action_id = "CommandPalette/Commit".into();
    node.frame = TemplateNodeFrameData {
        x: 20.25,
        y: 80.5,
        width: 360.5,
        height: 240.0,
    };
    node.structured_options = ModelRc::with_metadata(
        vec![TemplatePaneOptionData {
            id: "asset-name-prompt-confirm".into(),
            ..Default::default()
        }],
        (),
    );
    let bounds = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 600.0,
    };
    node.has_clip_frame = true;
    node.clip_frame = node.frame.clone();
    node.clip_frame.height = metrics.list_top - 1.0;
    assert!(
        option_popup_keyboard_target(&node, &HostPaneInteractionStateData::default(), &bounds)
            .is_none()
    );
    node.clip_frame.height = metrics.list_top + metrics.row_height * 0.5;
    let target =
        option_popup_keyboard_target(&node, &HostPaneInteractionStateData::default(), &bounds)
            .expect("a partially painted candidate keeps its visible keyboard frame");
    assert_eq!(target.rows.len(), 1);
    assert_eq!(target.rows[0].frame.height, metrics.row_height * 0.5);
    assert_eq!(
        target.rows[0].frame.y + target.rows[0].frame.height,
        node.clip_frame.y + node.clip_frame.height
    );
}

#[test]
fn asset_dropdown_keeps_its_dispatch_source_for_keyboard_selection() {
    let mut node = TemplatePaneNodeData::default();
    node.dispatch_kind = "asset:browser".into();

    assert_eq!(option_popup_dispatch_kind(&node), "asset:browser");
}

#[test]
fn ordinary_dropdown_keeps_the_workbench_option_dispatch() {
    let node = TemplatePaneNodeData::default();

    assert_eq!(option_popup_dispatch_kind(&node), "workbench_option");
}

#[test]
fn option_keyboard_projection_preserves_source_identity_flags_and_clipped_rows() {
    use crate::ui::retained_host::host_contract::data::TemplatePaneOptionData;
    use crate::ui::retained_host::primitives::ModelRc;

    let mut node = TemplatePaneNodeData::default();
    node.control_id = "AssetTypeDropdown".into();
    node.role = "DropdownPopup".into();
    node.action_id = "fallback.action".into();
    node.edit_action_id = "apply.exact.option".into();
    node.dispatch_kind = "asset:browser".into();
    node.frame.x = 10.0;
    node.frame.y = 20.0;
    node.frame.width = 240.0;
    node.frame.height = 120.0;
    node.structured_options = ModelRc::with_metadata(
        (0..4)
            .map(|index| TemplatePaneOptionData {
                id: format!("uuid-{index}"),
                label: if index == 1 {
                    String::new()
                } else {
                    format!("Option {index}")
                },
                description: "presentation-only description".repeat(64),
                tone: "presentation-tone".into(),
                disabled: index == 0 || index == 3,
                focused: index == 1,
                special: index == 2,
                ..Default::default()
            })
            .collect(),
        (),
    );
    let interaction = HostPaneInteractionStateData {
        hovered_template_value_text: "uuid-2".into(),
        ..Default::default()
    };
    let bounds = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 400.0,
        height: 400.0,
    };
    let target = option_popup_keyboard_target(&node, &interaction, &bounds).unwrap();
    assert_eq!(target.control_id.as_str(), "AssetTypeDropdown");
    assert_eq!(target.dispatch_kind.as_str(), "asset:browser");
    assert_eq!(target.rows.len(), 2);
    assert_eq!(target.rows[0].identity.as_str(), "uuid-1");
    assert_eq!(target.rows[0].search_text.as_str(), "uuid-1");
    assert!(target.rows[0].focused);
    assert_eq!(target.rows[1].identity.as_str(), "uuid-2");
    assert_eq!(target.rows[1].search_text.as_str(), "Option 2");
    assert!(target.rows[1].selected);
    assert!(target
        .rows
        .iter()
        .all(|row| row.action_id.as_str() == "apply.exact.option"));
    assert_eq!(target.current_index, 1);
    assert_eq!(
        target.current_row.as_ref().unwrap().value_text.as_str(),
        "uuid-2"
    );
    assert_eq!(
        target.rows[0].frame,
        FrameRect {
            x: 10.0,
            y: 50.0,
            width: 240.0,
            height: 30.0
        }
    );
    assert_eq!(
        target.current_frame,
        FrameRect {
            x: 10.0,
            y: 80.0,
            width: 240.0,
            height: 30.0
        }
    );
    assert_eq!(node.structured_options.get(1).unwrap().label.as_str(), "");
    assert!(!node
        .structured_options
        .get(2)
        .unwrap()
        .description
        .is_empty());

    node.role = "ComboBox".into();
    node.frame.y = 10.0;
    node.frame.height = 24.0;
    let clipped_bounds = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 400.0,
        height: 90.0,
    };
    let clipped = option_popup_keyboard_target(&node, &interaction, &clipped_bounds).unwrap();
    assert_eq!(clipped.rows.len(), 1);
    assert_eq!(clipped.rows[0].identity.as_str(), "uuid-1");
    assert!(clipped.rows[0].frame.y + clipped.rows[0].frame.height <= clipped_bounds.height);
}

#[test]
fn menu_keyboard_projection_preserves_original_row_geometry_and_exact_action() {
    use super::super::menu::menu_popup_keyboard_target;
    use crate::ui::retained_host::host_contract::data::TemplatePaneMenuItemData;
    use crate::ui::retained_host::primitives::ModelRc;

    let mut node = TemplatePaneNodeData::default();
    node.control_id = "ExactMenu".into();
    node.frame.x = 10.0;
    node.frame.y = 20.0;
    node.frame.width = 240.0;
    node.frame.height = 150.0;
    node.structured_menu_items = ModelRc::with_metadata(
        (0..5)
            .map(|index| TemplatePaneMenuItemData {
                raw: "unused presentation menu payload".repeat(64),
                action_id: if index == 3 {
                    String::new()
                } else {
                    format!("exact.action.{index}")
                },
                label: format!("Menu {index}"),
                shortcut: "unused shortcut presentation".repeat(16),
                disabled: index == 1,
                separator: index == 2,
                focused: index == 0,
                pressed: index == 4,
                checked: index == 4,
                ..Default::default()
            })
            .collect(),
        (),
    );
    let interaction = HostPaneInteractionStateData {
        hovered_template_action_id: "exact.action.4".into(),
        ..Default::default()
    };
    let target = menu_popup_keyboard_target(&node, &interaction).unwrap();
    assert_eq!(target.control_id.as_str(), "ExactMenu");
    assert_eq!(target.dispatch_kind.as_str(), "workbench_menu_item");
    assert_eq!(target.rows.len(), 2);
    assert_eq!(target.rows[0].identity.as_str(), "exact.action.0");
    assert_eq!(target.rows[0].search_text.as_str(), "Menu 0");
    assert!(target.rows[0].focused);
    assert_eq!(target.current_index, 1);
    let current = target.current_row.as_ref().unwrap();
    assert_eq!(current.action_id.as_str(), "exact.action.4");
    assert_eq!(current.value_text.as_str(), "Menu 4");
    assert!(current.focused && current.selected);
    assert_eq!(
        target.current_frame,
        FrameRect {
            x: 10.0,
            y: 140.0,
            width: 240.0,
            height: 30.0
        }
    );
    assert!(!node.structured_menu_items.get(4).unwrap().raw.is_empty());
    assert!(!node
        .structured_menu_items
        .get(4)
        .unwrap()
        .shortcut
        .is_empty());
}
