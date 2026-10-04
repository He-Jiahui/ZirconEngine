use super::super::access::{RenderGraphResourceAccessIntent, RenderGraphResourceAccessRange};
use super::{QueueLane, RenderGraphResource, RenderGraphResourceAccessId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RenderGraphResourceState {
    Legacy,
    SampledTexture,
    StorageTextureRead,
    StorageTextureWrite,
    ColorAttachment,
    DepthStencilAttachment,
    UniformBuffer,
    StorageBufferRead,
    StorageBufferReadWrite,
    CopySource,
    CopyDestination,
    Indirect,
    Present,
    Readback,
}

impl From<RenderGraphResourceAccessIntent> for RenderGraphResourceState {
    fn from(intent: RenderGraphResourceAccessIntent) -> Self {
        match intent {
            RenderGraphResourceAccessIntent::Legacy => Self::Legacy,
            RenderGraphResourceAccessIntent::SampledTexture { .. } => Self::SampledTexture,
            RenderGraphResourceAccessIntent::StorageTextureRead { .. } => Self::StorageTextureRead,
            RenderGraphResourceAccessIntent::StorageTextureWrite { .. } => {
                Self::StorageTextureWrite
            }
            RenderGraphResourceAccessIntent::ColorAttachment => Self::ColorAttachment,
            RenderGraphResourceAccessIntent::DepthStencilAttachment => Self::DepthStencilAttachment,
            RenderGraphResourceAccessIntent::UniformBuffer { .. } => Self::UniformBuffer,
            RenderGraphResourceAccessIntent::StorageBufferRead { .. } => Self::StorageBufferRead,
            RenderGraphResourceAccessIntent::StorageBufferReadWrite { .. } => {
                Self::StorageBufferReadWrite
            }
            RenderGraphResourceAccessIntent::CopySource => Self::CopySource,
            RenderGraphResourceAccessIntent::CopyDestination => Self::CopyDestination,
            RenderGraphResourceAccessIntent::Indirect => Self::Indirect,
            RenderGraphResourceAccessIntent::Present => Self::Present,
            RenderGraphResourceAccessIntent::Readback => Self::Readback,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompiledRenderGraphResourceStateTransition {
    pub resource: RenderGraphResource,
    pub range: RenderGraphResourceAccessRange,
    pub from_access: RenderGraphResourceAccessId,
    pub to_access: RenderGraphResourceAccessId,
    pub from_state: RenderGraphResourceState,
    pub to_state: RenderGraphResourceState,
    pub from_queue: QueueLane,
    pub to_queue: QueueLane,
}

impl CompiledRenderGraphResourceStateTransition {
    pub fn crosses_queue(self) -> bool {
        self.from_queue != self.to_queue
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CompiledRenderGraphResourceStatePlan {
    transitions: Vec<CompiledRenderGraphResourceStateTransition>,
    legacy_access_count: usize,
    queue_transition_count: usize,
    tracking_work_count: usize,
    tracking_lookup_visits: usize,
    tracking_split_visits: usize,
    tracking_read_visits: usize,
    tracking_update_visits: usize,
    tracking_merge_visits: usize,
    tracking_plane_visits: usize,
}

impl CompiledRenderGraphResourceStatePlan {
    pub fn transitions(&self) -> &[CompiledRenderGraphResourceStateTransition] {
        &self.transitions
    }

    pub const fn legacy_access_count(&self) -> usize {
        self.legacy_access_count
    }

    pub const fn queue_transition_count(&self) -> usize {
        self.queue_transition_count
    }

    /// Tracked work units in state-plan construction only.
    ///
    /// Includes boundary searches, successful splits, returned interval reads,
    /// history updates and neighboring interval comparisons. This excludes map
    /// implementation comparisons, dependency inference and cull-root queries.
    /// Compact texture-plane traversal is included once per visited plane;
    /// layer cardinality itself adds no per-layer work units.
    pub const fn tracking_work_count(&self) -> usize {
        self.tracking_work_count
    }

    /// Boundary predecessor searches, not all map API calls or tree comparisons.
    pub const fn tracking_lookup_visits(&self) -> usize {
        self.tracking_lookup_visits
    }

    pub const fn tracking_split_visits(&self) -> usize {
        self.tracking_split_visits
    }

    pub const fn tracking_read_visits(&self) -> usize {
        self.tracking_read_visits
    }

    pub const fn tracking_update_visits(&self) -> usize {
        self.tracking_update_visits
    }

    pub const fn tracking_merge_visits(&self) -> usize {
        self.tracking_merge_visits
    }

    /// Compact texture planes visited during state-plan construction. A plane
    /// is one mip/aspect key; its layer range remains an interval.
    pub const fn tracking_plane_visits(&self) -> usize {
        self.tracking_plane_visits
    }

    pub(crate) fn record_access_work(
        &mut self,
        intent: RenderGraphResourceAccessIntent,
        lookup_visits: usize,
        split_visits: usize,
        read_visits: usize,
        update_visits: usize,
        merge_visits: usize,
    ) {
        self.legacy_access_count +=
            matches!(intent, RenderGraphResourceAccessIntent::Legacy) as usize;
        self.tracking_lookup_visits += lookup_visits;
        self.tracking_split_visits += split_visits;
        self.tracking_read_visits += read_visits;
        self.tracking_update_visits += update_visits;
        self.tracking_merge_visits += merge_visits;
        self.tracking_work_count +=
            lookup_visits + split_visits + read_visits + update_visits + merge_visits;
    }

    /// Adds compact texture-plane traversal work after the interval receipt.
    /// This is separate so existing interval counters retain their meaning and
    /// the compile owner can integrate the new receipt without changing the
    /// existing five-argument interval call.
    pub(crate) fn record_texture_plane_work(&mut self, plane_visits: usize) {
        self.tracking_plane_visits += plane_visits;
        self.tracking_work_count += plane_visits;
    }

    pub(crate) fn push_transition(
        &mut self,
        transition: CompiledRenderGraphResourceStateTransition,
    ) {
        self.queue_transition_count += transition.crosses_queue() as usize;
        self.transitions.push(transition);
    }
}
