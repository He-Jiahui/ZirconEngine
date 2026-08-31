use super::super::super::frame_geometry::union_optional_frames;
use super::super::super::globals::{PaneSurfaceHostContext, UiHostContext};
use super::super::super::redraw::NativePointerDispatchResult;
use super::super::super::window::UiHostWindow;
use super::super::target::{
    PopupKeyboardRow, PopupKeyboardTarget, PopupKeyboardWindowFocus, PopupKeyboardWindowRequest,
    HOST_DOCK_OVERFLOW_DISPATCH_KIND, HOST_PAGE_OVERFLOW_DISPATCH_KIND,
};
use crate::ui::retained_host::asset_control_ids::{
    asset_dispatch_source, asset_surface_binding_control_id,
};
use crate::ui::retained_host::callback_dispatch::{
    WORKBENCH_COMMAND_PALETTE_COMMIT_BINDING_ID, WORKBENCH_COMMAND_PALETTE_CONTROL_ID,
};
use crate::ui::retained_host::host_contract::data::{
    HostDockOverflowMenuStateData, HostPageOverflowMenuStateData,
};
use crate::ui::retained_host::host_contract::host_dock_overflow_menu::{
    host_dock_overflow_projection, host_dock_overflow_scroll_offset_for_tab,
};
use crate::ui::retained_host::host_contract::host_page_overflow_menu::host_page_overflow_scroll_offset_for_page_with_state;
use crate::ui::retained_host::workbench_popup_actions::WORKBENCH_POPUP_CANCEL_ACTION_ID;

pub(super) fn dispatch_popup_accept(
    ui: &UiHostWindow,
    target: PopupKeyboardTarget,
) -> NativePointerDispatchResult {
    let Some(row) = target.current_row.clone() else {
        return NativePointerDispatchResult::idle();
    };
    let popup_frame = target.popup_frame.clone();
    if target.dispatch_kind.as_str() == HOST_DOCK_OVERFLOW_DISPATCH_KIND {
        let Some(tab_index) = row.source_index else {
            return NativePointerDispatchResult::idle();
        };
        let generation = ui.get_host_presentation_generation();
        let Some(projection) = host_dock_overflow_projection(
            generation.structure(),
            generation.dock_overflow_menu_state(),
        ) else {
            return NativePointerDispatchResult::idle();
        };
        let surface_key = projection.surface_key.into();
        let drawer = projection.drawer;
        let host = ui.global::<UiHostContext>();
        host.set_host_dock_overflow_menu_state(HostDockOverflowMenuStateData::default());
        if drawer {
            host.invoke_drawer_header_pointer_clicked(
                surface_key,
                tab_index as i32,
                0.0,
                0.0,
                0.0,
                0.0,
            );
        } else {
            host.invoke_document_tab_pointer_clicked(
                surface_key,
                tab_index as i32,
                0.0,
                0.0,
                0.0,
                0.0,
            );
        }
        return NativePointerDispatchResult::region_with_frame_update(popup_frame);
    }
    if target.dispatch_kind.as_str() == HOST_PAGE_OVERFLOW_DISPATCH_KIND {
        let Some(page_index) = row.source_index else {
            return NativePointerDispatchResult::idle();
        };
        let host = ui.global::<UiHostContext>();
        host.set_host_page_overflow_menu_state(HostPageOverflowMenuStateData::default());
        host.invoke_host_page_pointer_clicked(page_index as i32, false);
        return NativePointerDispatchResult::region_with_frame_update(popup_frame);
    }
    let pane_host = ui.global::<PaneSurfaceHostContext>();
    if let Some(source) = asset_dispatch_source(target.dispatch_kind.as_str()) {
        let control_id = asset_surface_binding_control_id(row.action_id.as_str())
            .unwrap_or(row.action_id.as_str());
        pane_host.invoke_asset_control_changed(source.into(), control_id.into(), row.value_text);
        return NativePointerDispatchResult::region_with_frame_update(popup_frame);
    }
    match target.dispatch_kind.as_str() {
        "workbench_option"
            if target.control_id.as_str() == WORKBENCH_COMMAND_PALETTE_CONTROL_ID =>
        {
            pane_host.invoke_surface_control_edited(
                target.control_id,
                WORKBENCH_COMMAND_PALETTE_COMMIT_BINDING_ID.into(),
                row.value_text,
            )
        }
        "workbench_option" => pane_host.invoke_component_showcase_option_selected(
            target.control_id,
            row.action_id,
            row.value_text,
        ),
        "workbench_menu_item" => {
            pane_host.invoke_surface_control_clicked(target.control_id, row.action_id)
        }
        _ => return NativePointerDispatchResult::idle(),
    }
    NativePointerDispatchResult::region_with_frame_update(popup_frame)
}

pub(super) fn dispatch_popup_cancel(
    ui: &UiHostWindow,
    target: PopupKeyboardTarget,
) -> NativePointerDispatchResult {
    let popup_frame = target.popup_frame.clone();
    if target.dispatch_kind.as_str() == HOST_DOCK_OVERFLOW_DISPATCH_KIND {
        ui.global::<UiHostContext>()
            .set_host_dock_overflow_menu_state(HostDockOverflowMenuStateData::default());
        return NativePointerDispatchResult::region_with_frame_update(popup_frame);
    }
    if target.dispatch_kind.as_str() == HOST_PAGE_OVERFLOW_DISPATCH_KIND {
        ui.global::<UiHostContext>()
            .set_host_page_overflow_menu_state(HostPageOverflowMenuStateData::default());
        return NativePointerDispatchResult::region_with_frame_update(popup_frame);
    }
    let pane_host = ui.global::<PaneSurfaceHostContext>();
    pane_host
        .invoke_surface_control_clicked(target.control_id, WORKBENCH_POPUP_CANCEL_ACTION_ID.into());
    ui.clear_hovered_template_node_for_pointer_move();
    NativePointerDispatchResult::region_with_frame_update(popup_frame)
}

pub(super) fn dispatch_popup_hover_row(
    ui: &UiHostWindow,
    target: PopupKeyboardTarget,
    next: PopupKeyboardRow,
) -> NativePointerDispatchResult {
    if target.dispatch_kind.as_str() == HOST_DOCK_OVERFLOW_DISPATCH_KIND {
        let Some(tab_index) = next.source_index else {
            return NativePointerDispatchResult::idle();
        };
        let generation = ui.get_host_presentation_generation();
        let state = generation.dock_overflow_menu_state();
        let scroll_offset = host_dock_overflow_scroll_offset_for_tab(
            generation.structure(),
            &target.popup_frame,
            state,
            tab_index,
        );
        ui.global::<UiHostContext>()
            .set_host_dock_overflow_menu_state(HostDockOverflowMenuStateData {
                open: true,
                surface_key: state.surface_key.clone(),
                hovered_tab_index: tab_index as i32,
                scroll_offset,
            });
        return NativePointerDispatchResult::region(target.popup_frame);
    }
    if target.dispatch_kind.as_str() == HOST_PAGE_OVERFLOW_DISPATCH_KIND {
        let Some(page_index) = next.source_index else {
            return NativePointerDispatchResult::idle();
        };
        let presentation = ui.get_host_presentation_generation();
        let scroll_offset = host_page_overflow_scroll_offset_for_page_with_state(
            presentation.structure(),
            &target.popup_frame,
            page_index,
            presentation.page_overflow_menu_state(),
        );
        ui.global::<UiHostContext>()
            .set_host_page_overflow_menu_state(HostPageOverflowMenuStateData {
                open: true,
                hovered_page_index: page_index as i32,
                scroll_offset,
            });
        return NativePointerDispatchResult::region(target.popup_frame);
    }
    ui.set_hovered_template_row_for_pointer_move(
        &target.control_id,
        &target.dispatch_kind,
        &next.action_id,
        &next.value_text,
        &next.frame,
    );
    NativePointerDispatchResult::region(
        union_optional_frames(Some(target.current_frame), Some(next.frame)).unwrap_or_default(),
    )
}

pub(super) fn dispatch_popup_window_request(
    ui: &UiHostWindow,
    target: PopupKeyboardTarget,
    request: PopupKeyboardWindowRequest,
) -> NativePointerDispatchResult {
    if target.control_id.as_str() != WORKBENCH_COMMAND_PALETTE_CONTROL_ID {
        return NativePointerDispatchResult::idle();
    }
    let focus = match request.focus {
        PopupKeyboardWindowFocus::First => "first",
        PopupKeyboardWindowFocus::Last => "last",
    };
    ui.global::<PaneSurfaceHostContext>()
        .invoke_surface_control_edited(
            target.control_id,
            "CommandPalette/WindowRequested".into(),
            format!(
                "{}|{}|{focus}|{}",
                request.current_offset,
                request.target_offset,
                request.query.as_str()
            )
            .into(),
        );
    NativePointerDispatchResult::region_with_frame_update(target.popup_frame)
}
