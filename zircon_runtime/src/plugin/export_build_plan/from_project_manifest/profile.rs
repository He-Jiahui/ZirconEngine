//! 导出 profile 的运行时描述符查找入口；后续目标模式校验依赖这里返回的确切 descriptor。
use crate::core::framework::project::ExportProfile;
use crate::plugin::RuntimeProfileDescriptor;

/// 有明确 RuntimeProfileId 才解析 descriptor；缺失 id 的 fatal 由 profile validation 负责。
pub(super) fn runtime_profile_for_export_profile(
    profile: &ExportProfile,
) -> Option<RuntimeProfileDescriptor> {
    profile
        .runtime_profile_id
        .map(RuntimeProfileDescriptor::for_id)
}
