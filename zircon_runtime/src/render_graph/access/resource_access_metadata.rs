use super::{RenderGraphResourceAccessIntent, RenderGraphResourceAccessRange};

/// Immutable range and use intent attached to one declared graph access.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RenderGraphResourceAccessMetadata {
    pub range: RenderGraphResourceAccessRange,
    pub intent: RenderGraphResourceAccessIntent,
}

impl RenderGraphResourceAccessMetadata {
    pub const fn new(
        range: RenderGraphResourceAccessRange,
        intent: RenderGraphResourceAccessIntent,
    ) -> Self {
        Self { range, intent }
    }
}
