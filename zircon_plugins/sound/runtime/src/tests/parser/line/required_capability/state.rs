use super::super::super::super::super::state::PendingOptionManifest;

pub(super) fn set_option_required_capability(
    pending: &mut PendingOptionManifest,
    required_capability: String,
) {
    pending.required_capability = Some(required_capability);
}
