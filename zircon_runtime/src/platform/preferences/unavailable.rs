use crate::core::framework::platform::{
    PreferenceKey, PreferenceStorageBackendKind, PreferenceStorageError,
    PreferenceStorageErrorKind, PreferenceStorageOperation,
};

use super::{PreferenceBackendWorkAuthority, PreferenceStorageBackend};

const BACKEND_NAME: &str = "unavailable";
const UNAVAILABLE_MESSAGE: &str = "host did not install a persistent preference backend";

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct UnavailablePreferenceStorageBackend;

impl PreferenceStorageBackend for UnavailablePreferenceStorageBackend {
    fn backend_kind(&self) -> PreferenceStorageBackendKind {
        PreferenceStorageBackendKind::Unavailable
    }

    fn open_read(
        &self,
        _authority: &PreferenceBackendWorkAuthority,
        _key: &PreferenceKey,
    ) -> Result<Option<Box<dyn std::io::Read + Send>>, PreferenceStorageError> {
        Err(unavailable(PreferenceStorageOperation::Read))
    }

    fn write(
        &self,
        _authority: &PreferenceBackendWorkAuthority,
        _key: &PreferenceKey,
        _value: &[u8],
    ) -> Result<(), PreferenceStorageError> {
        Err(unavailable(PreferenceStorageOperation::Write))
    }

    fn remove(
        &self,
        _authority: &PreferenceBackendWorkAuthority,
        _key: &PreferenceKey,
    ) -> Result<(), PreferenceStorageError> {
        Err(unavailable(PreferenceStorageOperation::Remove))
    }

    fn flush(
        &self,
        _authority: &PreferenceBackendWorkAuthority,
    ) -> Result<(), PreferenceStorageError> {
        Err(unavailable(PreferenceStorageOperation::Flush))
    }
}

fn unavailable(operation: PreferenceStorageOperation) -> PreferenceStorageError {
    PreferenceStorageError::new(
        PreferenceStorageErrorKind::Unavailable,
        operation,
        BACKEND_NAME,
        UNAVAILABLE_MESSAGE,
    )
}
