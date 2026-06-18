use crate::builtin::RuntimeTargetMode;
use crate::plugin::{ExportProfile, RuntimeProfileDescriptor, RuntimeProfileId};

pub(super) fn runtime_profile_for_export_profile(
    profile: &ExportProfile,
) -> RuntimeProfileDescriptor {
    if let Some(profile_id) = profile.runtime_profile_id {
        return RuntimeProfileDescriptor::for_id(profile_id);
    }
    match profile.target_mode {
        RuntimeTargetMode::ServerRuntime => {
            RuntimeProfileDescriptor::for_id(RuntimeProfileId::Server)
        }
        RuntimeTargetMode::EditorHost => RuntimeProfileDescriptor::for_id(RuntimeProfileId::Editor),
        RuntimeTargetMode::ClientRuntime if profile.name.to_ascii_lowercase().contains("3d") => {
            RuntimeProfileDescriptor::for_id(RuntimeProfileId::Client3d)
        }
        RuntimeTargetMode::ClientRuntime => {
            RuntimeProfileDescriptor::for_id(RuntimeProfileId::Client2d)
        }
    }
}
