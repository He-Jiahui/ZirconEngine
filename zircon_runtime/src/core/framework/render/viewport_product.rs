use super::RenderViewportHandle;

/// Backend-neutral identity for a GPU-resident viewport presentation product.
///
/// The descriptor intentionally contains no native texture handle. The render backend retains
/// that owner behind the resource key; consumers can safely carry this value across the
/// runtime/editor boundary and fall back to an explicit CPU capture when no matching presenter
/// product is available.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderViewportProduct {
    resource_key: String,
    width: u32,
    height: u32,
    generation: u64,
}

impl RenderViewportProduct {
    pub fn new(viewport: RenderViewportHandle, width: u32, height: u32, generation: u64) -> Self {
        Self {
            resource_key: format!("viewport:{}:{generation}", viewport.raw()),
            width,
            height,
            generation,
        }
    }

    pub fn resource_key(&self) -> &str {
        &self.resource_key
    }

    pub const fn width(&self) -> u32 {
        self.width
    }

    pub const fn height(&self) -> u32 {
        self.height
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub const fn is_valid(&self) -> bool {
        !self.resource_key.is_empty() && self.width != 0 && self.height != 0 && self.generation != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn product_identity_is_generation_scoped_and_backend_neutral() {
        let first = RenderViewportProduct::new(RenderViewportHandle::new(7), 640, 360, 3);
        let next = RenderViewportProduct::new(RenderViewportHandle::new(7), 640, 360, 4);

        assert_eq!(first.resource_key(), "viewport:7:3");
        assert_ne!(first.resource_key(), next.resource_key());
        assert!(first.is_valid());
    }
}
