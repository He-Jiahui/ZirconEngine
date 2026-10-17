//! 文本输入的纯清洗层：按节点策略过滤替换文本，并重映射 IME 预编辑元数据。
//!
//! 键盘、Text/IME、剪贴板及无障碍入口共用这些拥有的返回值；此模块不提交 Surface 或文档事务。
//! 长度预算按 grapheme 计算，范围与映射使用 UTF-8 字节；文档索引的时效由调用方同步保障。

use unicode_segmentation::UnicodeSegmentation;
use zircon_runtime_interface::ui::{
    dispatch::{UiImePreeditClause, UiTextByteRange, UiTextInputConstraintReceipt},
    event_ui::UiNodeId,
    surface::UiTextRange,
    tree::UiTemplateNodeMetadata,
};

use super::super::surface::UiSurface;
use crate::ui::text::clamp_grapheme_boundary;

use self::preedit_mapping::{
    remap_preedit_clauses, remap_preedit_cursor_range, TextInputBoundaryMap,
};

#[cfg(test)]
#[path = "text_constraints/tests/optimization_tests.rs"]
mod optimization_tests;
#[path = "text_constraints/preedit_mapping.rs"]
mod preedit_mapping;
#[cfg(test)]
#[path = "text_constraints/tests/cases.rs"]
mod tests;

// 使用固定名称区分文档索引与后备扫描，避免把输入内容放入 profiling 名称。
const TEXT_INPUT_GRAPHEME_AUTHORITY_COUNTER_NAMES: [&str; 3] = [
    "text_input_grapheme_document_index_count",
    "text_input_grapheme_source_scan_count",
    "text_input_grapheme_source_scan_bytes",
];

/// 替换范围之外的 grapheme 数的来源，只在存在长度上限时消费。
/// 调用方提供索引值时须保证它属于当前文档与本次替换范围；此层不会重新校验版本。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum TextInputRetainedGraphemeCount {
    /// 由已同步文档索引计算的保留数量，可跳过对当前文本的后备扫描。
    DocumentIndex(usize),
    #[default]
    /// 根据当前文本与替换范围扫描两侧保留片段。
    SourceScan,
}

/// 清洗后的替换值及本次约束回执；调用方随后决定是否提交编辑。
pub(crate) struct SanitizedTextInputReplacement {
    /// 拥有的替换文本，生命周期不再依赖输入切片。
    pub(crate) text: String,
    /// 记录移除或截断的事实，不代表后续文本事务已经成功。
    pub(crate) receipt: UiTextInputConstraintReceipt,
}

/// 预编辑清洗结果，所有偏移均相对返回的预编辑文本。
pub(crate) struct SanitizedTextInputPreedit {
    /// 清洗后拥有的预编辑文本。
    pub(crate) text: String,
    /// 已映射并吸附到 grapheme 边界的光标范围，无法映射时为空。
    pub(crate) cursor_range: Option<UiTextByteRange>,
    /// 保留原顺序的可映射样式区段，不额外吸附到 grapheme 边界。
    pub(crate) preedit_clauses: Vec<UiImePreeditClause>,
    /// 包含文本清洗及光标/区段调整的累计回执。
    pub(crate) receipt: UiTextInputConstraintReceipt,
}

impl SanitizedTextInputReplacement {
    /// 仅在约束实际产生回执标志或计数时返回副本，供上层省略空诊断。
    pub(crate) fn receipt_if_changed(&self) -> Option<UiTextInputConstraintReceipt> {
        (!self.receipt.is_empty()).then_some(self.receipt)
    }
}

/// 一次节点策略快照，可复制到各输入入口；默认允许原样多行文本且无长度上限。
///
/// # 使用与生命周期
/// 由节点元数据生成后用于本次清洗，返回值不持有节点或元数据借用。
/// 清洗只生成值与回执；提交、只读检查和安全文本策略由调用入口负责。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TextInputConstraints {
    /// 替换后允许的总 grapheme 数；先扣除替换范围之外的保留数量。
    max_graphemes: Option<usize>,
    /// 对每个 Unicode scalar 的许可规则，不承担完整数值语法解析。
    filter: TextInputFilter,
    /// 为 false 时移除硬换行分隔符；CRLF 计为一个分隔符。
    multiline: bool,
}

impl Default for TextInputConstraints {
    fn default() -> Self {
        Self {
            max_graphemes: None,
            filter: TextInputFilter::Any,
            multiline: true,
        }
    }
}

impl TextInputConstraints {
    /// 键盘 Enter 分支据此选择插入换行或触发提交，此查询本身不改变状态。
    pub(crate) fn allows_multiline(self) -> bool {
        self.multiline
    }

    /// 使用源码扫描作为长度预算来源，生成拥有的替换文本与约束回执。
    /// `replaced_range` 是当前文本的 UTF-8 字节范围；无长度约束时不需要扫描保留片段。
    pub(crate) fn sanitize_replacement(
        self,
        current_text: &str,
        replaced_range: UiTextRange,
        replacement: &str,
    ) -> SanitizedTextInputReplacement {
        self.sanitize_replacement_with_boundary_map(
            current_text,
            replaced_range,
            replacement,
            TextInputRetainedGraphemeCount::SourceScan,
            None,
        )
    }

    /// 接受调用方选定的保留数量来源，供同步文档索引的输入路径复用。
    ///
    /// # 参数与返回
    /// 文本切片只借用于本次调用，字节范围指向当前文本；返回清洗后的独立拥有值。
    /// `retained_graphemes` 的索引值须与该文本及范围一致，版本检查由调用方负责。
    pub(crate) fn sanitize_replacement_with_retained_grapheme_count(
        self,
        current_text: &str,
        replaced_range: UiTextRange,
        replacement: &str,
        retained_graphemes: TextInputRetainedGraphemeCount,
    ) -> SanitizedTextInputReplacement {
        self.sanitize_replacement_with_boundary_map(
            current_text,
            replaced_range,
            replacement,
            retained_graphemes,
            None,
        )
    }

    /// 清洗预编辑文本并映射其局部字节范围，长度预算使用后备源码扫描。
    /// 调用方须先验证光标与样式区段是源预编辑文本内的合法 UTF-8 范围。
    pub(crate) fn sanitize_preedit_replacement(
        self,
        current_text: &str,
        replaced_range: UiTextRange,
        replacement: &str,
        cursor_range: Option<UiTextByteRange>,
        preedit_clauses: &[UiImePreeditClause],
    ) -> SanitizedTextInputPreedit {
        self.sanitize_preedit_replacement_with_retained_grapheme_count(
            current_text,
            replaced_range,
            replacement,
            cursor_range,
            preedit_clauses,
            TextInputRetainedGraphemeCount::SourceScan,
        )
    }

    /// 预编辑路径的完整清洗入口，同时消费文档保留数量与平台引用的少量边界。
    ///
    /// # 约束与返回
    /// 范围先由 IME 入口验证；映射表跟随过滤过程记录源/输出字节对应，再按截断长度夹住。
    /// 返回拥有的文本、光标、样式区段与回执；光标另吸附到 grapheme 边界，区段不做该吸附。
    pub(crate) fn sanitize_preedit_replacement_with_retained_grapheme_count(
        self,
        current_text: &str,
        replaced_range: UiTextRange,
        replacement: &str,
        cursor_range: Option<UiTextByteRange>,
        preedit_clauses: &[UiImePreeditClause],
        retained_graphemes: TextInputRetainedGraphemeCount,
    ) -> SanitizedTextInputPreedit {
        let mut boundary_map = TextInputBoundaryMap::new(cursor_range, preedit_clauses);
        let SanitizedTextInputReplacement {
            text,
            receipt: mut receipt,
        } = self.sanitize_replacement_with_boundary_map(
            current_text,
            replaced_range,
            replacement,
            retained_graphemes,
            Some(&mut boundary_map),
        );
        let cursor_range =
            remap_preedit_cursor_range(&boundary_map, &text, cursor_range, &mut receipt);
        let preedit_clauses = remap_preedit_clauses(&boundary_map, preedit_clauses, &mut receipt);
        SanitizedTextInputPreedit {
            text,
            cursor_range,
            preedit_clauses,
            receipt,
        }
    }

    /// 各清洗入口的共同实现；可选映射表只在本次扫描期间独占借用。
    fn sanitize_replacement_with_boundary_map(
        self,
        current_text: &str,
        replaced_range: UiTextRange,
        replacement: &str,
        retained_graphemes: TextInputRetainedGraphemeCount,
        mut boundary_map: Option<&mut TextInputBoundaryMap>,
    ) -> SanitizedTextInputReplacement {
        if self.max_graphemes.is_none()
            && self.filter == TextInputFilter::Any
            && self.multiline
            && boundary_map.is_none()
        {
            // Without constraints or requested IME offsets, the owned result is byte-identical.
            return SanitizedTextInputReplacement {
                text: replacement.to_owned(),
                receipt: UiTextInputConstraintReceipt::default(),
            };
        }
        let mut receipt = UiTextInputConstraintReceipt::default();
        let mut filtered = String::with_capacity(replacement.len());
        let mut characters = replacement.char_indices().peekable();
        if let Some(boundary_map) = boundary_map.as_deref_mut() {
            boundary_map.record(0, 0);
        }
        while let Some((offset, character)) = characters.next() {
            let character_end = offset + character.len_utf8();
            if !self.multiline && crate::text::is_hard_line_separator(character) {
                receipt.removed_hard_line_count = receipt.removed_hard_line_count.saturating_add(1);
                if let Some(boundary_map) = boundary_map.as_deref_mut() {
                    boundary_map.record(character_end, filtered.len());
                }
                if character == '\r'
                    && characters
                        .peek()
                        .is_some_and(|(_, next_character)| *next_character == '\n')
                {
                    let (line_feed_offset, line_feed) = characters.next().unwrap();
                    if let Some(boundary_map) = boundary_map.as_deref_mut() {
                        boundary_map
                            .record(line_feed_offset + line_feed.len_utf8(), filtered.len());
                    }
                }
                continue;
            }
            if !self.filter.accepts(character) {
                receipt.removed_filter_scalar_count =
                    receipt.removed_filter_scalar_count.saturating_add(1);
                if let Some(boundary_map) = boundary_map.as_deref_mut() {
                    boundary_map.record(character_end, filtered.len());
                }
                continue;
            }
            filtered.push(character);
            if let Some(boundary_map) = boundary_map.as_deref_mut() {
                boundary_map.record(character_end, filtered.len());
            }
        }

        if let Some(max_graphemes) = self.max_graphemes {
            // 保留数量决定还能插入多少 grapheme；索引分支直接信任调用方已同步的计数。
            let retained = match retained_graphemes {
                TextInputRetainedGraphemeCount::DocumentIndex(retained) => {
                    crate::profile_counter!(
                        "runtime",
                        TEXT_INPUT_GRAPHEME_AUTHORITY_COUNTER_NAMES[0],
                        1
                    );
                    retained
                }
                TextInputRetainedGraphemeCount::SourceScan => {
                    let (retained, scanned_bytes) =
                        retained_grapheme_count_from_source(current_text, replaced_range);
                    crate::profile_counter!(
                        "runtime",
                        TEXT_INPUT_GRAPHEME_AUTHORITY_COUNTER_NAMES[1],
                        1
                    );
                    crate::profile_counter!(
                        "runtime",
                        TEXT_INPUT_GRAPHEME_AUTHORITY_COUNTER_NAMES[2],
                        scanned_bytes
                    );
                    retained
                }
            };
            let available = max_graphemes.saturating_sub(retained);
            if let Some((truncate_at, _)) = filtered.grapheme_indices(true).nth(available) {
                filtered.truncate(truncate_at);
                receipt.max_graphemes_truncated = true;
            }
        }
        if let Some(boundary_map) = boundary_map {
            boundary_map.clamp_output(filtered.len());
        }
        SanitizedTextInputReplacement {
            text: filtered,
            receipt,
        }
    }

    /// 上层仅在确有长度上限时向文档索引请求保留数量，避免无约束输入的额外查询。
    pub(crate) const fn requires_retained_grapheme_count(self) -> bool {
        self.max_graphemes.is_some()
    }
}

/// 标量过滤规则；Number 只限制候选字符，完整数值合法性仍由 NumberField 提交路径判断。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum TextInputFilter {
    #[default]
    /// 不按字符种类过滤。
    Any,
    /// 仅 ASCII 数字。
    Digits,
    /// ASCII 数字及小数点、符号和指数标记字符。
    Number,
    /// 任意 ASCII 字符；换行是否移除另由 multiline 控制。
    Ascii,
    /// Unicode 字母或数字。
    Alphanumeric,
}

impl TextInputFilter {
    /// 元数据别名使用借用比较；未知名称回退为不限制字符种类。
    fn from_token(value: &str) -> Self {
        if normalized_constraint_matches(value, &["digits", "digit", "numericdigits"]) {
            Self::Digits
        } else if normalized_constraint_matches(value, &["number", "numeric", "decimal"]) {
            Self::Number
        } else if normalized_constraint_matches(value, &["ascii"]) {
            Self::Ascii
        } else if normalized_constraint_matches(value, &["alphanumeric", "alnum"]) {
            Self::Alphanumeric
        } else {
            Self::Any
        }
    }

    fn accepts(self, ch: char) -> bool {
        match self {
            Self::Any => true,
            Self::Digits => ch.is_ascii_digit(),
            Self::Number => ch.is_ascii_digit() || matches!(ch, '.' | '-' | '+' | 'e' | 'E'),
            Self::Ascii => ch.is_ascii(),
            Self::Alphanumeric => ch.is_alphanumeric(),
        }
    }
}

/// 从目标节点元数据取得本次输入策略，缺少元数据时返回默认值。
///
/// # 约束
/// 长度别名按声明顺序取首个正值；NumberField 还以编辑预算常量夹住上限，长度仍按 grapheme 执行。
/// 显式过滤与多行配置优先于组件默认值；返回快照不保留 Surface 借用。
pub(crate) fn text_input_constraints_for_node(
    surface: &UiSurface,
    target: UiNodeId,
) -> TextInputConstraints {
    let Some(metadata) = surface
        .tree
        .nodes
        .get(&target)
        .and_then(|node| node.template_metadata.as_ref())
    else {
        return TextInputConstraints::default();
    };
    let configured_max_graphemes = positive_usize_attribute(metadata, "max_graphemes")
        .or_else(|| positive_usize_attribute(metadata, "max_chars"))
        .or_else(|| positive_usize_attribute(metadata, "max_length"));
    let max_graphemes = if metadata.component == "NumberField" {
        Some(
            configured_max_graphemes
                .unwrap_or(super::number_field::MVP_MAX_NUMBER_FIELD_EDIT_BYTES)
                .min(super::number_field::MVP_MAX_NUMBER_FIELD_EDIT_BYTES),
        )
    } else {
        configured_max_graphemes
    };
    TextInputConstraints {
        max_graphemes,
        filter: metadata
            .attributes
            .get("input_filter")
            .or_else(|| metadata.attributes.get("text_filter"))
            .and_then(toml::Value::as_str)
            .map(TextInputFilter::from_token)
            .unwrap_or_else(|| {
                if metadata.component == "NumberField" {
                    TextInputFilter::Number
                } else {
                    TextInputFilter::Any
                }
            }),
        multiline: bool_attribute(metadata, "multiline")
            .unwrap_or(metadata.component != "NumberField"),
    }
}

/// 将替换端点向前夹到 grapheme 边界后扫描两侧，返回保留数量及实际扫描的字节数。
fn retained_grapheme_count_from_source(text: &str, replaced_range: UiTextRange) -> (usize, usize) {
    let start = clamp_grapheme_boundary(text, replaced_range.start);
    let end = clamp_grapheme_boundary(text, replaced_range.end).max(start);
    (
        text[..start].graphemes(true).count() + text[end..].graphemes(true).count(),
        start.saturating_add(text.len().saturating_sub(end)),
    )
}

fn usize_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<usize> {
    metadata.attributes.get(key).and_then(|value| match value {
        toml::Value::Integer(value) => (*value >= 0).then_some(*value as usize),
        toml::Value::Float(value) if value.is_finite() && *value >= 0.0 => Some(*value as usize),
        _ => None,
    })
}

// 零值不表示禁止输入；它不形成长度上限，让后续别名或组件默认值继续决定策略。
fn positive_usize_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<usize> {
    usize_attribute(metadata, key).filter(|value| *value > 0)
}

fn bool_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<bool> {
    metadata.attributes.get(key).and_then(toml::Value::as_bool)
}

/// 忽略 ASCII 空白、下划线与连字符并折叠 ASCII 大小写，不分配中间规范化字符串。
fn normalized_constraint_matches(value: &str, expected: &[&str]) -> bool {
    expected.iter().any(|expected| {
        value
            .bytes()
            .filter(|byte| !matches!(byte, b'_' | b'-') && !byte.is_ascii_whitespace())
            .map(|byte| byte.to_ascii_lowercase())
            .eq(expected.bytes())
    })
}
