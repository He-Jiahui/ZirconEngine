use crate::core::math::UVec2;

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

    pub(crate) fn is_valid(&self) -> bool {
        self.state.valid
    }

    pub(crate) fn invalidate(&mut self) {
        self.state.invalidate();
    }

    pub(crate) fn flip_after_success(&mut self) {
        self.state.flip_after_success();
    }

    pub(crate) fn previous_view(&self) -> &wgpu::TextureView {
        &self.textures[self.state.read_index].view
    }

    pub(crate) fn current_view(&self) -> &wgpu::TextureView {
        &self.textures[self.state.write_index()].view
    }
}

struct TemporalHistoryTexture {
    #[allow(dead_code)]
    texture: wgpu::Texture,
    view: wgpu::TextureView,
}

impl TemporalHistoryTexture {
    fn new(texture: wgpu::Texture, view: wgpu::TextureView) -> Self {
        Self { texture, view }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct TemporalHistoryState {
    read_index: usize,
    valid: bool,
}

impl TemporalHistoryState {
    fn write_index(self) -> usize {
        1 - self.read_index
    }

    fn invalidate(&mut self) {
        self.valid = false;
    }

    fn flip_after_success(&mut self) {
        self.read_index = self.write_index();
        self.valid = true;
    }
}

#[cfg(test)]
mod tests {
    use super::TemporalHistoryState;

    #[test]
    fn temporal_history_state_starts_invalid_and_flips_read_write_slots() {
        let mut state = TemporalHistoryState::default();

        assert_eq!(state.read_index, 0);
        assert_eq!(state.write_index(), 1);
        assert!(!state.valid);

        state.flip_after_success();

        assert_eq!(state.read_index, 1);
        assert_eq!(state.write_index(), 0);
        assert!(state.valid);
    }

    #[test]
    fn temporal_history_state_invalidation_keeps_slots_but_drops_validity() {
        let mut state = TemporalHistoryState::default();
        state.flip_after_success();
        let read_index = state.read_index;

        state.invalidate();

        assert_eq!(state.read_index, read_index);
        assert!(!state.valid);
    }
}
