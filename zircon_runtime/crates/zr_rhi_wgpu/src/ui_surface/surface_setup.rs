use std::num::NonZeroIsize;

use zr_rhi::{RenderNativeSurfaceTarget, RhiError};

use crate::GPU_TIMESTAMP_REQUIRED_FEATURES;

const SURFACE_FRAME_LATENCY: u32 = 2;

pub(super) fn configure_surface(
    surface: &wgpu::Surface<'static>,
    adapter: &wgpu::Adapter,
    device: &wgpu::Device,
    size: (u32, u32),
) -> Result<wgpu::SurfaceConfiguration, RhiError> {
    let caps = surface.get_capabilities(adapter);
    let Some(format) = choose_surface_format(&caps.formats) else {
        return Err(RhiError::SurfaceUnavailable(
            "surface has no compatible formats".to_string(),
        ));
    };
    let Some(present_mode) = choose_present_mode(&caps.present_modes) else {
        return Err(RhiError::SurfaceUnavailable(
            "surface has no compatible present modes".to_string(),
        ));
    };
    let config = wgpu::SurfaceConfiguration {
        usage: choose_surface_usage(caps.usages),
        format,
        width: size.0.max(1),
        height: size.1.max(1),
        present_mode,
        desired_maximum_frame_latency: SURFACE_FRAME_LATENCY,
        alpha_mode: choose_alpha_mode(&caps.alpha_modes),
        view_formats: vec![],
    };
    surface.configure(device, &config);
    Ok(config)
}

pub(super) fn choose_surface_usage(supported_usages: wgpu::TextureUsages) -> wgpu::TextureUsages {
    let mut usage = wgpu::TextureUsages::RENDER_ATTACHMENT;
    if supported_usages.contains(wgpu::TextureUsages::COPY_DST) {
        usage |= wgpu::TextureUsages::COPY_DST;
    }
    usage
}

pub(super) fn choose_surface_format(
    formats: &[wgpu::TextureFormat],
) -> Option<wgpu::TextureFormat> {
    [
        wgpu::TextureFormat::Bgra8Unorm,
        wgpu::TextureFormat::Rgba8Unorm,
    ]
    .into_iter()
    .find(|format| formats.contains(format))
}

fn choose_present_mode(present_modes: &[wgpu::PresentMode]) -> Option<wgpu::PresentMode> {
    if present_modes.contains(&wgpu::PresentMode::AutoVsync) {
        Some(wgpu::PresentMode::AutoVsync)
    } else if present_modes.contains(&wgpu::PresentMode::Fifo) {
        Some(wgpu::PresentMode::Fifo)
    } else {
        present_modes.first().copied()
    }
}

pub(super) fn choose_alpha_mode(
    alpha_modes: &[wgpu::CompositeAlphaMode],
) -> wgpu::CompositeAlphaMode {
    if alpha_modes.contains(&wgpu::CompositeAlphaMode::Opaque) {
        wgpu::CompositeAlphaMode::Opaque
    } else if alpha_modes.contains(&wgpu::CompositeAlphaMode::Auto) {
        wgpu::CompositeAlphaMode::Auto
    } else {
        alpha_modes
            .first()
            .copied()
            .unwrap_or(wgpu::CompositeAlphaMode::Auto)
    }
}

pub(super) fn request_device(
    adapter: &wgpu::Adapter,
    allow_gpu_timing: bool,
) -> Result<(wgpu::Device, wgpu::Queue), RhiError> {
    let requested_features = requested_device_features(adapter.features(), allow_gpu_timing);
    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("zircon-ui-device"),
        required_features: requested_features,
        required_limits: wgpu::Limits::default(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
    }))
    .map_err(|error| RhiError::SurfaceUnavailable(error.to_string()))
}

pub(super) fn requested_device_features(
    adapter_features: wgpu::Features,
    allow_gpu_timing: bool,
) -> wgpu::Features {
    let mut requested_features = wgpu::Features::empty();
    if adapter_features.contains(wgpu::Features::INDIRECT_FIRST_INSTANCE) {
        requested_features |= wgpu::Features::INDIRECT_FIRST_INSTANCE;
    }
    if allow_gpu_timing && adapter_features.contains(GPU_TIMESTAMP_REQUIRED_FEATURES) {
        requested_features |= GPU_TIMESTAMP_REQUIRED_FEATURES;
    }
    requested_features
}

pub(super) fn instance_descriptor() -> wgpu::InstanceDescriptor {
    let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
    descriptor.backends = std::env::var("WGPU_BACKEND")
        .ok()
        .as_deref()
        .map(wgpu::Backends::from_comma_list)
        .unwrap_or_default();
    descriptor.flags = wgpu::InstanceFlags::from_build_config();
    if let Ok(debug) = std::env::var("WGPU_DEBUG") {
        descriptor
            .flags
            .set(wgpu::InstanceFlags::DEBUG, debug != "0");
    }
    if let Ok(validation) = std::env::var("WGPU_VALIDATION") {
        descriptor
            .flags
            .set(wgpu::InstanceFlags::VALIDATION, validation != "0");
    }
    descriptor.backend_options = wgpu::BackendOptions::from_env_or_default();
    descriptor
}

#[cfg(target_os = "windows")]
pub(super) fn create_surface(
    instance: &wgpu::Instance,
    target: RenderNativeSurfaceTarget,
) -> Result<wgpu::Surface<'static>, RhiError> {
    match target {
        RenderNativeSurfaceTarget::Win32 { hwnd, hinstance } => {
            let hwnd = required_nonzero_isize(hwnd, "invalid win32 hwnd")?;
            let mut window = wgpu::rwh::Win32WindowHandle::new(hwnd);
            window.hinstance = optional_nonzero_isize(hinstance)?;
            let raw_window_handle = wgpu::rwh::RawWindowHandle::Win32(window);
            let raw_display_handle =
                wgpu::rwh::RawDisplayHandle::Windows(wgpu::rwh::WindowsDisplayHandle::new());
            let target = wgpu::SurfaceTargetUnsafe::RawHandle {
                raw_display_handle: Some(raw_display_handle),
                raw_window_handle,
            };
            // The editor host owns the native window lifetime; runtime receives raw handles and
            // therefore must create the surface through wgpu's raw-handle entrypoint.
            unsafe { instance.create_surface_unsafe(target) }
                .map_err(|error| RhiError::SurfaceUnavailable(error.to_string()))
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub(super) fn create_surface(
    _instance: &wgpu::Instance,
    target: RenderNativeSurfaceTarget,
) -> Result<wgpu::Surface<'static>, RhiError> {
    match target {
        RenderNativeSurfaceTarget::Win32 { .. } => Err(RhiError::SurfaceUnavailable(
            "win32 retained UI surfaces are only supported on windows".to_string(),
        )),
    }
}

fn required_nonzero_isize(value: u64, error: &'static str) -> Result<NonZeroIsize, RhiError> {
    isize::try_from(value)
        .ok()
        .and_then(NonZeroIsize::new)
        .ok_or_else(|| RhiError::SurfaceUnavailable(error.to_string()))
}

fn optional_nonzero_isize(value: Option<u64>) -> Result<Option<NonZeroIsize>, RhiError> {
    value
        .map(|value| required_nonzero_isize(value, "invalid win32 hinstance"))
        .transpose()
}
