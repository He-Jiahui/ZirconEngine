use zircon_runtime::core::framework::net::SyncComponentDescriptor;

use super::MILLIS_PER_SECOND;

pub(in crate::manager) fn update_interval_ms(descriptor: &SyncComponentDescriptor) -> u64 {
    if descriptor.update_hz == 0 {
        return MILLIS_PER_SECOND;
    }
    MILLIS_PER_SECOND.div_ceil(u64::from(descriptor.update_hz))
}
