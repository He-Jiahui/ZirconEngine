pub(super) fn take_required_option_signature_field(
    value: &mut Option<String>,
    missing_message: &'static str,
) -> String {
    value.take().expect(missing_message)
}
