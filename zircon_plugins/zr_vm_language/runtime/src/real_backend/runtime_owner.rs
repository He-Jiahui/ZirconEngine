use zr_vm_rust_binding as zrvm;

use std::sync::MutexGuard;

use super::lock::acquire_zr_vm_lock;
use super::ZrVmRegistration;

/// Owns every raw-pointer-backed ZrVM object that belongs to one plugin instance.
///
/// All access and destruction are serialized by the process-wide ZrVM lock. The
/// explicit drop order is session, native registrations, then runtime because
/// each earlier object may retain pointers into the following owner.
pub(super) struct ZrVmRuntimeOwner {
    session: Option<zrvm::ProjectSession>,
    registrations: Option<Vec<ZrVmRegistration>>,
    runtime: Option<zrvm::Runtime>,
}

// SAFETY: the contained binding types wrap process-global C runtime pointers.
// They are never accessed or destroyed without `acquire_zr_vm_lock`, so moving
// the owner between host threads cannot create concurrent ZrVM access.
unsafe impl Send for ZrVmRuntimeOwner {}
// SAFETY: shared references never expose the binding values; all mutable access
// is reached through `&mut self` while holding the same process-wide lock.
unsafe impl Sync for ZrVmRuntimeOwner {}

impl ZrVmRuntimeOwner {
    pub(super) fn new(
        session: zrvm::ProjectSession,
        registrations: Vec<ZrVmRegistration>,
        runtime: zrvm::Runtime,
    ) -> Self {
        Self {
            session: Some(session),
            registrations: Some(registrations),
            runtime: Some(runtime),
        }
    }

    pub(super) fn call_module_export(
        &mut self,
        _guard: &MutexGuard<'static, ()>,
        module_name: &str,
        export_name: &str,
        arguments: &[zrvm::Value],
    ) -> Result<zrvm::Value, zrvm::Error> {
        self.session_mut()
            .call_module_export(module_name, export_name, arguments)
    }

    pub(super) fn gc_step(
        &mut self,
        _guard: &MutexGuard<'static, ()>,
        max_micros_per_frame: u64,
    ) -> Result<zrvm::GcStepResult, zrvm::Error> {
        self.session_mut().gc_step(max_micros_per_frame)
    }

    fn session_mut(&mut self) -> &mut zrvm::ProjectSession {
        self.session
            .as_mut()
            .expect("ZrVM runtime owner session remains live until drop")
    }
}

impl Drop for ZrVmRuntimeOwner {
    fn drop(&mut self) {
        let _guard = acquire_zr_vm_lock();
        drop(self.session.take());
        drop(self.registrations.take());
        drop(self.runtime.take());
    }
}
