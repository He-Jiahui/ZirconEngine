use super::duplicate_occurrence::{DuplicateOccurrence, EmbeddedFeatureKind};
use super::{RuntimePluginPackageValidationMetrics, RuntimePluginPackageValidationProjection};

// 重复查询的行号必须来自建立此投影的原清单，且保持数组顺序；查询本身不会校验行号对应的声明。
impl RuntimePluginPackageValidationProjection<'_> {
    /// 当前投影的操作计数；测试入口另外填入实际构建次数和内嵌特性视图次数。
    pub(in crate::plugin::runtime_plugin) fn metrics(
        &self,
    ) -> RuntimePluginPackageValidationMetrics {
        RuntimePluginPackageValidationMetrics {
            projection_builds: 1,
            standalone_feature_projection_builds: 0,
            embedded_feature_projection_views: 0,
            identity_rows_indexed: self.identity_rows_indexed,
            membership_probes: self.membership_probes.get(),
        }
    }

    pub(in crate::plugin::runtime_plugin) fn package_capability_is_duplicate(
        &self,
        index: usize,
    ) -> bool {
        self.is_duplicate(DuplicateOccurrence::PackageCapability(index))
    }

    pub(in crate::plugin::runtime_plugin) fn asset_root_is_duplicate(&self, index: usize) -> bool {
        self.is_duplicate(DuplicateOccurrence::AssetRoot(index))
    }

    pub(in crate::plugin::runtime_plugin) fn content_root_is_duplicate(
        &self,
        index: usize,
    ) -> bool {
        self.is_duplicate(DuplicateOccurrence::ContentRoot(index))
    }

    pub(in crate::plugin::runtime_plugin) fn asset_importer_id_is_duplicate(
        &self,
        importer: usize,
    ) -> bool {
        self.is_duplicate(DuplicateOccurrence::AssetImporterId(importer))
    }

    pub(in crate::plugin::runtime_plugin) fn asset_importer_capability_is_duplicate(
        &self,
        importer: usize,
        capability: usize,
    ) -> bool {
        self.is_duplicate(DuplicateOccurrence::AssetImporterCapability {
            importer,
            capability,
        })
    }

    pub(in crate::plugin::runtime_plugin) fn dependency_capability_is_duplicate(
        &self,
        dependency: usize,
    ) -> bool {
        self.is_duplicate(DuplicateOccurrence::DependencyCapability(dependency))
    }

    pub(in crate::plugin::runtime_plugin) fn capability_status_is_duplicate(
        &self,
        status: usize,
    ) -> bool {
        self.is_duplicate(DuplicateOccurrence::CapabilityStatus(status))
    }

    pub(in crate::plugin::runtime_plugin) fn capability_status_reference_is_duplicate(
        &self,
        status: usize,
        reference: usize,
    ) -> bool {
        self.is_duplicate(DuplicateOccurrence::CapabilityStatusReference { status, reference })
    }

    pub(in crate::plugin::runtime_plugin) fn option_key_is_duplicate(&self, option: usize) -> bool {
        self.is_duplicate(DuplicateOccurrence::OptionKey(option))
    }

    pub(in crate::plugin::runtime_plugin) fn event_catalog_namespace_is_duplicate(
        &self,
        catalog: usize,
    ) -> bool {
        self.is_duplicate(DuplicateOccurrence::EventCatalogNamespace(catalog))
    }

    pub(in crate::plugin::runtime_plugin) fn component_type_id_is_duplicate(
        &self,
        component: usize,
    ) -> bool {
        self.is_duplicate(DuplicateOccurrence::ComponentTypeId(component))
    }

    pub(in crate::plugin::runtime_plugin) fn ui_component_id_is_duplicate(
        &self,
        component: usize,
    ) -> bool {
        self.is_duplicate(DuplicateOccurrence::UiComponentId(component))
    }

    pub(in crate::plugin::runtime_plugin) fn embedded_feature_provider_is_duplicate(
        &self,
        kind: EmbeddedFeatureKind,
        feature: usize,
    ) -> bool {
        self.is_duplicate(DuplicateOccurrence::EmbeddedFeatureProvider { kind, feature })
    }

    pub(in crate::plugin::runtime_plugin) fn feature_capability_is_duplicate(
        &self,
        kind: EmbeddedFeatureKind,
        feature: usize,
        capability: usize,
    ) -> bool {
        self.is_duplicate(DuplicateOccurrence::FeatureCapability {
            kind,
            feature,
            capability,
        })
    }

    pub(in crate::plugin::runtime_plugin) fn feature_dependency_is_duplicate(
        &self,
        kind: EmbeddedFeatureKind,
        feature: usize,
        dependency: usize,
    ) -> bool {
        self.is_duplicate(DuplicateOccurrence::FeatureDependency {
            kind,
            feature,
            dependency,
        })
    }

    pub(in crate::plugin::runtime_plugin) fn feature_module_name_is_duplicate(
        &self,
        kind: EmbeddedFeatureKind,
        feature: usize,
        module: usize,
    ) -> bool {
        self.is_duplicate(DuplicateOccurrence::FeatureModuleName {
            kind,
            feature,
            module,
        })
    }

    pub(in crate::plugin::runtime_plugin) fn feature_module_capability_is_duplicate(
        &self,
        kind: EmbeddedFeatureKind,
        feature: usize,
        module: usize,
        capability: usize,
    ) -> bool {
        self.is_duplicate(DuplicateOccurrence::FeatureModuleCapability {
            kind,
            feature,
            module,
            capability,
        })
    }

    pub(in crate::plugin::runtime_plugin) fn dependency_interface_is_duplicate(
        &self,
        dependency: usize,
        interface: usize,
    ) -> bool {
        self.is_duplicate(DuplicateOccurrence::DependencyInterface {
            dependency,
            interface,
        })
    }

    pub(in crate::plugin::runtime_plugin) fn provided_interface_is_duplicate(
        &self,
        interface: usize,
    ) -> bool {
        self.is_duplicate(DuplicateOccurrence::ProvidedInterface(interface))
    }

    pub(in crate::plugin::runtime_plugin) fn provided_method_name_is_duplicate(
        &self,
        interface: usize,
        method: usize,
    ) -> bool {
        self.is_duplicate(DuplicateOccurrence::ProvidedMethodName { interface, method })
    }

    pub(in crate::plugin::runtime_plugin) fn provided_method_slot_is_duplicate(
        &self,
        interface: usize,
        method: usize,
    ) -> bool {
        self.is_duplicate(DuplicateOccurrence::ProvidedMethodSlot { interface, method })
    }

    pub(in crate::plugin::runtime_plugin) fn provided_method_capability_is_duplicate(
        &self,
        interface: usize,
        method: usize,
        capability: usize,
    ) -> bool {
        self.is_duplicate(DuplicateOccurrence::ProvidedMethodCapability {
            interface,
            method,
            capability,
        })
    }

    pub(in crate::plugin::runtime_plugin) fn package_module_name_is_duplicate(
        &self,
        module: usize,
    ) -> bool {
        self.is_duplicate(DuplicateOccurrence::PackageModuleName(module))
    }

    pub(in crate::plugin::runtime_plugin) fn package_module_capability_is_duplicate(
        &self,
        module: usize,
        capability: usize,
    ) -> bool {
        self.is_duplicate(DuplicateOccurrence::PackageModuleCapability { module, capability })
    }

    pub(in crate::plugin::runtime_plugin) fn package_module_system_set_is_duplicate(
        &self,
        module: usize,
        system_set: usize,
    ) -> bool {
        self.is_duplicate(DuplicateOccurrence::PackageModuleSystemSet { module, system_set })
    }

    pub(in crate::plugin::runtime_plugin) fn package_module_system_anchor_is_duplicate(
        &self,
        module: usize,
        system_anchor: usize,
    ) -> bool {
        self.is_duplicate(DuplicateOccurrence::PackageModuleSystemAnchor {
            module,
            system_anchor,
        })
    }

    /// 能力状态的声明归属查询，范围是包及其可选特性；不代表特性已注册或能力已可用。
    pub(in crate::plugin::runtime_plugin) fn owns_capability(&self, capability: &str) -> bool {
        self.probe();
        self.owned_capabilities.contains(capability)
    }

    /// 按清单顺序保留运行时模块行，供核对需要稳定顺序的声明；成员判断另用集合视图。
    pub(in crate::plugin::runtime_plugin) fn runtime_module_names(
        &self,
    ) -> impl Iterator<Item = &str> + '_ {
        self.runtime_module_names.iter().copied()
    }

    pub(in crate::plugin::runtime_plugin) fn is_runtime_module(&self, module_name: &str) -> bool {
        self.probe();
        self.runtime_module_name_membership.contains(module_name)
    }

    /// 注册核对按清单顺序报告缺少的导出；重复声明仍保留，以维持逐声明诊断。
    pub(in crate::plugin::runtime_plugin) fn provided_interface_ids(
        &self,
    ) -> impl Iterator<Item = &str> + '_ {
        self.provided_interface_ids.iter().copied()
    }

    /// 按依赖及其接口数组的顺序核对导入，不把不同依赖中的同名接口在有序视图里折叠。
    pub(in crate::plugin::runtime_plugin) fn dependency_interface_ids(
        &self,
    ) -> impl Iterator<Item = &str> + '_ {
        self.dependency_interface_ids.iter().copied()
    }

    pub(in crate::plugin::runtime_plugin) fn declares_provided_interface(
        &self,
        interface_id: &str,
    ) -> bool {
        self.probe();
        self.provided_interface_membership.contains(interface_id)
    }

    pub(in crate::plugin::runtime_plugin) fn declares_dependency_interface(
        &self,
        interface_id: &str,
    ) -> bool {
        self.probe();
        self.dependency_interface_membership.contains(interface_id)
    }

    /// 系统锚点归属运行时模块；注册核对必须同时匹配模块名称和锚点名称。
    pub(in crate::plugin::runtime_plugin) fn runtime_system_anchors(
        &self,
    ) -> impl Iterator<Item = (&str, &str)> + '_ {
        self.runtime_system_anchors.iter().copied()
    }

    fn is_duplicate(&self, occurrence: DuplicateOccurrence) -> bool {
        self.probe();
        self.duplicates.contains(&occurrence)
    }

    fn probe(&self) {
        self.membership_probes
            .set(self.membership_probes.get().saturating_add(1));
    }
}
