use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use crate::core::framework::render::{
    RenderFrameSubmissionReceipt, RenderViewportHandle, RenderViewportProduct,
};
use crate::graphics::GraphicsError;
use zr_rhi_wgpu::{
    WgpuUiExternalImage, WgpuUiExternalImageCopyReceipt, WgpuUiSurfaceExternalImageProvider,
};

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
    cache_revision: AtomicU64,
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
    fn cache_revision(&self) -> u64 {
        self.cache_revision.load(Ordering::Acquire)
    }

    pub(in crate::graphics::runtime::render_framework) fn publish(
        &self,
        viewport: RenderViewportHandle,
        copy: WgpuUiExternalImageCopyReceipt,
        scene_receipt: &RenderFrameSubmissionReceipt,
    ) -> Result<(), GraphicsError> {
        let descriptor =
            RenderViewportProduct::new(viewport, copy.width(), copy.height(), copy.generation());
        let product_submission = copy.submission();
        scene_receipt
            .validate_viewport_product_publication(copy.generation(), product_submission)
            .map_err(|source| GraphicsError::FrameProductPublicationFailed {
                receipt: scene_receipt.clone(),
                product_submission: Some(product_submission),
                source: Box::new(source.into()),
            })?;
        let image = copy.into_image();
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
        self.cache_revision.fetch_add(1, Ordering::AcqRel);
        Ok(())
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
        let changed = if let Some(product) = products.by_viewport.remove(&viewport) {
            for resource_key in product.resource_keys {
                products.by_resource_key.remove(&resource_key);
            }
            true
        } else {
            false
        };
        if products.direct_viewports.remove(&viewport).is_some() || changed {
            self.cache_revision.fetch_add(1, Ordering::AcqRel);
        }
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
        let changed = !products.by_viewport.is_empty()
            || !products.by_resource_key.is_empty()
            || !products.direct_viewports.is_empty();
        products.by_viewport.clear();
        products.by_resource_key.clear();
        products.direct_viewports.clear();
        if changed {
            self.cache_revision.fetch_add(1, Ordering::AcqRel);
        }
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

    fn cache_revision(&self) -> Option<u64> {
        Some(self.products.cache_revision())
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
#[path = "tests/viewport_product_registry.rs"]
mod tests;
