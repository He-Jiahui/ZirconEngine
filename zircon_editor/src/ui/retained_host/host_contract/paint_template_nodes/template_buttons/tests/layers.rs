use super::*;

#[test]
fn button_orders_keep_surface_overlay_content_and_label_stack() {
    let surface = 12;
    let overlay = surface_overlay_order(surface);
    let content = content_order(surface);
    let label = label_order(content);

    assert_eq!(overlay, 13);
    assert_eq!(content, 14);
    assert_eq!(label, 15);
    assert!(surface < overlay);
    assert!(overlay < content);
    assert!(content < label);
}
