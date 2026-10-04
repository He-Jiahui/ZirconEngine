use crate::core::framework::scene::EntityId;
use crate::core::math::UVec2;

use super::RenderViewportHandle;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RenderViewportPickTicket(u64);

impl RenderViewportPickTicket {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }

    pub const fn is_valid(self) -> bool {
        self.0 != 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderViewportPickPurpose {
    Hover,
    Press,
    Selection,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderViewportPickPolicy(u32);

impl RenderViewportPickPolicy {
    pub const INCLUDE_TRANSLUCENT: u32 = 1 << 0;
    pub const INCLUDE_BACKFACES: u32 = 1 << 1;
    pub const KNOWN_FLAGS: u32 = Self::INCLUDE_TRANSLUCENT | Self::INCLUDE_BACKFACES;

    pub const fn from_bits(bits: u32) -> Option<Self> {
        if bits & !Self::KNOWN_FLAGS == 0 {
            Some(Self(bits))
        } else {
            None
        }
    }

    pub const fn bits(self) -> u32 {
        self.0
    }

    pub const fn includes_translucent(self) -> bool {
        self.0 & Self::INCLUDE_TRANSLUCENT != 0
    }

    pub const fn includes_backfaces(self) -> bool {
        self.0 & Self::INCLUDE_BACKFACES != 0
    }
}

/// 编辑器对已展示帧的异步拾取请求；视口、像素、帧代次与输入序号共同限定身份。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderViewportPickRequest {
    pub viewport: RenderViewportHandle,
    pub viewport_size: UVec2,
    pub pixel: UVec2,
    pub frame_generation: u64,
    pub input_sequence: u64,
    pub purpose: RenderViewportPickPurpose,
    pub policy: RenderViewportPickPolicy,
}

impl RenderViewportPickRequest {
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        viewport: RenderViewportHandle,
        viewport_size: UVec2,
        pixel: UVec2,
        frame_generation: u64,
        input_sequence: u64,
        purpose: RenderViewportPickPurpose,
        policy: RenderViewportPickPolicy,
    ) -> Self {
        Self {
            viewport,
            viewport_size,
            pixel,
            frame_generation,
            input_sequence,
            purpose,
            policy,
        }
    }

    pub const fn is_valid(self) -> bool {
        self.viewport.raw() != 0
            && self.viewport_size.x != 0
            && self.viewport_size.y != 0
            && self.pixel.x < self.viewport_size.x
            && self.pixel.y < self.viewport_size.y
            && self.frame_generation != 0
            && self.input_sequence != 0
            && self.policy.bits() & !RenderViewportPickPolicy::KNOWN_FLAGS == 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderViewportPickDisposition {
    NoHit,
    Hit,
    StaleFrame,
    Unavailable,
    Rejected,
    Cancelled,
}

/// GPU 拾取的终态载荷；消费端必须同时核对票据和原请求，避免迟到结果误选新帧。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderViewportPickResult {
    pub disposition: RenderViewportPickDisposition,
    pub ticket: RenderViewportPickTicket,
    pub viewport: RenderViewportHandle,
    pub viewport_size: UVec2,
    pub pixel: UVec2,
    pub purpose: RenderViewportPickPurpose,
    pub frame_generation: u64,
    pub input_sequence: u64,
    pub world_generation: u64,
    pub entity: EntityId,
    pub instance: u64,
    pub subobject: u64,
    pub depth: f32,
    pub world_position: [f32; 3],
    pub world_normal: [f32; 3],
    pub applied_policy: RenderViewportPickPolicy,
}

impl RenderViewportPickResult {
    pub const fn terminal(
        disposition: RenderViewportPickDisposition,
        ticket: RenderViewportPickTicket,
        request: RenderViewportPickRequest,
        world_generation: u64,
    ) -> Self {
        Self {
            disposition,
            ticket,
            viewport: request.viewport,
            viewport_size: request.viewport_size,
            pixel: request.pixel,
            purpose: request.purpose,
            frame_generation: request.frame_generation,
            input_sequence: request.input_sequence,
            world_generation,
            entity: 0,
            instance: 0,
            subobject: 0,
            depth: 0.0,
            world_position: [0.0; 3],
            world_normal: [0.0; 3],
            applied_policy: request.policy,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub const fn hit(
        ticket: RenderViewportPickTicket,
        request: RenderViewportPickRequest,
        world_generation: u64,
        entity: EntityId,
        instance: u64,
        subobject: u64,
        depth: f32,
        world_position: [f32; 3],
        world_normal: [f32; 3],
    ) -> Self {
        Self {
            disposition: RenderViewportPickDisposition::Hit,
            ticket,
            viewport: request.viewport,
            viewport_size: request.viewport_size,
            pixel: request.pixel,
            purpose: request.purpose,
            frame_generation: request.frame_generation,
            input_sequence: request.input_sequence,
            world_generation,
            entity,
            instance,
            subobject,
            depth,
            world_position,
            world_normal,
            applied_policy: request.policy,
        }
    }

    pub fn is_valid(self) -> bool {
        if !self.ticket.is_valid()
            || self.viewport.raw() == 0
            || self.viewport_size.x == 0
            || self.viewport_size.y == 0
            || self.pixel.x >= self.viewport_size.x
            || self.pixel.y >= self.viewport_size.y
            || self.frame_generation == 0
            || self.input_sequence == 0
            || self.applied_policy.bits() & !RenderViewportPickPolicy::KNOWN_FLAGS != 0
            || !self.depth.is_finite()
            || !self.world_position.iter().all(|value| value.is_finite())
            || !self.world_normal.iter().all(|value| value.is_finite())
        {
            return false;
        }

        if self.disposition == RenderViewportPickDisposition::Hit {
            self.world_generation != 0 && self.entity != 0 && self.depth >= 0.0
        } else {
            self.entity == 0
                && self.instance == 0
                && self.subobject == 0
                && self.depth == 0.0
                && self.world_position == [0.0; 3]
                && self.world_normal == [0.0; 3]
        }
    }

    pub fn matches_request(self, request: RenderViewportPickRequest) -> bool {
        request.is_valid()
            && self.is_valid()
            && self.viewport == request.viewport
            && self.viewport_size == request.viewport_size
            && self.pixel == request.pixel
            && self.purpose == request.purpose
            && self.frame_generation == request.frame_generation
            && self.input_sequence == request.input_sequence
            && self.applied_policy == request.policy
    }

    pub fn matches_ticketed_request(
        self,
        ticket: RenderViewportPickTicket,
        request: RenderViewportPickRequest,
    ) -> bool {
        self.ticket == ticket && self.matches_request(request)
    }
}

#[cfg(test)]
#[path = "tests/viewport_pick.rs"]
mod tests;
