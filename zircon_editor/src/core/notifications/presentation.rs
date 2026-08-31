use std::fmt::Write as _;
use std::sync::Arc;
use std::time::Duration;

use crate::core::i18n::{EditorI18nService, EditorLocale};
use crate::core::jobs::EditorJobProgressSnapshot;

use super::{
    DecisionNotificationSnapshot, DecisionOptionId, DecisionReceipt, DecisionTicket,
    NotificationId, NotificationSource, ProgressNotificationSnapshot, ToastNotificationSnapshot,
    ToastSeverity,
};

/// Localized display data for a decision option. The option id remains the sole action identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalizedDecisionOption {
    id: DecisionOptionId,
    label: Arc<str>,
}

impl LocalizedDecisionOption {
    pub fn id(&self) -> &DecisionOptionId {
        &self.id
    }

    pub fn label(&self) -> &str {
        &self.label
    }
}

/// Read-only display projection for a decision. It does not own or resolve the decision state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalizedDecisionNotification {
    locale: EditorLocale,
    ticket: DecisionTicket,
    id: NotificationId,
    source: NotificationSource,
    title: Arc<str>,
    message: Arc<str>,
    display_subject: Option<Arc<str>>,
    options: Vec<LocalizedDecisionOption>,
    default_option: Option<DecisionOptionId>,
    cancel_option: Option<DecisionOptionId>,
    resolved: Option<DecisionReceipt>,
}

impl LocalizedDecisionNotification {
    pub fn locale(&self) -> &EditorLocale {
        &self.locale
    }

    pub fn ticket(&self) -> &DecisionTicket {
        &self.ticket
    }

    pub fn id(&self) -> &NotificationId {
        &self.id
    }

    pub fn source(&self) -> &NotificationSource {
        &self.source
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    /// Bounded, non-localized operator context supplied by the Decision producer.
    pub fn display_subject(&self) -> Option<&str> {
        self.display_subject.as_deref()
    }

    pub fn options(&self) -> &[LocalizedDecisionOption] {
        &self.options
    }

    pub fn default_option(&self) -> Option<&DecisionOptionId> {
        self.default_option.as_ref()
    }

    pub fn cancel_option(&self) -> Option<&DecisionOptionId> {
        self.cancel_option.as_ref()
    }

    pub fn resolved(&self) -> Option<&DecisionReceipt> {
        self.resolved.as_ref()
    }
}

/// Read-only display projection for an expiring toast.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalizedToastNotification {
    locale: EditorLocale,
    id: NotificationId,
    source: NotificationSource,
    severity: ToastSeverity,
    title: Arc<str>,
    message: Arc<str>,
    expires_at: Duration,
}

impl LocalizedToastNotification {
    pub fn locale(&self) -> &EditorLocale {
        &self.locale
    }

    pub fn id(&self) -> &NotificationId {
        &self.id
    }

    pub fn source(&self) -> &NotificationSource {
        &self.source
    }

    pub const fn severity(&self) -> ToastSeverity {
        self.severity
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub const fn expires_at(&self) -> Duration {
        self.expires_at
    }
}

/// Read-only display projection for progress associated with one editor job.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalizedProgressNotification {
    locale: EditorLocale,
    id: NotificationId,
    source: NotificationSource,
    title: Arc<str>,
    job: EditorJobProgressSnapshot,
}

impl LocalizedProgressNotification {
    pub fn locale(&self) -> &EditorLocale {
        &self.locale
    }

    pub fn id(&self) -> &NotificationId {
        &self.id
    }

    pub fn source(&self) -> &NotificationSource {
        &self.source
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn job(&self) -> &EditorJobProgressSnapshot {
        &self.job
    }
}

/// Resolves display text at the edge while preserving the decision ticket and option ids.
pub fn present_decision(
    snapshot: &DecisionNotificationSnapshot,
    i18n: &EditorI18nService,
) -> LocalizedDecisionNotification {
    let locale = captured_locale(i18n);
    let notification = snapshot.notification();
    LocalizedDecisionNotification {
        locale: locale.clone(),
        ticket: snapshot.ticket().clone(),
        id: notification.id().clone(),
        source: notification.source().clone(),
        title: i18n.translate_for_locale(&locale, notification.title_key()),
        message: format_decision_message(i18n, &locale, notification),
        display_subject: notification.display_subject().map(Arc::from),
        options: notification
            .options()
            .iter()
            .map(|option| LocalizedDecisionOption {
                id: option.id().clone(),
                label: i18n.translate_for_locale(&locale, option.label_key()),
            })
            .collect(),
        default_option: notification.default_option().cloned(),
        cancel_option: notification.cancel_option().cloned(),
        resolved: snapshot.resolved().cloned(),
    }
}

fn format_decision_message(
    i18n: &EditorI18nService,
    locale: &EditorLocale,
    notification: &super::DecisionNotification,
) -> Arc<str> {
    format_decision_message_template(
        i18n.translate_for_locale(locale, notification.message_key()),
        |name| {
            notification
                .message_arguments()
                .find_map(|(argument_name, value)| (argument_name == name).then_some(value))
        },
    )
}

fn format_decision_message_template(
    template: Arc<str>,
    mut argument_value: impl FnMut(&str) -> Option<u64>,
) -> Arc<str> {
    let mut output = None::<String>;
    let mut copied_until = 0;
    let mut scan_from = 0;
    while let Some(relative_open) = template[scan_from..].find('{') {
        let open = scan_from + relative_open;
        let name_start = open + 1;
        let Some(relative_close) = template[name_start..].find('}') else {
            break;
        };
        let close = name_start + relative_close;
        let name = &template[name_start..close];
        if let Some(value) = argument_value(name) {
            let output = output
                .get_or_insert_with(|| String::with_capacity(template.len().saturating_add(20)));
            output.push_str(&template[copied_until..open]);
            write!(output, "{value}").expect("writing to String cannot fail");
            copied_until = close + 1;
            scan_from = close + 1;
        } else {
            // An unknown outer brace may still contain a known placeholder, as in
            // `{{count}}`; keep looking from the byte after this opening brace.
            scan_from = name_start;
        }
    }

    let Some(mut output) = output else {
        return template;
    };
    output.push_str(&template[copied_until..]);
    Arc::from(output)
}

/// Resolves toast text without changing expiry, severity, or notification identity.
pub fn present_toast(
    snapshot: &ToastNotificationSnapshot,
    i18n: &EditorI18nService,
) -> LocalizedToastNotification {
    let locale = captured_locale(i18n);
    let notification = snapshot.notification();
    LocalizedToastNotification {
        locale: locale.clone(),
        id: notification.id().clone(),
        source: notification.source().clone(),
        severity: notification.severity(),
        title: i18n.translate_for_locale(&locale, notification.title_key()),
        message: i18n.translate_for_locale(&locale, notification.message_key()),
        expires_at: snapshot.expires_at(),
    }
}

/// Resolves progress text without changing its bound job snapshot.
pub fn present_progress(
    snapshot: &ProgressNotificationSnapshot,
    i18n: &EditorI18nService,
) -> LocalizedProgressNotification {
    let locale = captured_locale(i18n);
    let notification = snapshot.notification();
    LocalizedProgressNotification {
        locale: locale.clone(),
        id: notification.id().clone(),
        source: notification.source().clone(),
        title: i18n.translate_for_locale(&locale, notification.title_key()),
        job: snapshot.job().clone(),
    }
}

fn captured_locale(i18n: &EditorI18nService) -> EditorLocale {
    let locale = i18n.active_locale();
    #[cfg(test)]
    i18n.run_after_locale_capture_hook();
    locale
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::sync::Arc;
    use std::time::Instant;

    use super::format_decision_message_template;

    #[test]
    fn decision_message_template_reuses_unmodified_translation_storage() {
        let without_arguments: Arc<str> = Arc::from("No pending work.");
        let projected = format_decision_message_template(Arc::clone(&without_arguments), |_| None);
        assert!(Arc::ptr_eq(&without_arguments, &projected));

        let without_matching_placeholder: Arc<str> = Arc::from("Keep {unknown} intact.");
        let projected =
            format_decision_message_template(Arc::clone(&without_matching_placeholder), |name| {
                (name == "pending_count").then_some(2)
            });
        assert!(Arc::ptr_eq(&without_matching_placeholder, &projected));
    }

    #[test]
    fn decision_message_template_formats_repeated_values_and_preserves_unknown_placeholders() {
        let projected = format_decision_message_template(
            Arc::from("{count} queued, {count} total; keep {unknown}; nested {{count}}."),
            |name| (name == "count").then_some(42),
        );

        assert_eq!(
            projected.as_ref(),
            "42 queued, 42 total; keep {unknown}; nested {42}."
        );
    }

    #[test]
    #[ignore = "managed release performance evidence"]
    fn optimization_wave_20260825_editor10_message_template_evidence() {
        const PROJECTIONS: usize = 100_000;
        const ARGUMENT_COUNT: usize = 8;
        const MAX_ELAPSED_NS: u128 = 3_000_000_000;
        const ARGUMENTS: [(&str, u64); ARGUMENT_COUNT] = [
            ("one", 1),
            ("two", 2),
            ("three", 3),
            ("four", 4),
            ("five", 5),
            ("six", 6),
            ("seven", 7),
            ("eight", 8),
        ];

        let template: Arc<str> =
            Arc::from("{one}/{two}/{three}/{four}/{five}/{six}/{seven}/{eight}: {unknown}");
        let started = Instant::now();
        for _ in 0..PROJECTIONS {
            black_box(format_decision_message_template(
                Arc::clone(&template),
                |name| {
                    ARGUMENTS.iter().find_map(|(argument_name, value)| {
                        (*argument_name == name).then_some(*value)
                    })
                },
            ));
        }
        let elapsed_ns = started.elapsed().as_nanos();
        let legacy_full_template_passes = PROJECTIONS * ARGUMENT_COUNT;
        let optimized_template_passes = PROJECTIONS;
        let pass_reduction_bps = legacy_full_template_passes
            .saturating_sub(optimized_template_passes)
            .saturating_mul(10_000)
            / legacy_full_template_passes;

        println!(
            "EDITOR_DECISION_MESSAGE_FORMAT_BENCH_V1 projections={PROJECTIONS} arguments={ARGUMENT_COUNT} legacy_full_template_passes={legacy_full_template_passes} optimized_template_passes={optimized_template_passes} pass_reduction_bps={pass_reduction_bps} elapsed_ns={elapsed_ns} max_elapsed_ns={MAX_ELAPSED_NS}"
        );

        assert_eq!(pass_reduction_bps, 8_750);
        assert!(elapsed_ns <= MAX_ELAPSED_NS);
    }
}
