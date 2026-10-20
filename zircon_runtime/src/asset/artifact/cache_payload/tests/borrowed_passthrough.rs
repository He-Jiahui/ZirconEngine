use super::super::{from_imported_for_write, ArtifactCacheAsset};
use crate::asset::{
    AssetReference, AssetUri, ImportedAsset, MaterialGraphAsset, TerrainAsset, TerrainLayerAsset,
    TerrainLayerStackAsset, TileMapAsset, TileMapLayerAsset, TileSetAsset, TileSetTileAsset,
    UiIconAsset, UiIconSource, UiIconSourceKind, UiThemeAsset,
};
use crate::core::framework::animation::{
    AnimationClipAsset, AnimationEventTrackAsset, AnimationGraphAsset, AnimationSequenceAsset,
    AnimationSkeletonAsset, AnimationSkeletonBoneAsset, AnimationStateMachineAsset,
};
use crate::core::framework::navigation::{NavMeshAsset, NavigationSettingsAsset};
use zircon_runtime_interface::serialization::PositionalEncoding;

fn uri(path: &str) -> AssetUri {
    AssetUri::parse(path).unwrap()
}

fn reference(path: &str) -> AssetReference {
    AssetReference::from_locator(uri(path))
}

fn terrain() -> TerrainAsset {
    TerrainAsset {
        uri: uri("res://memory/terrain.toml"),
        name: "borrowed terrain".into(),
        width: 2,
        height: 2,
        sample_spacing: 0.5,
        height_scale: 3.0,
        height_samples: vec![0.0, 0.25, 0.5, 1.0],
        layers: vec![TerrainLayerAsset {
            name: "ground".into(),
            material: Some(reference("res://memory/ground.material")),
            weightmap: None,
            strength: 0.75,
        }],
    }
}

#[test]
fn borrowed_passthrough_dtos_keep_wire_bytes_variant_tags_and_roundtrip() {
    let assets = vec![
        (
            4_u32,
            ImportedAsset::MaterialGraph(MaterialGraphAsset {
                uri: uri("res://memory/graph.toml"),
                name: "borrowed graph".into(),
                shader: Some(reference("res://memory/graph.shader")),
                nodes: Vec::new(),
                links: Vec::new(),
                parameters: Default::default(),
            }),
        ),
        (
            8,
            ImportedAsset::NavMesh(NavMeshAsset::simple_quad("humanoid", 2.0)),
        ),
        (
            9,
            ImportedAsset::NavigationSettings(NavigationSettingsAsset::default()),
        ),
        (10, ImportedAsset::Terrain(terrain())),
        (
            11,
            ImportedAsset::TerrainLayerStack(TerrainLayerStackAsset {
                uri: uri("res://memory/layers.toml"),
                layers: terrain().layers,
            }),
        ),
        (
            12,
            ImportedAsset::TileSet(TileSetAsset {
                uri: uri("res://memory/tiles.toml"),
                tile_width: 16,
                tile_height: 16,
                image: reference("res://memory/tiles.png"),
                tiles: vec![TileSetTileAsset {
                    id: 5,
                    name: Some("floor".into()),
                    collider: Some("box".into()),
                }],
            }),
        ),
        (
            13,
            ImportedAsset::TileMap(TileMapAsset {
                uri: uri("res://memory/map.toml"),
                width: 2,
                height: 2,
                projection: Default::default(),
                tile_set: reference("res://memory/tiles.toml"),
                layers: vec![TileMapLayerAsset {
                    name: "floor".into(),
                    visible: true,
                    opacity: 0.5,
                    tiles: vec![Some(5), None, Some(5), Some(5)],
                }],
            }),
        ),
        (
            18,
            ImportedAsset::AnimationSkeleton(AnimationSkeletonAsset {
                name: Some("borrowed skeleton".into()),
                bones: vec![AnimationSkeletonBoneAsset {
                    name: "root".into(),
                    parent_index: None,
                    local_translation: [0.0; 3],
                    local_rotation: [0.0, 0.0, 0.0, 1.0],
                    local_scale: [1.0; 3],
                }],
            }),
        ),
        (
            19,
            ImportedAsset::AnimationClip(AnimationClipAsset {
                name: Some("borrowed clip".into()),
                skeleton: reference("res://memory/skeleton.anim"),
                duration_seconds: 1.0,
                tracks: Vec::new(),
                event_tracks: vec![AnimationEventTrackAsset {
                    target_id: Some("root".into()),
                    event: "step".into(),
                    time_seconds: 0.5,
                    payload: Some("left".into()),
                }],
            }),
        ),
        (
            20,
            ImportedAsset::AnimationSequence(AnimationSequenceAsset {
                name: Some("borrowed sequence".into()),
                duration_seconds: 2.0,
                frames_per_second: 30.0,
                bindings: Vec::new(),
            }),
        ),
        (
            21,
            ImportedAsset::AnimationGraph(AnimationGraphAsset {
                name: Some("borrowed animation graph".into()),
                parameters: Vec::new(),
                nodes: Vec::new(),
            }),
        ),
        (
            22,
            ImportedAsset::AnimationStateMachine(AnimationStateMachineAsset {
                name: Some("borrowed state machine".into()),
                entry_state: "idle".into(),
                states: Vec::new(),
                transitions: Vec::new(),
                layers: Vec::new(),
            }),
        ),
        (
            26,
            ImportedAsset::UiTheme(UiThemeAsset {
                document: Default::default(),
            }),
        ),
        (
            27,
            ImportedAsset::UiIcon(UiIconAsset {
                source: UiIconSource {
                    kind: UiIconSourceKind::Svg,
                    text: Some("<svg viewBox='0 0 16 16'/>".into()),
                    uri: None,
                },
                default_size: 16.0,
                semantic_id: "memory-icon".into(),
            }),
        ),
    ];
    let encoding = PositionalEncoding::FixedLittleEndian;
    for (tag, asset) in assets {
        // The owned path exercises the pre-optimization payload representation.
        let legacy = encoding
            .encode(&ArtifactCacheAsset::from_imported(&asset).unwrap())
            .unwrap();
        let borrowed = encoding
            .encode(&from_imported_for_write(&asset).unwrap())
            .unwrap();
        assert_eq!(borrowed, legacy, "wire tag {tag}");
        assert_eq!(&borrowed[..4], &tag.to_le_bytes());
        // Cache reads must own their data independently of the input buffer.
        let decoded: ArtifactCacheAsset<'static> = encoding.decode(&borrowed).unwrap();
        assert_eq!(decoded.into_imported().unwrap(), asset, "wire tag {tag}");
    }
}

#[cfg(feature = "memory-validation")]
#[test]
fn memory_profile_borrowed_passthrough_dtos_do_not_clone_large_geometry() {
    use crate::memory_validation::measure;

    let mut terrain = terrain();
    terrain.width = 512;
    terrain.height = 512;
    terrain.height_samples = vec![0.5; 512 * 512];
    let mut navmesh = NavMeshAsset::simple_quad("humanoid", 2.0);
    navmesh.vertices = vec![[0.0; 3]; 128 * 1024];
    for asset in [
        ImportedAsset::Terrain(terrain),
        ImportedAsset::NavMesh(navmesh),
    ] {
        let (owned, before) = measure(|| ArtifactCacheAsset::from_imported(&asset).unwrap());
        let (borrowed, after) = measure(|| from_imported_for_write(&asset).unwrap());
        assert!(before.peak_bytes >= 512 * 512 * std::mem::size_of::<crate::core::math::Real>());
        assert_eq!(after.allocations, 0);
        assert_eq!(after.live_bytes, 0);
        let encoding = PositionalEncoding::FixedLittleEndian;
        assert_eq!(
            encoding.encode(&borrowed).unwrap(),
            encoding.encode(&owned).unwrap()
        );
        after.report("terrain_navmesh_write_dto", "borrowed_passthrough");
    }
}
