use std::collections::HashMap;

use zircon_runtime_interface::{
    ZrRuntimeViewportPickDispositionV1, ZrRuntimeViewportPickRequestV1,
    ZrRuntimeViewportPickResultV1, ZrRuntimeViewportPickTicket,
};

use crate::core::framework::render::{
    RenderViewportPickDisposition, RenderViewportPickRequest, RenderViewportPickResult,
    RenderViewportPickTicket,
};

use super::super::runtime_loop::{RuntimeRenderBridge, RuntimeViewportPickAdmission};

pub(super) const MAX_OUTSTANDING_VIEWPORT_PICKS: usize = 64;

#[derive(Debug, PartialEq, Eq)]
pub(super) enum RuntimeViewportPickError {
    InvalidRequest,
    InvalidTicket,
    LimitExceeded,
    TicketSpaceExhausted,
    NotFound,
    Backend(String),
}

#[derive(Clone, Copy)]
enum RuntimeViewportPickState {
    Immediate(RenderViewportPickDisposition),
    Backend {
        request: RenderViewportPickRequest,
        ticket: RenderViewportPickTicket,
    },
}

#[derive(Clone, Copy)]
struct RuntimeViewportPickEntry {
    request: ZrRuntimeViewportPickRequestV1,
    state: RuntimeViewportPickState,
}

pub(super) struct RuntimeViewportPickStore {
    next_ticket: u64,
    entries: HashMap<u64, RuntimeViewportPickEntry>,
}

impl Default for RuntimeViewportPickStore {
    fn default() -> Self {
        Self {
            next_ticket: 1,
            entries: HashMap::new(),
        }
    }
}

impl RuntimeViewportPickStore {
    /// Admits the request into the session-owned lifecycle.
    ///
    /// A missing or unsupported renderer deliberately completes as `Unavailable`. The session
    /// never falls back to authoring-world or proxy geometry.
    pub(super) fn request(
        &mut self,
        request: ZrRuntimeViewportPickRequestV1,
        render_bridge: Option<&RuntimeRenderBridge>,
    ) -> Result<ZrRuntimeViewportPickTicket, RuntimeViewportPickError> {
        if !request.validate_viewport_pick() {
            return Err(RuntimeViewportPickError::InvalidRequest);
        }
        if self.entries.len() >= MAX_OUTSTANDING_VIEWPORT_PICKS {
            return Err(RuntimeViewportPickError::LimitExceeded);
        }
        let raw = self.next_ticket;
        if raw == 0 {
            return Err(RuntimeViewportPickError::TicketSpaceExhausted);
        }
        let admission = match render_bridge {
            Some(render_bridge) => render_bridge
                .request_viewport_pick(request)
                .map_err(|error| RuntimeViewportPickError::Backend(error.to_string()))?,
            None => {
                RuntimeViewportPickAdmission::Terminal(RenderViewportPickDisposition::Unavailable)
            }
        };
        self.next_ticket = raw.checked_add(1).unwrap_or(0);
        let ticket = ZrRuntimeViewportPickTicket::new(raw);
        let state = match admission {
            RuntimeViewportPickAdmission::Backend { request, ticket } => {
                RuntimeViewportPickState::Backend { request, ticket }
            }
            RuntimeViewportPickAdmission::Terminal(disposition) => {
                RuntimeViewportPickState::Immediate(disposition)
            }
        };
        self.entries
            .insert(raw, RuntimeViewportPickEntry { request, state });
        Ok(ticket)
    }

    pub(super) fn poll(
        &mut self,
        ticket: ZrRuntimeViewportPickTicket,
        render_bridge: Option<&RuntimeRenderBridge>,
    ) -> Result<ZrRuntimeViewportPickResultV1, RuntimeViewportPickError> {
        if !ticket.is_valid() {
            return Err(RuntimeViewportPickError::InvalidTicket);
        }
        let entry = self
            .entries
            .get(&ticket.raw())
            .copied()
            .ok_or(RuntimeViewportPickError::NotFound)?;
        let result = match entry.state {
            RuntimeViewportPickState::Immediate(disposition) => {
                self.entries.remove(&ticket.raw());
                terminal_result(disposition, ticket, entry.request, 0)
            }
            RuntimeViewportPickState::Backend {
                request: backend_request,
                ticket: backend_ticket,
            } => {
                let Some(render_bridge) = render_bridge else {
                    self.entries.remove(&ticket.raw());
                    return Ok(ZrRuntimeViewportPickResultV1::empty(
                        ZrRuntimeViewportPickDispositionV1::Unavailable,
                        ticket,
                        entry.request,
                        0,
                    ));
                };
                match render_bridge.poll_viewport_pick(backend_ticket) {
                    Ok(None) => ZrRuntimeViewportPickResultV1::empty(
                        ZrRuntimeViewportPickDispositionV1::Pending,
                        ticket,
                        entry.request,
                        0,
                    ),
                    Ok(Some(result))
                        if result.matches_ticketed_request(backend_ticket, backend_request) =>
                    {
                        self.entries.remove(&ticket.raw());
                        abi_result(ticket, entry.request, result)
                    }
                    Ok(Some(_)) | Err(_) => {
                        self.entries.remove(&ticket.raw());
                        ZrRuntimeViewportPickResultV1::empty(
                            ZrRuntimeViewportPickDispositionV1::Rejected,
                            ticket,
                            entry.request,
                            0,
                        )
                    }
                }
            }
        };
        Ok(result)
    }

    pub(super) fn cancel(
        &mut self,
        ticket: ZrRuntimeViewportPickTicket,
        render_bridge: Option<&RuntimeRenderBridge>,
    ) -> Result<(), RuntimeViewportPickError> {
        if !ticket.is_valid() {
            return Err(RuntimeViewportPickError::InvalidTicket);
        }
        let entry = self
            .entries
            .remove(&ticket.raw())
            .ok_or(RuntimeViewportPickError::NotFound)?;
        if let RuntimeViewportPickState::Backend { ticket, .. } = entry.state {
            if let Some(render_bridge) = render_bridge {
                render_bridge
                    .cancel_viewport_pick(ticket)
                    .map_err(|error| RuntimeViewportPickError::Backend(error.to_string()))?;
            }
        }
        Ok(())
    }
}

fn terminal_result(
    disposition: RenderViewportPickDisposition,
    ticket: ZrRuntimeViewportPickTicket,
    request: ZrRuntimeViewportPickRequestV1,
    world_generation: u64,
) -> ZrRuntimeViewportPickResultV1 {
    ZrRuntimeViewportPickResultV1::empty(
        match disposition {
            RenderViewportPickDisposition::NoHit => ZrRuntimeViewportPickDispositionV1::NoHit,
            RenderViewportPickDisposition::Hit => ZrRuntimeViewportPickDispositionV1::Rejected,
            RenderViewportPickDisposition::StaleFrame => {
                ZrRuntimeViewportPickDispositionV1::StaleFrame
            }
            RenderViewportPickDisposition::Unavailable => {
                ZrRuntimeViewportPickDispositionV1::Unavailable
            }
            RenderViewportPickDisposition::Rejected => ZrRuntimeViewportPickDispositionV1::Rejected,
            RenderViewportPickDisposition::Cancelled => {
                ZrRuntimeViewportPickDispositionV1::Cancelled
            }
        },
        ticket,
        request,
        world_generation,
    )
}

fn abi_result(
    ticket: ZrRuntimeViewportPickTicket,
    request: ZrRuntimeViewportPickRequestV1,
    result: RenderViewportPickResult,
) -> ZrRuntimeViewportPickResultV1 {
    if result.disposition == RenderViewportPickDisposition::Hit {
        return ZrRuntimeViewportPickResultV1::hit(
            ticket,
            request,
            result.world_generation,
            result.entity,
            result.instance,
            result.subobject,
            result.depth,
            result.world_position,
            result.world_normal,
        );
    }
    terminal_result(result.disposition, ticket, request, result.world_generation)
}

#[cfg(test)]
mod tests {
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
}
