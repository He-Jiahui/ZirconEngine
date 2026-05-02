use crate::core::framework::render::RenderHybridGiExtract;

pub fn enabled_hybrid_gi_extract(
    extract: Option<&RenderHybridGiExtract>,
) -> Option<&RenderHybridGiExtract> {
    extract.filter(|extract| extract.enabled)
}

pub fn hybrid_gi_extract_uses_scene_representation_budget(extract: &RenderHybridGiExtract) -> bool {
    extract.trace_budget > 0 || extract.card_budget > 0 || extract.voxel_budget > 0
}
