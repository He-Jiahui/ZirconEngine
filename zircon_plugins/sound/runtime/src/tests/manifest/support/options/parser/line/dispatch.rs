use super::super::super::state::PendingOptionManifest;
use super::{identity, requirement, value};

pub(in crate::tests::manifest::support::options::parser) fn parse_option_manifest_line(
    line: &str,
    pending: &mut PendingOptionManifest,
) {
    if identity::parse_option_identity_line(line, pending) {
        return;
    }
    if value::parse_option_value_line(line, pending) {
        return;
    }
    requirement::parse_option_requirement_line(line, pending);
}
