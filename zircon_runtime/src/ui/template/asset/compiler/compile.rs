use std::collections::BTreeMap;

use crate::ui::template::{validate_asset_bindings, UiTemplateInstance};
use zircon_runtime_interface::ui::template::{UiAssetDocument, UiAssetError};

use super::super::component_contract::validate_document_component_contracts;
use super::super::localization::validate_document_localization;
use super::super::resource_ref::collect_document_resource_dependencies;
use super::binding_program::compile_binding_program;
use super::cache::{compile_cache_key_from_compiler, UiAssetCompileCache, UiCompileCacheOutcome};
use super::control_scope::validate_unique_control_ids;
use super::shape_validator::validate_document_shape;
use super::ui_document_compiler::{CompilationArtifacts, UiCompiledDocument, UiDocumentCompiler};
use super::ui_style_resolver::UiStyleResolver;
use super::value_normalizer::compose_tokens;

impl UiDocumentCompiler {
    // 缓存命中也要检查当前文档与注册契约；结果复用不会免除宿主对合法输入和描述符集合的责任。
    pub(super) fn validate_compiler_preconditions(
        &self,
        document: &UiAssetDocument,
    ) -> Result<(), UiAssetError> {
        validate_document_shape(document)?;
        validate_document_localization(document)?;
        validate_document_component_contracts(document, &self.widget_imports, &self.style_imports)?;
        validate_asset_bindings(document, self.component_registry())
    }

    /// 编译宿主已加载并注册导入的布局或 widget；返回展开树、绑定程序和资源报告，运行时安装仍由构建层完成。
    pub fn compile(&self, document: &UiAssetDocument) -> Result<UiCompiledDocument, UiAssetError> {
        self.validate_compiler_preconditions(document)?;
        self.compile_validated(document)
    }

    // 仅供已完成当前前置检查的入口复用；缓存未命中和打包都从此处进入，避免重复扫描同一文档。
    pub(super) fn compile_validated(
        &self,
        document: &UiAssetDocument,
    ) -> Result<UiCompiledDocument, UiAssetError> {
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

        // 组件局部控件名已映射为实例身份；在绑定端点生成前拒绝最终重名，保证树与引用共有唯一控制目标。
        validate_unique_control_ids(&root, &document.asset.id)?;

        let mut instance = UiTemplateInstance::new(root);
        UiStyleResolver::apply(document, self, &mut instance.root, &artifacts)?;
        let binding_program = compile_binding_program(&instance.root, &document.asset.id)?;
        let instance = UiTemplateInstance::with_binding_program(instance.root, binding_program);
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

    /// 用宿主提供的缓存复用完整编译结果；导入或注册表改变会进入新键，未命中报告只解释相对上次快照的变化。
    pub fn compile_with_cache(
        &self,
        document: &UiAssetDocument,
        cache: &mut UiAssetCompileCache,
    ) -> Result<UiCompileCacheOutcome, UiAssetError> {
        self.validate_compiler_preconditions(document)?;
        let key = compile_cache_key_from_compiler(self, document)?;
        if let Some(compiled) = cache.get(&key) {
            return Ok(UiCompileCacheOutcome {
                compiled,
                cache_hit: true,
                invalidation_report: Default::default(),
            });
        }

        let invalidation_report = cache.report_for_miss(&key, document);
        let compiled = self.compile_validated(document)?;
        cache.store(key, compiled.clone());
        Ok(UiCompileCacheOutcome {
            compiled,
            cache_hit: false,
            invalidation_report,
        })
    }
}
