use super::{
    CorePipelineKind, RenderPhase, RenderPhaseItem, RenderPhaseMeshSource, RenderPhaseQueueSummary,
    RenderPhaseSortComponents, RenderQueueValue,
};
use crate::core::framework::scene::EntityId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderPhaseQueue {
    pub items: Vec<RenderPhaseItem>,
}

impl RenderPhaseQueue {
    pub fn new(mut items: Vec<RenderPhaseItem>) -> Self {
        sort_phase_queue_items(&mut items);
        Self { items }
    }

    pub fn items_for_phase(&self, phase: RenderPhase) -> impl Iterator<Item = &RenderPhaseItem> {
        let phase_order = phase.queue_order();
        let start = self
            .items
            .partition_point(|item| item.phase.queue_order() < phase_order);
        let end = self
            .items
            .partition_point(|item| item.phase.queue_order() <= phase_order);
        self.items[start..end]
            .iter()
            .filter(move |item| item.phase == phase)
    }

    pub fn summary(&self) -> RenderPhaseQueueSummary {
        RenderPhaseQueueSummary::from_sorted_items(&self.items)
    }
}

fn sort_phase_queue_items(items: &mut [RenderPhaseItem]) {
    if items.len() < 2 {
        return;
    }

    let mut ordered = Vec::with_capacity(items.len());
    ordered.extend(
        items
            .iter()
            .enumerate()
            .map(|(source_index, item)| (item.ordering_key(), source_index)),
    );
    ordered.sort_unstable();

    let mut destination_for_source = vec![0_usize; items.len()];
    for (destination, (_, source)) in ordered.into_iter().enumerate() {
        destination_for_source[source] = destination;
    }
    for source in 0..items.len() {
        while destination_for_source[source] != source {
            let destination = destination_for_source[source];
            items.swap(source, destination);
            destination_for_source.swap(source, destination);
        }
    }
}

pub fn build_mesh_phase_queue(
    pipeline: CorePipelineKind,
    meshes: impl IntoIterator<Item = MeshPhaseInput>,
) -> RenderPhaseQueue {
    RenderPhaseQueue::new(
        meshes
            .into_iter()
            .map(|mesh| mesh.into_phase_item(pipeline))
            .collect(),
    )
}

pub fn build_sprite_phase_queue(
    pipeline: CorePipelineKind,
    sprites: impl IntoIterator<Item = SpritePhaseInput>,
) -> RenderPhaseQueue {
    RenderPhaseQueue::new(
        sprites
            .into_iter()
            .map(|sprite| sprite.into_phase_item(pipeline))
            .collect(),
    )
}

#[derive(Clone, Copy, Debug)]
pub struct MeshPhaseInput {
    pub entity: EntityId,
    pub mesh_index: usize,
    pub queue: RenderQueueValue,
    pub depth: f32,
    pub depth_bias: f32,
    pub camera_order: i32,
    pub sorting_layer: i32,
    pub order_in_layer: i32,
    pub y_sort: Option<f32>,
    pub ui_z_index: i32,
}

impl MeshPhaseInput {
    pub const fn new(
        entity: EntityId,
        mesh_index: usize,
        queue: RenderQueueValue,
        depth: f32,
    ) -> Self {
        MeshPhaseInput {
            entity,
            mesh_index,
            queue,
            depth,
            depth_bias: 0.0,
            camera_order: 0,
            sorting_layer: 0,
            order_in_layer: 0,
            y_sort: None,
            ui_z_index: 0,
        }
    }

    pub const fn with_depth_bias(mut self, depth_bias: f32) -> Self {
        self.depth_bias = depth_bias;
        self
    }

    pub const fn with_camera_order(mut self, camera_order: i32) -> Self {
        self.camera_order = camera_order;
        self
    }

    pub const fn with_queue(mut self, queue: RenderQueueValue) -> Self {
        self.queue = queue;
        self
    }

    pub fn with_queue_offset(mut self, offset: i32) -> Self {
        self.queue = self.queue.with_material_offset_i32(offset);
        self
    }

    pub const fn with_sorting_layer(mut self, sorting_layer: i32) -> Self {
        self.sorting_layer = sorting_layer;
        self
    }

    pub const fn with_order_in_layer(mut self, order_in_layer: i32) -> Self {
        self.order_in_layer = order_in_layer;
        self
    }

    pub const fn with_y_sort(mut self, y_sort: Option<f32>) -> Self {
        self.y_sort = y_sort;
        self
    }

    pub const fn with_ui_z_index(mut self, ui_z_index: i32) -> Self {
        self.ui_z_index = ui_z_index;
        self
    }

    fn into_phase_item(self, pipeline: CorePipelineKind) -> RenderPhaseItem {
        let phase = self.queue.phase(pipeline);
        let sort_components = RenderPhaseSortComponents::new(self.depth, self.entity)
            .with_depth_bias(self.depth_bias)
            .with_camera_order(self.camera_order)
            .with_queue(self.queue)
            .with_sorting_layer(self.sorting_layer)
            .with_order_in_layer(self.order_in_layer)
            .with_y_sort(self.y_sort)
            .with_ui_z_index(self.ui_z_index);
        RenderPhaseItem {
            entity: self.entity,
            phase,
            sort_key: super::RenderPhaseSortKey::for_components(phase, sort_components),
            mesh_source: RenderPhaseMeshSource::MeshIndex(self.mesh_index),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SpritePhaseInput {
    pub entity: EntityId,
    pub sprite_index: usize,
    pub queue: RenderQueueValue,
    pub z_order: i32,
    pub depth: f32,
    pub depth_bias: f32,
    pub camera_order: i32,
    pub sorting_layer: i32,
    pub y_sort: Option<f32>,
    pub ui_z_index: i32,
}

impl SpritePhaseInput {
    pub const fn new(
        entity: EntityId,
        sprite_index: usize,
        queue: RenderQueueValue,
        z_order: i32,
        depth: f32,
    ) -> Self {
        Self {
            entity,
            sprite_index,
            queue,
            z_order,
            depth,
            depth_bias: 0.0,
            camera_order: 0,
            sorting_layer: 0,
            y_sort: None,
            ui_z_index: 0,
        }
    }

    pub const fn with_depth_bias(mut self, depth_bias: f32) -> Self {
        self.depth_bias = depth_bias;
        self
    }

    pub const fn with_camera_order(mut self, camera_order: i32) -> Self {
        self.camera_order = camera_order;
        self
    }

    pub const fn with_queue(mut self, queue: RenderQueueValue) -> Self {
        self.queue = queue;
        self
    }

    pub fn with_queue_offset(mut self, offset: i32) -> Self {
        self.queue = self.queue.with_material_offset_i32(offset);
        self
    }

    pub const fn with_sorting_layer(mut self, sorting_layer: i32) -> Self {
        self.sorting_layer = sorting_layer;
        self
    }

    pub const fn with_y_sort(mut self, y_sort: Option<f32>) -> Self {
        self.y_sort = y_sort;
        self
    }

    pub const fn with_ui_z_index(mut self, ui_z_index: i32) -> Self {
        self.ui_z_index = ui_z_index;
        self
    }

    fn into_phase_item(self, pipeline: CorePipelineKind) -> RenderPhaseItem {
        let phase = self.queue.phase(pipeline);
        let sort_components = RenderPhaseSortComponents::new(self.depth, self.entity)
            .with_depth_bias(self.depth_bias)
            .with_camera_order(self.camera_order)
            .with_queue(self.queue)
            .with_sorting_layer(self.sorting_layer)
            .with_order_in_layer(self.z_order)
            .with_y_sort(self.y_sort)
            .with_ui_z_index(self.ui_z_index);
        RenderPhaseItem {
            entity: self.entity,
            phase,
            sort_key: super::RenderPhaseSortKey::for_components(phase, sort_components),
            mesh_source: RenderPhaseMeshSource::SpriteIndex(self.sprite_index),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::sort_phase_queue_items;
    use crate::core::framework::render::{
        RenderPhase, RenderPhaseItem, RenderPhaseMeshSource, RenderPhaseSortKey,
    };

    const SAMPLE_PAIRS: usize = 17;

    #[test]
    fn phase_iteration_limits_work_to_the_sorted_phase_order_span() {
        let source = include_str!("phase_queue.rs");

        assert!(source.contains(concat!("partition", "_point")));
        assert!(source.contains(concat!("let phase_order", " = phase.queue_order();")));
        assert!(!source.contains(concat!(
            "self.items.iter().",
            "filter(move |item| item.phase == phase)"
        )));
    }

    #[test]
    fn optimization_batch_dc_phase_queue_indirect_sort_preserves_stable_ties() {
        let mut actual = vec![
            item(3, 7, 30),
            item(1, 2, 10),
            item(3, 7, 31),
            item(2, 4, 20),
            item(3, 7, 32),
        ];
        let mut expected = actual.clone();
        expected.sort_by_key(RenderPhaseItem::ordering_key);

        sort_phase_queue_items(&mut actual);

        assert_eq!(actual, expected);
        assert_eq!(
            actual
                .iter()
                .filter(|item| item.entity == 3)
                .map(|item| item.mesh_source)
                .collect::<Vec<_>>(),
            vec![
                RenderPhaseMeshSource::MeshIndex(30),
                RenderPhaseMeshSource::MeshIndex(31),
                RenderPhaseMeshSource::MeshIndex(32),
            ]
        );
    }

    #[test]
    fn optimization_batch_dc_phase_queue_uses_compact_indirect_sort() {
        let production = include_str!("phase_queue.rs")
            .split_once("#[cfg(test)]")
            .expect("production source and tests must remain separated")
            .0;

        assert!(production.contains("Vec::with_capacity(items.len())"));
        assert!(production.contains("ordered.sort_unstable();"));
        assert!(production.contains("destination_for_source"));
        assert!(!production.contains("items.sort_by_key(RenderPhaseItem::ordering_key)"));
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_dc_runtime409_phase_queue_indirect_sort_p95() {
        const ITEM_COUNT: usize = 65_536;
        let template = (0..ITEM_COUNT)
            .map(|index| {
                let mixed = (index as u64)
                    .wrapping_mul(0x9e37_79b9_7f4a_7c15)
                    .rotate_left(17);
                item(mixed & 0x3fff, mixed >> 9, index)
            })
            .collect::<Vec<_>>();

        for _ in 0..3 {
            black_box(measure_sort(&template, false));
            black_box(measure_sort(&template, true));
        }

        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure_sort(&template, false));
                optimized.push(measure_sort(&template, true));
            } else {
                optimized.push(measure_sort(&template, true));
                legacy.push(measure_sort(&template, false));
            }
        }

        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!(
            "RUNTIME409_PHASE_QUEUE_INDIRECT_SORT_BENCH_V1 sample_pairs={SAMPLE_PAIRS} item_count={ITEM_COUNT} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            csv(&legacy),
            csv(&optimized)
        );
        assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
    }

    fn item(entity: u64, raw_sort_key: u64, mesh_index: usize) -> RenderPhaseItem {
        RenderPhaseItem {
            entity,
            phase: RenderPhase::Opaque3d,
            sort_key: RenderPhaseSortKey::new(raw_sort_key),
            mesh_source: RenderPhaseMeshSource::MeshIndex(mesh_index),
        }
    }

    fn measure_sort(template: &[RenderPhaseItem], optimized: bool) -> u128 {
        let mut items = template.to_vec();
        let started = Instant::now();
        if optimized {
            sort_phase_queue_items(&mut items);
        } else {
            items.sort_by_key(RenderPhaseItem::ordering_key);
        }
        black_box(items);
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
