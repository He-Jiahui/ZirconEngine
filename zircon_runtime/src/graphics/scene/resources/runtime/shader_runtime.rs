use crate::asset::{ShaderImportRedirectAsset, ShaderSurfaceSourceContract};
use crate::core::framework::render::{MaterialOptionTable, ShaderAssetKind};
use std::sync::Arc;

/// 单次 shader 资产解析后的运行时契约快照；材质准备与布局诊断须读取相同的源、导入重定向和生成 WGSL。
#[derive(Clone, Debug)]
pub(in crate::graphics::scene::resources) struct ShaderRuntime {
    pub(in crate::graphics::scene::resources) source: Arc<str>,
    pub(in crate::graphics::scene::resources) kind: ShaderAssetKind,
    pub(in crate::graphics::scene::resources) surface_source_contract:
        Option<ShaderSurfaceSourceContract>,
    pub(in crate::graphics::scene::resources) import_path: Option<String>,
    pub(in crate::graphics::scene::resources) imports: Vec<ShaderImportRedirectAsset>,
    pub(in crate::graphics::scene::resources) material_option_table: MaterialOptionTable,
    pub(in crate::graphics::scene::resources) generated_material_wgsl: String,
}
