use zr_rhi::DiagnosticReadbackReceipt;

/// Backend-neutral diagnostic terminal receipt paired with copied bytes when
/// the request completed successfully. Native WGPU objects never escape this
/// delivery boundary.
#[derive(Debug)]
pub struct WgpuDiagnosticReadbackDelivery {
    pub(super) receipt: DiagnosticReadbackReceipt,
    pub(super) bytes: Option<Vec<u8>>,
}

impl WgpuDiagnosticReadbackDelivery {
    pub const fn receipt(&self) -> DiagnosticReadbackReceipt {
        self.receipt
    }

    pub fn bytes(&self) -> Option<&[u8]> {
        self.bytes.as_deref()
    }

    pub fn into_bytes(self) -> Option<Vec<u8>> {
        self.bytes
    }

    pub(super) fn byte_len_for_budget(&self) -> u64 {
        self.bytes.as_ref().map_or(0, |bytes| bytes.len() as u64)
    }
}
