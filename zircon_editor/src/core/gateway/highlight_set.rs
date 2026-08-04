use std::collections::BTreeSet;

use zircon_runtime_interface::ZrRuntimeViewportHandle;

/// Editor-side value object for one runtime overlay submission.
///
/// Authoring state is projected before it crosses the gateway; the runtime
/// receives only entity identifiers plus render attributes.
#[derive(Clone, Debug, PartialEq)]
pub struct EditorRuntimeHighlightSet {
    viewport: ZrRuntimeViewportHandle,
    generation: u64,
    entities: Vec<u64>,
    outline_enabled: bool,
    tint_rgba: [f32; 4],
}

impl EditorRuntimeHighlightSet {
    pub fn new(
        viewport: ZrRuntimeViewportHandle,
        generation: u64,
        entities: impl IntoIterator<Item = u64>,
        outline_enabled: bool,
        tint_rgba: [f32; 4],
    ) -> Self {
        Self {
            viewport,
            generation,
            entities: entities
                .into_iter()
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect(),
            outline_enabled,
            tint_rgba,
        }
    }

    pub const fn viewport(&self) -> ZrRuntimeViewportHandle {
        self.viewport
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub fn entities(&self) -> &[u64] {
        &self.entities
    }

    pub const fn outline_enabled(&self) -> bool {
        self.outline_enabled
    }

    pub const fn tint_rgba(&self) -> [f32; 4] {
        self.tint_rgba
    }

    pub fn is_valid(&self) -> bool {
        self.viewport.is_valid() && self.tint_rgba.iter().all(|component| component.is_finite())
    }
}

#[cfg(test)]
mod tests {
    use super::EditorRuntimeHighlightSet;
    use zircon_runtime_interface::ZrRuntimeViewportHandle;

    #[test]
    fn canonicalizes_entity_ids_before_crossing_gateway() {
        let set = EditorRuntimeHighlightSet::new(
            ZrRuntimeViewportHandle::new(2),
            4,
            [7, 1, 7, 3],
            true,
            [0.3, 0.5, 0.8, 1.0],
        );
        assert_eq!(set.entities(), &[1, 3, 7]);
    }
}
