use super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchiveStatistics,
};

pub(in crate::scene::dynamic_scene::session) fn statistics(
    archive: &RuntimeSessionArchive,
) -> Result<RuntimeSessionArchiveStatistics, RuntimeSessionArchiveError> {
    archive.ensure_supported()?;
    let mut statistics = RuntimeSessionArchiveStatistics {
        format_version: archive.format_version,
        slot_count: archive.slots.len(),
        ..Default::default()
    };

    for slot in &archive.slots {
        let entity_count = slot.scene.entities.len();
        let resource_count = slot.scene.resources.len();
        statistics.total_entity_count += entity_count;
        statistics.total_resource_count += resource_count;
        statistics.max_slot_entity_count = statistics.max_slot_entity_count.max(entity_count);
        statistics.max_slot_resource_count = statistics.max_slot_resource_count.max(resource_count);

        if let Some(updated_at) = slot.metadata.updated_at_unix_millis {
            statistics.earliest_updated_at_unix_millis = Some(
                statistics
                    .earliest_updated_at_unix_millis
                    .map_or(updated_at, |current| current.min(updated_at)),
            );
            statistics.latest_updated_at_unix_millis = Some(
                statistics
                    .latest_updated_at_unix_millis
                    .map_or(updated_at, |current| current.max(updated_at)),
            );
        } else {
            statistics.untimed_slot_count += 1;
        }
    }

    Ok(statistics)
}
