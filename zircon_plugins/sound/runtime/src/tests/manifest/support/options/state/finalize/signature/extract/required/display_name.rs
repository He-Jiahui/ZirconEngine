use super::field;

pub(in crate::tests::manifest::support::options::state::finalize::signature::extract) fn take_required_option_display_name(
    value: &mut Option<String>,
) -> String {
    field::take_required_option_signature_field(value, "sound option should declare display_name")
}
