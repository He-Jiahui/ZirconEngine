pub(super) fn capability_status_manifest(
    capability: String,
    status: zircon_runtime::plugin::CapabilityStatus,
    bevy_references: impl IntoIterator<Item = String>,
) -> zircon_runtime::plugin::CapabilityStatusManifest {
    let mut manifest = zircon_runtime::plugin::CapabilityStatusManifest::new(capability, status);
    for reference in bevy_references {
        manifest = manifest.with_bevy_reference(reference);
    }
    manifest
}
