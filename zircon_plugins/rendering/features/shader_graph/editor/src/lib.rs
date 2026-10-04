//! 着色器图的编辑器公共入口；宿主可查询能力与清单，具体编辑器扩展仍由注册契约决定。
mod capability;
mod plugin;

pub use capability::{CAPABILITY, EDITOR_CAPABILITIES, FEATURE_ID};
pub use plugin::{
    editor_capabilities, editor_feature, feature_manifest, RenderingShaderGraphEditorFeature,
};
/// 预留的资产视图身份；编辑器特性当前仅导出元数据，尚未据此注册视图。
pub const SHADER_GRAPH_ASSET_VIEW_ID: &str = "rendering.shader_graph.asset";
