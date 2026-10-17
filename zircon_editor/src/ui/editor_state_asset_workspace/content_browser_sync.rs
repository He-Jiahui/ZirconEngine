use crate::ui::workbench::state::EditorState;
use std::collections::BTreeSet;
use zircon_runtime_interface::reflect::ReflectedValue;

impl EditorState {
    pub(crate) fn content_browser_assets_to_sync(
        &self,
        current_scene_uri: Option<&str>,
    ) -> Result<Vec<String>, String> {
        let selection = self.viewport_controller.selection();
        let selected = selection.active_items();
        // Authoring references must never be read using a play-world entity ID.
        if !selected.is_empty() && selection.active_domain() != crate::core::play::WorldDomain::Edit
        {
            return Ok(Vec::new());
        }
        let mut identities = BTreeSet::new();
        if !selected.is_empty() {
            self.world
                .with_world(|scene| {
                    for entity in selected {
                        if let Some(fields) = scene.inspection_fields_artifact(*entity) {
                            for field in fields.fields() {
                                collect_resource_identities(&field.value, &mut identities);
                            }
                        }
                    }
                })
                .map_err(|error| error.to_string())?;
        }
        Ok(self.asset_workspace.content_browser_asset_uuids(
            !selected.is_empty(),
            &identities,
            current_scene_uri,
        ))
    }
}

fn collect_resource_identities(value: &ReflectedValue, identities: &mut BTreeSet<String>) {
    match value {
        ReflectedValue::Resource(identity) if !identity.is_empty() => {
            identities.insert(identity.clone());
        }
        ReflectedValue::List(values) => {
            for value in values {
                collect_resource_identities(value, identities);
            }
        }
        ReflectedValue::Map(values) => {
            for value in values.values() {
                collect_resource_identities(value, identities);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scene_asset_find_collects_only_typed_resources_and_deduplicates_references() {
        let mut ids = BTreeSet::new();
        collect_resource_identities(&ReflectedValue::Resource("asset-id".into()), &mut ids);
        collect_resource_identities(&ReflectedValue::Resource("asset-id".into()), &mut ids);
        collect_resource_identities(
            &ReflectedValue::List(vec![ReflectedValue::Resource("second".into())]),
            &mut ids,
        );
        collect_resource_identities(&ReflectedValue::String("res://fake.scene".into()), &mut ids);
        assert_eq!(
            ids,
            BTreeSet::from(["asset-id".to_owned(), "second".to_owned()])
        );
    }
}
