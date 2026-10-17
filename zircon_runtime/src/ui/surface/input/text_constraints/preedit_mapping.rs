//! 为文本清洗记录平台实际引用的预编辑边界，再投影光标与样式区段。
//! 表在一次清洗内创建、记录、截断夹取并消费，不保存平台或文本借用。

use zircon_runtime_interface::ui::dispatch::{
    UiImePreeditClause, UiTextByteRange, UiTextInputConstraintReceipt,
};

use crate::ui::text::clamp_grapheme_boundary;

/// Maps only platform-referenced boundaries, so memory is proportional to IME metadata, not text.
/// 端点先排序去重；调用方须先验证源范围，此表不修复非法 UTF-8 端点。
pub(super) struct TextInputBoundaryMap {
    /// 请求的源字节端点及已命中的输出字节位置。
    requested: Vec<(usize, Option<usize>)>,
    /// 下一个等待命中的端点；record 的输入位置必须单调推进。
    next_request: usize,
}

impl TextInputBoundaryMap {
    /// 只收集光标与 clauses 引用的端点，空间随元数据数量增长，而非文本长度。
    pub(super) fn new(
        cursor_range: Option<UiTextByteRange>,
        preedit_clauses: &[UiImePreeditClause],
    ) -> Self {
        let capacity = preedit_clauses.len().saturating_mul(2).saturating_add(2);
        let mut requested = Vec::with_capacity(capacity);
        if let Some(range) = cursor_range {
            requested.extend([range.start_byte as usize, range.end_byte as usize]);
        }
        for clause in preedit_clauses {
            requested.extend([
                clause.range.start_byte as usize,
                clause.range.end_byte as usize,
            ]);
        }
        requested.sort_unstable();
        requested.dedup();
        Self {
            requested: requested.into_iter().map(|offset| (offset, None)).collect(),
            next_request: 0,
        }
    }

    // TODO: [CR-UI-SURFACE-0006] 确认平台 preedit 范围是否保证落在 UTF-8 标量边界；
    // 较早的非法字节偏移无法被 record 命中，会阻住后面合法范围的映射，需补混合合法/非法端点用例。
    /// 调用方按源字节顺序提供经过过滤后的对应位置，记录所请求的少量边界。
    /// 只推进恰好命中的请求；生产 IME 入口在进入清洗前已验证范围与 UTF-8 标量边界。
    pub(super) fn record(&mut self, input_offset: usize, output_offset: usize) {
        if self
            .requested
            .get(self.next_request)
            .is_some_and(|(requested, _)| *requested == input_offset)
        {
            self.requested[self.next_request].1 = Some(output_offset);
            self.next_request += 1;
        }
    }

    /// 长度截断后夹住所有已记录的输出位置；未命中的请求仍保持不可映射。
    pub(super) fn clamp_output(&mut self, output_len: usize) {
        for (_, mapped) in &mut self.requested {
            if let Some(mapped) = mapped {
                *mapped = (*mapped).min(output_len);
            }
        }
    }

    /// 两端都命中才返回范围，并保证输出终点不早于起点。
    fn map_range(&self, range: UiTextByteRange) -> Option<UiTextByteRange> {
        let start_byte = self.map_offset(range.start_byte)?;
        let end_byte = self.map_offset(range.end_byte)?;
        Some(UiTextByteRange::new(start_byte, end_byte.max(start_byte)))
    }

    /// 查找请求端点；未记录或输出偏移无法装入协议的 u32 时返回 None。
    fn map_offset(&self, offset: u32) -> Option<u32> {
        let index = self
            .requested
            .binary_search_by_key(&(offset as usize), |(requested, _)| *requested)
            .ok()?;
        u32::try_from(self.requested[index].1?).ok()
    }
}

/// 将局部字节光标映射到清洗文本并向前吸附到 grapheme 边界。
/// 原值缺失则仍缺失；映射失败或位置变化会在回执中标记调整。
pub(super) fn remap_preedit_cursor_range(
    boundary_map: &TextInputBoundaryMap,
    text: &str,
    cursor_range: Option<UiTextByteRange>,
    receipt: &mut UiTextInputConstraintReceipt,
) -> Option<UiTextByteRange> {
    let original = cursor_range?;
    let mapped = boundary_map.map_range(original);
    let adjusted = mapped.and_then(|range| {
        let start = clamp_grapheme_boundary(text, range.start_byte as usize);
        let end = clamp_grapheme_boundary(text, range.end_byte as usize).max(start);
        Some(UiTextByteRange::new(
            u32::try_from(start).ok()?,
            u32::try_from(end).ok()?,
        ))
    });
    if adjusted != Some(original) {
        receipt.preedit_cursor_range_adjusted = true;
    }
    adjusted
}

/// 按原顺序映射样式区段；不可映射或由非空坍缩为空的区段被丢弃并计数。
/// 保留下来的范围沿用标量边界映射，不执行光标使用的 grapheme 吸附。
pub(super) fn remap_preedit_clauses(
    boundary_map: &TextInputBoundaryMap,
    clauses: &[UiImePreeditClause],
    receipt: &mut UiTextInputConstraintReceipt,
) -> Vec<UiImePreeditClause> {
    let mut mapped_clauses = Vec::with_capacity(clauses.len());
    for clause in clauses {
        let Some(range) = boundary_map.map_range(clause.range) else {
            receipt.preedit_clause_dropped_count =
                receipt.preedit_clause_dropped_count.saturating_add(1);
            continue;
        };
        if clause.range.start_byte < clause.range.end_byte && range.start_byte == range.end_byte {
            receipt.preedit_clause_dropped_count =
                receipt.preedit_clause_dropped_count.saturating_add(1);
            continue;
        }
        if range != clause.range {
            receipt.preedit_clause_range_adjusted_count = receipt
                .preedit_clause_range_adjusted_count
                .saturating_add(1);
        }
        mapped_clauses.push(UiImePreeditClause::new(range, clause.kind));
    }
    mapped_clauses
}
