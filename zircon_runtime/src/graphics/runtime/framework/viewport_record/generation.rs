use super::viewport_record::ViewportRecord;

impl ViewportRecord {
    pub(in crate::graphics::runtime::render_framework) fn generation(&self) -> u64 {
        self.generation
    }

    pub(in crate::graphics::runtime::render_framework) fn bump_generation(&mut self) {
        self.generation = self.generation.wrapping_add(1);
    }
}
