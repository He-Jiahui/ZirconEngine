use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use crate::core::framework::render::{
    RenderFrameworkError, RenderViewportHandle, RenderViewportPickDisposition,
    RenderViewportPickRequest, RenderViewportPickResult, RenderViewportPickTicket,
};

pub(in crate::graphics::runtime::render_framework) const MAX_VIEWPORT_PICK_TICKETS: usize = 64;

pub(in crate::graphics::runtime::render_framework) struct ViewportPickStore {
    next_ticket: u64,
    tickets: HashMap<RenderViewportPickTicket, ViewportPickEntry>,
    completions: Arc<Mutex<VecDeque<(RenderViewportPickTicket, RenderViewportPickResult)>>>,
}

struct ViewportPickEntry {
    viewport: RenderViewportHandle,
    result: Option<RenderViewportPickResult>,
}

pub(in crate::graphics::runtime::render_framework) struct ViewportPickCompletionSender {
    ticket: RenderViewportPickTicket,
    request: RenderViewportPickRequest,
    world_generation: u64,
    completions: Arc<Mutex<VecDeque<(RenderViewportPickTicket, RenderViewportPickResult)>>>,
}

impl Default for ViewportPickStore {
    fn default() -> Self {
        Self {
            next_ticket: 1,
            tickets: HashMap::new(),
            completions: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
}

impl ViewportPickCompletionSender {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::graphics::runtime::render_framework) fn complete_hit(
        self,
        entity: crate::core::framework::scene::EntityId,
        instance: u64,
        subobject: u64,
        depth: f32,
        world_position: [f32; 3],
        world_normal: [f32; 3],
    ) -> bool {
        let result = RenderViewportPickResult::hit(
            self.ticket,
            self.request,
            self.world_generation,
            entity,
            instance,
            subobject,
            depth,
            world_position,
            world_normal,
        );
        self.enqueue(result)
    }

    pub(in crate::graphics::runtime::render_framework) fn complete_terminal(
        self,
        disposition: RenderViewportPickDisposition,
    ) -> bool {
        if disposition == RenderViewportPickDisposition::Hit {
            return false;
        }
        let result = RenderViewportPickResult::terminal(
            disposition,
            self.ticket,
            self.request,
            self.world_generation,
        );
        self.enqueue(result)
    }

    fn enqueue(self, result: RenderViewportPickResult) -> bool {
        if !result.matches_ticketed_request(self.ticket, self.request) {
            return false;
        }
        self.completions
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push_back((self.ticket, result));
        true
    }
}

impl ViewportPickStore {
    pub(in crate::graphics::runtime::render_framework) fn request_terminal(
        &mut self,
        request: RenderViewportPickRequest,
        world_generation: u64,
        disposition: RenderViewportPickDisposition,
    ) -> Result<RenderViewportPickTicket, RenderFrameworkError> {
        if disposition == RenderViewportPickDisposition::Hit {
            return Err(RenderFrameworkError::InvalidViewportPickRequest);
        }
        let ticket = self.allocate_ticket(request)?;
        let result =
            RenderViewportPickResult::terminal(disposition, ticket, request, world_generation);
        self.tickets.insert(
            ticket,
            ViewportPickEntry {
                viewport: request.viewport,
                result: Some(result),
            },
        );
        Ok(ticket)
    }

    pub(in crate::graphics::runtime::render_framework) fn request_pending(
        &mut self,
        request: RenderViewportPickRequest,
        world_generation: u64,
    ) -> Result<(RenderViewportPickTicket, ViewportPickCompletionSender), RenderFrameworkError>
    {
        let ticket = self.allocate_ticket(request)?;
        self.tickets.insert(
            ticket,
            ViewportPickEntry {
                viewport: request.viewport,
                result: None,
            },
        );
        Ok((
            ticket,
            ViewportPickCompletionSender {
                ticket,
                request,
                world_generation,
                completions: Arc::clone(&self.completions),
            },
        ))
    }

    pub(in crate::graphics::runtime::render_framework) fn poll(
        &mut self,
        ticket: RenderViewportPickTicket,
    ) -> Result<Option<RenderViewportPickResult>, RenderFrameworkError> {
        validate_ticket(ticket)?;
        self.drain_completions();
        let Some(entry) = self.tickets.get(&ticket) else {
            return Err(RenderFrameworkError::UnknownViewportPickTicket {
                ticket: ticket.raw(),
            });
        };
        let Some(result) = entry.result else {
            return Ok(None);
        };
        self.tickets.remove(&ticket);
        Ok(Some(result))
    }

    pub(in crate::graphics::runtime::render_framework) fn cancel(
        &mut self,
        ticket: RenderViewportPickTicket,
    ) -> Result<(), RenderFrameworkError> {
        validate_ticket(ticket)?;
        self.drain_completions();
        self.tickets.remove(&ticket).map(|_| ()).ok_or(
            RenderFrameworkError::UnknownViewportPickTicket {
                ticket: ticket.raw(),
            },
        )
    }

    pub(in crate::graphics::runtime::render_framework) fn remove_viewport(
        &mut self,
        viewport: RenderViewportHandle,
    ) {
        self.drain_completions();
        self.tickets.retain(|_, entry| entry.viewport != viewport);
    }

    fn allocate_ticket(
        &mut self,
        request: RenderViewportPickRequest,
    ) -> Result<RenderViewportPickTicket, RenderFrameworkError> {
        if !request.is_valid() {
            return Err(RenderFrameworkError::InvalidViewportPickRequest);
        }
        self.drain_completions();
        if self.tickets.len() >= MAX_VIEWPORT_PICK_TICKETS {
            return Err(RenderFrameworkError::ViewportPickCapacityExceeded {
                limit: MAX_VIEWPORT_PICK_TICKETS,
            });
        }
        if self.next_ticket == 0 {
            return Err(RenderFrameworkError::ViewportPickTicketSpaceExhausted);
        }
        let ticket = RenderViewportPickTicket::new(self.next_ticket);
        self.next_ticket = self.next_ticket.checked_add(1).unwrap_or(0);
        Ok(ticket)
    }

    fn drain_completions(&mut self) {
        let mut completions = self
            .completions
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        while let Some((ticket, result)) = completions.pop_front() {
            if let Some(entry) = self.tickets.get_mut(&ticket) {
                entry.result = Some(result);
            }
        }
    }

    #[cfg(test)]
    pub(super) fn set_next_ticket_for_tests(&mut self, next_ticket: u64) {
        self.next_ticket = next_ticket;
    }
}

fn validate_ticket(ticket: RenderViewportPickTicket) -> Result<(), RenderFrameworkError> {
    if ticket.is_valid() {
        Ok(())
    } else {
        Err(RenderFrameworkError::InvalidViewportPickTicket {
            ticket: ticket.raw(),
        })
    }
}
