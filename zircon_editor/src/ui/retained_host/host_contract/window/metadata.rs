use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime_interface::ui::dispatch::{
    UiInputEventMetadata, UiInputSequence, UiInputTimestamp, UiWindowId,
};

use super::constants::NATIVE_HOST_WINDOW_ID;
use crate::ui::retained_host::primitives::PlatformError;

pub(in crate::ui::retained_host::host_contract) fn native_input_metadata(
    sequence: u64,
) -> UiInputEventMetadata {
    let micros = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_micros().min(u128::from(u64::MAX)) as u64)
        .unwrap_or_default();
    let mut metadata = UiInputEventMetadata::new(
        UiInputTimestamp::from_micros(micros),
        UiInputSequence::new(sequence),
    );
    metadata.window_id = Some(UiWindowId::new(NATIVE_HOST_WINDOW_ID));
    metadata
}

pub(in crate::ui::retained_host::host_contract) fn platform_error(
    error: impl std::fmt::Display,
) -> PlatformError {
    PlatformError::Other(error.to_string())
}
