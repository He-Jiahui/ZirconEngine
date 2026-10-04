use serde::{Deserialize, Serialize};

use super::super::{EditorEventRecord, EditorEventSource};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 监听器对操作路径、操作组、来源和成功/失败的组合筛选；各维度之间取交集，同一维度的候选取并集。
/// 空候选列表表示该维度不限；要求操作路径或组时，没有对应元数据的记录不会通过。
pub struct EditorEventListenerFilter {
    #[serde(default)]
    pub operation_path_prefixes: Vec<String>,
    #[serde(default)]
    pub operation_groups: Vec<String>,
    #[serde(default)]
    pub sources: Vec<EditorEventSource>,
    #[serde(default = "default_filter_includes_events")]
    pub include_successes: bool,
    #[serde(default = "default_filter_includes_events")]
    pub include_failures: bool,
}

impl Default for EditorEventListenerFilter {
    fn default() -> Self {
        Self {
            operation_path_prefixes: Vec::new(),
            operation_groups: Vec::new(),
            sources: Vec::new(),
            include_successes: true,
            include_failures: true,
        }
    }
}

impl EditorEventListenerFilter {
    pub fn operation_prefix(prefix: impl Into<String>) -> Self {
        let prefix = prefix.into();
        Self {
            operation_path_prefixes: vec![normalize_operation_path_prefix(&prefix)],
            ..Self::default()
        }
    }

    pub fn operation_group(group: impl Into<String>) -> Self {
        Self {
            operation_groups: vec![group.into()],
            ..Self::default()
        }
    }

    pub fn source(source: EditorEventSource) -> Self {
        Self {
            sources: vec![source],
            ..Self::default()
        }
    }

    pub fn with_sources<I>(mut self, sources: I) -> Self
    where
        I: IntoIterator<Item = EditorEventSource>,
    {
        self.sources = sources.into_iter().collect();
        self
    }

    pub fn failures_only(mut self) -> Self {
        self.include_successes = false;
        self.include_failures = true;
        self
    }

    pub fn successes_only(mut self) -> Self {
        self.include_successes = true;
        self.include_failures = false;
        self
    }

    pub fn operation_groups(&self) -> &[String] {
        &self.operation_groups
    }

    // 在控制配置写入时统一规范与去重，使逐记录过滤只读固定条件；组匹配依赖这里建立的排序不变量。
    pub(super) fn normalized(mut self) -> Self {
        for prefix in &mut self.operation_path_prefixes {
            *prefix = normalize_operation_path_prefix(prefix);
        }
        self.operation_path_prefixes.sort_unstable();
        self.operation_path_prefixes.dedup();
        self.operation_groups.sort_unstable();
        self.operation_groups.dedup();

        let mut seen_sources = [false; 5];
        self.sources.retain(|source| {
            let seen = &mut seen_sources[editor_event_source_index(source)];
            !std::mem::replace(seen, true)
        });
        self
    }

    fn accepts_operation_group(&self, operation_group: &str) -> bool {
        self.operation_groups
            .binary_search_by(|group| group.as_str().cmp(operation_group))
            .is_ok()
    }

    pub(super) fn accepts(&self, record: &EditorEventRecord) -> bool {
        if !self.operation_path_prefixes.is_empty() {
            let Some(operation_id) = record.operation_id.as_deref() else {
                return false;
            };
            if !self
                .operation_path_prefixes
                .iter()
                .any(|prefix| operation_id.starts_with(prefix))
            {
                return false;
            }
        }

        if !self.operation_groups.is_empty() {
            let Some(operation_group) = record.operation_group.as_deref() else {
                return false;
            };
            if !self.accepts_operation_group(operation_group) {
                return false;
            }
        }

        if !self.sources.is_empty() && !self.sources.contains(&record.source) {
            return false;
        }

        if record.result.error.is_some() {
            return self.include_failures;
        }
        self.include_successes
    }
}

fn normalize_operation_path_prefix(prefix: &str) -> String {
    prefix.trim().to_ascii_lowercase()
}

fn editor_event_source_index(source: &EditorEventSource) -> usize {
    match source {
        EditorEventSource::RetainedHost => 0,
        EditorEventSource::Headless => 1,
        EditorEventSource::Cli => 2,
        EditorEventSource::Mcp => 3,
        EditorEventSource::Replay => 4,
    }
}

fn default_filter_includes_events() -> bool {
    true
}

#[cfg(test)]
#[path = "tests/filter.rs"]
mod tests;
