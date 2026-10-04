//! 命名策略扫描需区分生产源码、测试夹具和已分类的历史名称。以结果断言检查当前接口或源码快照对应的边界。
pub(super) fn production_source_lines(source: &str) -> Vec<(usize, &str)> {
    let mut result = Vec::new();
    let mut brace_depth = 0isize;
    let mut pending_cfg_test = false;
    let mut cfg_test_base_depth = None;

    for (line_index, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        // BUG: [CR-RUNTIME-TESTS-ABS-0008] 逐行统计花括号把字符串内容也计入测试模块深度，导致夹具行泄漏或后续生产行被跳过；证据：当前函数及独立复现。
        let delta = line.matches('{').count() as isize - line.matches('}').count() as isize;

        if let Some(base_depth) = cfg_test_base_depth {
            brace_depth += delta;
            if brace_depth <= base_depth {
                cfg_test_base_depth = None;
            }
            continue;
        }
        if trimmed == "#[cfg(test)]" {
            pending_cfg_test = true;
            continue;
        }
        if pending_cfg_test && !trimmed.starts_with("#[") {
            pending_cfg_test = false;
            if trimmed.starts_with("mod tests") && line.contains('{') {
                let base_depth = brace_depth;
                brace_depth += delta;
                cfg_test_base_depth = Some(base_depth);
                continue;
            }
        }

        result.push((line_index + 1, line));
        brace_depth += delta;
    }

    result
}

#[test]
fn naming_scan_excludes_embedded_cfg_test_module_lines() {
    let source = "pub fn runtime_owner() {}\n#[cfg(test)]\nmod tests {\n    const EDITOR_FIXTURE: &str = \"editor base.zui\";\n}\npub fn runtime_tail() {}\n";
    let production = production_source_lines(source)
        .into_iter()
        .map(|(_, line)| line)
        .collect::<Vec<_>>()
        .join("\n");

    assert!(production.contains("runtime_owner"));
    assert!(production.contains("runtime_tail"));
    assert!(!production.contains("EDITOR_FIXTURE"));
}
