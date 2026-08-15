use std::time::Duration;

use zircon_runtime_interface::{
    ZIRCON_RUNTIME_ABI_VERSION_V1, ZR_RUNTIME_FRAME_DEMAND_AFTER_V1,
    ZR_RUNTIME_FRAME_DEMAND_IDLE_V1, ZR_RUNTIME_FRAME_DEMAND_IMMEDIATE_V1, ZrRuntimeFrameDemandV1,
};

pub(in crate::dynamic_api::session) const MAX_RUNTIME_FRAME_DEMAND_DELAY: Duration =
    Duration::from_secs(60);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::dynamic_api::session) enum RuntimeFrameDemand {
    #[default]
    Idle,
    Immediate,
    After(Duration),
}

impl RuntimeFrameDemand {
    pub(in crate::dynamic_api::session) fn into_abi(self) -> ZrRuntimeFrameDemandV1 {
        match self.clamped() {
            Self::Idle => ZrRuntimeFrameDemandV1::idle(),
            Self::Immediate => ZrRuntimeFrameDemandV1::immediate(),
            Self::After(delay) => ZrRuntimeFrameDemandV1::after(delay.as_nanos() as u64),
        }
    }

    fn clamped(self) -> Self {
        match self {
            Self::After(delay) => Self::After(delay.min(MAX_RUNTIME_FRAME_DEMAND_DELAY)),
            demand => demand,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::dynamic_api::session) enum InvalidRuntimeFrameDemand {
    UnsupportedVersion,
    UnknownKind,
    InvalidDelay,
}

impl TryFrom<ZrRuntimeFrameDemandV1> for RuntimeFrameDemand {
    type Error = InvalidRuntimeFrameDemand;

    fn try_from(demand: ZrRuntimeFrameDemandV1) -> Result<Self, Self::Error> {
        if demand.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V1 {
            return Err(InvalidRuntimeFrameDemand::UnsupportedVersion);
        }
        match demand.kind {
            ZR_RUNTIME_FRAME_DEMAND_IDLE_V1 if demand.delay_nanoseconds == 0 => Ok(Self::Idle),
            ZR_RUNTIME_FRAME_DEMAND_IMMEDIATE_V1 if demand.delay_nanoseconds == 0 => {
                Ok(Self::Immediate)
            }
            ZR_RUNTIME_FRAME_DEMAND_AFTER_V1 => {
                Ok(Self::After(Duration::from_nanos(demand.delay_nanoseconds)).clamped())
            }
            ZR_RUNTIME_FRAME_DEMAND_IDLE_V1 | ZR_RUNTIME_FRAME_DEMAND_IMMEDIATE_V1 => {
                Err(InvalidRuntimeFrameDemand::InvalidDelay)
            }
            _ => Err(InvalidRuntimeFrameDemand::UnknownKind),
        }
    }
}

#[derive(Debug, Default)]
pub(super) struct FrameDemandAccumulator {
    current: RuntimeFrameDemand,
}

impl FrameDemandAccumulator {
    pub(super) fn merge(&mut self, incoming: RuntimeFrameDemand) {
        let incoming = incoming.clamped();
        self.current = match (self.current, incoming) {
            (RuntimeFrameDemand::Immediate, _) | (_, RuntimeFrameDemand::Immediate) => {
                RuntimeFrameDemand::Immediate
            }
            (RuntimeFrameDemand::After(current), RuntimeFrameDemand::After(incoming)) => {
                RuntimeFrameDemand::After(current.min(incoming))
            }
            (RuntimeFrameDemand::Idle, incoming) => incoming,
            (current, RuntimeFrameDemand::Idle) => current,
        };
    }

    pub(super) fn consume(&mut self) -> RuntimeFrameDemand {
        std::mem::take(&mut self.current)
    }
}
