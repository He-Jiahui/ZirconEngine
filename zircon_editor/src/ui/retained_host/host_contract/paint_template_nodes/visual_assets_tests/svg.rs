use super::super::svg;

#[test]
fn svg_font_scan_is_reserved_for_text_svg() {
    assert!(!svg::svg_may_need_fonts(
        br#"<svg viewBox="0 0 16 16"><path d="M0 0h16v16H0z"/></svg>"#
    ));
    assert!(svg::svg_may_need_fonts(
        br#"<svg viewBox="0 0 16 16"><text x="0" y="12">A</text></svg>"#
    ));
    assert!(svg::svg_may_need_fonts(
        br#"<svg viewBox="0 0 16 16"><path style="font-family:Arial" /></svg>"#
    ));
}
