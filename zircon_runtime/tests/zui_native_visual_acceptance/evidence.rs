use super::catalog::Case;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Evidence {
    pub case_id: String,
    pub source_path: String,
    pub status: String,
    pub renderer_kind: String,
    pub renderer_sha256: String,
    pub renderer_path: String,
    pub screenshot_path: String,
    pub screenshot_sha256: String,
    pub source_sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_host_sha256: Option<String>,
    pub dependency_sha256: String,
    pub case_sha256: String,
    pub geometry_path: Option<String>,
    pub geometry_sha256: Option<String>,
    pub text_path: Option<String>,
    pub text_sha256: Option<String>,
    pub error: Option<String>,
    pub runtime_asset_fingerprints: Vec<(String, String)>,
}

pub(super) fn failed(
    case: &Case,
    source_path: &str,
    source: String,
    dependency: String,
    case_hash: String,
    error: String,
) -> Evidence {
    Evidence {
        case_id: case.id.clone(),
        source_path: source_path.into(),
        status: "failed".into(),
        renderer_kind: "zircon-runtime-wgpu-headless".into(),
        renderer_sha256: String::new(),
        renderer_path: String::new(),
        screenshot_path: String::new(),
        screenshot_sha256: String::new(),
        source_sha256: source,
        review_host_sha256: None,
        dependency_sha256: dependency,
        case_sha256: case_hash,
        geometry_path: None,
        geometry_sha256: None,
        text_path: None,
        text_sha256: None,
        error: Some(error),
        runtime_asset_fingerprints: Vec::new(),
    }
}
