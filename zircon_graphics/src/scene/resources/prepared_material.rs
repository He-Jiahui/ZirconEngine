use super::material_runtime::MaterialRuntime;

pub(super) struct PreparedMaterial {
    pub(super) revision: u64,
    pub(super) runtime: MaterialRuntime,
}
