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
        let mut entities = entities.into_iter().collect::<Vec<_>>();
        entities.sort_unstable();
        entities.dedup();
        Self {
            entities,
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
#[path = "tests/highlight_set.rs"]
mod tests;
