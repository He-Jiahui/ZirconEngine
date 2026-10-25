use super::super::super::super::*;
use crate::ui::asset_editor::UiAssetCanvasAnchorPresetTicket;
use crate::ui::retained_host::host_contract::{
    CanvasAnchorPopupEvent, CanvasAnchorPopupId, CanvasAnchorPopupKind, CanvasAnchorPopupOpen,
};
use crate::ui::workbench::view::ViewInstanceId;

pub(in crate::ui::retained_host::app) struct PendingCanvasAnchorPopup {
    id: CanvasAnchorPopupId,
    source_ui: UiHostWindow,
    instance_id: ViewInstanceId,
    ticket: UiAssetCanvasAnchorPresetTicket,
}

impl RetainedEditorHost {
    fn canvas_popup_source_is_registered(&self, source_ui: &UiHostWindow) -> bool {
        match resolve_callback_source_window_id(source_ui) {
            Some(window_id) => self
                .native_window_presenters
                .window(&window_id)
                .is_some_and(|registered| registered.same_native_window(source_ui)),
            None => self.ui.same_native_window(source_ui),
        }
    }

    pub(in crate::ui::retained_host::app) fn open_canvas_anchor_popup_from_native_action(
        &mut self,
        source_ui: &UiHostWindow,
        open: CanvasAnchorPopupOpen,
    ) {
        if !self.canvas_popup_source_is_registered(source_ui) {
            return;
        }
        // Validate the entire captured opener before retiring another popup or
        // beginning a ticket. The same inner frame can survive an outer reflow.
        let Some(address) = open.into_current_address(source_ui) else {
            return;
        };
        let instance_id = address.instance_id.as_str();
        // Retire the preceding actual popup, never a generic dismiss or a new Session.
        if let Some(previous) = self.canvas_anchor_popup.take() {
            previous
                .source_ui
                .close_canvas_anchor_popup(&previous.id, false);
        }
        let instance = ViewInstanceId::new(instance_id);
        let ticket = match self
            .editor_manager
            .enter()
            .map_err(EditorError::from)
            .and_then(|manager| {
                manager.begin_ui_asset_editor_canvas_anchor_popup(&instance, &address.source_frame)
            }) {
            Ok(Some(ticket)) => ticket,
            Ok(None) => return,
            Err(error) => {
                self.set_status_line(error.to_string());
                return;
            }
        };
        let id = CanvasAnchorPopupId::new();
        let manager_handle = self.editor_manager.clone();
        let original_ticket = ticket.clone();
        let original_instance = instance.clone();
        let retire = Rc::new(move || {
            manager_handle
                .enter()
                .map_err(|error| error.to_string())
                .and_then(|manager| {
                    manager
                        .cancel_ui_asset_editor_canvas_anchor_popup(
                            &original_instance,
                            &original_ticket,
                        )
                        .map(|_| ())
                        .map_err(|error| error.to_string())
                })
        });
        if !source_ui.show_canvas_anchor_popup(id.clone(), address, retire) {
            // No popup was published: retire this same ticket without synchronization/focus.
            if let Err(error) = self
                .editor_manager
                .enter()
                .map_err(EditorError::from)
                .and_then(|manager| {
                    manager.cancel_ui_asset_editor_canvas_anchor_popup(&instance, &ticket)
                })
            {
                self.set_status_line(error.to_string());
            }
            return;
        }
        self.canvas_anchor_popup = Some(PendingCanvasAnchorPopup {
            id,
            source_ui: source_ui.clone_strong(),
            instance_id: instance,
            ticket,
        });
        // Opening only publishes native popup pixels; never rebuild/sync the preview.
    }

    pub(in crate::ui::retained_host::app) fn dispatch_canvas_anchor_popup_event(
        &mut self,
        source_ui: &UiHostWindow,
        event: CanvasAnchorPopupEvent,
    ) {
        let Some(pending) = self.canvas_anchor_popup.as_ref() else {
            return;
        };
        // All ownership checks precede focus, close, status or presentation mutation.
        if !pending.id.same(&event.id)
            || !pending.source_ui.same_native_window(source_ui)
            || !source_ui.canvas_anchor_popup_is_open(&event.id)
            || !self.canvas_popup_source_is_registered(source_ui)
        {
            return;
        }
        let address_current = source_ui.canvas_anchor_popup_has_current_address(&event.id);
        let pending = self.canvas_anchor_popup.take().expect("identity checked");
        let mut return_focus = false;
        let mut changed = false;
        match event.kind {
            CanvasAnchorPopupKind::Submit {
                index,
                shift,
                control,
            } if address_current => {
                let result = self
                    .editor_manager
                    .enter()
                    .map_err(EditorError::from)
                    .and_then(|manager| {
                        manager.submit_ui_asset_editor_canvas_anchor_popup(
                            &pending.instance_id,
                            &pending.ticket,
                            index,
                            shift,
                            control,
                        )
                    });
                match result {
                    Ok(Some(outcome)) => {
                        changed = outcome.changed;
                        return_focus = true;
                        if let Some(error) = outcome.sync_error {
                            self.set_status_line(error);
                        }
                    }
                    Ok(None) => {}
                    Err(error) => self.set_status_line(error.to_string()),
                }
            }
            CanvasAnchorPopupKind::Escape if address_current => {
                match self
                    .editor_manager
                    .enter()
                    .map_err(EditorError::from)
                    .and_then(|manager| {
                        manager.ui_asset_editor_canvas_anchor_popup_is_current(
                            &pending.instance_id,
                            &pending.ticket,
                        )
                    }) {
                    Ok(current) => return_focus = current,
                    Err(error) => self.set_status_line(error.to_string()),
                }
            }
            _ => {}
        }
        // The window's exact-id retirement invokes only original-ticket cancel.
        // A stale event can never close or focus a later popup, even in the same view.
        let closed = source_ui.close_canvas_anchor_popup(&event.id, return_focus);
        if closed && return_focus && self.canvas_popup_source_is_registered(source_ui) {
            self.focus_callback_source_window();
        }
        if changed {
            self.mark_presentation_dirty_for_view(&pending.instance_id);
            if closed && self.canvas_popup_source_is_registered(source_ui) {
                let size = source_ui.window().size();
                // Queues the existing frame-update entry; never invokes the app while borrowed.
                // The ticket has already been consumed and its exact popup closed.
                source_ui.request_frame_update_region(
                    crate::ui::retained_host::host_contract::FrameRect {
                        x: 0.0,
                        y: 0.0,
                        width: size.width as f32,
                        height: size.height as f32,
                    },
                );
            }
        }
    }
}
