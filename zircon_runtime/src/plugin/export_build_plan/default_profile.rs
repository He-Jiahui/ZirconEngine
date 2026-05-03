use crate::{plugin::ExportProfile, plugin::ExportTargetPlatform, RuntimeTargetMode};

pub(super) fn default_profile(profile_name: &str) -> Option<ExportProfile> {
    match profile_name {
        "client" => Some(ExportProfile::default()),
        "server" => Some(ExportProfile::new(
            "server",
            RuntimeTargetMode::ServerRuntime,
            ExportTargetPlatform::Linux,
        )),
        _ => None,
    }
}
