use super::super::super::super::super::state::PendingOptionManifest;

pub(super) fn set_option_enum_values(
    pending: &mut PendingOptionManifest,
    enum_values: Vec<String>,
) {
    pending.enum_values = enum_values;
}
