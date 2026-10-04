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
    /// 发起鼠标选择前捕获当前完整会话身份；后续票据操作保持在此端点。
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

    /// 同时核对票据与原请求；旧会话或交叉请求的结果不可改变当前选择。
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
#[path = "tests/viewport_pick_route.rs"]
mod tests;
