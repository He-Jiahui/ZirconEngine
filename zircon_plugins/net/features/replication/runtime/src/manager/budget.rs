//! 将声明的更新频率转换成调度间隔；0 频率回退到一秒，避免除零。
//! 调度器使用此值判断 session 的下一次 snapshot 是否到期。

use zircon_runtime::core::framework::net::SyncComponentDescriptor;

use super::MILLIS_PER_SECOND;

pub(in crate::manager) fn update_interval_ms(descriptor: &SyncComponentDescriptor) -> u64 {
    if descriptor.update_hz == 0 {
        return MILLIS_PER_SECOND;
    }
    MILLIS_PER_SECOND.div_ceil(u64::from(descriptor.update_hz))
}
