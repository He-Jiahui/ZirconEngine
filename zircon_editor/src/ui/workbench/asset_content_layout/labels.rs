use zircon_runtime_interface::resource::ResourceKind;

pub(crate) fn resource_kind_badge_code(kind: ResourceKind) -> &'static str {
    match kind {
        ResourceKind::Data => "DAT",
        ResourceKind::Model => "MDL",
        ResourceKind::Mesh => "MSH",
        ResourceKind::Material | ResourceKind::MaterialGraph | ResourceKind::PhysicsMaterial => {
            "MAT"
        }
        ResourceKind::Texture => "TEX",
        ResourceKind::Shader => "SHD",
        ResourceKind::Scene => "SCN",
        ResourceKind::Sound => "AUD",
        ResourceKind::Font => "FNT",
        ResourceKind::NavMesh | ResourceKind::NavigationSettings => "NAV",
        ResourceKind::Terrain | ResourceKind::TerrainLayerStack => "TRN",
        ResourceKind::TileSet | ResourceKind::TileMap => "TIL",
        ResourceKind::Prefab => "PFB",
        ResourceKind::AnimationSkeleton
        | ResourceKind::AnimationClip
        | ResourceKind::AnimationSequence
        | ResourceKind::AnimationGraph
        | ResourceKind::AnimationStateMachine => "ANM",
        ResourceKind::UiLayout => "UI",
        ResourceKind::UiWidget => "WDG",
        ResourceKind::UiStyle => "STY",
    }
}
