use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::{Deserialize, Serialize};
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    tree::{UiDirtyFlags, UiTreeError},
};

#[cfg(test)]
#[path = "invalidation/domain_bitset_tests.rs"]
mod domain_bitset_tests;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum UiInvalidationReason {
    Structure,
    Layout,
    Text,
    HitTest,
    Render,
    Interaction,
    Resource,
}

impl UiInvalidationReason {
    const ALL: [Self; 7] = [
        Self::Structure,
        Self::Layout,
        Self::Text,
        Self::HitTest,
        Self::Render,
        Self::Interaction,
        Self::Resource,
    ];

    const fn domain_bit(self) -> u8 {
        1 << match self {
            Self::Structure => 0,
            Self::Layout => 1,
            Self::Text => 2,
            Self::HitTest => 3,
            Self::Render => 4,
            Self::Interaction => 5,
            Self::Resource => 6,
        }
    }

    pub fn dirty_flags(self) -> UiDirtyFlags {
        match self {
            Self::Structure => UiDirtyFlags {
                layout: true,
                hit_test: true,
                render: true,
                input: true,
                ..UiDirtyFlags::default()
            },
            Self::Layout => UiDirtyFlags {
                layout: true,
                hit_test: true,
                render: true,
                ..UiDirtyFlags::default()
            },
            Self::Text => UiDirtyFlags {
                text: true,
                ..UiDirtyFlags::default()
            },
            Self::HitTest => UiDirtyFlags {
                hit_test: true,
                ..UiDirtyFlags::default()
            },
            Self::Render => UiDirtyFlags {
                render: true,
                ..UiDirtyFlags::default()
            },
            Self::Interaction => UiDirtyFlags {
                hit_test: true,
                render: true,
                input: true,
                ..UiDirtyFlags::default()
            },
            Self::Resource => UiDirtyFlags {
                style: true,
                render: true,
                ..UiDirtyFlags::default()
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiInvalidationGenerations {
    pub generation: u64,
    pub structure: u64,
    pub layout: u64,
    pub text: u64,
    pub hit_test: u64,
    pub render: u64,
    pub interaction: u64,
    pub resource: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiInvalidationChange {
    pub node_id: UiNodeId,
    pub dirty: UiDirtyFlags,
    pub reasons: BTreeSet<UiInvalidationReason>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiInvalidationCommit {
    pub base_generation: u64,
    pub generation: u64,
    pub generations: UiInvalidationGenerations,
    pub dirty: UiDirtyFlags,
    pub changed_nodes: Vec<UiInvalidationChange>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiInvalidationTransaction {
    base_generation: u64,
    changes: BTreeMap<UiNodeId, UiInvalidationChange>,
}

impl UiInvalidationTransaction {
    pub fn new(base_generation: u64) -> Self {
        Self {
            base_generation,
            changes: BTreeMap::new(),
        }
    }

    pub const fn base_generation(&self) -> u64 {
        self.base_generation
    }

    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    pub fn changed_node_count(&self) -> usize {
        self.changes.len()
    }

    pub fn changes(&self) -> impl ExactSizeIterator<Item = &UiInvalidationChange> {
        self.changes.values()
    }

    pub(crate) fn into_changes(self) -> impl ExactSizeIterator<Item = UiInvalidationChange> {
        self.changes.into_values()
    }

    pub fn record_reason(&mut self, node_id: UiNodeId, reason: UiInvalidationReason) {
        record_reason(&mut self.changes, node_id, reason);
    }

    pub fn record_dirty(&mut self, node_id: UiNodeId, dirty: UiDirtyFlags) {
        if !dirty.any() {
            return;
        }
        record_change(&mut self.changes, node_id, dirty, reasons_for_dirty(dirty));
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiSurfaceInvalidationState {
    generations: UiInvalidationGenerations,
    #[serde(default)]
    last_commit: Option<UiInvalidationCommit>,
    #[serde(default)]
    pending: BTreeMap<UiNodeId, UiInvalidationChange>,
}

impl UiSurfaceInvalidationState {
    pub const fn generations(&self) -> UiInvalidationGenerations {
        self.generations
    }

    pub fn last_commit(&self) -> Option<&UiInvalidationCommit> {
        self.last_commit.as_ref()
    }

    pub fn pending_changed_node_count(&self) -> usize {
        self.pending.len()
    }

    pub(crate) fn pending_changed_node_ids(&self) -> BTreeSet<UiNodeId> {
        self.pending.keys().copied().collect()
    }

    pub(crate) fn pending_dirty_flags(&self) -> UiDirtyFlags {
        self.pending
            .values()
            .fold(UiDirtyFlags::default(), |mut dirty, change| {
                merge_dirty(&mut dirty, change.dirty);
                dirty
            })
    }

    pub fn begin_transaction(&self) -> UiInvalidationTransaction {
        UiInvalidationTransaction::new(self.generations.generation)
    }

    pub fn validate_transaction(
        &self,
        transaction: &UiInvalidationTransaction,
    ) -> Result<(), UiInvalidationApplyError> {
        if transaction.base_generation != self.generations.generation {
            return Err(UiInvalidationApplyError::StaleGeneration {
                expected: transaction.base_generation,
                actual: self.generations.generation,
            });
        }
        Ok(())
    }

    pub fn record_reason(&mut self, node_id: UiNodeId, reason: UiInvalidationReason) {
        record_reason(&mut self.pending, node_id, reason);
    }

    pub fn record_dirty(&mut self, node_id: UiNodeId, dirty: UiDirtyFlags) {
        if !dirty.any() {
            return;
        }
        record_change(&mut self.pending, node_id, dirty, reasons_for_dirty(dirty));
    }

    pub(crate) fn record_change(&mut self, change: &UiInvalidationChange) {
        record_change(
            &mut self.pending,
            change.node_id,
            change.dirty,
            change.reasons.iter().copied(),
        );
    }

    pub(crate) fn record_dirty_with_reason(
        &mut self,
        node_id: UiNodeId,
        dirty: UiDirtyFlags,
        reason: UiInvalidationReason,
    ) {
        let mut reasons = reasons_for_dirty(dirty);
        reasons.insert(reason);
        record_change(&mut self.pending, node_id, dirty, reasons);
    }

    pub fn clear_pending(&mut self) {
        self.pending.clear();
    }

    pub fn commit_pending(
        &mut self,
    ) -> Result<Option<UiInvalidationCommit>, UiInvalidationApplyError> {
        let transaction = UiInvalidationTransaction {
            base_generation: self.generations.generation,
            changes: std::mem::take(&mut self.pending),
        };
        self.apply_transaction(transaction)
    }

    pub fn apply_transaction(
        &mut self,
        transaction: UiInvalidationTransaction,
    ) -> Result<Option<UiInvalidationCommit>, UiInvalidationApplyError> {
        self.validate_transaction(&transaction)?;
        if transaction.is_empty() {
            return Ok(None);
        }

        let base_generation = transaction.base_generation;
        let mut dirty = UiDirtyFlags::default();
        let mut touched_domains = 0u8;
        let changed_nodes = transaction
            .changes
            .into_values()
            .inspect(|change| {
                merge_dirty(&mut dirty, change.dirty);
                for reason in &change.reasons {
                    touched_domains |= reason.domain_bit();
                }
            })
            .collect::<Vec<_>>();

        self.generations.generation = self.generations.generation.saturating_add(1);
        for reason in UiInvalidationReason::ALL {
            if touched_domains & reason.domain_bit() != 0 {
                advance_domain(&mut self.generations, reason);
            }
        }

        let commit = UiInvalidationCommit {
            base_generation,
            generation: self.generations.generation,
            generations: self.generations,
            dirty,
            changed_nodes,
        };
        self.last_commit = Some(commit.clone());
        Ok(Some(commit))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiInvalidationApplyError {
    StaleGeneration { expected: u64, actual: u64 },
}

impl fmt::Display for UiInvalidationApplyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StaleGeneration { expected, actual } => write!(
                formatter,
                "UI invalidation transaction expected generation {expected}, current generation is {actual}"
            ),
        }
    }
}

impl std::error::Error for UiInvalidationApplyError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UiSurfaceInvalidationApplyError {
    InvalidTransaction(UiInvalidationApplyError),
    Tree(UiTreeError),
}

impl fmt::Display for UiSurfaceInvalidationApplyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTransaction(error) => error.fmt(formatter),
            Self::Tree(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for UiSurfaceInvalidationApplyError {}

impl From<UiInvalidationApplyError> for UiSurfaceInvalidationApplyError {
    fn from(error: UiInvalidationApplyError) -> Self {
        Self::InvalidTransaction(error)
    }
}

impl From<UiTreeError> for UiSurfaceInvalidationApplyError {
    fn from(error: UiTreeError) -> Self {
        Self::Tree(error)
    }
}

fn record_change(
    changes: &mut BTreeMap<UiNodeId, UiInvalidationChange>,
    node_id: UiNodeId,
    dirty: UiDirtyFlags,
    reasons: impl IntoIterator<Item = UiInvalidationReason>,
) {
    let change = changes
        .entry(node_id)
        .or_insert_with(|| UiInvalidationChange {
            node_id,
            dirty: UiDirtyFlags::default(),
            reasons: BTreeSet::new(),
        });
    merge_dirty(&mut change.dirty, dirty);
    change.reasons.extend(reasons);
}

fn record_reason(
    changes: &mut BTreeMap<UiNodeId, UiInvalidationChange>,
    node_id: UiNodeId,
    reason: UiInvalidationReason,
) {
    let dirty = reason.dirty_flags();
    let mut reasons = reasons_for_dirty(dirty);
    reasons.insert(reason);
    record_change(changes, node_id, dirty, reasons);
}

fn reasons_for_dirty(dirty: UiDirtyFlags) -> BTreeSet<UiInvalidationReason> {
    let mut reasons = BTreeSet::new();
    let rebuilds_layout = dirty.layout || dirty.style || dirty.text || dirty.visible_range;
    if rebuilds_layout {
        reasons.insert(UiInvalidationReason::Layout);
        reasons.insert(UiInvalidationReason::HitTest);
        reasons.insert(UiInvalidationReason::Render);
    }
    if dirty.text {
        reasons.insert(UiInvalidationReason::Text);
    }
    if dirty.hit_test {
        reasons.insert(UiInvalidationReason::HitTest);
    }
    if dirty.render {
        reasons.insert(UiInvalidationReason::Render);
    }
    if dirty.input {
        reasons.insert(UiInvalidationReason::Interaction);
        reasons.insert(UiInvalidationReason::HitTest);
    }
    if dirty.style {
        reasons.insert(UiInvalidationReason::Resource);
    }
    if dirty.layout && dirty.hit_test && dirty.render && dirty.input {
        reasons.insert(UiInvalidationReason::Structure);
    }
    reasons
}

fn advance_domain(generations: &mut UiInvalidationGenerations, reason: UiInvalidationReason) {
    let domain = match reason {
        UiInvalidationReason::Structure => &mut generations.structure,
        UiInvalidationReason::Layout => &mut generations.layout,
        UiInvalidationReason::Text => &mut generations.text,
        UiInvalidationReason::HitTest => &mut generations.hit_test,
        UiInvalidationReason::Render => &mut generations.render,
        UiInvalidationReason::Interaction => &mut generations.interaction,
        UiInvalidationReason::Resource => &mut generations.resource,
    };
    *domain = domain.saturating_add(1);
}

fn merge_dirty(target: &mut UiDirtyFlags, dirty: UiDirtyFlags) {
    target.layout |= dirty.layout;
    target.hit_test |= dirty.hit_test;
    target.render |= dirty.render;
    target.style |= dirty.style;
    target.text |= dirty.text;
    target.input |= dirty.input;
    target.visible_range |= dirty.visible_range;
}
