//! 共享 GPU 表示转换边界；任一分量转换失败时，整体采用调用方指定的回退值。
//! 具体默认值由顶点或 uniform 使用场景选择；这里不拼接部分成功的分量。
#[path = "mat4/mod.rs"]
mod mat4;
#[path = "vec3/mod.rs"]
mod vec3;
#[path = "vec4/mod.rs"]
mod vec4;

pub(crate) use mat4::render_mat4_or;
pub(crate) use vec3::render_vec3_or;
pub(crate) use vec4::render_vec4_or;
