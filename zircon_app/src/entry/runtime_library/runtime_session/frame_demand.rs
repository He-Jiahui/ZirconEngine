use std::time::Duration;

use zircon_runtime_interface::{
    ZrRuntimeFrameDemandV1, ZIRCON_RUNTIME_ABI_VERSION_V1, ZR_RUNTIME_FRAME_DEMAND_AFTER_V1,
    ZR_RUNTIME_FRAME_DEMAND_IDLE_V1, ZR_RUNTIME_FRAME_DEMAND_IMMEDIATE_V1,
};

use super::super::RuntimeLibraryError;

pub(crate) const MAX_HOST_RUNTIME_FRAME_DELAY: Duration = Duration::from_secs(60);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RuntimeFrameDemand {
    Idle,
    Immediate,
    After(Duration),
}

impl TryFrom<ZrRuntimeFrameDemandV1> for RuntimeFrameDemand {
    type Error = RuntimeLibraryError;

    fn try_from(demand: ZrRuntimeFrameDemandV1) -> Result<Self, Self::Error> {
        if demand.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V1 {
            return Err(RuntimeLibraryError::new(format!(
                "runtime frame demand used unsupported ABI version {}",
                demand.abi_version
            )));
        }
        match demand.kind {
            ZR_RUNTIME_FRAME_DEMAND_IDLE_V1 | ZR_RUNTIME_FRAME_DEMAND_IMMEDIATE_V1
                if demand.delay_nanoseconds != 0 =>
            {
                Err(RuntimeLibraryError::new(format!(
                    "runtime frame demand kind {} requires zero delay",
                    demand.kind
                )))
            }
            ZR_RUNTIME_FRAME_DEMAND_IDLE_V1 => Ok(Self::Idle),
            ZR_RUNTIME_FRAME_DEMAND_IMMEDIATE_V1 => Ok(Self::Immediate),
            ZR_RUNTIME_FRAME_DEMAND_AFTER_V1 => Ok(Self::After(
                Duration::from_nanos(demand.delay_nanoseconds).min(MAX_HOST_RUNTIME_FRAME_DELAY),
            )),
            kind => Err(RuntimeLibraryError::new(format!(
                "unsupported runtime frame demand kind {kind}"
            ))),
        }
    }
}
