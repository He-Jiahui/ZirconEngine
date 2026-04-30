use std::collections::{BTreeMap, BTreeSet};

use toml::Value;

use crate::ui::component::{UiComponentDescriptor, UiComponentDescriptorRegistry};
use crate::ui::template::{
    UiAssetDocument, UiAssetError, UiAssetHeader, UiAssetKind, UiStyleSheet, UiTemplateInstance,
};

use super::value_normalizer::compose_tokens;

#[derive(Clone, Debug, PartialEq)]
pub struct UiCompiledDocument {
    pub asset: UiAssetHeader,
    pub(super) instance: UiTemplateInstance,
}

impl UiCompiledDocument {
    pub fn into_template_instance(self) -> UiTemplateInstance {
        self.instance
    }

    pub fn template_instance(&self) -> &UiTemplateInstance {
        &self.instance
    }
}

pub struct UiDocumentCompiler {
    pub(super) widget_imports: BTreeMap<String, UiAssetDocument>,
    pub(super) style_imports: BTreeMap<String, UiAssetDocument>,
    component_registry: UiComponentDescriptorRegistry,
}

impl Default for UiDocumentCompiler {
    fn default() -> Self {
        Self {
            widget_imports: BTreeMap::new(),
            style_imports: BTreeMap::new(),
            component_registry: UiComponentDescriptorRegistry::editor_showcase(),
        }
    }
}

impl UiDocumentCompiler {
    pub fn with_component_registry(mut self, registry: UiComponentDescriptorRegistry) -> Self {
        self.component_registry = registry;
        self
    }

    pub(super) fn component_descriptor(
        &self,
        component_id: &str,
    ) -> Option<&UiComponentDescriptor> {
        self.component_registry.descriptor(component_id)
    }

    pub fn register_widget_import(
        &mut self,
        reference: impl Into<String>,
        document: UiAssetDocument,
    ) -> Result<&mut Self, UiAssetError> {
        let reference = reference.into();
        if !matches!(
            document.asset.kind,
            UiAssetKind::Layout | UiAssetKind::Widget
        ) {
            return Err(UiAssetError::ImportKindMismatch {
                reference,
                expected: UiAssetKind::Widget,
                actual: document.asset.kind,
            });
        }
        let _ = self.widget_imports.insert(reference, document);
        Ok(self)
    }

    pub fn register_style_import(
        &mut self,
        reference: impl Into<String>,
        document: UiAssetDocument,
    ) -> Result<&mut Self, UiAssetError> {
        let reference = reference.into();
        if document.asset.kind != UiAssetKind::Style {
            return Err(UiAssetError::ImportKindMismatch {
                reference,
                expected: UiAssetKind::Style,
                actual: document.asset.kind,
            });
        }
        let _ = self.style_imports.insert(reference, document);
        Ok(self)
    }
}

#[derive(Default)]
pub(super) struct CompilationArtifacts {
    widget_styles: Vec<ResolvedStyleSheet>,
    seen_widget_assets: BTreeSet<String>,
}

impl CompilationArtifacts {
    pub(super) fn record_widget_styles(
        &mut self,
        document: &UiAssetDocument,
        inherited: &BTreeMap<String, Value>,
    ) {
        if !self.seen_widget_assets.insert(document.asset.id.clone()) {
            return;
        }
        let tokens = compose_tokens(inherited, &document.tokens);
        for stylesheet in &document.stylesheets {
            self.widget_styles.push(ResolvedStyleSheet {
                stylesheet: stylesheet.clone(),
                tokens: tokens.clone(),
            });
        }
    }

    pub(super) fn widget_styles(&self) -> &[ResolvedStyleSheet] {
        &self.widget_styles
    }
}

#[derive(Clone)]
pub(super) struct ResolvedStyleSheet {
    pub(super) stylesheet: UiStyleSheet,
    pub(super) tokens: BTreeMap<String, Value>,
}
