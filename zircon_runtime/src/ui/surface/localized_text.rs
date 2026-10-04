use super::{UiPropertyMutationRequest, UiPropertyMutationStatus, UiSurface};
use thiserror::Error;
use zircon_runtime_interface::ui::{
    component::UiValue, event_ui::UiNodeId, template::UiLocalizedTextRef, tree::UiTreeError,
};

#[derive(Debug, Error)]
pub enum UiLocalizedTextSynchronizationError<E: std::error::Error + 'static> {
    #[error(transparent)]
    Tree(#[from] UiTreeError),
    #[error("could not resolve localized property `{property}` on node {node_id:?}")]
    Resolve {
        node_id: UiNodeId,
        property: String,
        #[source]
        source: E,
    },
    #[error("localized property `{property}` on node {node_id:?} was rejected: {message}")]
    Rejected {
        node_id: UiNodeId,
        property: String,
        message: String,
    },
}

impl UiSurface {
    /// Resolves source references into the same typed property and metadata used by
    /// layout, rendering, and retained projections. The caller owns locale/catalog.
    pub fn synchronize_localized_text<E: std::error::Error + 'static>(
        &mut self,
        mut resolve: impl FnMut(&UiLocalizedTextRef) -> Result<String, E>,
    ) -> Result<usize, UiLocalizedTextSynchronizationError<E>> {
        let mut bindings = Vec::new();
        for (&node_id, node) in &mut self.tree.nodes {
            let Some(metadata) = node.template_metadata.as_mut() else {
                continue;
            };
            for (property, value) in &metadata.attributes {
                let toml::Value::Table(table) = value else {
                    continue;
                };
                let Some(key) = table.get("text_key").and_then(toml::Value::as_str) else {
                    continue;
                };
                metadata
                    .localized_text_references
                    .entry(property.clone())
                    .or_insert_with(|| UiLocalizedTextRef {
                        key: key.to_owned(),
                        table: table
                            .get("table")
                            .and_then(toml::Value::as_str)
                            .map(str::to_owned),
                        fallback: table
                            .get("fallback")
                            .and_then(toml::Value::as_str)
                            .map(str::to_owned),
                    });
            }
            bindings.extend(
                metadata
                    .localized_text_references
                    .iter()
                    .map(|(property, reference)| (node_id, property.clone(), reference.clone())),
            );
        }
        // Resolve the whole captured locale projection before committing property writes.
        let resolved = bindings
            .into_iter()
            .map(|(node_id, property, reference)| {
                resolve(&reference)
                    .map(|text| (node_id, property.clone(), reference, text))
                    .map_err(|source| UiLocalizedTextSynchronizationError::Resolve {
                        node_id,
                        property,
                        source,
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut changed = 0;
        for (node_id, property, reference, text) in resolved {
            let report = self.mutate_property(UiPropertyMutationRequest::new(
                node_id,
                property.clone(),
                UiValue::String(text),
            ))?;
            if report.status == UiPropertyMutationStatus::Rejected {
                return Err(UiLocalizedTextSynchronizationError::Rejected {
                    node_id,
                    property,
                    message: report.message.unwrap_or_default(),
                });
            }
            changed += usize::from(report.status == UiPropertyMutationStatus::Accepted);
            // The generic mutation clears bindings for explicit writes. This resolver
            // retains this source binding so a later locale change can resolve it again.
            self.tree
                .node_mut(node_id)
                .expect("the property mutation retains its node")
                .template_metadata
                .as_mut()
                .expect("the property mutation retains metadata")
                .localized_text_references
                .insert(property, reference);
        }
        Ok(changed)
    }
}

#[cfg(test)]
#[path = "localized_text/tests/cases.rs"]
mod tests;
