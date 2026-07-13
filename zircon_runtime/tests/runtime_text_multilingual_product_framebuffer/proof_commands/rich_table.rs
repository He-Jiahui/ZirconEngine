use super::*;

const HORIZONTAL_TABLE_MARKUP: &str = "[table=3][cell colspan=3 expand=1 border=#73D7FF bg=#102638 padding=16,10,16,12][b]BBCode V2 CELL BOX / COLSPAN + ROWSPAN PRODUCT[/b][/cell][cell rowspan=2 shrink=false border=#F6B65A bg=#182F3D,#213B4B padding=12,8,14,10][color=#64d8ff]Span owner[/color][/cell][cell colspan=2 border=#73D7FF bg=#12202C,#182F3D padding=14,7,16,9]colspan shares measured width across two runtime tracks[/cell][cell border=#73D7FF bg=#12202C,#182F3D padding=10,6,12,8]wrapped detail alpha beta gamma delta epsilon zeta eta theta[/cell][cell border=#F6B65A bg=#1C2633,#2A3440 padding=10,6,12,8][icon=★|Microsoft YaHei UI] real WGPU cell box frame[/cell][/table]";

const VERTICAL_TABLE_MARKUP: &str = "[table=2][cell colspan=2 expand=1 border=#73D7FF bg=#102638 padding=6,8,10,12][b]VERTICAL 纵表[/b][/cell][cell rowspan=2 shrink=false border=#F6B65A bg=#182F3D,#213B4B padding=7,9,11,13][color=#64d8ff]跨行 SPAN[/color][/cell][cell border=#73D7FF bg=#12202C,#182F3D padding=5,7,9,11]列一 A1[/cell][cell border=#73D7FF bg=#12202C,#182F3D padding=5,7,9,11]列二 B2[/cell][cell colspan=2 border=#F6B65A bg=#1C2633,#2A3440 padding=6,8,10,12]RTL AXES 终[/cell][/table]";

pub(in crate::proof_commands) fn proof_horizontal_rich_table() -> UiRenderCommand {
    proof_bbcode_text(
        118,
        UiFrame::new(42.0, 1120.0, 600.0, 300.0),
        HORIZONTAL_TABLE_MARKUP,
        UiTextWrap::WordSmart,
    )
}

pub(in crate::proof_commands) fn proof_vertical_rich_table() -> UiRenderCommand {
    let frame = UiFrame::new(690.0, 1120.0, 348.0, 300.0);
    let mut command = proof_text(
        119,
        frame,
        VERTICAL_TABLE_MARKUP,
        UiTextDirection::LeftToRight,
        Some("zh-Hans"),
        UiTextRenderMode::Sdf,
    );
    command.style.rich_text_format = UiRichTextFormat::BbCode;
    command.style.text_writing_mode = UiTextWritingMode::VerticalRl;
    command.style.font_family = Some("Microsoft YaHei UI".to_string());
    command.style.font_size = 22.0;
    command.style.line_height = 32.0;
    command.style.wrap = UiTextWrap::Glyph;
    command.text_layout = Some(layout_text(
        VERTICAL_TABLE_MARKUP,
        &command.style,
        frame,
        None,
    ));
    command
}
