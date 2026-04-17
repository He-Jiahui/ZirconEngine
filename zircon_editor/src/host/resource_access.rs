use zircon_manager::{ResourceManager, ResourceStateRecord};
use zircon_resource::{ResourceHandle, ResourceId, ResourceKind, ResourceLocator, ResourceMarker};

pub(crate) fn resolve_ready_handle<TMarker>(
    resource_server: &(impl ResourceManager + ?Sized),
    locator: &ResourceLocator,
) -> Result<ResourceHandle<TMarker>, String>
where
    TMarker: ResourceMarker,
{
    let status = resource_server
        .resource_status(&locator.to_string())
        .ok_or_else(|| format!("resource {locator} is missing from the resource registry"))?;

    if !record_kind_matches::<TMarker>(status.kind) {
        return Err(format!(
            "resource {locator} has kind {:?}, expected {}",
            status.kind,
            resource_kind_name(TMarker::KIND)
        ));
    }

    if status.state != ResourceStateRecord::Ready {
        let diagnostics = if status.diagnostics.is_empty() {
            String::new()
        } else {
            format!(": {}", status.diagnostics.join("; "))
        };
        return Err(format!(
            "resource {locator} is not ready ({:?}){diagnostics}",
            status.state
        ));
    }

    let id = status.id.parse::<ResourceId>().map_err(|error| {
        format!(
            "resource {locator} returned invalid id {}: {error}",
            status.id
        )
    })?;
    Ok(ResourceHandle::new(id))
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
