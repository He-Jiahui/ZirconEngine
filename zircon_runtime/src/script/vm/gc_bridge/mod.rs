mod budget;
mod host_handle;
mod vm_object_ref;

pub use budget::{
    VmGcBudget, VmGcDiagnostics, VmGcSlotStepReport, VmGcStepOutcome, VmGcStepReport,
    DEFAULT_VM_GC_MAX_MICROS_PER_FRAME, VM_GC_DIAGNOSTICS_HISTORY_CAPACITY,
};
pub use host_handle::HostHandle;
pub use vm_object_ref::{
    VmGcRootRegistrationError, VmGcRootRegistry, VmGcRootToken, VmObjectId, VmObjectRef,
    VmObjectRefError,
};
