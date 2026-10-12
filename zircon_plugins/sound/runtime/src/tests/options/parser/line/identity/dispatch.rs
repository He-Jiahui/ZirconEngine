use super::super::super::super::state::PendingOptionManifest;
use super::{display_name, key};

pub(in crate::tests::manifest::support::options::parser::line) fn parse_option_identity_line(
    line: &str,
    pending: &mut PendingOptionManifest,
) -> bool {
    if key::parse_option_key_line(line, pending) {
        return true;
    }
    display_name::parse_option_display_name_line(line, pending)
}
