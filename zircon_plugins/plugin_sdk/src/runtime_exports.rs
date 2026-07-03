//! Runtime plugin export helper macros.

#[macro_export]
macro_rules! runtime_plugin_exports {
    ($plugin_ty:ty) => {
        pub fn runtime_plugin() -> $plugin_ty {
            <$plugin_ty>::new()
        }

        pub fn package_manifest() -> zircon_runtime::plugin::PluginPackageManifest {
            zircon_runtime::plugin::RuntimePlugin::package_manifest(&runtime_plugin())
        }

        pub fn runtime_selection() -> zircon_runtime::plugin::ProjectPluginSelection {
            zircon_runtime::plugin::RuntimePlugin::project_selection(&runtime_plugin())
        }

        pub fn plugin_registration() -> zircon_runtime::plugin::RuntimePluginRegistrationReport {
            zircon_runtime::plugin::RuntimePluginRegistrationReport::from_plugin(&runtime_plugin())
        }
    };
    ($plugin_ty:ty, $constructor:expr) => {
        pub fn runtime_plugin() -> $plugin_ty {
            $constructor
        }

        pub fn package_manifest() -> zircon_runtime::plugin::PluginPackageManifest {
            zircon_runtime::plugin::RuntimePlugin::package_manifest(&runtime_plugin())
        }

        pub fn runtime_selection() -> zircon_runtime::plugin::ProjectPluginSelection {
            zircon_runtime::plugin::RuntimePlugin::project_selection(&runtime_plugin())
        }

        pub fn plugin_registration() -> zircon_runtime::plugin::RuntimePluginRegistrationReport {
            zircon_runtime::plugin::RuntimePluginRegistrationReport::from_plugin(&runtime_plugin())
        }
    };
}

#[cfg(test)]
mod tests {
    const PLUGIN_ID: &str = "animation";
    const PLUGIN_CRATE: &str = "zircon_plugin_sdk_runtime_export";
    const MANIFEST_OVERRIDE_DESCRIPTION: &str = "manifest override from runtime plugin";

    #[derive(Clone, Debug)]
    struct SdkRuntimeExportPlugin {
        descriptor: zircon_runtime::plugin::RuntimePluginDescriptor,
    }

    impl SdkRuntimeExportPlugin {
        fn new() -> Self {
            Self {
                descriptor: zircon_runtime::plugin::RuntimePluginDescriptor::builder(
                    PLUGIN_ID,
                    "SDK Runtime Export",
                    zircon_runtime::builtin::RuntimePluginId::Animation,
                    PLUGIN_CRATE,
                )
                .with_category("runtime")
                .with_init_level(zircon_runtime::core::InitLevel::Scene)
                .with_module_dependency(zircon_runtime::core::ModuleDependencySpec::named(
                    "animation.runtime",
                ))
                .with_target_modes([zircon_runtime::builtin::RuntimeTargetMode::ClientRuntime])
                .with_capability("runtime.plugin.sdk_runtime_export")
                .build(),
            }
        }
    }

    impl zircon_runtime::plugin::RuntimePlugin for SdkRuntimeExportPlugin {
        fn descriptor(&self) -> &zircon_runtime::plugin::RuntimePluginDescriptor {
            &self.descriptor
        }

        fn package_manifest(&self) -> zircon_runtime::plugin::PluginPackageManifest {
            let mut manifest =
                zircon_runtime::plugin::RuntimePlugin::descriptor(self).package_manifest();
            manifest.description = MANIFEST_OVERRIDE_DESCRIPTION.to_string();
            manifest
        }
    }

    mod generated_exports {
        use super::SdkRuntimeExportPlugin;

        crate::runtime_plugin_exports!(SdkRuntimeExportPlugin);
    }

    #[test]
    fn runtime_plugin_exports_macro_generates_standard_helpers() {
        let plugin = generated_exports::runtime_plugin();
        assert_eq!(plugin.descriptor.package_id(), PLUGIN_ID);

        let manifest = generated_exports::package_manifest();
        assert_eq!(manifest.id, PLUGIN_ID);
        assert_eq!(manifest.description, MANIFEST_OVERRIDE_DESCRIPTION);
        assert_eq!(
            manifest.modules[0].init_level,
            zircon_runtime::core::InitLevel::Scene
        );
        assert_eq!(
            manifest.modules[0].module_dependencies,
            [zircon_runtime::core::ModuleDependencySpec::named(
                "animation.runtime"
            )]
        );

        let selection = generated_exports::runtime_selection();
        assert_eq!(selection.id, PLUGIN_ID);
        assert_eq!(selection.runtime_crate.as_deref(), Some(PLUGIN_CRATE));

        let report = generated_exports::plugin_registration();
        assert!(report.is_success(), "{:?}", report.diagnostics);
        assert_eq!(
            report.package_manifest.description,
            MANIFEST_OVERRIDE_DESCRIPTION
        );
    }
}
