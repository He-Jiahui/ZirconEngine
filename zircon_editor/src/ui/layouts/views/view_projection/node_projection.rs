use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

use crate::ui::retained_host::primitives::ModelRc;

use super::super::ViewTemplateNodeData;
use super::projection_cache;

#[derive(Clone)]
pub(crate) struct ViewTemplateNodeProjection {
    pub(super) base_rows: Rc<Vec<Rc<ViewTemplateNodeData>>>,
    pub(super) row_patches: Rc<BTreeMap<usize, Rc<ViewTemplateNodeData>>>,
}

impl ViewTemplateNodeProjection {
    pub(crate) fn iter(&self) -> impl Iterator<Item = &ViewTemplateNodeData> {
        (0..self.base_rows.len()).map(|row| {
            self.row_patches
                .get(&row)
                .or_else(|| self.base_rows.get(row))
                .expect("projection row must resolve")
                .as_ref()
        })
    }

    pub(super) fn row_rc(&self, row: usize) -> Option<&Rc<ViewTemplateNodeData>> {
        self.row_patches
            .get(&row)
            .or_else(|| self.base_rows.get(row))
    }

    pub(super) fn changed_rows_from(&self, previous: &Self) -> Option<BTreeSet<usize>> {
        if !Rc::ptr_eq(&self.base_rows, &previous.base_rows) {
            return None;
        }
        Some(
            self.row_patches
                .keys()
                .chain(previous.row_patches.keys())
                .copied()
                .filter(|row| {
                    self.row_rc(*row)
                        .zip(previous.row_rc(*row))
                        .is_some_and(|(next, previous)| !Rc::ptr_eq(next, previous))
                })
                .collect(),
        )
    }

    pub(super) fn shares_rows_with(&self, other: &Self) -> bool {
        self.changed_rows_from(other)
            .is_some_and(|changed_rows| changed_rows.is_empty())
    }

    pub(crate) fn into_model(self) -> ModelRc<ViewTemplateNodeData> {
        ModelRc::from_shared_rows_overlay(self.base_rows, self.row_patches)
    }

    #[cfg(test)]
    pub(crate) fn into_vec(self) -> Vec<ViewTemplateNodeData> {
        projection_cache::record_legacy_full_clone_for_tests();
        self.iter().cloned().collect()
    }
}
