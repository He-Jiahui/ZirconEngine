//! 记录富文本表格布局阶段的工作量，并按性能采集配置发布报告副本。
//! 阶段计数可以先于后续失败发生；是否采用布局结果由调用方决定。

/// 一次布局会话对富文本表格各阶段的工作量回执，供诊断与性能画像读取。
/// 计数是观测值而非准入预算；溢出饱和避免遥测改变布局控制流。
/// 会话的 `begin_frame` 重置报告；`finish_frame` 发布当前副本但不清零，`clear` 只清缓存。
/// 各累计项按调用次数或传入数量饱和累加，`max_table_cell_count` 取单次最大值。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct TextTableLayoutWorkReport {
    pub(crate) table_layout_attempt_count: usize,
    pub(crate) table_source_byte_count: usize,
    pub(crate) table_cell_count: usize,
    pub(crate) max_table_cell_count: usize,
    pub(crate) preferred_cell_layout_count: usize,
    pub(crate) preferred_cell_input_bytes: usize,
    pub(crate) final_cell_layout_count: usize,
    pub(crate) final_cell_input_bytes: usize,
    pub(crate) column_track_count: usize,
    pub(crate) row_track_count: usize,
    pub(crate) published_line_count: usize,
    pub(crate) published_box_count: usize,
}

impl TextTableLayoutWorkReport {
    /// 在表格网格建立后记录一次尝试，后续布局仍可能失败。
    ///
    /// # 参数
    /// `source_bytes` 是调用方提供的本次表格源范围 UTF-8 字节数；`cell_count` 是该网格单元格数。
    pub(crate) fn record_layout_attempt(&mut self, source_bytes: usize, cell_count: usize) {
        self.table_layout_attempt_count = self.table_layout_attempt_count.saturating_add(1);
        self.table_source_byte_count = self.table_source_byte_count.saturating_add(source_bytes);
        self.table_cell_count = self.table_cell_count.saturating_add(cell_count);
        self.max_table_cell_count = self.max_table_cell_count.max(cell_count);
    }

    /// 累计本次求得的列、行轨道数；此时后续单元格排布仍可能失败。
    pub(crate) fn record_tracks(&mut self, column_count: usize, row_count: usize) {
        self.column_track_count = self.column_track_count.saturating_add(column_count);
        self.row_track_count = self.row_track_count.saturating_add(row_count);
    }

    /// 在单元格优选尺寸布局调用前记一次及其 UTF-8 输入字节数，包含后来失败或延期的调用。
    pub(crate) fn record_preferred_cell_layout(&mut self, source_bytes: usize) {
        self.preferred_cell_layout_count = self.preferred_cell_layout_count.saturating_add(1);
        self.preferred_cell_input_bytes =
            self.preferred_cell_input_bytes.saturating_add(source_bytes);
    }

    /// 在单元格最终尺寸布局调用前记一次及其 UTF-8 输入字节数，包含后来失败或延期的调用。
    pub(crate) fn record_final_cell_layout(&mut self, source_bytes: usize) {
        self.final_cell_layout_count = self.final_cell_layout_count.saturating_add(1);
        self.final_cell_input_bytes = self.final_cell_input_bytes.saturating_add(source_bytes);
    }

    /// 由调用方在本地几何检查通过后，记录拟交给 Ready 结果的行与盒数量。
    /// 这些数量不证明后续合成或屏幕呈现已经完成。
    pub(crate) fn record_output(&mut self, line_count: usize, box_count: usize) {
        self.published_line_count = self.published_line_count.saturating_add(line_count);
        self.published_box_count = self.published_box_count.saturating_add(box_count);
    }

    /// 将当前副本交给条件编译的性能计数宏，不清空原报告。
    ///
    /// # 约束与生命周期
    /// 本类型为 `Copy`；按值调用只消费副本。宏按功能开关及当前采集状态决定是否上报，数量会转换为 `f64`。
    /// 未启用相关功能时仅丢弃副本；参见 `SharedTextLayoutSession::finish_frame` 与 `profile_counter!`。
    pub(crate) fn publish_profile_counters(self) {
        #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
        {
            crate::profile_counter!(
                "runtime",
                "rich_table_layout_attempt_count",
                self.table_layout_attempt_count
            );
            crate::profile_counter!(
                "runtime",
                "rich_table_source_byte_count",
                self.table_source_byte_count
            );
            crate::profile_counter!("runtime", "rich_table_cell_count", self.table_cell_count);
            crate::profile_counter!(
                "runtime",
                "rich_table_max_cell_count",
                self.max_table_cell_count
            );
            crate::profile_counter!(
                "runtime",
                "rich_table_preferred_cell_layout_count",
                self.preferred_cell_layout_count
            );
            crate::profile_counter!(
                "runtime",
                "rich_table_preferred_cell_input_bytes",
                self.preferred_cell_input_bytes
            );
            crate::profile_counter!(
                "runtime",
                "rich_table_final_cell_layout_count",
                self.final_cell_layout_count
            );
            crate::profile_counter!(
                "runtime",
                "rich_table_final_cell_input_bytes",
                self.final_cell_input_bytes
            );
            crate::profile_counter!(
                "runtime",
                "rich_table_column_track_count",
                self.column_track_count
            );
            crate::profile_counter!(
                "runtime",
                "rich_table_row_track_count",
                self.row_track_count
            );
            crate::profile_counter!(
                "runtime",
                "rich_table_published_line_count",
                self.published_line_count
            );
            crate::profile_counter!(
                "runtime",
                "rich_table_published_box_count",
                self.published_box_count
            );
        }
        #[cfg(not(any(feature = "profiling", feature = "profiling-tracy")))]
        let _ = self;
    }
}

#[cfg(test)]
#[path = "tests/table_work.rs"]
mod tests;
