//! 已成形 Tatweel 候选在虚拟源锚点投影前的局部结构准入。
//!
//! 调用方提供插入后的全文、其中的 UTF-8 字节偏移和插入前行宽；本模块不生成候选或调用后端。
//! 验证读取成形数据而不修改它；成功回执只保存宽度与数量，不携带候选或字体代次身份。
//! 调用方仍须保持回执与本次候选的对应关系，并另行判断目标宽度是否适配。
//! 簇 advance 来自共享迭代器的恢复与聚合，不证明每个原始字形 advance 有限、为正或具有可见墨迹。

use crate::text::{text_glyph_clusters, FontFaceId, InstancedFaceId, ShapedGlyph, TextRange};

use super::MeasuredTextLine;

/// 固定插入文本；偏移校验按其 UTF-8 字节长度构造范围。
const ARABIC_TATWEEL: &str = "\u{0640}";
/// 整体宽度增长和插入簇聚合 advance 的比较阈值，不决定目标行宽。
const TATWEEL_ADVANCE_EPSILON: f32 = 0.01;

/// 本次验证得到的两个标量，不是可跨候选或字体状态使用的凭证。
///
/// # 使用与生命周期
/// 由 [`validate_arabic_tatweel_candidate`] 成功返回；调用方负责核对数量及后续目标宽度。
/// `Copy` 只复制宽度与数量，不持有输入借用、成形数据或字体资源，也不固定字体代次。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ArabicTatweelCandidateReceipt {
    /// 直接保留已通过有限值与增长检查的测量行宽，不重新求和簇 advance。
    width: f32,
    /// 已逐项找到独立簇并补齐右邻上下文的输入插入区间数量。
    insertion_count: usize,
}

impl ArabicTatweelCandidateReceipt {
    /// 返回本次候选的测量宽度；不重新验证输入或判定目标宽度适配。
    pub(crate) const fn width(self) -> f32 {
        self.width
    }

    /// 返回通过本次验证的插入区间数量；不重新读取候选。
    pub(crate) const fn insertion_count(self) -> usize {
        self.insertion_count
    }
}

/// 按检查执行顺序返回的首个拒绝原因。
///
/// 原因仅描述实码检查到的局部条件，不是所有成形、字体或几何问题的完整分类。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ArabicTatweelCandidateRejection {
    /// 没有需要验证的插入偏移。
    MissingInsertion,
    /// 成形全文或范围不对应候选，或者成形结果不是唯一一行。
    InvalidSourceIdentity,
    /// 起点未形成候选中有序、不交叠的 Tatweel UTF-8 字节区间。
    InvalidInsertionRange,
    /// 迭代簇源范围倒退、为空或超出候选；此项不要求范围连续覆盖。
    MalformedClusterOrder,
    /// 插入区间被簇遍历跨过，或遍历结束仍有区间未匹配。
    MissingInsertionCluster,
    /// 与插入区间相交的簇范围不恰好等于该区间。
    MixedSourceCluster,
    /// 插入簇缺少左邻，或遍历结束仍未取得它的右邻迭代簇。
    MissingJoiningContext,
    /// 簇字形切片无效或为空，或者其中存在零字形编号。
    InvalidGlyph,
    /// 簇首字形没有字体面标识；此检查不查询字体数据库。
    MissingFace,
    /// 同簇其他字形的字体面或可选实例标识不等于首字形。
    MixedClusterFace,
    /// 插入簇与邻接迭代簇的字体面或可选实例标识不同。
    JoiningFaceMismatch,
    /// 插入簇或它的邻接迭代簇未给出一致的 RTL 方向。
    NonRtlJoiningContext,
    /// 整体宽度未通过有限值或增长检查，或插入簇聚合 advance 未越过阈值。
    NonExpandingCandidate,
}

impl ArabicTatweelCandidateRejection {
    /// 返回稳定的 1 至 13 聚合编码，不携带候选身份；零仅是调用方的无拒绝约定。
    ///
    /// Stable low-cardinality code for aggregate profiling receipts. Zero means no rejection.
    pub(crate) const fn profile_code(self) -> usize {
        match self {
            Self::MissingInsertion => 1,
            Self::InvalidSourceIdentity => 2,
            Self::InvalidInsertionRange => 3,
            Self::MalformedClusterOrder => 4,
            Self::MissingInsertionCluster => 5,
            Self::MixedSourceCluster => 6,
            Self::MissingJoiningContext => 7,
            Self::InvalidGlyph => 8,
            Self::MissingFace => 9,
            Self::MixedClusterFace => 10,
            Self::JoiningFaceMismatch => 11,
            Self::NonRtlJoiningContext => 12,
            Self::NonExpandingCandidate => 13,
        }
    }
}

/// 单个迭代簇采集到的字体身份值，只用于相等比较，不解析数据库或验证代次。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ClusterFaceIdentity {
    /// 首字形提供的字体面标识；标识存在不等同于数据库中仍然有效。
    face: FontFaceId,
    /// 可选实例标识，`None` 也参与精确相等比较。
    instance: Option<InstancedFaceId>,
}

/// 仅保留前一个迭代簇的身份与方向；相邻迭代簇的源字节范围仍可能有间隙。
#[derive(Clone, Copy)]
struct PreviousCluster {
    /// 前一迭代簇的面与实例值，不持有字体资源。
    face: ClusterFaceIdentity,
    /// 簇内方向一致时保存方向；混合方向由 `None` 表示。
    right_to_left: Option<bool>,
}

/// Validates a fully shaped Tatweel candidate before UI materializes zero-width source anchors.
///
/// Unicode Joining_Type only admits a candidate. This receipt additionally proves that the
/// selected backend produced an independent, visible Tatweel cluster in the same face/instance
/// and RTL context as both neighbors. The later logical-virtual projection can therefore map the
/// cluster to a zero-width source anchor without mixing generated and source-owned content.
///
/// # 调用时机与参数
/// 在候选完成成形、调用方尚未提交虚拟锚点投影时调用。
/// - `measured`：候选的测量行；成形全文、总范围和唯一一行的源范围须对应候选全文。
/// - `candidate`：已经插入 Tatweel 的 UTF-8 全文。
/// - `insertion_offsets`：该候选中的 Tatweel 起始字节偏移，必须有序且区间不交叠，不是原文偏移。
/// - `natural_width`：调用方给出的插入前行宽，须有限；本函数不独立验证它的来源。
///
/// # 返回与约束
/// 成功返回拥有的宽度与数量；失败按执行顺序返回首个原因，不修改借用输入。
/// 簇源范围须非空、有序且落入候选，但允许间隙，不验证所有范围端点都是 UTF-8 边界。
/// 同面、同实例及 RTL 条件只约束插入簇与它的直接左右迭代簇，不要求整行同面或 RTL。
/// 检查不重做连接类型、语言或字体数据库准入，也不验证全部几何字段。
/// 宽度增长使用普通浮点加法比较；目标宽度适配由调用方另行判断。
///
/// # 借用与参考
/// 输入引用只在调用期间使用，回执不延长它们或字体资源的生命周期。
/// 簇划分及 advance 聚合见 [`text_glyph_clusters`]；恢复后的聚合值不能证明每个原始字形 advance 有限。
pub(crate) fn validate_arabic_tatweel_candidate(
    measured: &MeasuredTextLine,
    candidate: &str,
    insertion_offsets: &[usize],
    natural_width: f32,
) -> Result<ArabicTatweelCandidateReceipt, ArabicTatweelCandidateRejection> {
    // 只有插入簇及两侧 RTL 邻接簇属于同一字体身份且确实增加宽度，调用层才接受虚拟锚点投影。
    if insertion_offsets.is_empty() {
        return Err(ArabicTatweelCandidateRejection::MissingInsertion);
    }
    let shaped = measured.shaped.as_ref();
    let expected_range = TextRange {
        start: 0,
        end: candidate.len(),
    };
    let line = shaped.lines.first().filter(|_| shaped.lines.len() == 1);
    if shaped.source_text.as_ref() != candidate
        || shaped.source_range != expected_range
        || line.is_none_or(|line| line.source_range != expected_range)
    {
        return Err(ArabicTatweelCandidateRejection::InvalidSourceIdentity);
    }
    if !natural_width.is_finite()
        || !measured.metrics.width.is_finite()
        || measured.metrics.width <= natural_width + TATWEEL_ADVANCE_EPSILON
    {
        return Err(ArabicTatweelCandidateRejection::NonExpandingCandidate);
    }

    let insertion_ranges = validate_insertion_ranges(candidate, insertion_offsets)?;
    let line = line.expect("single shaped line was checked above");
    let mut insertion_index = 0_usize;
    let mut previous: Option<PreviousCluster> = None;
    // 插入簇通过后，由下一迭代簇补齐右邻检查；遍历结束仍待补齐时不能成功。
    let mut pending_right_context = None;
    let mut previous_source_end = 0_usize;

    for cluster in text_glyph_clusters(&line.glyphs) {
        if cluster.source_range.start < previous_source_end
            || cluster.source_range.start >= cluster.source_range.end
            || cluster.source_range.end > candidate.len()
        {
            return Err(ArabicTatweelCandidateRejection::MalformedClusterOrder);
        }
        let face = cluster_face_identity(&line.glyphs, cluster.glyph_start, cluster.glyph_end)?;
        if let Some(expected_face) = pending_right_context.take() {
            if face != expected_face {
                return Err(ArabicTatweelCandidateRejection::JoiningFaceMismatch);
            }
            if cluster.right_to_left != Some(true) {
                return Err(ArabicTatweelCandidateRejection::NonRtlJoiningContext);
            }
        }

        let insertion = insertion_ranges.get(insertion_index).copied();
        if insertion.is_some_and(|range| range.start < cluster.source_range.start) {
            return Err(ArabicTatweelCandidateRejection::MissingInsertionCluster);
        }
        if insertion.is_some_and(|range| ranges_overlap(range, cluster.source_range)) {
            let insertion = insertion.expect("overlap requires an insertion range");
            if cluster.source_range != insertion {
                return Err(ArabicTatweelCandidateRejection::MixedSourceCluster);
            }
            let Some(left) = previous else {
                return Err(ArabicTatweelCandidateRejection::MissingJoiningContext);
            };
            if left.face != face {
                return Err(ArabicTatweelCandidateRejection::JoiningFaceMismatch);
            }
            if left.right_to_left != Some(true) || cluster.right_to_left != Some(true) {
                return Err(ArabicTatweelCandidateRejection::NonRtlJoiningContext);
            }
            if !cluster.advance.is_finite() || cluster.advance <= TATWEEL_ADVANCE_EPSILON {
                return Err(ArabicTatweelCandidateRejection::NonExpandingCandidate);
            }
            pending_right_context = Some(face);
            insertion_index = insertion_index.saturating_add(1);
        }

        previous = Some(PreviousCluster {
            face,
            right_to_left: cluster.right_to_left,
        });
        previous_source_end = cluster.source_range.end;
    }

    if insertion_index != insertion_ranges.len() {
        return Err(ArabicTatweelCandidateRejection::MissingInsertionCluster);
    }
    if pending_right_context.is_some() {
        return Err(ArabicTatweelCandidateRejection::MissingJoiningContext);
    }
    Ok(ArabicTatweelCandidateReceipt {
        width: measured.metrics.width,
        insertion_count: insertion_ranges.len(),
    })
}

/// 将候选中的 Tatweel 起点转换为不交叠的 UTF-8 字节区间。
///
/// `candidate` 是插入后的全文，`insertion_offsets` 是其中的起点；输入借用不会保存。
/// 成功返回拥有的区间向量；加法溢出、倒序或交叠、越界及非 Tatweel 切片均拒绝。
/// 空偏移切片在本 helper 中返回空向量，由上层验证入口单独拒绝。
fn validate_insertion_ranges(
    candidate: &str,
    insertion_offsets: &[usize],
) -> Result<Vec<TextRange>, ArabicTatweelCandidateRejection> {
    let mut ranges = Vec::with_capacity(insertion_offsets.len());
    let mut previous_end = 0_usize;
    for offset in insertion_offsets.iter().copied() {
        let end = offset
            .checked_add(ARABIC_TATWEEL.len())
            .ok_or(ArabicTatweelCandidateRejection::InvalidInsertionRange)?;
        if offset < previous_end || candidate.get(offset..end) != Some(ARABIC_TATWEEL) {
            return Err(ArabicTatweelCandidateRejection::InvalidInsertionRange);
        }
        ranges.push(TextRange { start: offset, end });
        previous_end = end;
    }
    Ok(ranges)
}

/// 核对一个迭代簇的非空字形切片、非零编号及面与实例的一致性。
///
/// `start..end` 是 `glyphs` 的半开下标范围，不是源字节范围。
/// 成功返回首字形的身份副本；仅检查标识存在及同簇等值，不查询字体数据库或验证字形编号上界。
/// 输入借用只在本次检查中使用，失败返回首先遇到的局部拒绝原因。
fn cluster_face_identity(
    glyphs: &[ShapedGlyph],
    start: usize,
    end: usize,
) -> Result<ClusterFaceIdentity, ArabicTatweelCandidateRejection> {
    let cluster = glyphs
        .get(start..end)
        .filter(|cluster| !cluster.is_empty())
        .ok_or(ArabicTatweelCandidateRejection::InvalidGlyph)?;
    let first = cluster
        .first()
        .expect("non-empty cluster was checked above");
    if first.glyph_id == 0 {
        return Err(ArabicTatweelCandidateRejection::InvalidGlyph);
    }
    let face = ClusterFaceIdentity {
        face: first
            .font_id
            .ok_or(ArabicTatweelCandidateRejection::MissingFace)?,
        instance: first.font_instance_id,
    };
    for glyph in cluster.iter().skip(1) {
        if glyph.glyph_id == 0 {
            return Err(ArabicTatweelCandidateRejection::InvalidGlyph);
        }
        if glyph.font_id != Some(face.face) || glyph.font_instance_id != face.instance {
            return Err(ArabicTatweelCandidateRejection::MixedClusterFace);
        }
    }
    Ok(face)
}

/// 比较半开范围是否严格交叠；相接端点不重叠，本 helper 不校验范围顺序或来源身份。
const fn ranges_overlap(left: TextRange, right: TextRange) -> bool {
    left.start < right.end && right.start < left.end
}

#[cfg(test)]
#[path = "tests/arabic_justification.rs"]
mod tests;
