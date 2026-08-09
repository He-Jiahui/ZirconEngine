use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use crate::core::framework::render::{RenderViewportHandle, RenderViewportProduct};
use crate::graphics::ViewportFrameTextureHandle;
use zr_rhi_wgpu::{WgpuUiExternalImage, WgpuUiSurfaceContext, WgpuUiSurfaceExternalImageProvider};

/// Bounded owner for runtime products exported to same-device retained UI presenters.
///
/// The registry keeps a short generation ring per viewport so a UI command can resolve the frame
/// it observed even when the renderer has already produced the next one. It never owns CPU pixels
/// or waits for GPU completion.
const MAX_RETAINED_VIEWPORT_PRODUCT_GENERATIONS: usize = 3;

#[derive(Default)]
pub(in crate::graphics::runtime::render_framework) struct ViewportProductRegistry {
    products: Mutex<ViewportProductRegistryState>,
    direct_presenter_count: AtomicUsize,
}

#[derive(Default)]
struct ViewportProductRegistryState {
    by_viewport: HashMap<RenderViewportHandle, ViewportProductEntry>,
    by_resource_key: HashMap<String, RetainedViewportProduct>,
    direct_viewports: HashMap<RenderViewportHandle, usize>,
}

struct ViewportProductEntry {
    descriptor: RenderViewportProduct,
    resource_keys: VecDeque<String>,
}

struct RetainedViewportProduct {
    viewport: RenderViewportHandle,
    generation: u64,
    image: WgpuUiExternalImage,
}

impl ViewportProductRegistry {
    pub(in crate::graphics::runtime::render_framework) fn publish(
        &self,
        viewport: RenderViewportHandle,
        texture: ViewportFrameTextureHandle,
        ui_context: &WgpuUiSurfaceContext,
    ) -> RenderViewportProduct {
        debug_assert!(
            texture.usage.contains(wgpu::TextureUsages::COPY_SRC),
            "renderer products must be copy sources before retained UI export"
        );
        let descriptor =
            RenderViewportProduct::new(viewport, texture.width, texture.height, texture.generation);
        // This must be a new GPU texture. The scene renderer reuses its final-color target on the
        // next frame and may also serve another viewport of the same size.
        let image = ui_context.copy_texture_for_external_image(
            &texture.texture,
            texture.width,
            texture.height,
            texture.format,
            texture.generation,
        );
        let mut products = self
            .products
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let expired_key = {
            let product =
                products
                    .by_viewport
                    .entry(viewport)
                    .or_insert_with(|| ViewportProductEntry {
                        descriptor: descriptor.clone(),
                        resource_keys: VecDeque::new(),
                    });
            product.descriptor = descriptor.clone();
            retain_resource_key(
                &mut product.resource_keys,
                descriptor.resource_key().to_owned(),
            )
        };
        products.by_resource_key.insert(
            descriptor.resource_key().to_owned(),
            RetainedViewportProduct {
                viewport,
                generation: descriptor.generation(),
                image,
            },
        );
        if let Some(expired) = expired_key {
            products.by_resource_key.remove(&expired);
        }
        descriptor
    }

    pub(in crate::graphics::runtime::render_framework) fn poll_if_newer(
        &self,
        viewport: RenderViewportHandle,
        last_generation: Option<u64>,
    ) -> Option<RenderViewportProduct> {
        let products = self
            .products
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let product = products.by_viewport.get(&viewport)?.descriptor.clone();
        (last_generation.is_none_or(|generation| product.generation() > generation))
            .then_some(product)
    }

    pub(in crate::graphics::runtime::render_framework) fn remove(
        &self,
        viewport: RenderViewportHandle,
    ) {
        let mut products = self
            .products
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(product) = products.by_viewport.remove(&viewport) {
            for resource_key in product.resource_keys {
                products.by_resource_key.remove(&resource_key);
            }
        }
        products.direct_viewports.remove(&viewport);
    }

    pub(in crate::graphics::runtime::render_framework) fn requires_async_capture(
        &self,
        viewport: RenderViewportHandle,
    ) -> bool {
        self.direct_presenter_count.load(Ordering::Acquire) == 0
            || !self
                .products
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .direct_viewports
                .contains_key(&viewport)
    }

    pub(in crate::graphics::runtime::render_framework) fn has_direct_presenter(&self) -> bool {
        self.direct_presenter_count.load(Ordering::Acquire) != 0
    }

    fn resolve(&self, resource_key: &str, generation: u64) -> Option<WgpuUiExternalImage> {
        let products = self
            .products
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let product = products.by_resource_key.get(resource_key)?;
        (product.generation == generation).then_some(())?;
        let image = product.image.clone();
        Some(image)
    }

    #[cfg(test)]
    fn mark_direct_viewport_for_test(&self, viewport: RenderViewportHandle) {
        self.add_direct_consumer(viewport);
    }

    fn viewport_for_resource(
        &self,
        resource_key: &str,
        generation: u64,
    ) -> Option<RenderViewportHandle> {
        let products = self
            .products
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        products
            .by_resource_key
            .get(resource_key)
            .filter(|product| product.generation == generation)
            .map(|product| product.viewport)
    }

    fn add_direct_consumer(&self, viewport: RenderViewportHandle) {
        let mut products = self
            .products
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *products.direct_viewports.entry(viewport).or_default() += 1;
    }

    fn release_direct_consumers(&self, viewports: HashSet<RenderViewportHandle>) {
        let mut products = self
            .products
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        for viewport in viewports {
            let remaining = products
                .direct_viewports
                .get(&viewport)
                .copied()
                .unwrap_or_default()
                .saturating_sub(1);
            if remaining == 0 {
                products.direct_viewports.remove(&viewport);
            } else {
                products.direct_viewports.insert(viewport, remaining);
            }
        }
    }

    fn clear_after_last_direct_presenter(&self) {
        let mut products = self
            .products
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        products.by_viewport.clear();
        products.by_resource_key.clear();
        products.direct_viewports.clear();
    }
}

fn retain_resource_key(
    resource_keys: &mut VecDeque<String>,
    resource_key: String,
) -> Option<String> {
    resource_keys.retain(|key| key != &resource_key);
    resource_keys.push_back(resource_key);
    (resource_keys.len() > MAX_RETAINED_VIEWPORT_PRODUCT_GENERATIONS)
        .then(|| resource_keys.pop_front())
        .flatten()
}

pub(in crate::graphics::runtime::render_framework) struct WgpuViewportProductProvider {
    products: Arc<ViewportProductRegistry>,
    confirmed_viewports: Mutex<HashSet<RenderViewportHandle>>,
}

impl WgpuViewportProductProvider {
    pub(in crate::graphics::runtime::render_framework) fn new(
        products: Arc<ViewportProductRegistry>,
    ) -> Self {
        products
            .direct_presenter_count
            .fetch_add(1, Ordering::AcqRel);
        Self {
            products,
            confirmed_viewports: Mutex::new(HashSet::new()),
        }
    }

    fn confirm_viewport(&self, viewport: RenderViewportHandle) {
        let mut confirmed = self
            .confirmed_viewports
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if confirmed.insert(viewport) {
            self.products.add_direct_consumer(viewport);
        }
    }

    #[cfg(test)]
    fn confirm_viewport_for_test(&self, viewport: RenderViewportHandle) {
        self.confirm_viewport(viewport);
    }
}

impl Drop for WgpuViewportProductProvider {
    fn drop(&mut self) {
        let confirmed_viewports = std::mem::take(
            &mut *self
                .confirmed_viewports
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
        );
        self.products.release_direct_consumers(confirmed_viewports);
        if self
            .products
            .direct_presenter_count
            .fetch_sub(1, Ordering::AcqRel)
            == 1
        {
            self.products.clear_after_last_direct_presenter();
        }
    }
}

impl WgpuUiSurfaceExternalImageProvider for WgpuViewportProductProvider {
    fn resolve(&self, resource_key: &str, generation: u64) -> Option<WgpuUiExternalImage> {
        self.products.resolve(resource_key, generation)
    }

    fn confirm_resident(&self, resource_key: &str, generation: u64) {
        if let Some(viewport) = self
            .products
            .viewport_for_resource(resource_key, generation)
        {
            self.confirm_viewport(viewport);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direct_presenter_keeps_capture_until_its_viewport_is_resolved() {
        let products = Arc::new(ViewportProductRegistry::default());
        let direct_viewport = RenderViewportHandle::new(7);
        let fallback_viewport = RenderViewportHandle::new(8);
        assert!(products.requires_async_capture(direct_viewport));

        let provider = WgpuViewportProductProvider::new(Arc::clone(&products));
        assert!(products.requires_async_capture(direct_viewport));
        products.mark_direct_viewport_for_test(direct_viewport);
        assert!(!products.requires_async_capture(direct_viewport));
        assert!(products.requires_async_capture(fallback_viewport));
        drop(provider);

        assert!(products.requires_async_capture(direct_viewport));
    }

    #[test]
    fn direct_consumers_are_reference_counted_per_presenter() {
        let products = Arc::new(ViewportProductRegistry::default());
        let viewport = RenderViewportHandle::new(7);
        let first = WgpuViewportProductProvider::new(Arc::clone(&products));
        let second = WgpuViewportProductProvider::new(Arc::clone(&products));

        first.confirm_viewport_for_test(viewport);
        second.confirm_viewport_for_test(viewport);
        assert!(!products.requires_async_capture(viewport));
        drop(first);
        assert!(!products.requires_async_capture(viewport));
        drop(second);
        assert!(products.requires_async_capture(viewport));
    }

    #[test]
    fn product_registry_exports_independent_gpu_snapshots() {
        let source = include_str!("viewport_product_registry.rs");

        assert!(source.contains("copy_texture_for_external_image"));
        assert!(source.contains("image: WgpuUiExternalImage"));
        assert!(!source.contains("texture: texture,"));
        assert!(source.contains("products.by_viewport.clear()"));
        assert!(source.contains("products.by_resource_key.clear()"));
    }

    #[test]
    fn resource_keys_keep_a_bounded_generation_ring() {
        let mut resource_keys = VecDeque::new();

        for generation in 1..=MAX_RETAINED_VIEWPORT_PRODUCT_GENERATIONS {
            assert!(
                retain_resource_key(&mut resource_keys, format!("viewport:7:{generation}"),)
                    .is_none()
            );
        }

        assert_eq!(
            retain_resource_key(&mut resource_keys, "viewport:7:4".to_string()),
            Some("viewport:7:1".to_string())
        );
        assert_eq!(
            resource_keys.into_iter().collect::<Vec<_>>(),
            vec!["viewport:7:2", "viewport:7:3", "viewport:7:4"]
        );
    }
}
