#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HybridGiInputSet {
    depth: bool,
    normal: bool,
    roughness: bool,
    base_color: bool,
    emissive: bool,
    history_validity: bool,
    motion_vectors: bool,
}

impl HybridGiInputSet {
    pub(crate) fn deferred() -> Self {
        Self::complete()
    }

    #[cfg(test)]
    pub(crate) fn forward_plus() -> Self {
        Self::complete()
    }

    #[cfg(test)]
    pub(crate) fn is_complete(&self) -> bool {
        self.depth
            && self.normal
            && self.roughness
            && self.base_color
            && self.emissive
            && self.history_validity
            && self.motion_vectors
    }

    #[cfg(test)]
    pub(crate) const fn required_input_count(&self) -> usize {
        7
    }

    const fn complete() -> Self {
        Self {
            depth: true,
            normal: true,
            roughness: true,
            base_color: true,
            emissive: true,
            history_validity: true,
            motion_vectors: true,
        }
    }
}
