use zircon_runtime_interface::ui::surface::{UiTextRange, UiTextRunKind};

use super::super::candidate_line::{append_segment, insert_virtual_text, CandidateLine};
use super::super::virtual_fragment_sequence::capture;
use super::apply_visual_order_with_virtual_sequence;

#[test]
fn virtual_tatweel_uses_display_bidi_order_before_the_line_is_materialized() {
    let mut line = CandidateLine::empty();
    append_segment(
        &mut line,
        UiTextRunKind::Plain,
        "سلام",
        UiTextRange { start: 0, end: 8 },
    );
    assert!(insert_virtual_text(&mut line, 2, "ـ"));
    let mut sequence = capture(
        &line,
        zircon_runtime_interface::ui::surface::UiTextDirection::RightToLeft,
    )
    .expect("virtual source anchor retains the logical sidecar");

    apply_visual_order_with_virtual_sequence(
        &mut line,
        zircon_runtime_interface::ui::surface::UiTextDirection::RightToLeft,
        Some(&mut sequence),
        None,
    )
    .expect("display-owned UAX#9 accepts zero-width source anchors");

    assert_eq!(line.text, "مالـس");
    assert!(line.runs.iter().any(|run| {
        run.text == "ـ" && run.source_range.start == 2 && run.source_range.end == 2
    }));
}
