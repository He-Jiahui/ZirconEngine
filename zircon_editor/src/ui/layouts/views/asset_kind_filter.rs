use zircon_runtime_interface::resource::ResourceKind;

use crate::ui::retained_host::primitives::SharedString;

pub(crate) const ASSETS_ACTIVITY_KIND_FILTER_CONTROL_ID: &str = "AssetsActivityKindFilterDropdown";
pub(crate) const ASSET_BROWSER_KIND_FILTER_CONTROL_ID: &str = "AssetBrowserKindFilterDropdown";
pub(crate) const ASSET_KIND_FILTER_OPTIONS: [(&str, &str); 16] = [
    ("All", "All Types"),
    ("Texture", "Textures"),
    ("Material", "Materials"),
    ("Scene", "Scenes"),
    ("Model", "Models"),
    ("Mesh", "Meshes"),
    ("Shader", "Shaders"),
    ("PhysicsMaterial", "Physics Materials"),
    ("AnimationSkeleton", "Animation Skeletons"),
    ("AnimationClip", "Animation Clips"),
    ("AnimationSequence", "Animation Sequences"),
    ("AnimationGraph", "Animation Graphs"),
    ("AnimationStateMachine", "Animation State Machines"),
    ("UiLayout", "UI Layouts"),
    ("UiWidget", "UI Widgets"),
    ("UiStyle", "UI Styles"),
];

pub(crate) fn asset_kind_filter_options(
    kind_filter: Option<ResourceKind>,
) -> (&'static str, Vec<SharedString>) {
    let (selected_id, selected_label) = asset_kind_filter_identity(kind_filter);
    let selected_is_supported = asset_kind_filter_is_supported(selected_id);
    let mut options =
        Vec::with_capacity(ASSET_KIND_FILTER_OPTIONS.len() + usize::from(!selected_is_supported));
    options.extend(
        ASSET_KIND_FILTER_OPTIONS
            .iter()
            .map(|(id, label)| asset_kind_filter_option(id, label, *id == selected_id, false)),
    );
    if !selected_is_supported {
        options.push(asset_kind_filter_option(
            selected_id,
            selected_label,
            true,
            true,
        ));
    }
    (selected_label, options)
}

pub(crate) fn asset_kind_filter_identity(
    kind_filter: Option<ResourceKind>,
) -> (&'static str, &'static str) {
    match kind_filter {
        None => ("All", "All Types"),
        Some(ResourceKind::Data) => ("Data", "Data"),
        Some(ResourceKind::Model) => ("Model", "Models"),
        Some(ResourceKind::Mesh) => ("Mesh", "Meshes"),
        Some(ResourceKind::Material) => ("Material", "Materials"),
        Some(ResourceKind::MaterialGraph) => ("MaterialGraph", "Material Graphs"),
        Some(ResourceKind::Texture) => ("Texture", "Textures"),
        Some(ResourceKind::Shader) => ("Shader", "Shaders"),
        Some(ResourceKind::Scene) => ("Scene", "Scenes"),
        Some(ResourceKind::Sound) => ("Sound", "Sounds"),
        Some(ResourceKind::Font) => ("Font", "Fonts"),
        Some(ResourceKind::PhysicsMaterial) => ("PhysicsMaterial", "Physics Materials"),
        Some(ResourceKind::NavMesh) => ("NavMesh", "Navigation Meshes"),
        Some(ResourceKind::NavigationSettings) => ("NavigationSettings", "Navigation Settings"),
        Some(ResourceKind::Terrain) => ("Terrain", "Terrains"),
        Some(ResourceKind::TerrainLayerStack) => ("TerrainLayerStack", "Terrain Layer Stacks"),
        Some(ResourceKind::TileSet) => ("TileSet", "Tile Sets"),
        Some(ResourceKind::TileMap) => ("TileMap", "Tile Maps"),
        Some(ResourceKind::Prefab) => ("Prefab", "Prefabs"),
        Some(ResourceKind::AnimationSkeleton) => ("AnimationSkeleton", "Animation Skeletons"),
        Some(ResourceKind::AnimationClip) => ("AnimationClip", "Animation Clips"),
        Some(ResourceKind::AnimationSequence) => ("AnimationSequence", "Animation Sequences"),
        Some(ResourceKind::AnimationGraph) => ("AnimationGraph", "Animation Graphs"),
        Some(ResourceKind::AnimationStateMachine) => {
            ("AnimationStateMachine", "Animation State Machines")
        }
        Some(ResourceKind::UiLayout) => ("UiLayout", "UI Layouts"),
        Some(ResourceKind::UiWidget) => ("UiWidget", "UI Widgets"),
        Some(ResourceKind::UiStyle) => ("UiStyle", "UI Styles"),
    }
}

pub(crate) fn asset_kind_filter_is_supported(id: &str) -> bool {
    ASSET_KIND_FILTER_OPTIONS
        .iter()
        .any(|(candidate, _)| *candidate == id)
}

fn asset_kind_filter_option(id: &str, label: &str, selected: bool, disabled: bool) -> SharedString {
    let mut option = format!("{id}|label={label}");
    if selected {
        option.push_str(",selected");
    }
    if disabled {
        option.push_str(",disabled");
    }
    option.into()
}
