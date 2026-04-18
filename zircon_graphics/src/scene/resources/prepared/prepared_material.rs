use super::super::MaterialRuntime;

pub(in crate::scene::resources) struct PreparedMaterial {
    pub(in crate::scene::resources) revision: u64,
    pub(in crate::scene::resources) runtime: MaterialRuntime,
}
