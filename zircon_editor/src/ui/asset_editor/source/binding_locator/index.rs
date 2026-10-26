use std::collections::BTreeMap;
use std::ops::Range;

use serde::Deserialize;
use toml::Spanned;
use zircon_runtime_interface::ui::{template::UiBindingRef, v2::UiV2AssetDocument};

use super::{
    ownership::source_owners, positions::source_range, UiAssetBindingSourceDeclaration,
    UiAssetBindingSourceError as Error, UiAssetBindingSourceSelector,
};

#[derive(Debug)]
struct SourceDocument {
    nodes: BTreeMap<String, SourceNode>,
}

#[derive(Debug)]
struct SourceNode {
    events: Vec<SourceBinding>,
}

#[derive(Debug)]
struct SourceBinding {
    binding: UiBindingRef,
    declaration: Range<usize>,
    id: Range<usize>,
    event: Range<usize>,
    component_event: Option<Range<usize>>,
}

/// Lazily attached to the existing source outline artifact; no parallel source store.
#[derive(Debug)]
pub(crate) struct UiAssetBindingSourceIndex {
    source_digest: blake3::Hash,
    document: Result<SourceDocument, Error>,
}

impl UiAssetBindingSourceIndex {
    pub(crate) fn parse(source: &str) -> Self {
        Self {
            source_digest: blake3::hash(source.as_bytes()),
            document: parse_document(source),
        }
    }

    pub(crate) fn locate(
        &self,
        source: &str,
        document: &UiV2AssetDocument,
        selector: &UiAssetBindingSourceSelector,
    ) -> Result<UiAssetBindingSourceDeclaration, Error> {
        if self.source_digest != blake3::hash(source.as_bytes()) {
            return Err(Error::SourceMismatch);
        }
        let source_document = self.document.as_ref().map_err(Clone::clone)?;
        let node = document
            .nodes
            .get(&selector.node_id)
            .ok_or_else(|| Error::MissingNode {
                node_id: selector.node_id.clone(),
            })?;
        let source_node = source_document
            .nodes
            .get(&selector.node_id)
            .ok_or(Error::SourceMismatch)?;
        if source_node.events.len() != node.events.len() {
            return Err(Error::SourceMismatch);
        }
        // The typed validated document supplies the original route, targets and payload.
        // The span projection must refer to those same declarations in the same order.
        for (source_binding, binding) in source_node.events.iter().zip(&node.events) {
            if &source_binding.binding != binding {
                return Err(Error::SourceMismatch);
            }
        }
        let matches = node
            .events
            .iter()
            .enumerate()
            .filter(|(_, binding)| binding.id == selector.binding_id)
            .collect::<Vec<_>>();
        let (index, binding) = match matches.as_slice() {
            [] => {
                return Err(Error::MissingBinding {
                    binding_id: selector.binding_id.clone(),
                })
            }
            [entry] => *entry,
            _ => {
                return Err(Error::AmbiguousBinding {
                    binding_id: selector.binding_id.clone(),
                    count: matches.len(),
                })
            }
        };
        if binding.event != selector.event || binding.component_event != selector.component_event {
            return Err(Error::EventMismatch);
        }
        let owners = source_owners(document, &selector.node_id);
        let owner = if let Some(expected) = selector.owner.as_ref() {
            owners
                .iter()
                .find(|owner| *owner == expected)
                .cloned()
                .ok_or(Error::OwnerMismatch)?
        } else {
            match owners.as_slice() {
                [] => return Err(Error::MissingOwner),
                [owner] => owner.clone(),
                _ => return Err(Error::AmbiguousOwner { owners }),
            }
        };
        let value = &source_node.events[index];
        Ok(UiAssetBindingSourceDeclaration {
            asset_id: document.asset.id.clone(),
            node_id: selector.node_id.clone(),
            owner,
            control_component: node.component.clone(),
            source_binding_index: index,
            binding: binding.clone(),
            declaration: source_range(source, value.declaration.clone())?,
            id: source_range(source, value.id.clone())?,
            event: source_range(source, value.event.clone())?,
            component_event: value
                .component_event
                .as_ref()
                .map(|event| source_range(source, event.clone()))
                .transpose()?,
        })
    }
}

fn parse_document(source: &str) -> Result<SourceDocument, Error> {
    let parse_error = |error: toml::de::Error| Error::ParseSource {
        detail: error.to_string(),
    };
    let syntax = toml::de::DeTable::parse(source).map_err(parse_error)?;
    let mut document = SourceDocument {
        nodes: BTreeMap::new(),
    };
    if let Some(nodes) = syntax
        .get_ref()
        .get("nodes")
        .and_then(|value| value.get_ref().as_table())
    {
        for (node_id, node) in nodes {
            let node = node.get_ref().as_table().ok_or(Error::SourceMismatch)?;
            let mut entries = Vec::new();
            if let Some(events) = node.get("events") {
                let events = events.get_ref().as_array().ok_or(Error::SourceMismatch)?;
                for event in events.iter() {
                    let table = event.get_ref().as_table().ok_or(Error::SourceMismatch)?;
                    // Deserialize the engine's original binding type from this same parsed
                    // syntax value. Retain only owned data and byte ranges, never AST borrows.
                    let binding =
                        UiBindingRef::deserialize(toml::de::ValueDeserializer::from(event.clone()))
                            .map_err(parse_error)?;
                    entries.push(SourceBinding {
                        binding,
                        declaration: declaration_range(event),
                        id: table.get("id").ok_or(Error::SourceMismatch)?.span(),
                        event: table.get("event").ok_or(Error::SourceMismatch)?.span(),
                        component_event: table.get("component_event").map(Spanned::span),
                    });
                }
            }
            document.nodes.insert(
                node_id.get_ref().to_string(),
                SourceNode { events: entries },
            );
        }
    }
    Ok(document)
}

fn declaration_range(value: &Spanned<toml::de::DeValue<'_>>) -> Range<usize> {
    let mut range = value.span();
    let mut pending = vec![value];
    while let Some(value) = pending.pop() {
        range.start = range.start.min(value.span().start);
        range.end = range.end.max(value.span().end);
        match value.get_ref() {
            toml::de::DeValue::Table(table) => {
                for (key, child) in table {
                    range.start = range.start.min(key.span().start);
                    range.end = range.end.max(key.span().end);
                    pending.push(child);
                }
            }
            toml::de::DeValue::Array(array) => pending.extend(array.iter()),
            _ => {}
        }
    }
    // TOML array-of-table spans cover the header alone. Include all descendant
    // key/value spans to obtain this declaration, without consuming the next one.
    range
}

#[cfg(test)]
#[path = "tests/index.rs"]
mod tests;
