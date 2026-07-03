use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{UiTextOverflow, UiTextWrap},
};

use super::{layout_text, measure_text_size, test_style};

#[test]
fn text_tab_stop_advances_to_next_interval() {
    let mut style = test_style(UiTextWrap::None, UiTextOverflow::Clip);
    style.tab_size = 4.0;
    let text = "a\tb";
    let a_width = measure_text_size("a", &style).width;
    let tab_interval = measure_text_size(" ", &style).width * style.tab_size;
    let expected_tab_advance = tab_interval - a_width.rem_euclid(tab_interval);

    let layout = layout_text(text, &style, UiFrame::new(0.0, 0.0, 200.0, 12.0), None);
    let line = &layout.lines[0];

    assert_eq!(line.text, text);
    assert_eq!(line.glyph_advances.len(), 3);
    assert!(
        (line.glyph_advances[1] - expected_tab_advance).abs() < 0.1,
        "tab advance should move the cursor to the next tab interval"
    );
    assert!((line.measured_width - line.glyph_advances.iter().sum::<f32>()).abs() < 0.1);
}
