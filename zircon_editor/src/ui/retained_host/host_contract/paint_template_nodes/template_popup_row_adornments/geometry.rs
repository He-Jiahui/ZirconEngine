use super::super::super::data::FrameRect;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn popup_row_adornment_rect(
    row_rect: &FrameRect,
    clip: &FrameRect,
) -> Option<FrameRect> {
    if !has_paintable_extent(row_rect) || !has_paintable_extent(clip) {
        return None;
    }
    let metrics = super::super::template_popup_rows::metrics::workbench_popup_row_metrics();
    let rect = FrameRect {
        x: row_rect.x + row_rect.width - metrics.adornment_right - metrics.adornment_size,
        y: row_rect.y + (row_rect.height - metrics.adornment_size).max(0.0) * 0.5,
        width: metrics.adornment_size,
        height: metrics.adornment_size,
    };
    (has_paintable_extent(&rect)
        && frame_is_within(row_rect, &rect)
        && frame_is_within(clip, &rect))
    .then_some(rect)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn local_rect(
    origin: &FrameRect,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> FrameRect {
    FrameRect {
        x: origin.x + x,
        y: origin.y + y,
        width,
        height,
    }
}

fn has_paintable_extent(rect: &FrameRect) -> bool {
    rect.x.is_finite()
        && rect.y.is_finite()
        && rect.width.is_finite()
        && rect.height.is_finite()
        && rect.width > 0.0
        && rect.height > 0.0
}

fn frame_is_within(container: &FrameRect, rect: &FrameRect) -> bool {
    rect.x >= container.x
        && rect.y >= container.y
        && rect.x + rect.width <= container.x + container.width
        && rect.y + rect.height <= container.y + container.height
}

#[cfg(test)]
mod tests {
    use super::popup_row_adornment_rect;
    use crate::ui::retained_host::host_contract::data::FrameRect;
    use crate::ui::retained_host::host_contract::paint_template_nodes::template_popup_rows::metrics::workbench_popup_row_metrics;

    #[test]
    fn popup_row_adornment_requires_a_fully_paintable_row_and_clip() {
        let metrics = workbench_popup_row_metrics();
        let minimum_width = metrics.adornment_right + metrics.adornment_size;
        let full_row = FrameRect {
            x: 10.0,
            y: 20.0,
            width: minimum_width + 1.0,
            height: metrics.adornment_size,
        };

        assert!(popup_row_adornment_rect(&full_row, &full_row).is_some());
        assert!(
            popup_row_adornment_rect(
                &FrameRect {
                    width: minimum_width - 0.1,
                    ..full_row.clone()
                },
                &full_row,
            )
            .is_none()
        );
        assert!(
            popup_row_adornment_rect(
                &FrameRect {
                    height: metrics.adornment_size - 0.1,
                    ..full_row.clone()
                },
                &full_row,
            )
            .is_none()
        );
        assert!(
            popup_row_adornment_rect(
                &full_row,
                &FrameRect {
                    width: minimum_width - 0.1,
                    ..full_row.clone()
                },
            )
            .is_none()
        );
        assert!(
            popup_row_adornment_rect(
                &FrameRect {
                    x: f32::NAN,
                    ..full_row.clone()
                },
                &full_row,
            )
            .is_none()
        );
    }
}
