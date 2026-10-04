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
        let mut entities = entities.into_iter().collect::<Vec<_>>();
        entities.sort_unstable();
        entities.dedup();
        Self {
            viewport,
            generation,
            entities,
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

    /// 两种网关提交前共用的边界检查；视口须有效且颜色分量必须为有限值。
    pub fn is_valid(&self) -> bool {
        self.viewport.is_valid() && self.tint_rgba.iter().all(|component| component.is_finite())
    }
}

#[cfg(test)]
#[path = "tests/highlight_set_optimization_tests.rs"]
mod optimization_tests;
