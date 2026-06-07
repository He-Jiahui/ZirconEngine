use crate::assets::{AssetCatalogEntry, PROJECT_ASSET_SOURCE, SELECTED_PROJECT_ASSET_SOURCE};
use crate::learn::{LearnCatalogEntry, SOURCE_ENGINE_LEARN_SOURCE};
use crate::plugins::{PluginCatalogEntry, ENGINE_PLUGIN_SCOPE, PROJECT_PLUGIN_SCOPE};
use crate::settings::HubLanguage;
use crate::state::HubSnapshot;

use super::display::{format_bytes, path_text_en};
use super::{HubAssetItem, HubLearnItem, HubPluginItem, HubTextBundle};

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

fn plugin_maturity_tone(maturity: &str) -> &'static str {
    let normalized = maturity.trim().to_ascii_lowercase();
    if normalized.contains("stable") || maturity.contains("稳定") {
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

fn catalog_category_key(category: &str) -> &'static str {
    let normalized = category.trim().to_ascii_lowercase();
    if normalized.contains("guide") || category.contains("指南") {
        "guide"
    } else if normalized.contains("reference") || category.contains("参考") {
        "reference"
    } else if normalized.contains("workflow") || category.contains("工作流") {
        "workflow"
    } else if normalized.contains("documentation") || category.contains("文档") {
        "documentation"
    } else {
        "local"
    }
}

#[cfg(test)]
mod tests {
    use crate::learn::SELECTED_PROJECT_LEARN_SOURCE;

    use super::*;

    #[test]
    fn catalog_scope_key_stays_stable_across_localized_display_copy() {
        assert_eq!(catalog_scope_key(PROJECT_ASSET_SOURCE), "project");
        assert_eq!(catalog_scope_key(SELECTED_PROJECT_LEARN_SOURCE), "project");
        assert_eq!(catalog_scope_key(ENGINE_PLUGIN_SCOPE), "engine");
        assert_eq!(catalog_scope_key(SOURCE_ENGINE_LEARN_SOURCE), "engine");
        assert_eq!(catalog_scope_key("项目"), "project");
        assert_eq!(catalog_scope_key("源码引擎"), "engine");
    }

    #[test]
    fn catalog_scope_key_maps_engine_asset_roots_to_engine_filter_key() {
        assert_eq!(catalog_scope_key("Editor"), "engine");
        assert_eq!(catalog_scope_key("Runtime"), "engine");
    }

    #[test]
    fn catalog_category_key_stays_stable_across_localized_learn_copy() {
        assert_eq!(catalog_category_key("guide"), "guide");
        assert_eq!(catalog_category_key("Reference"), "reference");
        assert_eq!(catalog_category_key("指南"), "guide");
        assert_eq!(catalog_category_key("参考"), "reference");
    }

    #[test]
    fn catalog_scope_display_uses_current_language() {
        assert_eq!(
            localized_catalog_scope(SELECTED_PROJECT_ASSET_SOURCE, HubLanguage::Chinese),
            "已选项目"
        );
        assert_eq!(
            localized_catalog_scope(SOURCE_ENGINE_LEARN_SOURCE, HubLanguage::Chinese),
            "源码引擎"
        );
        assert_eq!(
            localized_catalog_scope(PROJECT_PLUGIN_SCOPE, HubLanguage::English),
            "Project"
        );
    }

    #[test]
    fn asset_detail_punctuation_is_localized_before_react_renders_it() {
        assert_eq!(
            asset_detail("Material", "E:\\Assets\\Hero", HubLanguage::English),
            "Material - E:\\Assets\\Hero"
        );
        assert_eq!(
            asset_detail("材质", "E:\\Assets\\Hero", HubLanguage::Chinese),
            "材质：E:\\Assets\\Hero"
        );
    }

    #[test]
    fn plugin_maturity_tone_does_not_parse_only_english_display_copy() {
        assert_eq!(plugin_maturity_tone("stable"), "success");
        assert_eq!(plugin_maturity_tone("稳定"), "success");
        assert_eq!(plugin_maturity_tone("preview"), "warning");
    }
}
