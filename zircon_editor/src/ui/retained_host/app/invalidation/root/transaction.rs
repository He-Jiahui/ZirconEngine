use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use crate::ui::retained_host::HostShellContentScope;
use crate::ui::workbench::view::ViewInstanceId;

use super::super::HostInvalidationMask;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::ui::retained_host::app) enum HostInvalidationScope {
    All,
    View(ViewInstanceId),
    ShellContent(HostShellContentScope),
}

impl Hash for HostInvalidationScope {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::All => 0_u8.hash(state),
            Self::View(view) => {
                1_u8.hash(state);
                view.hash(state);
            }
            Self::ShellContent(scope) => {
                2_u8.hash(state);
                scope.slot.hash(state);
                scope.instance_id.hash(state);
            }
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(in crate::ui::retained_host::app) struct HostInvalidationTransaction {
    reasons_by_scope: HashMap<HostInvalidationScope, HostInvalidationMask>,
}

impl HostInvalidationTransaction {
    pub(in crate::ui::retained_host::app) fn insert(
        &mut self,
        scope: HostInvalidationScope,
        reasons: HostInvalidationMask,
    ) {
        self.reasons_by_scope
            .entry(scope)
            .or_default()
            .insert(reasons);
    }

    pub(in crate::ui::retained_host::app) fn reasons(&self) -> HostInvalidationMask {
        self.reasons_by_scope
            .values()
            .copied()
            .fold(HostInvalidationMask::NONE, HostInvalidationMask::union)
    }

    pub(in crate::ui::retained_host::app) fn reasons_for(
        &self,
        scope: &HostInvalidationScope,
    ) -> Option<HostInvalidationMask> {
        self.reasons_by_scope.get(scope).copied()
    }

    pub(in crate::ui::retained_host::app) fn scope_count(&self) -> usize {
        self.reasons_by_scope.len()
    }

    pub(in crate::ui::retained_host::app) fn requires_presentation_recompute(&self) -> bool {
        self.reasons().requires_host_recompute()
    }

    pub(in crate::ui::retained_host::app) fn presentation_only_view_ids(
        &self,
    ) -> Option<Vec<ViewInstanceId>> {
        let mut views = Vec::with_capacity(self.reasons_by_scope.len());
        for (scope, reasons) in &self.reasons_by_scope {
            let HostInvalidationScope::View(view) = scope else {
                return None;
            };
            if *reasons != HostInvalidationMask::PRESENTATION_DATA {
                return None;
            }
            views.push(view.clone());
        }
        views.sort_unstable();
        (!views.is_empty()).then_some(views)
    }

    pub(in crate::ui::retained_host::app) fn shell_content_scope(
        &self,
    ) -> Option<HostShellContentScope> {
        if self.reasons_by_scope.len() != 1 {
            return None;
        }
        let (scope, reasons) = self.reasons_by_scope.iter().next()?;
        let allowed =
            HostInvalidationMask::SHELL_CONTENT.union(HostInvalidationMask::PRESENTATION_DATA);
        if !reasons.contains(HostInvalidationMask::SHELL_CONTENT)
            || reasons.intersection(allowed) != *reasons
        {
            return None;
        }
        let HostInvalidationScope::ShellContent(scope) = scope else {
            return None;
        };
        Some(scope.clone())
    }

    pub(in crate::ui::retained_host::app) fn consume(
        &mut self,
        mask: HostInvalidationMask,
    ) -> HostInvalidationMask {
        let mut consumed = HostInvalidationMask::NONE;
        self.reasons_by_scope.retain(|_, reasons| {
            let scoped = reasons.intersection(mask);
            consumed.insert(scoped);
            reasons.remove(mask);
            !reasons.is_empty()
        });
        consumed
    }
}

#[cfg(test)]
#[path = "transaction/hash_index_tests.rs"]
mod hash_index_tests;
