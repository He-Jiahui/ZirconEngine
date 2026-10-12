use super::super::super::super::super::state::PendingOptionManifest;

pub(super) fn set_option_value_type(pending: &mut PendingOptionManifest, value_type: String) {
    pending.value_type = Some(value_type);
}
