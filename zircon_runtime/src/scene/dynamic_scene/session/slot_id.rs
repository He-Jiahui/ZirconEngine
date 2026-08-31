use super::RuntimeSessionArchiveError;

pub(super) fn normalize_slot_id(mut slot_id: String) -> Result<String, RuntimeSessionArchiveError> {
    trim_slot_id_in_place(&mut slot_id);
    validate_slot_id(&slot_id)?;
    Ok(slot_id)
}

fn trim_slot_id_in_place(slot_id: &mut String) {
    let trimmed_end = slot_id.trim_end().len();
    slot_id.truncate(trimmed_end);

    let trimmed_start = slot_id.len() - slot_id.trim_start().len();
    if trimmed_start != 0 {
        slot_id.drain(..trimmed_start);
    }
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

#[cfg(test)]
#[path = "slot_id/in_place_tests.rs"]
mod in_place_tests;
