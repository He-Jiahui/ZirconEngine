/// A static byte window within a graph buffer access.
///
/// `None` means from `offset` through the end of the declared buffer. Dynamic
/// offsets remain command-recording state and never enter compiled graph metadata.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RenderGraphBufferRange {
    pub offset: u64,
    pub size: Option<u64>,
}

impl RenderGraphBufferRange {
    pub const fn full() -> Self {
        Self {
            offset: 0,
            size: None,
        }
    }

    pub const fn new(offset: u64, size: Option<u64>) -> Self {
        Self { offset, size }
    }
}
