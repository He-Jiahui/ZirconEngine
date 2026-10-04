//! 指定名称未在项目声明时仅识别内建 profile 名称；from_project_manifest 用它建立明确的导出目标。
use crate::core::framework::platform::RuntimeTargetMode;
use crate::{
    core::framework::project::ExportProfile, core::framework::project::ExportTargetPlatform,
    core::framework::project::RuntimeProfileId,
};

/// 项目未声明指定名称时只接受 client/server 内建名，其他名称仍由建计划入口返回 MissingProfile。
pub(super) fn default_profile(profile_name: &str) -> Option<ExportProfile> {
    match profile_name {
        "client" => Some(ExportProfile::default()),
        "server" => Some(ExportProfile::new(
            "server",
            RuntimeTargetMode::ServerRuntime,
            ExportTargetPlatform::Headless,
            RuntimeProfileId::Server,
        )),
        _ => None,
    }
}
