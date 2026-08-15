mod generation;

pub(crate) use generation::{
    TimelineStripGeneration, TimelineStripGenerationInput, TimelineStripKey,
    TimelineStripStaticContent, TimelineStripTick,
};

#[cfg(test)]
pub(crate) use generation::{STATIC_CONTENT_CACHE_CAPACITY, static_content_cache_entry_count};

#[cfg(test)]
mod tests;
