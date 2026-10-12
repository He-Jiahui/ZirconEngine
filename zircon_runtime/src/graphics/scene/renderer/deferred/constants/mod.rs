#[path = "../../../deferred/constants/gbuffer_albedo_format.rs"]
mod gbuffer_albedo_format;
#[path = "../../../deferred/constants/gbuffer_emissive_format.rs"]
mod gbuffer_emissive_format;
#[path = "../../../deferred/constants/gbuffer_material_format.rs"]
mod gbuffer_material_format;

pub(crate) use gbuffer_albedo_format::GBUFFER_ALBEDO_FORMAT;
pub(crate) use gbuffer_emissive_format::GBUFFER_EMISSIVE_FORMAT;
pub(crate) use gbuffer_material_format::GBUFFER_MATERIAL_FORMAT;
