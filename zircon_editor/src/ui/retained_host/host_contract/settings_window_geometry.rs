use super::data::FrameRect;
use super::paint_theme::HostControlMetrics;

pub(in crate::ui::retained_host::host_contract) struct SettingsWindowLayout {
    pub title: FrameRect,
    pub persistence_status: FrameRect,
    pub persistence_retry: FrameRect,
    pub sidebar: FrameRect,
    pub category_list: FrameRect,
    pub category_scrollbar_track: Option<FrameRect>,
    pub category_scrollbar_thumb: Option<FrameRect>,
    pub content_heading: FrameRect,
    pub setting_list: FrameRect,
    pub setting_scrollbar_track: Option<FrameRect>,
    pub setting_scrollbar_thumb: Option<FrameRect>,
    pub category_row_height: f32,
    pub setting_row_height: f32,
    pub row_gap: f32,
    control_size: f32,
    control_gap: f32,
    category_scroll_offset: f32,
    max_category_scroll_offset: f32,
    setting_scroll_offset: f32,
    max_setting_scroll_offset: f32,
}

impl SettingsWindowLayout {
    pub(in crate::ui::retained_host::host_contract) fn new(
        rect: &FrameRect,
        metrics: HostControlMetrics,
        requested_category_scroll_offset: f32,
        category_row_count: usize,
        requested_setting_scroll_offset: f32,
        setting_row_count: usize,
    ) -> Self {
        let header_height = metrics.control_large_height + metrics.gap_m;
        let sidebar_width = (rect.width * 0.27)
            .clamp(120.0, 240.0)
            .min(rect.width * 0.38);
        let retry_size = metrics.control_default_height;
        let persistence_retry = FrameRect {
            x: rect.x + rect.width - metrics.gap_l - retry_size,
            y: rect.y + metrics.gap_m,
            width: retry_size,
            height: retry_size,
        };
        let persistence_status_width = (rect.width * 0.34)
            .clamp(160.0, 320.0)
            .min((persistence_retry.x - rect.x - metrics.gap_l * 3.0).max(1.0));
        let persistence_status = FrameRect {
            x: persistence_retry.x - metrics.gap_m - persistence_status_width,
            y: rect.y + metrics.gap_l,
            width: persistence_status_width,
            height: metrics.line_height(metrics.font_body),
        };
        let title = FrameRect {
            x: rect.x + metrics.gap_l,
            y: rect.y + metrics.gap_l,
            width: (persistence_status.x - rect.x - metrics.gap_l * 2.0).max(1.0),
            height: metrics.line_height(metrics.font_large),
        };
        let sidebar = FrameRect {
            x: rect.x,
            y: rect.y + header_height,
            width: sidebar_width,
            height: (rect.height - header_height).max(1.0),
        };
        let category_row_height = metrics.row_height + metrics.gap_m;
        let mut category_list = FrameRect {
            x: sidebar.x + metrics.gap_m,
            y: sidebar.y + metrics.gap_m,
            width: (sidebar.width - metrics.gap_m * 2.0).max(1.0),
            height: (sidebar.height - metrics.gap_m * 2.0).max(1.0),
        };
        let category_content_extent = category_row_count as f32 * category_row_height;
        let max_category_scroll_offset = (category_content_extent - category_list.height).max(0.0);
        let category_scroll_offset = if requested_category_scroll_offset.is_finite() {
            requested_category_scroll_offset.clamp(0.0, max_category_scroll_offset)
        } else {
            0.0
        };
        let (category_scrollbar_track, category_scrollbar_thumb) =
            if max_category_scroll_offset > 0.0 {
                reserve_scrollbar(
                    &mut category_list,
                    category_content_extent,
                    category_scroll_offset,
                    metrics,
                )
            } else {
                (None, None)
            };
        let content_x = rect.x + sidebar_width + metrics.gap_l;
        let content_width = (rect.x + rect.width - content_x - metrics.gap_l).max(1.0);
        let content_heading = FrameRect {
            x: content_x,
            y: rect.y + header_height + metrics.gap_l,
            width: content_width,
            height: metrics.line_height(metrics.font_body),
        };
        let setting_list_top = content_heading.y + content_heading.height + metrics.gap_l;
        let mut setting_list = FrameRect {
            x: content_x,
            y: setting_list_top,
            width: content_width,
            height: (rect.y + rect.height - setting_list_top - metrics.gap_l).max(1.0),
        };
        let setting_row_height = metrics.row_height * 2.0 + metrics.gap_l;
        let setting_content_extent = setting_row_count as f32 * setting_row_height;
        let max_setting_scroll_offset = (setting_content_extent - setting_list.height).max(0.0);
        let setting_scroll_offset = if requested_setting_scroll_offset.is_finite() {
            requested_setting_scroll_offset.clamp(0.0, max_setting_scroll_offset)
        } else {
            0.0
        };
        let (setting_scrollbar_track, setting_scrollbar_thumb) = if max_setting_scroll_offset > 0.0
        {
            reserve_scrollbar(
                &mut setting_list,
                setting_content_extent,
                setting_scroll_offset,
                metrics,
            )
        } else {
            (None, None)
        };
        Self {
            title,
            persistence_status,
            persistence_retry,
            sidebar,
            category_list,
            category_scrollbar_track,
            category_scrollbar_thumb,
            content_heading,
            setting_list,
            setting_scrollbar_track,
            setting_scrollbar_thumb,
            category_row_height,
            setting_row_height,
            row_gap: metrics.gap_m,
            control_size: metrics.control_default_height,
            control_gap: metrics.gap_m,
            category_scroll_offset,
            max_category_scroll_offset,
            setting_scroll_offset,
            max_setting_scroll_offset,
        }
    }

    pub(in crate::ui::retained_host::host_contract) fn category_row(
        &self,
        row: usize,
    ) -> FrameRect {
        FrameRect {
            x: self.category_list.x,
            y: self.category_list.y + row as f32 * self.category_row_height
                - self.category_scroll_offset,
            width: self.category_list.width,
            height: self.category_row_height,
        }
    }

    pub(in crate::ui::retained_host::host_contract) fn category_row_index_at(
        &self,
        y: f32,
        row_count: usize,
    ) -> Option<usize> {
        row_index_at(
            &self.category_list,
            y,
            self.category_scroll_offset,
            self.category_row_height,
            row_count,
        )
    }

    pub(in crate::ui::retained_host::host_contract) fn category_scroll_offset(&self) -> f32 {
        self.category_scroll_offset
    }

    pub(in crate::ui::retained_host::host_contract) fn category_scroll_offset_for_delta(
        &self,
        delta: f32,
    ) -> f32 {
        scroll_offset_for_delta(
            self.category_scroll_offset,
            self.max_category_scroll_offset,
            delta,
        )
    }

    pub(in crate::ui::retained_host::host_contract) fn setting_row(&self, row: usize) -> FrameRect {
        FrameRect {
            x: self.setting_list.x,
            y: self.setting_list.y + row as f32 * self.setting_row_height
                - self.setting_scroll_offset,
            width: self.setting_list.width,
            height: (self.setting_row_height - self.row_gap).max(1.0),
        }
    }

    pub(in crate::ui::retained_host::host_contract) fn setting_row_index_at(
        &self,
        y: f32,
        row_count: usize,
    ) -> Option<usize> {
        row_index_at(
            &self.setting_list,
            y,
            self.setting_scroll_offset,
            self.setting_row_height,
            row_count,
        )
    }

    pub(in crate::ui::retained_host::host_contract) fn setting_scroll_offset(&self) -> f32 {
        self.setting_scroll_offset
    }

    pub(in crate::ui::retained_host::host_contract) fn max_setting_scroll_offset(&self) -> f32 {
        self.max_setting_scroll_offset
    }

    pub(in crate::ui::retained_host::host_contract) fn setting_scroll_offset_for_delta(
        &self,
        delta: f32,
    ) -> f32 {
        scroll_offset_for_delta(
            self.setting_scroll_offset,
            self.max_setting_scroll_offset,
            delta,
        )
    }

    pub(in crate::ui::retained_host::host_contract) fn setting_value_control(
        &self,
        row: usize,
        resettable: bool,
    ) -> FrameRect {
        let row = self.setting_row(row);
        let x = row.x + row.width * self.setting_value_column_ratio();
        let reset_reserve = if resettable {
            self.control_size + self.control_gap
        } else {
            0.0
        };
        FrameRect {
            x,
            y: row.y + self.control_gap,
            width: (row.x + row.width - self.control_gap - reset_reserve - x).max(1.0),
            height: self
                .control_size
                .min((row.height - self.control_gap * 2.0).max(1.0)),
        }
    }

    pub(in crate::ui::retained_host::host_contract) fn setting_text_width(
        &self,
        row: usize,
    ) -> f32 {
        let row = self.setting_row(row);
        let value_x = row.x + row.width * self.setting_value_column_ratio();
        (value_x - row.x - self.control_gap * 2.0).max(1.0)
    }

    pub(in crate::ui::retained_host::host_contract) fn setting_bool_control(
        &self,
        row: usize,
        resettable: bool,
    ) -> FrameRect {
        let value = self.setting_value_control(row, resettable);
        let size = value.height.min(value.width).max(1.0);
        FrameRect {
            x: value.x,
            y: value.y,
            width: size,
            height: size,
        }
    }

    pub(in crate::ui::retained_host::host_contract) fn setting_enum_control(
        &self,
        row: usize,
        resettable: bool,
    ) -> FrameRect {
        self.setting_value_control(row, resettable)
    }

    pub(in crate::ui::retained_host::host_contract) fn setting_color_control(
        &self,
        row: usize,
        resettable: bool,
    ) -> FrameRect {
        self.setting_value_control(row, resettable)
    }

    pub(in crate::ui::retained_host::host_contract) fn setting_numeric_decrement_control(
        &self,
        row: usize,
        resettable: bool,
    ) -> FrameRect {
        let value = self.setting_value_control(row, resettable);
        let size = numeric_step_button_size(&value);
        FrameRect {
            x: value.x,
            y: value.y,
            width: size,
            height: value.height,
        }
    }

    pub(in crate::ui::retained_host::host_contract) fn setting_numeric_increment_control(
        &self,
        row: usize,
        resettable: bool,
    ) -> FrameRect {
        let value = self.setting_value_control(row, resettable);
        let size = numeric_step_button_size(&value);
        FrameRect {
            x: value.x + value.width - size,
            y: value.y,
            width: size,
            height: value.height,
        }
    }

    pub(in crate::ui::retained_host::host_contract) fn setting_numeric_value_frame(
        &self,
        row: usize,
        resettable: bool,
    ) -> FrameRect {
        let value = self.setting_value_control(row, resettable);
        let size = numeric_step_button_size(&value);
        FrameRect {
            x: value.x + size,
            y: value.y,
            width: (value.width - size * 2.0).max(1.0),
            height: value.height,
        }
    }

    pub(in crate::ui::retained_host::host_contract) fn setting_reset_control(
        &self,
        row: usize,
    ) -> FrameRect {
        let row = self.setting_row(row);
        let size = self
            .control_size
            .min((row.height - self.control_gap * 2.0).max(1.0));
        FrameRect {
            x: row.x + row.width - self.control_gap - size,
            y: row.y + self.control_gap,
            width: size,
            height: size,
        }
    }

    fn setting_value_column_ratio(&self) -> f32 {
        if self.setting_list.width < 320.0 {
            0.52
        } else if self.setting_list.width < 480.0 {
            0.62
        } else {
            0.72
        }
    }
}

fn reserve_scrollbar(
    list: &mut FrameRect,
    content_extent: f32,
    scroll_offset: f32,
    metrics: HostControlMetrics,
) -> (Option<FrameRect>, Option<FrameRect>) {
    let thickness = metrics
        .scrollbar_thickness
        .max(metrics.border_width * 4.0)
        .min(list.width.max(0.0));
    let reserve = (thickness + metrics.gap_s).min((list.width - 1.0).max(0.0));
    list.width = (list.width - reserve).max(1.0);
    let inset = metrics.border_width.max(0.0);
    let track_height = (list.height - inset * 2.0).max(0.0);
    if thickness <= 0.0 || track_height <= 0.0 || content_extent <= list.height {
        return (None, None);
    }
    let track = FrameRect {
        x: list.x + list.width + metrics.gap_s,
        y: list.y + inset,
        width: thickness,
        height: track_height,
    };
    let proportional_thumb = track_height * (list.height / content_extent);
    let thumb_height = proportional_thumb
        .max(metrics.scrollbar_min_thumb_length)
        .min(track_height);
    let max_scroll = (content_extent - list.height).max(0.0);
    let travel = (track_height - thumb_height).max(0.0);
    let thumb_y = track.y + (scroll_offset / max_scroll) * travel;
    let thumb = FrameRect {
        x: track.x,
        y: thumb_y,
        width: track.width,
        height: thumb_height,
    };
    (Some(track), Some(thumb))
}

fn row_index_at(
    list: &FrameRect,
    y: f32,
    scroll_offset: f32,
    row_height: f32,
    row_count: usize,
) -> Option<usize> {
    if !y.is_finite() || y < list.y || y >= list.y + list.height {
        return None;
    }
    let row = ((y - list.y + scroll_offset) / row_height).floor();
    usize::try_from(row as isize)
        .ok()
        .filter(|row| *row < row_count)
}

fn scroll_offset_for_delta(current: f32, maximum: f32, delta: f32) -> f32 {
    if !delta.is_finite() {
        return current;
    }
    (current + delta).clamp(0.0, maximum)
}

fn numeric_step_button_size(value: &FrameRect) -> f32 {
    value.height.min((value.width / 3.0).max(1.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::current_host_metrics;

    fn ultra_preferences_frame() -> FrameRect {
        FrameRect {
            x: 12.0,
            y: 12.0,
            width: 396.0,
            height: 336.0,
        }
    }

    #[test]
    fn setting_scroll_projects_paint_and_hit_rows_from_one_offset_authority() {
        let frame = ultra_preferences_frame();
        let metrics = current_host_metrics();
        let initial = SettingsWindowLayout::new(&frame, metrics, 0.0, 0, 0.0, 12);
        let scrolled =
            SettingsWindowLayout::new(&frame, metrics, 0.0, 0, initial.setting_row_height, 12);

        assert_eq!(
            scrolled.setting_row_index_at(scrolled.setting_list.y, 12),
            Some(1)
        );
        assert_eq!(scrolled.setting_row(1).y, scrolled.setting_list.y);
    }

    #[test]
    fn setting_scroll_is_clamped_to_the_current_content_extent() {
        let frame = ultra_preferences_frame();
        let metrics = current_host_metrics();
        let overflow = SettingsWindowLayout::new(&frame, metrics, 0.0, 0, f32::MAX, 12);
        let no_overflow = SettingsWindowLayout::new(&frame, metrics, 0.0, 0, f32::MAX, 1);

        assert_eq!(
            overflow.setting_scroll_offset(),
            overflow.max_setting_scroll_offset()
        );
        assert!(overflow.setting_scroll_offset() > 0.0);
        assert_eq!(no_overflow.setting_scroll_offset(), 0.0);
        assert_eq!(no_overflow.setting_scroll_offset_for_delta(100.0), 0.0);
    }

    #[test]
    fn overflowing_settings_reserve_a_tokenized_scrollbar_and_move_the_thumb() {
        let frame = ultra_preferences_frame();
        let metrics = current_host_metrics();
        let top = SettingsWindowLayout::new(&frame, metrics, 0.0, 0, 0.0, 12);
        let bottom = SettingsWindowLayout::new(&frame, metrics, 0.0, 0, f32::MAX, 12);
        let track = top
            .setting_scrollbar_track
            .as_ref()
            .expect("overflowing settings need a visible scroll track");
        let top_thumb = top.setting_scrollbar_thumb.as_ref().unwrap();
        let bottom_thumb = bottom.setting_scrollbar_thumb.as_ref().unwrap();

        assert_eq!(track.width, metrics.scrollbar_thickness);
        assert!(top.setting_list.x + top.setting_list.width + metrics.gap_s <= track.x);
        assert_eq!(top_thumb.y, track.y);
        assert!(bottom_thumb.y > top_thumb.y);
        assert!(bottom_thumb.y + bottom_thumb.height <= track.y + track.height);

        let no_overflow = SettingsWindowLayout::new(&frame, metrics, 0.0, 0, 0.0, 1);
        assert!(no_overflow.setting_scrollbar_track.is_none());
        assert!(no_overflow.setting_scrollbar_thumb.is_none());
    }

    #[test]
    fn ultra_preferences_preserve_readable_label_and_value_columns() {
        let frame = ultra_preferences_frame();
        let metrics = current_host_metrics();
        let layout = SettingsWindowLayout::new(&frame, metrics, 0.0, 0, 0.0, 12);
        let value = layout.setting_value_control(0, true);

        assert!(layout.sidebar.width <= 120.0);
        assert!(layout.setting_text_width(0) >= 96.0);
        assert!(value.width >= metrics.control_default_height * 2.0);
        assert!(value.x >= layout.setting_list.x + layout.setting_text_width(0));
        assert!(value.x + value.width <= layout.setting_reset_control(0).x - metrics.gap_m);
    }

    #[test]
    fn category_scroll_projects_rows_and_thumb_from_the_same_offset() {
        let frame = ultra_preferences_frame();
        let metrics = current_host_metrics();
        let initial = SettingsWindowLayout::new(&frame, metrics, 0.0, 20, 0.0, 0);
        let scrolled =
            SettingsWindowLayout::new(&frame, metrics, initial.category_row_height, 20, 0.0, 0);

        assert_eq!(
            scrolled.category_row_index_at(scrolled.category_list.y, 20),
            Some(1)
        );
        assert_eq!(scrolled.category_row(1).y, scrolled.category_list.y);
        assert!(scrolled.category_scrollbar_track.is_some());
        assert!(
            scrolled.category_scrollbar_thumb.as_ref().unwrap().y
                > initial.category_scrollbar_thumb.as_ref().unwrap().y
        );
    }
}
