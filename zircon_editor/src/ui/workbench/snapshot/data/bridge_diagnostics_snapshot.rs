use zircon_runtime::plugin::{BridgeDiagnosticsMatrix, BridgeInterfaceSnapshot};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EditorBridgeDiagnosticsSnapshot {
    pub summary: EditorBridgeDiagnosticsSummarySnapshot,
    pub rows: Vec<EditorBridgeInterfaceRowSnapshot>,
    pub diagnostic_lines: Vec<String>,
}

impl EditorBridgeDiagnosticsSnapshot {
    pub fn from_runtime_matrix(matrix: &BridgeDiagnosticsMatrix) -> Self {
        Self {
            summary: EditorBridgeDiagnosticsSummarySnapshot {
                total_interfaces: matrix.summary.total_interfaces,
                enabled_interfaces: matrix.summary.enabled_interfaces,
                disabled_interfaces: matrix.summary.disabled_interfaces,
                installed_providers: matrix.summary.installed_providers,
                missing_providers: matrix.summary.missing_providers,
                enabled_calls: matrix.summary.enabled_calls,
                not_enabled_calls: matrix.summary.not_enabled_calls,
            },
            rows: matrix
                .rows
                .iter()
                .map(EditorBridgeInterfaceRowSnapshot::from_runtime_row)
                .collect(),
            diagnostic_lines: matrix.diagnostic_lines(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EditorBridgeDiagnosticsSummarySnapshot {
    pub total_interfaces: usize,
    pub enabled_interfaces: usize,
    pub disabled_interfaces: usize,
    pub installed_providers: usize,
    pub missing_providers: usize,
    pub enabled_calls: u64,
    pub not_enabled_calls: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorBridgeInterfaceRowSnapshot {
    pub slot: u32,
    pub interface_id: String,
    pub owner_module_slot: u32,
    pub generation: u32,
    pub provider_installed: bool,
    pub status: String,
    pub enabled_calls: u64,
    pub not_enabled_calls: u64,
}

impl EditorBridgeInterfaceRowSnapshot {
    fn from_runtime_row(row: &BridgeInterfaceSnapshot) -> Self {
        Self {
            slot: row.slot.raw(),
            interface_id: row.interface_id.clone(),
            owner_module_slot: row.owner.raw(),
            generation: row.generation,
            provider_installed: row.provider_installed,
            status: format!("{:?}", row.status),
            enabled_calls: row.diagnostics.enabled_calls,
            not_enabled_calls: row.diagnostics.not_enabled_calls,
        }
    }
}
