use super::*;

#[test]
fn tooltip_layers_keep_shadow_bubble_text_arrow_icon_order() {
    let shadow = 20;

    assert!(shadow < bubble_order(shadow));
    assert!(bubble_order(shadow) < text_order(shadow));
    assert!(text_order(shadow) < arrow_order(shadow));
    assert!(arrow_order(shadow) < icon_order(shadow));
    assert!(text_order(shadow) < body_order(text_order(shadow)));
}
