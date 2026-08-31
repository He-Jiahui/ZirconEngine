use crate::render_graph::{
    RenderGraphResource, RenderGraphResourceAccessId, RenderGraphResourceAccessKind,
    RenderGraphResourceAccessMetadata, RenderGraphResourceAccessRange, RenderGraphResourceVersion,
};

/// Immutable logical binding identity for one compiled resource access.
///
/// It is backend-neutral and survives graph scheduling. Materialization later
/// pairs this key with a device-qualified physical allocation and view/slice;
/// WGPU objects never enter the compiled graph.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RenderGraphVersionedAccessKey {
    pub access_id: RenderGraphResourceAccessId,
    pub resource: RenderGraphResource,
    pub access: RenderGraphResourceAccessKind,
    pub version: RenderGraphResourceVersion,
    pub range: RenderGraphResourceAccessRange,
    pub intent: crate::render_graph::RenderGraphResourceAccessIntent,
}

impl RenderGraphVersionedAccessKey {
    pub(crate) const fn new(
        access_id: RenderGraphResourceAccessId,
        resource: RenderGraphResource,
        access: RenderGraphResourceAccessKind,
        version: RenderGraphResourceVersion,
        metadata: RenderGraphResourceAccessMetadata,
    ) -> Self {
        Self {
            access_id,
            resource,
            access,
            version,
            range: metadata.range,
            intent: metadata.intent,
        }
    }
}
