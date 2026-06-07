use super::super::super::super::state::CapabilityStatusParserState;

pub(super) fn set_capability_status_bevy_references(
    parser: &mut CapabilityStatusParserState,
    references: Vec<String>,
) {
    parser.set_bevy_references(references);
}
