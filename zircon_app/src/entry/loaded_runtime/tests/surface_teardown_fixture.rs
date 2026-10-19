//! Controlled ABI responses for real App/RuntimeSession teardown tests.
//! This provider creates no native window or runtime worker; physical tests can borrow the App's
//! real Window through read-only Weak observations. The actual session constructor validates the
//! table, receives a valid handle, and decodes/releases an owned composition receipt.

use std::{cell::RefCell, collections::HashMap, path::Path, ptr::NonNull, rc::Rc};

use zircon_runtime_interface::runtime_build_set::{
    ZrRuntimeDigestV1, ZrRuntimeModuleCompositionReceiptV1, ZrRuntimeModuleCompositionTargetV1,
    ZrRuntimeSessionProfileV1,
};
use zircon_runtime_interface::{
    ProfileControlCommand, ProfileControlRequest, ProfileControlResponse, ZrByteSlice, ZrHostApiV1,
    ZrOwnedResultV2, ZrRuntimeAllocationId, ZrRuntimeApiV8, ZrRuntimeBindViewportSurfaceRequestV1,
    ZrRuntimeEventV1, ZrRuntimeSessionConfigV3, ZrRuntimeSessionHandle, ZrRuntimeViewportHandle,
    ZrStatus, ZrStatusCode, ZIRCON_RUNTIME_ABI_VERSION_V1, ZR_RUNTIME_PROFILE_REQUEST_LIMIT_V1,
};

use super::{
    allocate_runtime_instance_id, validate_v8_api, LoadedRuntime, NEXT_RUNTIME_INSTANCE_ID,
};
use crate::entry::runtime_library::RuntimeSession;

#[path = "../../tests/surface_teardown_fixture/native_window_probe.rs"]
mod native_window_probe;
pub(in crate::entry) use native_window_probe::{
    current_native_thread_id, native_hwnd_is_live, NativeWindowObservation,
};

const SESSION: ZrRuntimeSessionHandle = ZrRuntimeSessionHandle::new(73);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::entry) enum Call {
    Create,
    Receipt,
    ReleaseAllocation,
    Bind,
    Unbind,
    Destroy,
    Event,
}

#[derive(Clone, Copy, Debug)]
pub(in crate::entry) struct Policy {
    pub unbind_failures: usize,
    pub destroy_failures: usize,
    pub unbind_export: bool,
    pub surface_v2_export: bool,
    pub surface_v2_bind_failures: usize,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            unbind_failures: 0,
            destroy_failures: 0,
            unbind_export: true,
            surface_v2_export: false,
            surface_v2_bind_failures: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub(in crate::entry) struct Snapshot {
    pub calls: Vec<Call>,
    pub active: bool,
    pub bound: bool,
    pub allocations: usize,
}

struct ProviderState {
    policy: Policy,
    calls: Vec<Call>,
    active: bool,
    bound: bool,
    next_allocation: u64,
    allocations: HashMap<u64, Box<[u8]>>,
    last_bound_target: Option<zircon_runtime_interface::ZrRuntimeNativeSurfaceTargetV1>,
    surface_v2_requests: Vec<zircon_runtime_interface::ZrRuntimeBindViewportSurfaceRequestV2>,
    native_window_probe: Option<native_window_probe::NativeWindowProbe>,
    native_window_observations: Vec<NativeWindowObservation>,
}

#[derive(Clone)]
pub(in crate::entry) struct ProviderControl(Rc<RefCell<ProviderState>>);

impl ProviderControl {
    pub fn snapshot(&self) -> Snapshot {
        let state = self.0.borrow();
        Snapshot {
            calls: state.calls.clone(),
            active: state.active,
            bound: state.bound,
            allocations: state.allocations.len(),
        }
    }

    pub fn surface_v2_requests(
        &self,
    ) -> Vec<zircon_runtime_interface::ZrRuntimeBindViewportSurfaceRequestV2> {
        self.0.borrow().surface_v2_requests.clone()
    }

    pub fn fail_next_surface_v2_bind(&self) {
        self.0.borrow_mut().policy.surface_v2_bind_failures = 1;
    }

    pub fn last_bound_target(
        &self,
    ) -> Option<zircon_runtime_interface::ZrRuntimeNativeSurfaceTargetV1> {
        self.0.borrow().last_bound_target
    }

    pub fn watch_native_window(
        &self,
        owner: std::sync::Weak<dyn winit::window::Window>,
    ) -> NativeWindowObservation {
        let mut state = self.0.borrow_mut();
        assert!(
            state.native_window_probe.is_none(),
            "only one App Window observation registration"
        );
        let target = state
            .last_bound_target
            .expect("the actual provider bind target");
        let probe = native_window_probe::NativeWindowProbe::new(owner, target);
        let initial = probe.observe(Call::Bind);
        state.native_window_probe = Some(probe);
        initial
    }

    pub fn native_window_observations(&self) -> Vec<NativeWindowObservation> {
        self.0.borrow().native_window_observations.clone()
    }

    pub fn allow_cleanup(&self) {
        let mut state = self.0.borrow_mut();
        state.policy.unbind_failures = 0;
        state.policy.destroy_failures = 0;
    }
}

thread_local! {
    static PROVIDERS: RefCell<Vec<ProviderControl>> = const { RefCell::new(Vec::new()) };
}

struct ProviderRegistration {
    slot: usize,
}

impl Drop for ProviderRegistration {
    fn drop(&mut self) {
        PROVIDERS.with(|providers| {
            let mut providers = providers.borrow_mut();
            assert_eq!(
                providers.len(),
                self.slot + 1,
                "provider scopes must unwind in order"
            );
            providers.pop();
        });
    }
}

struct ApiTableOwner {
    table: Box<ZrRuntimeApiV8>,
    control: ProviderControl,
}

impl Drop for ApiTableOwner {
    fn drop(&mut self) {
        if self.control.snapshot().active {
            eprintln!(
                "fatal fixture API lifetime failure: live session still borrows its owned table"
            );
            std::process::abort();
        }
    }
}

/// The closure must drop its session/App before returning; the original helper remains unchanged.
pub(in crate::entry) fn with_session(
    policy: Policy,
    run: impl FnOnce(RuntimeSession, &ProviderControl),
) {
    with_session_for_project(policy, None, run);
}

pub(in crate::entry) fn with_session_for_project(
    policy: Policy,
    project_root: Option<&Path>,
    run: impl FnOnce(RuntimeSession, &ProviderControl),
) {
    with_runtime(policy, |runtime, control| {
        let session = match RuntimeSession::create_with_profile_and_project(
            runtime,
            b"minimal",
            project_root,
            None,
            None,
            None,
        ) {
            Ok(session) => session,
            Err(mut error) => {
                if let Some(mut session) = error.take_session() {
                    session.destroy_for_host_resource_release();
                }
                // ApiTableOwner aborts if an unextractable live session would outlive its table.
                panic!("real fixture session creation failed: {error}");
            }
        };
        run(session, control);
    });
}

/// Retains the owned table and current-process library until every borrowed session is destroyed.
/// Two nested scopes use distinct monomorphized callbacks even when their raw handles are equal.
pub(in crate::entry) fn with_runtime(
    policy: Policy,
    run: impl FnOnce(LoadedRuntime, &ProviderControl),
) {
    let control = ProviderControl(Rc::new(RefCell::new(ProviderState {
        policy,
        calls: Vec::new(),
        active: false,
        bound: false,
        next_allocation: 1,
        allocations: HashMap::new(),
        last_bound_target: None,
        surface_v2_requests: Vec::new(),
        native_window_probe: None,
        native_window_observations: Vec::new(),
    })));
    let provider_slot = PROVIDERS.with(|providers| {
        let mut providers = providers.borrow_mut();
        assert!(
            providers.len() < 2,
            "only two live fixture providers are supported"
        );
        let slot = providers.len();
        providers.push(control.clone());
        slot
    });
    let _registration = ProviderRegistration {
        slot: provider_slot,
    };

    let host = ZrHostApiV1::empty(ZIRCON_RUNTIME_ABI_VERSION_V1);
    // SAFETY: host is a live ABI-V1 value; this linked getter returns a process-static V8 table.
    let template = unsafe { zircon_runtime::dynamic_api::zircon_runtime_get_api_v8(&host) };
    assert!(
        !template.is_null(),
        "the current runtime must expose its frozen V8 table"
    );
    // The table is copied; every slot used by these tests is replaced below. Unused slots retain
    // their genuine signatures and satisfy the loader's mandatory-slot validation.
    let mut table = ApiTableOwner {
        // SAFETY: the non-null pointer above is the linked runtime process-static, aligned V8 table.
        table: Box::new(unsafe { *template }),
        control: control.clone(),
    };
    match provider_slot {
        0 => install_provider_api::<0>(&mut table.table, policy),
        1 => install_provider_api::<1>(&mut table.table, policy),
        _ => unreachable!("provider registration is bounded to two owners"),
    }
    let api = NonNull::from(table.table.as_mut());
    // SAFETY: api points to our aligned, fully initialized Box<V8>, retained through the session.
    let validated =
        unsafe { validate_v8_api(api) }.expect("fixture must pass the actual ABI validator");
    let viewport_surface_binding_v2: Option<
        zircon_runtime_interface::ZrRuntimeBindViewportSurfaceFnV2,
    > = if policy.surface_v2_export {
        match provider_slot {
            0 => Some(bind_surface_v2::<0>),
            1 => Some(bind_surface_v2::<1>),
            _ => unreachable!("provider registration is bounded to two owners"),
        }
    } else {
        None
    };
    let runtime = LoadedRuntime {
        _library: libloading::os::windows::Library::this()
            .expect("pin the fixture callback module")
            .into(),
        _artifact_manifest: None,
        api,
        size_bytes: validated.size_bytes,
        required: validated.required,
        app_session_configuration: None,
        viewport_surface_binding_v2,
        runtime_instance: allocate_runtime_instance_id(&NEXT_RUNTIME_INSTANCE_ID)
            .expect("allocate a unique fixture loaded-owner identity"),
    };
    run(runtime, &control);
    let final_state = control.snapshot();
    assert!(
        !final_state.active,
        "the real session owner must destroy its handle"
    );
    assert_eq!(
        final_state.allocations, 0,
        "owned receipt output must be released"
    );
    drop(table);
}

fn install_provider_api<const SLOT: usize>(table: &mut ZrRuntimeApiV8, policy: Policy) {
    table.create_session = Some(create_session::<SLOT>);
    table.destroy_session = Some(destroy_session::<SLOT>);
    table.handle_event = Some(handle_event::<SLOT>);
    table.release_allocation = Some(release_allocation::<SLOT>);
    table.profile_control = Some(profile_control::<SLOT>);
    table.bind_viewport_surface = Some(bind_surface::<SLOT>);
    table.unbind_viewport_surface = policy.unbind_export.then_some(unbind_surface::<SLOT>);
}

fn with_provider<const SLOT: usize>(call: impl FnOnce(&mut ProviderState) -> ZrStatus) -> ZrStatus {
    PROVIDERS.with(|providers| match providers.borrow().get(SLOT) {
        Some(control) => call(&mut control.0.borrow_mut()),
        None => failure(b"fixture callback has no owning provider"),
    })
}

fn failure(message: &'static [u8]) -> ZrStatus {
    ZrStatus::new(ZrStatusCode::Error, ZrByteSlice::from_static(message))
}

fn live_session(state: &ProviderState, handle: ZrRuntimeSessionHandle) -> bool {
    handle == SESSION && state.active
}

unsafe extern "C" fn create_session<const SLOT: usize>(
    _config: ZrRuntimeSessionConfigV3,
    out: *mut ZrRuntimeSessionHandle,
) -> ZrStatus {
    with_provider::<SLOT>(|state| {
        state.calls.push(Call::Create);
        if out.is_null() || state.active {
            return failure(b"fixture create requires a new output handle");
        }
        // SAFETY: the real typed session caller supplies writable aligned output; null was rejected.
        unsafe { out.write(SESSION) };
        state.active = true;
        ZrStatus::ok()
    })
}

unsafe extern "C" fn destroy_session<const SLOT: usize>(
    handle: ZrRuntimeSessionHandle,
) -> ZrStatus {
    with_provider::<SLOT>(|state| {
        state.calls.push(Call::Destroy);
        if let Some(probe) = &state.native_window_probe {
            state
                .native_window_observations
                .push(probe.observe(Call::Destroy));
        }
        if !live_session(state, handle) {
            return failure(b"fixture destroy used an inactive session");
        }
        if state.policy.destroy_failures > 0 {
            state.policy.destroy_failures -= 1;
            return failure(b"controlled session destroy failure");
        }
        state.active = false;
        state.bound = false;
        ZrStatus::ok()
    })
}

unsafe extern "C" fn handle_event<const SLOT: usize>(
    handle: ZrRuntimeSessionHandle,
    _event: ZrRuntimeEventV1,
) -> ZrStatus {
    with_provider::<SLOT>(|state| {
        state.calls.push(Call::Event);
        if !live_session(state, handle) {
            return failure(b"fixture event used an inactive session");
        }
        ZrStatus::ok()
    })
}

unsafe extern "C" fn bind_surface<const SLOT: usize>(
    handle: ZrRuntimeSessionHandle,
    request: ZrRuntimeBindViewportSurfaceRequestV1,
) -> ZrStatus {
    with_provider::<SLOT>(|state| {
        state.calls.push(Call::Bind);
        if !live_session(state, handle) {
            return failure(b"fixture bind used an inactive session");
        }
        state.bound = true;
        state.last_bound_target = Some(request.target);
        ZrStatus::ok()
    })
}

unsafe extern "C" fn bind_surface_v2<const SLOT: usize>(
    handle: ZrRuntimeSessionHandle,
    request: zircon_runtime_interface::ZrRuntimeBindViewportSurfaceRequestV2,
) -> ZrStatus {
    with_provider::<SLOT>(|state| {
        state.calls.push(Call::Bind);
        state.surface_v2_requests.push(request);
        if !live_session(state, handle) || !request.is_valid() {
            return failure(b"fixture V2 bind used an invalid session or request");
        }
        if state.policy.surface_v2_bind_failures > 0 {
            state.policy.surface_v2_bind_failures -= 1;
            return failure(b"controlled V2 bind failure");
        }
        state.bound = true;
        state.last_bound_target = Some(request.target);
        ZrStatus::ok()
    })
}

unsafe extern "C" fn unbind_surface<const SLOT: usize>(
    handle: ZrRuntimeSessionHandle,
    _viewport: ZrRuntimeViewportHandle,
) -> ZrStatus {
    with_provider::<SLOT>(|state| {
        state.calls.push(Call::Unbind);
        if let Some(probe) = &state.native_window_probe {
            state
                .native_window_observations
                .push(probe.observe(Call::Unbind));
        }
        if !live_session(state, handle) || !state.bound {
            return failure(b"fixture unbind requires a live binding");
        }
        if state.policy.unbind_failures > 0 {
            state.policy.unbind_failures -= 1;
            return failure(b"controlled surface unbind failure");
        }
        state.bound = false;
        ZrStatus::ok()
    })
}

unsafe extern "C" fn profile_control<const SLOT: usize>(
    handle: ZrRuntimeSessionHandle,
    request: ZrByteSlice,
    out: *mut ZrOwnedResultV2,
) -> ZrStatus {
    with_provider::<SLOT>(|state| {
        state.calls.push(Call::Receipt);
        if !live_session(state, handle) || out.is_null() {
            return failure(b"fixture receipt requires a live session and output");
        }
        // SAFETY: the typed host retains its serialized request buffer throughout this callback.
        let bytes = match unsafe {
            request.checked_slice(ZR_RUNTIME_PROFILE_REQUEST_LIMIT_V1.max_encoded_bytes)
        } {
            Ok(bytes) => bytes,
            Err(_) => return failure(b"fixture receipt request is invalid"),
        };
        let request: ProfileControlRequest = match serde_json::from_slice(bytes) {
            Ok(request) => request,
            Err(_) => return failure(b"fixture receipt request JSON is invalid"),
        };
        if !matches!(
            request.command,
            ProfileControlCommand::RuntimeModuleCompositionReceipt
        ) {
            return failure(b"fixture only handles the real constructor receipt request");
        }
        let mut response = ProfileControlResponse::ok("controlled composition receipt");
        response.module_composition_receipt = Some(ZrRuntimeModuleCompositionReceiptV1::new(
            1,
            7,
            ZrRuntimeModuleCompositionTargetV1::ClientRuntime,
            None,
            ZrRuntimeSessionProfileV1::Minimal,
            ZrRuntimeDigestV1::parse(
                "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff",
            )
            .expect("valid fixed receipt digest"),
        ));
        let bytes = match serde_json::to_vec(&response) {
            Ok(bytes) => bytes.into_boxed_slice(),
            Err(_) => return failure(b"fixture receipt encoding failed"),
        };
        let allocation = ZrRuntimeAllocationId::new(state.next_allocation);
        state.next_allocation += 1;
        let output = ZrOwnedResultV2 {
            data: bytes.as_ptr(),
            len: bytes.len() as u64,
            allocation,
        };
        state.allocations.insert(allocation.raw(), bytes);
        // SAFETY: the typed caller owns valid output storage; payload bytes remain in allocations.
        unsafe { out.write(output) };
        ZrStatus::ok()
    })
}

unsafe extern "C" fn release_allocation<const SLOT: usize>(
    handle: ZrRuntimeSessionHandle,
    allocation: ZrRuntimeAllocationId,
) -> ZrStatus {
    with_provider::<SLOT>(|state| {
        state.calls.push(Call::ReleaseAllocation);
        if !live_session(state, handle) || state.allocations.remove(&allocation.raw()).is_none() {
            return failure(
                b"fixture allocation must be released by its live session exactly once",
            );
        }
        ZrStatus::ok()
    })
}
