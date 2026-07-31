use serde::{Deserialize, Serialize};

use super::super::{EditorEventRecord, EditorEventSource};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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

    pub(super) fn normalized(mut self) -> Self {
        for prefix in &mut self.operation_path_prefixes {
            *prefix = normalize_operation_path_prefix(prefix);
        }
        self
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
            if !self
                .operation_groups
                .iter()
                .any(|group| group == operation_group)
            {
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

fn default_filter_includes_events() -> bool {
    true
}

#[cfg(test)]
mod tests {
    #[test]
    fn listener_acceptance_does_not_normalize_prefixes_per_record() {
        let source = include_str!("filter.rs");
        let hot_normalization = [
            "operation_id.starts_with(&",
            "normalize_operation_path_prefix(prefix))",
        ]
        .concat();
        assert!(!source.contains(&hot_normalization));
    }

    #[test]
    fn listener_filter_normalizes_operation_prefixes_once() {
        let filter = super::EditorEventListenerFilter::operation_prefix("  Scene.Node  ");
        assert_eq!(filter.operation_path_prefixes, vec!["scene.node"]);
    }
}
