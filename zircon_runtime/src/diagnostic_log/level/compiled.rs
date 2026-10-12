//! 将配置中的作用域前缀编译为字节 trie，使每条日志在写入 worker 前按最长匹配规则判定。
use std::collections::HashMap;

use super::{DiagnosticLogFilter, DiagnosticLogFilterConfig, DiagnosticLogLevel};

/// Byte-prefix trie compiled once during logger initialization.
pub(crate) struct CompiledDiagnosticLogFilter {
    minimum: DiagnosticLogFilter,
    nodes: Vec<FilterNode>,
}

#[derive(Default)]
struct FilterNode {
    filter: Option<DiagnosticLogFilter>,
    children: HashMap<u8, usize>,
}

impl CompiledDiagnosticLogFilter {
    pub(crate) fn new(config: &DiagnosticLogFilterConfig) -> Self {
        let mut compiled = Self {
            minimum: config.minimum,
            nodes: vec![FilterNode::default()],
        };
        for rule in &config.module_filters {
            compiled.insert(rule.scope_prefix.as_bytes(), rule.filter);
        }
        compiled
    }

    pub(crate) fn allows(&self, level: DiagnosticLogLevel, scope: &str) -> bool {
        self.filter_for_scope(scope).allows(level)
    }

    fn insert(&mut self, prefix: &[u8], filter: DiagnosticLogFilter) {
        let mut node_index = 0;
        for byte in prefix {
            let next = self.nodes[node_index].children.get(byte).copied();
            node_index = match next {
                Some(index) => index,
                None => {
                    let index = self.nodes.len();
                    self.nodes.push(FilterNode::default());
                    self.nodes[node_index].children.insert(*byte, index);
                    index
                }
            };
        }
        self.nodes[node_index].filter = Some(filter);
    }

    // 先应用根节点的空前缀规则或全局 minimum，再沿 scope 的字节路径保留最长命中的过滤值。
    fn filter_for_scope(&self, scope: &str) -> DiagnosticLogFilter {
        let mut filter = self.nodes[0].filter.unwrap_or(self.minimum);
        if self.nodes.len() == 1 {
            return filter;
        }

        let mut node_index = 0;
        for byte in scope.as_bytes() {
            let Some(next) = self.nodes[node_index].children.get(byte).copied() else {
                break;
            };
            node_index = next;
            if let Some(node_filter) = self.nodes[node_index].filter {
                filter = node_filter;
            }
        }
        filter
    }
}
