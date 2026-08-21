use std::collections::BTreeSet;

use crate::core::framework::scene::EntityId;

/// Runtime-owned, editor-neutral overlay input for one viewport frame.
///
/// Entity IDs are canonicalized on construction so every consumer observes a
/// stable order regardless of the editor-side container that produced them.
#[derive(Clone, Debug, PartialEq)]
pub struct HighlightSet {
    entities: Vec<EntityId>,
    attributes: HighlightRenderAttributes,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HighlightRenderAttributes {
    pub outline_enabled: bool,
    pub tint_rgba: [f32; 4],
}

impl HighlightRenderAttributes {
    pub const fn outlined(tint_rgba: [f32; 4]) -> Self {
        Self {
            outline_enabled: true,
            tint_rgba,
        }
    }

    pub fn is_valid(self) -> bool {
        self.tint_rgba.iter().all(|component| component.is_finite())
    }
}

impl HighlightSet {
    pub fn new(
        entities: impl IntoIterator<Item = EntityId>,
        attributes: HighlightRenderAttributes,
    ) -> Self {
        Self {
            entities: entities
                .into_iter()
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect(),
            attributes,
        }
    }

    pub fn entities(&self) -> &[EntityId] {
        &self.entities
    }

    pub(crate) fn entity_capacity(&self) -> usize {
        self.entities.capacity()
    }

    pub const fn attributes(&self) -> HighlightRenderAttributes {
        self.attributes
    }
}

#[cfg(test)]
mod tests {
    use super::{HighlightRenderAttributes, HighlightSet};

    #[test]
    fn canonicalizes_entity_order_and_duplicates() {
        let set = HighlightSet::new(
            [9, 2, 9, 4],
            HighlightRenderAttributes::outlined([0.2, 0.4, 0.6, 1.0]),
        );

        assert_eq!(set.entities(), &[2, 4, 9]);
        assert!(set.entity_capacity() >= set.entities().len());
    }
}
