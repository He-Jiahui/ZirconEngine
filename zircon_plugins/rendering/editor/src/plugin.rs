//! 把运行时包清单附加编辑器模块，供宿主发现渲染主包的编辑器身份。
use crate::{CAPABILITY, PLUGIN_ID};

/// 编辑器侧的主包描述符载体；当前 trait 实现没有注册面板或事件消费者。
#[derive(Clone, Debug)]
pub struct RenderingEditorPlugin {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl RenderingEditorPlugin {
    /// 构造元数据实例；该入口不会建立渲染设备或启用可选特性。
    pub fn new() -> Self {
        Self {
            descriptor: editor_plugin_descriptor(),
        }
    }
}

impl zircon_editor::EditorPlugin for RenderingEditorPlugin {
    fn descriptor(&self) -> &zircon_editor::EditorPluginDescriptor {
        &self.descriptor
    }
}

/// 描述主包的编辑器模块与能力，供包清单投影和外部宿主查询。
pub fn editor_plugin_descriptor() -> zircon_editor::EditorPluginDescriptor {
    zircon_editor::EditorPluginDescriptor::new(
        PLUGIN_ID,
        "Rendering",
        "zircon_plugin_rendering_editor",
    )
    .with_capability(CAPABILITY)
}

/// 公共构造入口，返回可通过编辑器 trait 查询的独立实例。
pub fn editor_plugin() -> RenderingEditorPlugin {
    RenderingEditorPlugin::new()
}

/// 在运行时主包清单上附加编辑器模块；主包及其可选特性的身份仍来自运行时声明。
pub fn package_manifest() -> zircon_runtime::plugin::PluginPackageManifest {
    zircon_editor::EditorPlugin::package_manifest(
        &editor_plugin(),
        zircon_plugin_rendering_runtime::package_manifest(),
    )
}

/// 提供不依赖临时插件实例生命周期的能力快照，供宿主进行发现与选择。
pub fn editor_capabilities() -> Vec<String> {
    zircon_editor::EditorPlugin::editor_capabilities(&editor_plugin()).to_vec()
}
