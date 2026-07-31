use super::empty_state::empty_state_card_frame;
use crate::ui::retained_host::host_contract::data::FrameRect;

#[test]
fn empty_state_card_centers_inside_a_usable_pane() {
    let pane = FrameRect {
        x: 40.0,
        y: 80.0,
        width: 600.0,
        height: 360.0,
    };

    let Some(card) = empty_state_card_frame(&pane) else {
        panic!("large pane shows an empty state card");
    };

    assert!(card.width < pane.width);
    assert!(card.height < pane.height);
    assert!((card.x + card.width * 0.5 - (pane.x + pane.width * 0.5)).abs() < 0.01);
    assert!((card.y + card.height * 0.5 - (pane.y + pane.height * 0.5)).abs() < 0.01);
}

#[test]
fn empty_state_card_yields_to_the_compact_pane_title() {
    let compact_pane = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 60.0,
        height: 48.0,
    };

    assert!(empty_state_card_frame(&compact_pane).is_none());
}
