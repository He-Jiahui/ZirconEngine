use std::collections::BTreeMap;

use crate::ui::template::{validate_asset_bindings, UiTemplateInstance};
use zircon_runtime_interface::ui::template::{UiAssetDocument, UiAssetError};

use super::super::component_contract::validate_document_component_contracts;
use super::super::localization::validate_document_localization;
use super::super::resource_ref::collect_document_resource_dependencies;
use super::cache::{UiAssetCompileCache, UiCompileCacheKey, UiCompileCacheOutcome};
use super::shape_validator::validate_document_shape;
use super::ui_document_compiler::{CompilationArtifacts, UiCompiledDocument, UiDocumentCompiler};
use super::ui_style_resolver::UiStyleResolver;
use super::value_normalizer::compose_tokens;

impl UiDocumentCompiler {
    pub(super) fn validate_compiler_preconditions(
        &self,
        document: &UiAssetDocument,
    ) -> Result<(), UiAssetError> {
        validate_document_shape(document)?;
        validate_document_localization(document)?;
        validate_document_component_contracts(document, &self.widget_imports, &self.style_imports)?;
        validate_asset_bindings(document, self.component_registry())
    }

    pub fn compile(&self, document: &UiAssetDocument) -> Result<UiCompiledDocument, UiAssetError> {
        self.validate_compiler_preconditions(document)?;
        let root = document
            .root
            .as_ref()
            .ok_or_else(|| UiAssetError::InvalidDocument {
                asset_id: document.asset.id.clone(),
                detail: "layout/widget assets require a root node".to_string(),
            })?;

        let mut artifacts = CompilationArtifacts::default();
        let tokens = compose_tokens(&BTreeMap::new(), &document.tokens);
        let mut roots = self.expand_node(
            document,
            root,
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
        let resource_report = collect_document_resource_dependencies(
            document,
            &self.widget_imports,
            &self.style_imports,
        )?;

        Ok(UiCompiledDocument {
            asset: document.asset.clone(),
            instance,
            resource_dependencies: resource_report.dependencies,
            resource_diagnostics: resource_report.diagnostics,
        })
    }

    pub fn compile_with_cache(
        &self,
        document: &UiAssetDocument,
        cache: &mut UiAssetCompileCache,
    ) -> Result<UiCompileCacheOutcome, UiAssetError> {
        self.validate_compiler_preconditions(document)?;
        let key = UiCompileCacheKey::from_compiler(self, document)?;
        if let Some(compiled) = cache.get(&key) {
            return Ok(UiCompileCacheOutcome {
                compiled,
                cache_hit: true,
                invalidation_report: Default::default(),
            });
        }

        let invalidation_report = cache.report_for_miss(&key, document);
        let compiled = self.compile(document)?;
        cache.store(key, compiled.clone());
        Ok(UiCompileCacheOutcome {
            compiled,
            cache_hit: false,
            invalidation_report,
        })
    }
}
