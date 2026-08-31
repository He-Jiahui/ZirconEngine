use std::sync::Arc;

use crate::core::framework::render::{
    MaterialPropertyOverrideBlock, RenderMaterialPropertyUniformPayload,
};
use crate::core::math::Vec4;
use crate::core::resource::ResourceId;

use super::super::prepared::{
    PreparedMaterialBundle, PreparedMaterialTextureBinding, PreparedMaterialTextureResource,
};
use super::super::{
    GpuMaterialUniformResource, GpuTextureResource, MaterialCaptureSeed, MaterialRuntime,
    OutputTargetTextureResource,
};
use super::ResourceStreamer;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum MaterialDrawGenerationSelection {
    #[default]
    Published,
    PreviousPublished,
    ErrorProxy,
}

#[derive(Clone)]
pub(in crate::graphics::scene) enum PublishedMaterialTextureBinding {
    Texture(Arc<GpuTextureResource>),
    OutputTarget(Arc<OutputTargetTextureResource>),
}

#[derive(Clone)]
pub(in crate::graphics::scene) struct PublishedMaterialTextureSet {
    pub(in crate::graphics::scene) base_color: PublishedMaterialTextureBinding,
    pub(in crate::graphics::scene) normal: PublishedMaterialTextureBinding,
    pub(in crate::graphics::scene) metallic_roughness: PublishedMaterialTextureBinding,
    pub(in crate::graphics::scene) occlusion: PublishedMaterialTextureBinding,
    pub(in crate::graphics::scene) emissive: PublishedMaterialTextureBinding,
    pub(in crate::graphics::scene) clearcoat_normal: PublishedMaterialTextureBinding,
}

/// One immutable view of every draw-facing field in a published material generation.
///
/// Render extraction must keep this proxy intact when it later selects a context-qualified
/// last-good generation. Looking up runtime state and GPU bindings independently would allow
/// that selection to mix two generations in one cached draw command.
#[derive(Clone, Copy)]
pub(crate) struct PublishedMaterialDrawProxy<'a> {
    streamer: &'a ResourceStreamer,
    bundle: Option<&'a PreparedMaterialBundle>,
}

impl<'a> PublishedMaterialDrawProxy<'a> {
    fn new(
        streamer: &'a ResourceStreamer,
        id: &ResourceId,
        selection: MaterialDrawGenerationSelection,
    ) -> Self {
        let prepared = streamer.materials.get(id);
        let bundle = match selection {
            MaterialDrawGenerationSelection::Published => {
                prepared.and_then(|prepared| prepared.published.as_ref())
            }
            MaterialDrawGenerationSelection::PreviousPublished => {
                prepared.and_then(|prepared| prepared.previous_published.as_ref())
            }
            MaterialDrawGenerationSelection::ErrorProxy => None,
        };
        Self { streamer, bundle }
    }

    pub(crate) fn runtime(self) -> Option<&'a MaterialRuntime> {
        self.bundle.map(|bundle| &bundle.runtime)
    }

    pub(crate) fn capture_seed(self) -> Option<MaterialCaptureSeed> {
        let bundle = self.bundle?;
        let mut seed = bundle.runtime.capture_seed();
        (
            seed.base_color_texture_revision,
            seed.base_color_texture_center_rgba,
        ) = capture_texture_snapshot(&bundle.textures.base_color, seed.base_color_texture);
        (
            seed.normal_texture_revision,
            seed.normal_texture_center_rgba,
        ) = capture_texture_snapshot(&bundle.textures.normal, seed.normal_texture);
        (
            seed.metallic_roughness_texture_revision,
            seed.metallic_roughness_texture_center_rgba,
        ) = capture_texture_snapshot(
            &bundle.textures.metallic_roughness,
            seed.metallic_roughness_texture,
        );
        (
            seed.occlusion_texture_revision,
            seed.occlusion_texture_center_rgba,
        ) = capture_texture_snapshot(&bundle.textures.occlusion, seed.occlusion_texture);
        (
            seed.emissive_texture_revision,
            seed.emissive_texture_center_rgba,
        ) = capture_texture_snapshot(&bundle.textures.emissive, seed.emissive_texture);
        Some(seed)
    }

    pub(crate) fn draw_generation(self) -> Option<u64> {
        self.bundle.map(|bundle| bundle.draw_generation)
    }

    pub(crate) fn uniform(self) -> Arc<GpuMaterialUniformResource> {
        self.bundle
            .map(|bundle| Arc::clone(&bundle.uniform))
            .unwrap_or_else(|| Arc::clone(&self.streamer.fallback_material_uniform))
    }

    pub(crate) fn standard_uniform(self) -> Arc<GpuMaterialUniformResource> {
        self.bundle
            .map(|bundle| Arc::clone(&bundle.standard_uniform))
            .unwrap_or_else(|| Arc::clone(&self.streamer.fallback_standard_material_uniform))
    }

    pub(in crate::graphics::scene) fn textures(self) -> PublishedMaterialTextureSet {
        let Some(bundle) = self.bundle else {
            return PublishedMaterialTextureSet {
                base_color: PublishedMaterialTextureBinding::Texture(Arc::clone(
                    &self.streamer.fallback_texture,
                )),
                normal: PublishedMaterialTextureBinding::Texture(Arc::clone(
                    &self.streamer.fallback_normal_texture,
                )),
                metallic_roughness: PublishedMaterialTextureBinding::Texture(Arc::clone(
                    &self.streamer.fallback_texture,
                )),
                occlusion: PublishedMaterialTextureBinding::Texture(Arc::clone(
                    &self.streamer.fallback_texture,
                )),
                emissive: PublishedMaterialTextureBinding::Texture(Arc::clone(
                    &self.streamer.fallback_texture,
                )),
                clearcoat_normal: PublishedMaterialTextureBinding::Texture(Arc::clone(
                    &self.streamer.fallback_normal_texture,
                )),
            };
        };
        PublishedMaterialTextureSet {
            base_color: self.resolve_texture_binding(&bundle.textures.base_color),
            normal: self.resolve_texture_binding(&bundle.textures.normal),
            metallic_roughness: self.resolve_texture_binding(&bundle.textures.metallic_roughness),
            occlusion: self.resolve_texture_binding(&bundle.textures.occlusion),
            emissive: self.resolve_texture_binding(&bundle.textures.emissive),
            clearcoat_normal: self.resolve_texture_binding(&bundle.textures.clearcoat_normal),
        }
    }

    pub(crate) fn uniform_payload_with_overrides(
        self,
        overrides: &MaterialPropertyOverrideBlock,
    ) -> Option<RenderMaterialPropertyUniformPayload> {
        if overrides.is_empty() {
            return None;
        }
        self.runtime().map(|runtime| {
            runtime
                .shader_property_uniform_payload
                .with_override_block(overrides)
        })
    }

    fn resolve_texture_binding(
        self,
        binding: &PreparedMaterialTextureBinding,
    ) -> PublishedMaterialTextureBinding {
        match &binding.resource {
            PreparedMaterialTextureResource::Texture(snapshot) => {
                let resource = binding
                    .id
                    .and_then(|id| self.streamer.textures.get(&id))
                    .filter(|prepared| {
                        same_texture_revision(binding.revision, Some(prepared.revision))
                    })
                    .map(|prepared| Arc::clone(&prepared.resource))
                    .unwrap_or_else(|| Arc::clone(snapshot));
                PublishedMaterialTextureBinding::Texture(resource)
            }
            PreparedMaterialTextureResource::OutputTarget(snapshot) => {
                let resource = binding
                    .id
                    .and_then(|id| self.streamer.output_target_textures.get(&id))
                    .filter(|prepared| {
                        same_texture_revision(binding.revision, Some(prepared.revision))
                    })
                    .map(|prepared| Arc::clone(prepared.resource()))
                    .unwrap_or_else(|| Arc::clone(snapshot));
                PublishedMaterialTextureBinding::OutputTarget(resource)
            }
        }
    }
}

fn capture_texture_snapshot(
    binding: &PreparedMaterialTextureBinding,
    expected_id: Option<ResourceId>,
) -> (Option<u64>, Option<Vec4>) {
    if binding.id != expected_id {
        return (None, None);
    }
    (
        binding.revision,
        binding.capture_sample_rgba.map(Vec4::from_array),
    )
}

fn same_texture_revision(snapshot: Option<u64>, current: Option<u64>) -> bool {
    snapshot.is_some() && snapshot == current
}

impl ResourceStreamer {
    pub(crate) fn material_draw_generations(&self, id: &ResourceId) -> [Option<u64>; 3] {
        let Some(prepared) = self.materials.get(id) else {
            return [None, None, None];
        };
        [
            prepared
                .published
                .as_ref()
                .map(|bundle| bundle.draw_generation),
            prepared
                .previous_published
                .as_ref()
                .map(|bundle| bundle.draw_generation),
            prepared
                .staged_candidate
                .as_ref()
                .map(|bundle| bundle.draw_generation),
        ]
    }

    pub(crate) fn staged_material_draw_generation(&self, id: &ResourceId) -> Option<u64> {
        self.material_draw_generations(id)[2]
    }

    pub(crate) fn published_material_draw_proxy(
        &self,
        id: &ResourceId,
    ) -> PublishedMaterialDrawProxy<'_> {
        self.material_draw_proxy(id, MaterialDrawGenerationSelection::Published)
    }

    pub(crate) fn material_draw_proxy(
        &self,
        id: &ResourceId,
        selection: MaterialDrawGenerationSelection,
    ) -> PublishedMaterialDrawProxy<'_> {
        PublishedMaterialDrawProxy::new(self, id, selection)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn published_draw_proxy_is_the_only_non_test_bundle_projection() {
        let accessors = include_str!("resource_streamer_accessors.rs");
        let proxy = include_str!("published_material_draw_proxy.rs")
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("published material draw proxy test boundary");

        assert!(proxy.contains("prepared.published.as_ref()"));
        assert!(proxy.contains("prepared.previous_published.as_ref()"));
        assert!(proxy.contains("bundle.runtime"));
        assert!(proxy.contains("bundle.uniform"));
        assert!(proxy.contains("bundle.standard_uniform"));
        assert!(proxy.contains("bundle.textures"));
        assert!(proxy.contains("same_texture_revision"));
        assert!(!accessors.contains("pub(crate) fn published_material_uniform("));
        assert!(!accessors.contains("pub(crate) fn published_standard_material_uniform("));
    }

    #[test]
    fn same_revision_accepts_mip_streaming_but_rejects_another_asset_generation() {
        assert!(super::same_texture_revision(Some(7), Some(7)));
        assert!(!super::same_texture_revision(Some(7), Some(8)));
        assert!(!super::same_texture_revision(None, Some(7)));
    }

    #[test]
    fn resource_streamer_publishes_only_the_three_live_material_generation_slots() {
        let source = include_str!("published_material_draw_proxy.rs")
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("published material draw proxy test boundary");
        let generations = source
            .split("fn material_draw_generations(")
            .nth(1)
            .expect("live material generation projection");

        assert!(generations.contains("prepared.published"));
        assert!(generations.contains("prepared.previous_published"));
        assert!(generations.contains("prepared.staged_candidate"));
        assert!(source.contains("fn staged_material_draw_generation("));
    }
}
