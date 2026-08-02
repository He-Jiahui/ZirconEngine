use serde::{Deserialize, Serialize};

/// An OpenType feature setting supplied through the neutral text service contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TextOpenTypeFeature {
    pub tag: [u8; 4],
    pub value: u32,
}

impl TextOpenTypeFeature {
    pub const fn new(tag: [u8; 4], value: u32) -> Self {
        Self { tag, value }
    }
}
