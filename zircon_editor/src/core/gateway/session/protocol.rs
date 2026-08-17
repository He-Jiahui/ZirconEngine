use std::time::Duration;

use zircon_runtime_interface::{
    ZrRuntimeFrameDemandV1, ZrRuntimeOperationHandle, ZrRuntimeOperationStatusV2, ZrStatus,
    ZIRCON_RUNTIME_ABI_VERSION_V1, ZIRCON_RUNTIME_ABI_VERSION_V2, ZR_RUNTIME_FRAME_DEMAND_AFTER_V1,
    ZR_RUNTIME_FRAME_DEMAND_IDLE_V1, ZR_RUNTIME_FRAME_DEMAND_IMMEDIATE_V1,
    ZR_RUNTIME_FRAME_MAX_DIMENSION_V1, ZR_RUNTIME_FRAME_MAX_RGBA_BYTES_V1,
    ZR_RUNTIME_STATUS_DIAGNOSTICS_MAX_ENCODED_BYTES_V1,
};

use super::super::{EditorRuntimeFrameDemand, GatewayError};

const MAX_EDITOR_RUNTIME_FRAME_DELAY: Duration = Duration::from_secs(60);

pub(super) fn frame_demand_from_runtime(
    demand: ZrRuntimeFrameDemandV1,
) -> Result<EditorRuntimeFrameDemand, GatewayError> {
    if demand.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V1 {
        return Err(GatewayError::Protocol {
            message: format!(
                "runtime frame demand used unsupported ABI version {}",
                demand.abi_version
            ),
        });
    }
    if !demand.has_known_kind() {
        return Err(GatewayError::Protocol {
            message: format!("runtime frame demand used unknown kind {}", demand.kind),
        });
    }
    if demand.is_valid() {
        return match demand.kind {
            ZR_RUNTIME_FRAME_DEMAND_IDLE_V1 => Ok(EditorRuntimeFrameDemand::OnDemand),
            ZR_RUNTIME_FRAME_DEMAND_AFTER_V1 => Ok(EditorRuntimeFrameDemand::SleepUntil(
                Duration::from_nanos(demand.delay_nanoseconds).min(MAX_EDITOR_RUNTIME_FRAME_DELAY),
            )),
            ZR_RUNTIME_FRAME_DEMAND_IMMEDIATE_V1 => Ok(EditorRuntimeFrameDemand::Continuous),
            _ => Err(GatewayError::Protocol {
                message: format!(
                    "runtime frame demand kind {} became unsupported after validation",
                    demand.kind
                ),
            }),
        };
    }
    Err(GatewayError::Protocol {
        message: format!(
            "runtime frame demand kind {} returned invalid delay {}ns",
            demand.kind, demand.delay_nanoseconds
        ),
    })
}

pub(super) fn ensure_frame_rgba_shape(
    width: u32,
    height: u32,
    rgba: &[u8],
) -> Result<(), GatewayError> {
    if width > ZR_RUNTIME_FRAME_MAX_DIMENSION_V1 || height > ZR_RUNTIME_FRAME_MAX_DIMENSION_V1 {
        return Err(GatewayError::Protocol {
            message: format!(
                "runtime frame dimensions {width}x{height} exceed maximum {ZR_RUNTIME_FRAME_MAX_DIMENSION_V1}"
            ),
        });
    }
    let expected = u64::from(width)
        .checked_mul(u64::from(height))
        .and_then(|pixels| pixels.checked_mul(4))
        .and_then(|bytes| usize::try_from(bytes).ok())
        .ok_or_else(|| GatewayError::Protocol {
            message: format!("runtime frame dimensions {width}x{height} overflow RGBA byte length"),
        })?;
    if expected > ZR_RUNTIME_FRAME_MAX_RGBA_BYTES_V1 {
        return Err(GatewayError::Protocol {
            message: format!(
                "runtime frame RGBA length {expected} exceeds maximum {ZR_RUNTIME_FRAME_MAX_RGBA_BYTES_V1}"
            ),
        });
    }
    if rgba.is_empty() {
        return Ok(());
    }
    if rgba.len() == expected {
        return Ok(());
    }
    Err(GatewayError::Protocol {
        message: format!(
            "runtime frame {width}x{height} returned {} RGBA bytes; expected {expected}",
            rgba.len()
        ),
    })
}

pub(super) fn ensure_operation_handle(
    response: ZrRuntimeOperationHandle,
    requested: ZrRuntimeOperationHandle,
    output_kind: &'static str,
) -> Result<(), GatewayError> {
    if response == requested {
        return Ok(());
    }
    Err(GatewayError::Protocol {
        message: format!(
            "{output_kind} handle {} did not match requested handle {}",
            response.raw(),
            requested.raw()
        ),
    })
}

pub(super) fn ensure_output_abi(
    abi_version: u32,
    expected_abi_version: u32,
    output_kind: &'static str,
) -> Result<(), GatewayError> {
    if abi_version == expected_abi_version {
        return Ok(());
    }
    Err(GatewayError::Protocol {
        message: format!(
            "{output_kind} used unsupported ABI version {abi_version}; expected {expected_abi_version}"
        ),
    })
}

pub(super) fn ensure_operation_status(
    status: &ZrRuntimeOperationStatusV2,
    requested: ZrRuntimeOperationHandle,
) -> Result<(), GatewayError> {
    if status.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V2 {
        return Err(GatewayError::Protocol {
            message: format!(
                "runtime operation status used unsupported ABI version {}",
                status.abi_version
            ),
        });
    }
    if status.reserved != 0 {
        return Err(GatewayError::Protocol {
            message: format!(
                "runtime operation status reserved field must be zero, got {}",
                status.reserved
            ),
        });
    }
    ensure_operation_handle(status.handle, requested, "runtime operation status")?;
    if status.phase().is_none() {
        return Err(GatewayError::Protocol {
            message: format!(
                "runtime operation status used unknown phase {}",
                status.phase
            ),
        });
    }
    if status.detail_kind().is_none() {
        return Err(GatewayError::Protocol {
            message: format!(
                "runtime operation status used unknown detail kind {}",
                status.detail_kind
            ),
        });
    }
    Ok(())
}

pub(super) fn ensure_status(status: ZrStatus, operation: &'static str) -> Result<(), GatewayError> {
    if status.is_ok() {
        return Ok(());
    }
    let diagnostics = unsafe {
        status
            .diagnostics
            .checked_slice(ZR_RUNTIME_STATUS_DIAGNOSTICS_MAX_ENCODED_BYTES_V1)
    }
    .map_err(|error| GatewayError::Protocol {
        message: format!("{operation} returned invalid status diagnostics: {error:?}"),
    })?;
    let diagnostics = String::from_utf8_lossy(diagnostics);
    Err(GatewayError::Runtime {
        message: format!(
            "{operation} failed with status {:?}: {diagnostics}",
            status.status_code()
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_shape_rejects_shared_dimension_and_rgba_limits() {
        let dimension_error =
            ensure_frame_rgba_shape(ZR_RUNTIME_FRAME_MAX_DIMENSION_V1 + 1, 1, &[])
                .expect_err("oversized foreign frame dimensions must be rejected");
        assert!(matches!(
            dimension_error,
            GatewayError::Protocol { message }
                if message.contains("exceed maximum 16384")
        ));

        let rgba_error = ensure_frame_rgba_shape(ZR_RUNTIME_FRAME_MAX_DIMENSION_V1, 4_097, &[])
            .expect_err("oversized foreign RGBA lengths must be rejected");
        assert!(matches!(
            rgba_error,
            GatewayError::Protocol { message }
                if message.contains("exceeds maximum 268435456")
        ));
    }
}
