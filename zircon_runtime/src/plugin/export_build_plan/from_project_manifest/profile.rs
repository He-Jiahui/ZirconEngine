use crate::core::framework::project::ExportProfile;
use crate::plugin::RuntimeProfileDescriptor;

pub(super) fn runtime_profile_for_export_profile(
    profile: &ExportProfile,
) -> Option<RuntimeProfileDescriptor> {
    profile
        .runtime_profile_id
        .map(RuntimeProfileDescriptor::for_id)
}
