//! 回退候选的 cmap 覆盖预筛：已知覆盖压缩为有序区间，未知覆盖不在此阶段误拒绝。
//! 本层不进行字体选择或字形映射；查询结果交回外层回退策略使用。
//! 压缩结果拥有区间，不保留源字体字节或输入迭代器的借用。
use std::collections::HashSet;

/// 按进入规范化时的元素总数分流；重复项也计入，单位不是字节或去重后数量。
const HASH_DEDUP_CODEPOINT_THRESHOLD: usize = 128;

/// Unknown 表示尚不能证明缺字，可保留候选；最终字形能力仍由后续 shaping/raster 阶段决定。
/// Known 的端点均包含在范围内，区间应按起点递增且互不重叠。
/// 这是构造路径建立的约束，变体本身不校验任意手写范围。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum FontCoverage {
    /// 拥有的闭区间向量；克隆复制范围值，查询只在调用期间借用。
    Known(Vec<(u32, u32)>),
    /// 缺少可用覆盖证据时保留候选；查询为真不等同于存在字形。
    Unknown,
}

impl FontCoverage {
    /// 消费任意顺序的整数值，规范化后压缩为覆盖区间。
    ///
    /// # 参数
    /// - `codepoints`：按值移入的码点数值；重复项会移除，本层不检查 Unicode 标量合法性。
    ///
    /// # 返回与约束
    /// 返回拥有区间的 Self，空输入产生 Unknown；不借用输入缓冲。
    /// 未规范化的集合走此入口，已排序的元数据投影可直接使用有序入口。
    pub(super) fn from_codepoint_values(codepoints: Vec<u32>) -> Self {
        Self::from_sorted_unique_codepoints(normalize_codepoint_values(codepoints))
    }

    /// Compacts a canonical codepoint stream without copying or re-sorting it.
    /// 消费调用方已升序排列的值流，逐段合并相邻整数。
    ///
    /// # 参数
    /// - `codepoints`：可消费的 u32 迭代源；实际元数据调用投影已排序去重的字形表。
    ///   `impl IntoIterator` 是匿名输入类型约束，不要求调用方先创建 Vec。
    ///
    /// # 返回与约束
    /// 返回自有闭区间；空流返回 Unknown，输入借用不会保存在结果中。
    /// 不重新排序；不大于当前末端的值会被跳过，不能据此恢复乱序流中丢失的值。
    /// 参见 FontGlyphMap::coverage 的有序生产路径。
    pub(super) fn from_sorted_unique_codepoints(codepoints: impl IntoIterator<Item = u32>) -> Self {
        let mut codepoints = codepoints.into_iter();
        let Some(mut start) = codepoints.next() else {
            return Self::Unknown;
        };

        let mut end = start;
        let mut ranges = Vec::new();
        for codepoint in codepoints {
            // 只在有序前提下跳过已覆盖值；这里没有收集乱序值再排序的补救步骤。
            if codepoint <= end {
                continue;
            }
            if codepoint == end.saturating_add(1) {
                end = codepoint;
                continue;
            }
            ranges.push((start, end));
            start = codepoint;
            end = codepoint;
        }
        ranges.push((start, end));
        Self::Known(ranges)
    }

    /// 查询单个字符能否通过这一覆盖预筛。
    ///
    /// # 参数
    /// - `self`：只读借用覆盖记录，查询结束后不保留范围引用。
    /// - `codepoint`：按值传入的字符；比较时转换为 u32，单位是码点值。
    ///
    /// # 返回与约束
    /// Known 在某个闭区间包含该值时返回 true；Unknown 一律返回 true 以保留候选。
    /// 不能据此证明整簇 shaping 成功；连接符与变体选择符的跳过规则由 face_access 外层处理。
    pub(super) fn contains(&self, codepoint: char) -> bool {
        match self {
            Self::Known(ranges) => {
                let codepoint = codepoint as u32;
                ranges
                    .binary_search_by(|(start, end)| {
                        if *end < codepoint {
                            std::cmp::Ordering::Less
                        } else if *start > codepoint {
                            std::cmp::Ordering::Greater
                        } else {
                            std::cmp::Ordering::Equal
                        }
                    })
                    .is_ok()
            }
            Self::Unknown => true,
        }
    }

    #[cfg(test)]
    /// 为测试将借用的字符切片转成自有整数向量，再走通用规范化入口。
    /// `codepoints` 只借用到调用结束；返回的 Self 不保留切片引用。
    pub(super) fn from_codepoints(codepoints: &[char]) -> Self {
        let codepoints = codepoints
            .iter()
            .map(|codepoint| *codepoint as u32)
            .collect::<Vec<_>>();
        Self::from_codepoint_values(codepoints)
    }
}

/// 将拥有的整数向量规范化为严格升序且无重复的向量。
///
/// # 参数与返回
/// `codepoints` 被消费，返回自有 Vec<u32>；不借用输入，也不验证 Unicode 标量合法性。
///
/// # 约束
/// 小输入直接排序去重；较大输入先散列去重，再只排序唯一值。
/// 分流依据原始元素数；散列表容量也按该数量预留，不能当作实际内存或分配次数。
fn normalize_codepoint_values(mut codepoints: Vec<u32>) -> Vec<u32> {
    if codepoints.len() < HASH_DEDUP_CODEPOINT_THRESHOLD {
        codepoints.sort_unstable();
        codepoints.dedup();
        return codepoints;
    }

    // 先缩小排序集合；仍会建立散列表和输出向量，不承诺本分支无分配。
    let mut unique = HashSet::with_capacity(codepoints.len());
    for codepoint in codepoints {
        unique.insert(codepoint);
    }
    let mut codepoints = unique.into_iter().collect::<Vec<_>>();
    codepoints.sort_unstable();
    codepoints
}

#[cfg(test)]
#[path = "tests/coverage.rs"]
mod tests;

#[cfg(test)]
#[path = "coverage/tests/hash_dedup_tests.rs"]
mod hash_dedup_tests;
