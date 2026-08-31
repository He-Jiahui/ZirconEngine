use std::path::PathBuf;
use std::time::Duration;

use super::library_path::{
    default_runtime_library_path, platform_runtime_library_name,
    runtime_library_path_for_executable,
};
use super::loaded_runtime::{
    runtime_api_field_available, runtime_api_required_layout_available,
    runtime_api_supports_viewport_surface_present, runtime_library_startup_error_for_request,
    validate_runtime_api_pointer, LoadedRuntime,
};
use zircon_runtime_interface::{
    ZrByteSlice, ZrOwnedResultV2, ZrRuntimeAllocationId, ZrRuntimeApiV8,
    ZrRuntimeBindViewportSurfaceRequestV1, ZrRuntimeEventV1, ZrRuntimeFrameRequestV1,
    ZrRuntimeFrameV2, ZrRuntimeHighlightSetV1, ZrRuntimeOperationHandle,
    ZrRuntimeOperationStatusV2, ZrRuntimePluginEventSubscriptionHandle, ZrRuntimeSessionConfigV3,
    ZrRuntimeSessionHandle, ZrRuntimeViewportHandle, ZrRuntimeViewportPickRequestV1,
    ZrRuntimeViewportPickResultV1, ZrRuntimeViewportPickTicket, ZrRuntimeViewportSizeV1, ZrStatus,
};

use super::runtime_session::{RuntimeFrameDemand, MAX_HOST_RUNTIME_FRAME_DELAY};
use zircon_runtime_interface::runtime_build_set::{
    ZrRuntimeDigestV1, ZrRuntimeModuleCompositionReceiptV1, ZrRuntimeModuleCompositionTargetV1,
    ZrRuntimeSessionProfileV1, ZR_RUNTIME_MODULE_COMPOSITION_RECEIPT_SCHEMA_V1,
};
use zircon_runtime_interface::ProfileControlResponse;

mod api_v8_layout;
mod editor_gateway;

#[test]
fn runtime_session_accepts_only_a_current_typed_module_composition_receipt() {
    use super::runtime_session::module_composition_receipt::{
        require_response, validate_requested_profile,
    };

    let receipt = ZrRuntimeModuleCompositionReceiptV1::new(
        1,
        7,
        ZrRuntimeModuleCompositionTargetV1::ClientRuntime,
        None,
        ZrRuntimeSessionProfileV1::Runtime,
        ZrRuntimeDigestV1::parse(
            "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff",
        )
        .unwrap(),
    );
    let mut response = ProfileControlResponse::ok("composition receipt");
    response.module_composition_receipt = Some(receipt.clone());

    assert_eq!(require_response(Some(response)).unwrap(), receipt);
    validate_requested_profile(&receipt, b"runtime").unwrap();

    let mut wrong_target = receipt.clone();
    wrong_target.target_mode = ZrRuntimeModuleCompositionTargetV1::EditorHost;
    assert!(validate_requested_profile(&wrong_target, b"runtime")
        .unwrap_err()
        .to_string()
        .contains("does not match requested runtime profile"));

    let mut wrong_session_profile = receipt.clone();
    wrong_session_profile.session_profile = ZrRuntimeSessionProfileV1::Editor;
    assert!(
        validate_requested_profile(&wrong_session_profile, b"runtime")
            .unwrap_err()
            .to_string()
            .contains("does not match requested runtime profile")
    );

    assert!(require_response(None)
        .unwrap_err()
        .to_string()
        .contains("does not provide required module composition receipt control"));

    assert!(
        require_response(Some(ProfileControlResponse::error("identity unavailable")))
            .unwrap_err()
            .to_string()
            .contains("runtime rejected module composition receipt request")
    );
    assert!(
        require_response(Some(ProfileControlResponse::ok("receipt omitted")))
            .unwrap_err()
            .to_string()
            .contains("runtime omitted the required module composition receipt")
    );

    let mut stale = receipt;
    stale.schema_version = ZR_RUNTIME_MODULE_COMPOSITION_RECEIPT_SCHEMA_V1 + 1;
    let mut response = ProfileControlResponse::ok("stale composition receipt");
    response.module_composition_receipt = Some(stale);
    assert!(require_response(Some(response))
        .unwrap_err()
        .to_string()
        .contains("unsupported module composition receipt schema"));
}

unsafe extern "C" fn fake_subscribe_plugin_event(
    _session: ZrRuntimeSessionHandle,
    _request: ZrByteSlice,
    _out_subscription: *mut ZrRuntimePluginEventSubscriptionHandle,
) -> ZrStatus {
    ZrStatus::ok()
}

unsafe extern "C" fn fake_unsubscribe_plugin_event(
    _session: ZrRuntimeSessionHandle,
    _subscription: ZrRuntimePluginEventSubscriptionHandle,
) -> ZrStatus {
    ZrStatus::ok()
}

unsafe extern "C" fn fake_drain_plugin_events(
    _session: ZrRuntimeSessionHandle,
    _subscription: ZrRuntimePluginEventSubscriptionHandle,
    _out_deliveries: *mut ZrOwnedResultV2,
) -> ZrStatus {
    ZrStatus::ok()
}

#[test]
fn runtime_library_path_uses_environment_override() {
    let previous = std::env::var_os("ZIRCON_RUNTIME_LIBRARY");
    let expected = std::env::temp_dir().join("custom-runtime-library");
    std::env::set_var("ZIRCON_RUNTIME_LIBRARY", &expected);

    let actual = default_runtime_library_path().unwrap();

    match previous {
        Some(previous) => std::env::set_var("ZIRCON_RUNTIME_LIBRARY", previous),
        None => std::env::remove_var("ZIRCON_RUNTIME_LIBRARY"),
    }
    assert!(matches!(
        actual,
        super::library_path::RuntimeLibraryPathSelection::EnvironmentOverride { .. }
    ));
    assert_eq!(actual.path(), expected);
}

#[test]
fn relative_runtime_library_load_failure_retains_the_environment_request() {
    let path = std::env::temp_dir()
        .join("zircon-runtime-library-product")
        .join("plugins")
        .join("missing-runtime-library.dll");
    let request = "ZIRCON_RUNTIME_LIBRARY=plugins/missing-runtime-library.dll";

    let error = match LoadedRuntime::load_for_request(&path, request.to_string()) {
        Ok(_) => panic!("a missing runtime library must retain its configuration request"),
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("requested_path=ZIRCON_RUNTIME_LIBRARY=plugins/missing-runtime-library.dll"),
        "runtime library failure must preserve the relative environment request: {error}"
    );
}

#[test]
fn runtime_preflight_rejects_a_missing_library_before_dynamic_loading() {
    let path = std::env::temp_dir()
        .join("zircon-runtime-library-product")
        .join("plugins")
        .join("missing-runtime-library.dll");
    let request = "ZIRCON_RUNTIME_LIBRARY=plugins/missing-runtime-library.dll";

    let error = LoadedRuntime::preflight_for_request(&path, request.to_string())
        .expect_err("runtime BuildSet preflight must fail before the library can load");

    assert!(
        error
            .to_string()
            .contains("requested_path=ZIRCON_RUNTIME_LIBRARY=plugins/missing-runtime-library.dll"),
        "runtime preflight failure must preserve the relative environment request: {error}"
    );
}

#[test]
fn platform_runtime_library_name_matches_target() {
    let name = platform_runtime_library_name();

    #[cfg(target_os = "windows")]
    assert_eq!(name, "zircon_runtime.dll");
    #[cfg(target_os = "macos")]
    assert_eq!(name, "libzircon_runtime.dylib");
    #[cfg(all(unix, not(target_os = "macos")))]
    assert_eq!(name, "libzircon_runtime.so");
}

#[test]
fn runtime_library_path_prefers_executable_sibling_when_present() {
    let temp = runtime_library_temp_dir("sibling");
    let bin_dir = temp.join("debug");
    std::fs::create_dir_all(bin_dir.join("deps")).unwrap();
    let executable = bin_dir.join("zircon_editor-test-exe");
    let sibling = bin_dir.join(platform_runtime_library_name());
    let deps = bin_dir.join("deps").join(platform_runtime_library_name());
    std::fs::write(&sibling, []).unwrap();
    std::fs::write(&deps, []).unwrap();

    let actual = runtime_library_path_for_executable(&executable).unwrap();

    remove_runtime_library_temp_dir(&temp);
    assert_eq!(actual, sibling);
}

#[test]
fn runtime_library_path_falls_back_to_cargo_deps_sibling() {
    let temp = runtime_library_temp_dir("deps");
    let bin_dir = temp.join("debug");
    std::fs::create_dir_all(bin_dir.join("deps")).unwrap();
    let executable = bin_dir.join("zircon_editor-test-exe");
    let deps = bin_dir.join("deps").join(platform_runtime_library_name());
    std::fs::write(&deps, []).unwrap();

    let actual = runtime_library_path_for_executable(&executable).unwrap();

    remove_runtime_library_temp_dir(&temp);
    assert_eq!(actual, deps);
}

#[cfg(any(unix, windows))]
#[test]
fn runtime_library_default_path_uses_the_physical_product_directory_identity() {
    let temp = runtime_library_temp_dir("default-physical-identity");
    let physical_product = temp.join("physical-product");
    std::fs::create_dir_all(&physical_product).unwrap();
    let product_alias = temp.join("product-alias");
    create_directory_link(&physical_product, &product_alias);
    let executable = product_alias.join("zircon_editor-test-exe");
    let physical_sibling = physical_product.join(platform_runtime_library_name());
    std::fs::write(&physical_sibling, []).unwrap();

    let actual = runtime_library_path_for_executable(&executable)
        .expect("the default runtime library path should resolve from the product directory");
    let expected =
        zircon_runtime::asset::project::ProjectPaths::resolve_existing_path(&physical_product)
            .unwrap()
            .join(platform_runtime_library_name());

    remove_runtime_library_temp_dir(&temp);
    assert_eq!(actual, expected);
}

#[test]
fn runtime_api_field_availability_rejects_truncated_or_overflowing_fields() {
    assert!(runtime_api_field_available(16, 8, 8));
    assert!(!runtime_api_field_available(15, 8, 8));
    assert!(!runtime_api_field_available(usize::MAX, usize::MAX, 1));
}

#[test]
fn default_runtime_library_resolution_uses_the_startup_diagnostic_contract() {
    let error = runtime_library_startup_error_for_request(
        "<runtime-library-default>",
        "failed to resolve current executable: access denied",
    );

    assert_eq!(
        error.to_string(),
        "runtime startup diagnostic: component=runtime_library requested_path=<runtime-library-default> cause=failed to resolve current executable: access denied recovery=stage the runtime library beside the product executable or set ZIRCON_RUNTIME_LIBRARY to a compatible path relative to the product executable or an absolute path"
    );
}

#[test]
fn runtime_api_required_layout_must_cover_world_sync_drain_field() {
    let required_size = core::mem::offset_of!(ZrRuntimeApiV8, drain_world_invalidations)
        + core::mem::size_of::<
            Option<zircon_runtime_interface::ZrRuntimeDrainWorldInvalidationsFnV2>,
        >();

    assert!(runtime_api_required_layout_available(required_size));
    assert!(!runtime_api_required_layout_available(required_size - 1));
}

#[test]
fn runtime_api_pointer_rejects_null_from_entry_symbol() {
    let error = unsafe { validate_runtime_api_pointer(core::ptr::null()) }
        .expect_err("null runtime API pointer should be rejected");

    assert_eq!(
        error.to_string(),
        "runtime library rejected host ABI version"
    );
}

#[test]
fn runtime_api_pointer_rejects_misaligned_table_before_reading_header() {
    let alignment = core::mem::align_of::<ZrRuntimeApiV8>();
    let storage = vec![0_u8; core::mem::size_of::<ZrRuntimeApiV8>() + alignment];
    let aligned_offset = (alignment - (storage.as_ptr() as usize % alignment)) % alignment;
    let misaligned = unsafe { storage.as_ptr().add(aligned_offset + 1) }.cast::<ZrRuntimeApiV8>();

    let error = unsafe { validate_runtime_api_pointer(misaligned) }
        .expect_err("misaligned runtime API table pointer should be rejected before dereference");

    assert_eq!(
        error.to_string(),
        format!("runtime API table pointer is not aligned to {alignment} bytes")
    );
}

#[test]
fn runtime_api_pointer_rejects_version_mismatch_before_session_creation() {
    let api = valid_runtime_api_table(zircon_runtime_interface::ZIRCON_RUNTIME_API_VERSION_V8 + 1);

    let error = unsafe { validate_runtime_api_pointer(&api) }
        .expect_err("unsupported runtime API table version should be rejected");

    assert_eq!(error.to_string(), "unsupported runtime API table version 9");
}

#[test]
fn runtime_api_pointer_names_every_missing_required_function() {
    macro_rules! assert_missing_required_function {
        ($($field:ident),+ $(,)?) => {
            $(
                let mut api = valid_runtime_api_table(
                    zircon_runtime_interface::ZIRCON_RUNTIME_API_VERSION_V8,
                );
                api.$field = None;

                let error = unsafe { validate_runtime_api_pointer(&api) }
                    .expect_err(concat!(stringify!($field), " must be required by V8"));

                assert_eq!(
                    error.to_string(),
                    concat!(
                        "runtime API table is missing required function `",
                        stringify!($field),
                        "`",
                    ),
                );
            )+
        };
    }

    assert_missing_required_function!(
        create_session,
        destroy_session,
        release_allocation,
        handle_event,
        capture_frame,
        submit_highlight_set,
        subscribe_plugin_event,
        unsubscribe_plugin_event,
        drain_plugin_events,
        submit_operation,
        poll_operation,
        harvest_operation,
        tick_frame,
        query_world,
        watch_world,
        unwatch_world,
        drain_world_invalidations,
        request_viewport_pick,
        poll_viewport_pick,
        cancel_viewport_pick,
    );
}

#[test]
fn runtime_session_does_not_recheck_required_v8_mirror_or_operation_capabilities() {
    let session_source = include_str!("runtime_session.rs");
    let operation_source = include_str!("runtime_session/operation.rs");

    for forbidden in [
        "let Some(subscribe) = self.runtime().subscribe_plugin_event()",
        "let Some(unsubscribe) = self.runtime().unsubscribe_plugin_event()",
        "let Some(drain) = self.runtime().drain_plugin_events()",
        "CapabilityMissing {\n                    capability: \"runtime.operation.submit\"",
        "CapabilityMissing {\n                    capability: \"runtime.operation.poll\"",
        "CapabilityMissing {\n                    capability: \"runtime.operation.harvest\"",
    ] {
        assert!(
            !session_source.contains(forbidden),
            "validated V8 required entry must not retain capability fallback `{forbidden}`"
        );
    }
    for forbidden in [
        "let Some(submit) = self.runtime().submit_operation()",
        "let Some(poll) = self.runtime().poll_operation()",
        "let Some(harvest) = self.runtime().harvest_operation()",
    ] {
        assert!(
            !operation_source.contains(forbidden),
            "validated V8 required entry must not retain capability fallback `{forbidden}`"
        );
    }
}

#[test]
fn runtime_library_loader_reports_missing_entry_symbol_source_path() {
    let source = include_str!("loaded_runtime.rs");

    assert!(source.contains(".get::<ZrRuntimeGetApiFnV8>(ZR_RUNTIME_GET_API_SYMBOL_V8)"));
    assert!(!source.contains("ZrRuntimeGetApiFnV2"));
    assert!(!source.contains("ZR_RUNTIME_GET_API_SYMBOL_V2"));
    assert!(source.contains("failed to resolve zircon runtime API V8 symbol"));
}

#[test]
fn runtime_library_hard_cuts_to_v7_allocation_contract() {
    let loader = include_str!("loaded_runtime.rs");
    let session = include_str!("runtime_session.rs");
    let frame_demand = include_str!("runtime_session/frame_demand.rs");
    let runner = include_str!("../entry_runner/runtime.rs");

    assert!(loader.contains("ZrRuntimeGetApiFnV8"));
    assert!(loader.contains("ZR_RUNTIME_GET_API_SYMBOL_V8"));
    assert!(loader.contains("ZrRuntimeApiV8"));
    assert!(!loader.contains("ZrRuntimeApiV6"));
    assert!(!loader.contains("ZrRuntimeApiV2"));
    assert!(!loader.contains("ZR_RUNTIME_GET_API_SYMBOL_V2"));
    assert!(!loader.contains("ZrRuntimeGetApiFnV2"));

    assert!(session.contains("ZrRuntimeSessionConfigV3"));
    assert!(session.contains("wake_sink:"));
    assert!(frame_demand.contains("ZrRuntimeFrameDemandV1"));
    assert!(frame_demand.contains("RuntimeFrameDemand::try_from"));
    assert!(frame_demand.contains("unsupported runtime frame demand kind"));
    assert!(!session.contains("ZrRuntimeSessionConfigV1"));

    let event_loop = runner.find("EventLoop::new().map_err").unwrap();
    let proxy = runner.find("create_proxy()").unwrap();
    let session_create = runner
        .find("RuntimeSession::create_with_profile_and_project")
        .unwrap();
    assert!(event_loop < proxy && proxy < session_create);
}

#[test]
fn runtime_frame_demand_checked_conversion_rejects_unknowns_and_clamps_delay() {
    use zircon_runtime_interface::{
        ZrRuntimeFrameDemandV1, ZIRCON_RUNTIME_ABI_VERSION_V1, ZR_RUNTIME_FRAME_DEMAND_AFTER_V1,
    };

    assert_eq!(
        RuntimeFrameDemand::try_from(ZrRuntimeFrameDemandV1::idle()).unwrap(),
        RuntimeFrameDemand::Idle
    );
    assert_eq!(
        RuntimeFrameDemand::try_from(ZrRuntimeFrameDemandV1::immediate()).unwrap(),
        RuntimeFrameDemand::Immediate
    );
    assert_eq!(
        RuntimeFrameDemand::try_from(ZrRuntimeFrameDemandV1::after(u64::MAX)).unwrap(),
        RuntimeFrameDemand::After(MAX_HOST_RUNTIME_FRAME_DELAY)
    );

    let unknown = ZrRuntimeFrameDemandV1 {
        abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
        kind: 99,
        delay_nanoseconds: 0,
    };
    assert!(RuntimeFrameDemand::try_from(unknown)
        .unwrap_err()
        .to_string()
        .contains("unsupported runtime frame demand kind 99"));

    let malformed = ZrRuntimeFrameDemandV1 {
        abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
        kind: ZR_RUNTIME_FRAME_DEMAND_AFTER_V1 - 1,
        delay_nanoseconds: Duration::from_millis(1).as_nanos() as u64,
    };
    assert!(RuntimeFrameDemand::try_from(malformed)
        .unwrap_err()
        .to_string()
        .contains("requires zero delay"));
}

#[test]
fn runtime_session_destroys_before_releasing_host_wake_token() {
    let session = include_str!("runtime_session.rs");
    let try_destroy_body = session
        .split("pub(in crate::entry) fn try_destroy")
        .nth(1)
        .and_then(|body| body.split("fn runtime(&self)").next())
        .expect("RuntimeSession should expose retryable explicit teardown");
    let drop_body = session
        .split("impl Drop for RuntimeSession")
        .nth(1)
        .expect("RuntimeSession should own session teardown");
    let destroy = try_destroy_body
        .find("destroy_session(self.handle)")
        .unwrap();
    let unregister = try_destroy_body
        .find("wake_registration.unregister()")
        .unwrap();

    assert!(destroy < unregister);
    assert!(
        try_destroy_body.contains("ensure_status(destroy_status, \"destroy runtime session\")?")
    );
    assert!(try_destroy_body.contains("self.handle = ZrRuntimeSessionHandle::invalid();"));
    assert!(drop_body.contains("self.try_destroy()"));
    assert!(drop_body.contains("self.teardown_failure_state.record(error)"));
    assert!(drop_body.contains("abort_after_runtime_session_teardown_failure(&detail);"));
    assert!(!drop_body.contains("std::mem::forget("));
}

#[test]
fn runtime_library_loader_reports_missing_entry_symbol_from_dynamic_library() {
    let result = LoadedRuntime::load(missing_entry_symbol_fixture_library());
    let error = match result {
        Ok(_) => panic!("system library should not export the zircon runtime API symbol"),
        Err(error) => error,
    };
    let message = error.to_string();

    assert!(
        message.starts_with("runtime startup diagnostic: component=runtime_library "),
        "{message}"
    );
    assert!(
        message.contains(&format!(
            "requested_path={}",
            missing_entry_symbol_fixture_library()
        )),
        "{message}"
    );
    assert!(
        message.contains("failed to resolve zircon runtime API V8 symbol"),
        "{message}"
    );
    assert!(
        message.contains("recovery=stage the runtime library"),
        "{message}"
    );
}

#[test]
fn runtime_library_loader_reports_missing_library_with_recovery() {
    let missing = runtime_library_temp_dir("missing-library").join(platform_runtime_library_name());
    let error = match LoadedRuntime::load(&missing) {
        Ok(_) => panic!("a missing runtime library must not load"),
        Err(error) => error,
    };
    let message = error.to_string();

    assert!(
        message.starts_with("runtime startup diagnostic: component=runtime_library "),
        "{message}"
    );
    assert!(
        message.contains(&format!("requested_path={}", missing.display())),
        "{message}"
    );
    assert!(message.contains("cause="), "{message}");
    assert!(message.contains("recovery="), "{message}");
}

#[test]
fn runtime_session_create_reports_first_call_failure_context() {
    let source = include_str!("runtime_session.rs");
    let create_start = source
        .find("fn create_with_profile_and_project")
        .expect("runtime session create path");
    let create_body = &source[create_start..];

    assert!(create_body.contains("create_session("));
    assert!(create_body.contains("ensure_status(status, \"create runtime session\")?;"));
}

#[test]
fn runtime_surface_present_support_requires_all_optional_fields_in_size() {
    let full_size = core::mem::size_of::<ZrRuntimeApiV8>();
    let before_bind = core::mem::offset_of!(ZrRuntimeApiV8, bind_viewport_surface);
    let before_unbind = core::mem::offset_of!(ZrRuntimeApiV8, unbind_viewport_surface);
    let before_present = core::mem::offset_of!(ZrRuntimeApiV8, present_viewport);
    let bind = Some(fake_bind_viewport_surface as _);
    let unbind = Some(fake_unbind_viewport_surface as _);
    let present = Some(fake_present_viewport as _);

    assert!(runtime_api_supports_viewport_surface_present(
        full_size, bind, unbind, present
    ));
    assert!(!runtime_api_supports_viewport_surface_present(
        before_bind,
        bind,
        unbind,
        present
    ));
    assert!(!runtime_api_supports_viewport_surface_present(
        before_unbind,
        bind,
        unbind,
        present
    ));
    assert!(!runtime_api_supports_viewport_surface_present(
        before_present,
        bind,
        unbind,
        present
    ));
    assert!(!runtime_api_supports_viewport_surface_present(
        full_size, bind, None, present
    ));
}

#[test]
#[cfg_attr(
    not(feature = "backend-zr-vm"),
    ignore = "requires backend-zr-vm, ZIRCON_RUNTIME_LIBRARY, and ZR_VM_RUST_BINDING_LIB_DIR"
)]
fn runtime_library_project_capture_frame_draws_vampire_hud() {
    let runtime = LoadedRuntime::load_default().unwrap();
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .to_path_buf();
    let project_root = workspace_root.join("examples").join("vampire");
    let session = super::runtime_session::RuntimeSession::create_with_profile_and_project(
        runtime,
        b"runtime",
        Some(project_root.as_path()),
        None,
        None,
        None,
    )
    .unwrap();

    for _ in 0..3 {
        session.tick_frame().unwrap();
    }

    let frame = session
        .capture_frame(
            ZrRuntimeViewportHandle::new(1),
            ZrRuntimeViewportSizeV1::new(640, 360),
        )
        .unwrap();
    let hud_panel_pixels =
        count_vampire_hud_panel_pixels(frame.rgba(), frame.width(), frame.height());

    assert!(
        hud_panel_pixels > 48,
        "runtime library capture should include the vampire HUD panel, found {hud_panel_pixels}"
    );
}

fn runtime_library_temp_dir(case_name: &str) -> PathBuf {
    let temp = std::env::temp_dir().join(format!(
        "zircon-runtime-library-path-{}-{case_name}",
        std::process::id()
    ));
    remove_runtime_library_temp_dir(&temp);
    temp
}

fn count_vampire_hud_panel_pixels(rgba: &[u8], width: u32, height: u32) -> usize {
    let width = width as usize;
    let height = height as usize;
    let y_start = 16usize.min(height);
    let y_end = 80usize.min(height);
    let x_start = 16usize.min(width);
    let x_end = 260usize.min(width);
    let mut count = 0;
    for y in y_start..y_end {
        for x in x_start..x_end {
            let index = (y * width + x) * 4;
            let Some(pixel) = rgba.get(index..index + 4) else {
                continue;
            };
            if pixel[0] <= 48 && pixel[1] <= 58 && pixel[2] <= 76 && pixel[3] >= 180 {
                count += 1;
            }
        }
    }
    count
}

fn remove_runtime_library_temp_dir(path: &std::path::Path) {
    if path.exists() {
        std::fs::remove_dir_all(path).unwrap();
    }
}

#[cfg(unix)]
fn create_directory_link(target: &std::path::Path, link: &std::path::Path) {
    std::os::unix::fs::symlink(target, link).expect("create runtime-library product alias fixture");
}

#[cfg(windows)]
fn create_directory_link(target: &std::path::Path, link: &std::path::Path) {
    let command = format!(r#"mklink /J "{}" "{}""#, link.display(), target.display());
    let output = std::process::Command::new("cmd")
        .args(["/D", "/S", "/C"])
        .arg(command)
        .output()
        .expect("start mklink for runtime-library product alias fixture");
    assert!(
        output.status.success(),
        "create runtime-library product junction fixture failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn missing_entry_symbol_fixture_library() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "kernel32.dll"
    }
    #[cfg(target_os = "macos")]
    {
        "/usr/lib/libSystem.B.dylib"
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        "libc.so.6"
    }
}

fn valid_runtime_api_table(api_version: u32) -> ZrRuntimeApiV8 {
    let mut api = ZrRuntimeApiV8::empty();
    api.abi_version = api_version;
    api.size_bytes = core::mem::size_of::<ZrRuntimeApiV8>();
    api.create_session = Some(fake_create_session);
    api.destroy_session = Some(fake_destroy_session);
    api.release_allocation = Some(fake_release_allocation);
    api.handle_event = Some(fake_handle_event);
    api.capture_frame = Some(fake_capture_frame);
    api.submit_highlight_set = Some(fake_submit_highlight_set);
    api.subscribe_plugin_event = Some(fake_subscribe_plugin_event);
    api.unsubscribe_plugin_event = Some(fake_unsubscribe_plugin_event);
    api.drain_plugin_events = Some(fake_drain_plugin_events);
    api.submit_operation = Some(fake_submit_operation);
    api.poll_operation = Some(fake_poll_operation);
    api.harvest_operation = Some(fake_harvest_operation);
    api.tick_frame = Some(fake_tick_frame);
    api.query_world = Some(fake_query_world);
    api.watch_world = Some(fake_watch_world);
    api.unwatch_world = Some(fake_unwatch_world);
    api.drain_world_invalidations = Some(fake_drain_world_invalidations);
    api.request_viewport_pick = Some(fake_request_viewport_pick);
    api.poll_viewport_pick = Some(fake_poll_viewport_pick);
    api.cancel_viewport_pick = Some(fake_cancel_viewport_pick);
    api
}

unsafe extern "C" fn fake_request_viewport_pick(
    _session: ZrRuntimeSessionHandle,
    _request: ZrRuntimeViewportPickRequestV1,
    _out_ticket: *mut ZrRuntimeViewportPickTicket,
) -> ZrStatus {
    ZrStatus::ok()
}

unsafe extern "C" fn fake_poll_viewport_pick(
    _session: ZrRuntimeSessionHandle,
    _ticket: ZrRuntimeViewportPickTicket,
    _out_result: *mut ZrRuntimeViewportPickResultV1,
) -> ZrStatus {
    ZrStatus::ok()
}

unsafe extern "C" fn fake_cancel_viewport_pick(
    _session: ZrRuntimeSessionHandle,
    _ticket: ZrRuntimeViewportPickTicket,
) -> ZrStatus {
    ZrStatus::ok()
}

unsafe extern "C" fn fake_query_world(
    _session: ZrRuntimeSessionHandle,
    _request: ZrByteSlice,
    _out_result: *mut ZrOwnedResultV2,
) -> ZrStatus {
    ZrStatus::ok()
}

unsafe extern "C" fn fake_watch_world(
    _session: ZrRuntimeSessionHandle,
    _registration: ZrByteSlice,
    out_token: *mut zircon_runtime_interface::world_sync::WatchToken,
) -> ZrStatus {
    unsafe { out_token.write(zircon_runtime_interface::world_sync::WatchToken::new(1)) };
    ZrStatus::ok()
}

unsafe extern "C" fn fake_unwatch_world(
    _session: ZrRuntimeSessionHandle,
    _token: zircon_runtime_interface::world_sync::WatchToken,
    out_removed: *mut u8,
) -> ZrStatus {
    unsafe { out_removed.write(1) };
    ZrStatus::ok()
}

unsafe extern "C" fn fake_drain_world_invalidations(
    _session: ZrRuntimeSessionHandle,
    _out_batches: *mut ZrOwnedResultV2,
) -> ZrStatus {
    ZrStatus::ok()
}

unsafe extern "C" fn fake_submit_highlight_set(
    _session: ZrRuntimeSessionHandle,
    _request: ZrRuntimeHighlightSetV1,
) -> ZrStatus {
    ZrStatus::ok()
}

unsafe extern "C" fn fake_submit_operation(
    _session: ZrRuntimeSessionHandle,
    _request: ZrByteSlice,
    _out_operation: *mut ZrRuntimeOperationHandle,
) -> ZrStatus {
    ZrStatus::ok()
}

unsafe extern "C" fn fake_poll_operation(
    _session: ZrRuntimeSessionHandle,
    _operation: ZrRuntimeOperationHandle,
    out_status: *mut ZrRuntimeOperationStatusV2,
) -> ZrStatus {
    unsafe {
        out_status.write(ZrRuntimeOperationStatusV2::new(
            ZrRuntimeOperationHandle::new(1),
            zircon_runtime_interface::ZrRuntimeOperationPhase::Queued,
            0,
            1,
            zircon_runtime_interface::ZrRuntimeOperationDetailKindV2::None,
            0,
        ));
    }
    ZrStatus::ok()
}

unsafe extern "C" fn fake_harvest_operation(
    _session: ZrRuntimeSessionHandle,
    _operation: ZrRuntimeOperationHandle,
    _out_result: *mut ZrOwnedResultV2,
) -> ZrStatus {
    ZrStatus::ok()
}

unsafe extern "C" fn fake_create_session(
    _config: ZrRuntimeSessionConfigV3,
    _out_session: *mut ZrRuntimeSessionHandle,
) -> ZrStatus {
    ZrStatus::ok()
}

unsafe extern "C" fn fake_destroy_session(_session: ZrRuntimeSessionHandle) -> ZrStatus {
    ZrStatus::ok()
}

unsafe extern "C" fn fake_release_allocation(
    _session: ZrRuntimeSessionHandle,
    _allocation: ZrRuntimeAllocationId,
) -> ZrStatus {
    ZrStatus::ok()
}

unsafe extern "C" fn fake_handle_event(
    _session: ZrRuntimeSessionHandle,
    _event: ZrRuntimeEventV1,
) -> ZrStatus {
    ZrStatus::ok()
}

unsafe extern "C" fn fake_capture_frame(
    _session: ZrRuntimeSessionHandle,
    _request: ZrRuntimeFrameRequestV1,
    _out_frame: *mut ZrRuntimeFrameV2,
) -> ZrStatus {
    ZrStatus::ok()
}

unsafe extern "C" fn fake_bind_viewport_surface(
    _session: ZrRuntimeSessionHandle,
    _request: ZrRuntimeBindViewportSurfaceRequestV1,
) -> ZrStatus {
    ZrStatus::ok()
}

unsafe extern "C" fn fake_unbind_viewport_surface(
    _session: ZrRuntimeSessionHandle,
    _viewport: ZrRuntimeViewportHandle,
) -> ZrStatus {
    ZrStatus::ok()
}

unsafe extern "C" fn fake_present_viewport(
    _session: ZrRuntimeSessionHandle,
    _request: ZrRuntimeFrameRequestV1,
) -> ZrStatus {
    ZrStatus::ok()
}

unsafe extern "C" fn fake_profile_control(
    _session: ZrRuntimeSessionHandle,
    _request_json: ZrByteSlice,
    _out_json: *mut ZrOwnedResultV2,
) -> ZrStatus {
    ZrStatus::ok()
}

unsafe extern "C" fn fake_tick_frame(
    _session: ZrRuntimeSessionHandle,
    out_demand: *mut zircon_runtime_interface::ZrRuntimeFrameDemandV1,
) -> ZrStatus {
    if !out_demand.is_null() {
        unsafe { *out_demand = zircon_runtime_interface::ZrRuntimeFrameDemandV1::idle() };
    }
    ZrStatus::ok()
}

unsafe extern "C" fn fake_drain_host_requests(
    _session: ZrRuntimeSessionHandle,
    _out_requests: *mut ZrOwnedResultV2,
) -> ZrStatus {
    ZrStatus::ok()
}
