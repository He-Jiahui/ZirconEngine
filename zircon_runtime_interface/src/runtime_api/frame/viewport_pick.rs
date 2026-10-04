use crate::{ZrRuntimeViewportHandle, ZrRuntimeViewportSizeV1, ZIRCON_RUNTIME_ABI_VERSION_V1};

pub const ZR_RUNTIME_VIEWPORT_PICK_POLICY_INCLUDE_TRANSLUCENT_V1: u32 = 1 << 0;
pub const ZR_RUNTIME_VIEWPORT_PICK_POLICY_INCLUDE_BACKFACES_V1: u32 = 1 << 1;
const ZR_RUNTIME_VIEWPORT_PICK_POLICY_KNOWN_FLAGS_V1: u32 =
    ZR_RUNTIME_VIEWPORT_PICK_POLICY_INCLUDE_TRANSLUCENT_V1
        | ZR_RUNTIME_VIEWPORT_PICK_POLICY_INCLUDE_BACKFACES_V1;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ZrRuntimeViewportPickTicket(u64);

impl ZrRuntimeViewportPickTicket {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn invalid() -> Self {
        Self(0)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }

    pub const fn is_valid(self) -> bool {
        self.0 != 0
    }
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZrRuntimeViewportPickPurposeV1 {
    Hover = 1,
    Press = 2,
    Selection = 3,
}

impl ZrRuntimeViewportPickPurposeV1 {
    pub const fn raw(self) -> u32 {
        self as u32
    }

    pub const fn from_raw(value: u32) -> Option<Self> {
        match value {
            1 => Some(Self::Hover),
            2 => Some(Self::Press),
            3 => Some(Self::Selection),
            _ => None,
        }
    }
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZrRuntimeViewportPickDispositionV1 {
    Pending = 1,
    NoHit = 2,
    Hit = 3,
    StaleFrame = 4,
    Unavailable = 5,
    Rejected = 6,
    Cancelled = 7,
}

impl ZrRuntimeViewportPickDispositionV1 {
    pub const fn raw(self) -> u32 {
        self as u32
    }

    pub const fn from_raw(value: u32) -> Option<Self> {
        match value {
            1 => Some(Self::Pending),
            2 => Some(Self::NoHit),
            3 => Some(Self::Hit),
            4 => Some(Self::StaleFrame),
            5 => Some(Self::Unavailable),
            6 => Some(Self::Rejected),
            7 => Some(Self::Cancelled),
            _ => None,
        }
    }

    pub const fn is_terminal(self) -> bool {
        !matches!(self, Self::Pending)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZrRuntimeViewportPixelV1 {
    pub x: u32,
    pub y: u32,
}

impl ZrRuntimeViewportPixelV1 {
    pub const fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

/// Allocation-free request for one renderer-owned viewport hit.
///
/// `frame_generation` identifies the already-presented render product. A backend must reject or
/// return `StaleFrame` instead of silently evaluating the pixel against a newer frame.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZrRuntimeViewportPickRequestV1 {
    pub abi_version: u32,
    pub purpose: u32,
    pub viewport: ZrRuntimeViewportHandle,
    pub viewport_size: ZrRuntimeViewportSizeV1,
    pub pixel: ZrRuntimeViewportPixelV1,
    pub frame_generation: u64,
    pub input_sequence: u64,
    pub policy_flags: u32,
    pub reserved: u32,
}

impl ZrRuntimeViewportPickRequestV1 {
    pub const fn new(
        viewport: ZrRuntimeViewportHandle,
        viewport_size: ZrRuntimeViewportSizeV1,
        pixel: ZrRuntimeViewportPixelV1,
        frame_generation: u64,
        input_sequence: u64,
        purpose: ZrRuntimeViewportPickPurposeV1,
        policy_flags: u32,
    ) -> Self {
        Self {
            abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
            purpose: purpose.raw(),
            viewport,
            viewport_size,
            pixel,
            frame_generation,
            input_sequence,
            policy_flags,
            reserved: 0,
        }
    }

    pub const fn purpose(self) -> Option<ZrRuntimeViewportPickPurposeV1> {
        ZrRuntimeViewportPickPurposeV1::from_raw(self.purpose)
    }

    pub const fn validate_viewport_pick(self) -> bool {
        self.abi_version == ZIRCON_RUNTIME_ABI_VERSION_V1
            && self.purpose().is_some()
            && self.viewport.is_valid()
            && self.viewport_size.width > 0
            && self.viewport_size.height > 0
            && self.pixel.x < self.viewport_size.width
            && self.pixel.y < self.viewport_size.height
            && self.frame_generation != 0
            && self.input_sequence != 0
            && self.policy_flags & !ZR_RUNTIME_VIEWPORT_PICK_POLICY_KNOWN_FLAGS_V1 == 0
            && self.reserved == 0
    }
}

/// Fixed-layout completion for one viewport pick ticket.
///
/// The owning gateway supplies the runtime-session identity. This payload repeats every remaining
/// identity needed to reject a completion from another viewport, input edge, frame, or world.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ZrRuntimeViewportPickResultV1 {
    pub abi_version: u32,
    pub disposition: u32,
    pub ticket: ZrRuntimeViewportPickTicket,
    pub viewport: ZrRuntimeViewportHandle,
    pub viewport_size: ZrRuntimeViewportSizeV1,
    pub pixel: ZrRuntimeViewportPixelV1,
    pub purpose: u32,
    pub frame_generation: u64,
    pub input_sequence: u64,
    pub world_generation: u64,
    pub entity: u64,
    pub instance: u64,
    pub subobject: u64,
    pub depth: f32,
    pub world_position: [f32; 3],
    pub world_normal: [f32; 3],
    pub applied_policy_flags: u32,
    pub reserved: u32,
}

impl ZrRuntimeViewportPickResultV1 {
    pub const fn invalid() -> Self {
        Self {
            abi_version: 0,
            disposition: 0,
            ticket: ZrRuntimeViewportPickTicket::invalid(),
            viewport: ZrRuntimeViewportHandle::invalid(),
            viewport_size: ZrRuntimeViewportSizeV1::new(0, 0),
            pixel: ZrRuntimeViewportPixelV1::new(0, 0),
            purpose: 0,
            frame_generation: 0,
            input_sequence: 0,
            world_generation: 0,
            entity: 0,
            instance: 0,
            subobject: 0,
            depth: 0.0,
            world_position: [0.0; 3],
            world_normal: [0.0; 3],
            applied_policy_flags: 0,
            reserved: 0,
        }
    }

    pub const fn empty(
        disposition: ZrRuntimeViewportPickDispositionV1,
        ticket: ZrRuntimeViewportPickTicket,
        request: ZrRuntimeViewportPickRequestV1,
        world_generation: u64,
    ) -> Self {
        Self {
            abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
            disposition: disposition.raw(),
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
            applied_policy_flags: request.policy_flags,
            reserved: 0,
        }
    }

    // EXEMPT(GEN-Q7): viewport-pick receipt construction mirrors the fixed runtime ABI fields.
    #[allow(clippy::too_many_arguments)]
    pub const fn hit(
        ticket: ZrRuntimeViewportPickTicket,
        request: ZrRuntimeViewportPickRequestV1,
        world_generation: u64,
        entity: u64,
        instance: u64,
        subobject: u64,
        depth: f32,
        world_position: [f32; 3],
        world_normal: [f32; 3],
    ) -> Self {
        Self {
            abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
            disposition: ZrRuntimeViewportPickDispositionV1::Hit.raw(),
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
            applied_policy_flags: request.policy_flags,
            reserved: 0,
        }
    }

    pub const fn disposition(self) -> Option<ZrRuntimeViewportPickDispositionV1> {
        ZrRuntimeViewportPickDispositionV1::from_raw(self.disposition)
    }

    /// FFI 出口与会话轮询用它检查 disposition、身份字段及命中数据组合是否合法；请求关联另由 `matches_request` 检查。
    pub fn validate_viewport_pick(self) -> bool {
        let Some(disposition) = self.disposition() else {
            return false;
        };
        let identity_is_valid = self.abi_version == ZIRCON_RUNTIME_ABI_VERSION_V1
            && self.ticket.is_valid()
            && self.viewport.is_valid()
            && self.viewport_size.width > 0
            && self.viewport_size.height > 0
            && self.pixel.x < self.viewport_size.width
            && self.pixel.y < self.viewport_size.height
            && ZrRuntimeViewportPickPurposeV1::from_raw(self.purpose).is_some()
            && self.frame_generation != 0
            && self.input_sequence != 0
            && self.applied_policy_flags & !ZR_RUNTIME_VIEWPORT_PICK_POLICY_KNOWN_FLAGS_V1 == 0
            && self.reserved == 0;
        if !identity_is_valid {
            return false;
        }

        let geometry_is_finite = self.depth.is_finite()
            && self.world_position.iter().all(|value| value.is_finite())
            && self.world_normal.iter().all(|value| value.is_finite());
        if !geometry_is_finite {
            return false;
        }

        if disposition == ZrRuntimeViewportPickDispositionV1::Hit {
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

    /// 编辑器消费异步 ticket 前逐项核对原请求身份，拒绝迟到帧或其他输入边沿的结果作用于当前视口。
    pub fn matches_request(self, request: ZrRuntimeViewportPickRequestV1) -> bool {
        request.validate_viewport_pick()
            && self.validate_viewport_pick()
            && self.viewport == request.viewport
            && self.viewport_size == request.viewport_size
            && self.pixel == request.pixel
            && self.purpose == request.purpose
            && self.frame_generation == request.frame_generation
            && self.input_sequence == request.input_sequence
            && self.applied_policy_flags == request.policy_flags
    }
}

#[cfg(test)]
#[path = "tests/viewport_pick.rs"]
mod tests;
