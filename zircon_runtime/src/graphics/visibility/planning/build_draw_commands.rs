use crate::core::framework::scene::EntityId;

use super::super::declarations::{VisibilityBatch, VisibilityDrawCommand};

pub(crate) fn build_draw_commands(
    visible_batches: &[VisibilityBatch],
) -> (Vec<EntityId>, Vec<VisibilityDrawCommand>) {
    let visible_instance_count = visible_batches
        .iter()
        .map(|batch| batch.entities.len())
        .sum();
    let mut visible_instances = Vec::with_capacity(visible_instance_count);
    let mut draw_commands = Vec::with_capacity(visible_batches.len());

    for batch in visible_batches {
        let visible_instance_offset = u32::try_from(visible_instances.len())
            .expect("visible instance count should fit in u32");
        let visible_instance_count =
            u32::try_from(batch.entities.len()).expect("batch instance count should fit in u32");
        visible_instances.extend(batch.entities.iter().copied());
        draw_commands.push(VisibilityDrawCommand {
            key: batch.key.clone(),
            visible_instance_offset,
            visible_instance_count,
        });
    }

    (visible_instances, draw_commands)
}

#[cfg(test)]
mod tests {
    #[test]
    fn draw_command_builder_preallocates_visible_instances() {
        let source = include_str!("build_draw_commands.rs");
        let capacity = concat!("Vec::with_capacity(", "visible_instance_count)");

        assert!(source.contains(capacity));
    }
}
