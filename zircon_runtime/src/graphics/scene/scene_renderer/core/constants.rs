// 场景内部保持线性 HDR，最终输出转换后才写入 sRGB 目标；跨阶段纹理格式须与 shader 和图声明一致。
pub(crate) const SCENE_COLOR_HDR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
pub(crate) const FINAL_COLOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
pub(crate) const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

#[cfg(test)]
#[path = "tests/constants.rs"]
mod tests;
