//! Controlled ABI responses for real App/RuntimeSession teardown tests.
//! No native window or runtime worker is created; the actual session constructor validates the
//! table, receives a valid handle, and decodes/releases an owned composition receipt.

use std::{cell::RefCell, collections::HashMap, ptr::NonNull, rc::Rc};

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

use super::{validate_v8_api, LoadedRuntime};
use crate::entry::runtime_library::RuntimeSession;

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
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            unbind_failures: 0,
            destroy_failures: 0,
            unbind_export: true,
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

    pub fn allow_cleanup(&self) {
        let mut state = self.0.borrow_mut();
        state.policy.unbind_failures = 0;
        state.policy.destroy_failures = 0;
    }
}

thread_local! {
    static PROVIDER: RefCell<Option<ProviderControl>> = const { RefCell::new(None) };
}

struct ProviderRegistration;

impl Drop for ProviderRegistration {
    fn drop(&mut self) {
        PROVIDER.with(|slot| {
            slot.borrow_mut().take();
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

/// The closure must finish dropping its session/App before returning. The owned table remains
/// alive until then, and the current-process library pin covers every fixture callback.
pub(in crate::entry) fn with_session(
    policy: Policy,
    run: impl FnOnce(RuntimeSession, &ProviderControl),
) {
    let control = ProviderControl(Rc::new(RefCell::new(ProviderState {
        policy,
        calls: Vec::new(),
        active: false,
        bound: false,
        next_allocation: 1,
        allocations: HashMap::new(),
    })));
    PROVIDER.with(|slot| {
        assert!(
            slot.borrow().is_none(),
            "a fixture provider is already registered"
        );
        *slot.borrow_mut() = Some(control.clone());
    });
    let _registration = ProviderRegistration;

    let host = ZrHostApiV1::empty(ZIRCON_RUNTIME_ABI_VERSION_V1);
    let template = unsafe { zircon_runtime::dynamic_api::zircon_runtime_get_api_v8(&host) };
    assert!(
        !template.is_null(),
        "the current runtime must expose its frozen V8 table"
    );
    // The table is copied; every slot used by these tests is replaced below. Unused slots retain
    // their genuine signatures and satisfy the loader's mandatory-slot validation.
    let mut table = ApiTableOwner {
        table: Box::new(unsafe { *template }),
        control: control.clone(),
    };
    table.table.create_session = Some(create_session);
    table.table.destroy_session = Some(destroy_session);
    table.table.handle_event = Some(handle_event);
    table.table.release_allocation = Some(release_allocation);
    table.table.profile_control = Some(profile_control);
    table.table.bind_viewport_surface = Some(bind_surface);
    table.table.unbind_viewport_surface = policy.unbind_export.then_some(unbind_surface);
    let api = NonNull::from(table.table.as_mut());
    let validated =
        unsafe { validate_v8_api(api) }.expect("fixture must pass the actual ABI validator");
    let runtime = LoadedRuntime {
        _library: libloading::os::windows::Library::this()
            .expect("pin the fixture callback module")
            .into(),
        _artifact_manifest: None,
        api,
        size_bytes: validated.size_bytes,
        required: validated.required,
        app_session_configuration: None,
    };
    let session = match RuntimeSession::create_with_profile_and_project(
        runtime, b"minimal", None, None, None, None,
    ) {
        Ok(session) => session,
        Err(mut error) => {
            // The real constructor can retain its failed session in the shared startup owner.
            // Extract only this original packet and release it while its ABI table still lives.
            if let Some(mut session) = error.take_session() {
                session.destroy_for_host_resource_release();
            }
            // ApiTableOwner aborts if an unextractable live session would outlive its table.
            panic!("real fixture session creation failed: {error}");
        }
    };
    run(session, &control);
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

fn with_provider(call: impl FnOnce(&mut ProviderState) -> ZrStatus) -> ZrStatus {
    PROVIDER.with(|slot| match slot.borrow().as_ref() {
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

unsafe extern "C" fn create_session(
    _config: ZrRuntimeSessionConfigV3,
    out: *mut ZrRuntimeSessionHandle,
) -> ZrStatus {
    with_provider(|state| {
        state.calls.push(Call::Create);
        if out.is_null() || state.active {
            return failure(b"fixture create requires a new output handle");
        }
        unsafe { out.write(SESSION) };
        state.active = true;
        ZrStatus::ok()
    })
}

unsafe extern "C" fn destroy_session(handle: ZrRuntimeSessionHandle) -> ZrStatus {
    with_provider(|state| {
        state.calls.push(Call::Destroy);
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

unsafe extern "C" fn handle_event(
    handle: ZrRuntimeSessionHandle,
    _event: ZrRuntimeEventV1,
) -> ZrStatus {
    with_provider(|state| {
        state.calls.push(Call::Event);
        if !live_session(state, handle) {
            return failure(b"fixture event used an inactive session");
        }
        ZrStatus::ok()
    })
}

unsafe extern "C" fn bind_surface(
    handle: ZrRuntimeSessionHandle,
    _request: ZrRuntimeBindViewportSurfaceRequestV1,
) -> ZrStatus {
    with_provider(|state| {
        state.calls.push(Call::Bind);
        if !live_session(state, handle) {
            return failure(b"fixture bind used an inactive session");
        }
        state.bound = true;
        ZrStatus::ok()
    })
}

unsafe extern "C" fn unbind_surface(
    handle: ZrRuntimeSessionHandle,
    _viewport: ZrRuntimeViewportHandle,
) -> ZrStatus {
    with_provider(|state| {
        state.calls.push(Call::Unbind);
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

unsafe extern "C" fn profile_control(
    handle: ZrRuntimeSessionHandle,
    request: ZrByteSlice,
    out: *mut ZrOwnedResultV2,
) -> ZrStatus {
    with_provider(|state| {
        state.calls.push(Call::Receipt);
        if !live_session(state, handle) || out.is_null() {
            return failure(b"fixture receipt requires a live session and output");
        }
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
        unsafe { out.write(output) };
        ZrStatus::ok()
    })
}

unsafe extern "C" fn release_allocation(
    handle: ZrRuntimeSessionHandle,
    allocation: ZrRuntimeAllocationId,
) -> ZrStatus {
    with_provider(|state| {
        state.calls.push(Call::ReleaseAllocation);
        if !live_session(state, handle) || state.allocations.remove(&allocation.raw()).is_none() {
            return failure(
                b"fixture allocation must be released by its live session exactly once",
            );
        }
        ZrStatus::ok()
    })
}
