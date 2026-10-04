//! 给离线产物分配稳定的版本化缓存位置；路径只用于定位候选文件，实际可复用性由载入后的完整 identity 核验决定。

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use super::SdfOfflineArtifactIdentity;
use crate::text::sdf::SdfMode;

/// 路径未包含 source_hash，字体内容变化后仍会找到同一路径的候选；调用方须通过 artifact.validate_identity 拒绝旧数据。
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
        .join(hex(identity.variation_hash.as_bytes()))
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
