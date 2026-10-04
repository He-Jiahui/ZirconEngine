//! Editor-side plugin authoring helpers.
//! 声明可镜像运行时包清单，再叠加编辑器能力、资源根目录与运行时事件消费者。

pub use zircon_editor;
pub use zircon_runtime;

use zircon_editor::{
    core::runtime_event_consumer::{
        EditorRuntimeEventConsumerRegistration, EditorRuntimeEventConsumerRegistry,
    },
    EditorPlugin, EditorPluginDescriptor, EditorPluginRegistrationReport,
};
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::plugin::{PluginMaturity, PluginPackageManifest, PluginPackageRole};

use crate::{PluginManifestBuilder, RuntimePluginDeclaration};

#[derive(Clone, Debug)]
/// 汇总编辑器插件 descriptor、包清单投影、事件消费者注册结果与诊断。
pub struct EditorPluginDeclaration {
    descriptor: EditorPluginDescriptor,
    base_manifest: PluginPackageManifest,
    mirrored_runtime_package_id: Option<String>,
    runtime_event_consumers: EditorRuntimeEventConsumerRegistry,
    diagnostics: Vec<String>,
}

impl EditorPluginDeclaration {
    /// 创建面向 EditorHost 的声明，并以该目标初始化包清单。
    pub fn new(
        package_id: impl Into<String>,
        display_name: impl Into<String>,
        crate_name: impl Into<String>,
    ) -> Self {
        let package_id = package_id.into();
        let display_name = display_name.into();
        let crate_name = crate_name.into();
        Self {
            descriptor: EditorPluginDescriptor::new(
                package_id.clone(),
                display_name.clone(),
                crate_name,
            ),
            base_manifest: PluginManifestBuilder::new(package_id, display_name)
                .with_supported_targets([RuntimeTargetMode::EditorHost])
                .build(),
            mirrored_runtime_package_id: None,
            runtime_event_consumers: EditorRuntimeEventConsumerRegistry::default(),
            diagnostics: Vec::new(),
        }
    }

    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        let category = category.into();
        self.descriptor = self.descriptor.with_category(category.clone());
        self.base_manifest = self.base_manifest.with_category(category);
        self
    }

    pub fn with_package_role(mut self, package_role: PluginPackageRole) -> Self {
        self.base_manifest = self.base_manifest.with_package_role(package_role);
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.base_manifest.description = description.into();
        self
    }

    pub fn with_maturity(mut self, maturity: PluginMaturity) -> Self {
        self.base_manifest = self.base_manifest.with_maturity(maturity);
        self
    }

    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        let capability = capability.into();
        self.descriptor = self.descriptor.with_capability(capability.clone());
        self.base_manifest = self.base_manifest.with_capability(capability);
        self
    }

    pub fn with_capabilities<I, S>(mut self, capabilities: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for capability in capabilities {
            self = self.with_capability(capability);
        }
        self
    }

    /// 注册成功时同步更新 descriptor；重复或冲突注册会保留为诊断。
    pub fn with_runtime_event_consumer_registration(
        mut self,
        registration: EditorRuntimeEventConsumerRegistration,
    ) -> Self {
        let manifest = registration.manifest().clone();
        match self.runtime_event_consumers.register(registration) {
            Ok(()) => {
                self.descriptor = self.descriptor.with_event_consumer(manifest);
            }
            Err(error) => self.diagnostics.push(error.to_string()),
        }
        self
    }

    pub fn mirrors_runtime(mut self, runtime: &RuntimePluginDeclaration) -> Self {
        self = self.mirrors_runtime_manifest(runtime.package_manifest());
        self
    }

    /// 用运行时包清单替换当前底稿，再并入编辑器能力及此前配置的资源根目录。
    ///
    /// 能力与根目录按值去重，包 ID 同步为运行时清单中的 ID。
    pub fn mirrors_runtime_manifest(mut self, runtime_manifest: PluginPackageManifest) -> Self {
        let editor_capabilities = self.descriptor.capabilities.clone();
        let asset_roots = std::mem::take(&mut self.base_manifest.asset_roots);
        let content_roots = std::mem::take(&mut self.base_manifest.content_roots);
        let mut base_manifest = runtime_manifest;
        self.descriptor.package_id = base_manifest.id.clone();
        self.mirrored_runtime_package_id = Some(base_manifest.id.clone());
        for capability in editor_capabilities {
            push_unique(&mut base_manifest.capabilities, capability);
        }
        merge_unique(&mut base_manifest.asset_roots, asset_roots);
        merge_unique(&mut base_manifest.content_roots, content_roots);
        self.base_manifest = base_manifest;
        self
    }

    pub fn with_asset_root(mut self, asset_root: impl Into<String>) -> Self {
        self.base_manifest = self.base_manifest.with_asset_root(asset_root);
        self
    }

    pub fn with_content_root(mut self, content_root: impl Into<String>) -> Self {
        self.base_manifest = self.base_manifest.with_content_root(content_root);
        self
    }

    pub fn descriptor(&self) -> &EditorPluginDescriptor {
        &self.descriptor
    }

    pub fn base_manifest(&self) -> PluginPackageManifest {
        self.base_manifest.clone()
    }

    /// 将编辑器 descriptor 附加到底稿，形成供编辑器插件注册报告使用的包清单。
    pub fn package_manifest(&self) -> PluginPackageManifest {
        self.descriptor.attach_to_package(self.base_manifest())
    }

    pub fn capabilities(&self) -> &[String] {
        &self.descriptor.capabilities
    }

    pub fn mirrored_runtime_package_id(&self) -> Option<&str> {
        self.mirrored_runtime_package_id.as_deref()
    }

    pub fn runtime_event_consumers(&self) -> EditorRuntimeEventConsumerRegistry {
        self.runtime_event_consumers.clone()
    }

    /// 由插件 trait 实现生成注册报告，并把声明阶段收集的诊断追加到报告。
    pub fn registration_report(&self, plugin: &dyn EditorPlugin) -> EditorPluginRegistrationReport {
        let mut report = EditorPluginRegistrationReport::from_plugin(plugin, self.base_manifest());
        report.diagnostics.extend(self.diagnostics.iter().cloned());
        report
    }
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.iter().any(|existing| existing == &value) {
        values.push(value);
    }
}

fn merge_unique(values: &mut Vec<String>, incoming: Vec<String>) {
    if values.is_empty() {
        *values = incoming;
        return;
    }
    for value in incoming {
        push_unique(values, value);
    }
}

#[macro_export]
/// 按静态声明生成编辑器插件类型、descriptor/清单访问器与 EditorPlugin 实现。
///
/// 可选 runtime 清单、能力、事件消费者和资源根会在默认构造时累积到同一声明中。
macro_rules! authoring_plugin {
    (
        $(#[$meta:meta])*
        $vis:vis struct $plugin_ty:ident {
            package_id: $package_id:expr,
            display_name: $display_name:expr,
            crate_name: $crate_name:expr,
            category: $category:expr,
            description: $description:expr,
            maturity: $maturity:expr,
            $(mirrors_runtime: $runtime_declaration:expr,)?
            $(mirrors_runtime_manifest: $runtime_manifest:expr,)?
            capabilities: $capabilities:expr,
            $(runtime_event_consumers: $runtime_event_consumers:expr,)?
            $(asset_root: $asset_root:expr,)?
            $(content_root: $content_root:expr,)?
            register_extensions: $register_extensions:path $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Debug)]
        $vis struct $plugin_ty {
            declaration: $crate::editor::EditorPluginDeclaration,
        }

        impl $plugin_ty {
            pub fn new() -> Self {
                Self::default()
            }

            pub fn declaration(&self) -> &$crate::editor::EditorPluginDeclaration {
                &self.declaration
            }

            pub fn package_manifest(
                &self,
            ) -> $crate::editor::zircon_runtime::plugin::PluginPackageManifest {
                self.declaration.package_manifest()
            }

            pub fn editor_capabilities(&self) -> ::std::vec::Vec<::std::string::String> {
                self.declaration.capabilities().to_vec()
            }

            pub fn registration_report(
                &self,
            ) -> $crate::editor::zircon_editor::EditorPluginRegistrationReport {
                self.declaration.registration_report(self)
            }
        }

        impl ::std::default::Default for $plugin_ty {
            // 先创建编辑器声明，再镜像运行时清单，最后依次并入能力、消费者和资源根。
            fn default() -> Self {
                let declaration = $crate::editor::EditorPluginDeclaration::new(
                    $package_id,
                    $display_name,
                    $crate_name,
                )
                .with_category($category)
                .with_description($description)
                .with_maturity($maturity);
                $(
                    let declaration = declaration.mirrors_runtime(&$runtime_declaration);
                )?
                $(
                    let declaration = declaration.mirrors_runtime_manifest($runtime_manifest);
                )?
                let declaration = declaration.with_capabilities(($capabilities).iter().copied());
                $(
                    let declaration = ($runtime_event_consumers).into_iter().fold(
                        declaration,
                        |declaration, registration| {
                            declaration.with_runtime_event_consumer_registration(registration)
                        },
                    );
                )?
                $(
                    let declaration = declaration.with_asset_root($asset_root);
                )?
                $(
                    let declaration = declaration.with_content_root($content_root);
                )?
                Self { declaration }
            }
        }

        impl $crate::editor::zircon_editor::EditorPlugin for $plugin_ty {
            fn descriptor(&self) -> &$crate::editor::zircon_editor::EditorPluginDescriptor {
                self.declaration.descriptor()
            }

            fn register_editor_extensions(
                &self,
                registry: &mut $crate::editor::zircon_editor::core::editor_extension::EditorExtensionRegistry,
            ) -> ::std::result::Result<
                (),
                $crate::editor::zircon_editor::core::editor_extension::EditorExtensionRegistryError,
            > {
                $register_extensions(registry)
            }

            fn runtime_event_consumers(
                &self,
            ) -> $crate::editor::zircon_editor::core::runtime_event_consumer::EditorRuntimeEventConsumerRegistry {
                self.declaration.runtime_event_consumers()
            }
        }
    };
}

pub use crate::authoring_plugin;

#[cfg(test)]
#[path = "tests/editor.rs"]
mod tests;
