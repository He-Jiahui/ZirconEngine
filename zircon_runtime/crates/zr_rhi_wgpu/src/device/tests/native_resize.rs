//! Opt-in DX12 regression on one real, hidden Win32 window per test.

use std::{ffi::c_void, io, ptr, ptr::NonNull, time::Duration};

use zr_rhi::{
    AdapterSelectionPolicy, DeviceFaultKind, GpuMemorySnapshot, PresentMode, RenderAdapterClass,
    RenderBackendKind, RenderClearColor, RenderDevice, RenderDeviceRequestPolicy,
    RenderNativeSurfaceTarget, RenderPassColorAttachmentDesc, RenderPassColorLoadOp,
    RenderPassStoreOp, RenderPassTextureViewDesc, RenderQueueClass, RenderSurfaceDescriptor,
    RenderSurfaceHandleError, RhiError, SubmissionStatus, SubmissionTicket, SurfaceAcquireOutcome,
    SurfaceFrameLease, SurfaceFrameTerminal, SurfaceSessionCreateOutcome, SurfaceSessionReceipt,
    SwapchainDesc, TextureFormat,
};

use crate::{wgpu_device_request, WgpuRenderDevice, WgpuSurfaceBootstrap};

const INITIAL: (u32, u32) = (192, 128);
const RESIZED: (u32, u32) = (256, 144);
const COMPLETION_TIMEOUT: Duration = Duration::from_secs(5);

#[test]
#[ignore = "requires a real Win32 desktop and exclusive physical DX12 adapter"]
fn native_resize_retired_frame() {
    let (fixture, initial) = Fixture::new();
    let (old_frame, ticket) = fixture.clear_present(&initial, INITIAL, "PREFLIGHT");
    fixture.wait_completed(ticket, "PREFLIGHT");

    // The empty, exclusive device has created only this one texture. This snapshot reads
    // the registry without pruning. Views are not counted as separate allocations.
    let retired = fixture.owner.memory_snapshot();
    assert_eq!(
        retired,
        GpuMemorySnapshot {
            retired_texture_bytes: u64::from(INITIAL.0) * u64::from(INITIAL.1) * 4,
            retired_allocations: 1,
            ..GpuMemorySnapshot::default()
        },
        "NATIVE_RESIZE_PREFLIGHT: the presented frame must still own one retired texture"
    );
    fixture.window.resize(RESIZED);
    eprintln!(
        "NATIVE_RESIZE_READY: {INITIAL:?} -> {RESIZED:?}; scene ticket {ticket:?} Completed; {retired:?}"
    );

    let resized = fixture.reconfigure(&initial, RESIZED, "WITNESS");
    let old_frame_id = old_frame.frame();
    assert!(matches!(
        fixture.owner.discard_surface_frame(old_frame),
        Err(RhiError::SurfaceFrameAlreadyTerminal { frame, terminal: SurfaceFrameTerminal::Presented })
            if frame == old_frame_id
    ));
    let (_, new_ticket) = fixture.clear_present(&resized, RESIZED, "POST_RESIZE");
    fixture.wait_completed(new_ticket, "POST_RESIZE");
    fixture
        .owner
        .destroy_surface_session(resized.session())
        .unwrap();
    eprintln!("NATIVE_RESIZE_WITNESS_PASSED: new frame presented and {new_ticket:?} Completed");
}

#[test]
#[ignore = "requires a real Win32 desktop and exclusive physical DX12 adapter"]
fn native_resize_drained_control() {
    let (fixture, initial) = Fixture::new();
    let (_, ticket) = fixture.clear_present(&initial, INITIAL, "CONTROL_STEADY");
    fixture.wait_completed(ticket, "CONTROL_STEADY");
    // A single normal maintenance call is the deliberate difference from the witness.
    fixture.owner.poll_submissions().unwrap();
    assert_eq!(
        fixture.owner.memory_snapshot(),
        GpuMemorySnapshot::default()
    );

    fixture.window.resize(RESIZED);
    let resized = fixture.reconfigure(&initial, RESIZED, "CONTROL");
    let (_, ticket) = fixture.clear_present(&resized, RESIZED, "CONTROL_RESIZED");
    fixture.wait_completed(ticket, "CONTROL_RESIZED");

    // Exercise the existing raw RHI gate without minimizing or resizing the HWND to zero.
    let zero = swapchain_at(&resized, (0, RESIZED.1));
    let outcome = fixture
        .owner
        .reconfigure_surface_session(resized.session(), &zero);
    fixture.assert_healthy("CONTROL_ZERO");
    let SurfaceSessionCreateOutcome::NonRenderable(non_renderable) =
        outcome.expect("NATIVE_RESIZE_CONTROL_ZERO: raw zero reconfigure")
    else {
        panic!("NATIVE_RESIZE_CONTROL_ZERO: zero width must not be renderable");
    };
    assert_eq!(non_renderable.swapchain.width, 0);
    assert_eq!(non_renderable.swapchain.height, RESIZED.1);
    assert_eq!(
        fixture
            .owner
            .acquire_surface_frame(non_renderable.session())
            .unwrap(),
        SurfaceAcquireOutcome::NonRenderable {
            session: non_renderable.session()
        }
    );
    fixture
        .owner
        .destroy_surface_session(non_renderable.session())
        .unwrap();
    eprintln!("NATIVE_RESIZE_CONTROL_PASSED: physical clear/present and raw zero gate");
}

struct Fixture {
    // Drop the device/surface and the native wait handle before destroying the HWND.
    owner: WgpuRenderDevice,
    native: wgpu::Device,
    window: HiddenWindow,
}

impl Fixture {
    fn new() -> (Self, SurfaceSessionReceipt) {
        let window = HiddenWindow::new(INITIAL);
        let mut desc = wgpu::InstanceDescriptor::new_without_display_handle();
        desc.backends = wgpu::Backends::DX12;
        let instance = wgpu::Instance::new(desc);
        let descriptor = RenderSurfaceDescriptor::new(
            "native-resize",
            window.target(),
            SwapchainDesc {
                width: INITIAL.0,
                height: INITIAL.1,
                present_mode: PresentMode::Fifo,
                format: TextureFormat::Bgra8UnormSrgb,
            },
        );
        let bootstrap = WgpuSurfaceBootstrap::new(instance, descriptor)
            .expect("NATIVE_RESIZE_PREFLIGHT: create real Win32 surface");
        let adapter = bootstrap
            .select_compatible_adapter(wgpu::Backends::DX12, &AdapterSelectionPolicy::default())
            .expect("NATIVE_RESIZE_PREFLIGHT: no compatible physical DX12 adapter is a hold");
        let facts = adapter.adapter_facts();
        eprintln!("NATIVE_RESIZE_ADAPTER: {facts:?}");
        assert_eq!(facts.backend, RenderBackendKind::Dx12);
        assert_eq!(adapter.adapter().get_info().backend, wgpu::Backend::Dx12);
        assert!(
            matches!(
                facts.adapter_class,
                RenderAdapterClass::Discrete | RenderAdapterClass::Integrated
            ),
            "NATIVE_RESIZE_PREFLIGHT: requires an identified physical graphics adapter"
        );
        let request = wgpu_device_request(
            adapter.adapter().features(),
            &RenderDeviceRequestPolicy::mvp_baseline(),
        )
        .expect("NATIVE_RESIZE_PREFLIGHT: negotiate the MVP device request");
        let native_desc = wgpu::DeviceDescriptor {
            label: Some("native-resize-dx12"),
            required_features: request.requested_features(),
            required_limits: wgpu::Limits::default(),
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::Off,
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
        };
        let (owner, native, outcome) =
            pollster::block_on(adapter.request_graphics_device_async(&native_desc, request))
                .expect(
                    "NATIVE_RESIZE_PREFLIGHT: create DX12 device and configure initial surface",
                );
        let fixture = Self {
            owner,
            native,
            window,
        };
        fixture.assert_healthy("PREFLIGHT");
        assert_eq!(
            fixture.owner.memory_snapshot(),
            GpuMemorySnapshot::default()
        );
        (fixture, renderable(outcome, INITIAL, "PREFLIGHT"))
    }

    fn clear_present(
        &self,
        surface: &SurfaceSessionReceipt,
        extent: (u32, u32),
        stage: &str,
    ) -> (SurfaceFrameLease, SubmissionTicket) {
        let outcome = self.owner.acquire_surface_frame(surface.session());
        let Ok(SurfaceAcquireOutcome::Acquired(frame)) = outcome else {
            panic!("NATIVE_RESIZE_{stage}: native acquire did not return a lease: {outcome:?}");
        };
        assert_eq!(frame.session(), surface.session());
        assert_eq!((frame.desc().width, frame.desc().height), extent);
        assert_eq!(frame.desc().format, surface.swapchain.format);
        {
            // Borrow the actual acquired WGPU texture; do not extend its native lifetime.
            let registry = self.owner.lock_registry();
            let texture = registry.texture(frame.target()).unwrap();
            assert_eq!((texture.width(), texture.height()), extent);
        }
        let mut list = self
            .owner
            .create_command_list(RenderQueueClass::Graphics, "native-resize")
            .unwrap();
        let attachment = RenderPassColorAttachmentDesc::new(
            frame.target(),
            RenderPassColorLoadOp::Clear(RenderClearColor::BLACK),
            RenderPassStoreOp::Store,
        )
        .with_view(
            RenderPassTextureViewDesc::new(frame.target())
                .with_registered_view(frame.default_view()),
        );
        list.begin_render_pass("native-resize-clear", vec![attachment], None);
        list.end_render_pass();
        let ticket = self.owner.enqueue_command_list(list).unwrap();
        assert_eq!(self.owner.flush_submissions().unwrap(), 1);
        let presented = self
            .owner
            .present_surface_frame(frame.clone(), ticket)
            .unwrap();
        assert_eq!(presented.frame, frame.frame());
        assert_eq!(presented.submission, ticket);
        assert_eq!(presented.terminal, SurfaceFrameTerminal::Presented);
        assert!(self.owner.texture_desc(frame.target()).is_err());
        assert!(self.owner.texture_view_desc(frame.default_view()).is_err());
        (frame, ticket)
    }

    fn wait_completed(&self, ticket: SubmissionTicket, stage: &str) {
        // Native blocking fence wait runs the real queue callback without pumping RHI retirement.
        // There is only one producer and the just-presented frame is its most recent submission.
        self.native
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: Some(COMPLETION_TIMEOUT),
            })
            .unwrap_or_else(|error| {
                panic!("NATIVE_RESIZE_{stage}: native completion wait: {error}")
            });
        assert_eq!(
            self.owner.submission_status(ticket).unwrap(),
            SubmissionStatus::Completed
        );
        self.assert_healthy(stage);
    }

    fn assert_healthy(&self, stage: &str) {
        let fault = self.owner.first_fault();
        assert!(
            fault.is_none(),
            "NATIVE_RESIZE_{stage}: unexpected device fault: {fault:?}"
        );
    }

    fn reconfigure(
        &self,
        old: &SurfaceSessionReceipt,
        extent: (u32, u32),
        stage: &str,
    ) -> SurfaceSessionReceipt {
        let result = self
            .owner
            .reconfigure_surface_session(old.session(), &swapchain_at(old, extent));
        if let Some(fault) = self.owner.first_fault() {
            if stage == "WITNESS"
                && fault.kind == DeviceFaultKind::Validation
                && fault.message.contains("Surface::configure")
                && fault.message.contains("Invalid surface")
            {
                panic!("NATIVE_RESIZE_CONFIGURE_FAULT: {fault:?}; RHI returned {result:?}");
            }
            panic!("NATIVE_RESIZE_{stage}: unexpected reconfigure fault: {fault:?}; {result:?}");
        }
        let outcome =
            result.unwrap_or_else(|error| panic!("NATIVE_RESIZE_{stage}: reconfigure: {error}"));
        let resized = renderable(outcome, extent, stage);
        assert_ne!(resized.session(), old.session());
        assert_eq!(resized.session().device_id(), old.session().device_id());
        assert_eq!(resized.session().generation(), old.session().generation());
        assert!(matches!(
            self.owner.acquire_surface_frame(old.session()),
            Err(RhiError::SurfaceHandle(
                RenderSurfaceHandleError::StaleHandle { .. }
            ))
        ));
        resized
    }
}

fn renderable(
    outcome: SurfaceSessionCreateOutcome,
    extent: (u32, u32),
    stage: &str,
) -> SurfaceSessionReceipt {
    let SurfaceSessionCreateOutcome::Renderable(receipt) = outcome else {
        panic!("NATIVE_RESIZE_{stage}: expected a renderable session, got {outcome:?}");
    };
    assert_eq!((receipt.swapchain.width, receipt.swapchain.height), extent);
    receipt
}

fn swapchain_at(surface: &SurfaceSessionReceipt, extent: (u32, u32)) -> SwapchainDesc {
    SwapchainDesc {
        width: extent.0,
        height: extent.1,
        ..surface.swapchain.clone()
    }
}

// The system STATIC class needs no registration or test window procedure. WS_POPUP without
// WS_VISIBLE yields a hidden top-level window with a borderless client area.
struct HiddenWindow {
    hwnd: NonNull<c_void>,
    hinstance: NonNull<c_void>,
}

impl HiddenWindow {
    fn new(extent: (u32, u32)) -> Self {
        const STATIC_CLASS: [u16; 7] = [83, 84, 65, 84, 73, 67, 0]; // "STATIC\0"
        const WS_POPUP: u32 = 0x8000_0000;
        // SAFETY: null selects the current executable and does not take a module reference.
        let hinstance = NonNull::new(unsafe { GetModuleHandleW(ptr::null()) })
            .unwrap_or_else(|| window_failure("GetModuleHandleW"));
        let (width, height) = win_size(extent);
        // SAFETY: class is NUL-terminated, instance is live, and other optional inputs are null.
        let hwnd = unsafe {
            CreateWindowExW(
                0,
                STATIC_CLASS.as_ptr(),
                ptr::null(),
                WS_POPUP,
                0,
                0,
                width,
                height,
                ptr::null_mut(),
                ptr::null_mut(),
                hinstance.as_ptr(),
                ptr::null(),
            )
        };
        let hwnd = NonNull::new(hwnd).unwrap_or_else(|| window_failure("CreateWindowExW"));
        let window = Self { hwnd, hinstance };
        window.assert_hidden_extent(extent);
        window
    }

    fn target(&self) -> RenderNativeSurfaceTarget {
        RenderNativeSurfaceTarget::Win32 {
            hwnd: self.hwnd.as_ptr() as usize as u64,
            hinstance: Some(self.hinstance.as_ptr() as usize as u64),
        }
    }

    fn resize(&self, extent: (u32, u32)) {
        const SWP_NOMOVE: u32 = 0x0002;
        const SWP_NOZORDER: u32 = 0x0004;
        const SWP_NOACTIVATE: u32 = 0x0010;
        let (width, height) = win_size(extent);
        // SAFETY: this is the live, owner-thread HWND; flags ignore the position and z-order inputs.
        let ok = unsafe {
            SetWindowPos(
                self.hwnd.as_ptr(),
                ptr::null_mut(),
                0,
                0,
                width,
                height,
                SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
            )
        };
        if ok == 0 {
            window_failure("SetWindowPos");
        }
        self.assert_hidden_extent(extent);
    }

    fn assert_hidden_extent(&self, extent: (u32, u32)) {
        let mut rect = Rect::default();
        // SAFETY: the HWND is still owned here and `rect` is writable RECT storage.
        if unsafe { GetClientRect(self.hwnd.as_ptr(), &mut rect) } == 0 {
            window_failure("GetClientRect");
        }
        // SAFETY: IsWindowVisible only queries the same live HWND.
        assert_eq!(unsafe { IsWindowVisible(self.hwnd.as_ptr()) }, 0);
        assert_eq!(
            (rect.right - rect.left, rect.bottom - rect.top),
            win_size(extent)
        );
    }
}

impl Drop for HiddenWindow {
    fn drop(&mut self) {
        // SAFETY: NonNull keeps this owner on its creation thread; surface owners drop first.
        let ok = unsafe { DestroyWindow(self.hwnd.as_ptr()) };
        if ok == 0 && !std::thread::panicking() {
            window_failure("DestroyWindow");
        }
    }
}

fn win_size((width, height): (u32, u32)) -> (i32, i32) {
    (width.try_into().unwrap(), height.try_into().unwrap())
}

fn window_failure(operation: &str) -> ! {
    panic!(
        "NATIVE_RESIZE_WINDOW: {operation}: {}",
        io::Error::last_os_error()
    );
}

#[repr(C)]
#[derive(Default)]
struct Rect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

// Test-only imports match the user32/kernel32 Win32 ABI; no Cargo features are needed.
#[link(name = "user32")]
unsafe extern "system" {
    fn CreateWindowExW(
        ex_style: u32,
        class: *const u16,
        title: *const u16,
        style: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        parent: *mut c_void,
        menu: *mut c_void,
        instance: *mut c_void,
        param: *const c_void,
    ) -> *mut c_void;
    fn DestroyWindow(hwnd: *mut c_void) -> i32;
    fn GetClientRect(hwnd: *mut c_void, rect: *mut Rect) -> i32;
    fn IsWindowVisible(hwnd: *mut c_void) -> i32;
    fn SetWindowPos(
        hwnd: *mut c_void,
        after: *mut c_void,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        flags: u32,
    ) -> i32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetModuleHandleW(name: *const u16) -> *mut c_void;
}
