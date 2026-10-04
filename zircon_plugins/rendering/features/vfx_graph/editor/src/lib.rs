//! 视效图的编辑器公共入口；宿主可查询能力与清单，具体编辑器扩展仍由注册契约决定。
mod capability;
mod plugin;

pub use capability::{CAPABILITY, EDITOR_CAPABILITIES, FEATURE_ID};
pub use plugin::{
    editor_capabilities, editor_feature, feature_manifest, RenderingVfxGraphEditorFeature,
};
/// 预留的视效图资产视图身份；编辑器提供者当前没有注册具体视图。
pub const VFX_GRAPH_ASSET_VIEW_ID: &str = "rendering.vfx_graph.asset";
