use super::super::MaterialRuntime;

pub(in crate::graphics::scene::resources) struct PreparedMaterial {
    pub(in crate::graphics::scene::resources) revision: u64,
    pub(in crate::graphics::scene::resources) runtime: MaterialRuntime,
}
