use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime_interface::ui::dispatch::{
    UiInputEventMetadata, UiInputSequence, UiInputTimestamp, UiWindowId,
};

use super::constants::NATIVE_HOST_WINDOW_ID;
use crate::ui::retained_host::primitives::PlatformError;

pub(in crate::ui::retained_host::host_contract) fn native_input_metadata(
    sequence: u64,
) -> UiInputEventMetadata {
    let mut metadata = native_input_metadata_without_window_id(sequence);
    attach_native_window_id(&mut metadata);
    metadata
}

pub(in crate::ui::retained_host::host_contract) fn native_input_metadata_without_window_id(
    sequence: u64,
) -> UiInputEventMetadata {
    let micros = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_micros().min(u128::from(u64::MAX)) as u64)
        .unwrap_or_default();
    UiInputEventMetadata::new(
        UiInputTimestamp::from_micros(micros),
        UiInputSequence::new(sequence),
    )
}

pub(in crate::ui::retained_host::host_contract) fn attach_native_window_id(
    metadata: &mut UiInputEventMetadata,
) {
    metadata.window_id = Some(UiWindowId::new(NATIVE_HOST_WINDOW_ID));
}

pub(in crate::ui::retained_host::host_contract) fn platform_error(
    error: impl std::fmt::Display,
) -> PlatformError {
    PlatformError::Other(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reserved_input_metadata_preserves_arrival_identity_when_window_id_is_attached() {
        let mut metadata = native_input_metadata_without_window_id(41);
        let timestamp = metadata.timestamp;
        let sequence = metadata.sequence;

        assert!(metadata.window_id.is_none());
        attach_native_window_id(&mut metadata);

        assert_eq!(metadata.timestamp, timestamp);
        assert_eq!(metadata.sequence, sequence);
        assert_eq!(
            metadata.window_id,
            Some(UiWindowId::new(NATIVE_HOST_WINDOW_ID))
        );
    }
}
