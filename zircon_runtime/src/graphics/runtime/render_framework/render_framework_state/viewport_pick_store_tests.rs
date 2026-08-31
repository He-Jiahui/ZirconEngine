use crate::core::framework::render::{
    RenderFrameworkError, RenderViewportHandle, RenderViewportPickDisposition,
    RenderViewportPickPolicy, RenderViewportPickPurpose, RenderViewportPickRequest,
    RenderViewportPickTicket,
};
use crate::core::math::UVec2;

use super::viewport_pick_store::{ViewportPickStore, MAX_VIEWPORT_PICK_TICKETS};

#[test]
fn viewport_pick_store_consumes_terminal_results_once() {
    let mut store = ViewportPickStore::default();
    let request = request(RenderViewportHandle::new(7), 11);
    let ticket = store
        .request_terminal(request, 13, RenderViewportPickDisposition::Unavailable)
        .expect("terminal pick admission");

    let result = store
        .poll(ticket)
        .expect("first terminal poll")
        .expect("terminal result");
    assert_eq!(result.ticket, ticket);
    assert_eq!(result.frame_generation, 11);
    assert_eq!(result.world_generation, 13);
    assert_eq!(
        result.disposition,
        RenderViewportPickDisposition::Unavailable
    );
    assert_eq!(
        store.poll(ticket),
        Err(RenderFrameworkError::UnknownViewportPickTicket {
            ticket: ticket.raw()
        })
    );
}

#[test]
fn viewport_pick_store_promotes_one_exact_pending_completion_once() {
    let mut store = ViewportPickStore::default();
    let request = request(RenderViewportHandle::new(7), 11);
    let (ticket, completion) = store
        .request_pending(request, 13)
        .expect("pending pick admission");

    assert_eq!(store.poll(ticket), Ok(None));
    assert!(completion.complete_hit(31, 37, 41, 0.25, [1.0, 2.0, 3.0], [0.0, 1.0, 0.0]));

    let result = store
        .poll(ticket)
        .expect("completed pending poll")
        .expect("completed pending result");
    assert_eq!(result.ticket, ticket);
    assert_eq!(result.world_generation, 13);
    assert_eq!(result.entity, 31);
    assert_eq!(result.instance, 37);
    assert_eq!(result.subobject, 41);
    assert_eq!(result.depth, 0.25);
    assert_eq!(result.world_position, [1.0, 2.0, 3.0]);
    assert_eq!(result.world_normal, [0.0, 1.0, 0.0]);
    assert_eq!(
        store.poll(ticket),
        Err(RenderFrameworkError::UnknownViewportPickTicket {
            ticket: ticket.raw()
        })
    );
}

#[test]
fn viewport_pick_store_ignores_late_completion_after_cancellation() {
    let mut store = ViewportPickStore::default();
    let request = request(RenderViewportHandle::new(7), 11);
    let (ticket, completion) = store
        .request_pending(request, 13)
        .expect("pending pick admission");

    store.cancel(ticket).expect("pending cancellation");
    assert!(completion.complete_terminal(RenderViewportPickDisposition::Cancelled));
    assert_eq!(
        store.poll(ticket),
        Err(RenderFrameworkError::UnknownViewportPickTicket {
            ticket: ticket.raw()
        })
    );
}

#[test]
fn viewport_pick_store_bounds_tickets_and_never_reuses_wrapped_ids() {
    let viewport = RenderViewportHandle::new(7);
    let mut store = ViewportPickStore::default();
    for sequence in 1..=MAX_VIEWPORT_PICK_TICKETS {
        store
            .request_terminal(
                request(viewport, sequence as u64),
                3,
                RenderViewportPickDisposition::NoHit,
            )
            .expect("bounded pick admission");
    }
    assert_eq!(
        store.request_terminal(
            request(viewport, 99),
            3,
            RenderViewportPickDisposition::NoHit,
        ),
        Err(RenderFrameworkError::ViewportPickCapacityExceeded {
            limit: MAX_VIEWPORT_PICK_TICKETS
        })
    );

    let mut exhausted = ViewportPickStore::default();
    exhausted.set_next_ticket_for_tests(u64::MAX);
    let last = exhausted
        .request_terminal(
            request(viewport, 1),
            3,
            RenderViewportPickDisposition::NoHit,
        )
        .expect("last nonzero ticket");
    assert_eq!(last, RenderViewportPickTicket::new(u64::MAX));
    exhausted.poll(last).expect("consume last ticket");
    assert_eq!(
        exhausted.request_terminal(
            request(viewport, 2),
            3,
            RenderViewportPickDisposition::NoHit,
        ),
        Err(RenderFrameworkError::ViewportPickTicketSpaceExhausted)
    );
}

#[test]
fn viewport_pick_store_retires_only_the_destroyed_viewport() {
    let first_viewport = RenderViewportHandle::new(7);
    let second_viewport = RenderViewportHandle::new(9);
    let mut store = ViewportPickStore::default();
    let first = store
        .request_terminal(
            request(first_viewport, 1),
            3,
            RenderViewportPickDisposition::NoHit,
        )
        .unwrap();
    let second = store
        .request_terminal(
            request(second_viewport, 2),
            5,
            RenderViewportPickDisposition::NoHit,
        )
        .unwrap();

    store.remove_viewport(first_viewport);

    assert!(matches!(
        store.poll(first),
        Err(RenderFrameworkError::UnknownViewportPickTicket { .. })
    ));
    assert!(store.poll(second).unwrap().is_some());
}

fn request(viewport: RenderViewportHandle, input_sequence: u64) -> RenderViewportPickRequest {
    RenderViewportPickRequest::new(
        viewport,
        UVec2::new(320, 180),
        UVec2::new(17, 19),
        11,
        input_sequence,
        RenderViewportPickPurpose::Press,
        RenderViewportPickPolicy::default(),
    )
}
