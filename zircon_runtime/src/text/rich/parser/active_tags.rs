use std::collections::HashMap;

use crate::text::{LinkRef, StyleOverride};

use super::super::RichTextParseError;

#[derive(Clone, Debug)]
pub(super) struct ActiveTag {
    pub(super) name: String,
    pub(super) style: StyleOverride,
    pub(super) link: Option<LinkRef>,
    pub(super) source_range: (u32, u32),
}

pub(super) enum ActiveTagClose {
    NotFound,
    Closed { implicitly_closed: usize },
}

const ACTIVE_TAG_INDEX_THRESHOLD: usize = 32;

pub(super) struct ActiveTagStack {
    tags: Vec<ActiveTag>,
    positions: Option<HashMap<String, Vec<usize>>>,
    max_depth: usize,
}

impl ActiveTagStack {
    pub(super) fn new(max_depth: usize) -> Self {
        Self {
            tags: Vec::new(),
            positions: None,
            max_depth,
        }
    }

    pub(super) fn push(&mut self, tag: ActiveTag) -> Result<(), RichTextParseError> {
        let attempted_depth = self.tags.len() + 1;
        if attempted_depth > self.max_depth {
            return Err(RichTextParseError::ActiveTagDepthBudgetExceeded {
                attempted_depth,
                max_depth: self.max_depth,
            });
        }
        let position = self.tags.len();
        if let Some(positions) = self.positions.as_mut() {
            positions
                .entry(tag.name.clone())
                .or_default()
                .push(position);
        }
        self.tags.push(tag);
        if self.positions.is_none() && self.tags.len() > ACTIVE_TAG_INDEX_THRESHOLD {
            self.rebuild_positions();
        }
        Ok(())
    }

    pub(super) fn close(&mut self, name: &str) -> ActiveTagClose {
        let position = if let Some(positions) = self.positions.as_ref() {
            positions
                .get(name)
                .and_then(|positions| positions.last())
                .copied()
        } else {
            self.tags.iter().rposition(|active| active.name == name)
        };
        let Some(position) = position else {
            return ActiveTagClose::NotFound;
        };
        let implicitly_closed = self.tags.len().saturating_sub(position).saturating_sub(1);

        while self.tags.len() > position {
            let removed_position = self.tags.len() - 1;
            let removed = self.tags.pop().expect("active tag length checked");
            let Some(positions) = self.positions.as_mut() else {
                continue;
            };
            let remove_name = {
                let name_positions = positions
                    .get_mut(&removed.name)
                    .expect("indexed active tag must have a position");
                debug_assert_eq!(name_positions.pop(), Some(removed_position));
                name_positions.is_empty()
            };
            if remove_name {
                positions.remove(&removed.name);
            }
        }
        ActiveTagClose::Closed { implicitly_closed }
    }

    pub(super) fn source_ranges(&self) -> impl Iterator<Item = (u32, u32)> + '_ {
        self.tags.iter().map(|tag| tag.source_range)
    }

    fn rebuild_positions(&mut self) {
        let mut positions: HashMap<String, Vec<usize>> = HashMap::new();
        for (position, active) in self.tags.iter().enumerate() {
            positions
                .entry(active.name.clone())
                .or_default()
                .push(position);
        }
        self.positions = Some(positions);
    }
}

pub(super) fn current_style(active_tags: &ActiveTagStack) -> StyleOverride {
    active_tags
        .tags
        .last()
        .map(|active| &active.style)
        .cloned()
        .unwrap_or_default()
}

pub(super) fn current_link(active_tags: &ActiveTagStack) -> Option<LinkRef> {
    active_tags
        .tags
        .last()
        .and_then(|active| active.link.clone())
}
