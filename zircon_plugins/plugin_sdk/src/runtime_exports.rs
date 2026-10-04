//! Runtime plugin export helper macros.
//! 生成 Rust Runtime 插件目录使用的辅助函数；原生动态库符号由 `dist` 模块负责。

/// 为插件类型生成实例、包清单、项目选择和注册报告的统一 Rust 入口。
/// 每次取值都会构造新实例；包清单经 `RuntimePlugin` trait 分派，保留插件自身的覆盖实现。
#[macro_export]
macro_rules! runtime_plugin_exports {
    ($plugin_ty:ty) => {
        pub fn runtime_plugin() -> $plugin_ty {
            <$plugin_ty>::new()
        }

        pub fn package_manifest() -> zircon_runtime::plugin::PluginPackageManifest {
            zircon_runtime::plugin::RuntimePlugin::package_manifest(&runtime_plugin())
        }

        pub fn runtime_selection(
        ) -> zircon_runtime::core::framework::project::ProjectPluginSelection {
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

        pub fn runtime_selection(
        ) -> zircon_runtime::core::framework::project::ProjectPluginSelection {
            zircon_runtime::plugin::RuntimePlugin::project_selection(&runtime_plugin())
        }

        pub fn plugin_registration() -> zircon_runtime::plugin::RuntimePluginRegistrationReport {
            zircon_runtime::plugin::RuntimePluginRegistrationReport::from_plugin(&runtime_plugin())
        }
    };
}

#[cfg(test)]
#[path = "tests/runtime_exports.rs"]
mod tests;
