//! 把已扫描的资产、插件和学习资源转为界面目录项；本模块不发起扫描。
//! 显示文案可本地化，过滤键保持固定语义，以免切换语言改变目录筛选结果。

use crate::assets::{AssetCatalogEntry, PROJECT_ASSET_SOURCE, SELECTED_PROJECT_ASSET_SOURCE};
use crate::learn::{LearnCatalogEntry, SOURCE_ENGINE_LEARN_SOURCE};
use crate::plugins::{PluginCatalogEntry, ENGINE_PLUGIN_SCOPE, PROJECT_PLUGIN_SCOPE};
use crate::settings::HubLanguage;
use crate::state::HubSnapshot;

use super::display::{format_bytes, path_text_en};
use super::{HubAssetItem, HubLearnItem, HubPluginItem, HubTextBundle};

/// 投影会话中已发现的资产目录；路径和稳定筛选键不随界面语言变化。
pub(super) fn asset_rows(snapshot: &HubSnapshot) -> Vec<HubAssetItem> {
    snapshot
        .assets
        .iter()
        .map(|asset| asset_row(asset, snapshot.settings.language))
        .collect()
}

fn asset_row(asset: &AssetCatalogEntry, language: HubLanguage) -> HubAssetItem {
    let path = path_text_en(&asset.path);
    HubAssetItem {
        id: path.clone(),
        name: asset.name.clone(),
        kind: asset.kind.clone(),
        detail: asset_detail(&asset.kind, &path, language),
        source: localized_catalog_scope(&asset.source, language),
        source_key: catalog_scope_key(&asset.source).to_string(),
        size: format_bytes(asset.size_bytes),
        path,
    }
}

/// 向只读插件目录投影清单信息；成熟度颜色属于提示，不授予安装或激活能力。
pub(super) fn plugin_rows(snapshot: &HubSnapshot) -> Vec<HubPluginItem> {
    snapshot
        .plugins
        .iter()
        .map(|plugin| plugin_row(plugin, snapshot.settings.language))
        .collect()
}

fn plugin_row(plugin: &PluginCatalogEntry, language: HubLanguage) -> HubPluginItem {
    HubPluginItem {
        id: plugin.id.clone(),
        display_name: plugin.display_name.clone(),
        description: plugin.description.clone(),
        category: plugin.category.clone(),
        maturity: plugin.maturity.clone(),
        maturity_tone: plugin_maturity_tone(&plugin.maturity).to_string(),
        scope: localized_catalog_scope(&plugin.scope, language),
        scope_key: catalog_scope_key(&plugin.scope).to_string(),
        editor_scoped: plugin.editor_scoped,
        module_count: plugin.module_count,
        default_packaging: plugin.default_packaging.clone(),
        package_root: path_text_en(&plugin.package_root),
        manifest_path: path_text_en(&plugin.manifest_path),
    }
}

/// 投影会话当前范围中的学习资源，保持路径身份供打开资源动作再次校验。
pub(super) fn learn_rows(snapshot: &HubSnapshot) -> Vec<HubLearnItem> {
    snapshot
        .learn_resources
        .iter()
        .map(|resource| learn_row(resource, snapshot.settings.language))
        .collect()
}

fn learn_row(resource: &LearnCatalogEntry, language: HubLanguage) -> HubLearnItem {
    let path = path_text_en(&resource.path);
    HubLearnItem {
        id: path.clone(),
        title: resource.title.clone(),
        category: resource.category.clone(),
        category_key: catalog_category_key(&resource.category).to_string(),
        source: localized_catalog_scope(&resource.source, language),
        source_key: catalog_scope_key(&resource.source).to_string(),
        summary: resource.summary.clone(),
        path,
    }
}

// BUG: [CR-HUBSTATE-0004] 清单允许任意成熟度字符串，“unstable”也包含稳定关键词而被标为成功色；证据：清单读取未限制枚举值且这里做子串匹配。
fn plugin_maturity_tone(maturity: &str) -> &'static str {
    if contains_ascii_case_insensitive(maturity, b"stable") || maturity.contains("稳定") {
        "success"
    } else {
        "warning"
    }
}

fn asset_detail(kind: &str, path: &str, language: HubLanguage) -> String {
    match language {
        HubLanguage::English => format!("{kind} - {path}"),
        HubLanguage::Chinese => format!("{kind}：{path}"),
    }
}

fn localized_catalog_scope(scope: &str, language: HubLanguage) -> String {
    let text = HubTextBundle::new(language);
    if scope == SELECTED_PROJECT_ASSET_SOURCE {
        text.pair("Selected Project", "已选项目").to_string()
    } else if scope == PROJECT_ASSET_SOURCE || scope == PROJECT_PLUGIN_SCOPE {
        text.pair("Project", "项目").to_string()
    } else if scope == ENGINE_PLUGIN_SCOPE {
        text.pair("Engine", "引擎").to_string()
    } else if scope == SOURCE_ENGINE_LEARN_SOURCE {
        text.pair("Source Engine", "源码引擎").to_string()
    } else if scope == "Documentation" {
        text.pair("Documentation", "文档").to_string()
    } else {
        scope.to_string()
    }
}

// Web 依据固定键筛选来源，显示标签的语言和旧目录来源文案不能改变其分类。
fn catalog_scope_key(scope: &str) -> &'static str {
    if scope == SELECTED_PROJECT_ASSET_SOURCE
        || scope == PROJECT_ASSET_SOURCE
        || scope == PROJECT_PLUGIN_SCOPE
        || scope == "项目"
        || scope == "已选项目"
    {
        "project"
    } else if scope == ENGINE_PLUGIN_SCOPE
        || scope == SOURCE_ENGINE_LEARN_SOURCE
        || scope == "Editor"
        || scope == "Runtime"
        || scope == "引擎"
        || scope == "源码引擎"
    {
        "engine"
    } else if scope == "Documentation" || scope == "Docs" || scope == "文档" {
        "documentation"
    } else {
        "local"
    }
}

// 类别键服务学习资源筛选；无法识别的自定义类别归为本地，其原文仍用于显示。
fn catalog_category_key(category: &str) -> &'static str {
    if contains_ascii_case_insensitive(category, b"guide") || category.contains("指南") {
        "guide"
    } else if contains_ascii_case_insensitive(category, b"reference") || category.contains("参考")
    {
        "reference"
    } else if contains_ascii_case_insensitive(category, b"workflow") || category.contains("工作流")
    {
        "workflow"
    } else if contains_ascii_case_insensitive(category, b"documentation")
        || category.contains("文档")
    {
        "documentation"
    } else {
        "local"
    }
}

// 这些分类只需要识别 ASCII 关键词；中文分支另行匹配，避免为每行创建小写副本。
fn contains_ascii_case_insensitive(value: &str, needle: &[u8]) -> bool {
    !needle.is_empty()
        && value
            .as_bytes()
            .windows(needle.len())
            .any(|window| window.eq_ignore_ascii_case(needle))
}

#[cfg(test)]
#[path = "tests/catalog.rs"]
mod tests;
