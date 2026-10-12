/// G-buffer 法线附件的共享格式；离屏纹理、写入管线和片元输出验证必须引用同一值。
pub(crate) const NORMAL_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
