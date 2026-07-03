use crate::ui::workbench::snapshot::AssetItemSnapshot;
use zircon_runtime_interface::resource::ResourceKind;

pub(super) fn resource_kind_badge_code(kind: ResourceKind) -> &'static str {
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

pub(super) fn compact_resource_kind_label(kind: ResourceKind) -> &'static str {
    match kind {
        ResourceKind::Texture => "Tex",
        ResourceKind::Material | ResourceKind::MaterialGraph => "Mat",
        ResourceKind::Scene => "Scene",
        ResourceKind::Model | ResourceKind::Mesh => "Mesh",
        ResourceKind::Shader => "Shader",
        ResourceKind::Prefab => "Prefab",
        ResourceKind::UiLayout => "UI",
        ResourceKind::UiWidget => "Widget",
        ResourceKind::UiStyle => "Style",
        _ => "Asset",
    }
}

pub(super) fn summary_resource_kind_label(kind: ResourceKind) -> &'static str {
    match kind {
        ResourceKind::Texture => "Texture",
        ResourceKind::Material => "Material",
        ResourceKind::MaterialGraph => "Material Graph",
        ResourceKind::PhysicsMaterial => "Physics Mat",
        ResourceKind::Scene => "Scene",
        ResourceKind::Model => "Model",
        ResourceKind::Mesh => "Mesh",
        ResourceKind::Shader => "Shader",
        ResourceKind::Sound => "Sound",
        ResourceKind::Font => "Font",
        ResourceKind::NavMesh => "Nav Mesh",
        ResourceKind::NavigationSettings => "Nav Settings",
        ResourceKind::Terrain => "Terrain",
        ResourceKind::TerrainLayerStack => "Terrain Layer",
        ResourceKind::TileSet => "Tile Set",
        ResourceKind::TileMap => "Tile Map",
        ResourceKind::Prefab => "Prefab",
        ResourceKind::AnimationSkeleton => "Skeleton",
        ResourceKind::AnimationClip => "Animation Clip",
        ResourceKind::AnimationSequence => "Animation Seq",
        ResourceKind::AnimationGraph => "Animation Graph",
        ResourceKind::AnimationStateMachine => "State Machine",
        ResourceKind::UiLayout => "UI Layout",
        ResourceKind::UiWidget => "UI Widget",
        ResourceKind::UiStyle => "UI Style",
        ResourceKind::Data => "Data",
    }
}

pub(super) fn asset_state_label(asset: &AssetItemSnapshot) -> &'static str {
    if asset.diagnostics.is_empty() {
        "Ready"
    } else {
        "Diagnostics"
    }
}

pub(super) fn resource_kind_label(kind: ResourceKind) -> &'static str {
    match kind {
        ResourceKind::Texture => "Texture",
        ResourceKind::Material => "Material",
        ResourceKind::Scene => "Scene",
        ResourceKind::Model => "Model",
        ResourceKind::Mesh => "Mesh",
        ResourceKind::Shader => "Shader",
        ResourceKind::Sound => "Sound",
        ResourceKind::Font => "Font",
        ResourceKind::PhysicsMaterial => "PhysicsMaterial",
        ResourceKind::NavMesh => "NavMesh",
        ResourceKind::NavigationSettings => "NavigationSettings",
        ResourceKind::Terrain => "Terrain",
        ResourceKind::TerrainLayerStack => "TerrainLayerStack",
        ResourceKind::TileSet => "TileSet",
        ResourceKind::TileMap => "TileMap",
        ResourceKind::Prefab => "Prefab",
        ResourceKind::AnimationSkeleton => "AnimationSkeleton",
        ResourceKind::AnimationClip => "AnimationClip",
        ResourceKind::AnimationSequence => "AnimationSequence",
        ResourceKind::AnimationGraph => "AnimationGraph",
        ResourceKind::AnimationStateMachine => "AnimationStateMachine",
        ResourceKind::UiLayout => "UiLayout",
        ResourceKind::UiWidget => "UiWidget",
        ResourceKind::UiStyle => "UiStyle",
        ResourceKind::Data => "Data",
        ResourceKind::MaterialGraph => "MaterialGraph",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_kind_badge_codes_stay_short_enough_for_thumbnail_type_pills() {
        for kind in [
            ResourceKind::Data,
            ResourceKind::Model,
            ResourceKind::Mesh,
            ResourceKind::Material,
            ResourceKind::MaterialGraph,
            ResourceKind::Texture,
            ResourceKind::Shader,
            ResourceKind::Scene,
            ResourceKind::Sound,
            ResourceKind::Font,
            ResourceKind::PhysicsMaterial,
            ResourceKind::NavMesh,
            ResourceKind::NavigationSettings,
            ResourceKind::Terrain,
            ResourceKind::TerrainLayerStack,
            ResourceKind::TileSet,
            ResourceKind::TileMap,
            ResourceKind::Prefab,
            ResourceKind::AnimationSkeleton,
            ResourceKind::AnimationClip,
            ResourceKind::AnimationSequence,
            ResourceKind::AnimationGraph,
            ResourceKind::AnimationStateMachine,
            ResourceKind::UiLayout,
            ResourceKind::UiWidget,
            ResourceKind::UiStyle,
        ] {
            let code = resource_kind_badge_code(kind);
            assert!(
                (2..=3).contains(&code.chars().count()),
                "{kind:?} badge code `{code}` should fit the dense Asset Browser type pill"
            );
            assert_eq!(code, code.to_ascii_uppercase());
        }
    }

    #[test]
    fn resource_kind_badge_codes_keep_dense_editor_resource_types_distinct() {
        assert_eq!(resource_kind_badge_code(ResourceKind::UiLayout), "UI");
        assert_eq!(resource_kind_badge_code(ResourceKind::UiWidget), "WDG");
        assert_eq!(resource_kind_badge_code(ResourceKind::UiStyle), "STY");
        assert_eq!(resource_kind_badge_code(ResourceKind::Texture), "TEX");
        assert_eq!(resource_kind_badge_code(ResourceKind::Material), "MAT");
        assert_eq!(resource_kind_badge_code(ResourceKind::Shader), "SHD");
        assert_eq!(resource_kind_badge_code(ResourceKind::Scene), "SCN");
        assert_eq!(resource_kind_badge_code(ResourceKind::Prefab), "PFB");
    }

    #[test]
    fn summary_resource_kind_labels_are_readable_without_changing_dense_badges() {
        assert_eq!(resource_kind_badge_code(ResourceKind::UiLayout), "UI");
        assert_eq!(resource_kind_badge_code(ResourceKind::Texture), "TEX");
        assert_eq!(
            summary_resource_kind_label(ResourceKind::UiLayout),
            "UI Layout"
        );
        assert_eq!(
            summary_resource_kind_label(ResourceKind::Texture),
            "Texture"
        );
    }
}
