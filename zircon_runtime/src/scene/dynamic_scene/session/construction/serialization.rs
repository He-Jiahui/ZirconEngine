use super::super::{RuntimeSessionArchive, RuntimeSessionArchiveError};
use crate::scene::dynamic_scene::DynamicScene;

pub(in crate::scene::dynamic_scene::session) fn from_versioned_json(
    json: &str,
) -> Result<RuntimeSessionArchive, RuntimeSessionArchiveError> {
    let document: serde_json::Value = serde_json::from_str(json)?;
    validate_embedded_scene_headers(&document)?;
    let mut archive: RuntimeSessionArchive = serde_json::from_value(document)?;
    archive.normalize_slot_metadata();
    archive.ensure_supported()?;
    archive.sort_slots();
    Ok(archive)
}

pub(in crate::scene::dynamic_scene::session) fn to_versioned_json_pretty(
    archive: &RuntimeSessionArchive,
) -> Result<String, RuntimeSessionArchiveError> {
    let mut archive = archive.clone();
    archive.normalize_slot_metadata();
    archive.sort_slots();
    archive.ensure_supported()?;
    Ok(serde_json::to_string_pretty(&archive)?)
}

fn validate_embedded_scene_headers(
    document: &serde_json::Value,
) -> Result<(), RuntimeSessionArchiveError> {
    let Some(slots) = document.get("slots").and_then(serde_json::Value::as_array) else {
        return Ok(());
    };
    for slot in slots {
        let Some(scene) = slot.get("scene") else {
            continue;
        };
        let text = serde_json::to_string(scene)?;
        DynamicScene::from_versioned_json(&text)?;
    }
    Ok(())
}
