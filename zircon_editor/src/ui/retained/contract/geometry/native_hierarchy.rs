use crate::ui::retained_host::app::hierarchy_rename::{
    hierarchy_inline_rename_target_id, HIERARCHY_INLINE_RENAME_CONTROL_ID,
};
use crate::ui::retained_host::hierarchy_pointer::current_hierarchy_row_metrics;
use crate::ui::retained_host::host_contract::data::{
    FrameRect, HostPaneInteractionStateData, HostTextInputFocusData, HostWindowPresentationData,
    PaneData,
};
use crate::ui::retained_host::host_contract::paint_workbench_renderer::{
    hierarchy_row_frame, hierarchy_viewport_frame,
};

use super::super::schema::{
    UiProfileFrame, UiProfileNativeHierarchy, UiProfileNativeHierarchyRename,
    UiProfileNativeHierarchyRow, UiProfileNativePopupFocus,
};
use super::frame_math::{intersect_frames, translated};
use super::pane_frames::{floating_window_content_frame, side_dock_content_frame};

pub(in crate::ui::retained_host::host_contract) fn collect_native_hierarchy_profiles(
    presentation: &HostWindowPresentationData,
) -> Vec<UiProfileNativeHierarchy> {
    let scene = &presentation.host_scene_data;
    let interaction = &presentation.pane_interaction_state;
    let focus = &presentation.text_input_focus;
    let popup_focus = UiProfileNativePopupFocus {
        menu_open: presentation.menu_state.open_menu_index >= 0,
        page_overflow_open: presentation.host_page_overflow_menu_state.open,
        dock_overflow_open: presentation.host_dock_overflow_menu_state.open,
    };
    let mut profiles = Vec::new();
    if let Some(authored) =
        crate::ui::retained_host::host_contract::componentized_workbench_regions::authored_hierarchy(
            presentation,
        )
    {
        collect_native_hierarchy_profile_with_geometry(
            "left",
            authored.pane,
            authored.viewport,
            authored.metrics,
            interaction,
            focus,
            &popup_focus,
            &mut profiles,
        );
    } else if !crate::ui::retained_host::host_contract::componentized_workbench_regions::owns_ordinary_panes(presentation) {
        collect_native_hierarchy_profile(
            "left",
            &scene.left_dock.pane,
            &side_dock_content_frame(&scene.left_dock),
            interaction,
            focus,
            &popup_focus,
            &mut profiles,
        );
    }
    if !crate::ui::retained_host::host_contract::componentized_workbench_regions::owns_ordinary_panes(presentation) {
    collect_native_hierarchy_profile(
        "right",
        &scene.right_dock.pane,
        &side_dock_content_frame(&scene.right_dock),
        interaction,
        focus,
        &popup_focus,
        &mut profiles,
    );
    }
    collect_native_hierarchy_profile(
        "document",
        &scene.document_dock.pane,
        &translated(
            &scene.document_dock.content_frame,
            scene.document_dock.region_frame.x,
            scene.document_dock.region_frame.y,
        ),
        interaction,
        focus,
        &popup_focus,
        &mut profiles,
    );
    collect_native_hierarchy_profile(
        "bottom",
        &scene.bottom_dock.pane,
        &translated(
            &scene.bottom_dock.content_frame,
            scene.bottom_dock.region_frame.x,
            scene.bottom_dock.region_frame.y,
        ),
        interaction,
        focus,
        &popup_focus,
        &mut profiles,
    );
    for row in 0..scene.floating_layer.floating_windows.row_count() {
        let Some(window) = scene.floating_layer.floating_windows.row_data(row) else {
            continue;
        };
        collect_native_hierarchy_profile(
            window.window_id.as_str(),
            &window.active_pane,
            &floating_window_content_frame(&window.frame, &window.header_frame),
            interaction,
            focus,
            &popup_focus,
            &mut profiles,
        );
    }
    profiles
}

fn collect_native_hierarchy_profile(
    surface: &str,
    pane: &PaneData,
    body: &FrameRect,
    interaction: &HostPaneInteractionStateData,
    focus: &HostTextInputFocusData,
    popup_focus: &UiProfileNativePopupFocus,
    output: &mut Vec<UiProfileNativeHierarchy>,
) {
    if pane.kind.as_str() != "Hierarchy" {
        return;
    }
    let viewport = hierarchy_viewport_frame(pane, body);
    let metrics = current_hierarchy_row_metrics();
    collect_native_hierarchy_profile_with_geometry(
        surface,
        pane,
        viewport,
        metrics,
        interaction,
        focus,
        popup_focus,
        output,
    );
}

fn collect_native_hierarchy_profile_with_geometry(
    surface: &str,
    pane: &PaneData,
    viewport: FrameRect,
    metrics: crate::ui::retained_host::hierarchy_pointer::HierarchyRowMetrics,
    interaction: &HostPaneInteractionStateData,
    focus: &HostTextInputFocusData,
    popup_focus: &UiProfileNativePopupFocus,
    output: &mut Vec<UiProfileNativeHierarchy>,
) {
    let scroll_offset = interaction.hierarchy_scroll_px.max(0.0);
    let rows = pane
        .hierarchy
        .hierarchy_nodes
        .iter()
        .enumerate()
        .map(|(index, node)| {
            let frame = intersect_frames(
                &hierarchy_row_frame(&viewport, index, scroll_offset, metrics),
                &viewport,
            )
            .map(UiProfileFrame::from);
            UiProfileNativeHierarchyRow {
                index,
                node_id: node.id.to_string(),
                name: node.name.to_string(),
                depth: node.depth,
                selected: node.selected,
                frame,
            }
        })
        .collect();
    let selected_node_ids = pane
        .hierarchy
        .hierarchy_nodes
        .iter()
        .filter(|node| node.selected)
        .map(|node| node.id.to_string())
        .collect::<Vec<_>>();
    let selected_names = pane
        .hierarchy
        .hierarchy_nodes
        .iter()
        .filter(|node| node.selected)
        .map(|node| node.name.to_string())
        .collect::<Vec<_>>();
    let inline_rename = (focus.control_id.as_str() == HIERARCHY_INLINE_RENAME_CONTROL_ID)
        .then(|| hierarchy_inline_rename_target_id(focus.dispatch_kind.as_str()))
        .flatten()
        .map(|node_id| UiProfileNativeHierarchyRename {
            control_id: focus.control_id.to_string(),
            node_id: node_id.to_owned(),
            value_text: focus.value_text.to_string(),
            edit_frame: focus.edit_frame.clone().into(),
        });
    output.push(UiProfileNativeHierarchy {
        surface: surface.to_owned(),
        viewport_frame: viewport.into(),
        scroll_offset,
        rows,
        selected_node_ids,
        selected_names,
        focused_control_id: (!focus.control_id.is_empty()).then(|| focus.control_id.to_string()),
        popup_focus: UiProfileNativePopupFocus {
            menu_open: popup_focus.menu_open,
            page_overflow_open: popup_focus.page_overflow_open,
            dock_overflow_open: popup_focus.dock_overflow_open,
        },
        inline_rename,
    });
}

#[cfg(test)]
#[path = "tests/native_hierarchy.rs"]
mod tests;
