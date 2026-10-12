// 视口网格和计算着色器的 dispatch 契约：瓦片边长与工作组边长分别控制空间分区和并行度。
pub(in crate::graphics::scene::scene_renderer::post_process) const CLUSTER_TILE_SIZE: u32 = 16;
pub(in crate::graphics::scene::scene_renderer::post_process) const CLUSTER_WORKGROUP_SIZE: u32 = 8;
