use super::*;
use std::{sync::Arc, time::Duration};
use crate::core::{
    i18n::EditorI18nService,
    notifications::{EditorNotificationService, NotificationId, NotificationSource, ToastNotification, ToastSeverity},
};
use crate::ui::{
    activity::activity_toast_views,
    layouts::common::model_rc,
    retained_host::{
        callback_dispatch::{BuiltinWorkbenchWindowTemplateSurfaceBridge, WORKBENCH_TOAST_CONTROL_ID},
        host_contract::presenter::paint_host_presentation_snapshot,
        to_host_contract_workbench_window_nodes,
    },
};
use zircon_runtime_interface::ui::layout::UiSize;

#[derive(Clone, Copy)]
enum PresentMode { Submit, CaptureNoop, ErrorAfterPaint }
struct Presenter {
    mode: PresentMode,
    diagnostics: HostRefreshDiagnostics,
}
impl HostChromePresenter for Presenter {
    fn resize(&mut self, _: (u32, u32)) -> Result<(), HostPresenterError> { Ok(()) }
    fn present(&mut self, presentation: &HostWindowPresentationData,
        _: HostPresentationGenerationCursor, _: Option<FrameRect>, _: HostInvalidationDiagnostics,
    ) -> Result<HostRefreshDiagnostics, HostPresenterError> {
        if matches!(self.mode, PresentMode::CaptureNoop) { return Ok(self.diagnostics.clone()); }
        // Execute the production command-stream painter; only the surface submit is injected.
        let _frame = paint_host_presentation_snapshot(1672, 941, presentation);
        self.diagnostics.present_count = self.diagnostics.present_count.saturating_add(1);
        if matches!(self.mode, PresentMode::ErrorAfterPaint) {
            Err(HostPresenterError::RetryableSurfacePresent)
        } else { Ok(self.diagnostics.clone()) }
    }
    fn diagnostics_snapshot(&self) -> HostRefreshDiagnostics { self.diagnostics.clone() }
}
fn presenter(mode: PresentMode) -> Presenter {
    Presenter { mode, diagnostics: HostRefreshDiagnostics::default() }
}
fn cursor(value: u64) -> HostPresentationGenerationCursor {
    HostPresentationGenerationCursor::new(value, value, value, value, value, value)
}
fn publish(service: &EditorNotificationService, id: &str, lifetime: Duration) {
    service.publish_toast(ToastNotification::new(NotificationId::parse(id).unwrap(),
        NotificationSource::builtin("editor.first-display.test").unwrap(), ToastSeverity::Error,
        "editor.notification.project_saved.title", "editor.notification.project_saved.message",
        lifetime).unwrap()).unwrap();
}
fn projected(service: &EditorNotificationService,
    bridge: &mut BuiltinWorkbenchWindowTemplateSurfaceBridge,
) -> HostWindowPresentationData {
    let (now, snapshots) = service.live_toast_snapshot();
    let views = activity_toast_views(&snapshots, &EditorI18nService::default(), now);
    bridge.sync_notification_snapshot(&[], &views, &[]).unwrap();
    let projection = bridge.host_projection();
    HostWindowPresentationData {
        workbench_window_nodes: to_host_contract_workbench_window_nodes(Some(&projection)),
        ..Default::default()
    }
}
fn observer(service: &Arc<EditorNotificationService>) -> ToastPresentationObserver {
    let observer = ToastPresentationObserver::default();
    let service = Arc::clone(service);
    observer.register(move |id, token| service.acknowledge_toast_presented(id, token).unwrap()).unwrap();
    observer
}
fn present(presenter: &mut Presenter, data: &HostWindowPresentationData, frame: u64)
    -> Option<ToastPresentationReceipt>
{
    let (result, receipt, _) = present_with_toast_confirmation(17, presenter, data, cursor(frame),
        None, HostInvalidationDiagnostics::default(), true);
    result.unwrap();
    receipt
}

#[test]
fn activity_toast_first_display_service_view_producer_paint_ack_starts_each_full_lifetime() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let service = Arc::new(EditorNotificationService::default());
    let lifetime = Duration::from_secs(3);
    publish(&service, "editor.first-display.9", lifetime);
    publish(&service, "editor.first-display.10", lifetime);
    let waiting = service.toasts().snapshot_at(Duration::from_secs(100));
    assert_eq!(waiting.len(), 2);
    assert!(waiting.iter().all(|entry| entry.expires_at().is_none()));
    assert_eq!(waiting[1].remaining_lifetime_at(Duration::from_secs(100)), lifetime);
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    let data = projected(&service, &mut bridge);
    let node = data.workbench_window_nodes.iter()
        .find(|node| node.control_id.as_str() == WORKBENCH_TOAST_CONTROL_ID).unwrap();
    assert_eq!(node.toast_presentation.as_ref().unwrap().0.as_str(), "editor.first-display.9");
    let second_token = waiting[1].delivery_token();
    bridge.mutate_control_property_for_test(WORKBENCH_TOAST_CONTROL_ID, "expired_toast_id",
        zircon_runtime_interface::ui::component::UiValue::String("editor.first-display.10".into())).unwrap();
    assert_eq!(bridge.dismissed_activity_toast(), Some(("editor.first-display.10".into(), second_token)),
        "a promoted queue entry must not inherit the previous head's delivery token");
    bridge.mutate_control_property_for_test(WORKBENCH_TOAST_CONTROL_ID, "expired_toast_id",
        zircon_runtime_interface::ui::component::UiValue::String(String::new())).unwrap();
    let observer = observer(&service);
    let before_ack = service.live_toast_snapshot().0;
    let receipt = present(&mut presenter(PresentMode::Submit), &data, 1)
        .expect("the real toast command stream must contain this delivery instance");
    assert!(observer.notify(receipt, 17, cursor(1), true));
    let after_ack = service.live_toast_snapshot().0;
    let shown = service.toast_snapshot();
    let deadline = shown[0].expires_at().unwrap();
    assert!(deadline >= before_ack + lifetime && deadline <= after_ack + lifetime);
    assert_eq!(service.toasts().snapshot_at(deadline - Duration::from_nanos(1)).len(), 2);
    assert_eq!(service.toasts().snapshot_at(deadline).len(), 1);
    assert_eq!(service.toast_snapshot()[0].notification().id().as_str(), "editor.first-display.10");
    let second = projected(&service, &mut bridge);
    let receipt = present(&mut presenter(PresentMode::Submit), &second, 2).unwrap();
    assert!(observer.notify(receipt, 17, cursor(2), true));
    assert!(service.toast_snapshot()[0].expires_at().is_some());
}

#[test]
fn activity_toast_first_display_capture_error_and_saturated_counter_cannot_ack() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let service = Arc::new(EditorNotificationService::default());
    publish(&service, "editor.first-display.capture", Duration::from_secs(1));
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    let data = projected(&service, &mut bridge);
    for mode in [PresentMode::CaptureNoop, PresentMode::ErrorAfterPaint, PresentMode::Submit] {
        let mut presenter = presenter(mode);
        if matches!(mode, PresentMode::Submit) { presenter.diagnostics.present_count = u64::MAX; }
        let (result, receipt, awaiting) = present_with_toast_confirmation(17, &mut presenter,
            &data, cursor(1), None, HostInvalidationDiagnostics::default(), true);
        assert!(awaiting);
        assert!(receipt.is_none());
        assert_eq!(result.is_err(), matches!(mode, PresentMode::ErrorAfterPaint));
        assert_eq!(service.toasts().snapshot_at(Duration::from_secs(100))[0].expires_at(), None);
    }
    assert!(present(&mut presenter(PresentMode::Submit), &data, 2).is_some(),
        "failed attempts must leave no stale TLS state");
}

#[test]
fn activity_toast_first_display_hidden_clipped_resize_and_retired_generation_cannot_ack() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let service = Arc::new(EditorNotificationService::default());
    publish(&service, "editor.first-display.hidden", Duration::from_secs(1));
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    let data = projected(&service, &mut bridge);
    let mut painter = presenter(PresentMode::Submit);
    let (_, receipt, awaiting) = present_with_toast_confirmation(17, &mut painter, &data, cursor(1),
        None, HostInvalidationDiagnostics::default(), false);
    assert!(receipt.is_none() && !awaiting);
    let mut clipped = data.clone();
    clipped.workbench_window_nodes = model_rc(data.workbench_window_nodes.iter().map(|mut node| {
        node.has_clip_frame = true; node.clip_frame = Default::default(); node
    }).collect());
    assert!(present(&mut painter, &clipped, 2).is_none());
    painter.present_during_native_resize(&data, cursor(1), HostInvalidationDiagnostics::default()).unwrap();
    assert!(ACTIVE_PAINT.with(|active| active.borrow().is_none()), "resize outside ordinary redraw cannot ACK");
    let observer = observer(&service);
    assert!(!observer.notify(present(&mut painter, &data, 3).unwrap(), 17, cursor(4), true));
    assert!(!observer.notify(present(&mut painter, &data, 4).unwrap(), 18, cursor(4), true));
    assert!(!observer.notify(present(&mut painter, &data, 4).unwrap(), 17, cursor(4), false));
    assert_eq!(service.toast_snapshot()[0].expires_at(), None);
    observer.clear();
    assert!(!observer.notify(present(&mut painter, &data, 4).unwrap(), 17, cursor(4), true));
}

#[test]
fn activity_toast_first_display_same_id_replacement_and_repeat_submit_never_restart_ttl() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let service = Arc::new(EditorNotificationService::default());
    let id = "editor.first-display.replaced";
    publish(&service, id, Duration::from_secs(3));
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    let old = projected(&service, &mut bridge);
    let old_entry = service.toast_snapshot()[0].clone();
    let late_receipt = present(&mut presenter(PresentMode::Submit), &old, 1).unwrap();
    assert!(service.dismiss_toast(id, old_entry.delivery_token()));
    publish(&service, id, Duration::from_secs(3));
    let observer = observer(&service);
    assert!(!observer.notify(late_receipt, 17, cursor(1), true));
    let new = projected(&service, &mut bridge);
    assert!(observer.notify(present(&mut presenter(PresentMode::Submit), &new, 2).unwrap(), 17, cursor(2), true));
    let deadline = service.toast_snapshot()[0].expires_at().unwrap();
    assert!(!observer.notify(present(&mut presenter(PresentMode::Submit), &new, 2).unwrap(), 17, cursor(2), true));
    assert_eq!(service.toast_snapshot()[0].expires_at(), Some(deadline));
}

#[test]
fn activity_toast_first_display_nested_paint_scope_restores_the_outer_attempt() {
    let outer = PaintScope::enter(1, cursor(1), None);
    {
        let _inner = PaintScope::enter(2, cursor(2), None);
        assert_eq!(ACTIVE_PAINT.with(|active| active.borrow().as_ref().unwrap().session), 2);
    }
    assert_eq!(ACTIVE_PAINT.with(|active| active.borrow().as_ref().unwrap().session), 1);
    drop(outer);
    assert!(ACTIVE_PAINT.with(|active| active.borrow().is_none()));
}

#[test]
fn activity_toast_first_display_control_timer_waits_for_ack_and_rounds_positive_remainder_up() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let service = EditorNotificationService::default();
    let lifetime = Duration::from_secs(3);
    publish(&service, "editor.first-display.timer", lifetime);
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    let now = Duration::from_secs(100);
    let pending = service.toasts().snapshot_at(now);
    let views = activity_toast_views(&pending, &EditorI18nService::default(), now);
    bridge.sync_notification_snapshot(&[], &views, &[]).unwrap();
    assert_eq!(bridge.activity_toast_timer_state_for_test(), (Some(0), true));
    assert!(service.toasts().acknowledge_at(pending[0].notification().id(),
        pending[0].delivery_token(), now).unwrap());
    let almost_expired = now + lifetime - Duration::from_nanos(1);
    let shown = service.toasts().snapshot_at(almost_expired);
    let views = activity_toast_views(&shown, &EditorI18nService::default(), almost_expired);
    assert_eq!(views[0].remaining_lifetime(), Duration::from_nanos(1));
    bridge.sync_notification_snapshot(&[], &views, &[]).unwrap();
    assert_eq!(bridge.activity_toast_timer_state_for_test(), (Some(1), false));
    assert!(service.toasts().snapshot_at(now + lifetime).is_empty());
}

#[test]
fn activity_toast_first_display_partially_clipped_production_paint_starts_lifetime() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let service = Arc::new(EditorNotificationService::default());
    let lifetime = Duration::from_secs(3);
    publish(&service, "editor.first-display.partial-clip", lifetime);
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    let mut data = projected(&service, &mut bridge);
    data.workbench_window_nodes = model_rc(data.workbench_window_nodes.iter().map(|mut node| {
        if node.toast_presentation.is_some() {
            assert!(node.frame.width > 0.0 && node.frame.height > 0.0);
            node.has_clip_frame = true;
            node.clip_frame.x = node.frame.x + node.frame.width * 0.5;
            node.clip_frame.y = node.frame.y;
            node.clip_frame.width = node.frame.width * 0.5;
            node.clip_frame.height = node.frame.height;
        }
        node
    }).collect());
    let pending = service.toast_snapshot();
    assert_eq!(pending[0].expires_at(), None);
    let observer = observer(&service);
    let before_ack = service.live_toast_snapshot().0;
    let receipt = present(&mut presenter(PresentMode::Submit), &data, 1)
        .expect("a nonempty clipped production toast draw is a first presentation");
    assert!(observer.notify(receipt, 17, cursor(1), true));
    let after_ack = service.live_toast_snapshot().0;
    let deadline = service.toast_snapshot()[0].expires_at().unwrap();
    assert!(deadline >= before_ack + lifetime && deadline <= after_ack + lifetime);
}
