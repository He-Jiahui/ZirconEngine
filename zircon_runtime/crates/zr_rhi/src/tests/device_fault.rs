use crate::{DeviceAdmissionError, DeviceFaultGate, DeviceFaultKind, DeviceGeneration, DeviceId};

#[test]
fn device_fault_gate_admits_the_healthy_generation_without_a_lock() {
    let gate = DeviceFaultGate::new(DeviceId::new(7), DeviceGeneration::initial());

    assert!(gate.ensure_admission().is_ok());
    assert!(gate.first_fault().is_none());
}

#[test]
fn device_fault_gate_records_only_the_first_fault_and_closes_admission() {
    let gate = DeviceFaultGate::new(DeviceId::new(7), DeviceGeneration::initial());

    assert!(gate.record_first(DeviceFaultKind::Validation, "first validation failure"));
    assert!(!gate.record_first(DeviceFaultKind::OutOfMemory, "later OOM"));

    let fault = gate.first_fault().expect("first fault must be retained");
    assert_eq!(fault.device_id, DeviceId::new(7));
    assert_eq!(fault.generation, DeviceGeneration::initial());
    assert_eq!(fault.kind, DeviceFaultKind::Validation);
    assert_eq!(fault.message, "first validation failure");
    assert_eq!(
        gate.ensure_admission(),
        Err(DeviceAdmissionError::Faulted {
            kind: DeviceFaultKind::Validation,
        })
    );
}
