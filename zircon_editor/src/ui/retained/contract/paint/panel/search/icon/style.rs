//! 搜索装饰图标 tint的只读快照；供对应 painter 在命令构造时消费。
//! 颜色与尺寸由上游主题或行状态传入，此层不持有交互状态或布局 owner。

use super::super::super::super::palette::WorkbenchCommandPalettePalette;

pub(super) struct CommandPaletteSearchIconStyle {
    pub tint: Option<[u8; 4]>,
}

pub(super) fn command_palette_search_icon_style(
    palette: &WorkbenchCommandPalettePalette,
) -> CommandPaletteSearchIconStyle {
    CommandPaletteSearchIconStyle {
        tint: Some(palette.search_icon),
    }
}
