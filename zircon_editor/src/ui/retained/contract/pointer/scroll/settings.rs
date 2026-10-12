use crate::ui::retained_host::host_contract::data::{
    FrameRect, HostWindowPresentationData, TemplatePaneNodeData,
};
use crate::ui::retained_host::host_contract::frame_geometry::contains_point;
use crate::ui::retained_host::host_contract::globals::UiHostContext;
use crate::ui::retained_host::host_contract::paint_theme::current_host_metrics;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::settings_window_geometry::SettingsWindowLayout;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

enum SettingsWindowScrollTarget {
    Consumed,
    Changed {
        category_scroll_offset: f32,
        setting_scroll_offset: f32,
        damage: FrameRect,
    },
}

pub(super) fn dispatch_settings_window_scroll(
    ui: &UiHostWindow,
    presentation: &HostWindowPresentationData,
    x: f32,
    y: f32,
    delta: f32,
) -> Option<NativePointerDispatchResult> {
    let target = presentation
        .workbench_window_nodes
        .iter()
        .find_map(|node| settings_window_scroll_target(node, x, y, delta))?;
    match target {
        SettingsWindowScrollTarget::Consumed => Some(NativePointerDispatchResult::idle()),
        SettingsWindowScrollTarget::Changed {
            category_scroll_offset,
            setting_scroll_offset,
            damage,
        } => {
            ui.global::<UiHostContext>()
                .invoke_settings_window_scrolled(category_scroll_offset, setting_scroll_offset);
            Some(NativePointerDispatchResult::region(damage))
        }
    }
}

fn settings_window_scroll_target(
    node: &TemplatePaneNodeData,
    x: f32,
    y: f32,
    delta: f32,
) -> Option<SettingsWindowScrollTarget> {
    if node.component_role.as_str() != "settings-window" || !node.popup_open {
        return None;
    }
    let frame = FrameRect {
        x: node.frame.x,
        y: node.frame.y,
        width: node.frame.width,
        height: node.frame.height,
    };
    if !contains_point(&frame, x, y) {
        return None;
    }

    let category_row_count = node.settings_categories.row_count();
    let setting_row_count = node.settings_entries.row_count();
    let layout = SettingsWindowLayout::new(
        &frame,
        current_host_metrics(),
        node.settings_category_scroll_offset,
        category_row_count,
        node.settings_scroll_offset,
        setting_row_count,
    );
    if contains_point(&layout.category_list, x, y) {
        let category_scroll_offset = layout.category_scroll_offset_for_delta(delta);
        if (category_scroll_offset - layout.category_scroll_offset()).abs() <= f32::EPSILON {
            return Some(SettingsWindowScrollTarget::Consumed);
        }
        return Some(SettingsWindowScrollTarget::Changed {
            category_scroll_offset,
            setting_scroll_offset: layout.setting_scroll_offset(),
            damage: frame,
        });
    }
    if !contains_point(&layout.setting_list, x, y) || !node.settings_editor_open_kind.is_empty() {
        return Some(SettingsWindowScrollTarget::Consumed);
    }
    let setting_scroll_offset = layout.setting_scroll_offset_for_delta(delta);
    if (setting_scroll_offset - layout.setting_scroll_offset()).abs() <= f32::EPSILON {
        return Some(SettingsWindowScrollTarget::Consumed);
    }
    Some(SettingsWindowScrollTarget::Changed {
        category_scroll_offset: layout.category_scroll_offset(),
        setting_scroll_offset,
        damage: frame,
    })
}

#[cfg(test)]
#[path = "tests/settings.rs"]
mod tests;
