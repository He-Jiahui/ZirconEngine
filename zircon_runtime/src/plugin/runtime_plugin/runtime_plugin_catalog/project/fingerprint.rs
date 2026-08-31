use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::{ExportPackagingStrategy, ProjectPluginManifest};

pub(super) fn project_manifest_fingerprint(manifest: &ProjectPluginManifest) -> u64 {
    let mut fingerprint = ManifestFingerprint::new();
    fingerprint.write_len(manifest.selections.len());
    for selection in &manifest.selections {
        fingerprint.write_str(&selection.id);
        fingerprint.write_bool(selection.enabled);
        fingerprint.write_bool(selection.required);
        fingerprint.write_target_modes(&selection.target_modes);
        fingerprint.write_packaging(selection.packaging);
        fingerprint.write_optional_str(selection.runtime_crate.as_deref());
        fingerprint.write_optional_str(selection.editor_crate.as_deref());
        fingerprint.write_len(selection.features.len());
        for feature in &selection.features {
            fingerprint.write_str(&feature.id);
            fingerprint.write_bool(feature.enabled);
            fingerprint.write_bool(feature.required);
            fingerprint.write_target_modes(&feature.target_modes);
            fingerprint.write_packaging(feature.packaging);
            fingerprint.write_optional_str(feature.runtime_crate.as_deref());
            fingerprint.write_optional_str(feature.editor_crate.as_deref());
            fingerprint.write_optional_str(feature.provider_package_id.as_deref());
        }
    }
    fingerprint.finish()
}

struct ManifestFingerprint(u64);

impl ManifestFingerprint {
    const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;

    fn new() -> Self {
        Self(Self::OFFSET_BASIS)
    }

    fn finish(self) -> u64 {
        self.0
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 ^= u64::from(*byte);
            self.0 = self.0.wrapping_mul(Self::PRIME);
        }
    }

    fn write_len(&mut self, value: usize) {
        self.write_bytes(&(value as u64).to_le_bytes());
    }

    fn write_str(&mut self, value: &str) {
        self.write_len(value.len());
        self.write_bytes(value.as_bytes());
    }

    fn write_bool(&mut self, value: bool) {
        self.write_bytes(&[u8::from(value)]);
    }

    fn write_optional_str(&mut self, value: Option<&str>) {
        self.write_bool(value.is_some());
        if let Some(value) = value {
            self.write_str(value);
        }
    }

    fn write_target_modes(&mut self, values: &[RuntimeTargetMode]) {
        self.write_len(values.len());
        for value in values {
            self.write_bytes(&[project_plan_target_key(*value)]);
        }
    }

    fn write_packaging(&mut self, value: ExportPackagingStrategy) {
        self.write_bytes(&[match value {
            ExportPackagingStrategy::SourceTemplate => 0,
            ExportPackagingStrategy::LibraryEmbed => 1,
            ExportPackagingStrategy::NativeDynamic => 2,
        }]);
    }
}

fn project_plan_target_key(target: RuntimeTargetMode) -> u8 {
    match target {
        RuntimeTargetMode::ClientRuntime => 0,
        RuntimeTargetMode::ServerRuntime => 1,
        RuntimeTargetMode::EditorHost => 2,
    }
}
