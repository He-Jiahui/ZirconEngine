#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HybridGiInputSet {
    pub(crate) depth: bool,
    pub(crate) normal: bool,
    pub(crate) roughness: bool,
    pub(crate) base_color: bool,
    pub(crate) emissive: bool,
    pub(crate) history_validity: bool,
    pub(crate) motion_vectors: bool,
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
