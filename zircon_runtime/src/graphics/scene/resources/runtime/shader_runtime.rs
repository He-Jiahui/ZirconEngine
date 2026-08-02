use crate::asset::ShaderImportRedirectAsset;
use crate::core::framework::render::{MaterialOptionTable, ShaderAssetKind};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub(in crate::graphics::scene::resources) struct ShaderRuntime {
    pub(in crate::graphics::scene::resources) source: Arc<str>,
    pub(in crate::graphics::scene::resources) kind: ShaderAssetKind,
    pub(in crate::graphics::scene::resources) import_path: Option<String>,
    pub(in crate::graphics::scene::resources) imports: Vec<ShaderImportRedirectAsset>,
    pub(in crate::graphics::scene::resources) material_option_table: MaterialOptionTable,
    pub(in crate::graphics::scene::resources) generated_material_wgsl: String,
}
