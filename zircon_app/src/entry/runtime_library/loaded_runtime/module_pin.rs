//! Keep a dynamically loaded runtime image mapped across `FreeLibrary`.
//!
//! wgpu-hal 29.0.4 (`wgpu-hal/src/auxil/dxgi/exception.rs`, still the same on
//! trunk when checked on 2026-11-10) registers a vectored exception handler
//! when a DX12 device is created with `InstanceFlags::VALIDATION`:
//!
//! ```text
//! AddVectoredExceptionHandler(0, Some(output_debug_string_handler));
//! ```
//!
//! The returned `PVOID` handle is discarded. Unregister passes the handler
//! function pointer instead:
//!
//! ```text
//! RemoveVectoredExceptionHandler(output_debug_string_handler as *mut _);
//! ```
//!
//! Win32 `RemoveVectoredExceptionHandler` only accepts the handle from
//! `AddVectoredExceptionHandler` and returns 0 when that handle is not found.
//! wgpu-hal ignores the return value, so the handler stays in the process
//! list. `FreeLibrary` then leaves that entry pointing at
//! `zircon_runtime.dll_unloaded`. The next exception (including the access
//! violation from the unmapped handler itself) re-enters it until the main
//! thread overflows. Debug builds start from
//! `InstanceFlags::from_build_config()`, which includes `VALIDATION`, so the
//! handler is registered unless `WGPU_VALIDATION=0`.
//!
//! Required upstream change: store the `AddVectoredExceptionHandler` handle
//! and pass that handle to `RemoveVectoredExceptionHandler`, checking the
//! return value. A wgpu bump does not fix this while trunk still passes the
//! function pointer. This crate cannot remove the handler from outside
//! wgpu-hal because the handle was never saved.
//!
//! Windows mitigation: `GetModuleHandleExW` with `GET_MODULE_HANDLE_EX_FLAG_PIN`
//! (`0x00000001`) and `GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS` (`0x00000004`)
//! keeps the module mapped until the process is terminated, including after
//! this session's `FreeLibrary`. That covers the observed crash window, where
//! Rust still prints `thread 'main' has overflowed its stack` after
//! `EntryRunner` returns. It does not prove the faulting instruction was the
//! VEH, and it does not keep the image mapped after the loader unmaps modules
//! during process termination. A leaked `Arc<Device>` that never reaches
//! unregister is a separate, still unproven cause; the pin also keeps that
//! image mapped for the same window.
//!
//! `target-editor-host` does not enable `editor-scripting`, so Play calls
//! `load_after_preflight` and `LoadLibrary` again. The editor session already
//! holds one reference, and the pin is per module, so the intermediate
//! `FreeLibrary` is not what unmaps the image. The last successful session
//! drop is the unmap this seal exists to prevent.
//!
//! If the pin call fails, the session `Library` is intentionally leaked so
//! that drop does not `FreeLibrary` the last reference.

use std::sync::atomic::{AtomicBool, Ordering};

use libloading::Library;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ModuleMappingPolicy {
    ReleaseOnLastReference,
    PinUntilProcessExit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProcessModuleReceipt {
    ReleaseOnLastSession,
    PinnedUntilProcessExit,
    PinFailedReferenceLeaked,
}

impl ProcessModuleReceipt {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::ReleaseOnLastSession => "release_on_last_session",
            Self::PinnedUntilProcessExit => "pinned_until_process_exit",
            Self::PinFailedReferenceLeaked => "pin_failed_reference_leaked",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ModuleReleasePlan {
    pub(crate) retain_library_handle: bool,
    pub(crate) process_receipt: ProcessModuleReceipt,
}

pub(crate) const fn module_mapping_policy() -> ModuleMappingPolicy {
    if cfg!(windows) {
        ModuleMappingPolicy::PinUntilProcessExit
    } else {
        ModuleMappingPolicy::ReleaseOnLastReference
    }
}

pub(crate) const fn plan_module_release_for(
    policy: ModuleMappingPolicy,
    pin_succeeded: bool,
) -> ModuleReleasePlan {
    match policy {
        ModuleMappingPolicy::ReleaseOnLastReference => ModuleReleasePlan {
            retain_library_handle: true,
            process_receipt: ProcessModuleReceipt::ReleaseOnLastSession,
        },
        ModuleMappingPolicy::PinUntilProcessExit => {
            if pin_succeeded {
                ModuleReleasePlan {
                    retain_library_handle: true,
                    process_receipt: ProcessModuleReceipt::PinnedUntilProcessExit,
                }
            } else {
                ModuleReleasePlan {
                    retain_library_handle: false,
                    process_receipt: ProcessModuleReceipt::PinFailedReferenceLeaked,
                }
            }
        }
    }
}

pub(crate) fn plan_module_release(pin_succeeded: bool) -> ModuleReleasePlan {
    plan_module_release_for(module_mapping_policy(), pin_succeeded)
}

static MODULE_PINNED: AtomicBool = AtomicBool::new(false);
static MODULE_PIN_LEAKED: AtomicBool = AtomicBool::new(false);
static MODULE_RELEASE_ON_LAST_SESSION: AtomicBool = AtomicBool::new(false);

fn record_process_receipt(receipt: ProcessModuleReceipt) {
    match receipt {
        ProcessModuleReceipt::ReleaseOnLastSession => {
            MODULE_RELEASE_ON_LAST_SESSION.store(true, Ordering::Relaxed);
        }
        ProcessModuleReceipt::PinnedUntilProcessExit => {
            MODULE_PINNED.store(true, Ordering::Relaxed);
        }
        ProcessModuleReceipt::PinFailedReferenceLeaked => {
            MODULE_PIN_LEAKED.store(true, Ordering::Relaxed);
        }
    }
}

/// Process-wide observation of the dynamic runtime image.
///
/// `not_loaded` means this process has not sealed a dynamic runtime library.
/// A later editor teardown line can still be pending: it is written after
/// `EntryRunner` returns and before the OS exit code is projected.
pub fn runtime_module_unload_receipt() -> &'static str {
    if MODULE_PINNED.load(Ordering::Relaxed) {
        ProcessModuleReceipt::PinnedUntilProcessExit.label()
    } else if MODULE_PIN_LEAKED.load(Ordering::Relaxed) {
        ProcessModuleReceipt::PinFailedReferenceLeaked.label()
    } else if MODULE_RELEASE_ON_LAST_SESSION.load(Ordering::Relaxed) {
        ProcessModuleReceipt::ReleaseOnLastSession.label()
    } else {
        "not_loaded"
    }
}

pub(super) struct SealedDynamicLibrary {
    library: Option<Library>,
    receipt: &'static str,
}

impl SealedDynamicLibrary {
    pub(super) fn seal(library: Library, code_address: *const ()) -> Self {
        #[cfg(windows)]
        let pin_succeeded = pin_module_containing(code_address);
        #[cfg(not(windows))]
        let pin_succeeded = {
            let _ = code_address;
            false
        };
        let plan = plan_module_release(pin_succeeded);
        record_process_receipt(plan.process_receipt);
        if plan.retain_library_handle {
            return Self {
                library: Some(library),
                receipt: plan.process_receipt.label(),
            };
        }
        write_log(
            "runtime_library",
            "runtime_module_pin_failed action=leak_library_reference",
        );
        // PIN failed. Dropping `library` would `FreeLibrary` this reference.
        // Leaking it keeps the image mapped when this is the last reference.
        std::mem::forget(library);
        Self {
            library: None,
            receipt: plan.process_receipt.label(),
        }
    }

    pub(super) fn receipt_label(&self) -> &'static str {
        self.receipt
    }
}

#[cfg(windows)]
fn pin_module_containing(code_address: *const ()) -> bool {
    const GET_MODULE_HANDLE_EX_FLAG_PIN: u32 = 0x0000_0001;
    const GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS: u32 = 0x0000_0004;
    if code_address.is_null() {
        return false;
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn GetModuleHandleExW(
            flags: u32,
            module_name: *const u16,
            module: *mut *mut core::ffi::c_void,
        ) -> i32;
    }
    let mut module = core::ptr::null_mut();
    // SAFETY: `code_address` is a function pointer copied from the loaded
    // runtime export. PIN | FROM_ADDRESS asks the loader to keep that module
    // mapped until process termination. The returned HMODULE is not released;
    // releasing it would fight the pin.
    let pinned = unsafe {
        GetModuleHandleExW(
            GET_MODULE_HANDLE_EX_FLAG_PIN | GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS,
            code_address.cast::<u16>(),
            &mut module,
        )
    };
    pinned != 0
}

#[cfg(feature = "diagnostic-log")]
fn write_log(module: &str, message: impl AsRef<str>) {
    zircon_runtime::diagnostic_log::write_log(module, message);
}

#[cfg(not(feature = "diagnostic-log"))]
fn write_log(module: &str, message: impl AsRef<str>) {
    tracing::info!(module, message = message.as_ref(), "runtime module mapping");
}

/// Reference-count model of editor load, Play load, Play stop, and editor exit.
///
/// Production sealing calls [`plan_module_release_for`]. This ledger applies
/// that same plan so a host test can show which release unmaps the image.
#[cfg(test)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RuntimeImageLedger {
    session_handles: u32,
    leaked_handles: u32,
    pinned: bool,
}

#[cfg(test)]
impl RuntimeImageLedger {
    pub(crate) fn new() -> Self {
        Self {
            session_handles: 0,
            leaked_handles: 0,
            pinned: false,
        }
    }

    pub(crate) fn load_and_seal(&mut self, policy: ModuleMappingPolicy, pin_succeeded: bool) {
        self.session_handles = self.session_handles.saturating_add(1);
        let plan = plan_module_release_for(policy, pin_succeeded);
        if plan.process_receipt == ProcessModuleReceipt::PinnedUntilProcessExit {
            self.pinned = true;
        }
        if !plan.retain_library_handle {
            self.session_handles = self.session_handles.saturating_sub(1);
            self.leaked_handles = self.leaked_handles.saturating_add(1);
        }
    }

    pub(crate) fn release_session_handle(&mut self) {
        self.session_handles = self.session_handles.saturating_sub(1);
    }

    pub(crate) fn image_mapped(&self) -> bool {
        self.session_handles > 0 || self.leaked_handles > 0 || self.pinned
    }
}

#[cfg(test)]
mod tests {
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
}
