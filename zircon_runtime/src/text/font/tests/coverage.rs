//! 核对两个已排序闭区间的查询，不加载实际字体或执行 shaping。

use super::FontCoverage;

/// 用 ASCII、Cyrillic 正例和区间外负例检查二分查询。
/// 只覆盖这里手写的 Known 记录，不验证未知覆盖或全部 Unicode 字符。
#[test]
fn coverage_contains_uses_ordered_ranges_without_gap_false_positives() {
    let coverage = FontCoverage::Known(vec![(0x0020, 0x007E), (0x0400, 0x04FF)]);

    assert!(coverage.contains('A'));
    assert!(coverage.contains('\u{416}'));
    assert!(!coverage.contains('\u{4e2d}'));
}
