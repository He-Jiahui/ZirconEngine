use std::rc::Rc;

use super::*;
use crate::ui::retained_host::host_contract::data::{
    TemplateNodeFrameData, TemplateSettingEntryData,
};
use crate::ui::retained_host::primitives::{ModelRc, VecModel};

fn settings_node(row_count: usize) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        component_role: "settings-window".into(),
        popup_open: true,
        frame: TemplateNodeFrameData {
            x: 12.0,
            y: 12.0,
            width: 396.0,
            height: 336.0,
        },
        settings_entries: model(vec![TemplateSettingEntryData::default(); row_count]),
        ..TemplatePaneNodeData::default()
    }
}

#[test]
fn settings_list_scroll_changes_the_retained_offset() {
    let node = settings_node(12);
    let layout = SettingsWindowLayout::new(
        &FrameRect {
            x: node.frame.x,
            y: node.frame.y,
            width: node.frame.width,
            height: node.frame.height,
        },
        current_host_metrics(),
        0.0,
        node.settings_categories.row_count(),
        0.0,
        node.settings_entries.row_count(),
    );
    let target = settings_window_scroll_target(
        &node,
        layout.setting_list.x + 1.0,
        layout.setting_list.y + 1.0,
        layout.setting_row_height,
    );

    assert!(matches!(
        target,
        Some(SettingsWindowScrollTarget::Changed { setting_scroll_offset, .. })
            if (setting_scroll_offset - layout.setting_row_height).abs() <= f32::EPSILON
    ));
}

#[test]
fn category_list_scroll_changes_the_retained_category_offset() {
    let mut node = settings_node(1);
    node.settings_categories = model(vec![Default::default(); 20]);
    let layout = SettingsWindowLayout::new(
        &FrameRect {
            x: node.frame.x,
            y: node.frame.y,
            width: node.frame.width,
            height: node.frame.height,
        },
        current_host_metrics(),
        0.0,
        node.settings_categories.row_count(),
        0.0,
        node.settings_entries.row_count(),
    );
    let target = settings_window_scroll_target(
        &node,
        layout.category_list.x + 1.0,
        layout.category_list.y + 1.0,
        layout.category_row_height,
    );

    assert!(matches!(
        target,
        Some(SettingsWindowScrollTarget::Changed { category_scroll_offset, .. })
            if (category_scroll_offset - layout.category_row_height).abs() <= f32::EPSILON
    ));
}

#[test]
fn settings_window_consumes_scroll_outside_the_list_without_moving_content() {
    let node = settings_node(12);

    assert!(matches!(
        settings_window_scroll_target(&node, node.frame.x + 1.0, node.frame.y + 1.0, 48.0),
        Some(SettingsWindowScrollTarget::Consumed)
    ));
}

fn model<T: Clone + 'static>(values: Vec<T>) -> ModelRc<T> {
    Rc::new(VecModel::from(values)).into()
}
