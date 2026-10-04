//! 把 profile、项目选择、目录描述和提供者证据汇成可用性视图。
//! 报告用于组合失败诊断与导出展示；调用方仍须单独完成注册与装载裁决。

use std::collections::HashSet;

use crate::core::framework::project::ProjectPluginManifest;
use crate::plugin::{
    RuntimePluginCatalog, RuntimePluginDescriptor, RuntimePluginRegistrationReport,
};

use super::availability_projection::{
    RuntimePluginAvailabilityGeneration, RuntimePluginAvailabilityProjection,
};
#[cfg(test)]
use super::availability_projection::{
    RuntimePluginAvailabilityProjectionMetrics, RuntimePluginAvailabilitySelectionMetrics,
};
use super::availability_report::RuntimePluginAvailabilityReport;
use super::descriptor::RuntimeProfileDescriptor;

impl RuntimeProfileDescriptor {
    /// 供仅持有描述符与 linked ID 的调用方查看 profile 默认项；不要求外部提供者。
    pub fn availability_report<'a>(
        &self,
        descriptors: impl IntoIterator<Item = &'a RuntimePluginDescriptor>,
        linked_plugin_ids: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> RuntimePluginAvailabilityReport {
        RuntimePluginAvailabilityProjection::new(
            descriptors,
            linked_plugin_ids,
            std::iter::empty::<String>(),
        )
        .report_for_profile_defaults(self, false)
    }

    /// 启动按默认选择检查时，同时区分 linked 与 native dynamic 提供者。
    pub fn availability_report_with_providers<'a>(
        &self,
        descriptors: impl IntoIterator<Item = &'a RuntimePluginDescriptor>,
        linked_plugin_ids: impl IntoIterator<Item = impl AsRef<str>>,
        native_dynamic_plugin_ids: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> RuntimePluginAvailabilityReport {
        RuntimePluginAvailabilityProjection::new(
            descriptors,
            linked_plugin_ids,
            native_dynamic_plugin_ids,
        )
        .report_for_profile_defaults(self, true)
    }

    /// 以 profile 默认清单解释注册报告；项目覆盖清单应使用显式清单入口。
    pub fn availability_report_for_registration_reports<'a, 'b>(
        &self,
        descriptors: impl IntoIterator<Item = &'a RuntimePluginDescriptor>,
        registrations: impl IntoIterator<Item = &'b RuntimePluginRegistrationReport>,
    ) -> RuntimePluginAvailabilityReport {
        self.availability_report_for_manifest_and_registration_reports(
            descriptors,
            &self.project_manifest(),
            registrations,
        )
    }

    /// 装配层以有效项目清单和注册报告产生可用于缺失必需项判断的快照。
    pub fn availability_report_for_manifest_and_registration_reports<'a, 'b>(
        &self,
        descriptors: impl IntoIterator<Item = &'a RuntimePluginDescriptor>,
        manifest: &ProjectPluginManifest,
        registrations: impl IntoIterator<Item = &'b RuntimePluginRegistrationReport>,
    ) -> RuntimePluginAvailabilityReport {
        RuntimePluginAvailabilityProjection::from_registration_reports(
            descriptors,
            registrations,
            self.target_mode,
        )
        .report_for_manifest(self, manifest, true)
    }

    /// 导出或已解析计划提供包 ID 时，按显式项目选择投影分类结果。
    pub fn availability_report_for_manifest_with_providers<'a>(
        &self,
        descriptors: impl IntoIterator<Item = &'a RuntimePluginDescriptor>,
        manifest: &ProjectPluginManifest,
        linked_plugin_ids: impl IntoIterator<Item = impl AsRef<str>>,
        native_dynamic_plugin_ids: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> RuntimePluginAvailabilityReport {
        RuntimePluginAvailabilityProjection::new(
            descriptors,
            linked_plugin_ids,
            native_dynamic_plugin_ids,
        )
        .report_for_manifest(self, manifest, true)
    }

    /// 反复查询状态时可保留此生成代；其借用的描述符必须在整个使用期保持存活。
    /// Builds one immutable availability generation for consumers that poll or
    /// render status repeatedly. Materialize a report only at an export or
    /// diagnostic boundary that needs owned, serializable rows.
    pub fn availability_generation_for_manifest_with_providers<'a>(
        &self,
        descriptors: impl IntoIterator<Item = &'a RuntimePluginDescriptor>,
        manifest: &ProjectPluginManifest,
        linked_plugin_ids: impl IntoIterator<Item = impl AsRef<str>>,
        native_dynamic_plugin_ids: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> RuntimePluginAvailabilityGeneration<'a> {
        RuntimePluginAvailabilityProjection::new(
            descriptors,
            linked_plugin_ids,
            native_dynamic_plugin_ids,
        )
        .generation_for_manifest(self, manifest, true)
    }

    /// 复用调用方已建好的 linked 集合，避免目标模块选择重复复制成员关系。
    pub(crate) fn availability_report_for_manifest_with_linked_membership<'a, 'b>(
        &self,
        descriptors: impl IntoIterator<Item = &'a RuntimePluginDescriptor>,
        manifest: &ProjectPluginManifest,
        linked_plugin_ids: &'b HashSet<String>,
    ) -> RuntimePluginAvailabilityReport {
        RuntimePluginAvailabilityProjection::from_descriptors_with_provider_membership(
            descriptors,
            linked_plugin_ids,
            std::iter::empty::<&str>(),
        )
        .report_for_manifest(self, manifest, true)
    }

    /// 导出计划借用目录注册行和提供者索引，并按 profile 默认项生成报告。
    pub(crate) fn availability_report_for_catalog_with_provider_membership<'a, 'b>(
        &self,
        catalog: &'a RuntimePluginCatalog,
        linked_plugin_ids: &'b HashSet<String>,
        native_dynamic_plugin_ids: impl IntoIterator<Item = &'b str>,
    ) -> RuntimePluginAvailabilityReport {
        RuntimePluginAvailabilityProjection::from_catalog_with_provider_membership(
            catalog,
            linked_plugin_ids,
            native_dynamic_plugin_ids,
        )
        .report_for_profile_defaults(self, true)
    }

    #[cfg(test)]
    pub(crate) fn availability_report_for_manifest_with_providers_and_metrics<'a>(
        &self,
        descriptors: impl IntoIterator<Item = &'a RuntimePluginDescriptor>,
        manifest: &ProjectPluginManifest,
        linked_plugin_ids: impl IntoIterator<Item = impl AsRef<str>>,
        native_dynamic_plugin_ids: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> (
        RuntimePluginAvailabilityReport,
        RuntimePluginAvailabilitySelectionMetrics,
    ) {
        RuntimePluginAvailabilityProjection::new(
            descriptors,
            linked_plugin_ids,
            native_dynamic_plugin_ids,
        )
        .report_for_manifest_with_metrics(self, manifest, true)
    }

    #[cfg(test)]
    pub(crate) fn availability_report_for_manifest_with_linked_membership_and_metrics<'a, 'b>(
        &self,
        descriptors: impl IntoIterator<Item = &'a RuntimePluginDescriptor>,
        manifest: &ProjectPluginManifest,
        linked_plugin_ids: &'b HashSet<String>,
    ) -> (
        RuntimePluginAvailabilityReport,
        RuntimePluginAvailabilityProjectionMetrics,
    ) {
        let projection =
            RuntimePluginAvailabilityProjection::from_descriptors_with_provider_membership(
                descriptors,
                linked_plugin_ids,
                std::iter::empty::<&str>(),
            );
        let report = projection.report_for_manifest(self, manifest, true);
        (report, projection.metrics())
    }

    #[cfg(test)]
    pub(crate) fn availability_report_for_manifest_and_registration_reports_with_metrics<'a, 'b>(
        &self,
        descriptors: impl IntoIterator<Item = &'a RuntimePluginDescriptor>,
        manifest: &ProjectPluginManifest,
        registrations: impl IntoIterator<Item = &'b RuntimePluginRegistrationReport>,
    ) -> (
        RuntimePluginAvailabilityReport,
        RuntimePluginAvailabilityProjectionMetrics,
    ) {
        let projection = RuntimePluginAvailabilityProjection::from_registration_reports(
            descriptors,
            registrations,
            self.target_mode,
        );
        let report = projection.report_for_manifest(self, manifest, true);
        (report, projection.metrics())
    }
}
