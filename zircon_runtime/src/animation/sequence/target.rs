use crate::core::framework::scene::EntityPath;
use crate::scene::world::World;

pub(super) fn resolve_sequence_target_id(
    world: &World,
    target_id: &str,
) -> Option<crate::scene::EntityId> {
    let target_id = target_id.trim();
    if target_id.is_empty() {
        return None;
    }
    target_id
        .parse::<crate::scene::EntityId>()
        .ok()
        .filter(|entity| world.contains_entity(*entity))
        .or_else(|| {
            EntityPath::parse(target_id)
                .ok()
                .and_then(|path| world.get_entity_by_path(&path))
        })
}
