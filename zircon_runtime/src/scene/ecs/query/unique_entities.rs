use std::collections::HashSet;
use std::slice;

use crate::scene::EntityId;

use super::QueryEntityError;

const INLINE_UNIQUE_ENTITY_SCAN_LIMIT: usize = 16;

/// Fixed-size entity list that has been validated to contain no duplicate ids.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UniqueEntityArray<const N: usize> {
    entities: [EntityId; N],
}

impl<const N: usize> UniqueEntityArray<N> {
    pub fn new(entities: [EntityId; N]) -> Result<Self, QueryEntityError> {
        validate_unique_entities(&entities)?;
        Ok(Self { entities })
    }

    /// Creates a unique entity array without checking for duplicate ids.
    ///
    /// # Safety
    ///
    /// `entities` must not contain duplicate ids.
    pub const unsafe fn from_unique_unchecked(entities: [EntityId; N]) -> Self {
        Self { entities }
    }

    pub fn as_slice(&self) -> &[EntityId] {
        &self.entities
    }

    pub fn into_inner(self) -> [EntityId; N] {
        self.entities
    }
}

impl<const N: usize> TryFrom<[EntityId; N]> for UniqueEntityArray<N> {
    type Error = QueryEntityError;

    fn try_from(entities: [EntityId; N]) -> Result<Self, Self::Error> {
        Self::new(entities)
    }
}

impl<const N: usize> IntoIterator for UniqueEntityArray<N> {
    type IntoIter = std::array::IntoIter<EntityId, N>;
    type Item = EntityId;

    fn into_iter(self) -> Self::IntoIter {
        self.entities.into_iter()
    }
}

impl<'entity, const N: usize> IntoIterator for &'entity UniqueEntityArray<N> {
    type IntoIter = slice::Iter<'entity, EntityId>;
    type Item = &'entity EntityId;

    fn into_iter(self) -> Self::IntoIter {
        self.entities.iter()
    }
}

pub(crate) fn first_duplicate_entity<const N: usize>(entities: &[EntityId; N]) -> Option<EntityId> {
    if N <= INLINE_UNIQUE_ENTITY_SCAN_LIMIT {
        for current in 0..N {
            for previous in 0..current {
                if entities[current] == entities[previous] {
                    return Some(entities[current]);
                }
            }
        }
        return None;
    }

    first_duplicate_entity_hashed(entities)
}

fn first_duplicate_entity_hashed<const N: usize>(entities: &[EntityId; N]) -> Option<EntityId> {
    let mut seen = HashSet::with_capacity(N);
    for &entity in entities {
        if !seen.insert(entity) {
            return Some(entity);
        }
    }
    None
}

#[cfg(test)]
fn first_duplicate_entity_sorted<const N: usize>(
    entities: &[EntityId; N],
    mut record_comparison: impl FnMut(),
) -> Option<EntityId> {
    let mut indexed: [(EntityId, usize); N] = std::array::from_fn(|index| (entities[index], index));
    indexed.sort_unstable_by(|left, right| {
        record_comparison();
        left.cmp(right)
    });
    indexed
        .windows(2)
        .filter(|pair| pair[0].0 == pair[1].0)
        .map(|pair| (pair[1].1, pair[1].0))
        .min_by_key(|(index, _)| *index)
        .map(|(_, entity)| entity)
}

pub(crate) fn validate_unique_entities<const N: usize>(
    entities: &[EntityId; N],
) -> Result<(), QueryEntityError> {
    if let Some(entity) = first_duplicate_entity(entities) {
        return Err(QueryEntityError::DuplicateEntity(entity));
    }
    Ok(())
}

#[cfg(test)]
#[path = "tests/unique_entities.rs"]
mod tests;

#[cfg(test)]
#[path = "unique_entities/tests/hash_scan_tests.rs"]
mod hash_scan_tests;
