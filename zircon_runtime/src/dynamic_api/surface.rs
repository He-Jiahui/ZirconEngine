use zircon_runtime_interface::{
    ZrByteSlice, ZrRuntimeBindViewportSurfaceRequestV1, ZrStatus, ZrStatusCode,
    ZIRCON_RUNTIME_ABI_VERSION_V1, ZR_RUNTIME_NATIVE_SURFACE_KIND_WIN32_V1,
};

use crate::core::framework::render::RenderViewportSurfaceDescriptor;
use crate::core::math::UVec2;
use crate::rhi::RenderNativeSurfaceTarget;

pub(super) fn render_surface_descriptor(
    request: ZrRuntimeBindViewportSurfaceRequestV1,
) -> Result<RenderViewportSurfaceDescriptor, ZrStatus> {
    if request.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V1 {
        return Err(unsupported_version());
    }
    if request.target.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V1 {
        return Err(unsupported_version());
    }

    let size = UVec2::new(request.size.width.max(1), request.size.height.max(1));
    let target = match request.target.kind {
        ZR_RUNTIME_NATIVE_SURFACE_KIND_WIN32_V1 => {
            if request.target.window_handle == 0 {
                return Err(invalid_argument(b"invalid runtime native window handle"));
            }
            RenderNativeSurfaceTarget::Win32 {
                hwnd: request.target.window_handle,
                hinstance: (request.target.display_handle != 0)
                    .then_some(request.target.display_handle),
            }
        }
        _ => {
            return Err(invalid_argument(
                b"unsupported runtime native surface target",
            ))
        }
    };

    Ok(RenderViewportSurfaceDescriptor::new(size, target))
}

pub(super) fn render_surface_descriptor_v2(
    request: zircon_runtime_interface::ZrRuntimeBindViewportSurfaceRequestV2,
) -> Result<RenderViewportSurfaceDescriptor, ZrStatus> {
    use crate::core::framework::window::WindowPresentMode;
    use zircon_runtime_interface::{ZrRuntimePresentModeV2, ZIRCON_RUNTIME_ABI_VERSION_V2};
    if request.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V2 {
        return Err(unsupported_version());
    }
    if !request.is_valid() {
        return Err(invalid_argument(
            b"invalid V2 viewport surface request shape or mode",
        ));
    }
    let mode = match ZrRuntimePresentModeV2::from_raw(request.present_mode) {
        Some(ZrRuntimePresentModeV2::Fifo) => WindowPresentMode::Fifo,
        Some(ZrRuntimePresentModeV2::Immediate) => WindowPresentMode::Immediate,
        Some(ZrRuntimePresentModeV2::Mailbox) => WindowPresentMode::Mailbox,
        Some(ZrRuntimePresentModeV2::FifoRelaxed) => WindowPresentMode::FifoRelaxed,
        Some(ZrRuntimePresentModeV2::AutoVsync) => WindowPresentMode::AutoVsync,
        Some(ZrRuntimePresentModeV2::AutoNoVsync) => WindowPresentMode::AutoNoVsync,
        None => return Err(invalid_argument(b"invalid V2 presentation mode")),
    };
    render_surface_descriptor(ZrRuntimeBindViewportSurfaceRequestV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        request.viewport,
        request.size,
        request.target,
    ))
    .map(|descriptor| descriptor.with_present_mode(mode))
}

fn unsupported_version() -> ZrStatus {
    ZrStatus::new(
        ZrStatusCode::UnsupportedVersion,
        ZrByteSlice::from_static(b"unsupported runtime ABI version"),
    )
}

fn invalid_argument(message: &'static [u8]) -> ZrStatus {
    ZrStatus::new(
        ZrStatusCode::InvalidArgument,
        ZrByteSlice::from_static(message),
    )
}
