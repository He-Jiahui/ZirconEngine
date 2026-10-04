//! 将构建脚本从统一 TOML 生成的装配预设转换为运行时描述符。
//! 按 ID 查询只构造目标 profile，避免启动时复制整个预设表。

use crate::builtin::{BuiltinRuntimeModuleId, RuntimePluginId};
use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::RuntimeProfileId;
use crate::plugin::PluginMaturity;

use super::descriptor::{RuntimeProfileDescriptor, RuntimeProfilePluginSelection};

#[derive(Clone, Copy)]
struct RuntimeProfileAssemblyPluginPreset {
    id: &'static str,
    required: bool,
}

/// 生成表中的静态装配意图；字段在转换时才成为可编辑的描述符值。
struct RuntimeProfileAssemblyPreset {
    id: RuntimeProfileId,
    descriptor_name: &'static str,
    target_mode: RuntimeTargetMode,
    builtin_modules: &'static [BuiltinRuntimeModuleId],
    default_plugins: &'static [RuntimeProfileAssemblyPluginPreset],
    optional_plugins: &'static [&'static str],
    required_capabilities: &'static [&'static str],
    minimum_maturity: PluginMaturity,
    allow_externalized_required_plugins: bool,
}

include!(concat!(
    env!("OUT_DIR"),
    "/runtime_profile_assembly_presets_generated.rs"
));

impl RuntimeProfileAssemblyPreset {
    /// 供启动与导出取得独立拥有的 profile 值，避免把生成表生命周期传给消费者。
    fn descriptor(&self) -> RuntimeProfileDescriptor {
        RuntimeProfileDescriptor {
            id: self.id,
            name: self.descriptor_name.to_owned(),
            target_mode: self.target_mode,
            builtin_modules: self.builtin_modules.to_vec(),
            default_plugins: self
                .default_plugins
                .iter()
                .map(|plugin| {
                    RuntimeProfilePluginSelection::new(
                        RuntimePluginId::from_static(plugin.id),
                        plugin.required,
                    )
                })
                .collect(),
            optional_plugins: self
                .optional_plugins
                .iter()
                .map(|id| RuntimePluginId::from_static(id))
                .collect(),
            required_capabilities: self
                .required_capabilities
                .iter()
                .map(|capability| (*capability).to_owned())
                .collect(),
            minimum_maturity: self.minimum_maturity,
            allow_externalized_required_plugins: self.allow_externalized_required_plugins,
        }
    }
}

impl RuntimeProfileDescriptor {
    /// 启动或导出已明确选择 ID 时，仅展开该配置对应的装配意图。
    pub fn for_id(id: RuntimeProfileId) -> Self {
        generated_runtime_profile_assembly_preset_for(id).descriptor()
    }

    /// 供完整 profile 列表与一致性测试使用；普通单 ID 路径应调用按 ID 查询。
    pub fn builtin_profiles() -> Vec<Self> {
        RUNTIME_PROFILE_ASSEMBLY_PRESETS
            .iter()
            .map(RuntimeProfileAssemblyPreset::descriptor)
            .collect()
    }
}
