use super::super::super::super::super::state::PendingOptionManifest;

pub(super) fn set_option_default_value(pending: &mut PendingOptionManifest, default_value: String) {
    pending.default_value = Some(default_value);
}
