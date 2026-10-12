use super::super::super::super::state::CapabilityStatusParserState;

pub(super) fn set_capability_status_capability(
    parser: &mut CapabilityStatusParserState,
    capability: String,
) {
    parser.set_capability(capability);
}
