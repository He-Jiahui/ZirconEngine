use zircon_runtime_interface::{
    ZrNativeSystemAccessV1, ZrSystemRegistrationV2, ZR_NATIVE_SYSTEM_ACCESS_DOMAIN_COMPONENT_V1,
    ZR_NATIVE_SYSTEM_ACCESS_DOMAIN_RESOURCE_V1, ZR_NATIVE_SYSTEM_ACCESS_MODE_READ_V1,
    ZR_NATIVE_SYSTEM_ACCESS_MODE_WRITE_V1, ZR_NATIVE_SYSTEM_THREAD_AFFINITY_MAIN_THREAD_ONLY_V1,
    ZR_NATIVE_SYSTEM_THREAD_AFFINITY_WORKER_SAFE_V1,
};

use crate::scene::ecs::SystemStage;

use super::super::super::registration_manifest::NativePluginRegistrationThreadAffinity;
use super::{read_utf8_with, AbiDecodeError, AbiDecodeResult};

pub(super) const MAX_NATIVE_SYSTEM_ACCESS_ENTRIES: usize = 4_096;

pub(in super::super) fn validate_v4_registration_header(
    registration: &ZrSystemRegistrationV2,
) -> AbiDecodeResult<()> {
    if registration.abi_version != 4 {
        return Err(AbiDecodeError::InvalidV4RegistrationAbiVersion {
            actual: registration.abi_version,
        });
    }
    if registration.size_bytes != std::mem::size_of::<ZrSystemRegistrationV2>() {
        return Err(AbiDecodeError::InvalidV4RegistrationSize {
            actual: registration.size_bytes,
        });
    }
    if registration.access_count == 0 {
        return Err(AbiDecodeError::EmptyV4AccessContract);
    }
    Ok(())
}

pub(in super::super) fn stage_from_abi(stage: u32) -> AbiDecodeResult<SystemStage> {
    SystemStage::ORDER
        .get(stage as usize)
        .copied()
        .ok_or(AbiDecodeError::UnknownSystemStage { stage })
}

pub(in super::super) unsafe fn read_v4_system_accesses(
    values: *const ZrNativeSystemAccessV1,
    count: usize,
) -> AbiDecodeResult<Vec<String>> {
    if count == 0 {
        return Ok(Vec::new());
    }
    if values.is_null() {
        return Err(AbiDecodeError::InvalidV4AccessPointer { count });
    }
    if count > MAX_NATIVE_SYSTEM_ACCESS_ENTRIES {
        return Err(AbiDecodeError::TooManyV4Accesses { count });
    }
    unsafe { std::slice::from_raw_parts(values, count) }
        .iter()
        .map(|access| {
            if access.abi_version != 1 {
                return Err(AbiDecodeError::InvalidV4AccessAbiVersion {
                    actual: access.abi_version,
                });
            }
            if access.size_bytes != std::mem::size_of::<ZrNativeSystemAccessV1>() {
                return Err(AbiDecodeError::InvalidV4AccessSize {
                    actual: access.size_bytes,
                });
            }
            let mode = match access.mode {
                ZR_NATIVE_SYSTEM_ACCESS_MODE_READ_V1 => "read",
                ZR_NATIVE_SYSTEM_ACCESS_MODE_WRITE_V1 => "write",
                mode => return Err(AbiDecodeError::InvalidV4AccessMode { mode }),
            };
            let domain = match access.domain {
                ZR_NATIVE_SYSTEM_ACCESS_DOMAIN_COMPONENT_V1 => "component",
                ZR_NATIVE_SYSTEM_ACCESS_DOMAIN_RESOURCE_V1 => "resource",
                domain => return Err(AbiDecodeError::InvalidV4AccessDomain { domain }),
            };
            unsafe {
                read_utf8_with(access.stable_id, |stable_id| {
                    system_access_id(mode, domain, stable_id)
                })
            }
        })
        .collect()
}

fn system_access_id(mode: &str, domain: &str, stable_id: &str) -> String {
    let capacity = mode
        .len()
        .saturating_add(domain.len())
        .saturating_add(stable_id.len())
        .saturating_add(2);
    let mut access_id = String::with_capacity(capacity);
    access_id.push_str(mode);
    access_id.push(':');
    access_id.push_str(domain);
    access_id.push(':');
    access_id.push_str(stable_id);
    access_id
}

pub(in super::super) fn v4_thread_affinity_from_abi(
    affinity: u32,
) -> AbiDecodeResult<NativePluginRegistrationThreadAffinity> {
    match affinity {
        ZR_NATIVE_SYSTEM_THREAD_AFFINITY_MAIN_THREAD_ONLY_V1 => {
            Ok(NativePluginRegistrationThreadAffinity::MainThreadOnly)
        }
        ZR_NATIVE_SYSTEM_THREAD_AFFINITY_WORKER_SAFE_V1 => {
            Ok(NativePluginRegistrationThreadAffinity::WorkerSafe)
        }
        affinity => Err(AbiDecodeError::InvalidV4ThreadAffinity { affinity }),
    }
}

#[cfg(test)]
mod tests {
    use super::system_access_id;

    #[test]
    fn exact_capacity_system_access_id_preserves_output() {
        assert_eq!(
            system_access_id("read", "component", "weather.velocity"),
            "read:component:weather.velocity"
        );
    }
}
