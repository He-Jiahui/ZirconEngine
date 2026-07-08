const BUBBLE_OFFSET: i32 = 1;
const TEXT_OFFSET: i32 = 2;
const ARROW_OFFSET: i32 = 3;
const ICON_OFFSET: i32 = 4;
const BODY_OFFSET: i32 = 1;

pub(super) fn bubble_order(shadow_order: i32) -> i32 {
    shadow_order + BUBBLE_OFFSET
}

pub(super) fn text_order(shadow_order: i32) -> i32 {
    shadow_order + TEXT_OFFSET
}

pub(super) fn arrow_order(shadow_order: i32) -> i32 {
    shadow_order + ARROW_OFFSET
}

pub(super) fn icon_order(shadow_order: i32) -> i32 {
    shadow_order + ICON_OFFSET
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn body_order(
    title_order: i32,
) -> i32 {
    title_order + BODY_OFFSET
}

#[cfg(test)]
mod tests {
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
}
