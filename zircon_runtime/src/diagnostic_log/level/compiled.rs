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

    fn filter_for_scope(&self, scope: &str) -> DiagnosticLogFilter {
        let mut filter = self.minimum;
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
