use std::sync::Arc;
use zr_rhi::{DeviceFaultGate, DeviceFaultKind, DeviceGeneration, DeviceId};

use crate::WgpuDeviceErrorSupervisor;

#[test]
fn supervisor_keeps_the_shared_fault_gate_for_the_device_lifetime() {
    let gate = Arc::new(DeviceFaultGate::new(
        DeviceId::new(11),
        DeviceGeneration::initial(),
    ));
    let supervisor = WgpuDeviceErrorSupervisor::for_gate(Arc::clone(&gate));

    assert!(supervisor.fault_gate().ensure_admission().is_ok());
    assert!(gate.record_first(DeviceFaultKind::DeviceLostUnknown, "test loss"));
    assert_eq!(
        supervisor
            .fault_gate()
            .first_fault()
            .expect("shared fault gate")
            .kind,
        DeviceFaultKind::DeviceLostUnknown
    );
}
