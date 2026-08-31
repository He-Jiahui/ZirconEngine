use std::collections::BTreeSet;

use zircon_runtime_interface::ui::template::{
    UiAssetChange, UiAssetDocument, UiInvalidationReport, UiInvalidationSnapshot,
    UiInvalidationStage,
};

use super::collect_invalidation_diagnostics;

#[derive(Clone, Debug, Default)]
pub struct UiInvalidationGraph;

impl UiInvalidationGraph {
    pub fn classify(
        previous: Option<&UiInvalidationSnapshot>,
        next: &UiInvalidationSnapshot,
        document: &UiAssetDocument,
    ) -> UiInvalidationReport {
        let diagnostics = collect_invalidation_diagnostics(document);
        let Some(previous) = previous else {
            return UiInvalidationReport::from_stages(
                vec![UiAssetChange::Document],
                full_rebuild_stages(),
                diagnostics,
            );
        };

        let mut changes = Vec::new();
        let mut stages = BTreeSet::new();
        let full_rebuild = previous.document != next.document;
        if full_rebuild {
            changes.push(UiAssetChange::Document);
            stages.extend(full_rebuild_stages());
        }
        if previous.widget_imports != next.widget_imports
            || previous.declared_widget_imports_revision != next.declared_widget_imports_revision
        {
            changes.push(UiAssetChange::WidgetImport);
            extend_incremental_stages(
                &mut stages,
                full_rebuild,
                [
                    UiInvalidationStage::ImportGraph,
                    UiInvalidationStage::ComponentContract,
                    UiInvalidationStage::SelectorMatch,
                    UiInvalidationStage::StyleValue,
                    UiInvalidationStage::Layout,
                    UiInvalidationStage::Render,
                ],
            );
        }
        if previous.style_imports != next.style_imports
            || previous.declared_style_imports_revision != next.declared_style_imports_revision
        {
            changes.push(UiAssetChange::StyleImport);
            extend_incremental_stages(
                &mut stages,
                full_rebuild,
                [
                    UiInvalidationStage::ImportGraph,
                    UiInvalidationStage::SelectorMatch,
                    UiInvalidationStage::StyleValue,
                    UiInvalidationStage::Layout,
                    UiInvalidationStage::Render,
                ],
            );
        }
        if previous.descriptor_registry_revision != next.descriptor_registry_revision {
            changes.push(UiAssetChange::DescriptorRegistry);
            extend_incremental_stages(
                &mut stages,
                full_rebuild,
                [
                    UiInvalidationStage::DescriptorRegistry,
                    UiInvalidationStage::Layout,
                    UiInvalidationStage::Render,
                ],
            );
        }
        if previous.component_contract_revision != next.component_contract_revision {
            changes.push(UiAssetChange::ComponentContract);
            extend_incremental_stages(
                &mut stages,
                full_rebuild,
                [
                    UiInvalidationStage::ComponentContract,
                    UiInvalidationStage::SelectorMatch,
                    UiInvalidationStage::StyleValue,
                    UiInvalidationStage::Layout,
                    UiInvalidationStage::Render,
                ],
            );
        }
        if previous.resource_dependencies_revision != next.resource_dependencies_revision {
            changes.push(UiAssetChange::ResourceDependency);
            extend_incremental_stages(
                &mut stages,
                full_rebuild,
                [
                    UiInvalidationStage::ResourceDependency,
                    UiInvalidationStage::Render,
                    UiInvalidationStage::Projection,
                ],
            );
        }
        UiInvalidationReport::from_stages(changes, stages, diagnostics)
    }
}

fn extend_incremental_stages(
    stages: &mut BTreeSet<UiInvalidationStage>,
    full_rebuild: bool,
    additions: impl IntoIterator<Item = UiInvalidationStage>,
) {
    if !full_rebuild {
        stages.extend(additions);
    }
}

fn full_rebuild_stages() -> BTreeSet<UiInvalidationStage> {
    [
        UiInvalidationStage::SourceParse,
        UiInvalidationStage::DocumentShape,
        UiInvalidationStage::SelectorMatch,
        UiInvalidationStage::StyleValue,
        UiInvalidationStage::Layout,
        UiInvalidationStage::Render,
        UiInvalidationStage::Interaction,
        UiInvalidationStage::Projection,
    ]
    .into_iter()
    .collect()
}

#[cfg(test)]
#[path = "graph/full_rebuild_tests.rs"]
mod full_rebuild_tests;
