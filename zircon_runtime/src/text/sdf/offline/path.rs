use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use super::SdfOfflineArtifactIdentity;
use crate::text::sdf::SdfMode;

pub(crate) fn sdf_offline_artifact_path(
    cache_root: &Path,
    identity: &SdfOfflineArtifactIdentity,
) -> PathBuf {
    cache_root
        .join("text")
        .join("sdf")
        .join("v1")
        .join(&identity.asset_guid)
        .join(format!("face_{:04}", identity.face_index))
        .join(hex(&identity.variation_hash))
        .join(format!(
            "{}_{}_{}.zsdf",
            mode_name(identity.params.mode),
            identity.params.bake_em_px,
            identity.params.spread_px_milli
        ))
}

fn mode_name(mode: SdfMode) -> &'static str {
    match mode {
        SdfMode::Sdf => "sdf",
        SdfMode::Msdf => "msdf",
        SdfMode::Mtsdf => "mtsdf",
    }
}

fn hex(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(encoded, "{byte:02x}");
    }
    encoded
}
