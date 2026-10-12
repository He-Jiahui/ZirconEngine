use crate::core::math::UVec2;
use crate::graphics::resource_identity::SampledTextureIdentity;

pub(crate) const TAA_SCENE_COLOR_HISTORY_FORMAT: wgpu::TextureFormat =
    wgpu::TextureFormat::Rgba16Float;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TemporalHistoryKey {
    size: UVec2,
    format: wgpu::TextureFormat,
}

impl TemporalHistoryKey {
    pub(crate) fn new(size: UVec2, format: wgpu::TextureFormat) -> Self {
        Self { size, format }
    }
}

/// TAA 场景色的双缓冲所有者；只在场景帧成功提交后交换读写角色。
/// 尺寸或格式变化时须用新 key 重建，不能复用旧物理身份。
pub(crate) struct TemporalHistoryStore {
    key: TemporalHistoryKey,
    textures: [TemporalHistoryTexture; 2],
    state: TemporalHistoryState,
}

impl TemporalHistoryStore {
    pub(crate) fn new(
        key: TemporalHistoryKey,
        read_texture: wgpu::Texture,
        read_view: wgpu::TextureView,
        write_texture: wgpu::Texture,
        write_view: wgpu::TextureView,
    ) -> Self {
        Self {
            key,
            textures: [
                TemporalHistoryTexture::new(read_texture, read_view),
                TemporalHistoryTexture::new(write_texture, write_view),
            ],
            state: TemporalHistoryState::default(),
        }
    }

    pub(crate) fn matches_key(&self, key: TemporalHistoryKey) -> bool {
        self.key == key
    }

    pub(crate) fn flip_after_success(&mut self) {
        self.state.flip_after_success();
    }

    pub(crate) fn previous_view(&self) -> &wgpu::TextureView {
        &self.textures[self.state.read_index].view
    }

    pub(crate) fn previous_texture(&self) -> &wgpu::Texture {
        &self.textures[self.state.read_index].texture
    }

    pub(crate) fn previous_identity(&self) -> SampledTextureIdentity {
        self.textures[self.state.read_index].identity
    }

    pub(crate) fn current_view(&self) -> &wgpu::TextureView {
        &self.textures[self.state.write_index()].view
    }

    pub(crate) fn current_texture(&self) -> &wgpu::Texture {
        &self.textures[self.state.write_index()].texture
    }

    pub(crate) fn current_identity(&self) -> SampledTextureIdentity {
        self.textures[self.state.write_index()].identity
    }
}

struct TemporalHistoryTexture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    identity: SampledTextureIdentity,
}

impl TemporalHistoryTexture {
    fn new(texture: wgpu::Texture, view: wgpu::TextureView) -> Self {
        Self {
            texture,
            view,
            identity: SampledTextureIdentity::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct TemporalHistoryState {
    read_index: usize,
}

impl TemporalHistoryState {
    fn write_index(self) -> usize {
        1 - self.read_index
    }

    fn flip_after_success(&mut self) {
        self.read_index = self.write_index();
    }
}

#[cfg(test)]
#[path = "tests/temporal_history_store.rs"]
mod tests;
