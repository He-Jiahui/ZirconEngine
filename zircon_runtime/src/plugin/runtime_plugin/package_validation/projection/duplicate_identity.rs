use std::hash::Hash;

use super::EmbeddedFeatureKind;

/// 散列键中的变体区分语义域，嵌套行号限定局部作用域；文本借用原清单以免复制身份字符串。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum DuplicateIdentity<'a> {
    PackageCapability(&'a str),
    AssetRoot(&'a str),
    ContentRoot(&'a str),
    AssetImporterId(&'a str),
    AssetImporterCapability {
        importer: usize,
        value: &'a str,
    },
    DependencyCapability {
        provider: &'a str,
        capability: &'a str,
    },
    CapabilityStatus(&'a str),
    CapabilityStatusReference {
        status: usize,
        value: &'a str,
    },
    OptionKey(&'a str),
    EventCatalogNamespace(&'a str),
    ComponentTypeId(&'a str),
    UiComponentId(&'a str),
    EmbeddedFeatureProvider {
        feature: &'a str,
        provider: &'a str,
    },
    FeatureCapability {
        kind: EmbeddedFeatureKind,
        feature: usize,
        value: &'a str,
    },
    FeatureDependency {
        kind: EmbeddedFeatureKind,
        feature: usize,
        provider: &'a str,
        capability: &'a str,
    },
    FeatureModuleName {
        kind: EmbeddedFeatureKind,
        feature: usize,
        value: &'a str,
    },
    FeatureModuleCapability {
        kind: EmbeddedFeatureKind,
        feature: usize,
        module: usize,
        value: &'a str,
    },
    DependencyInterface {
        dependency: usize,
        value: &'a str,
    },
    ProvidedInterface(&'a str),
    ProvidedMethodName {
        interface: usize,
        value: &'a str,
    },
    ProvidedMethodSlot {
        interface: usize,
        value: u32,
    },
    ProvidedMethodCapability {
        interface: usize,
        method: usize,
        value: &'a str,
    },
    PackageModuleName(&'a str),
    PackageModuleCapability {
        module: usize,
        value: &'a str,
    },
    PackageModuleSystemSet {
        module: usize,
        value: &'a str,
    },
    PackageModuleSystemAnchor {
        module: usize,
        value: &'a str,
    },
}
