#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BuiltinRenderFeature {
    Mesh,
    SkinnedMesh,
    MeshLod,
    Sprite,
    DeferredGeometry,
    DeferredLighting,
    ClusteredLighting,
    Hzb,
    ScreenSpaceAmbientOcclusion,
    Bloom,
    ColorGrading,
    ReflectionProbes,
    BakedLighting,
    Temporal,
    AntiAlias,
    Shadows,
    PostProcess,
    Ui,
    DebugOverlay,
    Particle,
    GlobalIllumination,
    RayTracing,
    NeuralCompute,
    SparseTexture,
    Terrain,
    Tree,
    Decal,
    Projector,
    Halo,
    LensFlare,
    Trail,
    Billboard,
    Tilemap,
    TextShaping,
    Skybox,
    Cubemap,
    Texture2dArray,
    NormalMap,
    Mipmap,
    ColorSpace,
    VirtualGeometry,
}

impl BuiltinRenderFeature {
    pub const ALL: &[Self] = &[
        Self::Mesh,
        Self::SkinnedMesh,
        Self::MeshLod,
        Self::Sprite,
        Self::DeferredGeometry,
        Self::DeferredLighting,
        Self::ClusteredLighting,
        Self::Hzb,
        Self::ScreenSpaceAmbientOcclusion,
        Self::Bloom,
        Self::ColorGrading,
        Self::ReflectionProbes,
        Self::BakedLighting,
        Self::Temporal,
        Self::AntiAlias,
        Self::Shadows,
        Self::PostProcess,
        Self::Ui,
        Self::DebugOverlay,
        Self::Particle,
        Self::GlobalIllumination,
        Self::RayTracing,
        Self::NeuralCompute,
        Self::SparseTexture,
        Self::Terrain,
        Self::Tree,
        Self::Decal,
        Self::Projector,
        Self::Halo,
        Self::LensFlare,
        Self::Trail,
        Self::Billboard,
        Self::Tilemap,
        Self::TextShaping,
        Self::Skybox,
        Self::Cubemap,
        Self::Texture2dArray,
        Self::NormalMap,
        Self::Mipmap,
        Self::ColorSpace,
        Self::VirtualGeometry,
    ];

    pub const fn authoring_name(self) -> &'static str {
        match self {
            Self::Mesh => "Mesh",
            Self::SkinnedMesh => "SkinnedMesh",
            Self::MeshLod => "MeshLod",
            Self::Sprite => "Sprite",
            Self::DeferredGeometry => "DeferredGeometry",
            Self::DeferredLighting => "DeferredLighting",
            Self::ClusteredLighting => "ClusteredLighting",
            Self::Hzb => "Hzb",
            Self::ScreenSpaceAmbientOcclusion => "ScreenSpaceAmbientOcclusion",
            Self::Bloom => "Bloom",
            Self::ColorGrading => "ColorGrading",
            Self::ReflectionProbes => "ReflectionProbes",
            Self::BakedLighting => "BakedLighting",
            Self::Temporal => "Temporal",
            Self::AntiAlias => "AntiAlias",
            Self::Shadows => "Shadows",
            Self::PostProcess => "PostProcess",
            Self::Ui => "Ui",
            Self::DebugOverlay => "DebugOverlay",
            Self::Particle => "Particle",
            Self::GlobalIllumination => "GlobalIllumination",
            Self::RayTracing => "RayTracing",
            Self::NeuralCompute => "NeuralCompute",
            Self::SparseTexture => "SparseTexture",
            Self::Terrain => "Terrain",
            Self::Tree => "Tree",
            Self::Decal => "Decal",
            Self::Projector => "Projector",
            Self::Halo => "Halo",
            Self::LensFlare => "LensFlare",
            Self::Trail => "Trail",
            Self::Billboard => "Billboard",
            Self::Tilemap => "Tilemap",
            Self::TextShaping => "TextShaping",
            Self::Skybox => "Skybox",
            Self::Cubemap => "Cubemap",
            Self::Texture2dArray => "Texture2dArray",
            Self::NormalMap => "NormalMap",
            Self::Mipmap => "Mipmap",
            Self::ColorSpace => "ColorSpace",
            Self::VirtualGeometry => "VirtualGeometry",
        }
    }

    pub fn from_authoring_name(value: &str) -> Option<Self> {
        Some(match value {
            "Mesh" => Self::Mesh,
            "SkinnedMesh" => Self::SkinnedMesh,
            "MeshLod" => Self::MeshLod,
            "Sprite" => Self::Sprite,
            "DeferredGeometry" => Self::DeferredGeometry,
            "DeferredLighting" => Self::DeferredLighting,
            "ClusteredLighting" => Self::ClusteredLighting,
            "Hzb" => Self::Hzb,
            "ScreenSpaceAmbientOcclusion" => Self::ScreenSpaceAmbientOcclusion,
            "Bloom" => Self::Bloom,
            "ColorGrading" => Self::ColorGrading,
            "ReflectionProbes" => Self::ReflectionProbes,
            "BakedLighting" => Self::BakedLighting,
            "Temporal" => Self::Temporal,
            "AntiAlias" => Self::AntiAlias,
            "Shadows" => Self::Shadows,
            "PostProcess" => Self::PostProcess,
            "Ui" => Self::Ui,
            "DebugOverlay" => Self::DebugOverlay,
            "Particle" => Self::Particle,
            "GlobalIllumination" => Self::GlobalIllumination,
            "RayTracing" => Self::RayTracing,
            "NeuralCompute" => Self::NeuralCompute,
            "SparseTexture" => Self::SparseTexture,
            "Terrain" => Self::Terrain,
            "Tree" => Self::Tree,
            "Decal" => Self::Decal,
            "Projector" => Self::Projector,
            "Halo" => Self::Halo,
            "LensFlare" => Self::LensFlare,
            "Trail" => Self::Trail,
            "Billboard" => Self::Billboard,
            "Tilemap" => Self::Tilemap,
            "TextShaping" => Self::TextShaping,
            "Skybox" => Self::Skybox,
            "Cubemap" => Self::Cubemap,
            "Texture2dArray" => Self::Texture2dArray,
            "NormalMap" => Self::NormalMap,
            "Mipmap" => Self::Mipmap,
            "ColorSpace" => Self::ColorSpace,
            "VirtualGeometry" => Self::VirtualGeometry,
            _ => return None,
        })
    }
}

#[cfg(test)]
mod optimization_tests {
    use std::hint::black_box;

    use super::BuiltinRenderFeature;

    const LOOKUP_ROUNDS: usize = 65_536;

    #[test]
    fn optimization_batch_20260830dm_authoring_name_lookup_round_trips_every_feature() {
        for feature in BuiltinRenderFeature::ALL {
            assert_eq!(
                BuiltinRenderFeature::from_authoring_name(feature.authoring_name()),
                Some(*feature)
            );
        }
        assert_eq!(BuiltinRenderFeature::from_authoring_name(""), None);
        assert_eq!(
            BuiltinRenderFeature::from_authoring_name("UnknownFeature"),
            None
        );
    }

    #[test]
    #[ignore = "deterministic candidate-check model for the managed optimization batch"]
    fn optimization_batch_20260830dm_authoring_name_lookup_evidence() {
        let mut legacy_candidate_checks = 0_u64;
        let mut direct_lookup_calls = 0_u64;

        for round in 0..LOOKUP_ROUNDS {
            let feature = BuiltinRenderFeature::ALL[round % BuiltinRenderFeature::ALL.len()];
            let name = black_box(feature.authoring_name());
            let legacy = BuiltinRenderFeature::ALL.iter().copied().find(|candidate| {
                legacy_candidate_checks += 1;
                candidate.authoring_name() == name
            });
            direct_lookup_calls += 1;
            let optimized = BuiltinRenderFeature::from_authoring_name(name);
            assert_eq!(optimized, legacy);
        }

        let reduction_basis_points = legacy_candidate_checks
            .saturating_sub(direct_lookup_calls)
            .saturating_mul(10_000)
            / legacy_candidate_checks;
        println!(
            "RUNTIME524_BUILTIN_FEATURE_NAME_LOOKUP_BENCH_V1 rounds={LOOKUP_ROUNDS} legacy_candidate_checks={legacy_candidate_checks} direct_lookup_calls={direct_lookup_calls} candidate_check_reduction_basis_points_model={reduction_basis_points}"
        );
        assert!(direct_lookup_calls < legacy_candidate_checks);
    }
}
