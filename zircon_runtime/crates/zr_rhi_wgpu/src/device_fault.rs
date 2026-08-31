use std::sync::Arc;

use zr_rhi::{DeviceFaultGate, DeviceFaultKind};

/// Concrete bridge that records the first WGPU error without recovery work in callbacks.
#[derive(Clone)]
pub struct WgpuDeviceErrorSupervisor {
    fault_gate: Arc<DeviceFaultGate>,
}

impl WgpuDeviceErrorSupervisor {
    pub fn for_gate(fault_gate: Arc<DeviceFaultGate>) -> Self {
        Self { fault_gate }
    }

    /// Installs the only uncaptured-error and device-loss callbacks for this WGPU device.
    pub fn install(device: &wgpu::Device, fault_gate: Arc<DeviceFaultGate>) -> Self {
        let supervisor = Self::for_gate(fault_gate);
        let uncaptured_error_gate = Arc::clone(&supervisor.fault_gate);
        device.on_uncaptured_error(Arc::new(move |error| {
            let kind = match &error {
                wgpu::Error::OutOfMemory { .. } => DeviceFaultKind::OutOfMemory,
                wgpu::Error::Validation { .. } => DeviceFaultKind::Validation,
                wgpu::Error::Internal { .. } => DeviceFaultKind::Internal,
            };
            uncaptured_error_gate.record_first(kind, error.to_string());
        }));

        let device_lost_gate = Arc::clone(&supervisor.fault_gate);
        device.set_device_lost_callback(move |reason, message| {
            let kind = match reason {
                wgpu::DeviceLostReason::Unknown => DeviceFaultKind::DeviceLostUnknown,
                wgpu::DeviceLostReason::Destroyed => DeviceFaultKind::DeviceDestroyed,
            };
            device_lost_gate.record_first(kind, message);
        });

        supervisor
    }

    pub fn fault_gate(&self) -> &Arc<DeviceFaultGate> {
        &self.fault_gate
    }
}
