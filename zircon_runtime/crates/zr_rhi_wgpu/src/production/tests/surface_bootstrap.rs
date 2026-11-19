#[test]
fn surface_bootstrap_transfers_one_compatible_native_surface_into_the_neutral_device() {
    let bootstrap = include_str!("../surface_bootstrap.rs");
    let selected_adapter = include_str!("../surface_bootstrap_adapter.rs");
    let device = include_str!("../device.rs");
    let surface_lifecycle = include_str!("../device/surface_lifecycle.rs");
    let service = include_str!("../surface.rs");

    for required in [
        "pub struct WgpuSurfaceBootstrap",
        "instance: wgpu::Instance",
        "surface: wgpu::Surface<'static>",
        "pub fn select_compatible_adapter(\n        self,",
        "let surface_ref = &surface;",
        "surface_ref.get_capabilities(&adapter)",
        "surface_descriptor_is_supported(&capabilities, &descriptor)",
    ] {
        assert!(
            bootstrap.contains(required),
            "surface bootstrap must retain the one-way handoff step `{required}`"
        );
    }

    for required in [
        "pub struct WgpuSurfaceAdapterBootstrap",
        "pub fn adapter(&self) -> &wgpu::Adapter",
        "pub fn request_render_device(",
        "pollster::block_on(adapter.request_device(native_descriptor))",
        "WgpuRenderDeviceContext::new(instance, adapter, device, queue)",
        "WgpuRenderDevice::new(context, profile)?",
        "render_device.adopt_surface_session(descriptor, surface)?",
    ] {
        assert!(
            selected_adapter.contains(required),
            "selected-adapter bootstrap must retain the one-way handoff step `{required}`"
        );
    }

    assert!(
        device.contains("mod surface_lifecycle;")
            && surface_lifecycle.contains("impl WgpuRenderDevice {")
            && surface_lifecycle.contains("fn adopt_surface_session("),
        "the neutral device must own the bootstrap handoff endpoint"
    );
    assert!(
        service.contains("fn adopt_session("),
        "the surface service must register an adopted surface instead of recreating it"
    );
    assert!(
        service.contains("fn surface_descriptor_is_supported("),
        "surface bootstrap selection must use the same format and present-mode contract as session creation"
    );
    let frames = service
        .find("frames: HashMap<SurfaceFrameId, WgpuSurfaceFrame>")
        .expect("surface service must track acquired frame ownership");
    let sessions = service
        .find("sessions: HashMap<SurfaceSession, WgpuSurfaceSession>")
        .expect("surface service must track native surface ownership");
    assert!(
        frames < sessions,
        "acquired SurfaceTexture frames must drop before their owning native surfaces"
    );
    assert!(
        !bootstrap.contains("pub fn native_surface"),
        "surface bootstrap must not expose a native surface escape hatch"
    );
    assert!(
        !bootstrap.contains("pub fn into_surface"),
        "surface bootstrap must transfer native ownership only into WgpuRenderDevice"
    );
    assert!(
        !bootstrap.contains("pub fn request_render_device"),
        "only the selected surface-compatible adapter may create and transfer a render device"
    );
    assert!(
        !selected_adapter.contains("pub fn native_surface"),
        "selected-adapter bootstrap must not expose a native surface escape hatch"
    );
    assert!(
        !selected_adapter.contains("pub fn into_surface"),
        "selected-adapter bootstrap must retain native surface ownership until handoff"
    );
    assert!(
        !selected_adapter.contains("device: wgpu::Device"),
        "selected-adapter bootstrap must request its own native device rather than accept one"
    );
    assert!(
        !selected_adapter.contains("queue: wgpu::Queue"),
        "selected-adapter bootstrap must request its own native queue rather than accept one"
    );
    let profile_guard = selected_adapter
        .find("if profile.adapter() != &adapter_facts")
        .expect("selected-adapter bootstrap must reject a profile for another adapter");
    let device_request = selected_adapter
        .find("adapter.request_device(native_descriptor)")
        .expect("selected-adapter bootstrap must request a device from its stored adapter");
    assert!(
        profile_guard < device_request,
        "profile adapter validation must fail before requesting a native device"
    );
}

use std::{
    cell::Cell,
    future::{poll_fn, Future},
    task::{Context, Poll, Waker},
};

use zr_rhi::{RenderAdapterFacts, RenderDeviceProfile, RhiError};

use super::super::surface_bootstrap_adapter::request_with_supplied_profile;

const NATIVE_TOKENS: (u8, u8) = (11, 37);
type ProfileRequestResult = Result<(u8, u8, RenderDeviceProfile), RhiError>;

fn observe_profile_request(
    selected: &RenderAdapterFacts,
    profile: RenderDeviceProfile,
    provider_result: Result<(u8, u8), RhiError>,
) -> (ProfileRequestResult, (usize, usize)) {
    let starts = Cell::new(0);
    let polls = Cell::new(0);
    let (start_counter, poll_counter) = (&starts, &polls);
    let mut request = std::pin::pin!(request_with_supplied_profile(
        selected,
        profile,
        move || {
            start_counter.set(start_counter.get() + 1);
            let mut provider_result = Some(provider_result);
            poll_fn(move |_| {
                poll_counter.set(poll_counter.get() + 1);
                Poll::Ready(provider_result.take().expect("provider was polled again"))
            })
        }
    ));
    assert_eq!((starts.get(), polls.get()), (0, 0), "unpolled scheduler");
    let mut context = Context::from_waker(Waker::noop());
    let Poll::Ready(result) = request.as_mut().poll(&mut context) else {
        panic!("the ready provider must not suspend the scheduler");
    };
    (result, (starts.get(), polls.get()))
}

#[test]
fn profile_request_mismatch_does_not_start_request() {
    let profile = super::test_profile();
    let mut selected = profile.adapter().clone();
    selected.name.push_str(" (selected)");
    let expected = RhiError::NativeContextAdapterMismatch {
        profile_adapter: Box::new(profile.adapter().clone()),
        context_adapter: selected.clone(),
    };
    let (result, counts) = observe_profile_request(&selected, profile, Ok(NATIVE_TOKENS));

    assert_eq!(result, Err(expected));
    assert_eq!(
        counts,
        (0, 0),
        "mismatched profile must neither invoke the request thunk nor poll its future"
    );
}

#[test]
fn profile_request_match_passes_through_success() {
    let profile = super::test_profile();
    let selected = profile.adapter().clone();
    let (result, counts) = observe_profile_request(&selected, profile.clone(), Ok(NATIVE_TOKENS));

    assert_eq!(counts, (1, 1));
    assert_eq!(result, Ok((NATIVE_TOKENS.0, NATIVE_TOKENS.1, profile)));
}

#[test]
fn profile_request_match_preserves_provider_error() {
    let profile = super::test_profile();
    let selected = profile.adapter().clone();
    let provider_error = RhiError::SurfaceUnavailable("provider rejected device request".into());
    let (result, counts) = observe_profile_request(&selected, profile, Err(provider_error.clone()));

    assert_eq!(counts, (1, 1));
    assert_eq!(result, Err(provider_error));
}
