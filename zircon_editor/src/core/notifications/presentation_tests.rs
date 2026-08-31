use std::sync::Arc;
use std::time::Duration;

use crate::core::i18n::{EditorI18nService, EditorLocale};
use crate::core::jobs::{EditorJobProgress, EditorJobProgressSnapshot, JobCategory, JobId};

use super::{
    DecisionCenterConfig, DecisionNotification, DecisionNotificationCenter, DecisionOption,
    DecisionOptionId, NotificationId, NotificationSource, ProgressNotification,
    ProgressNotificationCenter, ToastCenterConfig, ToastNotification, ToastNotificationCenter,
    ToastSeverity, present_decision, present_progress, present_toast,
};

#[test]
fn decision_projection_localizes_active_locale_without_losing_action_identity() {
    let i18n = EditorI18nService::default();
    let center = DecisionNotificationCenter::new(DecisionCenterConfig::default()).unwrap();
    let apply = DecisionOptionId::parse("apply").unwrap();
    let discard = DecisionOptionId::parse("discard").unwrap();
    let ticket = center
        .publish(
            DecisionNotification::new(
                NotificationId::parse("editor.play.pending_edits").unwrap(),
                NotificationSource::builtin("editor.play").unwrap(),
                "editor.play.pending_edits.title",
                "editor.play.pending_edits.message",
                vec![
                    DecisionOption::new(apply.clone(), "editor.play.pending_edits.apply").unwrap(),
                    DecisionOption::new(discard.clone(), "editor.play.pending_edits.discard")
                        .unwrap(),
                ],
            )
            .unwrap()
            .with_display_subject("assets/scenes/main.zscene")
            .unwrap()
            .with_message_argument("pending_count", 2)
            .unwrap()
            .with_message_argument("payload_bytes", 256)
            .unwrap()
            .with_message_argument("oldest_age_secs", 4)
            .unwrap()
            .with_default_option(apply.clone())
            .unwrap()
            .with_cancel_option(discard.clone())
            .unwrap(),
        )
        .unwrap();
    let snapshot = center.pending_snapshot().pop().unwrap();

    let english = present_decision(&snapshot, &i18n);
    assert_eq!(english.ticket(), &ticket);
    assert_eq!(english.title(), "Unsaved changes");
    assert_eq!(
        english.message(),
        "Resolve 2 queued changes (256 bytes; oldest 4s) before starting Play."
    );
    assert_eq!(english.options()[0].id(), &apply);
    assert_eq!(english.options()[0].label(), "Apply changes");
    assert_eq!(english.display_subject(), Some("assets/scenes/main.zscene"));
    assert_eq!(english.default_option(), Some(&apply));
    assert_eq!(english.cancel_option(), Some(&discard));
    assert_eq!(english.locale().as_str(), "en");

    i18n.set_active_locale(EditorLocale::parse("zh-CN").unwrap())
        .unwrap();
    let simplified_chinese = present_decision(&snapshot, &i18n);
    assert_eq!(simplified_chinese.ticket(), &ticket);
    assert_eq!(simplified_chinese.title(), "未保存的更改");
    assert_eq!(
        simplified_chinese.message(),
        "开始运行前，请处理 2 项待处理的更改（256 字节；最久 4 秒）。"
    );
    assert_eq!(simplified_chinese.options()[0].id(), &apply);
    assert_eq!(simplified_chinese.options()[0].label(), "应用更改");
    assert_eq!(simplified_chinese.locale().as_str(), "zh-CN");

    let receipt = center.resolve(&ticket, &apply).unwrap().receipt().clone();
    let resolved_snapshot = center.snapshot().pop().unwrap();
    let resolved = present_decision(&resolved_snapshot, &i18n);
    let projected_receipt = resolved.resolved().unwrap();
    assert_eq!(projected_receipt.sequence(), receipt.sequence());
    assert_eq!(projected_receipt.ticket(), receipt.ticket());
    assert_eq!(projected_receipt.option_id(), receipt.option_id());
}

#[test]
fn toast_projection_preserves_unknown_keys_for_diagnosis() {
    let i18n = EditorI18nService::default();
    let center = ToastNotificationCenter::new(ToastCenterConfig::default());
    center
        .publish_at(
            ToastNotification::new(
                NotificationId::parse("editor.notification.untranslated").unwrap(),
                NotificationSource::builtin("editor.core").unwrap(),
                ToastSeverity::Warning,
                "notification.unknown.title",
                "notification.unknown.message",
                Duration::from_secs(3),
            )
            .unwrap(),
            Duration::ZERO,
        )
        .unwrap();

    let snapshot = center.snapshot_at(Duration::ZERO).pop().unwrap();
    let presented = present_toast(&snapshot, &i18n);
    assert_eq!(presented.title(), "notification.unknown.title");
    assert_eq!(presented.message(), "notification.unknown.message");
    assert_eq!(presented.severity(), ToastSeverity::Warning);
    assert_eq!(presented.expires_at(), Duration::from_secs(3));
}

#[test]
fn toast_projection_captures_one_locale_across_a_mid_projection_transition() {
    let i18n = Arc::new(EditorI18nService::default());
    let center = ToastNotificationCenter::new(ToastCenterConfig::default());
    center
        .publish_at(
            ToastNotification::new(
                NotificationId::parse("editor.notification.project_saved.notice").unwrap(),
                NotificationSource::builtin("editor.play").unwrap(),
                ToastSeverity::Info,
                "editor.notification.project_saved.title",
                "editor.notification.project_saved.message",
                Duration::from_secs(3),
            )
            .unwrap(),
            Duration::ZERO,
        )
        .unwrap();
    let snapshot = center.snapshot_at(Duration::ZERO).pop().unwrap();
    let switching_service = Arc::clone(&i18n);
    i18n.configure_after_locale_capture_hook(Arc::new(move || {
        switching_service
            .set_active_locale(EditorLocale::parse("zh-CN").unwrap())
            .unwrap();
    }));

    let presented = present_toast(&snapshot, &i18n);
    assert_eq!(presented.locale().as_str(), "en");
    assert_eq!(presented.title(), "Project saved");
    assert_eq!(presented.message(), "Project state was written to disk.");
    assert_eq!(i18n.active_locale().as_str(), "zh-CN");
}

#[test]
fn progress_projection_keeps_the_authoritative_job_snapshot() {
    let i18n = EditorI18nService::default();
    let center = ProgressNotificationCenter::default();
    let job_id = JobId::new(17);
    center
        .publish(
            ProgressNotification::new(
                NotificationId::parse("editor.play.progress").unwrap(),
                NotificationSource::builtin("editor.play").unwrap(),
                job_id,
                "editor.job.play.progress",
            )
            .unwrap(),
        )
        .unwrap();
    let job = EditorJobProgressSnapshot::new(
        job_id,
        "Play startup",
        JobCategory::Play,
        Some(EditorJobProgress::new(1, 4, "Loading world")),
        true,
    );
    let snapshot = center.synchronize([job.clone()]).pop().unwrap();

    let presented = present_progress(&snapshot, &i18n);
    assert_eq!(presented.locale().as_str(), "en");
    assert_eq!(presented.title(), "Starting Play");
    assert_eq!(presented.job(), &job);
}
