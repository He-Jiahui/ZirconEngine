use std::collections::{HashSet, VecDeque};

use crate::ui::asset_editor::UiDesignerSelectionModel;
use zircon_runtime::ui::template::UiAssetDocumentRuntimeExt;
use zircon_runtime_interface::ui::template::{
    UiAssetDocument, UiAssetHeader, UiAssetKind, UiNodeDefinition, UiNodeDefinitionKind,
    UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UiAssetExternalWidgetDraft {
    pub(crate) asset_id: String,
    pub(crate) component_name: String,
    pub(crate) document_id: String,
}

pub(crate) fn can_promote_selected_component_to_external_widget(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
) -> bool {
    selected_local_component_name(document, selection).is_some()
}

pub(crate) fn default_external_widget_draft(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
) -> Option<UiAssetExternalWidgetDraft> {
    let component_name = selected_local_component_name(document, selection)?;
    let slug = widget_slug(&component_name);
    Some(UiAssetExternalWidgetDraft {
        asset_id: format!("res://ui/widgets/{slug}.zui"),
        component_name,
        document_id: format!("ui.widgets.{slug}"),
    })
}

pub(crate) fn selected_local_component_name(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
) -> Option<String> {
    let node_id = selection.primary_node_id.as_deref()?;
    let node = document.node(node_id)?;
    if node.kind != UiNodeDefinitionKind::Component {
        return None;
    }
    let component_name = node.component.as_deref()?;
    document
        .components
        .contains_key(component_name)
        .then(|| component_name.to_string())
}

pub(crate) fn promote_selected_component_to_external_widget(
    document: &mut UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    widget_asset_id: &str,
    widget_component_name: &str,
    widget_document_id: &str,
) -> Option<UiAssetDocument> {
    let source_component_name = selected_local_component_name(document, selection)?;
    let component = document.components.get(&source_component_name)?.clone();
    let dependency_names = collect_component_dependency_names(document, &source_component_name)?;
    if widget_component_name != source_component_name
        && dependency_names.contains(widget_component_name)
    {
        return None;
    }
    let target_ref = ensure_component_ref(widget_asset_id, widget_component_name);

    let components = dependency_names
        .iter()
        .filter_map(|name| {
            document.components.get(name).cloned().map(|mut component| {
                rename_local_component_references_in_tree(
                    &mut component.root,
                    &source_component_name,
                    widget_component_name,
                );
                (
                    if name == &source_component_name {
                        widget_component_name.to_string()
                    } else {
                        name.clone()
                    },
                    component,
                )
            })
        })
        .collect();

    let mut root = component.root.clone();
    rename_local_component_references_in_tree(
        &mut root,
        &source_component_name,
        widget_component_name,
    );

    let widget_document = UiAssetDocument {
        asset: UiAssetHeader {
            kind: UiAssetKind::Widget,
            id: widget_document_id.to_string(),
            version: UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION,
            display_name: widget_component_name.to_string(),
        },
        imports: document.imports.clone(),
        tokens: document.tokens.clone(),
        root: Some(root),
        components,
        stylesheets: document.stylesheets.clone(),
    };

    if !document
        .imports
        .widgets
        .iter()
        .any(|reference| reference == &target_ref)
    {
        document.imports.widgets.push(target_ref.clone());
    }

    if let Some(root) = &mut document.root {
        convert_component_instances_to_reference(root, &source_component_name, &target_ref);
    }
    for component in document.components.values_mut() {
        convert_component_instances_to_reference(
            &mut component.root,
            &source_component_name,
            &target_ref,
        );
    }

    let _ = document.components.remove(&source_component_name);

    Some(widget_document)
}

fn ensure_component_ref(reference: &str, component_name: &str) -> String {
    if reference.contains('#') {
        reference.to_string()
    } else {
        format!("{reference}#{component_name}")
    }
}

fn collect_component_dependency_names(
    document: &UiAssetDocument,
    root_component: &str,
) -> Option<HashSet<String>> {
    let mut visited = HashSet::new();
    let mut pending = VecDeque::from([root_component.to_string()]);
    while let Some(component_name) = pending.pop_front() {
        if !visited.insert(component_name.clone()) {
            continue;
        }
        let component = document.components.get(&component_name)?;
        collect_local_component_references(&component.root, document, &mut pending);
    }
    Some(visited)
}

fn collect_local_component_references(
    node: &UiNodeDefinition,
    document: &UiAssetDocument,
    pending: &mut VecDeque<String>,
) {
    if node.kind == UiNodeDefinitionKind::Component {
        if let Some(local_component) = node.component.as_deref() {
            if document.components.contains_key(local_component) {
                pending.push_back(local_component.to_string());
            }
        }
    }
    for child in &node.children {
        collect_local_component_references(&child.node, document, pending);
    }
}

fn rename_local_component_references_in_tree(
    node: &mut UiNodeDefinition,
    source_component_name: &str,
    widget_component_name: &str,
) {
    if widget_component_name != source_component_name
        && node.kind == UiNodeDefinitionKind::Component
        && node.component.as_deref() == Some(source_component_name)
    {
        node.component = Some(widget_component_name.to_string());
    }
    for child in &mut node.children {
        rename_local_component_references_in_tree(
            &mut child.node,
            source_component_name,
            widget_component_name,
        );
    }
}

fn convert_component_instances_to_reference(
    node: &mut UiNodeDefinition,
    source_component_name: &str,
    target_ref: &str,
) {
    if node.kind == UiNodeDefinitionKind::Component
        && node.component.as_deref() == Some(source_component_name)
    {
        node.kind = UiNodeDefinitionKind::Reference;
        node.widget_type = None;
        node.component = None;
        node.component_ref = Some(target_ref.to_string());
        node.slot_name = None;
        node.props.clear();
        node.layout = None;
        node.bindings.clear();
    }
    for child in &mut node.children {
        convert_component_instances_to_reference(
            &mut child.node,
            source_component_name,
            target_ref,
        );
    }
}

fn widget_slug(component_name: &str) -> String {
    let mut slug = String::new();
    let mut previous_was_separator = true;
    let mut previous_was_lowercase = false;
    for ch in component_name.chars() {
        if ch.is_ascii_alphanumeric() {
            if ch.is_ascii_uppercase() {
                if !previous_was_separator && previous_was_lowercase {
                    slug.push('_');
                }
                slug.push(ch.to_ascii_lowercase());
                previous_was_separator = false;
                previous_was_lowercase = false;
            } else {
                slug.push(ch.to_ascii_lowercase());
                previous_was_separator = false;
                previous_was_lowercase = ch.is_ascii_lowercase();
            }
        } else if !previous_was_separator {
            slug.push('_');
            previous_was_separator = true;
            previous_was_lowercase = false;
        }
    }
    let trimmed = slug.trim_matches('_');
    if trimmed.is_empty() {
        "widget".to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod optimization_tests {
    use std::collections::{BTreeMap, BTreeSet};
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use zircon_runtime_interface::ui::template::{UiAssetImports, UiComponentDefinition};

    use super::*;

    const DEPENDENCY_ADMISSION_COUNT: usize = 65_536;
    const UNIQUE_DEPENDENCY_COUNT: usize = 8_192;
    const SAMPLE_COUNT: usize = 17;

    fn percentile_95(samples: &mut [Duration]) -> Duration {
        samples.sort_unstable();
        samples[(samples.len() - 1) * 95 / 100]
    }

    fn dependency_names() -> Vec<String> {
        (0..DEPENDENCY_ADMISSION_COUNT)
            .map(|index| {
                format!(
                    "GeneratedWidgetDependencyWithLongSharedIdentity{:05}",
                    (index * 4_099) % UNIQUE_DEPENDENCY_COUNT
                )
            })
            .collect()
    }

    fn ordered_unique_count(names: &[String]) -> usize {
        let mut visited = BTreeSet::new();
        names
            .iter()
            .filter(|name| visited.insert(name.as_str()))
            .count()
    }

    fn hash_unique_count(names: &[String]) -> usize {
        let mut visited = HashSet::new();
        names
            .iter()
            .filter(|name| visited.insert(name.as_str()))
            .count()
    }

    fn component_referencing(node_id: &str, component_name: &str) -> UiComponentDefinition {
        UiComponentDefinition {
            root: UiNodeDefinition {
                node_id: node_id.to_string(),
                kind: UiNodeDefinitionKind::Component,
                component: Some(component_name.to_string()),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn optimization_batch_20260826ac_editor23_hash_component_closure_terminates_cycles() {
        let document = UiAssetDocument {
            asset: UiAssetHeader {
                kind: UiAssetKind::Layout,
                id: "ui.tests.cyclic_components".to_string(),
                version: UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION,
                display_name: "Cyclic Components".to_string(),
            },
            imports: UiAssetImports::default(),
            tokens: BTreeMap::new(),
            root: None,
            components: BTreeMap::from([
                (
                    "Alpha".to_string(),
                    component_referencing("alpha.root", "Beta"),
                ),
                (
                    "Beta".to_string(),
                    component_referencing("beta.root", "Alpha"),
                ),
            ]),
            stylesheets: Vec::new(),
        };

        let dependencies = collect_component_dependency_names(&document, "Alpha").unwrap();

        assert_eq!(dependencies.len(), 2);
        assert!(dependencies.contains("Alpha"));
        assert!(dependencies.contains("Beta"));
    }

    #[test]
    fn optimization_batch_20260826ac_editor23_widget_promotion_uses_hash_closure_and_ordered_output(
    ) {
        let source = include_str!("promote_widget.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();

        assert!(production.contains("use std::collections::{HashSet, VecDeque};"));
        assert!(production.contains("-> Option<HashSet<String>>"));
        assert!(production.contains("let mut visited = HashSet::new();"));
        assert!(production.contains("document.components.get(name).cloned()"));
        assert!(!production.contains("BTreeSet"));
    }

    #[test]
    #[ignore = "release performance evidence"]
    fn optimization_batch_20260826ac_editor23_widget_dependency_hash_closure_performance_evidence()
    {
        let names = dependency_names();
        assert_eq!(ordered_unique_count(&names), hash_unique_count(&names));

        let mut ordered_samples = Vec::with_capacity(SAMPLE_COUNT);
        let mut hash_samples = Vec::with_capacity(SAMPLE_COUNT);
        for sample in 0..SAMPLE_COUNT {
            if sample % 2 == 0 {
                let started = Instant::now();
                black_box(ordered_unique_count(black_box(&names)));
                ordered_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(hash_unique_count(black_box(&names)));
                hash_samples.push(started.elapsed());
            } else {
                let started = Instant::now();
                black_box(hash_unique_count(black_box(&names)));
                hash_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(ordered_unique_count(black_box(&names)));
                ordered_samples.push(started.elapsed());
            }
        }

        let ordered_p95 = percentile_95(&mut ordered_samples);
        let hash_p95 = percentile_95(&mut hash_samples);
        println!(
            "EDITOR23_WIDGET_DEPENDENCY_HASH_CLOSURE_BENCH_V1 \
             admissions={DEPENDENCY_ADMISSION_COUNT} unique_dependencies={UNIQUE_DEPENDENCY_COUNT} \
             ordered_component_map=true ordered_p95_ns={} hash_p95_ns={}",
            ordered_p95.as_nanos(),
            hash_p95.as_nanos(),
        );
        assert!(
            hash_p95.as_nanos() * 100 <= ordered_p95.as_nanos() * 60,
            "hash-closure P95 {:?} exceeded 60% of ordered-closure P95 {:?}",
            hash_p95,
            ordered_p95,
        );
    }
}
