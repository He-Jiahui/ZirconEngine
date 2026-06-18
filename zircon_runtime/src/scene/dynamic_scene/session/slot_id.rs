use super::RuntimeSessionArchiveError;

pub(super) fn normalize_slot_id(slot_id: String) -> Result<String, RuntimeSessionArchiveError> {
    let slot_id = slot_id.trim().to_string();
    validate_slot_id(&slot_id)?;
    Ok(slot_id)
}

pub(super) fn validate_canonical_slot_id(slot_id: &str) -> Result<(), RuntimeSessionArchiveError> {
    validate_slot_id(slot_id)?;
    let canonical = slot_id.trim();
    if canonical != slot_id {
        return Err(RuntimeSessionArchiveError::NonCanonicalSlotId {
            slot_id: slot_id.to_string(),
            canonical: canonical.to_string(),
        });
    }
    Ok(())
}

fn validate_slot_id(slot_id: &str) -> Result<(), RuntimeSessionArchiveError> {
    if slot_id.trim().is_empty() {
        return Err(RuntimeSessionArchiveError::EmptySlotId);
    }
    Ok(())
}
