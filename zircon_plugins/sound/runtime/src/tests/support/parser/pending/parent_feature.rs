// 依赖与模块只能附着到当前功能；缺少父功能是清单结构错误，测试应直接失败。
use super::super::super::types::PendingOptionalFeatureManifest;

pub(super) fn required_parent_feature<'a>(
    feature: &'a mut Option<PendingOptionalFeatureManifest>,
    message: &'static str,
) -> &'a mut PendingOptionalFeatureManifest {
    feature.as_mut().expect(message)
}
