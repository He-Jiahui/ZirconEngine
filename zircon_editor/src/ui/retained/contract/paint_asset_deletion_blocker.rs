use super::data::{HostAssetDeletionBlockerData, HostWindowPresentationData};
use super::paint_close_prompt::button::draw_prompt_button;
use super::paint_close_prompt::colors::close_prompt_palette;
use super::paint_frame::HostRgbaFrame;
use super::paint_primitives::{draw_border, draw_rect, draw_text_bars_clipped};
use super::paint_theme::{current_host_metrics, current_host_palette};

const TITLE: &str = "Delete blocked";
const MESSAGE: &str = "This asset is referenced by other project assets.";

pub(in crate::ui::retained_host::host_contract) fn draw_asset_deletion_blocker(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
) {
    let blocker = &presentation.asset_deletion_blocker;
    if !blocker.visible {
        return;
    }
    let palette = close_prompt_palette(current_host_palette());
    let metrics = current_host_metrics();
    let inset = metrics.gap_l + metrics.gap_s;
    let title_x = blocker.dialog_frame.x + inset;
    let title_y = blocker.dialog_frame.y + inset;
    let line_height = metrics
        .line_height(metrics.font_body)
        .round()
        .max(metrics.font_body.ceil());

    draw_rect(frame, blocker.overlay_frame.clone(), palette.overlay);
    draw_rect(frame, blocker.dialog_frame.clone(), palette.dialog);
    draw_border(frame, blocker.dialog_frame.clone(), palette.warning);
    draw_text_bars_clipped(
        frame,
        title_x,
        title_y,
        TITLE,
        Some(&blocker.dialog_frame),
        palette.text,
    );
    draw_text_bars_clipped(
        frame,
        title_x,
        title_y + line_height + metrics.gap_s,
        MESSAGE,
        Some(&blocker.dialog_frame),
        palette.text_muted,
    );
    draw_text_bars_clipped(
        frame,
        title_x,
        title_y + (line_height + metrics.gap_s) * 2.0,
        &blocker.target,
        Some(&blocker.dialog_frame),
        palette.warning,
    );
    draw_rect(
        frame,
        blocker.referencer_list_frame.clone(),
        palette.dialog_inset,
    );

    draw_visible_referencers(frame, blocker, palette.text_muted);
    draw_prompt_button(frame, &blocker.close_button_frame, "Close", true, palette);
}

fn draw_visible_referencers(
    frame: &mut HostRgbaFrame,
    blocker: &HostAssetDeletionBlockerData,
    color: [u8; 4],
) {
    let row_height = HostAssetDeletionBlockerData::referencer_row_height();
    let row_inset = current_host_metrics().gap_s;
    let total = blocker.referencers.row_count();
    let visible = blocker.visible_referencer_rows.min(total);
    let text_rows = if total > visible {
        visible.saturating_sub(1)
    } else {
        visible
    };
    for (row, referencer) in blocker.referencers.iter().take(text_rows).enumerate() {
        draw_text_bars_clipped(
            frame,
            blocker.referencer_list_frame.x + row_inset,
            blocker.referencer_list_frame.y + row as f32 * row_height + row_inset,
            referencer,
            Some(&blocker.referencer_list_frame),
            color,
        );
    }
    if total > visible && visible > 0 {
        draw_text_bars_clipped(
            frame,
            blocker.referencer_list_frame.x + row_inset,
            blocker.referencer_list_frame.y + text_rows as f32 * row_height + row_inset,
            &blocker.overflow_label,
            Some(&blocker.referencer_list_frame),
            color,
        );
    }
}
