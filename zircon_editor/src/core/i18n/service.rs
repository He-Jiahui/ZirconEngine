use std::collections::VecDeque;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::core::settings::SettingsAuthority;

use super::{EditorI18nCatalog, EditorI18nError, EditorLocale, EditorLocalizationBundle};

pub(super) const MAX_PENDING_LOCALE_EVENTS: usize = 32;
pub(super) const MAX_PENDING_LOCALE_EVENT_BYTES: usize = 64;

pub struct EditorI18nService {
    catalog: EditorI18nCatalog,
    embedded_bundle_error: Option<Arc<str>>,
    transition: Mutex<LocaleTransitionState>,
    event_sink: Mutex<Option<Arc<dyn EditorI18nEventSink>>>,
    event_dispatch: Mutex<LocaleEventDispatchState>,
    #[cfg(test)]
    before_event_dispatch_hook: Mutex<Option<Arc<dyn Fn() + Send + Sync>>>,
    #[cfg(test)]
    after_failure_locale_read_hook: Mutex<Option<Arc<dyn Fn() + Send + Sync>>>,
    #[cfg(test)]
    after_locale_capture_hook: Mutex<Option<Arc<dyn Fn() + Send + Sync>>>,
}

#[derive(Default)]
struct LocaleTransitionState {
    latest_settings_generation: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LocaleChangeDelivery {
    NotConfigured,
    Delivered,
    Backpressured,
    Rejected,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EditorI18nEventDiagnostics {
    pub queued_events: usize,
    pub queued_bytes: usize,
    pub dropped_events: u64,
    pub resyncs: u64,
    pub failed_resyncs: u64,
}

pub trait EditorI18nEventSink: Send + Sync {
    fn locale_changed(&self, locale: &EditorLocale) -> LocaleChangeDelivery;

    /// Locale events were coalesced while this sink was slow. Rebuild from the supplied
    /// active locale rather than assuming every intermediate transition was delivered.
    fn locale_resync_required(&self, locale: &EditorLocale) -> LocaleChangeDelivery;
}

#[derive(Default)]
struct LocaleEventDispatchState {
    pending: VecDeque<LocaleEventDispatch>,
    pending_bytes: usize,
    resync: Option<LocaleEventResync>,
    dropped_events: u64,
    resyncs: u64,
    failed_resyncs: u64,
    dispatching: bool,
}

struct LocaleEventDispatch {
    locale: EditorLocale,
    sink: Arc<dyn EditorI18nEventSink>,
    estimated_bytes: usize,
}

struct LocaleEventResync {
    locale: EditorLocale,
    sink: Arc<dyn EditorI18nEventSink>,
}

enum LocaleEventQueueOutcome {
    Enqueued { dispatch_now: bool },
    Backpressured { dispatch_now: bool },
}

enum PendingLocaleDispatch {
    Change(LocaleEventDispatch),
    Resync(LocaleEventResync),
}

impl Default for EditorI18nService {
    fn default() -> Self {
        match EditorI18nCatalog::embedded() {
            Ok(catalog) => Self {
                catalog,
                embedded_bundle_error: None,
                transition: Mutex::new(LocaleTransitionState::default()),
                event_sink: Mutex::new(None),
                event_dispatch: Mutex::new(LocaleEventDispatchState::default()),
                #[cfg(test)]
                before_event_dispatch_hook: Mutex::new(None),
                #[cfg(test)]
                after_failure_locale_read_hook: Mutex::new(None),
                #[cfg(test)]
                after_locale_capture_hook: Mutex::new(None),
            },
            Err(error) => Self {
                catalog: EditorI18nCatalog::english_fallback(),
                embedded_bundle_error: Some(Arc::from(error.to_string())),
                transition: Mutex::new(LocaleTransitionState::default()),
                event_sink: Mutex::new(None),
                event_dispatch: Mutex::new(LocaleEventDispatchState::default()),
                #[cfg(test)]
                before_event_dispatch_hook: Mutex::new(None),
                #[cfg(test)]
                after_failure_locale_read_hook: Mutex::new(None),
                #[cfg(test)]
                after_locale_capture_hook: Mutex::new(None),
            },
        }
    }
}

impl EditorI18nService {
    pub fn active_locale(&self) -> EditorLocale {
        self.catalog.active_locale()
    }

    pub fn available_locales(&self) -> Vec<EditorLocale> {
        self.catalog.available_locales()
    }

    pub(crate) fn set_active_locale(&self, locale: EditorLocale) -> Result<bool, EditorI18nError> {
        self.apply_locale(locale, None)
    }

    /// Reads the User-owned locale from the sole settings authority; this service never caches
    /// or persists another locale preference.
    pub fn synchronize_user_locale(
        &self,
        settings: &SettingsAuthority,
    ) -> Result<bool, EditorI18nError> {
        let snapshot = settings.snapshot();
        self.synchronize_settings_snapshot(snapshot.as_ref())
    }

    pub(crate) fn synchronize_settings_snapshot(
        &self,
        snapshot: &crate::core::settings::SettingsSnapshot,
    ) -> Result<bool, EditorI18nError> {
        let locale = EditorLocale::parse(snapshot.locale())?;
        #[cfg(test)]
        self.run_after_locale_capture_hook();
        self.apply_locale(locale, Some(snapshot.generation()))
    }

    fn apply_locale(
        &self,
        locale: EditorLocale,
        settings_generation: Option<u64>,
    ) -> Result<bool, EditorI18nError> {
        let should_dispatch = {
            let mut transition = self.lock_transition();
            if let Some(settings_generation) = settings_generation {
                if transition
                    .latest_settings_generation
                    .is_some_and(|latest| latest >= settings_generation)
                {
                    return Ok(false);
                }
            } else {
                transition.latest_settings_generation = None;
            }
            let changed = self.catalog.set_active_locale(locale)?;
            if let Some(settings_generation) = settings_generation {
                transition.latest_settings_generation = Some(settings_generation);
            }
            if !changed {
                return Ok(false);
            }
            let locale = self.catalog.active_locale();
            self.lock_event_sink()
                .clone()
                .map(|sink| self.enqueue_locale_event(locale, sink))
        };
        if matches!(
            should_dispatch,
            Some(LocaleEventQueueOutcome::Enqueued { dispatch_now: true })
                | Some(LocaleEventQueueOutcome::Backpressured { dispatch_now: true })
        ) {
            self.dispatch_pending_events();
        }
        Ok(true)
    }

    pub fn translate(&self, key: &str) -> Arc<str> {
        self.catalog.translate(key)
    }

    pub fn translate_for_locale(&self, locale: &EditorLocale, key: &str) -> Arc<str> {
        self.catalog.translate_for_locale(locale, key)
    }

    /// Resolves a ticket-owned plugin bundle through the canonical locale fallback chain.
    pub fn translate_bundle_for_locale(
        &self,
        bundle: &EditorLocalizationBundle,
        locale: &EditorLocale,
        key: &str,
    ) -> Arc<str> {
        bundle
            .translation(locale, key)
            .or_else(|| {
                (locale.as_str() != EditorLocale::english_tag())
                    .then(|| bundle.translation_for_locale_tag(EditorLocale::english_tag(), key))
                    .flatten()
            })
            .unwrap_or_else(|| Arc::from(key))
    }

    pub fn embedded_bundle_error(&self) -> Option<&str> {
        self.embedded_bundle_error.as_deref()
    }

    pub fn event_diagnostics(&self) -> EditorI18nEventDiagnostics {
        let state = self.lock_event_dispatch();
        EditorI18nEventDiagnostics {
            queued_events: state.pending.len(),
            queued_bytes: state.pending_bytes,
            dropped_events: state.dropped_events,
            resyncs: state.resyncs,
            failed_resyncs: state.failed_resyncs,
        }
    }

    pub fn configure_event_sink(&self, sink: Arc<dyn EditorI18nEventSink>) {
        let _transition = self.lock_transition();
        *self.lock_event_sink() = Some(sink);
    }

    fn lock_event_sink(&self) -> MutexGuard<'_, Option<Arc<dyn EditorI18nEventSink>>> {
        self.event_sink
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn enqueue_locale_event(
        &self,
        locale: EditorLocale,
        sink: Arc<dyn EditorI18nEventSink>,
    ) -> LocaleEventQueueOutcome {
        let estimated_bytes = locale.as_str().len();
        let mut state = self.lock_event_dispatch();
        if let Some(resync) = state.resync.as_mut() {
            resync.locale = locale;
            resync.sink = sink;
            state.dropped_events = state.dropped_events.saturating_add(1);
            let dispatch_now = !state.dispatching;
            if dispatch_now {
                state.dispatching = true;
            }
            return LocaleEventQueueOutcome::Backpressured { dispatch_now };
        }
        let queue_is_full = state.pending.len() >= MAX_PENDING_LOCALE_EVENTS
            || estimated_bytes > MAX_PENDING_LOCALE_EVENT_BYTES
            || state.pending_bytes > MAX_PENDING_LOCALE_EVENT_BYTES - estimated_bytes;
        if queue_is_full {
            state.resync = Some(LocaleEventResync { locale, sink });
            state.dropped_events = state.dropped_events.saturating_add(1);
            let dispatch_now = !state.dispatching;
            if dispatch_now {
                state.dispatching = true;
            }
            return LocaleEventQueueOutcome::Backpressured { dispatch_now };
        }
        state.pending.push_back(LocaleEventDispatch {
            locale,
            sink,
            estimated_bytes,
        });
        state.pending_bytes += estimated_bytes;
        let dispatch_now = !state.dispatching;
        if dispatch_now {
            state.dispatching = true;
        }
        LocaleEventQueueOutcome::Enqueued { dispatch_now }
    }

    fn dispatch_pending_events(&self) {
        #[cfg(test)]
        self.run_before_event_dispatch_hook();

        loop {
            let dispatch = {
                let mut state = self.lock_event_dispatch();
                if let Some(dispatch) = state.pending.pop_front() {
                    state.pending_bytes -= dispatch.estimated_bytes;
                    Some(PendingLocaleDispatch::Change(dispatch))
                } else if let Some(resync) = state.resync.take() {
                    Some(PendingLocaleDispatch::Resync(resync))
                } else {
                    state.dispatching = false;
                    None
                }
            };
            let Some(dispatch) = dispatch else {
                return;
            };
            match dispatch {
                PendingLocaleDispatch::Change(dispatch) => {
                    let delivery = dispatch.sink.locale_changed(&dispatch.locale);
                    if locale_delivery_requires_resync(delivery) {
                        self.absorb_locale_delivery_failure(dispatch.sink);
                    }
                }
                PendingLocaleDispatch::Resync(resync) => {
                    let LocaleEventResync { locale, sink } = resync;
                    let delivery = sink.locale_resync_required(&locale);
                    if locale_delivery_requires_resync(delivery) {
                        self.retry_locale_resync(LocaleEventResync { locale, sink });
                        return;
                    }
                    self.note_locale_resync_delivered();
                }
            }
        }
    }

    fn absorb_locale_delivery_failure(&self, sink: Arc<dyn EditorI18nEventSink>) {
        let _transition = self.lock_transition();
        let locale = self.catalog.active_locale();
        #[cfg(test)]
        self.run_after_failure_locale_read_hook();
        let mut state = self.lock_event_dispatch();
        let mut newly_dropped_events = 1_u64;
        while let Some(dispatch) = state.pending.pop_front() {
            state.pending_bytes = state.pending_bytes.saturating_sub(dispatch.estimated_bytes);
            newly_dropped_events = newly_dropped_events.saturating_add(1);
        }
        state.resync = Some(LocaleEventResync { locale, sink });
        state.dropped_events = state.dropped_events.saturating_add(newly_dropped_events);
    }

    fn retry_locale_resync(&self, resync: LocaleEventResync) {
        let _transition = self.lock_transition();
        let locale = self.catalog.active_locale();
        #[cfg(test)]
        self.run_after_failure_locale_read_hook();
        let mut state = self.lock_event_dispatch();
        let mut newly_dropped_events = 0_u64;
        while let Some(dispatch) = state.pending.pop_front() {
            state.pending_bytes = state.pending_bytes.saturating_sub(dispatch.estimated_bytes);
            newly_dropped_events = newly_dropped_events.saturating_add(1);
        }
        state.resync = Some(LocaleEventResync {
            locale,
            sink: resync.sink,
        });
        state.dropped_events = state.dropped_events.saturating_add(newly_dropped_events);
        state.failed_resyncs = state.failed_resyncs.saturating_add(1);
        state.dispatching = false;
    }

    fn note_locale_resync_delivered(&self) {
        let mut state = self.lock_event_dispatch();
        state.resyncs = state.resyncs.saturating_add(1);
    }

    fn lock_event_dispatch(&self) -> MutexGuard<'_, LocaleEventDispatchState> {
        self.event_dispatch
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn lock_transition(&self) -> MutexGuard<'_, LocaleTransitionState> {
        self.transition
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    #[cfg(test)]
    pub(super) fn configure_before_event_dispatch_hook(&self, hook: Arc<dyn Fn() + Send + Sync>) {
        *self
            .before_event_dispatch_hook
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(hook);
    }

    #[cfg(test)]
    pub(super) fn configure_after_failure_locale_read_hook(
        &self,
        hook: Arc<dyn Fn() + Send + Sync>,
    ) {
        *self
            .after_failure_locale_read_hook
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(hook);
    }

    #[cfg(test)]
    pub(crate) fn configure_after_locale_capture_hook(&self, hook: Arc<dyn Fn() + Send + Sync>) {
        *self
            .after_locale_capture_hook
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(hook);
    }

    #[cfg(test)]
    fn run_before_event_dispatch_hook(&self) {
        if let Some(hook) = self
            .before_event_dispatch_hook
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
        {
            hook();
        }
    }

    #[cfg(test)]
    fn run_after_failure_locale_read_hook(&self) {
        if let Some(hook) = self
            .after_failure_locale_read_hook
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
        {
            hook();
        }
    }

    #[cfg(test)]
    pub(crate) fn run_after_locale_capture_hook(&self) {
        if let Some(hook) = self
            .after_locale_capture_hook
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
        {
            hook();
        }
    }
}

fn locale_delivery_requires_resync(delivery: LocaleChangeDelivery) -> bool {
    matches!(
        delivery,
        LocaleChangeDelivery::Backpressured | LocaleChangeDelivery::Rejected
    )
}
