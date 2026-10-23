use std::ptr;

use crate::{check_status, Error, ProjectSession};
use zr_vm_rust_binding_sys as sys;

/// 静止 session 边界上的 VM 快照；native 句柄保留 owner 与快照 root，回滚仍需原 session。
pub struct ProjectSessionCheckpoint {
    raw: *mut sys::ZrRustBindingProjectSessionCheckpoint,
}

impl ProjectSession {
    pub fn checkpoint(&mut self) -> Result<ProjectSessionCheckpoint, Error> {
        let mut raw = ptr::null_mut();
        // guard-allow(rust.unsafe-without-safety): Pinned upstream FFI operation retained verbatim; original hashes are in UPSTREAM.json; no unsafe operation added.
        check_status(unsafe { sys::ZrRustBinding_ProjectSession_Checkpoint(self.raw, &mut raw) })?;
        Ok(ProjectSessionCheckpoint { raw })
    }

    pub fn rollback(&mut self, checkpoint: &ProjectSessionCheckpoint) -> Result<(), Error> {
        // guard-allow(rust.unsafe-without-safety): Pinned upstream FFI operation retained verbatim; original hashes are in UPSTREAM.json; no unsafe operation added.
        check_status(unsafe {
            sys::ZrRustBinding_ProjectSession_Rollback(self.raw, checkpoint.raw)
        })
    }
}

impl Drop for ProjectSessionCheckpoint {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            // guard-allow(rust.unsafe-without-safety): Pinned upstream FFI operation retained verbatim; original hashes are in UPSTREAM.json; no unsafe operation added.
            unsafe {
                let _ = sys::ZrRustBinding_ProjectSessionCheckpoint_Free(self.raw);
            }
            self.raw = ptr::null_mut();
        }
    }
}
