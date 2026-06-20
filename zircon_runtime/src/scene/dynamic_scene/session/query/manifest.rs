use super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchiveManifest,
    RuntimeSessionSlot,
};

pub(in crate::scene::dynamic_scene::session) fn manifest(
    archive: &RuntimeSessionArchive,
) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
    archive.ensure_supported()?;
    let mut slots = archive
        .slots
        .iter()
        .map(RuntimeSessionSlot::summary)
        .collect::<Vec<_>>();
    slots.sort_by(|left, right| left.slot_id.cmp(&right.slot_id));
    Ok(RuntimeSessionArchiveManifest {
        format_version: archive.format_version,
        slots,
    })
}
