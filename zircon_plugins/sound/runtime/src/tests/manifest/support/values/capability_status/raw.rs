pub(super) fn capability_status_from_plugin_toml(
    value: &str,
) -> zircon_runtime::plugin::CapabilityStatus {
    match value {
        "complete" => zircon_runtime::plugin::CapabilityStatus::Complete,
        "partial" => zircon_runtime::plugin::CapabilityStatus::Partial,
        "stub" => zircon_runtime::plugin::CapabilityStatus::Stub,
        "externalized" => zircon_runtime::plugin::CapabilityStatus::Externalized,
        "unsupported" => zircon_runtime::plugin::CapabilityStatus::Unsupported,
        _ => panic!("unknown sound capability status {value}"),
    }
}
