//! 将文本提供方的换行机会投影为整形簇的标志及来源回执。
//!
//! # 调用与边界
//! 水平、垂直直接后端和 Cosmic 转换路径按请求创建机会表，再以簇的文本字节范围查询。
//! 坐标始终相对传入文本；这里不决定行宽，也不替代直接后端另行记录的断行安全性。
//!
//! # 状态与生命周期
//! 表持有机会及请求的 Unicode 快照身份，不借用原文；调用方在整形期间共享借用表，
//! 查询结果按值写入簇首字形，随成形结果保留。参见 [`LineBreakOpportunityMap`]。

use unicode_linebreak::{linebreaks, BreakOpportunity};

use crate::text::{
    compiled_unicode_data_snapshot_id, LineBreakTailoringProfile, ShapedGlyphLineBreakOpportunity,
    ShapedGlyphLineBreakReceipt, UnicodeDataSnapshotId,
};

/// 单个簇从提供方取得的换行状态，以及写入字形的来源信息。
///
/// `soft_break` 与 `mandatory_break` 供后续候选分块使用；`receipt` 区分已分析无机会、
/// 提供方机会及控制符补偿。默认值的来源未知，不能代替已分析的无机会回执。
/// 值可复制；只有簇首字形通过 [`Self::receipt_for_cluster`] 保留来源。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ClusterLineBreakFlags {
    pub soft_break: bool,
    pub mandatory_break: bool,
    pub receipt: ShapedGlyphLineBreakReceipt,
}

impl ClusterLineBreakFlags {
    /// 在写入字形前限制回执的归属，并为未获提供方机会的控制簇补足来源。
    ///
    /// # 参数
    /// - `self`：该簇的提供方分析结果，按值消费。
    /// - `cluster_start`：当前字形是否为该簇的首个字形。
    /// - `mandatory_control`：调用方是否在整段簇文本中发现强制换行控制符。
    ///
    /// # 返回
    /// 非簇首返回未知默认回执；簇首已有提供方机会时保留它，只有无机会且含控制符时
    /// 返回控制符来源。返回值自持有，不借用机会表或文本。
    pub(crate) fn receipt_for_cluster(
        self,
        cluster_start: bool,
        mandatory_control: bool,
    ) -> ShapedGlyphLineBreakReceipt {
        if !cluster_start {
            return ShapedGlyphLineBreakReceipt::default();
        }
        if mandatory_control
            && matches!(
                self.receipt.opportunity,
                ShapedGlyphLineBreakOpportunity::None
            )
        {
            return ShapedGlyphLineBreakReceipt::mandatory_control();
        }
        self.receipt
    }
}

/// 保留提供方的机会类别；实际是否在此断行仍由后续布局决定。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LineBreakKind {
    Soft,
    Mandatory,
}

/// 相对分析文本起点的断点；`byte_index` 指向断点后的字符，也允许等于文本长度。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LineBreakOpportunity {
    byte_index: usize,
    kind: LineBreakKind,
}

/// 一次文本分析产生的有序机会及其版本身份。
///
/// # 约束与生命周期
/// `opportunities` 沿用提供方递增的 UTF-8 字节坐标，供区间二分定位；
/// `unicode_data_snapshot` 记录请求身份，`tailoring_profile` 标识当前使用的提供方规则。
/// 表不保存文本借用，也不随传入身份切换运行时数据表；消费者必须查询构造时同一文本的范围。
/// 三个整形路径在请求内共享此表，复制出的回执随后由成形结果持有。
#[derive(Clone, Debug)]
pub(crate) struct LineBreakOpportunityMap {
    opportunities: Vec<LineBreakOpportunity>,
    unicode_data_snapshot: UnicodeDataSnapshotId,
    tailoring_profile: LineBreakTailoringProfile,
}

impl LineBreakOpportunityMap {
    /// 用当前编译快照分析 `text`；输入只在构造期间借用，返回表拥有分析结果。
    ///
    /// 有请求快照的整形入口使用 [`Self::for_snapshot`]，以便回执沿用请求身份。
    pub(crate) fn new(text: &str) -> Self {
        Self::for_snapshot(text, compiled_unicode_data_snapshot_id())
    }

    /// 分析文本并保留调用方指定的 Unicode 快照身份。
    ///
    /// # 参数
    /// - `text`：完整请求的 UTF-8 文本，机会和后续查询都以其起点为零。
    /// - `unicode_data_snapshot`：随请求传递的身份值；实际分析仍使用编译进来的提供方。
    ///
    /// # 返回与约束
    /// 返回自持有的有序机会表，规则来源固定为 `UnicodeDefault`。提供方补出的纯文本终点
    /// 不作为内容强制换行；终点前确有换行控制符时仍保留该机会。
    /// 测试或 profiling 配置下的计数只包围本次构造，表本身不持有计时状态。
    pub(crate) fn for_snapshot(text: &str, unicode_data_snapshot: UnicodeDataSnapshotId) -> Self {
        #[cfg(any(test, feature = "profiling", feature = "profiling-tracy"))]
        let profile_started = super::analysis_profile::start_build();
        let opportunities = linebreaks(text)
            .filter_map(|(byte_index, opportunity)| match opportunity {
                BreakOpportunity::Allowed => Some(LineBreakOpportunity {
                    byte_index,
                    kind: LineBreakKind::Soft,
                }),
                BreakOpportunity::Mandatory if is_content_mandatory_break(text, byte_index) => {
                    Some(LineBreakOpportunity {
                        byte_index,
                        kind: LineBreakKind::Mandatory,
                    })
                }
                BreakOpportunity::Mandatory => None,
            })
            .collect();

        let map = Self {
            opportunities,
            unicode_data_snapshot,
            tailoring_profile: LineBreakTailoringProfile::UnicodeDefault,
        };
        #[cfg(any(test, feature = "profiling", feature = "profiling-tracy"))]
        super::analysis_profile::record_line_break_build(text.len(), profile_started);
        map
    }

    /// 返回构造时保存的请求身份，供整形入口核对分析与请求是否一致。
    pub(crate) const fn unicode_data_snapshot(&self) -> UnicodeDataSnapshotId {
        self.unicode_data_snapshot
    }

    /// 将一个簇的文本范围映射为供字形携带的机会标志。
    ///
    /// # 参数
    /// - `self`：来自同一份文本的只读分析表。
    /// - `visual_start`、`visual_end`：相对该文本起点的半开 UTF-8 字节范围端点；
    ///   名称不表示屏幕坐标，也不能传入文档绝对偏移。
    ///
    /// # 返回与约束
    /// 软机会仅在簇末端生效；强制机会位于起点之后、终点以内，排除落在左端的强制机会。
    /// 两类布尔标志独立累积，来源回执保留最后匹配的机会。
    /// 正向范围即使没有机会也带分析规则回执；反向范围返回未知默认值。
    /// 本方法只比较整数，不验证字符边界或文本长度；这些范围由整形消费者校验和构造。
    pub(crate) fn flags_for_cluster(
        &self,
        visual_start: usize,
        visual_end: usize,
    ) -> ClusterLineBreakFlags {
        if visual_start > visual_end {
            return ClusterLineBreakFlags::default();
        }
        // 先按闭区间取候选，再以下方类别规则决定归属，避免遍历与当前簇无关的机会。
        let first = self
            .opportunities
            .partition_point(|opportunity| opportunity.byte_index < visual_start);
        let end = self
            .opportunities
            .partition_point(|opportunity| opportunity.byte_index <= visual_end);
        let mut flags = ClusterLineBreakFlags {
            receipt: ShapedGlyphLineBreakReceipt {
                profile: self.tailoring_profile,
                opportunity: ShapedGlyphLineBreakOpportunity::None,
            },
            ..ClusterLineBreakFlags::default()
        };
        for opportunity in &self.opportunities[first..end] {
            match opportunity.kind {
                LineBreakKind::Soft if opportunity.byte_index == visual_end => {
                    flags.soft_break = true;
                    flags.receipt.opportunity = ShapedGlyphLineBreakOpportunity::ProviderAllowed;
                }
                LineBreakKind::Mandatory if opportunity.byte_index > visual_start => {
                    flags.mandatory_break = true;
                    flags.receipt.opportunity = ShapedGlyphLineBreakOpportunity::ProviderMandatory;
                }
                _ => {}
            }
        }
        flags
    }
}

impl Default for LineBreakOpportunityMap {
    fn default() -> Self {
        Self::new("")
    }
}

/// 识别提供方强制断点是否来自内容中的换行控制符。
///
/// `text` 是分析原文，`byte_index` 是提供方返回的断点字节偏移；检查断点前一个标量，
/// 可排除普通文本末尾的合成机会。越过末尾时取末尾，非法字符边界或空前缀返回 false。
fn is_content_mandatory_break(text: &str, byte_index: usize) -> bool {
    let preceding_text = text.get(..byte_index.min(text.len())).unwrap_or_default();
    preceding_text
        .chars()
        .next_back()
        .is_some_and(is_mandatory_break_control)
}

/// 供簇标志补偿路径检查整段 `text`，允许控制符位于多标量簇的中间。
/// 返回是否含共享控制符集合中的任一字符；不保留输入借用。
pub(super) fn contains_mandatory_break_control(text: &str) -> bool {
    text.chars().any(is_mandatory_break_control)
}

/// 内容机会过滤与簇控制符补偿共用此集合，保持两条投影路径对控制字符的识别一致。
const fn is_mandatory_break_control(ch: char) -> bool {
    matches!(
        ch,
        '\n' | '\r' | '\u{000b}' | '\u{000c}' | '\u{0085}' | '\u{2028}' | '\u{2029}'
    )
}

#[cfg(test)]
#[path = "tests/line_break.rs"]
mod tests;
