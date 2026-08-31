/// Why one frame-owned logical packet must end at a physical backend submission boundary.
///
/// A producer category describes the work. This reason describes the ordering constraint that
/// prevents the packet from being merged with the work that follows it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum RenderFrameSubmissionBoundaryReason {
    /// Existing resident mips must be copied into a replacement texture before queue writes upload
    /// newly requested mips into that replacement.
    TextureMipPreservationBeforeUpload,
}
