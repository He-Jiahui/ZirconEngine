use super::super::super::{
    UiResolvedTextLine, UiTextCaret, UiTextCaretAffinity, UiTextDirection, UiTextRange,
};
use super::cluster::{
    leading_source_offset, logical_end_visual_offset, logical_start_visual_offset,
    trailing_source_offset, visual_source_clusters, UiTextVisualSourceCluster,
};
use super::{UiTextVisualBoundaryBias, UiTextVisualSpan};
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicU8, Ordering},
    OnceLock,
};

#[cfg(test)]
#[path = "../../source_map/line/tests/perf.rs"]
mod performance_tests;

const EXACT_ADVANCE_PREFIX_LINEAR_QUERY_LIMIT: u8 = 2;
const SOURCE_EDGE_LINEAR_QUERY_LIMIT: u8 = 2;

/// 每个映射独有的累计 advance 索引；计数只决定何时填表，不是布局版本。
struct UiTextExactAdvanceCache {
    query_count: AtomicU8,
    prefixes: OnceLock<Vec<f32>>,
}

impl UiTextExactAdvanceCache {
    fn new() -> Self {
        Self {
            query_count: AtomicU8::new(0),
            prefixes: OnceLock::new(),
        }
    }
}

/// 按完整源区间聚合视觉包络的惰性索引，不跨映射实例共享。
struct UiTextSourceEdgeCache {
    query_count: AtomicU8,
    bounds: OnceLock<HashMap<(usize, usize), (usize, usize)>>,
}

impl UiTextSourceEdgeCache {
    fn new() -> Self {
        Self {
            query_count: AtomicU8::new(0),
            bounds: OnceLock::new(),
        }
    }
}

/// One resolved line's authoritative source-byte to visual-cluster map.
///
/// The map is built from the post-UAX#9 resolved runs, so runtime hit testing,
/// editable paint and IME geometry do not reconstruct direction or cluster
/// ownership independently.
///
/// # 职责与生命周期
/// 借用解析行，拥有投影后的簇表及每实例缓存；构造仍会分配簇表。
/// 源与视觉位置使用字节坐标，命中回映另用簇边界索引；本类型不重新整形文本。
/// # 参考
/// [`Self::caret_for_visual_boundary`]、[`Self::visual_offset_for_caret`]。
pub struct UiTextLineSourceMap<'a> {
    line: &'a UiResolvedTextLine,
    clusters: Vec<UiTextVisualSourceCluster>,
    // 该条件只准许源范围二分路径，不是整个 DTO 已通过校验的证明。
    source_ranges_are_disjoint_and_ordered: bool,
    source_edge_cache: Option<UiTextSourceEdgeCache>,
    exact_advance_cache: Option<UiTextExactAdvanceCache>,
}

impl<'a> UiTextLineSourceMap<'a> {
    /// 建立视觉簇表，并按源区间顺序和 advance 条件选择查询路径。
    ///
    /// # 参数与返回
    /// `line` 是已解析的视觉行；返回借用该行、拥有簇表与惰性索引的映射。
    /// # 约束
    /// 不重新整形，也不验证 DTO 的全部区间、UTF-8 或算术边界。
    pub fn new(line: &'a UiResolvedTextLine) -> Self {
        let clusters = visual_source_clusters(line);
        let source_ranges_are_disjoint_and_ordered = clusters
            .windows(2)
            .all(|pair| pair[0].source_range.end <= pair[1].source_range.start);
        let exact_advance_cache = (line.glyph_advances.len() == clusters.len()
            && line
                .glyph_advances
                .iter()
                .all(|advance| advance.is_finite() && *advance >= 0.0))
        .then(UiTextExactAdvanceCache::new);
        let source_edge_cache =
            (!source_ranges_are_disjoint_and_ordered).then(UiTextSourceEdgeCache::new);
        Self {
            line,
            clusters,
            source_ranges_are_disjoint_and_ordered,
            source_edge_cache,
            exact_advance_cache,
        }
    }

    /// 返回成功投影的视觉簇数，不是源文本或视觉文本的字节长度。
    pub fn cluster_count(&self) -> usize {
        self.clusters.len()
    }

    /// 将源字节光标及其亲和性映射为视觉字节偏移。
    ///
    /// # 参数与返回
    /// `caret.offset` 在源区间坐标中；返回值在解析行的视觉字节坐标中。
    /// # 约束
    /// 源范围有序且不重叠时走二分，否则扫描；簇内部按亲和性收拢到逻辑边缘。
    /// 与 [`Self::caret_for_visual_boundary`] 接收的簇边界索引区分使用。
    pub fn visual_offset_for_caret(&self, caret: &UiTextCaret) -> usize {
        if self.clusters.is_empty() {
            return self.line.visual_range.start;
        }

        if self.source_ranges_are_disjoint_and_ordered {
            return self.visual_offset_for_ordered_caret(caret);
        }

        self.visual_offset_for_caret_linear(caret)
    }

    // 仅供源范围按视觉顺序分离的簇表，先找亲和性对应的精确边缘再处理簇内部。
    fn visual_offset_for_ordered_caret(&self, caret: &UiTextCaret) -> usize {
        let exact = match caret.affinity {
            UiTextCaretAffinity::Upstream => {
                let end = self
                    .clusters
                    .partition_point(|cluster| cluster.source_range.end <= caret.offset);
                end.checked_sub(1)
                    .and_then(|index| self.clusters.get(index))
                    .filter(|cluster| cluster.source_range.end == caret.offset)
                    .map(|cluster| unique_source_edge_visual_offset(cluster, false))
            }
            UiTextCaretAffinity::Downstream => {
                let start = self
                    .clusters
                    .partition_point(|cluster| cluster.source_range.start < caret.offset);
                self.clusters
                    .get(start)
                    .filter(|cluster| cluster.source_range.start == caret.offset)
                    .map(|cluster| unique_source_edge_visual_offset(cluster, true))
            }
        };
        if let Some(offset) = exact {
            return offset;
        }

        let end = self
            .clusters
            .partition_point(|cluster| cluster.source_range.start < caret.offset);
        if let Some(cluster) = end
            .checked_sub(1)
            .and_then(|index| self.clusters.get(index))
            .filter(|cluster| caret.offset < cluster.source_range.end)
        {
            return match caret.affinity {
                UiTextCaretAffinity::Upstream => unique_source_edge_visual_offset(cluster, true),
                UiTextCaretAffinity::Downstream => unique_source_edge_visual_offset(cluster, false),
            };
        }

        if caret.offset <= self.line.source_range.start {
            return self
                .clusters
                .first()
                .map(logical_start_visual_offset)
                .unwrap_or(self.line.visual_range.start);
        }
        self.clusters
            .last()
            .map(logical_end_visual_offset)
            .unwrap_or(self.line.visual_range.end)
    }

    // 允许重复或乱序源范围；同源范围的非等长投影使用整个视觉包络。
    fn visual_offset_for_caret_linear(&self, caret: &UiTextCaret) -> usize {
        let exact = match caret.affinity {
            UiTextCaretAffinity::Upstream => self
                .clusters
                .iter()
                .filter(|cluster| cluster.source_range.end == caret.offset)
                .max_by_key(|cluster| cluster.source_range.start)
                .map(|cluster| self.source_edge_visual_offset(cluster, false)),
            UiTextCaretAffinity::Downstream => self
                .clusters
                .iter()
                .filter(|cluster| cluster.source_range.start == caret.offset)
                .min_by_key(|cluster| cluster.source_range.end)
                .map(|cluster| self.source_edge_visual_offset(cluster, true)),
        };
        if let Some(offset) = exact {
            return offset;
        }

        if let Some(cluster) = self.clusters.iter().find(|cluster| {
            caret.offset > cluster.source_range.start && caret.offset < cluster.source_range.end
        }) {
            return match caret.affinity {
                UiTextCaretAffinity::Upstream => self.source_edge_visual_offset(cluster, true),
                UiTextCaretAffinity::Downstream => self.source_edge_visual_offset(cluster, false),
            };
        }

        if caret.offset <= self.line.source_range.start {
            return self
                .clusters
                .iter()
                .min_by_key(|cluster| cluster.source_range.start)
                .map(logical_start_visual_offset)
                .unwrap_or(self.line.visual_range.start);
        }
        self.clusters
            .iter()
            .max_by_key(|cluster| cluster.source_range.end)
            .map(logical_end_visual_offset)
            .unwrap_or(self.line.visual_range.end)
    }

    /// 将视觉簇边界索引回映为源字节光标。
    ///
    /// # 参数
    /// `visual_boundary` 是簇序号边界，不是视觉 UTF-8 字节偏移；`bias` 选择当前簇前缘或前簇后缘。
    /// `fallback` 由调用方提供，仅在所选簇不存在时原样使用，不补做 UTF-8 边界检查。
    /// # 返回
    /// 前缘固定为下游亲和性，后缘固定为上游亲和性，回退时也保持这一选择。
    pub fn caret_for_visual_boundary(
        &self,
        visual_boundary: usize,
        bias: UiTextVisualBoundaryBias,
        fallback: usize,
    ) -> UiTextCaret {
        match bias {
            UiTextVisualBoundaryBias::LeadingCurrent => self
                .clusters
                .get(visual_boundary)
                .map(|cluster| UiTextCaret {
                    offset: leading_source_offset(cluster),
                    affinity: UiTextCaretAffinity::Downstream,
                })
                .unwrap_or(UiTextCaret {
                    offset: fallback,
                    affinity: UiTextCaretAffinity::Downstream,
                }),
            UiTextVisualBoundaryBias::TrailingPrevious => visual_boundary
                .checked_sub(1)
                .and_then(|index| self.clusters.get(index))
                .map(|cluster| UiTextCaret {
                    offset: trailing_source_offset(cluster),
                    affinity: UiTextCaretAffinity::Upstream,
                })
                .unwrap_or(UiTextCaret {
                    offset: fallback,
                    affinity: UiTextCaretAffinity::Upstream,
                }),
        }
    }

    /// 选择与源范围相交的完整视觉簇，仅合并视觉相邻片段。
    ///
    /// # 参数与返回
    /// `range` 使用源字节坐标；返回拥有的视觉字节区间列表，不按源字节精确裁切簇内部。
    /// 双向文本可返回多个分离片段；本入口不补做区间合法性检查。
    pub fn visual_spans_for_source_range(&self, range: UiTextRange) -> Vec<UiTextVisualSpan> {
        if self.source_ranges_are_disjoint_and_ordered {
            return visual_spans_from_clusters(self.source_clusters_for_range(range).iter());
        }
        self.visual_spans_for_source_range_linear(range)
    }

    // 调用方已确认源范围有序且分离；两端二分只选择相交簇的连续切片。
    fn source_clusters_for_range(&self, range: UiTextRange) -> &[UiTextVisualSourceCluster] {
        let start = self
            .clusters
            .partition_point(|cluster| cluster.source_range.end <= range.start);
        let end = self
            .clusters
            .partition_point(|cluster| cluster.source_range.start < range.end);
        &self.clusters[start.min(end)..end]
    }

    fn visual_spans_for_source_range_linear(&self, range: UiTextRange) -> Vec<UiTextVisualSpan> {
        visual_spans_from_clusters(self.clusters.iter().filter(|cluster| {
            range.start < cluster.source_range.end && cluster.source_range.start < range.end
        }))
    }

    /// 累计视觉字节位置之前已完成的簇，得到行内 advance。
    ///
    /// # 参数与返回
    /// `visual_offset` 使用解析行的视觉字节坐标；度量不满足构造条件时仅提供零值或已知行尾宽度。
    /// # 约束
    /// 前缀索引按需建立；逐项有限且非负不保证普通浮点累计结果仍有限。
    pub fn advance_to_visual_offset(&self, visual_offset: usize) -> f32 {
        if let Some(cache) = &self.exact_advance_cache {
            // Visual ranges follow the visual text's UTF-8 byte order, so the
            // completed cluster count is monotonic and can use binary search.
            let completed = self
                .clusters
                .partition_point(|cluster| cluster.visual_range.end <= visual_offset);
            if let Some(advance) = cache
                .prefixes
                .get()
                .and_then(|prefixes| prefixes.get(completed))
            {
                return *advance;
            }

            // Caret and IME geometry commonly construct a temporary map for one
            // lookup. Preserve their old zero-allocation, bounded scan before
            // paying for a prefix table shared by repeated selection/decor queries.
            // 此处未分配的是前缀索引；构造映射时已建立并拥有簇表。
            if cache.query_count.fetch_add(1, Ordering::Relaxed)
                < EXACT_ADVANCE_PREFIX_LINEAR_QUERY_LIMIT
            {
                return self.exact_advance_to_visual_index(completed);
            }

            let prefixes = cache.prefixes.get_or_init(|| self.exact_advance_prefixes());
            if let Some(advance) = prefixes.get(completed) {
                return *advance;
            }
        }

        if visual_offset >= self.line.visual_range.end {
            sanitized_advance(self.line.measured_width)
        } else {
            0.0
        }
    }

    fn exact_advance_to_visual_index(&self, visual_index: usize) -> f32 {
        self.line
            .glyph_advances
            .iter()
            .take(visual_index)
            .map(|advance| sanitized_advance(*advance))
            .sum()
    }

    fn exact_advance_prefixes(&self) -> Vec<f32> {
        let mut prefixes = Vec::with_capacity(self.clusters.len().saturating_add(1));
        let mut total = 0.0;
        prefixes.push(total);
        for advance in &self.line.glyph_advances {
            total += sanitized_advance(*advance);
            prefixes.push(total);
        }
        prefixes
    }

    fn source_edge_visual_offset(
        &self,
        cluster: &UiTextVisualSourceCluster,
        logical_start: bool,
    ) -> usize {
        if cluster.source_isomorphic {
            return if logical_start {
                logical_start_visual_offset(cluster)
            } else {
                logical_end_visual_offset(cluster)
            };
        }

        let (visual_start, visual_end) = self.source_edge_visual_bounds(cluster);
        if matches!(cluster.direction, UiTextDirection::RightToLeft) == logical_start {
            visual_end
        } else {
            visual_start
        }
    }

    // 缓存键是完整源区间；未命中累计达到阈值后才建立包络表，否则保留扫描路径。
    fn source_edge_visual_bounds(&self, cluster: &UiTextVisualSourceCluster) -> (usize, usize) {
        if let Some(cache) = &self.source_edge_cache {
            let key = (cluster.source_range.start, cluster.source_range.end);
            if let Some(bounds) = cache.bounds.get().and_then(|bounds| bounds.get(&key)) {
                return *bounds;
            }
            if cache.query_count.fetch_add(1, Ordering::Relaxed) >= SOURCE_EDGE_LINEAR_QUERY_LIMIT {
                if let Some(bounds) = cache
                    .bounds
                    .get_or_init(|| self.source_edge_bounds_by_range())
                    .get(&key)
                {
                    return *bounds;
                }
            }
        }
        self.source_edge_visual_bounds_linear(cluster)
    }

    // 仅聚合未标为等长的投影，值是同源范围各视觉区间的最小起点与最大终点。
    fn source_edge_bounds_by_range(&self) -> HashMap<(usize, usize), (usize, usize)> {
        let mut bounds_by_range = HashMap::with_capacity(self.clusters.len());
        for cluster in self
            .clusters
            .iter()
            .filter(|cluster| !cluster.source_isomorphic)
        {
            let key = (cluster.source_range.start, cluster.source_range.end);
            bounds_by_range
                .entry(key)
                .and_modify(|bounds: &mut (usize, usize)| {
                    bounds.0 = bounds.0.min(cluster.visual_range.start);
                    bounds.1 = bounds.1.max(cluster.visual_range.end);
                })
                .or_insert((cluster.visual_range.start, cluster.visual_range.end));
        }
        bounds_by_range
    }

    fn source_edge_visual_bounds_linear(
        &self,
        cluster: &UiTextVisualSourceCluster,
    ) -> (usize, usize) {
        self.clusters
            .iter()
            .filter(|candidate| candidate.source_range == cluster.source_range)
            .fold(
                (cluster.visual_range.start, cluster.visual_range.end),
                |bounds, candidate| {
                    (
                        bounds.0.min(candidate.visual_range.start),
                        bounds.1.max(candidate.visual_range.end),
                    )
                },
            )
    }
}

// 输出拥有完整簇的视觉范围；相邻合并不意味着逻辑源范围连续。
fn visual_spans_from_clusters<'a>(
    clusters: impl Iterator<Item = &'a UiTextVisualSourceCluster>,
) -> Vec<UiTextVisualSpan> {
    let mut spans = Vec::<UiTextVisualSpan>::new();
    for cluster in clusters {
        if let Some(last) = spans.last_mut() {
            if last.visual_range.end == cluster.visual_range.start {
                last.visual_range.end = cluster.visual_range.end;
                continue;
            }
        }
        spans.push(UiTextVisualSpan {
            visual_range: cluster.visual_range,
        });
    }
    spans
}

fn unique_source_edge_visual_offset(
    cluster: &UiTextVisualSourceCluster,
    logical_start: bool,
) -> usize {
    if cluster.source_isomorphic {
        return if logical_start {
            logical_start_visual_offset(cluster)
        } else {
            logical_end_visual_offset(cluster)
        };
    }
    if matches!(cluster.direction, UiTextDirection::RightToLeft) == logical_start {
        cluster.visual_range.end
    } else {
        cluster.visual_range.start
    }
}

// 只规范化单个度量值，调用方的后续求和没有因此获得溢出保护。
fn sanitized_advance(advance: f32) -> f32 {
    if advance.is_finite() {
        advance.max(0.0)
    } else {
        0.0
    }
}
