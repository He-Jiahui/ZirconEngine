use std::slice;

use crate::scene::EntityId;

use super::QueryEntityError;

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

pub(crate) fn first_duplicate_entity(entities: &[EntityId]) -> Option<EntityId> {
    for current in 0..entities.len() {
        for previous in 0..current {
            if entities[current] == entities[previous] {
                return Some(entities[current]);
            }
        }
    }
    None
}

pub(crate) fn validate_unique_entities(entities: &[EntityId]) -> Result<(), QueryEntityError> {
    if let Some(entity) = first_duplicate_entity(entities) {
        return Err(QueryEntityError::DuplicateEntity(entity));
    }
    Ok(())
}
