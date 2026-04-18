use zircon_manager::ResourceManager;
use zircon_resource::{
    ResourceDiagnostic, ResourceHandle, ResourceKind, ResourceLocator, ResourceMarker,
    ResourceRecord, ResourceState,
};

pub(crate) fn resolve_ready_handle<TMarker>(
    resource_server: &(impl ResourceManager + ?Sized),
    locator: &ResourceLocator,
) -> Result<ResourceHandle<TMarker>, String>
where
    TMarker: ResourceMarker,
{
    let status: ResourceRecord = resource_server
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
        ResourceKind::Model => "Model",
        ResourceKind::Material => "Material",
        ResourceKind::Texture => "Texture",
        ResourceKind::Shader => "Shader",
        ResourceKind::Scene => "Scene",
        ResourceKind::UiLayout => "UiLayout",
        ResourceKind::UiWidget => "UiWidget",
        ResourceKind::UiStyle => "UiStyle",
    }
}

fn render_diagnostics(diagnostics: &[ResourceDiagnostic]) -> String {
    diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect::<Vec<_>>()
        .join("; ")
}
