use super::SkinningPalette;

#[derive(Clone, Debug, Default)]
pub struct SkinningPaletteDoubleBuffer {
    current: SkinningPalette,
    previous: SkinningPalette,
}

impl SkinningPaletteDoubleBuffer {
    pub fn upload(&mut self, palette: &SkinningPalette) {
        std::mem::swap(&mut self.current, &mut self.previous);
        self.current.clone_from(palette);
    }

    pub const fn current(&self) -> &SkinningPalette {
        &self.current
    }

    pub const fn previous(&self) -> &SkinningPalette {
        &self.previous
    }
}
