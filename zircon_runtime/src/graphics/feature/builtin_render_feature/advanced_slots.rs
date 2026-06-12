use super::super::render_feature_capability_requirement::RenderFeatureCapabilityRequirement;
use super::builtin_render_feature::BuiltinRenderFeature;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct AdvancedBuiltinFeatureSlot {
    feature: BuiltinRenderFeature,
    descriptor_name: &'static str,
    extract_section: Option<&'static str>,
    capability_requirement: Option<RenderFeatureCapabilityRequirement>,
    requires_capability_opt_in: bool,
}

impl AdvancedBuiltinFeatureSlot {
    const fn new(
        feature: BuiltinRenderFeature,
        descriptor_name: &'static str,
        extract_section: &'static str,
    ) -> Self {
        Self {
            feature,
            descriptor_name,
            extract_section: Some(extract_section),
            capability_requirement: None,
            requires_capability_opt_in: false,
        }
    }

    const fn with_capability_requirement(
        mut self,
        requirement: RenderFeatureCapabilityRequirement,
    ) -> Self {
        self.capability_requirement = Some(requirement);
        self
    }

    const fn with_capability_opt_in_required(mut self) -> Self {
        self.requires_capability_opt_in = true;
        self
    }

    #[cfg(test)]
    pub(crate) const fn feature(&self) -> BuiltinRenderFeature {
        self.feature
    }

    pub(crate) const fn descriptor_name(&self) -> &'static str {
        self.descriptor_name
    }

    pub(crate) const fn extract_section(&self) -> Option<&'static str> {
        self.extract_section
    }

    pub(crate) const fn capability_requirement(
        &self,
    ) -> Option<RenderFeatureCapabilityRequirement> {
        self.capability_requirement
    }

    #[cfg(test)]
    pub(crate) const fn requires_capability_opt_in(&self) -> bool {
        self.requires_capability_opt_in
    }
}

pub(crate) const DESCRIPTOR_ONLY_ADVANCED_SLOTS: &[AdvancedBuiltinFeatureSlot] = &[
    AdvancedBuiltinFeatureSlot::new(
        BuiltinRenderFeature::SkinnedMesh,
        "skinned_mesh",
        "skinned_mesh",
    ),
    AdvancedBuiltinFeatureSlot::new(BuiltinRenderFeature::MeshLod, "mesh_lod", "mesh_lod"),
    AdvancedBuiltinFeatureSlot::new(
        BuiltinRenderFeature::ReflectionProbes,
        "reflection_probes",
        "reflection_probes",
    ),
    AdvancedBuiltinFeatureSlot::new(
        BuiltinRenderFeature::BakedLighting,
        "baked_lighting",
        "baked_lighting",
    ),
    AdvancedBuiltinFeatureSlot::new(BuiltinRenderFeature::Particle, "particle", "particles"),
    AdvancedBuiltinFeatureSlot::new(
        BuiltinRenderFeature::SparseTexture,
        "sparse_texture",
        "sparse_texture",
    )
    .with_capability_requirement(RenderFeatureCapabilityRequirement::SparseTexture)
    .with_capability_opt_in_required(),
    AdvancedBuiltinFeatureSlot::new(BuiltinRenderFeature::Terrain, "terrain", "terrain"),
    AdvancedBuiltinFeatureSlot::new(BuiltinRenderFeature::Tree, "tree", "tree"),
    AdvancedBuiltinFeatureSlot::new(BuiltinRenderFeature::Decal, "decals", "decals"),
    AdvancedBuiltinFeatureSlot::new(BuiltinRenderFeature::Projector, "projector", "projector"),
    AdvancedBuiltinFeatureSlot::new(BuiltinRenderFeature::Halo, "halo", "halo"),
    AdvancedBuiltinFeatureSlot::new(BuiltinRenderFeature::LensFlare, "lens_flare", "lens_flare"),
    AdvancedBuiltinFeatureSlot::new(BuiltinRenderFeature::Trail, "trail", "trail"),
    AdvancedBuiltinFeatureSlot::new(BuiltinRenderFeature::Billboard, "billboard", "billboard"),
    AdvancedBuiltinFeatureSlot::new(BuiltinRenderFeature::Tilemap, "tilemap", "tilemap"),
    AdvancedBuiltinFeatureSlot::new(
        BuiltinRenderFeature::TextShaping,
        "text_shaping",
        "text_shaping",
    ),
    AdvancedBuiltinFeatureSlot::new(BuiltinRenderFeature::Skybox, "skybox", "skybox"),
    AdvancedBuiltinFeatureSlot::new(BuiltinRenderFeature::Cubemap, "cubemap", "cubemap"),
    AdvancedBuiltinFeatureSlot::new(
        BuiltinRenderFeature::Texture2dArray,
        "texture_2d_array",
        "texture_2d_array",
    ),
    AdvancedBuiltinFeatureSlot::new(BuiltinRenderFeature::NormalMap, "normal_map", "normal_map"),
    AdvancedBuiltinFeatureSlot::new(BuiltinRenderFeature::Mipmap, "mipmap", "mipmap"),
    AdvancedBuiltinFeatureSlot::new(
        BuiltinRenderFeature::ColorSpace,
        "color_space",
        "color_space",
    ),
];

#[cfg(test)]
pub(crate) fn descriptor_only_advanced_slots() -> &'static [AdvancedBuiltinFeatureSlot] {
    DESCRIPTOR_ONLY_ADVANCED_SLOTS
}

pub(crate) fn descriptor_only_advanced_slot(
    feature: BuiltinRenderFeature,
) -> Option<&'static AdvancedBuiltinFeatureSlot> {
    DESCRIPTOR_ONLY_ADVANCED_SLOTS
        .iter()
        .find(|slot| slot.feature == feature)
}

pub(crate) fn is_descriptor_only_advanced_slot(feature: BuiltinRenderFeature) -> bool {
    descriptor_only_advanced_slot(feature).is_some()
}

pub(crate) fn descriptor_only_advanced_slot_requires_capability_opt_in(
    feature: BuiltinRenderFeature,
    requirement: RenderFeatureCapabilityRequirement,
) -> bool {
    descriptor_only_advanced_slot(feature).is_some_and(|slot| {
        slot.requires_capability_opt_in && slot.capability_requirement == Some(requirement)
    })
}
