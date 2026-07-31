use std::io::Read;
use std::sync::Arc;

use crate::core::framework::platform::{
    PreferenceKey, PreferencePersistenceFailureProjection, PreferenceStorageError,
    PreferenceStorageErrorKind, PreferenceStorageOperation,
};
use crate::core::runtime::BoundedKeyedIoFailure;
use crate::platform::preferences::PreferenceStorageBackend;

use super::MAX_PREFERENCE_FAILURE_DETAIL_BYTES;

/// Capability issued only inside the persistence worker.
///
/// ```compile_fail
/// use zircon_runtime::platform::preferences::PreferenceBackendWorkAuthority;
/// let _ = PreferenceBackendWorkAuthority { _private: () };
/// ```
pub struct PreferenceBackendWorkAuthority {
    _private: (),
}

impl PreferenceBackendWorkAuthority {
    fn new() -> Self {
        Self { _private: () }
    }
}

pub(super) fn perform_read(
    backend: &Arc<dyn PreferenceStorageBackend>,
    key: &PreferenceKey,
    max_value_bytes: usize,
) -> Result<Option<Arc<[u8]>>, PreferencePersistenceFailureProjection> {
    let authority = PreferenceBackendWorkAuthority::new();
    let Some(reader) = backend
        .open_read(&authority, key)
        .map_err(project_backend_error)?
    else {
        return Ok(None);
    };
    read_bounded(reader, max_value_bytes, backend.backend_kind().as_str()).map(Some)
}

pub(super) fn perform_write(
    backend: &Arc<dyn PreferenceStorageBackend>,
    key: &PreferenceKey,
    value: &[u8],
) -> Result<(), PreferencePersistenceFailureProjection> {
    backend
        .write(&PreferenceBackendWorkAuthority::new(), key, value)
        .map_err(project_backend_error)
}

pub(super) fn perform_remove(
    backend: &Arc<dyn PreferenceStorageBackend>,
    key: &PreferenceKey,
) -> Result<(), PreferencePersistenceFailureProjection> {
    backend
        .remove(&PreferenceBackendWorkAuthority::new(), key)
        .map_err(project_backend_error)
}

pub(super) fn perform_flush(
    backend: &Arc<dyn PreferenceStorageBackend>,
) -> Result<(), PreferencePersistenceFailureProjection> {
    backend
        .flush(&PreferenceBackendWorkAuthority::new())
        .map_err(project_backend_error)
}

pub(super) fn read_bounded(
    reader: Box<dyn Read + Send>,
    max_value_bytes: usize,
    backend: &'static str,
) -> Result<Arc<[u8]>, PreferencePersistenceFailureProjection> {
    let max_plus_one = max_value_bytes.checked_add(1).ok_or_else(|| {
        projection(
            PreferenceStorageErrorKind::CapacityExceeded,
            PreferenceStorageOperation::Read,
            backend,
            "configured preference value limit cannot reserve max+1 probe",
        )
    })?;
    let mut value = Vec::with_capacity(max_value_bytes.min(8 * 1024));
    reader
        .take(max_plus_one as u64)
        .read_to_end(&mut value)
        .map_err(|error| {
            projection(
                PreferenceStorageErrorKind::TransientIo,
                PreferenceStorageOperation::Read,
                backend,
                &error.to_string(),
            )
        })?;
    if value.len() > max_value_bytes {
        return Err(projection(
            PreferenceStorageErrorKind::CapacityExceeded,
            PreferenceStorageOperation::Read,
            backend,
            "persisted preference value exceeds configured maximum",
        ));
    }
    Ok(Arc::from(value))
}

pub(super) fn project_backend_error(
    error: PreferenceStorageError,
) -> PreferencePersistenceFailureProjection {
    projection(
        error.kind(),
        error.operation(),
        error.backend(),
        error.message(),
    )
}

pub(super) fn lane_failure(
    projection: &PreferencePersistenceFailureProjection,
) -> BoundedKeyedIoFailure {
    let code = match projection.kind() {
        PreferenceStorageErrorKind::Unavailable => "preference_backend_unavailable",
        PreferenceStorageErrorKind::Denied => "preference_backend_denied",
        PreferenceStorageErrorKind::CapacityExceeded => "preference_capacity_exceeded",
        PreferenceStorageErrorKind::CorruptBackend => "preference_backend_corrupt",
        PreferenceStorageErrorKind::TransientIo => "preference_backend_transient_io",
    };
    BoundedKeyedIoFailure::new(code)
}

fn projection(
    kind: PreferenceStorageErrorKind,
    operation: PreferenceStorageOperation,
    backend: &'static str,
    detail: &str,
) -> PreferencePersistenceFailureProjection {
    PreferencePersistenceFailureProjection::new(
        kind,
        operation,
        backend,
        truncate_utf8_detail(detail, MAX_PREFERENCE_FAILURE_DETAIL_BYTES),
    )
}

fn truncate_utf8_detail(value: &str, max_bytes: usize) -> String {
    if value.len() <= max_bytes {
        return value.to_owned();
    }
    let mut end = max_bytes.min(value.len());
    while !value.is_char_boundary(end) {
        end = end.saturating_sub(1);
    }
    value[..end].to_owned()
}
