mod capability;
mod pair;
mod provider;

use crate::plugin::PluginDependencyManifest;

use self::capability::validate_runtime_plugin_package_dependency_row_capability;
use self::pair::validate_runtime_plugin_package_dependency_row_pair;
use self::provider::validate_runtime_plugin_package_dependency_provider;

/// 依赖可以请求能力、接口或两者；接口的格式及局部重复由独立接口校验分支检查。
/// 提供者身份在这里校验声明形状，其存在性和必需性随后由目录解析处理。
pub(super) fn validate_runtime_plugin_package_dependency_row(
    dependency: &PluginDependencyManifest,
    is_duplicate: bool,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_package_dependency_provider(dependency, diagnostics);
    if dependency.capability.is_none() && dependency.interfaces.is_empty() {
        diagnostics.push(format!(
            "runtime plugin package manifest dependency `{}` must declare a capability or at least one interface",
            dependency.id
        ));
        return;
    }
    let Some(capability) =
        validate_runtime_plugin_package_dependency_row_capability(dependency, diagnostics)
    else {
        return;
    };
    validate_runtime_plugin_package_dependency_row_pair(
        dependency.id.as_str(),
        capability,
        is_duplicate,
        diagnostics,
    );
}
