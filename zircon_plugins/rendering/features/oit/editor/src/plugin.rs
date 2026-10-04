//! 将无序透明的编辑器身份投影到宿主插件契约；本文件只提供元数据，不安装编辑器扩展。
use crate::{CAPABILITY, FEATURE_ID};

/// 宿主查询用的编辑器特性实例；持有描述符，不持有运行时渲染资源。
#[derive(Clone, Debug)]
pub struct RenderingOitEditorFeature {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl RenderingOitEditorFeature {
    /// 构造供能力和清单查询使用的独立实例；构造本身不启用运行时效果。
    pub fn new() -> Self {
        Self {
            descriptor: zircon_editor::EditorPluginDescriptor::new(
                FEATURE_ID,
                "Order Independent Transparency",
                "zircon_plugin_rendering_oit_editor",
            )
            .with_capability(CAPABILITY),
        }
    }
}

impl Default for RenderingOitEditorFeature {
    fn default() -> Self {
        Self::new()
    }
}

impl zircon_editor::EditorPlugin for RenderingOitEditorFeature {
    fn descriptor(&self) -> &zircon_editor::EditorPluginDescriptor {
        &self.descriptor
    }
}

/// 提供公共构造入口，供宿主或外部集成通过编辑器 trait 查询此特性。
pub fn editor_feature() -> RenderingOitEditorFeature {
    RenderingOitEditorFeature::new()
}

/// 返回脱离临时实例的能力快照；调用方还需按项目选择决定是否加载该模块。
pub fn editor_capabilities() -> Vec<String> {
    zircon_editor::EditorPlugin::editor_capabilities(&editor_feature()).to_vec()
}

/// 复用运行时特性清单作为跨端身份来源，避免编辑器另建一份依赖关系。
pub fn feature_manifest() -> zircon_runtime::plugin::PluginFeatureBundleManifest {
    zircon_plugin_rendering_oit_runtime::feature_manifest()
}
