//! 检索提示颜色跟随宿主语义角色；disabled 只降低呈现强度，不在 painter 中取消检索匹配。

use super::super::super::super::super::data::TemplatePaneOptionData;
use super::super::super::palette::command_palette_palette;

pub(super) fn command_row_match_indicator_color(option: &TemplatePaneOptionData) -> [u8; 4] {
    let palette = command_palette_palette();
    if option.disabled {
        palette.match_indicator_disabled
    } else {
        palette.match_indicator
    }
}
