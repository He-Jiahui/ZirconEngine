use zircon_runtime_interface::resource::ResourceKind;

use super::{
    AssetTypeContribution, AssetTypeId, AssetTypePresentation, AssetTypeRegistry,
    AssetTypeRegistryError, ThumbnailPlaceholderPalette, ThumbnailProviderDescriptor,
};
use std::sync::OnceLock;

const BUILTIN_OWNER: &str = "zircon.editor.builtin_asset_types";

const BUILTIN_RESOURCE_KINDS: [ResourceKind; 26] = [
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
];

pub fn builtin_asset_type_definition(
    kind: ResourceKind,
) -> Option<&'static super::AssetTypeDefinition> {
    static REGISTRY: OnceLock<Option<AssetTypeRegistry>> = OnceLock::new();
    REGISTRY
        .get_or_init(|| builtin_registry().ok())
        .as_ref()?
        .get(&AssetTypeId::from_resource_kind(kind))
}

pub(super) fn builtin_registry() -> Result<AssetTypeRegistry, AssetTypeRegistryError> {
    let mut registry = AssetTypeRegistry::default();
    for kind in BUILTIN_RESOURCE_KINDS {
        let id = AssetTypeId::from_resource_kind(kind);
        let metadata = builtin_metadata(kind);
        registry.apply_contribution(
            BUILTIN_OWNER,
            AssetTypeContribution::define(
                id.clone(),
                AssetTypePresentation::new(
                    metadata.display_name,
                    metadata.badge,
                    metadata.icon_name,
                    metadata.color_token,
                ),
                builtin_thumbnail_provider(kind, &id, metadata.icon_name),
            )
            .with_runtime_kind(kind),
        )?;
    }
    Ok(registry)
}

fn builtin_thumbnail_provider(
    kind: ResourceKind,
    id: &AssetTypeId,
    icon_name: &str,
) -> ThumbnailProviderDescriptor {
    if kind == ResourceKind::Texture {
        ThumbnailProviderDescriptor::SourceImage
    } else {
        ThumbnailProviderDescriptor::Placeholder {
            icon_name: icon_name.to_owned(),
            palette: placeholder_palette(id.as_str()),
        }
    }
}

fn placeholder_palette(id: &str) -> ThumbnailPlaceholderPalette {
    let mut hash = 0x811c9dc5_u32;
    for byte in id.bytes() {
        hash ^= u32::from(byte);
        hash = hash.wrapping_mul(0x01000193);
    }
    let channel =
        |shift: u32, base: u8, span: u8| base.saturating_add(((hash >> shift) as u8) % span);
    let primary = [
        channel(0, 72, 88),
        channel(8, 78, 82),
        channel(16, 92, 80),
        255,
    ];
    let secondary = [primary[0] / 2, primary[1] / 2, primary[2] / 2, 255];
    let accent = [
        primary[0].saturating_add(72),
        primary[1].saturating_add(72),
        primary[2].saturating_add(72),
        255,
    ];
    let banner = [
        accent[0].saturating_add(22),
        accent[1].saturating_add(22),
        accent[2].saturating_add(22),
        255,
    ];
    ThumbnailPlaceholderPalette {
        primary,
        secondary,
        accent,
        banner,
    }
}

struct BuiltinMetadata {
    display_name: &'static str,
    badge: &'static str,
    icon_name: &'static str,
    color_token: &'static str,
}

fn builtin_metadata(kind: ResourceKind) -> BuiltinMetadata {
    let (display_name, badge, icon_name) = match kind {
        ResourceKind::Data => ("Data", "DAT", "asset-data"),
        ResourceKind::Model => ("Model", "MDL", "asset-model"),
        ResourceKind::Mesh => ("Mesh", "MSH", "asset-mesh"),
        ResourceKind::Material => ("Material", "MAT", "asset-material"),
        ResourceKind::MaterialGraph => ("Material Graph", "MGR", "asset-material-graph"),
        ResourceKind::Texture => ("Texture", "TEX", "asset-texture"),
        ResourceKind::Shader => ("Shader", "SHD", "asset-shader"),
        ResourceKind::Scene => ("Scene", "SCN", "asset-scene"),
        ResourceKind::Sound => ("Sound", "SND", "asset-sound"),
        ResourceKind::Font => ("Font", "FNT", "asset-font"),
        ResourceKind::PhysicsMaterial => ("Physics Material", "PHY", "asset-physics-material"),
        ResourceKind::NavMesh => ("Navigation Mesh", "NAV", "asset-navigation-mesh"),
        ResourceKind::NavigationSettings => {
            ("Navigation Settings", "NVS", "asset-navigation-settings")
        }
        ResourceKind::Terrain => ("Terrain", "TER", "asset-terrain"),
        ResourceKind::TerrainLayerStack => ("Terrain Layer Stack", "TLS", "asset-terrain-layers"),
        ResourceKind::TileSet => ("Tile Set", "TLS", "asset-tile-set"),
        ResourceKind::TileMap => ("Tile Map", "TLM", "asset-tile-map"),
        ResourceKind::Prefab => ("Prefab", "PFB", "asset-prefab"),
        ResourceKind::AnimationSkeleton => {
            ("Animation Skeleton", "SKL", "asset-animation-skeleton")
        }
        ResourceKind::AnimationClip => ("Animation Clip", "CLP", "asset-animation-clip"),
        ResourceKind::AnimationSequence => {
            ("Animation Sequence", "SEQ", "asset-animation-sequence")
        }
        ResourceKind::AnimationGraph => ("Animation Graph", "AGR", "asset-animation-graph"),
        ResourceKind::AnimationStateMachine => (
            "Animation State Machine",
            "ASM",
            "asset-animation-state-machine",
        ),
        ResourceKind::UiLayout => ("UI Layout", "UIL", "asset-ui-layout"),
        ResourceKind::UiWidget => ("UI Widget", "UIW", "asset-ui-widget"),
        ResourceKind::UiStyle => ("UI Style", "UIS", "asset-ui-style"),
    };
    BuiltinMetadata {
        display_name,
        badge,
        icon_name,
        color_token: "asset.default",
    }
}
