use super::super::super::{globals::PaneSurfaceHostContext, redraw::NativePointerDispatchResult};
use super::{
    CanvasAnchorPopupEvent, CanvasAnchorPopupId, CanvasAnchorPopupKind, CanvasAnchorPopupVisual,
    UiHostWindow,
};
use zircon_runtime_interface::ui::{
    dispatch::{UiInputEvent, UiInputEventMetadata, UiKeyboardInputState},
    surface::{UiPointerButton, UiPointerEventKind},
};

impl UiHostWindow {
    pub(crate) fn dispatch_canvas_anchor_popup_input(
        &self,
        event: &UiInputEvent,
    ) -> Option<NativePointerDispatchResult> {
        {
            let state = self.state.borrow();
            if state.canvas_anchor_popup.is_none()
                && state.canvas_anchor_popup_pointer_tail.is_none()
            {
                return None;
            }
        }
        let native_window = super::super::metadata::native_input_metadata(self, 0).window_id;
        let metadata: Option<&UiInputEventMetadata> = match event {
            UiInputEvent::Pointer(input) => Some(&input.metadata),
            UiInputEvent::Keyboard(input) => Some(&input.metadata),
            UiInputEvent::Text(input) => Some(&input.metadata),
            UiInputEvent::Ime(input) => Some(&input.metadata),
            _ => None,
        };
        // A foreign/missing window cannot clear a tail, retire a popup or affect focus.
        if native_window.is_none()
            || metadata.is_none_or(|metadata| metadata.window_id != native_window)
        {
            return Some(NativePointerDispatchResult::idle());
        }
        // Match the native mouse dispatcher's default pointer before storing or comparing IDs.
        // Touch/pen cannot clear a mouse press or a release left behind by a closed popup.
        if let UiInputEvent::Pointer(pointer) = event {
            if pointer.metadata.pointer_source.is_touch_like() {
                return Some(NativePointerDispatchResult::idle());
            }
            let pointer_id = pointer.metadata.pointer_id.unwrap_or_default();
            let mut state = self.state.borrow_mut();
            let owns_tail = state.canvas_anchor_popup_pointer_tail == Some(pointer_id);
            // Consume only the retired popup's pointer, even if a newer popup has another press.
            if owns_tail
                && (pointer.event.kind == UiPointerEventKind::Cancel
                    || (pointer.event.kind == UiPointerEventKind::Up
                        && pointer.event.button == Some(UiPointerButton::Primary)))
            {
                state.canvas_anchor_popup_pointer_tail = None;
                return Some(NativePointerDispatchResult::idle());
            }
            if state
                .canvas_anchor_popup
                .as_ref()
                .and_then(|popup| popup.pressed)
                .is_some_and(|(_, owner)| owner != pointer_id)
            {
                return Some(NativePointerDispatchResult::idle());
            }
            if owns_tail
                && pointer.event.kind == UiPointerEventKind::Down
                && pointer.event.button == Some(UiPointerButton::Primary)
            {
                state.canvas_anchor_popup_pointer_tail = None;
            }
        }
        let (id, damage) = {
            let state = self.state.borrow();
            let popup = state.canvas_anchor_popup.as_ref()?;
            (popup.id.clone(), popup.visual.bounds.clone())
        };
        if !self.canvas_anchor_popup_has_current_address(&id) {
            self.close_canvas_anchor_popup(&id, false);
            return Some(NativePointerDispatchResult::region(damage));
        }
        let mut completion = None;
        {
            let mut state = self.state.borrow_mut();
            let popup = state
                .canvas_anchor_popup
                .as_mut()
                .filter(|popup| popup.id.same(&id))?;
            match event {
                UiInputEvent::Pointer(input) => {
                    let pointer_id = input.metadata.pointer_id.unwrap_or_default();
                    let point = input.event.point;
                    let hit = popup.visual.cell_at(point.x, point.y);
                    match input.event.kind {
                        UiPointerEventKind::Down
                            if input.event.button == Some(UiPointerButton::Primary) =>
                        {
                            popup.hovered = hit;
                            if let Some(index) = hit {
                                popup.focused = index;
                                popup.pressed = Some((index, pointer_id));
                            } else if !popup.visual.contains(point.x, point.y) {
                                popup.swallow_release = Some(pointer_id);
                                completion = Some(CanvasAnchorPopupKind::ClickAway);
                            }
                        }
                        UiPointerEventKind::Up
                            if input.event.button == Some(UiPointerButton::Primary) =>
                        {
                            popup.hovered = hit;
                            let press = popup.pressed.take();
                            if let (Some((pressed, pointer)), Some(index)) = (press, hit) {
                                if pressed == index && pointer == pointer_id {
                                    completion = Some(CanvasAnchorPopupKind::Submit {
                                        index,
                                        shift: input.metadata.modifiers.shift,
                                        control: input.metadata.modifiers.control,
                                    });
                                }
                            }
                        }
                        UiPointerEventKind::Cancel => {
                            popup.pressed = None;
                            popup.hovered = None;
                        }
                        // Hover cannot replace the user's keyboard choice. A click can.
                        UiPointerEventKind::Move => popup.hovered = hit,
                        _ => {}
                    }
                }
                UiInputEvent::Keyboard(input) if input.state != UiKeyboardInputState::Released => {
                    match input.logical_key.as_str() {
                        "Escape" if input.state == UiKeyboardInputState::Pressed => {
                            completion = Some(CanvasAnchorPopupKind::Escape)
                        }
                        "Enter" | " " if input.state == UiKeyboardInputState::Pressed => {
                            completion = Some(CanvasAnchorPopupKind::Submit {
                                index: popup.focused,
                                shift: input.metadata.modifiers.shift,
                                control: input.metadata.modifiers.control,
                            })
                        }
                        "ArrowLeft" => {
                            popup.focused = (popup.focused / 4) * 4 + (popup.focused + 3) % 4
                        }
                        "ArrowRight" => {
                            popup.focused = (popup.focused / 4) * 4 + (popup.focused + 1) % 4
                        }
                        "ArrowUp" => popup.focused = (popup.focused + 12) % 16,
                        "ArrowDown" => popup.focused = (popup.focused + 4) % 16,
                        "Home" => popup.focused = 0,
                        "End" => popup.focused = 15,
                        _ => {}
                    }
                }
                // Text/IME never reaches an asset or another pane while this popup owns input.
                _ => {}
            }
        }
        if let Some(kind) = completion {
            // Callback carries this same popup identity. It cannot resolve a newer popup.
            self.global::<PaneSurfaceHostContext>()
                .invoke_canvas_anchor_popup_event(CanvasAnchorPopupEvent { id, kind });
        } else {
            self.refresh_canvas_anchor_popup_visual(&id);
        }
        Some(NativePointerDispatchResult::region(damage))
    }

    fn refresh_canvas_anchor_popup_visual(&self, id: &CanvasAnchorPopupId) {
        let size = self.window().size();
        let scale = self.window().scale_factor();
        let mut state = self.state.borrow_mut();
        let Some(popup) = state
            .canvas_anchor_popup
            .as_mut()
            .filter(|popup| popup.id.same(id))
        else {
            return;
        };
        let Some(visual) = CanvasAnchorPopupVisual::new(
            &popup.address,
            size,
            scale,
            popup.focused,
            popup.hovered,
            popup.pressed.map(|value| value.0),
        ) else {
            return;
        };
        popup.visual = std::rc::Rc::new(visual);
        let visual = std::rc::Rc::clone(&popup.visual);
        state.update_host_presentation(|presentation| {
            presentation.canvas_anchor_popup = Some(visual)
        });
    }
}

#[cfg(test)]
mod hover_focus_tests {
    use super::*;
    use std::{cell::RefCell, rc::Rc};
    use zircon_runtime_interface::ui::{
        dispatch::{
            UiInputModifiers, UiKeyboardInputEvent, UiPointerEvent, UiPointerId,
            UiPointerInputEvent,
        },
        layout::UiPoint,
    };

    fn keyboard(ui: &UiHostWindow, key: &str, modifiers: UiInputModifiers) -> UiInputEvent {
        let mut metadata = super::super::super::metadata::native_input_metadata(ui, 1);
        metadata.modifiers = modifiers;
        UiInputEvent::Keyboard(UiKeyboardInputEvent {
            metadata,
            state: UiKeyboardInputState::Pressed,
            key_code: 0,
            scan_code: None,
            physical_key: key.into(),
            logical_key: key.into(),
            text: None,
        })
    }

    fn pointer(
        ui: &UiHostWindow,
        kind: UiPointerEventKind,
        preset: Option<usize>,
        modifiers: UiInputModifiers,
    ) -> UiInputEvent {
        // Hit the real projected Button frame instead of duplicating popup geometry.
        let point = preset.map_or(UiPoint::new(-1.0, -1.0), |index| {
            let state = ui.state.borrow();
            let popup = state.canvas_anchor_popup.as_ref().unwrap();
            let node = popup
                .visual
                .nodes
                .iter()
                .find(|node| node.inspector_property_item_key.as_str() == index.to_string())
                .expect("real preset Button");
            UiPoint::new(
                node.frame.x + node.frame.width / 2.0,
                node.frame.y + node.frame.height / 2.0,
            )
        });
        let mut metadata = super::super::super::metadata::native_input_metadata(ui, 2);
        metadata.pointer_id = Some(UiPointerId::default());
        metadata.modifiers = modifiers;
        let mut event = UiPointerEvent::new(kind, point);
        if matches!(kind, UiPointerEventKind::Down | UiPointerEventKind::Up) {
            event = event.with_button(UiPointerButton::Primary);
        }
        UiInputEvent::Pointer(UiPointerInputEvent {
            metadata,
            event,
            precise_scroll: None,
        })
    }

    #[test]
    fn canvas_anchor_pointer_hover_and_leave_preserve_keyboard_choice_and_original_ticket() {
        let (ui, session, _) = super::super::open_consumer_tests::published_canvas();
        let ticket = session
            .borrow_mut()
            .begin_canvas_anchor_preset()
            .unwrap()
            .unwrap();
        let address = ui.canvas_anchor_popup_address("canvas-view").unwrap();
        let frame = address.source_frame.clone();
        let source = session.borrow().source_buffer().text().to_string();
        let id = CanvasAnchorPopupId::new();
        let (cancel_session, cancel_ticket) = (session.clone(), ticket.clone());
        assert!(ui.show_canvas_anchor_popup(
            id.clone(),
            address,
            Rc::new(move || {
                assert!(cancel_session
                    .borrow_mut()
                    .cancel_canvas_anchor_preset(&cancel_ticket));
                Ok(())
            })
        ));
        let events = Rc::new(RefCell::new(Vec::new()));
        let received = events.clone();
        ui.global::<PaneSurfaceHostContext>()
            .on_canvas_anchor_popup_event(move |event| {
                received.borrow_mut().push(event);
            });

        assert!(ui
            .dispatch_canvas_anchor_popup_input(&keyboard(&ui, "End", Default::default()))
            .is_some());
        assert!(ui
            .dispatch_canvas_anchor_popup_input(&pointer(
                &ui,
                UiPointerEventKind::Move,
                Some(5),
                Default::default()
            ))
            .is_some());
        {
            let state = ui.state.borrow();
            let popup = state.canvas_anchor_popup.as_ref().unwrap();
            assert_eq!(popup.focused, 15);
            assert_eq!(popup.hovered, Some(5));
            let hovered = popup
                .visual
                .nodes
                .iter()
                .find(|node| node.inspector_property_item_key.as_str() == "5")
                .unwrap();
            let focused = popup
                .visual
                .nodes
                .iter()
                .find(|node| node.inspector_property_item_key.as_str() == "15")
                .unwrap();
            assert!(hovered.hovered && !hovered.focused);
            assert!(focused.focused && !focused.hovered);
        }
        assert!(ui
            .dispatch_canvas_anchor_popup_input(&pointer(
                &ui,
                UiPointerEventKind::Cancel,
                None,
                Default::default()
            ))
            .is_some());
        {
            let state = ui.state.borrow();
            let popup = state.canvas_anchor_popup.as_ref().unwrap();
            assert_eq!(popup.focused, 15);
            assert_eq!(popup.hovered, None);
        }
        let modifiers = UiInputModifiers {
            shift: true,
            control: true,
            ..Default::default()
        };
        assert!(ui
            .dispatch_canvas_anchor_popup_input(&keyboard(&ui, "Enter", modifiers))
            .is_some());
        let events = events.borrow();
        assert_eq!(events.len(), 1);
        assert!(events[0].id.same(&id));
        assert!(matches!(
            events[0].kind,
            CanvasAnchorPopupKind::Submit {
                index: 15,
                shift: true,
                control: true
            }
        ));
        assert!(ticket.is_current(&session.borrow()));
        assert_eq!(session.borrow().source_buffer().text(), source);
        assert!(std::sync::Arc::ptr_eq(
            &frame,
            &ui.canvas_anchor_popup_address("canvas-view")
                .unwrap()
                .source_frame
        ));
        assert!(ui.close_canvas_anchor_popup(&id, false));
        assert!(!ticket.is_current(&session.borrow()));
    }

    #[test]
    fn canvas_anchor_pointer_click_keeps_press_release_choice_and_release_modifiers() {
        let (ui, session, _) = super::super::open_consumer_tests::published_canvas();
        let ticket = session
            .borrow_mut()
            .begin_canvas_anchor_preset()
            .unwrap()
            .unwrap();
        let id = CanvasAnchorPopupId::new();
        let (cancel_session, cancel_ticket) = (session.clone(), ticket.clone());
        assert!(ui.show_canvas_anchor_popup(
            id.clone(),
            ui.canvas_anchor_popup_address("canvas-view").unwrap(),
            Rc::new(move || {
                assert!(cancel_session
                    .borrow_mut()
                    .cancel_canvas_anchor_preset(&cancel_ticket));
                Ok(())
            })
        ));
        let events = Rc::new(RefCell::new(Vec::new()));
        let received = events.clone();
        ui.global::<PaneSurfaceHostContext>()
            .on_canvas_anchor_popup_event(move |event| {
                received.borrow_mut().push(event);
            });

        for (kind, index) in [
            (UiPointerEventKind::Down, 8),
            (UiPointerEventKind::Move, 9),
            (UiPointerEventKind::Up, 9),
        ] {
            assert!(ui
                .dispatch_canvas_anchor_popup_input(&pointer(
                    &ui,
                    kind,
                    Some(index),
                    Default::default()
                ))
                .is_some());
        }
        assert!(
            events.borrow().is_empty(),
            "release over another cell must not submit"
        );
        {
            let state = ui.state.borrow();
            let popup = state.canvas_anchor_popup.as_ref().unwrap();
            assert_eq!(popup.focused, 8);
            assert_eq!(popup.hovered, Some(9));
            assert!(popup.pressed.is_none());
        }
        assert!(ui
            .dispatch_canvas_anchor_popup_input(&pointer(
                &ui,
                UiPointerEventKind::Down,
                Some(8),
                Default::default()
            ))
            .is_some());
        let modifiers = UiInputModifiers {
            control: true,
            ..Default::default()
        };
        assert!(ui
            .dispatch_canvas_anchor_popup_input(&pointer(
                &ui,
                UiPointerEventKind::Up,
                Some(8),
                modifiers
            ))
            .is_some());
        let events = events.borrow();
        assert_eq!(events.len(), 1);
        assert!(events[0].id.same(&id));
        assert!(matches!(
            events[0].kind,
            CanvasAnchorPopupKind::Submit {
                index: 8,
                shift: false,
                control: true
            }
        ));
        assert!(ticket.is_current(&session.borrow()));
        assert!(ui.close_canvas_anchor_popup(&id, false));
    }

    fn native_mouse_button(
        ui: &UiHostWindow,
        state: winit::event::ElementState,
        preset: Option<usize>,
        modifiers: UiInputModifiers,
    ) -> UiInputEvent {
        let point = if let Some(index) = preset {
            let UiInputEvent::Pointer(hit) = pointer(
                ui,
                UiPointerEventKind::Move,
                Some(index),
                Default::default(),
            ) else {
                unreachable!()
            };
            hit.event.point
        } else {
            let address = ui.canvas_anchor_popup_address("canvas-view").unwrap();
            UiPoint::new(
                address.opener.frame.x + address.opener.frame.width / 2.0,
                address.opener.frame.y + address.opener.frame.height / 2.0,
            )
        };
        let mut metadata = super::super::super::metadata::native_input_metadata(ui, 3);
        metadata.modifiers = modifiers;
        assert!(
            metadata.pointer_id.is_none(),
            "ordinary native mouse metadata"
        );
        let translated = zircon_runtime::ui::platform_input::translate_winit_window_event(
            zircon_runtime_interface::ui::window::UiWindowInputContext {
                metadata,
                ..Default::default()
            },
            &winit::event::WindowEvent::PointerButton {
                device_id: None,
                state,
                position: winit::dpi::PhysicalPosition::new(f64::from(point.x), f64::from(point.y)),
                primary: true,
                button: winit::event::ButtonSource::Mouse(winit::event::MouseButton::Left),
            },
        )
        .expect("actual native mouse translator");
        let zircon_runtime_interface::ui::window::UiWindowInputPumpEvent::Input(event) = translated
        else {
            panic!("mouse button must produce input")
        };
        let UiInputEvent::Pointer(input) = &event else {
            panic!("mouse button must produce pointer input")
        };
        assert!(input.metadata.pointer_id.is_none());
        event
    }

    #[test]
    fn canvas_anchor_native_mouse_without_explicit_id_clicks_and_leaves_original_popup() {
        let (ui, session, _) = super::super::open_consumer_tests::published_canvas();
        let ticket = session
            .borrow_mut()
            .begin_canvas_anchor_preset()
            .unwrap()
            .unwrap();
        let source = session.borrow().source_buffer().text().to_string();
        let address = ui.canvas_anchor_popup_address("canvas-view").unwrap();
        let frame = address.source_frame.clone();
        let id = CanvasAnchorPopupId::new();
        let (cancel_session, cancel_ticket) = (session.clone(), ticket.clone());
        assert!(ui.show_canvas_anchor_popup(
            id.clone(),
            address,
            Rc::new(move || {
                assert!(cancel_session
                    .borrow_mut()
                    .cancel_canvas_anchor_preset(&cancel_ticket));
                Ok(())
            })
        ));
        let events = Rc::new(RefCell::new(Vec::new()));
        let received = events.clone();
        ui.global::<PaneSurfaceHostContext>()
            .on_canvas_anchor_popup_event(move |event| {
                received.borrow_mut().push(event);
            });
        assert!(ui
            .dispatch_canvas_anchor_popup_input(&native_mouse_button(
                &ui,
                winit::event::ElementState::Pressed,
                Some(8),
                Default::default()
            ))
            .is_some());
        assert_eq!(
            ui.state
                .borrow()
                .canvas_anchor_popup
                .as_ref()
                .unwrap()
                .pressed,
            Some((8, UiPointerId::default()))
        );
        // The real native CursorLeft path also carries no explicit pointer ID.
        ui.dispatch_ui_asset_native_pointer_left_for_test();
        {
            let state = ui.state.borrow();
            let popup = state.canvas_anchor_popup.as_ref().unwrap();
            assert_eq!(popup.pressed, None);
            assert_eq!(popup.hovered, None);
            assert_eq!(popup.focused, 8);
        }
        assert!(events.borrow().is_empty());
        for state in [
            winit::event::ElementState::Pressed,
            winit::event::ElementState::Released,
        ] {
            let modifiers = UiInputModifiers {
                shift: true,
                control: true,
                ..Default::default()
            };
            assert!(ui
                .dispatch_canvas_anchor_popup_input(&native_mouse_button(
                    &ui,
                    state,
                    Some(8),
                    modifiers
                ))
                .is_some());
        }
        {
            let events = events.borrow();
            assert_eq!(events.len(), 1);
            assert!(events[0].id.same(&id));
            assert!(matches!(
                events[0].kind,
                CanvasAnchorPopupKind::Submit {
                    index: 8,
                    shift: true,
                    control: true
                }
            ));
        }
        assert!(ticket.is_current(&session.borrow()));
        assert_eq!(session.borrow().source_buffer().text(), source);
        assert!(std::sync::Arc::ptr_eq(
            &frame,
            &ui.canvas_anchor_popup_address("canvas-view")
                .unwrap()
                .source_frame
        ));
        assert!(ui.close_canvas_anchor_popup(&id, false));
        assert!(!ticket.is_current(&session.borrow()));
    }

    #[test]
    fn canvas_anchor_foreign_pointer_cannot_replace_release_or_cancel_the_owning_press() {
        use zircon_runtime_interface::ui::dispatch::UiPointerSource;
        let (ui, session, _) = super::super::open_consumer_tests::published_canvas();
        let ticket = session
            .borrow_mut()
            .begin_canvas_anchor_preset()
            .unwrap()
            .unwrap();
        let id = CanvasAnchorPopupId::new();
        let (cancel_session, cancel_ticket) = (session.clone(), ticket.clone());
        assert!(ui.show_canvas_anchor_popup(
            id.clone(),
            ui.canvas_anchor_popup_address("canvas-view").unwrap(),
            Rc::new(move || {
                assert!(cancel_session
                    .borrow_mut()
                    .cancel_canvas_anchor_preset(&cancel_ticket));
                Ok(())
            })
        ));
        let events = Rc::new(RefCell::new(Vec::new()));
        let received = events.clone();
        ui.global::<PaneSurfaceHostContext>()
            .on_canvas_anchor_popup_event(move |event| {
                received.borrow_mut().push(event);
            });
        let owner = UiPointerId::new(17);
        let with_owner = |mut event: UiInputEvent, pointer_id, pointer_source| {
            let UiInputEvent::Pointer(input) = &mut event else {
                unreachable!()
            };
            input.metadata.pointer_id = pointer_id;
            input.metadata.pointer_source = pointer_source;
            event
        };
        let down = with_owner(
            pointer(&ui, UiPointerEventKind::Down, Some(8), Default::default()),
            Some(owner),
            UiPointerSource::Mouse,
        );
        assert!(ui.dispatch_canvas_anchor_popup_input(&down).is_some());
        let visual = ui
            .state
            .borrow()
            .canvas_anchor_popup
            .as_ref()
            .unwrap()
            .visual
            .clone();
        for (pointer_id, source) in [
            (Some(UiPointerId::new(18)), UiPointerSource::Mouse),
            (None, UiPointerSource::Mouse),
            (Some(owner), UiPointerSource::Touch),
            (Some(owner), UiPointerSource::Pen),
        ] {
            for (kind, hit) in [
                (UiPointerEventKind::Down, Some(5)),
                (UiPointerEventKind::Down, None),
                (UiPointerEventKind::Move, Some(5)),
                (UiPointerEventKind::Up, Some(8)),
                (UiPointerEventKind::Cancel, None),
            ] {
                let event = with_owner(
                    pointer(&ui, kind, hit, Default::default()),
                    pointer_id,
                    source,
                );
                assert!(ui.dispatch_canvas_anchor_popup_input(&event).is_some());
                let state = ui.state.borrow();
                let popup = state.canvas_anchor_popup.as_ref().unwrap();
                assert!(popup.id.same(&id));
                assert_eq!(popup.pressed, Some((8, owner)));
                assert_eq!(popup.focused, 8);
                assert_eq!(popup.hovered, Some(8));
                assert!(Rc::ptr_eq(&popup.visual, &visual));
                assert!(events.borrow().is_empty());
                assert!(ticket.is_current(&session.borrow()));
            }
        }
        let modifiers = UiInputModifiers {
            control: true,
            ..Default::default()
        };
        let up = with_owner(
            pointer(&ui, UiPointerEventKind::Up, Some(8), modifiers),
            Some(owner),
            UiPointerSource::Mouse,
        );
        assert!(ui.dispatch_canvas_anchor_popup_input(&up).is_some());
        {
            let events = events.borrow();
            assert_eq!(events.len(), 1);
            assert!(events[0].id.same(&id));
            assert!(matches!(
                events[0].kind,
                CanvasAnchorPopupKind::Submit {
                    index: 8,
                    shift: false,
                    control: true
                }
            ));
        }
        assert!(ui.close_canvas_anchor_popup(&id, false));
    }

    #[test]
    fn canvas_anchor_closed_popup_release_tail_preserves_new_popup_pointer_ownership() {
        use zircon_runtime_interface::ui::dispatch::UiPointerSource;
        let (ui, session, _) = super::super::open_consumer_tests::published_canvas();
        let ticket = session
            .borrow_mut()
            .begin_canvas_anchor_preset()
            .unwrap()
            .unwrap();
        let id = CanvasAnchorPopupId::new();
        let (cancel_session, cancel_ticket) = (session.clone(), ticket.clone());
        assert!(ui.show_canvas_anchor_popup(
            id.clone(),
            ui.canvas_anchor_popup_address("canvas-view").unwrap(),
            Rc::new(move || {
                assert!(cancel_session
                    .borrow_mut()
                    .cancel_canvas_anchor_preset(&cancel_ticket));
                Ok(())
            })
        ));
        let owner = UiPointerId::new(17);
        let mut down = pointer(&ui, UiPointerEventKind::Down, Some(8), Default::default());
        let UiInputEvent::Pointer(input) = &mut down else {
            unreachable!()
        };
        input.metadata.pointer_id = Some(owner);
        assert!(ui.dispatch_canvas_anchor_popup_input(&down).is_some());
        let mut up = down.clone();
        let UiInputEvent::Pointer(input) = &mut up else {
            unreachable!()
        };
        input.event.kind = UiPointerEventKind::Up;
        assert!(ui.close_canvas_anchor_popup(&id, false));
        assert!(!ticket.is_current(&session.borrow()));
        assert_eq!(
            ui.state.borrow().canvas_anchor_popup_pointer_tail,
            Some(owner)
        );
        for (pointer_id, source, consumed) in [
            (Some(UiPointerId::new(18)), UiPointerSource::Mouse, false),
            (None, UiPointerSource::Mouse, false),
            (Some(owner), UiPointerSource::Touch, true),
            (Some(owner), UiPointerSource::Pen, true),
        ] {
            let mut event = down.clone();
            let UiInputEvent::Pointer(input) = &mut event else {
                unreachable!()
            };
            input.metadata.pointer_id = pointer_id;
            input.metadata.pointer_source = source;
            assert_eq!(
                ui.dispatch_canvas_anchor_popup_input(&event).is_some(),
                consumed
            );
            assert_eq!(
                ui.state.borrow().canvas_anchor_popup_pointer_tail,
                Some(owner)
            );
            if !source.is_touch_like() {
                let UiInputEvent::Pointer(input) = &mut event else {
                    unreachable!()
                };
                input.event.kind = UiPointerEventKind::Up;
                assert!(ui.dispatch_canvas_anchor_popup_input(&event).is_none());
                assert_eq!(
                    ui.state.borrow().canvas_anchor_popup_pointer_tail,
                    Some(owner)
                );
            }
        }
        let next_ticket = session
            .borrow_mut()
            .begin_canvas_anchor_preset()
            .unwrap()
            .unwrap();
        let next_id = CanvasAnchorPopupId::new();
        let (cancel_session, cancel_ticket) = (session.clone(), next_ticket.clone());
        assert!(ui.show_canvas_anchor_popup(
            next_id.clone(),
            ui.canvas_anchor_popup_address("canvas-view").unwrap(),
            Rc::new(move || {
                assert!(cancel_session
                    .borrow_mut()
                    .cancel_canvas_anchor_preset(&cancel_ticket));
                Ok(())
            })
        ));
        let next_owner = UiPointerId::new(18);
        let mut next_down = down.clone();
        let UiInputEvent::Pointer(input) = &mut next_down else {
            unreachable!()
        };
        input.metadata.pointer_id = Some(next_owner);
        assert!(ui.dispatch_canvas_anchor_popup_input(&next_down).is_some());
        assert_eq!(
            ui.state.borrow().canvas_anchor_popup_pointer_tail,
            Some(owner)
        );
        // The old release is consumed without taking the newer popup's held press.
        assert!(ui.dispatch_canvas_anchor_popup_input(&up).is_some());
        {
            let state = ui.state.borrow();
            assert_eq!(state.canvas_anchor_popup_pointer_tail, None);
            let popup = state.canvas_anchor_popup.as_ref().unwrap();
            assert!(popup.id.same(&next_id));
            assert_eq!(popup.pressed, Some((8, next_owner)));
        }
        assert!(!ui.close_canvas_anchor_popup(&id, false));
        assert!(next_ticket.is_current(&session.borrow()));
        let UiInputEvent::Pointer(input) = &mut next_down else {
            unreachable!()
        };
        input.event.kind = UiPointerEventKind::Up;
        assert!(ui.dispatch_canvas_anchor_popup_input(&next_down).is_some());
        assert!(ui
            .state
            .borrow()
            .canvas_anchor_popup
            .as_ref()
            .unwrap()
            .pressed
            .is_none());
        assert!(ui.close_canvas_anchor_popup(&next_id, false));
        assert!(!next_ticket.is_current(&session.borrow()));
        assert!(
            ui.dispatch_canvas_anchor_popup_input(&up).is_none(),
            "old release tail was consumed once"
        );
    }

    #[test]
    fn canvas_anchor_native_click_away_swallows_only_the_matching_default_mouse_release() {
        let (ui, session, _) = super::super::open_consumer_tests::published_canvas();
        let ticket = session
            .borrow_mut()
            .begin_canvas_anchor_preset()
            .unwrap()
            .unwrap();
        let id = CanvasAnchorPopupId::new();
        let (cancel_session, cancel_ticket) = (session.clone(), ticket.clone());
        assert!(ui.show_canvas_anchor_popup(
            id.clone(),
            ui.canvas_anchor_popup_address("canvas-view").unwrap(),
            Rc::new(move || {
                assert!(cancel_session
                    .borrow_mut()
                    .cancel_canvas_anchor_preset(&cancel_ticket));
                Ok(())
            })
        ));
        let events = Rc::new(RefCell::new(Vec::new()));
        let received = events.clone();
        ui.global::<PaneSurfaceHostContext>()
            .on_canvas_anchor_popup_event(move |event| {
                received.borrow_mut().push(event);
            });
        let down = native_mouse_button(
            &ui,
            winit::event::ElementState::Pressed,
            None,
            Default::default(),
        );
        let UiInputEvent::Pointer(input) = &down else {
            unreachable!()
        };
        {
            let state = ui.state.borrow();
            let popup = state.canvas_anchor_popup.as_ref().unwrap();
            assert!(
                !popup
                    .visual
                    .contains(input.event.point.x, input.event.point.y),
                "the actual opener is inside the host and outside the popup"
            );
            assert!(input.event.point.x >= 0.0 && input.event.point.y >= 0.0);
        }
        assert!(ui.dispatch_canvas_anchor_popup_input(&down).is_some());
        let event_id = {
            let events = events.borrow();
            assert_eq!(events.len(), 1);
            assert!(events[0].id.same(&id));
            assert!(matches!(events[0].kind, CanvasAnchorPopupKind::ClickAway));
            events[0].id.clone()
        };
        assert!(ui.close_canvas_anchor_popup(&event_id, false));
        assert!(!ticket.is_current(&session.borrow()));
        assert_eq!(
            ui.state.borrow().canvas_anchor_popup_pointer_tail,
            Some(UiPointerId::default())
        );
        let up = native_mouse_button(
            &ui,
            winit::event::ElementState::Released,
            None,
            Default::default(),
        );
        assert!(ui.dispatch_canvas_anchor_popup_input(&up).is_some());
        assert_eq!(ui.state.borrow().canvas_anchor_popup_pointer_tail, None);
        assert!(ui.dispatch_canvas_anchor_popup_input(&up).is_none());
        assert_eq!(
            events.borrow().len(),
            1,
            "the swallowed release cannot submit"
        );
    }
}
