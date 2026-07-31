#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(in crate::plugin::runtime_plugin) enum EmbeddedFeatureKind {
    Optional,
    Extension,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum DuplicateOccurrence {
    PackageCapability(usize),
    AssetRoot(usize),
    ContentRoot(usize),
    AssetImporterId(usize),
    AssetImporterCapability {
        importer: usize,
        capability: usize,
    },
    DependencyCapability(usize),
    CapabilityStatus(usize),
    CapabilityStatusReference {
        status: usize,
        reference: usize,
    },
    OptionKey(usize),
    EventCatalogNamespace(usize),
    ComponentTypeId(usize),
    UiComponentId(usize),
    EmbeddedFeatureProvider {
        kind: EmbeddedFeatureKind,
        feature: usize,
    },
    FeatureCapability {
        kind: EmbeddedFeatureKind,
        feature: usize,
        capability: usize,
    },
    FeatureDependency {
        kind: EmbeddedFeatureKind,
        feature: usize,
        dependency: usize,
    },
    FeatureModuleName {
        kind: EmbeddedFeatureKind,
        feature: usize,
        module: usize,
    },
    FeatureModuleCapability {
        kind: EmbeddedFeatureKind,
        feature: usize,
        module: usize,
        capability: usize,
    },
    DependencyInterface {
        dependency: usize,
        interface: usize,
    },
    ProvidedInterface(usize),
    ProvidedMethodName {
        interface: usize,
        method: usize,
    },
    ProvidedMethodSlot {
        interface: usize,
        method: usize,
    },
    ProvidedMethodCapability {
        interface: usize,
        method: usize,
        capability: usize,
    },
    PackageModuleName(usize),
    PackageModuleCapability {
        module: usize,
        capability: usize,
    },
    PackageModuleSystemSet {
        module: usize,
        system_set: usize,
    },
    PackageModuleSystemAnchor {
        module: usize,
        system_anchor: usize,
    },
}
