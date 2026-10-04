//! 渲染帧间保存当前与上一关节矩阵调色板；上传一次推进一次历史，不代表资源已送达 GPU。
use super::SkinningPalette;

#[derive(Clone, Debug, Default)]
pub struct SkinningPaletteDoubleBuffer {
    current: SkinningPalette,
    previous: SkinningPalette,
}

impl SkinningPaletteDoubleBuffer {
    /// 每次调用都会推进前一姿态；渲染帧所有者应按期望历史频率调用。
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
