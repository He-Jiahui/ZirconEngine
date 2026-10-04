use std::rc::Rc;

use super::*;
use crate::ui::retained_host::primitives::{ModelRc, VecModel};

fn model<T: Clone + 'static>(values: Vec<T>) -> ModelRc<T> {
    Rc::new(VecModel::from(values)).into()
}

#[test]
fn settings_window_preserves_fractional_post_dpi_surface_geometry() {
    let panel = FrameRect {
        x: 12.25,
        y: 16.5,
        width: 396.75,
        height: 336.25,
    };
    let node = TemplatePaneNodeData {
        component_role: "settings-window".into(),
        popup_open: true,
        ..TemplatePaneNodeData::default()
    };
    let mut commands = Vec::new();

    assert!(push_settings_window_commands(
        &mut commands,
        &node,
        &panel,
        &panel,
        None,
        0,
        1.0,
    ));

    assert_eq!(commands.first().map(|command| &command.frame), Some(&panel));
}

#[test]
fn scrolled_setting_commands_are_clipped_to_the_setting_list() {
    let metrics = current_host_metrics();
    let panel = FrameRect {
        x: 12.0,
        y: 12.0,
        width: 396.0,
        height: 336.0,
    };
    let initial_layout = SettingsWindowLayout::new(&panel, metrics, 0.0, 0, 0.0, 12);
    let node = TemplatePaneNodeData {
        component_role: "settings-window".into(),
        popup_open: true,
        settings_scroll_offset: initial_layout.setting_row_height * 0.5,
        settings_entries: model(vec![
            TemplateSettingEntryData {
                label: "First setting".into(),
                schema: "bool".into(),
                value_text: "true".into(),
                ..TemplateSettingEntryData::default()
            };
            12
        ]),
        ..TemplatePaneNodeData::default()
    };
    let layout = SettingsWindowLayout::new(
        &panel,
        metrics,
        0.0,
        0,
        node.settings_scroll_offset,
        node.settings_entries.row_count(),
    );
    let mut commands = Vec::new();

    assert!(push_settings_window_commands(
        &mut commands,
        &node,
        &panel,
        &panel,
        None,
        0,
        1.0,
    ));
    let first_label = commands
        .iter()
        .find(|command| command.text.as_deref() == Some("First setting"))
        .expect("the partially scrolled first setting must still be painted");

    assert_eq!(first_label.clip_frame.as_ref(), Some(&layout.setting_list));
}
