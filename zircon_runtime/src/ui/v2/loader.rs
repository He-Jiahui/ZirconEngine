//! 区分通用 V2 TOML 与产品 .zui 资源的入口约束，后续编译器再校验图和事件语义。

use std::path::Path;

use zircon_runtime_interface::ui::v2::{
    UiV2AssetDocument, UiV2AssetError, UiV2AssetKind, UI_V2_ASSET_SCHEMA_VERSION,
};

/// 接收通用版本化文档，仅解析结构并校验 schema 版本；不会解析导入或创建运行时节点。
#[derive(Default)]
pub struct UiV2AssetLoader;

/// 文件缓存的产品资源入口，额外约束 view/component/style 各类文档的根节点形态。
#[derive(Default)]
pub struct UiZuiAssetLoader;

impl UiV2AssetLoader {
    pub fn load_toml_str(input: &str) -> Result<UiV2AssetDocument, UiV2AssetError> {
        let document: UiV2AssetDocument =
            toml::from_str(input).map_err(|error| UiV2AssetError::ParseToml(error.to_string()))?;
        validate_version(document)
    }

    /// 同步读取 UTF-8 文件，适用于资源加载阶段；实时输入和渲染路径应复用已加载文档。
    pub fn load_toml_file<P: AsRef<Path>>(path: P) -> Result<UiV2AssetDocument, UiV2AssetError> {
        let path = path.as_ref();
        let input = std::fs::read_to_string(path)
            .map_err(|error| UiV2AssetError::Io(format!("{}: {error}", path.display())))?;
        Self::load_toml_str(&input)
    }
}

impl UiZuiAssetLoader {
    /// 先完成通用版本检查，再约束资源种类；导入存在性和组件事件由后续仓库与编译器处理。
    pub fn load_zui_str(input: &str) -> Result<UiV2AssetDocument, UiV2AssetError> {
        let document = UiV2AssetLoader::load_toml_str(input)?;
        validate_zui_document_profile(&document)?;
        Ok(document)
    }

    pub fn load_zui_file<P: AsRef<Path>>(path: P) -> Result<UiV2AssetDocument, UiV2AssetError> {
        let path = path.as_ref();
        let input = std::fs::read_to_string(path)
            .map_err(|error| UiV2AssetError::Io(format!("{}: {error}", path.display())))?;
        Self::load_zui_str(&input)
    }
}

fn validate_version(document: UiV2AssetDocument) -> Result<UiV2AssetDocument, UiV2AssetError> {
    if document.asset.version != UI_V2_ASSET_SCHEMA_VERSION {
        return Err(unsupported_schema_error(
            document.asset.id,
            document.asset.version,
        ));
    }
    Ok(document)
}

fn unsupported_schema_error(asset_id: String, version: u32) -> UiV2AssetError {
    UiV2AssetError::UnsupportedSchemaVersion {
        asset_id,
        version,
        expected: UI_V2_ASSET_SCHEMA_VERSION,
    }
}

fn validate_zui_document_profile(document: &UiV2AssetDocument) -> Result<(), UiV2AssetError> {
    match document.asset.kind {
        UiV2AssetKind::Component => validate_zui_component_profile(document),
        UiV2AssetKind::View => validate_zui_view_profile(document),
        UiV2AssetKind::Style | UiV2AssetKind::ThemeTokens => validate_zui_style_profile(document),
    }
}

fn validate_zui_component_profile(document: &UiV2AssetDocument) -> Result<(), UiV2AssetError> {
    let asset_id = document.asset.id.clone();
    if document.root.is_some() {
        return Err(UiV2AssetError::InvalidDocument {
            asset_id,
            detail: ".zui component assets must not declare a [root] view entry".to_string(),
        });
    }
    if document.components.len() != 1 {
        return Err(UiV2AssetError::InvalidDocument {
            asset_id,
            detail: format!(
                ".zui component assets must declare exactly one component; found {}",
                document.components.len()
            ),
        });
    }

    let (component_id, component) = document
        .components
        .iter()
        .next()
        .expect("zui component count validated above");
    if component.root.trim().is_empty() {
        return Err(UiV2AssetError::InvalidDocument {
            asset_id,
            detail: format!(".zui component {component_id} must declare a non-empty root node"),
        });
    }
    if !document.nodes.contains_key(&component.root) {
        return Err(UiV2AssetError::MissingNode {
            asset_id,
            node_id: component.root.clone(),
        });
    }
    Ok(())
}

fn validate_zui_view_profile(document: &UiV2AssetDocument) -> Result<(), UiV2AssetError> {
    let asset_id = document.asset.id.clone();
    let root = document
        .root
        .as_ref()
        .ok_or_else(|| UiV2AssetError::InvalidDocument {
            asset_id: asset_id.clone(),
            detail: ".zui view assets must declare a [root] view entry".to_string(),
        })?;
    if root.node.trim().is_empty() {
        return Err(UiV2AssetError::InvalidDocument {
            asset_id,
            detail: ".zui view assets must declare a non-empty root node".to_string(),
        });
    }
    if !document.nodes.contains_key(&root.node) {
        return Err(UiV2AssetError::MissingNode {
            asset_id,
            node_id: root.node.clone(),
        });
    }
    Ok(())
}

fn validate_zui_style_profile(document: &UiV2AssetDocument) -> Result<(), UiV2AssetError> {
    if let Some(root) = &document.root {
        let asset_id = document.asset.id.clone();
        if root.node.trim().is_empty() {
            return Err(UiV2AssetError::InvalidDocument {
                asset_id,
                detail:
                    ".zui style assets must declare a non-empty root node when [root] is present"
                        .to_string(),
            });
        }
        if !document.nodes.contains_key(&root.node) {
            return Err(UiV2AssetError::MissingNode {
                asset_id,
                node_id: root.node.clone(),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "loader/tests/owned_schema_error_tests.rs"]
mod owned_schema_error_tests;
