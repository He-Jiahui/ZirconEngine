use super::*;

#[test]
fn ui_window_events_carry_cursor_focus_scale_redraw_and_close_contracts() {
    let metadata = sample_window_metadata();
    let cursor_moved = UiWindowEvent::new(
        metadata.clone(),
        UiWindowEventKind::CursorMoved {
            position: UiPoint::new(32.0, 48.0),
            delta: Some(UiPoint::new(4.0, -2.0)),
        },
    );
    let cursor_left = UiWindowEvent::new(metadata.clone(), UiWindowEventKind::CursorLeft);
    let scale_factor = UiWindowEvent::new(
        metadata.clone(),
        UiWindowEventKind::ScaleFactorChanged { scale_factor: 2.0 },
    );
    let resized = UiWindowEvent::new(
        metadata.clone(),
        UiWindowEventKind::Resized {
            metrics: UiWindowMetrics::new(
                UiSize::new(640.0, 360.0),
                UiWindowPixelSize::new(1280, 720),
                2.0,
            ),
        },
    );
    let size_changed = UiWindowEvent::size_changed(
        metadata.clone(),
        UiWindowMetrics::new(
            UiSize::new(800.0, 450.0),
            UiWindowPixelSize::new(1600, 900),
            2.0,
        ),
    );
    let moved = UiWindowEvent::new(
        metadata.clone(),
        UiWindowEventKind::Moved {
            position: UiWindowPixelPosition::new(12, 24),
        },
    );
    let moved_window =
        UiWindowEvent::moved_window(metadata.clone(), UiWindowPixelPosition::new(18, 36));
    let focused = UiWindowEvent::new(
        metadata.clone(),
        UiWindowEventKind::Focused { focused: true },
    );
    let window_focused = UiWindowEvent::window_focused(metadata.clone(), true);
    let window_unfocused = UiWindowEvent::window_focused(metadata.clone(), false);
    let activated =
        UiWindowEvent::window_activation_changed(metadata.clone(), UiWindowActivation::Activate);
    let activated_by_mouse = UiWindowEvent::window_activation_changed(
        metadata.clone(),
        UiWindowActivation::ActivateByMouse,
    );
    let deactivated =
        UiWindowEvent::window_activation_changed(metadata.clone(), UiWindowActivation::Deactivate);
    let app_active = UiWindowEvent::application_activation_changed(metadata.clone(), true);
    let app_inactive = UiWindowEvent::application_activation_changed(metadata.clone(), false);
    let redraw = UiWindowEvent::new(
        metadata.clone(),
        UiWindowEventKind::RequestRedraw {
            reason: UiWindowRedrawReason::Input,
        },
    );
    let os_paint = UiWindowEvent::os_paint(metadata.clone());
    let resizing_window = UiWindowEvent::resizing_window(metadata.clone());
    let non_client_action =
        UiWindowEvent::window_action(metadata.clone(), UiWindowAction::ClickedNonClientArea);
    let maximize_action = UiWindowEvent::window_action(metadata.clone(), UiWindowAction::Maximize);
    let restore_action = UiWindowEvent::window_action(metadata.clone(), UiWindowAction::Restore);
    let window_menu_action =
        UiWindowEvent::window_action(metadata.clone(), UiWindowAction::WindowMenu);
    let close = UiWindowEvent::new(metadata.clone(), UiWindowEventKind::CloseRequested);
    let window_close = UiWindowEvent::window_close(metadata);

    assert_eq!(cursor_moved.window_id().unwrap().0, "editor.main");
    assert!(cursor_moved.impact().input_state_dirty);
    assert!(cursor_left.impact().clears_hover);
    assert!(cursor_left.impact().requests_redraw);
    assert!(scale_factor.impact().layout_metrics_dirty);
    assert!(!scale_factor.impact().input_state_dirty);
    assert!(!scale_factor.impact().requests_redraw);
    assert!(!scale_factor.impact().clears_hover);
    assert!(resized.impact().layout_metrics_dirty);
    assert!(resized.impact().requests_redraw);
    assert!(matches!(
        size_changed.kind,
        UiWindowEventKind::Resized { metrics }
            if metrics.logical_size == UiSize::new(800.0, 450.0)
                && metrics.physical_size == UiWindowPixelSize::new(1600, 900)
                && metrics.scale_factor == 2.0
    ));
    assert!(size_changed.impact().layout_metrics_dirty);
    assert!(size_changed.impact().requests_redraw);
    assert!(!moved.impact().input_state_dirty);
    assert!(matches!(
        moved_window.kind,
        UiWindowEventKind::Moved { position }
            if position == UiWindowPixelPosition::new(18, 36)
    ));
    assert_eq!(moved_window.impact(), moved.impact());
    assert!(focused.impact().input_state_dirty);
    assert!(matches!(
        window_focused.kind,
        UiWindowEventKind::Focused { focused: true }
    ));
    assert!(matches!(
        window_unfocused.kind,
        UiWindowEventKind::Focused { focused: false }
    ));
    assert_eq!(window_focused.impact(), focused.impact());
    assert_eq!(window_unfocused.impact(), focused.impact());
    assert!(UiWindowActivation::Activate.is_active());
    assert!(UiWindowActivation::ActivateByMouse.is_active());
    assert!(!UiWindowActivation::Deactivate.is_active());
    assert!(matches!(
        activated.kind,
        UiWindowEventKind::Focused { focused: true }
    ));
    assert!(matches!(
        activated_by_mouse.kind,
        UiWindowEventKind::Focused { focused: true }
    ));
    assert!(matches!(
        deactivated.kind,
        UiWindowEventKind::Focused { focused: false }
    ));
    assert!(matches!(
        app_active.kind,
        UiWindowEventKind::ApplicationActivation { is_active: true }
    ));
    assert!(matches!(
        app_inactive.kind,
        UiWindowEventKind::ApplicationActivation { is_active: false }
    ));
    assert_eq!(activated.impact(), focused.impact());
    assert_eq!(activated_by_mouse.impact(), focused.impact());
    assert_eq!(deactivated.impact(), focused.impact());
    assert_eq!(app_active.impact(), focused.impact());
    assert_eq!(app_inactive.impact(), focused.impact());
    assert!(redraw.impact().requests_redraw);
    assert!(matches!(
        os_paint.kind,
        UiWindowEventKind::RequestRedraw {
            reason: UiWindowRedrawReason::Paint
        }
    ));
    assert!(matches!(
        resizing_window.kind,
        UiWindowEventKind::RequestRedraw {
            reason: UiWindowRedrawReason::Paint
        }
    ));
    assert!(os_paint.impact().requests_redraw);
    assert_eq!(resizing_window.impact(), os_paint.impact());
    assert!(matches!(
        non_client_action.kind,
        UiWindowEventKind::WindowAction {
            action: UiWindowAction::ClickedNonClientArea
        }
    ));
    assert!(matches!(
        maximize_action.kind,
        UiWindowEventKind::WindowAction {
            action: UiWindowAction::Maximize
        }
    ));
    assert!(matches!(
        restore_action.kind,
        UiWindowEventKind::WindowAction {
            action: UiWindowAction::Restore
        }
    ));
    assert!(matches!(
        window_menu_action.kind,
        UiWindowEventKind::WindowAction {
            action: UiWindowAction::WindowMenu
        }
    ));
    assert_eq!(non_client_action.impact(), UiWindowEventImpact::clean());
    assert_eq!(maximize_action.impact(), UiWindowEventImpact::clean());
    assert_eq!(restore_action.impact(), UiWindowEventImpact::clean());
    assert_eq!(window_menu_action.impact(), UiWindowEventImpact::clean());
    assert!(close.impact().close_requested);
    assert_eq!(window_close.impact(), close.impact());
    assert!(matches!(
        window_close.kind,
        UiWindowEventKind::CloseRequested
    ));
    assert_eq!(round_trip(&cursor_moved), cursor_moved);
    assert_eq!(round_trip(&size_changed), size_changed);
    assert_eq!(round_trip(&activated), activated);
    assert_eq!(round_trip(&deactivated), deactivated);
    assert_eq!(
        round_trip(&UiWindowActivation::ActivateByMouse),
        UiWindowActivation::ActivateByMouse
    );
    assert_eq!(round_trip(&non_client_action), non_client_action);
    assert_eq!(round_trip(&window_menu_action), window_menu_action);
    assert_eq!(
        round_trip(&UiWindowAction::ClickedNonClientArea),
        UiWindowAction::ClickedNonClientArea
    );
    assert_eq!(round_trip(&os_paint), os_paint);
    assert_eq!(round_trip(&window_close), window_close);
}
