use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// 宿主策略比较和诊断使用的副作用粗分类；该枚举描述标识符所暗示的影响，不执行动作。
pub enum UiActionSideEffectClass {
    LocalUi,
    EditorMutation,
    AssetIo,
    SceneMutation,
    ExternalProcess,
    Network,
}

impl UiActionSideEffectClass {
    /// 按名称关键字推断类别；多个关键字同时命中时，网络、进程、场景、资产 IO、编辑器修改依次优先。
    /// 结果交给 `UiActionHostPolicy::allows` 作策略判断，因此该顺序会影响报告类别。
    pub fn infer(route: Option<&str>, action: Option<&str>) -> Self {
        let contains = |needle| {
            route.is_some_and(|text| contains_ascii_case_insensitive(text, needle))
                || action.is_some_and(|text| contains_ascii_case_insensitive(text, needle))
        };
        if contains("network") || contains("http") || contains("socket") {
            Self::Network
        } else if contains("process") || contains("command") || contains("shell") {
            Self::ExternalProcess
        } else if contains("scene") || contains("entity") || contains("world") {
            Self::SceneMutation
        } else if contains("asset")
            || contains("file")
            || contains("save")
            || contains("load")
            || contains("import")
        {
            Self::AssetIo
        } else if contains("editor") || contains("inspector") || contains("undo") {
            Self::EditorMutation
        } else {
            Self::LocalUi
        }
    }
}

// 标识符只按 ASCII 大小写折叠匹配，直接比较字节窗口可避免先分配小写副本；非 ASCII 字符不做 Unicode 折叠。
fn contains_ascii_case_insensitive(text: &str, needle: &str) -> bool {
    let needle = needle.as_bytes();
    text.as_bytes()
        .windows(needle.len())
        .any(|window| window.eq_ignore_ascii_case(needle))
}

#[cfg(test)]
#[path = "side_effect_class/tests/performance_tests.rs"]
mod performance_tests;
