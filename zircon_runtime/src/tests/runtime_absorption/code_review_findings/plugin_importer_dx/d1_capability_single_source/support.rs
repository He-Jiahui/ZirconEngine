//! 代码审查回归护栏核对已迁移接口、错误边界、插件入口及镜像状态。向相邻守卫提供源码读取、路径枚举或断言工具。
pub(super) fn assert_contains_all(label: &str, source: &str, needles: &[&str]) {
    for needle in needles {
        assert!(source.contains(needle), "{label} should contain `{needle}`");
    }
}
