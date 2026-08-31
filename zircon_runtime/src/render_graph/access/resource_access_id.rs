use crate::render_graph::RenderPassId;

/// Stable identity of one declared access within a compiled graph pass.
///
/// The pass handle and authoring access ordinal survive topological reordering;
/// resource name and read/write kind deliberately do not identify an access.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RenderGraphResourceAccessId {
    pass: RenderPassId,
    access_index: usize,
}

impl RenderGraphResourceAccessId {
    pub(crate) const fn new(pass: RenderPassId, access_index: usize) -> Self {
        Self { pass, access_index }
    }

    pub const fn pass(self) -> RenderPassId {
        self.pass
    }

    pub const fn access_index(self) -> usize {
        self.access_index
    }
}
