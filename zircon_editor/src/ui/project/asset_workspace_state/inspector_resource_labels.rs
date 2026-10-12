use super::AssetWorkspaceState;
use crate::ui::workbench::snapshot::InspectorNativeFieldSnapshot;
use zircon_runtime::core::resource::ResourceId;

impl AssetWorkspaceState {
    /// Uses the published catalog/resource generations; inspection never loads a resource.
    pub(crate) fn resolve_inspector_resource_labels(
        &self,
        fields: &mut [InspectorNativeFieldSnapshot],
    ) {
        for field in fields {
            let Some(id) = field.resource_id.as_deref() else {
                continue;
            };
            field.value = self
                .catalog
                .as_ref()
                .and_then(|catalog| catalog.assets.iter().find(|asset| asset.id == id))
                .map(|asset| asset.locator.clone())
                .or_else(|| {
                    id.parse::<ResourceId>()
                        .ok()
                        .and_then(|id| self.resources.row_by_id(id))
                        .map(|row| row.primary_locator.to_string())
                })
                .unwrap_or_else(|| "Unavailable".to_string());
        }
    }
}
