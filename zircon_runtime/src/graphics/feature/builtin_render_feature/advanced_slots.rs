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
        .binary_search_by_key(&feature, |slot| slot.feature)
        .ok()
        .map(|index| &DESCRIPTOR_ONLY_ADVANCED_SLOTS[index])
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

#[cfg(test)]
mod optimization_tests {
    use std::hint::black_box;

    use super::{
        BuiltinRenderFeature, DESCRIPTOR_ONLY_ADVANCED_SLOTS, descriptor_only_advanced_slot,
    };

    const LOOKUP_ROUNDS: usize = 65_536;

    #[test]
    fn optimization_batch_20260830dl_descriptor_slots_are_sorted_for_binary_lookup() {
        assert!(
            DESCRIPTOR_ONLY_ADVANCED_SLOTS
                .windows(2)
                .all(|slots| slots[0].feature < slots[1].feature)
        );

        for feature in BuiltinRenderFeature::ALL {
            let legacy = DESCRIPTOR_ONLY_ADVANCED_SLOTS
                .iter()
                .find(|slot| slot.feature == *feature)
                .map(|slot| slot.descriptor_name());
            let optimized =
                descriptor_only_advanced_slot(*feature).map(|slot| slot.descriptor_name());
            assert_eq!(optimized, legacy, "lookup changed for {feature:?}");
        }
    }

    #[test]
    #[ignore = "deterministic comparator-count evidence for the managed optimization batch"]
    fn optimization_batch_20260830dl_descriptor_slot_lookup_evidence() {
        let mut linear_comparisons = 0_u64;
        let mut binary_comparisons = 0_u64;

        for round in 0..LOOKUP_ROUNDS {
            let feature =
                black_box(BuiltinRenderFeature::ALL[round % BuiltinRenderFeature::ALL.len()]);
            let legacy = DESCRIPTOR_ONLY_ADVANCED_SLOTS.iter().find(|slot| {
                linear_comparisons += 1;
                slot.feature == feature
            });
            let optimized = DESCRIPTOR_ONLY_ADVANCED_SLOTS
                .binary_search_by(|slot| {
                    binary_comparisons += 1;
                    slot.feature.cmp(&feature)
                })
                .ok()
                .map(|index| &DESCRIPTOR_ONLY_ADVANCED_SLOTS[index]);
            assert_eq!(
                optimized.map(|slot| slot.descriptor_name()),
                legacy.map(|slot| slot.descriptor_name())
            );
        }

        let reduction_basis_points = linear_comparisons
            .saturating_sub(binary_comparisons)
            .saturating_mul(10_000)
            / linear_comparisons;
        println!(
            "RUNTIME523_ADVANCED_SLOT_BINARY_LOOKUP_BENCH_V1 rounds={LOOKUP_ROUNDS} linear_comparisons={linear_comparisons} binary_comparisons={binary_comparisons} comparison_reduction_basis_points={reduction_basis_points}"
        );
        assert!(binary_comparisons < linear_comparisons);
    }
}
