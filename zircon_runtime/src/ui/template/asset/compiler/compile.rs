use std::collections::BTreeMap;

use crate::ui::template::UiTemplateInstance;
use crate::ui::template::{UiAssetDocument, UiAssetError};

use super::shape_validator::validate_document_shape;
use super::ui_document_compiler::{CompilationArtifacts, UiCompiledDocument, UiDocumentCompiler};
use super::ui_style_resolver::UiStyleResolver;
use super::value_normalizer::compose_tokens;

impl UiDocumentCompiler {
    pub fn compile(&self, document: &UiAssetDocument) -> Result<UiCompiledDocument, UiAssetError> {
        validate_document_shape(document)?;
        let root_id = document
            .root
            .as_ref()
            .ok_or_else(|| UiAssetError::InvalidDocument {
                asset_id: document.asset.id.clone(),
                detail: "layout/widget assets require a root node".to_string(),
            })?
            .node
            .clone();

        let mut artifacts = CompilationArtifacts::default();
        let tokens = compose_tokens(&BTreeMap::new(), &document.tokens);
        let mut roots = self.expand_node(
            document,
            &root_id,
            &tokens,
            &BTreeMap::new(),
            None,
            &mut artifacts,
        )?;
        let root = roots
            .drain(..)
            .next()
            .ok_or_else(|| UiAssetError::InvalidDocument {
                asset_id: document.asset.id.clone(),
                detail: "asset expansion produced no root nodes".to_string(),
            })?;

        let mut instance = UiTemplateInstance { root };
        UiStyleResolver::apply(document, self, &mut instance.root, &artifacts)?;

        Ok(UiCompiledDocument {
            asset: document.asset.clone(),
            instance,
        })
    }
}
