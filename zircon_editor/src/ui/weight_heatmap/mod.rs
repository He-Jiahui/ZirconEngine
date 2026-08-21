mod generation;

pub(crate) use generation::{
    WeightHeatmapGeneration, WeightHeatmapGenerationInput, WeightHeatmapSource,
};

#[cfg(test)]
pub(crate) use generation::{static_field_cache_entry_count, STATIC_FIELD_CACHE_CAPACITY};

#[cfg(test)]
mod tests;
