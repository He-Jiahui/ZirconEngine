use zircon_runtime_interface::{
    ZrRuntimeViewportPickRequestV1, ZrRuntimeViewportPickResultV1, ZrRuntimeViewportPickTicket,
};

use super::{EditorRuntimeGatewayHandle, GatewayError, GatewayOrigin, GatewaySessionIdentity};

/// Identity-pinned route for one runtime viewport's asynchronous pick tickets.
///
/// The route retains the endpoint that created every ticket. Gateway replacement can retire the
/// route at the product owner, but it can never redirect a pending ticket into the next session.
#[derive(Clone)]
pub struct EditorRuntimeViewportPickRoute {
    origin: GatewayOrigin,
}

impl EditorRuntimeViewportPickRoute {
    pub fn capture_at_identity(
        gateway: &EditorRuntimeGatewayHandle,
        expected_identity: &GatewaySessionIdentity,
    ) -> Result<Self, GatewayError> {
        let lease = gateway.current_lease();
        if lease.identity() != expected_identity {
            return Err(GatewayError::StaleGeneration {
                expected_generation: expected_identity.gateway_generation(),
                current_generation: lease.generation(),
            });
        }
        Ok(Self {
            origin: lease.origin(),
        })
    }

    pub fn identity(&self) -> &GatewaySessionIdentity {
        self.origin.identity()
    }

    pub fn request_viewport_pick(
        &self,
        request: ZrRuntimeViewportPickRequestV1,
    ) -> Result<ZrRuntimeViewportPickTicket, GatewayError> {
        if !request.validate_viewport_pick() {
            return Err(GatewayError::Protocol {
                message: "invalid runtime viewport-pick request".to_owned(),
            });
        }
        let ticket = self.origin.gateway().request_viewport_pick(request)?;
        if !ticket.is_valid() {
            return Err(GatewayError::Protocol {
                message: "runtime viewport-pick request returned an invalid ticket".to_owned(),
            });
        }
        Ok(ticket)
    }

    pub fn poll_viewport_pick(
        &self,
        ticket: ZrRuntimeViewportPickTicket,
        request: ZrRuntimeViewportPickRequestV1,
    ) -> Result<ZrRuntimeViewportPickResultV1, GatewayError> {
        if !ticket.is_valid() || !request.validate_viewport_pick() {
            return Err(GatewayError::Protocol {
                message: "invalid runtime viewport-pick poll identity".to_owned(),
            });
        }
        let result = self.origin.gateway().poll_viewport_pick(ticket)?;
        if result.ticket != ticket || !result.matches_request(request) {
            return Err(GatewayError::Protocol {
                message: format!(
                    "runtime viewport-pick completion did not match ticket {} and its request identity",
                    ticket.raw()
                ),
            });
        }
        Ok(result)
    }

    pub fn cancel_viewport_pick(
        &self,
        ticket: ZrRuntimeViewportPickTicket,
    ) -> Result<(), GatewayError> {
        if !ticket.is_valid() {
            return Err(GatewayError::Protocol {
                message: "invalid runtime viewport-pick cancellation ticket".to_owned(),
            });
        }
        self.origin.gateway().cancel_viewport_pick(ticket)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use zircon_runtime_interface::{
        ZrRuntimeOperationHandle, ZrRuntimeOperationResultV1, ZrRuntimeOperationStatusV2,
        ZrRuntimeOperationSubmitRequestV1, ZrRuntimeSessionHandle, ZrRuntimeViewportHandle,
        ZrRuntimeViewportPickDispositionV1, ZrRuntimeViewportPickPurposeV1,
        ZrRuntimeViewportPickRequestV1, ZrRuntimeViewportPickResultV1, ZrRuntimeViewportPickTicket,
        ZrRuntimeViewportPixelV1, ZrRuntimeViewportSizeV1, ZIRCON_RUNTIME_ABI_VERSION_V1,
    };

    use super::*;
    use crate::core::gateway::EditorRuntimeGateway;

    #[derive(Default)]
    struct FakePickState {
        requested: Option<ZrRuntimeViewportPickRequestV1>,
        request_count: usize,
        poll_count: usize,
        cancel_count: usize,
    }

    struct FakePickGateway {
        identity: GatewaySessionIdentity,
        ticket: ZrRuntimeViewportPickTicket,
        entity: u64,
        state: Arc<Mutex<FakePickState>>,
    }

    impl FakePickGateway {
        fn new(
            runtime_instance: u64,
            session: u64,
            ticket: u64,
            entity: u64,
        ) -> (Self, Arc<Mutex<FakePickState>>) {
            let state = Arc::new(Mutex::new(FakePickState::default()));
            (
                Self {
                    identity: GatewaySessionIdentity::new(
                        runtime_instance,
                        ZrRuntimeSessionHandle::new(session),
                        1,
                        None,
                    ),
                    ticket: ZrRuntimeViewportPickTicket::new(ticket),
                    entity,
                    state: Arc::clone(&state),
                },
                state,
            )
        }
    }

    impl EditorRuntimeGateway for FakePickGateway {
        fn session_handle(&self) -> ZrRuntimeSessionHandle {
            self.identity.runtime_session()
        }

        fn session_identity(&self) -> GatewaySessionIdentity {
            self.identity.clone()
        }

        fn request_viewport_pick(
            &self,
            request: ZrRuntimeViewportPickRequestV1,
        ) -> Result<ZrRuntimeViewportPickTicket, GatewayError> {
            let mut state = self.state.lock().unwrap();
            state.requested = Some(request);
            state.request_count += 1;
            Ok(self.ticket)
        }

        fn poll_viewport_pick(
            &self,
            ticket: ZrRuntimeViewportPickTicket,
        ) -> Result<ZrRuntimeViewportPickResultV1, GatewayError> {
            let mut state = self.state.lock().unwrap();
            state.poll_count += 1;
            let request = state.requested.expect("request precedes poll");
            Ok(ZrRuntimeViewportPickResultV1::hit(
                ticket,
                request,
                31,
                self.entity,
                0,
                0,
                0.25,
                [1.0, 2.0, 3.0],
                [0.0, 1.0, 0.0],
            ))
        }

        fn cancel_viewport_pick(
            &self,
            _ticket: ZrRuntimeViewportPickTicket,
        ) -> Result<(), GatewayError> {
            self.state.lock().unwrap().cancel_count += 1;
            Ok(())
        }

        fn submit_operation(
            &self,
            _request: ZrRuntimeOperationSubmitRequestV1,
        ) -> Result<ZrRuntimeOperationHandle, GatewayError> {
            Err(GatewayError::CapabilityMissing {
                capability: "runtime.operation.submit",
            })
        }

        fn poll_operation(
            &self,
            _handle: ZrRuntimeOperationHandle,
        ) -> Result<ZrRuntimeOperationStatusV2, GatewayError> {
            Err(GatewayError::CapabilityMissing {
                capability: "runtime.operation.poll",
            })
        }

        fn harvest_operation(
            &self,
            _handle: ZrRuntimeOperationHandle,
        ) -> Result<ZrRuntimeOperationResultV1, GatewayError> {
            Err(GatewayError::CapabilityMissing {
                capability: "runtime.operation.harvest",
            })
        }
    }

    fn request() -> ZrRuntimeViewportPickRequestV1 {
        ZrRuntimeViewportPickRequestV1::new(
            ZrRuntimeViewportHandle::new(1),
            ZrRuntimeViewportSizeV1::new(1280, 720),
            ZrRuntimeViewportPixelV1::new(640, 360),
            19,
            23,
            ZrRuntimeViewportPickPurposeV1::Press,
            0,
        )
    }

    #[test]
    fn route_keeps_request_poll_and_cancel_on_the_captured_origin() {
        let (first, first_state) = FakePickGateway::new(11, 13, 17, 37);
        let gateway = EditorRuntimeGatewayHandle::new(Arc::new(first));
        let identity = gateway.identity();
        let route = EditorRuntimeViewportPickRoute::capture_at_identity(&gateway, &identity)
            .expect("capture first runtime origin");

        let (replacement, replacement_state) = FakePickGateway::new(41, 43, 47, 53);
        gateway
            .replace(Arc::new(replacement))
            .expect("replace published gateway");

        let request = request();
        let ticket = route
            .request_viewport_pick(request)
            .expect("request through captured origin");
        let result = route
            .poll_viewport_pick(ticket, request)
            .expect("poll through captured origin");
        route
            .cancel_viewport_pick(ticket)
            .expect("cancel through captured origin");

        assert_eq!(result.entity, 37);
        let first_state = first_state.lock().unwrap();
        assert_eq!(
            (
                first_state.request_count,
                first_state.poll_count,
                first_state.cancel_count
            ),
            (1, 1, 1)
        );
        let replacement_state = replacement_state.lock().unwrap();
        assert_eq!(
            (
                replacement_state.request_count,
                replacement_state.poll_count,
                replacement_state.cancel_count
            ),
            (0, 0, 0)
        );
    }

    #[test]
    fn route_rejects_invalid_requests_before_calling_the_runtime() {
        let (runtime, state) = FakePickGateway::new(11, 13, 17, 37);
        let gateway = EditorRuntimeGatewayHandle::new(Arc::new(runtime));
        let route =
            EditorRuntimeViewportPickRoute::capture_at_identity(&gateway, &gateway.identity())
                .expect("capture runtime origin");
        let mut invalid = request();
        invalid.abi_version = ZIRCON_RUNTIME_ABI_VERSION_V1 + 1;

        assert!(matches!(
            route.request_viewport_pick(invalid),
            Err(GatewayError::Protocol { .. })
        ));
        assert_eq!(state.lock().unwrap().request_count, 0);
    }

    #[test]
    fn route_rejects_a_completion_from_another_frame() {
        let (runtime, _state) = FakePickGateway::new(11, 13, 17, 37);
        let gateway = EditorRuntimeGatewayHandle::new(Arc::new(runtime));
        let route =
            EditorRuntimeViewportPickRoute::capture_at_identity(&gateway, &gateway.identity())
                .expect("capture runtime origin");
        let request = request();
        let ticket = route.request_viewport_pick(request).unwrap();
        let mut another_frame = request;
        another_frame.frame_generation += 1;

        assert!(matches!(
            route.poll_viewport_pick(ticket, another_frame),
            Err(GatewayError::Protocol { .. })
        ));
    }

    #[test]
    fn disposition_type_remains_available_to_gateway_consumers() {
        assert!(ZrRuntimeViewportPickDispositionV1::NoHit.is_terminal());
    }
}
