use std::collections::BTreeMap;
use std::sync::Arc;

use zircon_runtime::asset::pipeline::manager::ProjectAssetGenerationToken;
use zircon_runtime::asset::AssetUri;
use zircon_runtime_interface::resource::ResourceScheme;
use zircon_runtime_interface::ui::component::UiValue;

use crate::core::i18n::EditorI18nService;
use crate::ui::host::editor_asset_manager::EditorAssetCatalogGeneration;
use crate::ui::retained_host::callback_dispatch::WorkbenchCommandPaletteOpenState;

pub(super) const PROMPT_SOURCE: &str = "asset-name-prompt";
pub(super) const PROMPT_COMMIT_ID: &str = "asset-name-prompt-confirm";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum AssetNamePromptRequest {
    CreateFolder {
        parent: String,
    },
    RenameFolder {
        source: AssetUri,
    },
    RenameAsset {
        asset_uuid: String,
        source: AssetUri,
    },
}

impl AssetNamePromptRequest {
    pub(super) fn create_folder(parent: &str) -> Result<Self, &'static str> {
        if parent == "res://" {
            return Ok(Self::CreateFolder {
                parent: parent.to_owned(),
            });
        }
        Ok(Self::CreateFolder {
            parent: project_source(parent, false)?.to_string(),
        })
    }

    pub(super) fn rename_folder(source: &str) -> Result<Self, &'static str> {
        Ok(Self::RenameFolder {
            source: project_source(source, false)?,
        })
    }

    pub(super) fn rename_asset(asset_uuid: &str, source: &str) -> Result<Self, &'static str> {
        Ok(Self::RenameAsset {
            asset_uuid: asset_uuid.to_owned(),
            source: project_source(source, false)?,
        })
    }

    pub(super) fn source(&self) -> String {
        match self {
            Self::CreateFolder { parent } => parent.clone(),
            Self::RenameFolder { source } | Self::RenameAsset { source, .. } => source.to_string(),
        }
    }

    pub(super) fn initial_query(&self) -> &str {
        match self {
            Self::CreateFolder { .. } => "",
            Self::RenameFolder { source } | Self::RenameAsset { source, .. } => {
                source.path().rsplit('/').next().unwrap_or_default()
            }
        }
    }

    pub(super) fn title_key(&self) -> &'static str {
        match self {
            Self::CreateFolder { .. } => "asset.name_prompt.create_title",
            Self::RenameFolder { .. } => "asset.name_prompt.folder_title",
            Self::RenameAsset { .. } => "asset.name_prompt.asset_title",
        }
    }

    pub(super) fn placeholder_key(&self) -> &'static str {
        match self {
            Self::RenameAsset { .. } => "asset.name_prompt.asset_placeholder",
            _ => "asset.name_prompt.folder_placeholder",
        }
    }

    /// Entry text is one path segment. It can never change the selected parent or scheme.
    pub(super) fn target(&self, name: &str) -> Result<AssetUri, &'static str> {
        validate_name(name)?;
        if let Self::RenameAsset { source, .. } = self {
            let source_name = source.path().rsplit('/').next().unwrap_or_default();
            if !asset_extension(source_name).eq_ignore_ascii_case(asset_extension(name)) {
                return Err("asset.name_prompt.invalid_name");
            }
        }
        let parent = match self {
            Self::CreateFolder { parent } => parent.strip_prefix("res://").unwrap_or_default(),
            Self::RenameFolder { source } | Self::RenameAsset { source, .. } => source
                .path()
                .rsplit_once('/')
                .map(|(parent, _)| parent)
                .unwrap_or_default(),
        };
        let path = if parent.is_empty() {
            name.to_owned()
        } else {
            format!("{parent}/{name}")
        };
        let target = AssetUri::new(ResourceScheme::Res, path, None)
            .map_err(|_| "asset.name_prompt.invalid_name")?;
        if matches!(self, Self::RenameFolder { source } | Self::RenameAsset { source, .. } if target == *source)
        {
            return Err("asset.name_prompt.no_change");
        }
        Ok(target)
    }

    pub(super) fn source_is_current(&self, catalog: &EditorAssetCatalogGeneration) -> bool {
        match self {
            Self::CreateFolder { parent } => catalog.folder(parent).is_some(),
            Self::RenameFolder { source } => catalog.folder(&source.to_string()).is_some(),
            Self::RenameAsset { asset_uuid, source } => catalog
                .asset(asset_uuid)
                .is_some_and(|asset| asset.locator == source.to_string()),
        }
    }
}

/// Keep the importer dispatch suffix, including built-in compound document formats.
fn asset_extension(name: &str) -> &str {
    let Some((stem, extension)) = name.rsplit_once('.') else {
        return "";
    };
    if extension.eq_ignore_ascii_case("toml") || extension.eq_ignore_ascii_case("zranim") {
        if let Some((_, format)) = stem.rsplit_once('.') {
            return &name[name.len() - format.len() - extension.len() - 1..];
        }
    }
    extension
}

pub(in crate::ui::retained_host::app) struct AssetNamePromptSession {
    pub(super) generation: ProjectAssetGenerationToken,
    pub(super) catalog: Arc<EditorAssetCatalogGeneration>,
    pub(super) request: AssetNamePromptRequest,
}

impl AssetNamePromptSession {
    pub(super) fn candidate(&self, query: &str) -> Result<AssetUri, &'static str> {
        available_target(&self.request, &self.catalog, query)
    }

    pub(super) fn state(
        &self,
        query: &str,
        i18n: &EditorI18nService,
    ) -> WorkbenchCommandPaletteOpenState {
        prompt_state(&self.request, &self.catalog, query, i18n)
    }
}

pub(super) fn available_target(
    request: &AssetNamePromptRequest,
    catalog: &EditorAssetCatalogGeneration,
    name: &str,
) -> Result<AssetUri, &'static str> {
    let target = request.target(name)?;
    if catalog.asset_by_locator(&target.to_string()).is_some()
        || catalog.folder(&target.to_string()).is_some()
    {
        return Err("asset.name_prompt.occupied");
    }
    Ok(target)
}

fn prompt_state(
    request: &AssetNamePromptRequest,
    catalog: &EditorAssetCatalogGeneration,
    query: &str,
    i18n: &EditorI18nService,
) -> WorkbenchCommandPaletteOpenState {
    let target = available_target(request, catalog, query).ok();
    let commands = target.as_ref().map(|target| {
        let label_key = if matches!(request, AssetNamePromptRequest::CreateFolder { .. }) {
            "asset.name_prompt.create_candidate"
        } else {
            "asset.name_prompt.rename_candidate"
        };
        UiValue::Map(BTreeMap::from([
            (
                "id".to_owned(),
                UiValue::String(PROMPT_COMMIT_ID.to_owned()),
            ),
            (
                "label".to_owned(),
                UiValue::String(
                    i18n.translate(label_key)
                        .replace("{target}", &target.to_string()),
                ),
            ),
            (
                "source".to_owned(),
                UiValue::String(PROMPT_SOURCE.to_owned()),
            ),
            (
                "category".to_owned(),
                UiValue::String(i18n.translate("menu.assets.label").to_string()),
            ),
        ]))
    });
    let has_candidate = commands.is_some();
    WorkbenchCommandPaletteOpenState {
        query: query.to_owned(),
        commands: UiValue::Array(commands.into_iter().collect()),
        filtered_commands: UiValue::Array(
            has_candidate
                .then(|| UiValue::String(PROMPT_COMMIT_ID.to_owned()))
                .into_iter()
                .collect(),
        ),
        selected_command_id: if has_candidate {
            PROMPT_COMMIT_ID.to_owned()
        } else {
            String::new()
        },
        focused_index: if has_candidate { 0 } else { -1 },
        catalog_generation: catalog.publish_epoch,
        total_match_count: usize::from(has_candidate),
        window_offset: 0,
    }
}

fn project_source(value: &str, allow_root: bool) -> Result<AssetUri, &'static str> {
    let source = AssetUri::parse(value).map_err(|_| "asset.name_prompt.unsupported")?;
    if source.scheme() != ResourceScheme::Res
        || source.label().is_some()
        || (!allow_root && source.path().is_empty())
    {
        return Err("asset.name_prompt.unsupported");
    }
    Ok(source)
}

fn validate_name(name: &str) -> Result<(), &'static str> {
    if name.is_empty()
        || name.trim() != name
        || matches!(name, "." | "..")
        || name.ends_with('.')
        || name.chars().any(|ch| {
            ch.is_control()
                || matches!(
                    ch,
                    '/' | '\\' | ':' | '#' | '<' | '>' | '"' | '|' | '?' | '*'
                )
        })
    {
        return Err("asset.name_prompt.invalid_name");
    }
    let stem = name
        .split('.')
        .next()
        .unwrap_or_default()
        .to_ascii_uppercase();
    if matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || stem
            .strip_prefix("COM")
            .is_some_and(is_reserved_device_number)
        || stem
            .strip_prefix("LPT")
            .is_some_and(is_reserved_device_number)
    {
        return Err("asset.name_prompt.invalid_name");
    }
    Ok(())
}

fn is_reserved_device_number(value: &str) -> bool {
    value.len() == 1 && matches!(value.as_bytes()[0], b'1'..=b'9')
}

#[cfg(test)]
#[path = "tests/session.rs"]
mod tests;
