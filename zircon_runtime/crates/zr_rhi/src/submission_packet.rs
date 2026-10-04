use super::{
    CommandList, CommandListCommand, DeviceGeneration, DeviceId, DiagnosticPassQueryScope,
    DiagnosticQueryPlan, RenderQueueClass, RhiError,
};
use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

/// Logical queue selected by the graph compiler.  WGPU currently lowers all
/// lanes to one native queue, but retaining the lane in the receipt keeps the
/// compiler's ordering and ownership proof visible to the backend.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RhiGraphQueueLane {
    Graphics,
    AsyncCompute,
    AsyncCopy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RhiGraphResourceKind {
    Texture,
    Buffer,
    External,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RhiGraphResourceId {
    kind: RhiGraphResourceKind,
    index: usize,
    generation: u64,
}

impl RhiGraphResourceId {
    pub const fn new(kind: RhiGraphResourceKind, index: usize, generation: u64) -> Self {
        Self {
            kind,
            index,
            generation,
        }
    }

    pub const fn kind(self) -> RhiGraphResourceKind {
        self.kind
    }

    pub const fn index(self) -> usize {
        self.index
    }

    pub const fn generation(self) -> u64 {
        self.generation
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RhiGraphAccessId {
    pass_index: usize,
    pass_generation: u64,
    access_ordinal: usize,
}

impl RhiGraphAccessId {
    pub const fn new(pass_index: usize, pass_generation: u64, access_ordinal: usize) -> Self {
        Self {
            pass_index,
            pass_generation,
            access_ordinal,
        }
    }

    pub const fn pass_index(self) -> usize {
        self.pass_index
    }

    pub const fn pass_generation(self) -> u64 {
        self.pass_generation
    }

    pub const fn access_ordinal(self) -> usize {
        self.access_ordinal
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RhiGraphAccessRange {
    Buffer {
        offset: u64,
        size: u64,
    },
    Texture {
        mip_start: u32,
        mip_end: u32,
        layer_start: u32,
        layer_end: u32,
        aspect_mask: u8,
    },
    UnresolvedExternal,
}

impl RhiGraphAccessRange {
    pub const fn buffer(offset: u64, size: u64) -> Self {
        Self::Buffer { offset, size }
    }

    pub const fn texture(
        mip_start: u32,
        mip_end: u32,
        layer_start: u32,
        layer_end: u32,
        aspect_mask: u8,
    ) -> Self {
        Self::Texture {
            mip_start,
            mip_end,
            layer_start,
            layer_end,
            aspect_mask,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RhiGraphResourceBounds {
    Buffer {
        size: u64,
    },
    Texture {
        mip_levels: u32,
        array_layers: u32,
        supported_aspects: u8,
    },
}

impl RhiGraphResourceBounds {
    pub fn buffer(size: u64) -> Result<Self, &'static str> {
        if size == 0 {
            return Err("physical buffer lease must have non-zero size");
        }
        Ok(Self::Buffer { size })
    }

    pub fn texture(
        mip_levels: u32,
        array_layers: u32,
        supported_aspects: u8,
    ) -> Result<Self, &'static str> {
        if mip_levels == 0 || array_layers == 0 || supported_aspects == 0 {
            return Err("physical texture lease must have non-zero dimensions and aspect mask");
        }
        Ok(Self::Texture {
            mip_levels,
            array_layers,
            supported_aspects,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RhiGraphResourceAccessKind {
    Read,
    Write,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RhiGraphResourceState {
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RhiGraphExecutionAccess {
    id: RhiGraphAccessId,
    resource: RhiGraphResourceId,
    version: u64,
    kind: RhiGraphResourceAccessKind,
    range: RhiGraphAccessRange,
    state: RhiGraphResourceState,
    queue: RhiGraphQueueLane,
    allocation_id: Option<u64>,
    has_physical_binding: bool,
}

impl RhiGraphExecutionAccess {
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        id: RhiGraphAccessId,
        resource: RhiGraphResourceId,
        version: u64,
        kind: RhiGraphResourceAccessKind,
        range: RhiGraphAccessRange,
        state: RhiGraphResourceState,
        queue: RhiGraphQueueLane,
        allocation_id: Option<u64>,
        has_physical_binding: bool,
    ) -> Self {
        Self {
            id,
            resource,
            version,
            kind,
            range,
            state,
            queue,
            allocation_id,
            has_physical_binding,
        }
    }

    pub const fn id(&self) -> RhiGraphAccessId {
        self.id
    }
    pub const fn resource(&self) -> RhiGraphResourceId {
        self.resource
    }
    pub const fn version(&self) -> u64 {
        self.version
    }
    pub const fn kind(&self) -> RhiGraphResourceAccessKind {
        self.kind
    }
    pub const fn range(&self) -> RhiGraphAccessRange {
        self.range
    }
    pub const fn state(&self) -> RhiGraphResourceState {
        self.state
    }
    pub const fn queue(&self) -> RhiGraphQueueLane {
        self.queue
    }
    pub const fn allocation_id(&self) -> Option<u64> {
        self.allocation_id
    }
    pub const fn has_physical_binding(&self) -> bool {
        self.has_physical_binding
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RhiGraphExecutionTransition {
    resource: RhiGraphResourceId,
    range: RhiGraphAccessRange,
    from_access: RhiGraphAccessId,
    to_access: RhiGraphAccessId,
    from_state: RhiGraphResourceState,
    to_state: RhiGraphResourceState,
    from_queue: RhiGraphQueueLane,
    to_queue: RhiGraphQueueLane,
}

impl RhiGraphExecutionTransition {
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        resource: RhiGraphResourceId,
        range: RhiGraphAccessRange,
        from_access: RhiGraphAccessId,
        to_access: RhiGraphAccessId,
        from_state: RhiGraphResourceState,
        to_state: RhiGraphResourceState,
        from_queue: RhiGraphQueueLane,
        to_queue: RhiGraphQueueLane,
    ) -> Self {
        Self {
            resource,
            range,
            from_access,
            to_access,
            from_state,
            to_state,
            from_queue,
            to_queue,
        }
    }

    pub const fn resource(&self) -> RhiGraphResourceId {
        self.resource
    }
    pub const fn range(&self) -> RhiGraphAccessRange {
        self.range
    }
    pub const fn from_access(&self) -> RhiGraphAccessId {
        self.from_access
    }
    pub const fn to_access(&self) -> RhiGraphAccessId {
        self.to_access
    }
    pub const fn from_state(&self) -> RhiGraphResourceState {
        self.from_state
    }
    pub const fn to_state(&self) -> RhiGraphResourceState {
        self.to_state
    }
    pub const fn from_queue(&self) -> RhiGraphQueueLane {
        self.from_queue
    }
    pub const fn to_queue(&self) -> RhiGraphQueueLane {
        self.to_queue
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RhiGraphExecutionPass {
    graph_pass_index: usize,
    pass_index: usize,
    pass_generation: u64,
    queue: RhiGraphQueueLane,
    accesses: Vec<RhiGraphExecutionAccess>,
    transitions_before: Vec<RhiGraphExecutionTransition>,
}

impl RhiGraphExecutionPass {
    pub fn new(
        graph_pass_index: usize,
        pass_index: usize,
        pass_generation: u64,
        queue: RhiGraphQueueLane,
        accesses: Vec<RhiGraphExecutionAccess>,
        transitions_before: Vec<RhiGraphExecutionTransition>,
    ) -> Self {
        Self {
            graph_pass_index,
            pass_index,
            pass_generation,
            queue,
            accesses,
            transitions_before,
        }
    }

    pub const fn graph_pass_index(&self) -> usize {
        self.graph_pass_index
    }
    pub const fn pass_index(&self) -> usize {
        self.pass_index
    }
    pub const fn pass_generation(&self) -> u64 {
        self.pass_generation
    }
    pub const fn queue(&self) -> RhiGraphQueueLane {
        self.queue
    }
    pub fn accesses(&self) -> &[RhiGraphExecutionAccess] {
        &self.accesses
    }
    pub fn transitions_before(&self) -> &[RhiGraphExecutionTransition] {
        &self.transitions_before
    }
}

pub struct RhiGraphPhysicalResourceLease {
    access_id: RhiGraphAccessId,
    resource: RhiGraphResourceId,
    allocation_id: Option<u64>,
    device_id: DeviceId,
    generation: DeviceGeneration,
    bounds: RhiGraphResourceBounds,
    physical: Arc<dyn Any + Send + Sync>,
}

impl fmt::Debug for RhiGraphPhysicalResourceLease {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RhiGraphPhysicalResourceLease")
            .field("access_id", &self.access_id)
            .field("resource", &self.resource)
            .field("allocation_id", &self.allocation_id)
            .field("device_id", &self.device_id)
            .field("generation", &self.generation)
            .field("bounds", &self.bounds)
            .finish_non_exhaustive()
    }
}

impl RhiGraphPhysicalResourceLease {
    pub fn new(
        access_id: RhiGraphAccessId,
        resource: RhiGraphResourceId,
        allocation_id: Option<u64>,
        device_id: DeviceId,
        generation: DeviceGeneration,
        bounds: RhiGraphResourceBounds,
        physical: Arc<dyn Any + Send + Sync>,
    ) -> Self {
        Self {
            access_id,
            resource,
            allocation_id,
            device_id,
            generation,
            bounds,
            physical,
        }
    }

    pub const fn access_id(&self) -> RhiGraphAccessId {
        self.access_id
    }
    pub const fn resource(&self) -> RhiGraphResourceId {
        self.resource
    }
    pub const fn allocation_id(&self) -> Option<u64> {
        self.allocation_id
    }
    pub const fn device_id(&self) -> DeviceId {
        self.device_id
    }
    pub const fn generation(&self) -> DeviceGeneration {
        self.generation
    }
    pub const fn bounds(&self) -> RhiGraphResourceBounds {
        self.bounds
    }
    pub fn physical(&self) -> &(dyn Any + Send + Sync) {
        self.physical.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RhiGraphExecutionReceiptError(String);

impl fmt::Display for RhiGraphExecutionReceiptError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for RhiGraphExecutionReceiptError {}

impl RhiGraphExecutionReceiptError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

pub struct RhiGraphExecutionReceipt {
    device_id: DeviceId,
    generation: DeviceGeneration,
    frame_generation: u64,
    graph_generation: u64,
    submission_queue: RenderQueueClass,
    passes: Vec<RhiGraphExecutionPass>,
    leases: Vec<RhiGraphPhysicalResourceLease>,
}

impl fmt::Debug for RhiGraphExecutionReceipt {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RhiGraphExecutionReceipt")
            .field("device_id", &self.device_id)
            .field("generation", &self.generation)
            .field("frame_generation", &self.frame_generation)
            .field("graph_generation", &self.graph_generation)
            .field("submission_queue", &self.submission_queue)
            .field("passes", &self.passes)
            .field("physical_lease_count", &self.leases.len())
            .finish()
    }
}

impl RhiGraphExecutionReceipt {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        device_id: DeviceId,
        generation: DeviceGeneration,
        frame_generation: u64,
        graph_generation: u64,
        submission_queue: RenderQueueClass,
        passes: Vec<RhiGraphExecutionPass>,
        leases: Vec<RhiGraphPhysicalResourceLease>,
    ) -> Result<Self, RhiGraphExecutionReceiptError> {
        if passes.is_empty() {
            return Err(RhiGraphExecutionReceiptError::new(
                "graph execution receipt must contain a pass",
            ));
        }
        let mut access_locations = HashMap::new();
        let mut accesses = HashMap::new();
        let mut previous_graph_pass_index = None;
        for (pass_order, pass) in passes.iter().enumerate() {
            if previous_graph_pass_index.is_some_and(|previous| previous >= pass.graph_pass_index) {
                return Err(RhiGraphExecutionReceiptError::new(
                    "graph passes are not in compiled execution order",
                ));
            }
            previous_graph_pass_index = Some(pass.graph_pass_index);
            if pass.pass_generation != graph_generation {
                return Err(RhiGraphExecutionReceiptError::new(
                    "graph pass generation does not match receipt",
                ));
            }
            for (ordinal, access) in pass.accesses.iter().enumerate() {
                let id = access.id;
                if id.pass_index != pass.pass_index
                    || id.pass_generation != pass.pass_generation
                    || id.access_ordinal != ordinal
                {
                    return Err(RhiGraphExecutionReceiptError::new(format!(
                        "access identity does not match pass {}",
                        pass.pass_index
                    )));
                }
                if access_locations.insert(id, pass_order).is_some() {
                    return Err(RhiGraphExecutionReceiptError::new(
                        "duplicate graph access identity",
                    ));
                }
                if access.resource.generation != graph_generation {
                    return Err(RhiGraphExecutionReceiptError::new(
                        "graph resource generation does not match receipt",
                    ));
                }
                if access.range == RhiGraphAccessRange::UnresolvedExternal {
                    if access.has_physical_binding {
                        return Err(RhiGraphExecutionReceiptError::new(
                            "unresolved external access cannot claim a physical binding",
                        ));
                    }
                } else if !access.has_physical_binding {
                    return Err(RhiGraphExecutionReceiptError::new(
                        "device graph access has no physical binding",
                    ));
                }
                accesses.insert(id, access);
            }
        }

        let mut leases_by_access = HashMap::new();
        for lease in &leases {
            if lease.device_id != device_id || lease.generation != generation {
                return Err(RhiGraphExecutionReceiptError::new(
                    "physical lease device generation does not match receipt",
                ));
            }
            if leases_by_access.insert(lease.access_id, lease).is_some() {
                return Err(RhiGraphExecutionReceiptError::new(
                    "duplicate physical lease identity",
                ));
            }
            let access = accesses.get(&lease.access_id).ok_or_else(|| {
                RhiGraphExecutionReceiptError::new("physical lease has no graph access")
            })?;
            if access.resource != lease.resource || access.allocation_id != lease.allocation_id {
                return Err(RhiGraphExecutionReceiptError::new(
                    "physical lease resource or allocation does not match access",
                ));
            }
            validate_range(access.range, lease.bounds)?;
        }
        for (id, access) in &accesses {
            if access.has_physical_binding && !leases_by_access.contains_key(id) {
                return Err(RhiGraphExecutionReceiptError::new(
                    "physical graph access has no lease",
                ));
            }
            if !access.has_physical_binding && leases_by_access.contains_key(id) {
                return Err(RhiGraphExecutionReceiptError::new(
                    "logical-only graph access has a physical lease",
                ));
            }
        }

        for pass in &passes {
            for transition in &pass.transitions_before {
                let from = accesses.get(&transition.from_access).ok_or_else(|| {
                    RhiGraphExecutionReceiptError::new("transition source access is missing")
                })?;
                let to = accesses.get(&transition.to_access).ok_or_else(|| {
                    RhiGraphExecutionReceiptError::new("transition destination access is missing")
                })?;
                if access_locations[&transition.from_access]
                    >= access_locations[&transition.to_access]
                {
                    return Err(RhiGraphExecutionReceiptError::new(
                        "transition source access must precede destination",
                    ));
                }
                if transition.to_access.pass_index != pass.pass_index
                    || transition.to_access.pass_generation != pass.pass_generation
                    || transition.resource != from.resource
                    || transition.resource != to.resource
                    || !range_contains(from.range, transition.range)
                    || !range_contains(to.range, transition.range)
                    || transition.from_state != from.state
                    || transition.to_state != to.state
                    || transition.from_queue != from.queue
                    || transition.to_queue != to.queue
                {
                    return Err(RhiGraphExecutionReceiptError::new(
                        "transition does not match source and destination access proof",
                    ));
                }
            }
        }

        Ok(Self {
            device_id,
            generation,
            frame_generation,
            graph_generation,
            submission_queue,
            passes,
            leases,
        })
    }

    pub const fn device_id(&self) -> DeviceId {
        self.device_id
    }
    pub const fn generation(&self) -> DeviceGeneration {
        self.generation
    }
    pub const fn frame_generation(&self) -> u64 {
        self.frame_generation
    }
    pub const fn graph_generation(&self) -> u64 {
        self.graph_generation
    }
    pub const fn submission_queue(&self) -> RenderQueueClass {
        self.submission_queue
    }

    /// Requalifies an already validated graph proof for the frame generation
    /// that owns its submission ticket.  The graph/device/access proof is
    /// immutable; only the caller-owned frame sequence changes at submission
    /// time.
    pub fn with_frame_generation(mut self, frame_generation: u64) -> Self {
        self.frame_generation = frame_generation;
        self
    }
    pub fn passes(&self) -> &[RhiGraphExecutionPass] {
        &self.passes
    }
    pub fn physical_leases(&self) -> &[RhiGraphPhysicalResourceLease] {
        &self.leases
    }
    pub fn physical_lease_count(&self) -> usize {
        self.leases.len()
    }
    pub fn accesses(&self) -> impl Iterator<Item = &RhiGraphExecutionAccess> {
        self.passes.iter().flat_map(RhiGraphExecutionPass::accesses)
    }
}

fn validate_range(
    range: RhiGraphAccessRange,
    bounds: RhiGraphResourceBounds,
) -> Result<(), RhiGraphExecutionReceiptError> {
    match (range, bounds) {
        (
            RhiGraphAccessRange::Buffer { offset, size },
            RhiGraphResourceBounds::Buffer { size: bound },
        ) if size > 0 && offset.checked_add(size).is_some_and(|end| end <= bound) => Ok(()),
        (RhiGraphAccessRange::Buffer { .. }, RhiGraphResourceBounds::Buffer { .. }) => Err(
            RhiGraphExecutionReceiptError::new("access exceeds physical buffer lease"),
        ),
        (
            RhiGraphAccessRange::Texture {
                mip_start,
                mip_end,
                layer_start,
                layer_end,
                aspect_mask,
            },
            RhiGraphResourceBounds::Texture {
                mip_levels,
                array_layers,
                supported_aspects,
            },
        ) if mip_start < mip_end
            && layer_start < layer_end
            && mip_end <= mip_levels
            && layer_end <= array_layers
            && aspect_mask != 0
            && aspect_mask & !supported_aspects == 0 =>
        {
            Ok(())
        }
        (RhiGraphAccessRange::Texture { .. }, RhiGraphResourceBounds::Texture { .. }) => Err(
            RhiGraphExecutionReceiptError::new("access exceeds physical texture lease"),
        ),
        (RhiGraphAccessRange::UnresolvedExternal, _) => Err(RhiGraphExecutionReceiptError::new(
            "unresolved external access cannot be lowered to a physical lease",
        )),
        _ => Err(RhiGraphExecutionReceiptError::new(
            "graph access range kind does not match physical lease bounds",
        )),
    }
}

fn range_contains(outer: RhiGraphAccessRange, inner: RhiGraphAccessRange) -> bool {
    match (outer, inner) {
        (
            RhiGraphAccessRange::Buffer {
                offset: outer_offset,
                size: outer_size,
            },
            RhiGraphAccessRange::Buffer {
                offset: inner_offset,
                size: inner_size,
            },
        ) => {
            let Some(outer_end) = outer_offset.checked_add(outer_size) else {
                return false;
            };
            let Some(inner_end) = inner_offset.checked_add(inner_size) else {
                return false;
            };
            outer_offset <= inner_offset && inner_end <= outer_end
        }
        (
            RhiGraphAccessRange::Texture {
                mip_start: outer_mip_start,
                mip_end: outer_mip_end,
                layer_start: outer_layer_start,
                layer_end: outer_layer_end,
                aspect_mask: outer_aspects,
            },
            RhiGraphAccessRange::Texture {
                mip_start: inner_mip_start,
                mip_end: inner_mip_end,
                layer_start: inner_layer_start,
                layer_end: inner_layer_end,
                aspect_mask: inner_aspects,
            },
        ) => {
            outer_mip_start <= inner_mip_start
                && inner_mip_end <= outer_mip_end
                && outer_layer_start <= inner_layer_start
                && inner_layer_end <= outer_layer_end
                && inner_aspects & !outer_aspects == 0
        }
        (RhiGraphAccessRange::UnresolvedExternal, RhiGraphAccessRange::UnresolvedExternal) => true,
        _ => false,
    }
}

/// Immutable, device-generation-qualified command collection for one logical
/// queue submission.
///
/// Command lists keep their own render/compute pass scopes. The packet only
/// establishes the queue timeline and resource-retirement boundary shared by
/// those lists; it never exposes backend command buffers or queue objects.
pub struct RhiSubmissionPacket {
    device_id: DeviceId,
    generation: DeviceGeneration,
    queue_class: RenderQueueClass,
    command_lists: Vec<Box<dyn CommandList>>,
    diagnostic_query_plan: Option<DiagnosticQueryPlan>,
    graph_execution_receipt: Option<RhiGraphExecutionReceipt>,
}

impl RhiSubmissionPacket {
    pub fn new(
        device_id: DeviceId,
        generation: DeviceGeneration,
        queue_class: RenderQueueClass,
        command_lists: Vec<Box<dyn CommandList>>,
    ) -> Result<Self, RhiError> {
        if command_lists.is_empty() {
            return Err(RhiError::EmptySubmissionPacket);
        }
        for command_list in &command_lists {
            if command_list.queue_class() != queue_class {
                return Err(RhiError::SubmissionPacketQueueMismatch {
                    packet_queue: queue_class,
                    command_queue: command_list.queue_class(),
                });
            }
        }
        let packet = Self {
            device_id,
            generation,
            queue_class,
            command_lists,
            diagnostic_query_plan: None,
            graph_execution_receipt: None,
        };
        packet.validate_diagnostic_scopes()?;
        Ok(packet)
    }

    /// Creates a packet and attaches its diagnostic plan atomically, so scope
    /// validation cannot observe an intermediate no-plan packet.
    pub fn new_with_diagnostic_query_plan(
        device_id: DeviceId,
        generation: DeviceGeneration,
        queue_class: RenderQueueClass,
        command_lists: Vec<Box<dyn CommandList>>,
        diagnostic_query_plan: DiagnosticQueryPlan,
    ) -> Result<Self, RhiError> {
        if diagnostic_query_plan.frame_index().is_none() {
            return Err(RhiError::DiagnosticQueryFrameIndexRequired);
        }
        let packet = Self {
            device_id,
            generation,
            queue_class,
            command_lists,
            diagnostic_query_plan: Some(diagnostic_query_plan),
            graph_execution_receipt: None,
        };
        packet.validate_shape()?;
        packet.validate_diagnostic_scopes()?;
        Ok(packet)
    }

    pub const fn device_id(&self) -> DeviceId {
        self.device_id
    }

    pub const fn generation(&self) -> DeviceGeneration {
        self.generation
    }

    pub const fn queue_class(&self) -> RenderQueueClass {
        self.queue_class
    }

    pub fn command_list_count(&self) -> usize {
        self.command_lists.len()
    }

    pub fn command_lists(&self) -> &[Box<dyn CommandList>] {
        &self.command_lists
    }

    /// Attaches the bounded, frame-qualified query plan that owns every
    /// diagnostics scope recorded by this packet.
    pub fn with_diagnostic_query_plan(
        mut self,
        diagnostic_query_plan: DiagnosticQueryPlan,
    ) -> Result<Self, RhiError> {
        if diagnostic_query_plan.frame_index().is_none() {
            return Err(RhiError::DiagnosticQueryFrameIndexRequired);
        }
        self.diagnostic_query_plan = Some(diagnostic_query_plan);
        self.validate_diagnostic_scopes()?;
        Ok(self)
    }

    pub fn diagnostic_query_plan(&self) -> Option<&DiagnosticQueryPlan> {
        self.diagnostic_query_plan.as_ref()
    }

    /// Binds the graph compiler's frame/device-qualified execution proof to
    /// the command packet that will be lowered to a native queue submission.
    pub fn with_graph_execution_receipt(
        mut self,
        graph_execution_receipt: RhiGraphExecutionReceipt,
    ) -> Result<Self, RhiGraphExecutionReceiptError> {
        if self.graph_execution_receipt.is_some() {
            return Err(RhiGraphExecutionReceiptError::new(
                "submission packet already has a graph execution receipt",
            ));
        }
        if graph_execution_receipt.device_id() != self.device_id
            || graph_execution_receipt.generation() != self.generation
            || graph_execution_receipt.submission_queue() != self.queue_class
        {
            return Err(RhiGraphExecutionReceiptError::new(
                "graph receipt device generation or queue does not match submission packet",
            ));
        }
        self.graph_execution_receipt = Some(graph_execution_receipt);
        Ok(self)
    }

    pub fn graph_execution_receipt(&self) -> Option<&RhiGraphExecutionReceipt> {
        self.graph_execution_receipt.as_ref()
    }

    pub fn into_parts(
        self,
    ) -> (
        Vec<Box<dyn CommandList>>,
        Option<DiagnosticQueryPlan>,
        Option<RhiGraphExecutionReceipt>,
    ) {
        (
            self.command_lists,
            self.diagnostic_query_plan,
            self.graph_execution_receipt,
        )
    }

    pub fn into_command_lists(self) -> Vec<Box<dyn CommandList>> {
        self.command_lists
    }

    fn validate_diagnostic_scopes(&self) -> Result<(), RhiError> {
        let scopes = self
            .command_lists
            .iter()
            .flat_map(|command_list| command_list.recorded_commands())
            .filter_map(command_scope)
            .collect::<Vec<_>>();
        if scopes.iter().any(|scope| scope.is_empty()) {
            return Err(RhiError::EmptyDiagnosticPassScope);
        }
        match self.diagnostic_query_plan.as_ref() {
            Some(plan) => plan
                .validate_submission_scopes(&scopes)
                .map_err(RhiError::from),
            None if scopes.is_empty() => Ok(()),
            None => Err(RhiError::DiagnosticQueryPlanRequired),
        }
    }

    fn validate_shape(&self) -> Result<(), RhiError> {
        if self.command_lists.is_empty() {
            return Err(RhiError::EmptySubmissionPacket);
        }
        for command_list in &self.command_lists {
            if command_list.queue_class() != self.queue_class {
                return Err(RhiError::SubmissionPacketQueueMismatch {
                    packet_queue: self.queue_class,
                    command_queue: command_list.queue_class(),
                });
            }
        }
        Ok(())
    }
}

fn command_scope(command: &CommandListCommand) -> Option<DiagnosticPassQueryScope> {
    match command {
        CommandListCommand::BeginRenderPassWithDiagnostics {
            diagnostic_scope, ..
        }
        | CommandListCommand::BeginComputePassWithDiagnostics {
            diagnostic_scope, ..
        } => Some(*diagnostic_scope),
        _ => None,
    }
}
