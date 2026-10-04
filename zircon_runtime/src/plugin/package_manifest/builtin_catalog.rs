//! 把内建运行时描述符投影为可序列化的包声明，补齐对应编辑器 crate 名称。
//! 此投影不执行插件，也不替代目录注册、包校验或实际发行清单。
use crate::plugin::RuntimePluginDescriptor;

use super::PluginPackageManifest;

fn exact_builtin_editor_crate_name(package_id: &str) -> String {
    let capacity = "zircon_plugin_".len() + package_id.len() + "_editor".len();
    let mut crate_name = String::with_capacity(capacity);
    crate_name.push_str("zircon_plugin_");
    crate_name.push_str(package_id);
    crate_name.push_str("_editor");
    crate_name
}

impl PluginPackageManifest {
    /// 为清单消费者生成内建包视图；每次调用创建独立集合，编辑器模块名沿用包标识。
    pub fn builtin_catalog() -> Vec<Self> {
        RuntimePluginDescriptor::builtin_catalog()
            .into_iter()
            .map(|descriptor| {
                descriptor
                    .package_manifest()
                    .with_editor_crate(exact_builtin_editor_crate_name(descriptor.package_id()))
            })
            .collect()
    }
}

#[cfg(test)]
#[path = "tests/builtin_catalog.rs"]
mod tests;
