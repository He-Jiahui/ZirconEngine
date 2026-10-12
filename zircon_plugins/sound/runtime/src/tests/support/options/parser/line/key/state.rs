use super::super::super::super::super::state::PendingOptionManifest;

pub(super) fn set_option_key(pending: &mut PendingOptionManifest, key: String) {
    pending.key = Some(key);
}
