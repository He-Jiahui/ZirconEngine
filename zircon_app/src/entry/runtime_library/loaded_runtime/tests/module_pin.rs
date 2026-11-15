use super::{
    module_mapping_policy, plan_module_release_for, ModuleMappingPolicy, ProcessModuleReceipt,
    RuntimeImageLedger,
};

fn editor_exit_after_one_play(
    policy: ModuleMappingPolicy,
    pin_succeeded: bool,
) -> RuntimeImageLedger {
    let mut ledger = RuntimeImageLedger::new();
    ledger.load_and_seal(policy, pin_succeeded);
    ledger.load_and_seal(policy, pin_succeeded);
    ledger.release_session_handle();
    ledger.release_session_handle();
    ledger
}

#[test]
fn module_mapping_policy_matches_the_target_os() {
    if cfg!(windows) {
        assert_eq!(
            module_mapping_policy(),
            ModuleMappingPolicy::PinUntilProcessExit
        );
    } else {
        assert_eq!(
            module_mapping_policy(),
            ModuleMappingPolicy::ReleaseOnLastReference
        );
    }
}

#[test]
fn release_plan_pins_only_after_a_successful_windows_pin() {
    assert_eq!(
        plan_module_release_for(ModuleMappingPolicy::ReleaseOnLastReference, true),
        super::ModuleReleasePlan {
            retain_library_handle: true,
            process_receipt: ProcessModuleReceipt::ReleaseOnLastSession,
        }
    );
    assert_eq!(
        plan_module_release_for(ModuleMappingPolicy::PinUntilProcessExit, true).process_receipt,
        ProcessModuleReceipt::PinnedUntilProcessExit
    );
    assert!(
        !plan_module_release_for(ModuleMappingPolicy::PinUntilProcessExit, false)
            .retain_library_handle
    );
}

#[test]
fn play_stop_keeps_the_image_while_the_editor_session_holds_a_reference() {
    let mut ledger = RuntimeImageLedger::new();
    ledger.load_and_seal(ModuleMappingPolicy::ReleaseOnLastReference, false);
    ledger.load_and_seal(ModuleMappingPolicy::ReleaseOnLastReference, false);
    ledger.release_session_handle();
    assert!(ledger.image_mapped());
    ledger.release_session_handle();
    assert!(!ledger.image_mapped());
}

#[test]
fn windows_pin_keeps_the_image_mapped_after_play_and_editor_release() {
    let ledger = editor_exit_after_one_play(ModuleMappingPolicy::PinUntilProcessExit, true);
    assert!(ledger.image_mapped());
}

#[test]
fn failed_windows_pin_leaks_the_session_reference_instead_of_unmapping() {
    let ledger = editor_exit_after_one_play(ModuleMappingPolicy::PinUntilProcessExit, false);
    assert!(ledger.image_mapped());
    assert_eq!(ledger.leaked_handles, 2);
}

#[test]
fn non_windows_policy_unmaps_on_the_last_session_release() {
    let ledger = editor_exit_after_one_play(ModuleMappingPolicy::ReleaseOnLastReference, true);
    assert!(!ledger.image_mapped());
}

// This exercises the real Windows loader, not the runtime DLL, GPU teardown or PIE exit.
// A permanent pin must never affect the parent test process or its other tests.
#[cfg(windows)]
#[test]
fn windows_pin_retains_a_real_system_dll_after_library_drop() {
    const CHILD_ENV: &str = "ZIRCON_TEST_REAL_MODULE_PIN_CHILD";
    // A distinct exit code proves the child reached the actual loader assertions:
    // a misspelled --exact filter would run zero tests and exit 0, which must fail.
    const CHILD_VERIFIED_EXIT: i32 = 73;
    if let Some(value) = std::env::var_os(CHILD_ENV) {
        assert_eq!(
            value, "1",
            "the module-pin child flag is reserved for this test"
        );
        assert_real_system_dll_pin();
        std::process::exit(CHILD_VERIFIED_EXIT);
    }

    let full_name = concat!(
        module_path!(),
        "::windows_pin_retains_a_real_system_dll_after_library_drop"
    );
    let (_, test_name) = full_name
        .split_once("::")
        .expect("test path includes its crate");
    let mut child =
        std::process::Command::new(std::env::current_exe().expect("current test binary"))
            .args(["--exact", test_name, "--nocapture", "--test-threads=1"])
            .env(CHILD_ENV, "1")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .spawn()
            .expect("spawn the isolated Windows module-pin test");
    let pid = child.id();
    let timeout = std::time::Duration::from_secs(30);
    let deadline = std::time::Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                eprintln!("windows_module_pin_child pid={pid} exit={status}");
                assert_eq!(
                    status.code(),
                    Some(CHILD_VERIFIED_EXIT),
                    "child {pid} did not finish the real DLL pin assertions: {status}"
                );
                break;
            }
            Ok(None) if std::time::Instant::now() < deadline => {
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
            result => {
                // This handle belongs only to the child spawned above. Never search for
                // processes by name, and never turn a timeout into a skipped or passing test.
                let termination = child.kill();
                let exit = child.wait();
                panic!(
                    "module-pin child {pid} failed bounded wait ({timeout:?}): \
                     poll={result:?} termination={termination:?} actual_exit={exit:?}"
                );
            }
        }
    }
}

#[cfg(windows)]
fn assert_real_system_dll_pin() {
    use libloading::os::windows::{Library, LOAD_LIBRARY_SEARCH_SYSTEM32};
    use std::os::windows::ffi::OsStrExt;

    let directory = windows_system_directory();
    let candidates: [(&str, &[u8]); 3] = [
        ("version.dll", b"GetFileVersionInfoSizeW\0"),
        ("winhttp.dll", b"WinHttpCloseHandle\0"),
        ("wevtapi.dll", b"EvtClose\0"),
    ];
    let mut unavailable = Vec::new();
    for (name, export) in candidates {
        let module_name = name.encode_utf16().chain(Some(0)).collect::<Vec<_>>();
        // SAFETY: module_name is a live, NUL-terminated UTF-16 name. This query
        // does not load the DLL or add a reference that could keep it mapped.
        let before = unsafe { GetModuleHandleW(module_name.as_ptr()) };
        if !before.is_null() {
            unavailable.push(format!("{name}: already mapped at {before:p}"));
            continue;
        }
        let path = directory.join(name);
        let path_wide = path
            .as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect::<Vec<_>>();
        // SAFETY: this is a trusted Windows System32 DLL at an OS-provided absolute
        // path. Its dependencies are restricted to System32, never cwd or PATH.
        let library = match unsafe { Library::load_with_flags(&path, LOAD_LIBRARY_SEARCH_SYSTEM32) }
        {
            Ok(library) => library,
            Err(error) => {
                unavailable.push(format!("{}: {error}", path.display()));
                continue;
            }
        };
        // SAFETY: path_wide is a live, absolute, NUL-terminated UTF-16 path.
        let loaded = unsafe { GetModuleHandleW(path_wide.as_ptr()) };
        assert!(
            !loaded.is_null(),
            "loaded DLL has no module handle: {}",
            path.display()
        );
        // SAFETY: only the opaque export address is extracted while the Library
        // owns the module. No function pointer signature, call, or data dereference
        // is fabricated; the temporary Symbol is consumed before Library drops.
        let address = unsafe { library.get::<*const ()>(export) }
            .expect("the selected System32 DLL must expose its known export")
            .as_raw_ptr()
            .cast_const()
            .cast::<()>();
        assert!(
            !address.is_null(),
            "a successfully resolved export must have an address"
        );
        let mut export_owner = core::ptr::null_mut();
        // SAFETY: this queries a live symbol with FROM_ADDRESS | UNCHANGED_REFCOUNT.
        // It does not pin or acquire another reference. Some Windows versions forward
        // version.dll exports elsewhere; do not accidentally test an already-live DLL.
        let found_owner = unsafe {
            GetModuleHandleExW(0x0000_0004 | 0x0000_0002, address.cast(), &mut export_owner)
        };
        assert_ne!(
            found_owner,
            0,
            "query the actual export owner: {}",
            std::io::Error::last_os_error()
        );
        if export_owner != loaded {
            unavailable.push(format!("{name}: export forwarded to {export_owner:p}"));
            drop(library);
            continue;
        }

        assert!(
            super::pin_module_containing(address),
            "production PIN failed for {}: {}",
            path.display(),
            std::io::Error::last_os_error()
        );
        drop(library);
        // SAFETY: this is only a module lookup; it cannot remap or retain an image.
        let retained = unsafe { GetModuleHandleW(path_wide.as_ptr()) };
        assert_eq!(
            retained, loaded,
            "the pinned image must survive the last Library drop"
        );
        eprintln!(
            "windows_module_pin_verified path={} before={loaded:p} after={retained:p} ordinary_library_handles=0",
            path.display()
        );
        return;
    }
    panic!("no unused System32 DLL with a local export was available: {unavailable:?}");
}

#[cfg(windows)]
fn windows_system_directory() -> std::path::PathBuf {
    use std::os::windows::ffi::OsStringExt;

    const CAPACITY: u32 = 32_768;
    let mut buffer = vec![0_u16; CAPACITY as usize];
    // SAFETY: buffer provides CAPACITY writable UTF-16 code units.
    let length = unsafe { GetSystemDirectoryW(buffer.as_mut_ptr(), CAPACITY) };
    assert!(
        length > 0 && length < CAPACITY,
        "GetSystemDirectoryW failed or exceeded its bounded buffer: length={length}, error={}",
        std::io::Error::last_os_error()
    );
    let directory =
        std::path::PathBuf::from(std::ffi::OsString::from_wide(&buffer[..length as usize]));
    assert!(
        directory.is_absolute(),
        "System32 must be an absolute OS-provided path"
    );
    directory
}

#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {
    fn GetSystemDirectoryW(buffer: *mut u16, size: u32) -> u32;
    fn GetModuleHandleW(module_name: *const u16) -> *mut core::ffi::c_void;
    fn GetModuleHandleExW(
        flags: u32,
        module_name: *const u16,
        module: *mut *mut core::ffi::c_void,
    ) -> i32;
}
