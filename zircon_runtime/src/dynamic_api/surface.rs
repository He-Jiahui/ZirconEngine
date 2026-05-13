use zircon_runtime_interface::{
    ZrByteSlice, ZrRuntimeBindViewportSurfaceRequestV1, ZrStatus, ZrStatusCode,
    ZIRCON_RUNTIME_ABI_VERSION_V1, ZR_RUNTIME_NATIVE_SURFACE_KIND_WIN32_V1,
};

use crate::core::framework::render::{RenderNativeSurfaceTarget, RenderViewportSurfaceDescriptor};
use crate::core::math::UVec2;

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
