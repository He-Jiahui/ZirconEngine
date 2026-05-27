mod alpha_mode;
mod dependency_set;
mod material_asset;
mod property_values;
mod texture_slot;
mod validation;
mod zmaterial;

pub use alpha_mode::AlphaMode;
pub use material_asset::MaterialAsset;
pub use property_values::shader_property_values_for_shader;
pub use texture_slot::MaterialTextureSlotValue;
pub use validation::{validate_alpha_mode, validate_shader_contract, validate_wgsl_captures};
pub use zmaterial::ZMaterialDocument;
