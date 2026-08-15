use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Barrier, Mutex};
use std::thread;

use super::{
    EditorI18nCatalog, EditorI18nError, EditorI18nEventSink, EditorI18nService, EditorLocale,
    LocaleChangeDelivery, MAX_PENDING_LOCALE_EVENTS, MAX_PENDING_LOCALE_EVENT_BYTES,
};
use crate::core::settings::{
    SettingValue, SettingsAuthority, SettingsKey, SettingsScope, EDITOR_LOCALE_KEY,
    VIEWPORT_TRANSLATE_STEP_KEY,
};

const ENGLISH: &str = r#"
locale = "en"

[translations]
"editor.greeting" = "Hello"
"editor.fallback" = "English fallback"
"#;

const SIMPLIFIED_CHINESE: &str = r#"
locale = "zh-CN"

[translations]
"editor.greeting" = "你好"
"#;

#[derive(Default)]
struct OrderedLocaleSink {
    locales: Mutex<Vec<String>>,
}

struct SaturatingLocaleSink {
    gate: Arc<Barrier>,
    blocked_first_delivery: AtomicBool,
    deliveries: Mutex<Vec<String>>,
}

struct RetryingResyncLocaleSink {
    resync_attempts: AtomicBool,
}

#[derive(Default)]
struct FailureOrderingLocaleSink {
    deliveries: Mutex<Vec<String>>,
    rejected_first_change: AtomicBool,
}

impl EditorI18nEventSink for OrderedLocaleSink {
    fn locale_changed(&self, locale: &EditorLocale) -> LocaleChangeDelivery {
        self.locales
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(locale.to_string());
        LocaleChangeDelivery::Delivered
    }

    fn locale_resync_required(&self, locale: &EditorLocale) -> LocaleChangeDelivery {
        self.locale_changed(locale)
    }
}

impl EditorI18nEventSink for SaturatingLocaleSink {
    fn locale_changed(&self, locale: &EditorLocale) -> LocaleChangeDelivery {
        self.deliveries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(format!("change:{locale}"));
        if !self.blocked_first_delivery.swap(true, Ordering::SeqCst) {
            self.gate.wait();
            self.gate.wait();
        }
        LocaleChangeDelivery::Delivered
    }

    fn locale_resync_required(&self, locale: &EditorLocale) -> LocaleChangeDelivery {
        self.deliveries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(format!("resync:{locale}"));
        LocaleChangeDelivery::Delivered
    }
}

impl EditorI18nEventSink for RetryingResyncLocaleSink {
    fn locale_changed(&self, _locale: &EditorLocale) -> LocaleChangeDelivery {
        LocaleChangeDelivery::Rejected
    }

    fn locale_resync_required(&self, _locale: &EditorLocale) -> LocaleChangeDelivery {
        if self.resync_attempts.swap(true, Ordering::SeqCst) {
            LocaleChangeDelivery::Delivered
        } else {
            LocaleChangeDelivery::Rejected
        }
    }
}

impl EditorI18nEventSink for FailureOrderingLocaleSink {
    fn locale_changed(&self, locale: &EditorLocale) -> LocaleChangeDelivery {
        self.deliveries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(format!("change:{locale}"));
        if !self.rejected_first_change.swap(true, Ordering::SeqCst) {
            LocaleChangeDelivery::Rejected
        } else {
            LocaleChangeDelivery::Delivered
        }
    }

    fn locale_resync_required(&self, locale: &EditorLocale) -> LocaleChangeDelivery {
        self.deliveries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(format!("resync:{locale}"));
        LocaleChangeDelivery::Delivered
    }
}

fn catalog() -> EditorI18nCatalog {
    EditorI18nCatalog::from_toml_bundles(&[ENGLISH, SIMPLIFIED_CHINESE]).unwrap()
}

#[test]
fn active_language_then_english_then_raw_key_is_the_translation_fallback_chain() {
    let catalog = catalog();
    catalog
        .set_active_locale(EditorLocale::parse("zh-CN").unwrap())
        .unwrap();

    assert_eq!(catalog.translate("editor.greeting").as_ref(), "你好");
    assert_eq!(
        catalog.translate("editor.fallback").as_ref(),
        "English fallback"
    );
    assert_eq!(
        catalog.translate("editor.missing").as_ref(),
        "editor.missing"
    );
}

#[test]
fn language_switch_is_explicit_and_only_accepts_loaded_bundles() {
    let catalog = catalog();

    assert!(catalog
        .set_active_locale(EditorLocale::parse("zh-cn").unwrap())
        .unwrap());
    assert_eq!(catalog.active_locale().as_str(), "zh-CN");
    assert!(!catalog
        .set_active_locale(EditorLocale::parse("zh-CN").unwrap())
        .unwrap());
    assert!(matches!(
        catalog.set_active_locale(EditorLocale::parse("fr").unwrap()),
        Err(EditorI18nError::UnavailableLocale(locale)) if locale == "fr"
    ));
}

#[test]
fn malformed_bundles_and_keys_are_rejected_at_the_catalog_boundary() {
    assert!(matches!(
        EditorI18nCatalog::from_toml_bundles(&["locale = \"en\"\n[translations]\n\"bad key\" = \"value\""]),
        Err(EditorI18nError::InvalidTranslationKey(key)) if key == "bad key"
    ));
    assert!(matches!(
        EditorI18nCatalog::from_toml_bundles(&["locale = \"zh-CN\"\n[translations]"]),
        Err(EditorI18nError::MissingEnglishFallback)
    ));
}

#[test]
fn tr_macro_uses_the_explicit_service_instead_of_global_state() {
    let service = EditorI18nService::default();

    assert_eq!(crate::tr!(&service, "command.file.open").as_ref(), "Open");
    assert!(service.embedded_bundle_error().is_none());
}

#[test]
fn user_locale_setting_hot_syncs_i18n_from_the_authority_snapshot() {
    let settings = SettingsAuthority::with_defaults();
    let service = EditorI18nService::default();
    let locale_key = SettingsKey::parse(EDITOR_LOCALE_KEY).unwrap();

    assert_eq!(
        service
            .available_locales()
            .into_iter()
            .map(|locale| locale.to_string())
            .collect::<Vec<_>>(),
        ["en", "zh-CN"]
    );
    assert_eq!(settings.snapshot().locale(), "en");
    assert!(!service.synchronize_user_locale(&settings).unwrap());

    let change = settings
        .set(
            SettingsScope::User,
            &locale_key,
            SettingValue::Enum("zh-CN".to_owned()),
        )
        .unwrap()
        .expect("the User locale should produce a hot settings change");
    assert!(!change.requires_restart);
    assert_eq!(settings.snapshot().locale(), "zh-CN");
    assert!(service.synchronize_user_locale(&settings).unwrap());
    assert_eq!(service.active_locale().as_str(), "zh-CN");
    assert!(!service.synchronize_user_locale(&settings).unwrap());

    assert!(settings
        .set(
            SettingsScope::User,
            &locale_key,
            SettingValue::Enum("fr".to_owned()),
        )
        .is_err());
    assert_eq!(settings.snapshot().locale(), "zh-CN");

    settings
        .set(
            SettingsScope::Project,
            &SettingsKey::parse(VIEWPORT_TRANSLATE_STEP_KEY).unwrap(),
            SettingValue::Float(2.0),
        )
        .unwrap();
    assert!(!service.synchronize_user_locale(&settings).unwrap());
    assert_eq!(service.active_locale().as_str(), "zh-CN");

    settings.clear(SettingsScope::User, &locale_key).unwrap();
    assert_eq!(settings.snapshot().locale(), "en");
    assert!(service.synchronize_user_locale(&settings).unwrap());
    assert_eq!(service.active_locale().as_str(), "en");
}

#[test]
fn user_locale_sync_rejects_a_late_snapshot_generation() {
    let settings = Arc::new(SettingsAuthority::with_defaults());
    let service = Arc::new(EditorI18nService::default());
    let locale_key = SettingsKey::parse(EDITOR_LOCALE_KEY).unwrap();
    settings
        .set(
            SettingsScope::User,
            &locale_key,
            SettingValue::Enum("zh-CN".to_owned()),
        )
        .unwrap();

    let captured = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let block_once = Arc::new(AtomicBool::new(false));
    let hook_captured = Arc::clone(&captured);
    let hook_release = Arc::clone(&release);
    let hook_block_once = Arc::clone(&block_once);
    service.configure_after_locale_capture_hook(Arc::new(move || {
        if !hook_block_once.swap(true, Ordering::SeqCst) {
            hook_captured.wait();
            hook_release.wait();
        }
    }));

    let first_service = Arc::clone(&service);
    let first_settings = Arc::clone(&settings);
    let late_sync = thread::spawn(move || first_service.synchronize_user_locale(&first_settings));
    captured.wait();

    settings
        .set(
            SettingsScope::User,
            &locale_key,
            SettingValue::Enum("en".to_owned()),
        )
        .unwrap();
    assert!(!service.synchronize_user_locale(&settings).unwrap());
    release.wait();

    assert!(!late_sync.join().unwrap().unwrap());
    assert_eq!(service.active_locale().as_str(), "en");
}

#[test]
fn locale_change_events_are_fifo_and_end_at_the_active_locale_under_concurrency() {
    let service = Arc::new(EditorI18nService::default());
    let sink = Arc::new(OrderedLocaleSink::default());
    let event_sink: Arc<dyn EditorI18nEventSink> = Arc::clone(&sink);
    service.configure_event_sink(event_sink);
    let gate = Arc::new(Barrier::new(2));
    let hook_gate = Arc::clone(&gate);
    service.configure_before_event_dispatch_hook(Arc::new(move || {
        hook_gate.wait();
        hook_gate.wait();
    }));
    let first_service = Arc::clone(&service);
    let first = thread::spawn(move || {
        first_service
            .set_active_locale(EditorLocale::parse("zh-CN").unwrap())
            .unwrap()
    });
    gate.wait();

    assert!(service
        .set_active_locale(EditorLocale::parse("en").unwrap())
        .unwrap());
    gate.wait();
    assert!(first.join().unwrap());
    assert_eq!(service.active_locale().as_str(), "en");
    assert_eq!(
        sink.locales
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_slice(),
        ["zh-CN", "en"]
    );
}

#[test]
fn slow_locale_sink_bounds_pending_events_and_coalesces_the_latest_locale_as_a_resync() {
    let service = Arc::new(EditorI18nService::default());
    let gate = Arc::new(Barrier::new(2));
    let sink = Arc::new(SaturatingLocaleSink {
        gate: Arc::clone(&gate),
        blocked_first_delivery: AtomicBool::new(false),
        deliveries: Mutex::new(Vec::new()),
    });
    let event_sink: Arc<dyn EditorI18nEventSink> = Arc::clone(&sink);
    service.configure_event_sink(event_sink);

    let first_service = Arc::clone(&service);
    let first = thread::spawn(move || {
        first_service
            .set_active_locale(EditorLocale::parse("zh-CN").unwrap())
            .unwrap()
    });
    gate.wait();

    for index in 0..=MAX_PENDING_LOCALE_EVENTS {
        let locale = if index % 2 == 0 { "en" } else { "zh-CN" };
        assert!(service
            .set_active_locale(EditorLocale::parse(locale).unwrap())
            .unwrap());
    }
    assert!(service.event_diagnostics().queued_events < MAX_PENDING_LOCALE_EVENTS);
    assert!(service.event_diagnostics().queued_bytes <= MAX_PENDING_LOCALE_EVENT_BYTES);
    assert!(service.event_diagnostics().dropped_events > 0);

    gate.wait();
    assert!(first.join().unwrap());
    let deliveries = sink
        .deliveries
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    assert!(deliveries.len() < MAX_PENDING_LOCALE_EVENTS + 2);
    assert_eq!(deliveries.last(), Some(&"resync:en".to_string()));
    assert_eq!(service.active_locale().as_str(), "en");
    let diagnostics = service.event_diagnostics();
    assert_eq!(diagnostics.queued_events, 0);
    assert!(diagnostics.dropped_events > 0);
    assert_eq!(diagnostics.resyncs, 1);
}

#[test]
fn rejected_locale_resync_is_retained_until_the_next_transition() {
    let service = EditorI18nService::default();
    service.configure_event_sink(Arc::new(RetryingResyncLocaleSink {
        resync_attempts: AtomicBool::new(false),
    }));

    assert!(service
        .set_active_locale(EditorLocale::parse("zh-CN").unwrap())
        .unwrap());
    assert_eq!(service.event_diagnostics().resyncs, 0);
    assert_eq!(service.event_diagnostics().failed_resyncs, 1);

    assert!(service
        .set_active_locale(EditorLocale::parse("en").unwrap())
        .unwrap());
    assert_eq!(service.event_diagnostics().resyncs, 1);
    assert_eq!(service.event_diagnostics().failed_resyncs, 1);
}

#[test]
fn failed_delivery_resync_is_linearized_before_a_concurrent_locale_transition() {
    let service = Arc::new(EditorI18nService::default());
    let sink = Arc::new(FailureOrderingLocaleSink::default());
    let event_sink: Arc<dyn EditorI18nEventSink> = Arc::clone(&sink);
    service.configure_event_sink(event_sink);
    let entered_failure = Arc::new(Barrier::new(2));
    let release_failure = Arc::new(Barrier::new(2));
    let entered_hook = Arc::clone(&entered_failure);
    let release_hook = Arc::clone(&release_failure);
    service.configure_after_failure_locale_read_hook(Arc::new(move || {
        entered_hook.wait();
        release_hook.wait();
    }));

    let first_service = Arc::clone(&service);
    let first = thread::spawn(move || {
        first_service
            .set_active_locale(EditorLocale::parse("zh-CN").unwrap())
            .unwrap()
    });
    entered_failure.wait();

    let second_started = Arc::new(Barrier::new(2));
    let second_start = Arc::clone(&second_started);
    let second_service = Arc::clone(&service);
    let second = thread::spawn(move || {
        second_start.wait();
        second_service
            .set_active_locale(EditorLocale::parse("en").unwrap())
            .unwrap()
    });
    second_started.wait();
    release_failure.wait();

    assert!(first.join().unwrap());
    assert!(second.join().unwrap());
    let deliveries = sink
        .deliveries
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    let delivery_text = deliveries.iter().map(String::as_str).collect::<Vec<_>>();
    assert!(matches!(
        delivery_text.as_slice(),
        ["change:zh-CN", "resync:zh-CN", "change:en"] | ["change:zh-CN", "resync:en"]
    ));
    assert_eq!(service.active_locale().as_str(), "en");
}
