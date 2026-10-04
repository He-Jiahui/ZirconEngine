use super::*;
use zircon_runtime_interface::{
    ZrRuntimeViewportHandle, ZrRuntimeViewportPickPurposeV1, ZrRuntimeViewportPixelV1,
    ZrRuntimeViewportSizeV1,
};

fn request(sequence: u64) -> ZrRuntimeViewportPickRequestV1 {
    ZrRuntimeViewportPickRequestV1::new(
        ZrRuntimeViewportHandle::new(1),
        ZrRuntimeViewportSizeV1::new(1280, 720),
        ZrRuntimeViewportPixelV1::new(640, 360),
        19,
        sequence,
        ZrRuntimeViewportPickPurposeV1::Press,
        0,
    )
}

#[test]
fn unavailable_backend_still_closes_one_exact_ticket_lifecycle() {
    let mut store = RuntimeViewportPickStore::default();
    let request = request(23);
    let ticket = store.request(request, None).unwrap();
    let result = store.poll(ticket, None).unwrap();

    assert_eq!(
        result.disposition(),
        Some(ZrRuntimeViewportPickDispositionV1::Unavailable)
    );
    assert!(result.matches_request(request));
    assert_eq!(
        store.poll(ticket, None),
        Err(RuntimeViewportPickError::NotFound)
    );
}

#[test]
fn outstanding_ticket_budget_is_bounded_and_cancel_releases_capacity() {
    let mut store = RuntimeViewportPickStore::default();
    let mut first = ZrRuntimeViewportPickTicket::invalid();
    for index in 0..MAX_OUTSTANDING_VIEWPORT_PICKS {
        let ticket = store.request(request(index as u64 + 1), None).unwrap();
        if index == 0 {
            first = ticket;
        }
    }
    assert_eq!(
        store.request(request(1000), None),
        Err(RuntimeViewportPickError::LimitExceeded)
    );

    store.cancel(first, None).unwrap();
    assert!(store.request(request(1001), None).is_ok());
}
