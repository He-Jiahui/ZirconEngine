use super::super::super::super::super::state::PendingOptionManifest;

pub(super) fn set_option_display_name(pending: &mut PendingOptionManifest, display_name: String) {
    pending.display_name = Some(display_name);
}
