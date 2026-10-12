#[path = "alpha_mode.rs"]
mod alpha_mode;
#[path = "default_pbr.rs"]
mod default_pbr;
#[path = "dependency_set.rs"]
mod dependency_set;
#[path = "material_asset.rs"]
mod material_asset;
#[path = "material_control.rs"]
mod material_control;
#[path = "property_values.rs"]
mod property_values;
#[path = "texture_slot.rs"]
mod texture_slot;
#[path = "validation.rs"]
mod validation;
#[path = "zmaterial.rs"]
mod zmaterial;

pub use alpha_mode::AlphaMode;
pub use default_pbr::{default_pbr_shader_reference, DEFAULT_PBR_SHADER_URI};
pub use material_asset::{
    MaterialAsset, MaterialAssetManagementRecord, MaterialAssetManagementRecordSet,
    MaterialAssetManagementRecordSetSummary, MaterialAssetOverview,
};
pub use material_control::{
    STANDARD_MATERIAL_NORMAL_SCALE_PROPERTY, STANDARD_MATERIAL_OCCLUSION_STRENGTH_PROPERTY,
};
pub use property_values::shader_property_values_for_shader;
pub(super) use texture_slot::is_standard_texture_slot_alias;
pub use texture_slot::MaterialTextureSlotValue;
pub use validation::{
    validate_alpha_mode, validate_render_queue_alpha_mode, validate_shader_contract,
    validate_standard_material_texture_uv_channels, validate_wgsl_captures,
};
pub(super) use zmaterial::validate_zmaterial_version;
pub use zmaterial::{ZMaterialDocument, ZMaterialQueueOverride};
