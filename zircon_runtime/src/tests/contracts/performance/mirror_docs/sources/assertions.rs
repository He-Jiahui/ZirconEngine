//! 性能热点的结构守卫核对拥有者、文件预算和证据文档。对照源文件与文档的当前锚点，记录尚待运行验证的结构约束。
pub(super) fn assert_contains_all(label: &str, source: &str, anchors: &[&str]) {
    for anchor in anchors {
        assert!(
            source.contains(anchor),
            "{label} should mirror Runtime 07 performance-hotpath audit anchor `{anchor}`"
        );
    }
}
