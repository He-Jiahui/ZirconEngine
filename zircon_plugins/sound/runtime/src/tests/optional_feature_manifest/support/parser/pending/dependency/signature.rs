// 依赖签名保留插件 ID、能力与主依赖标志，供静态与运行时功能清单逐项比较。
mod required;

use super::super::super::super::types::OptionalFeatureDependencySignature;

pub(super) fn take_optional_feature_dependency(
    plugin_id: &mut Option<String>,
    capability: &mut Option<String>,
    primary: &mut Option<bool>,
) -> Option<OptionalFeatureDependencySignature> {
    let plugin_id = plugin_id.take()?;
    Some((
        plugin_id,
        required::take_required_capability(capability),
        required::take_required_primary(primary),
    ))
}
