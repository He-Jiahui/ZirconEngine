// 结构守卫读取生产源文件；模块声明比较使用逐行完全相等，生产文件新增注释也会被视为额外行。
// TODO: [CR-SOUND-TESTS-0002] 需确认纯模块声明文件是否允许文档注释；若允许，应只比较有效声明行。
use std::path::Path;

use super::source::read_source;

pub(crate) fn assert_structural_module(src: &Path, relative_path: &str, expected_lines: &[&str]) {
    let content = read_source(src, relative_path);
    let actual_lines = content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(
        actual_lines, expected_lines,
        "{relative_path} should remain a structural module declaration file"
    );
}

pub(crate) fn assert_source_contains(src: &Path, relative_path: &str, expected_fragments: &[&str]) {
    let content = read_source(src, relative_path);
    for fragment in expected_fragments {
        assert!(
            content.contains(fragment),
            "{relative_path} should contain {fragment}"
        );
    }
}
