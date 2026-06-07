use super::super::super::super::state::CapabilityStatusParserState;
use zircon_runtime::plugin::CapabilityStatus;

pub(super) fn set_capability_status(
    parser: &mut CapabilityStatusParserState,
    status: CapabilityStatus,
) {
    parser.set_status(status);
}
