use super::parse_css_color;
use zircon_runtime_interface::ui::style::UiRgbaColor;

#[test]
fn authored_transparent_button_color_does_not_trigger_opaque_fallback() {
    assert_eq!(
        parse_css_color(" transparent ").map(UiRgbaColor::to_u8),
        Some([0, 0, 0, 0])
    );
    assert_eq!(
        parse_css_color("#12aBcF80").map(UiRgbaColor::to_u8),
        Some([18, 171, 207, 128])
    );
    assert_eq!(parse_css_color("unknown"), None);
}
