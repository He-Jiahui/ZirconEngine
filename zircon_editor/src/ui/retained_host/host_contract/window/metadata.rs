use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime_interface::ui::dispatch::{
    UiInputEventMetadata, UiInputSequence, UiInputTimestamp, UiWindowId,
};

use super::constants::NATIVE_HOST_WINDOW_ID;
use super::UiHostWindow;
use crate::ui::retained_host::primitives::PlatformError;

pub(in crate::ui::retained_host::host_contract) fn native_input_metadata(
    ui: &UiHostWindow,
    sequence: u64,
) -> UiInputEventMetadata {
    let mut metadata = native_input_metadata_without_window_id(sequence);
    attach_native_window_id(ui, &mut metadata);
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
    ui: &UiHostWindow,
    metadata: &mut UiInputEventMetadata,
) {
    let state = ui.state.borrow();
    let shell = &state.host_presentation.host_shell;
    metadata.window_id = if shell.native_floating_window_mode {
        (!shell.native_floating_window_id.trim().is_empty())
            .then(|| UiWindowId::new(shell.native_floating_window_id.clone()))
    } else {
        Some(UiWindowId::new(NATIVE_HOST_WINDOW_ID))
    };
}

pub(in crate::ui::retained_host::host_contract) fn platform_error(
    error: impl std::fmt::Display,
) -> PlatformError {
    PlatformError::Other(error.to_string())
}

#[cfg(test)]
#[path = "tests/metadata_unit.rs"]
mod tests;
