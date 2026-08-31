use zircon_runtime::core::framework::asset::ResourceManager;
use zircon_runtime_interface::resource::{
    ResourceDiagnostic, ResourceHandle, ResourceKind, ResourceLocator, ResourceMarker,
    ResourceRecord, ResourceState,
};

pub(crate) fn resolve_ready_handle<TMarker>(
    resource_manager: &(impl ResourceManager + ?Sized),
    locator: &ResourceLocator,
) -> Result<ResourceHandle<TMarker>, String>
where
    TMarker: ResourceMarker,
{
    let status: ResourceRecord = resource_manager
        .resource_status(&locator.to_string())
        .ok_or_else(|| format!("resource {locator} is missing from the resource registry"))?;

    if !record_kind_matches::<TMarker>(status.kind) {
        return Err(format!(
            "resource {locator} has kind {:?}, expected {}",
            status.kind,
            resource_kind_name(TMarker::KIND)
        ));
    }

    if status.state != ResourceState::Ready {
        let diagnostics = render_diagnostics(&status.diagnostics);
        let diagnostics = if diagnostics.is_empty() {
            String::new()
        } else {
            format!(": {diagnostics}")
        };
        return Err(format!(
            "resource {locator} is not ready ({:?}){diagnostics}",
            status.state
        ));
    }

    Ok(ResourceHandle::new(status.id))
}

fn record_kind_matches<TMarker: ResourceMarker>(kind: ResourceKind) -> bool {
    kind == TMarker::KIND
}

fn resource_kind_name(kind: ResourceKind) -> &'static str {
    match kind {
        ResourceKind::Data => "Data",
        ResourceKind::Model => "Model",
        ResourceKind::Mesh => "Mesh",
        ResourceKind::Material => "Material",
        ResourceKind::MaterialGraph => "MaterialGraph",
        ResourceKind::PhysicsMaterial => "PhysicsMaterial",
        ResourceKind::NavMesh => "NavMesh",
        ResourceKind::NavigationSettings => "NavigationSettings",
        ResourceKind::Terrain => "Terrain",
        ResourceKind::TerrainLayerStack => "TerrainLayerStack",
        ResourceKind::TileSet => "TileSet",
        ResourceKind::TileMap => "TileMap",
        ResourceKind::Prefab => "Prefab",
        ResourceKind::Texture => "Texture",
        ResourceKind::Shader => "Shader",
        ResourceKind::Scene => "Scene",
        ResourceKind::Sound => "Sound",
        ResourceKind::Font => "Font",
        ResourceKind::AnimationSkeleton => "AnimationSkeleton",
        ResourceKind::AnimationClip => "AnimationClip",
        ResourceKind::AnimationSequence => "AnimationSequence",
        ResourceKind::AnimationGraph => "AnimationGraph",
        ResourceKind::AnimationStateMachine => "AnimationStateMachine",
        ResourceKind::UiLayout => "UiLayout",
        ResourceKind::UiWidget => "UiWidget",
        ResourceKind::UiStyle => "UiStyle",
    }
}

fn render_diagnostics(diagnostics: &[ResourceDiagnostic]) -> String {
    const SEPARATOR: &str = "; ";

    let capacity = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.len())
        .sum::<usize>()
        + diagnostics
            .len()
            .saturating_sub(1)
            .saturating_mul(SEPARATOR.len());
    let mut rendered = String::with_capacity(capacity);
    for (index, diagnostic) in diagnostics.iter().enumerate() {
        if index != 0 {
            rendered.push_str(SEPARATOR);
        }
        rendered.push_str(&diagnostic.message);
    }
    rendered
}

#[cfg(test)]
#[path = "resource_access/direct_join_tests.rs"]
mod direct_join_tests;

#[cfg(test)]
mod tests {
    use super::resource_kind_name;
    use zircon_runtime_interface::resource::ResourceKind;

    #[test]
    fn resource_kind_name_includes_font() {
        assert_eq!(resource_kind_name(ResourceKind::Font), "Font");
    }
}
