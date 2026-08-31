#[cfg(test)]
use std::cell::Cell;

use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};

pub(crate) struct HostPaneTemplateHitIndex {
    indexed_nodes: ModelRc<TemplatePaneNodeData>,
    popup_rows: Box<[usize]>,
    #[cfg(test)]
    last_popup_candidate_visit_count: Cell<usize>,
    #[cfg(test)]
    query_count: Cell<usize>,
}

impl HostPaneTemplateHitIndex {
    pub(crate) fn new(nodes: ModelRc<TemplatePaneNodeData>) -> Self {
        let popup_rows = boxed_popup_rows(
            nodes
                .iter()
                .enumerate()
                .filter_map(|(row, node)| {
                    (node.popup_open && !node.disabled && !node.control_id.is_empty())
                        .then_some(row)
                })
                .collect(),
        );
        Self {
            indexed_nodes: nodes,
            popup_rows,
            #[cfg(test)]
            last_popup_candidate_visit_count: Cell::new(0),
            #[cfg(test)]
            query_count: Cell::new(0),
        }
    }

    pub(crate) fn indexes_nodes(&self, nodes: &ModelRc<TemplatePaneNodeData>) -> bool {
        self.indexed_nodes.shares_values_with(nodes)
    }

    pub(super) fn popup_rows(&self) -> &[usize] {
        &self.popup_rows
    }

    pub(super) fn begin_query(&self) {
        record_current_ui_perf_counter(UiPerfCounter::PanePopupIndexQueryCount, 1.0);
        record_current_ui_perf_counter(
            UiPerfCounter::PanePopupIndexCandidateCount,
            self.popup_rows.len() as f64,
        );
        #[cfg(test)]
        {
            self.query_count.set(self.query_count.get() + 1);
            self.last_popup_candidate_visit_count.set(0);
        }
    }

    pub(super) fn record_popup_candidate_visit(&self) {
        #[cfg(test)]
        self.last_popup_candidate_visit_count
            .set(self.last_popup_candidate_visit_count.get() + 1);
    }

    #[cfg(test)]
    pub(crate) fn last_popup_candidate_visit_count_for_test(&self) -> usize {
        self.last_popup_candidate_visit_count.get()
    }

    #[cfg(test)]
    pub(crate) fn query_count_for_test(&self) -> usize {
        self.query_count.get()
    }
}

fn boxed_popup_rows(rows: Vec<usize>) -> Box<[usize]> {
    rows.into_boxed_slice()
}

#[cfg(test)]
#[path = "pane_index/boxed_popup_rows_tests.rs"]
mod boxed_popup_rows_tests;
