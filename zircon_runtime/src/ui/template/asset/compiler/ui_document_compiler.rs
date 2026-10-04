use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};

use toml::Value;

use crate::ui::component::UiComponentDescriptorRegistry;
use crate::ui::template::UiTemplateInstance;
use zircon_runtime_interface::ui::component::UiComponentDescriptor;
use zircon_runtime_interface::ui::template::{
    UiAssetDocument, UiAssetError, UiAssetHeader, UiAssetKind, UiResourceDependency,
    UiResourceDiagnostic, UiStyleSheet,
};

use super::value_normalizer::compose_tokens;

/// 宿主可重复实例化的编译结果，同时携带资源依赖和诊断；编译不会加载这些资源，也不创建运行时树。
#[derive(Clone, Debug, PartialEq)]
pub struct UiCompiledDocument {
    pub asset: UiAssetHeader,
    pub(super) instance: UiTemplateInstance,
    pub resource_dependencies: Vec<UiResourceDependency>,
    pub resource_diagnostics: Vec<UiResourceDiagnostic>,
}

impl UiCompiledDocument {
    /// 将模板交给实例或包构建层；资源报告不随此转换保留，仍需它的宿主应先记录依赖或保留完整编译结果。
    pub fn into_template_instance(self) -> UiTemplateInstance {
        self.instance
    }

    pub fn template_instance(&self) -> &UiTemplateInstance {
        &self.instance
    }

    pub fn resource_dependencies(&self) -> &[UiResourceDependency] {
        &self.resource_dependencies
    }

    pub fn resource_diagnostics(&self) -> &[UiResourceDiagnostic] {
        &self.resource_diagnostics
    }
}

/// 宿主先注册已解析的导入，再在当前描述符契约下编译；默认借用展示组件注册表，定制组件应显式提供自己的注册表。
pub struct UiDocumentCompiler {
    pub(super) widget_imports: BTreeMap<String, UiAssetDocument>,
    pub(super) style_imports: BTreeMap<String, UiAssetDocument>,
    component_registry: Cow<'static, UiComponentDescriptorRegistry>,
}

impl Default for UiDocumentCompiler {
    fn default() -> Self {
        Self {
            widget_imports: BTreeMap::new(),
            style_imports: BTreeMap::new(),
            component_registry: Cow::Borrowed(
                UiComponentDescriptorRegistry::editor_showcase_shared(),
            ),
        }
    }
}

impl UiDocumentCompiler {
    /// 接管宿主构造的注册表快照；编译缓存会纳入其 revision，以免复用另一套默认值和组件约束。
    pub fn with_component_registry(mut self, registry: UiComponentDescriptorRegistry) -> Self {
        self.component_registry = Cow::Owned(registry);
        self
    }

    /// 借用全进程有效的只读注册表，适合多个短寿命编译器共享同一契约；生命周期限制防止悬挂描述符。
    pub fn with_shared_component_registry(
        mut self,
        registry: &'static UiComponentDescriptorRegistry,
    ) -> Self {
        self.component_registry = Cow::Borrowed(registry);
        self
    }

    pub(super) fn component_descriptor(
        &self,
        component_id: &str,
    ) -> Option<&UiComponentDescriptor> {
        self.component_registry.descriptor(component_id)
    }

    pub(super) fn component_registry_revision(&self) -> u64 {
        self.component_registry.revision()
    }

    /// 供绑定报告等调用方使用与实际编译相同的描述符契约，避免诊断与最终实例采用两套组件定义。
    pub fn component_registry(&self) -> &UiComponentDescriptorRegistry {
        self.component_registry.as_ref()
    }

    // 仅为性能对照保留旧的拥有式默认构造；业务入口仍共享静态注册表，不能将计时基线接回普通编译。
    #[cfg(test)]
    pub(crate) fn legacy_owned_default_for_benchmark() -> Self {
        Self {
            widget_imports: BTreeMap::new(),
            style_imports: BTreeMap::new(),
            component_registry: Cow::Owned(
                UiComponentDescriptorRegistry::editor_showcase_shared().clone(),
            ),
        }
    }

    /// 宿主以文档中的完整组件引用注册布局/widget；同键再次注册替换旧快照，相关缓存键随内容改变而失效。
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

    /// 只接受 style 资产；宿主负责解析引用并提供快照，编译器不会在应用样式时再访问文件系统。
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

// 单次展开的旁路产物按资产首次出现顺序收集，样式解析在所有组件根展开完毕后统一执行。
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
        // TODO: [CR-UI-TEMPLATE-COMP-0002] 确认同一 widget 在不同继承 token 域下能否共享首份样式；当前只按资产 ID 去重并向全树应用；缺少不同父 token 的重复实例回归，下一步对照原型样式与实例隔离契约。
        if !self.seen_widget_assets.insert(document.asset.id.clone()) {
            return;
        }
        let tokens = compose_tokens(inherited, &document.tokens);
        append_resolved_stylesheets(&mut self.widget_styles, &document.stylesheets, tokens);
    }

    pub(super) fn widget_styles(&self) -> &[ResolvedStyleSheet] {
        &self.widget_styles
    }
}

// 样式必须随定义它的 token 域一起进入规则计划，避免应用阶段把导入样式误按布局自身的 token 解释。
#[derive(Clone)]
pub(super) struct ResolvedStyleSheet {
    pub(super) stylesheet: UiStyleSheet,
    pub(super) tokens: BTreeMap<String, Value>,
}

// 保持资产内样式顺序以维持同 specificity 的后写优先级；最后一张表接管 token，减少收集阶段的末次深拷贝。
fn append_resolved_stylesheets(
    output: &mut Vec<ResolvedStyleSheet>,
    stylesheets: &[UiStyleSheet],
    tokens: BTreeMap<String, Value>,
) {
    let Some((last, preceding)) = stylesheets.split_last() else {
        return;
    };
    output.extend(preceding.iter().map(|stylesheet| ResolvedStyleSheet {
        stylesheet: stylesheet.clone(),
        tokens: tokens.clone(),
    }));
    output.push(ResolvedStyleSheet {
        stylesheet: last.clone(),
        tokens,
    });
}

// 普通回归守住顺序和空输入；ignored 计时只衡量最终 token 所有权移交，不能代替多实例样式域的语义验证。
#[cfg(test)]
#[path = "tests/ui_document_compiler_performance_tests.rs"]
mod performance_tests;
