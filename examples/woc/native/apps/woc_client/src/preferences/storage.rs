use std::sync::Arc;

use zircon_runtime::core::framework::platform::{
    PreferenceDurabilityState, PreferenceKey, PreferenceMutationSubmission, PreferenceStorage,
    PreferenceWorkDeadline,
};

const WOC_CLIENT_PREFERENCE_NAMESPACE: &str = "woc.client";

pub(crate) enum PreferenceRead<T> {
    Pending,
    Ready(T),
}

impl<T> PreferenceRead<T> {
    pub(crate) fn map<U>(self, map: impl FnOnce(T) -> U) -> PreferenceRead<U> {
        match self {
            Self::Pending => PreferenceRead::Pending,
            Self::Ready(value) => PreferenceRead::Ready(map(value)),
        }
    }

    pub(crate) fn into_ready(self) -> Option<T> {
        match self {
            Self::Pending => None,
            Self::Ready(value) => Some(value),
        }
    }
}

pub(crate) fn read_preference_text(
    storage: &dyn PreferenceStorage,
    key: &str,
) -> PreferenceRead<Option<String>> {
    let Ok(key) = PreferenceKey::new(WOC_CLIENT_PREFERENCE_NAMESPACE, key) else {
        return PreferenceRead::Ready(None);
    };
    let Ok(snapshot) = storage.snapshot(&key) else {
        return PreferenceRead::Ready(None);
    };
    if let Some(value) = snapshot.value() {
        return PreferenceRead::Ready(std::str::from_utf8(value).ok().map(str::to_owned));
    }
    if snapshot.durability() == PreferenceDurabilityState::Pending {
        PreferenceRead::Pending
    } else {
        PreferenceRead::Ready(None)
    }
}

pub(crate) fn submit_preference_text(
    storage: &dyn PreferenceStorage,
    key: &str,
    value: &str,
) -> Option<PreferenceMutationSubmission> {
    let key = PreferenceKey::new(WOC_CLIENT_PREFERENCE_NAMESPACE, key).ok()?;
    storage
        .submit_write(
            key,
            Arc::from(value.as_bytes()),
            PreferenceWorkDeadline::none(),
        )
        .ok()
}
