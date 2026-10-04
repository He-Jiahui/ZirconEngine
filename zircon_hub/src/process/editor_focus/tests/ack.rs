use zircon_runtime_interface::hub_protocol::{HubEditorFocusAckV1, HubSessionToken};

use super::validate_focus_acknowledgement;
use zircon_runtime_interface::hub_protocol::{
    HubEditorFocusAckDispositionV1, HubEditorFocusSignalV1,
};

#[test]
fn acknowledgement_requires_the_exact_request_identity_and_a_focused_disposition() {
    let request = HubEditorFocusSignalV1::new(HubSessionToken::new(), "913-42", 1, 7, u64::MAX)
        .expect("valid request");
    assert!(
        validate_focus_acknowledgement(&request, &HubEditorFocusAckV1::focused(&request)).is_ok()
    );

    let rejected = HubEditorFocusAckV1::from_request(
        &request,
        HubEditorFocusAckDispositionV1::RejectedExpired,
    );
    assert!(validate_focus_acknowledgement(&request, &rejected).is_err());
}
