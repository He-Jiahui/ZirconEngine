//! 按项目选择顺序将每个插件归入一个主类别，并另建必需缺失索引。
//! 成熟度与目标门槛先于包提供者匹配，避免注册证据绕开描述符限制。

use std::collections::HashMap;

use crate::builtin::RuntimePluginId;
use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::ProjectPluginManifest;
use crate::plugin::{
    PluginMaturity, RuntimePluginAvailabilityCategory, RuntimePluginAvailabilityReport,
};

use super::generation::{
    row_from_descriptor, row_from_runtime, RuntimePluginAvailabilityDescriptorRef,
    RuntimePluginAvailabilityGenerationBuilder, RuntimePluginAvailabilityReason,
};
use super::selection::{merge_runtime_plugin_selection, project_manifest_plugin_selections};
#[cfg(test)]
use super::RuntimePluginAvailabilitySelectionMetrics;
use super::{
    RuntimePluginAvailabilityGeneration, RuntimePluginAvailabilityProjection,
    RuntimeProfileDescriptor,
};

impl<'descriptor, 'provider> RuntimePluginAvailabilityProjection<'descriptor, 'provider> {
    pub fn report_for_profile_defaults(
        &self,
        profile: &RuntimeProfileDescriptor,
        require_external_provider: bool,
    ) -> RuntimePluginAvailabilityReport {
        self.generation_for_profile_defaults(profile, require_external_provider)
            .materialize_report()
    }

    /// 默认入口合并默认和可选项；重复 ID 保留首次顺序，必需标志取并集。
    pub fn generation_for_profile_defaults(
        &self,
        profile: &RuntimeProfileDescriptor,
        require_external_provider: bool,
    ) -> RuntimePluginAvailabilityGeneration<'descriptor> {
        let mut plugins = Vec::with_capacity(
            profile
                .default_plugins
                .len()
                .saturating_add(profile.optional_plugins.len()),
        );
        let mut positions = HashMap::with_capacity(plugins.capacity());
        for (plugin_id, required) in profile
            .default_plugins
            .iter()
            .map(|plugin| (plugin.id.clone(), plugin.required))
            .chain(
                profile
                    .optional_plugins
                    .iter()
                    .cloned()
                    .map(|plugin_id| (plugin_id, false)),
            )
        {
            merge_runtime_plugin_selection(&mut plugins, &mut positions, plugin_id, required);
        }
        self.generation_for_runtime_plugins(profile, plugins, require_external_provider)
    }

    pub fn report_for_manifest(
        &self,
        profile: &RuntimeProfileDescriptor,
        manifest: &ProjectPluginManifest,
        require_external_provider: bool,
    ) -> RuntimePluginAvailabilityReport {
        self.generation_for_manifest(profile, manifest, require_external_provider)
            .materialize_report()
    }

    /// 显式项目清单只纳入当前目标启用且可解析的选择，不回填默认项。
    pub fn generation_for_manifest(
        &self,
        profile: &RuntimeProfileDescriptor,
        manifest: &ProjectPluginManifest,
        require_external_provider: bool,
    ) -> RuntimePluginAvailabilityGeneration<'descriptor> {
        let selection_projection = project_manifest_plugin_selections(profile, manifest);
        self.generation_for_runtime_plugins(
            profile,
            selection_projection.plugins,
            require_external_provider,
        )
    }

    #[cfg(test)]
    pub fn report_for_manifest_with_metrics(
        &self,
        profile: &RuntimeProfileDescriptor,
        manifest: &ProjectPluginManifest,
        require_external_provider: bool,
    ) -> (
        RuntimePluginAvailabilityReport,
        RuntimePluginAvailabilitySelectionMetrics,
    ) {
        let selection_projection = project_manifest_plugin_selections(profile, manifest);
        (
            self.generation_for_runtime_plugins(
                profile,
                selection_projection.plugins,
                require_external_provider,
            )
            .materialize_report(),
            selection_projection.metrics,
        )
    }

    fn generation_for_runtime_plugins(
        &self,
        profile: &RuntimeProfileDescriptor,
        plugins: impl IntoIterator<Item = (RuntimePluginId, bool)>,
        require_external_provider: bool,
    ) -> RuntimePluginAvailabilityGeneration<'descriptor> {
        let mut generation = RuntimePluginAvailabilityGenerationBuilder::new();
        for (plugin_id, required) in plugins {
            self.append_plugin_availability(
                profile,
                plugin_id,
                required,
                require_external_provider,
                &mut generation,
            );
        }
        generation.finish()
    }

    // 分类顺序决定对外诊断的首要原因，随后由装配报告把必需缺口升级为失败。
    fn append_plugin_availability(
        &self,
        profile: &RuntimeProfileDescriptor,
        plugin_id: RuntimePluginId,
        required: bool,
        require_external_provider: bool,
        generation: &mut RuntimePluginAvailabilityGenerationBuilder<'descriptor>,
    ) {
        let Some(descriptor) = self.descriptors.get(&plugin_id) else {
            if plugin_id == RuntimePluginId::Ui && !cfg!(feature = "ui") {
                generation.push(
                    required,
                    row_from_runtime(
                        plugin_id,
                        required,
                        PluginMaturity::Core,
                        RuntimePluginAvailabilityCategory::Stub,
                        RuntimePluginAvailabilityReason::BuiltinUnavailable,
                    ),
                );
                return;
            }
            if plugin_id == RuntimePluginId::Ui && cfg!(feature = "ui") {
                generation.push(
                    false,
                    row_from_runtime(
                        plugin_id,
                        required,
                        PluginMaturity::Core,
                        RuntimePluginAvailabilityCategory::Available,
                        RuntimePluginAvailabilityReason::BuiltinAvailable,
                    ),
                );
                return;
            }
            generation.push(
                required,
                row_from_runtime(
                    plugin_id,
                    required,
                    PluginMaturity::Stub,
                    RuntimePluginAvailabilityCategory::Stub,
                    RuntimePluginAvailabilityReason::MissingCatalog,
                ),
            );
            return;
        };
        if !supports_target(descriptor, profile.target_mode) {
            generation.push(
                required,
                row_from_descriptor(
                    descriptor,
                    required,
                    RuntimePluginAvailabilityCategory::BlockedByTarget,
                    RuntimePluginAvailabilityReason::TargetUnsupported(profile.target_mode),
                ),
            );
            return;
        }
        // 外置和 stub 成熟度优先于提供者匹配；提供者本身不提升目录成熟度。
        if descriptor.maturity == PluginMaturity::Externalized {
            generation.push(
                !profile.allow_externalized_required_plugins && required,
                row_from_descriptor(
                    descriptor,
                    required,
                    RuntimePluginAvailabilityCategory::ExternalizedMissing,
                    RuntimePluginAvailabilityReason::Externalized,
                ),
            );
            return;
        }
        if descriptor.maturity == PluginMaturity::Stub {
            generation.push(
                required,
                row_from_descriptor(
                    descriptor,
                    required,
                    RuntimePluginAvailabilityCategory::Stub,
                    RuntimePluginAvailabilityReason::Stub,
                ),
            );
            return;
        }
        // BUG: [CR-PLUGIN-BOUNDARY-0202] 此分类链漏判已废弃状态；最低门槛为 Stub 或 Deprecated 时，
        // Deprecated 可越过排位比较进入已链接或可用分类；等级比较本身符合其排序契约。
        // 证据：前两项特殊状态早退、plugin_maturity 的不可用状态集合及后续提供者分支。
        if !descriptor.maturity.meets_minimum(profile.minimum_maturity) {
            generation.push(
                required,
                row_from_descriptor(
                    descriptor,
                    required,
                    RuntimePluginAvailabilityCategory::BlockedByMaturity,
                    RuntimePluginAvailabilityReason::BelowMinimum(profile.minimum_maturity),
                ),
            );
            return;
        }
        // 提供者必须用包身份匹配；runtime ID 或项目别名不能代替已注册包。
        if self.linked_plugin_ids.contains(descriptor.package_id) {
            generation.push(
                false,
                row_from_descriptor(
                    descriptor,
                    required,
                    RuntimePluginAvailabilityCategory::Linked,
                    RuntimePluginAvailabilityReason::Linked,
                ),
            );
            return;
        }
        if self
            .native_dynamic_plugin_ids
            .contains(descriptor.package_id)
        {
            generation.push(
                false,
                row_from_descriptor(
                    descriptor,
                    required,
                    RuntimePluginAvailabilityCategory::NativeDynamic,
                    RuntimePluginAvailabilityReason::NativeDynamic,
                ),
            );
            return;
        }
        if require_external_provider && !builtin_runtime_domain_is_available(&descriptor.runtime_id)
        {
            generation.push(
                !profile.allow_externalized_required_plugins && required,
                row_from_descriptor(
                    descriptor,
                    required,
                    RuntimePluginAvailabilityCategory::ExternalizedMissing,
                    RuntimePluginAvailabilityReason::MissingProvider,
                ),
            );
            return;
        }
        generation.push(
            false,
            row_from_descriptor(
                descriptor,
                required,
                RuntimePluginAvailabilityCategory::Available,
                RuntimePluginAvailabilityReason::Available,
            ),
        );
    }
}

fn builtin_runtime_domain_is_available(id: &RuntimePluginId) -> bool {
    id == &RuntimePluginId::Ui && cfg!(feature = "ui")
}

fn supports_target(
    descriptor: &RuntimePluginAvailabilityDescriptorRef<'_>,
    target: RuntimeTargetMode,
) -> bool {
    descriptor.target_modes.is_empty() || descriptor.target_modes.contains(&target)
}
