use serde::{Deserialize, Serialize};

use super::NavMeshAsset;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
/// 编辑器操作与导航插件之间的烘焙请求；请求字段覆盖选中 surface 的默认配置。
pub struct NavMeshBakeRequest {
    // TODO: [CR-NAVIGATION-0001] 明确指定的 surface 缺失或禁用时是否允许回退到首个启用项；当前 select_bake_surface 会静默改选，需补契约测试。
    pub surface_entity: Option<u64>,
    pub agent_type: Option<String>,
    pub output_asset: Option<String>,
    // TODO: [CR-NAVIGATION-0002] 明确强制全量重建的调用语义；当前烘焙、分块与脏块入口均未读取此字段，需验证调用者预期。
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
/// 烘焙结果同时承载可发布网格和诊断；发布前的调用方应保留诊断以解释回退或空结果。
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
