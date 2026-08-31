use zircon_runtime_interface::{
    ZrRuntimeViewportPickRequestV1, ZrRuntimeViewportPickResultV1, ZrRuntimeViewportPickTicket,
};

use super::super::GatewayError;
use super::gateway::SessionGateway;
use super::protocol::ensure_status;

impl SessionGateway {
    pub(super) fn request_viewport_pick(
        &self,
        request: ZrRuntimeViewportPickRequestV1,
    ) -> Result<ZrRuntimeViewportPickTicket, GatewayError> {
        self.ensure_session_available("request runtime viewport pick")?;
        if !request.validate_viewport_pick() {
            return Err(GatewayError::Protocol {
                message: "invalid runtime viewport-pick request".to_owned(),
            });
        }
        let request_pick = Self::required(
            self.api.request_viewport_pick,
            "runtime.viewport.pick.request",
        )?;
        let mut ticket = ZrRuntimeViewportPickTicket::invalid();
        ensure_status(
            unsafe { request_pick(self.session, request, &mut ticket) },
            "request runtime viewport pick",
        )?;
        if !ticket.is_valid() {
            return Err(GatewayError::Protocol {
                message: "runtime viewport-pick request returned an invalid ticket".to_owned(),
            });
        }
        Ok(ticket)
    }

    pub(super) fn poll_viewport_pick(
        &self,
        ticket: ZrRuntimeViewportPickTicket,
    ) -> Result<ZrRuntimeViewportPickResultV1, GatewayError> {
        self.ensure_session_available("poll runtime viewport pick")?;
        if !ticket.is_valid() {
            return Err(GatewayError::Protocol {
                message: "invalid runtime viewport-pick poll ticket".to_owned(),
            });
        }
        let poll_pick = Self::required(self.api.poll_viewport_pick, "runtime.viewport.pick.poll")?;
        let mut result = ZrRuntimeViewportPickResultV1::invalid();
        ensure_status(
            unsafe { poll_pick(self.session, ticket, &mut result) },
            "poll runtime viewport pick",
        )?;
        if result.ticket != ticket || !result.validate_viewport_pick() {
            return Err(GatewayError::Protocol {
                message: format!(
                    "runtime viewport-pick result did not validate for ticket {}",
                    ticket.raw()
                ),
            });
        }
        Ok(result)
    }

    pub(super) fn cancel_viewport_pick(
        &self,
        ticket: ZrRuntimeViewportPickTicket,
    ) -> Result<(), GatewayError> {
        self.ensure_session_available("cancel runtime viewport pick")?;
        if !ticket.is_valid() {
            return Err(GatewayError::Protocol {
                message: "invalid runtime viewport-pick cancellation ticket".to_owned(),
            });
        }
        let cancel_pick = Self::required(
            self.api.cancel_viewport_pick,
            "runtime.viewport.pick.cancel",
        )?;
        ensure_status(
            unsafe { cancel_pick(self.session, ticket) },
            "cancel runtime viewport pick",
        )
    }
}
