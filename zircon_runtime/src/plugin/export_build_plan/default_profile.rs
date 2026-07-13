use crate::core::framework::platform::RuntimeTargetMode;
use crate::{
    core::framework::project::ExportProfile, core::framework::project::ExportTargetPlatform,
    core::framework::project::RuntimeProfileId,
};

pub(super) fn default_profile(profile_name: &str) -> Option<ExportProfile> {
    match profile_name {
        "client" => Some(ExportProfile::default()),
        "server" => Some(
            ExportProfile::new(
                "server",
                RuntimeTargetMode::ServerRuntime,
                ExportTargetPlatform::Headless,
            )
            .with_runtime_profile_id(RuntimeProfileId::Server),
        ),
        _ => None,
    }
}
