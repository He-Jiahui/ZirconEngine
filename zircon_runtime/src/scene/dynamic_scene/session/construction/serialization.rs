use std::io::Write;

use super::super::archive::RuntimeSessionArchiveWirePayload;
use super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, MAX_RUNTIME_SESSION_ARCHIVE_ARTIFACT_BYTES,
};

pub(in crate::scene::dynamic_scene::session) fn from_versioned_json(
    json: &str,
) -> Result<RuntimeSessionArchive, RuntimeSessionArchiveError> {
    ensure_archive_input_limit(json.len(), MAX_RUNTIME_SESSION_ARCHIVE_ARTIFACT_BYTES)?;
    let payload: RuntimeSessionArchiveWirePayload = serde_json::from_str(json)?;
    let mut archive = RuntimeSessionArchive::from_deserialized_payload(payload.into());
    archive.normalize_slot_metadata();
    archive.record_normalized();
    archive.ensure_supported()?;
    archive.record_validated();
    Ok(archive)
}

fn ensure_archive_input_limit(
    found: usize,
    limit: usize,
) -> Result<(), RuntimeSessionArchiveError> {
    if found > limit {
        return Err(RuntimeSessionArchiveError::ArtifactTooLarge {
            estimated_bytes: found,
            limit_bytes: limit,
        });
    }
    Ok(())
}

pub(in crate::scene::dynamic_scene::session) fn to_versioned_json_pretty(
    archive: &RuntimeSessionArchive,
) -> Result<String, RuntimeSessionArchiveError> {
    let artifact = archive.sealed_artifact()?;
    String::from_utf8(artifact.serialized_bytes().to_vec())
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error).into())
}

pub(in crate::scene::dynamic_scene::session) fn to_versioned_json_pretty_to<W>(
    archive: &RuntimeSessionArchive,
    sink: &mut W,
) -> Result<usize, RuntimeSessionArchiveError>
where
    W: Write + ?Sized,
{
    archive.sealed_artifact()?.write_to(sink)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_session_archive_text_input_rejects_before_json_decode_when_oversized() {
        let error = ensure_archive_input_limit(2, 1).unwrap_err();

        assert!(matches!(
            error,
            RuntimeSessionArchiveError::ArtifactTooLarge {
                estimated_bytes: 2,
                limit_bytes: 1,
            }
        ));
    }
}
