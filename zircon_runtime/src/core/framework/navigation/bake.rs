use serde::{Deserialize, Serialize};

use super::NavMeshAsset;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NavMeshBakeRequest {
    pub surface_entity: Option<u64>,
    pub agent_type: Option<String>,
    pub output_asset: Option<String>,
    pub force_full_rebuild: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NavMeshBakeDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NavMeshBakeDiagnostic {
    pub severity: NavMeshBakeDiagnosticSeverity,
    pub message: String,
    pub entity: Option<u64>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NavMeshBakeReport {
    pub asset: Option<NavMeshAsset>,
    pub output_asset: Option<String>,
    pub surfaces: usize,
    pub source_vertices: usize,
    pub source_triangles: usize,
    pub baked_vertices: usize,
    pub baked_polygons: usize,
    pub tiles: usize,
    pub diagnostics: Vec<NavMeshBakeDiagnostic>,
}
