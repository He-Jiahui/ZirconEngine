use std::fmt;

use super::ProductConfigSource;

/// Compact provenance set for resolved fields that merge more than one configuration source.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct ProductConfigSourceSet(u8);

impl ProductConfigSourceSet {
    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn single(source: ProductConfigSource) -> Self {
        Self(source_bit(source))
    }

    pub const fn with(self, source: ProductConfigSource) -> Self {
        Self(self.0 | source_bit(source))
    }

    pub const fn contains(self, source: ProductConfigSource) -> bool {
        self.0 & source_bit(source) != 0
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl fmt::Debug for ProductConfigSourceSet {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut sources = formatter.debug_set();
        for source in [
            ProductConfigSource::ProductRole,
            ProductConfigSource::RuntimeProfile,
            ProductConfigSource::EntryRequest,
            ProductConfigSource::ExportProfile,
        ] {
            if self.contains(source) {
                sources.entry(&source);
            }
        }
        sources.finish()
    }
}

const fn source_bit(source: ProductConfigSource) -> u8 {
    match source {
        ProductConfigSource::ProductRole => 1 << 0,
        ProductConfigSource::RuntimeProfile => 1 << 1,
        ProductConfigSource::EntryRequest => 1 << 2,
        ProductConfigSource::ExportProfile => 1 << 3,
    }
}
