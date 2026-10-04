//! 把 Runtime 动态 ABI 的帧需求转换为宿主事件循环能消费的调度建议。
//! 无效版本或互相矛盾的字段必须在进入事件循环前被拒绝。

use std::time::Duration;

use zircon_runtime_interface::{
    ZrRuntimeFrameDemandV1, ZIRCON_RUNTIME_ABI_VERSION_V1, ZR_RUNTIME_FRAME_DEMAND_AFTER_V1,
    ZR_RUNTIME_FRAME_DEMAND_IDLE_V1, ZR_RUNTIME_FRAME_DEMAND_IMMEDIATE_V1,
};

use super::super::RuntimeLibraryError;

pub(crate) const MAX_HOST_RUNTIME_FRAME_DELAY: Duration = Duration::from_secs(60);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 由一次 tick 返回给宿主的下一帧需求；After 值受宿主最长等待约束。
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
