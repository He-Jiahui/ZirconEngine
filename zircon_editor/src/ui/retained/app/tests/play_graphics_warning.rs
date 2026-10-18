use super::degraded_play_start_toast;
use crate::core::notifications::ToastSeverity;

#[test]
fn normal_graphics_has_no_warning_and_each_degraded_start_has_its_own_notice() {
    assert!(degraded_play_start_toast(1, None).is_none());
    assert!(degraded_play_start_toast(1, Some("  ")).is_none());
    let reason = "Software presenter (backend=softbuffer): GPU unavailable";
    let first = degraded_play_start_toast(3, Some(reason)).unwrap();
    let second = degraded_play_start_toast(4, Some(reason)).unwrap();
    assert_eq!(first.severity(), ToastSeverity::Warning);
    assert_eq!(first.message_key(), reason);
    assert_ne!(first.id(), second.id());
    assert_eq!(first.title_key(), "editor.notification.play_graphics_degraded.title");
}

#[test]
fn long_unicode_gpu_diagnostics_fit_the_notification_contract() {
    let diagnostic = "显卡不可用".repeat(80);
    let toast = degraded_play_start_toast(5, Some(&diagnostic)).unwrap();
    assert!(toast.message_key().len() <= 256);
    assert!(toast.message_key().ends_with("..."));
}
