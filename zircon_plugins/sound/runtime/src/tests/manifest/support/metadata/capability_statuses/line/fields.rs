mod bevy_references;
mod capability;
mod status;

use super::super::state::CapabilityStatusParserState;

pub(super) fn parse_capability_status_fields(line: &str, parser: &mut CapabilityStatusParserState) {
    if capability::parse_capability_field(line, parser) {
        return;
    }
    if status::parse_status_field(line, parser) {
        return;
    }
    bevy_references::parse_bevy_references_field(line, parser);
}
