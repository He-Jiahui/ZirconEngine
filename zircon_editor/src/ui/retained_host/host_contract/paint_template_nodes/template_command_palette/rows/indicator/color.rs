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
