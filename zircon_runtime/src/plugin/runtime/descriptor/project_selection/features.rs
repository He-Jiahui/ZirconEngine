use crate::core::framework::project::{ExportPackagingStrategy, ProjectPluginFeatureSelection};
use crate::plugin::{PluginFeatureBundleManifest, PluginModuleKind};

// 将可选功能的发布元数据转成项目配置初值；目录完成阶段会再补齐提供者与目标约束。
pub(super) fn project_feature_selection(
    feature: &PluginFeatureBundleManifest,
) -> ProjectPluginFeatureSelection {
    // TODO: [CR-PLUGIN-VALIDATION-0001] 确认公开的项目选择投影是否必须经过目录完成步骤再消费 feature 目标模式；此处保留空目标而另一 feature 投影会汇总模块目标，缺少直接调用的限制契约；下一步补充仅客户端功能的直接投影与目录完成对照测试。
    let mut selection =
        ProjectPluginFeatureSelection::new(feature.id.clone()).enabled(feature.enabled_by_default);
    selection.packaging = feature_project_selection_packaging(feature);
    selection.provider_package_id = feature.provider_package_id.clone();
    assign_feature_module_crates(feature, &mut selection);
    selection
}

// 优先选择可嵌入库，保持内建功能的默认导出方式；空清单的回退仅用于形成诊断前的投影。
fn feature_project_selection_packaging(
    feature: &PluginFeatureBundleManifest,
) -> ExportPackagingStrategy {
    feature
        .default_packaging
        .iter()
        .copied()
        .find(|packaging| *packaging == ExportPackagingStrategy::LibraryEmbed)
        .or_else(|| feature.default_packaging.first().copied())
        .unwrap_or(ExportPackagingStrategy::LibraryEmbed)
}

// 项目选择只存每种模块类型的首个 crate；多模块声明仍保留在功能清单供注册校验与构建计划使用。
fn assign_feature_module_crates(
    feature: &PluginFeatureBundleManifest,
    selection: &mut ProjectPluginFeatureSelection,
) {
    for module in &feature.modules {
        match module.kind {
            PluginModuleKind::Runtime if selection.runtime_crate.is_none() => {
                selection.runtime_crate = Some(module.crate_name.clone());
            }
            PluginModuleKind::Editor if selection.editor_crate.is_none() => {
                selection.editor_crate = Some(module.crate_name.clone());
            }
            _ => {}
        }
    }
}
