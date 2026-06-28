use std::path::{Path, PathBuf};

use crate::asset::{AssetUri, FontAsset, ProjectAssetManager};
use zircon_runtime_interface::ui::surface::UiTextRenderMode;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LoadedUiFontManifest {
    pub(crate) source_path: PathBuf,
    pub(crate) asset: Option<FontAsset>,
    pub(crate) family: Option<String>,
    pub(crate) render_mode: Option<UiTextRenderMode>,
    pub(crate) face_index: u32,
}

#[cfg(test)]
pub(crate) fn load_ui_font_manifest(asset_ref: &str) -> Option<LoadedUiFontManifest> {
    load_ui_font_manifest_with_asset_manager(asset_ref, None)
}

pub(crate) fn load_ui_font_manifest_with_asset_manager(
    asset_ref: &str,
    asset_manager: Option<&ProjectAssetManager>,
) -> Option<LoadedUiFontManifest> {
    if asset_ref.ends_with(".toml") {
        if let Some(manifest) =
            asset_manager.and_then(|manager| load_project_ui_font_manifest(manager, asset_ref))
        {
            return Some(manifest);
        }

        let manifest_path = resolve_font_asset_path(asset_ref)?;
        let manifest = std::fs::read_to_string(&manifest_path).ok()?;
        let manifest = FontAsset::from_toml_str(&manifest).ok()?;
        let source_path =
            resolve_manifest_source_path(asset_ref, &manifest_path, manifest.source.as_str())?;
        let render_mode = effective_ui_font_render_mode(&manifest);
        let family = manifest.family.clone();
        let face_index = manifest.face_index;
        return Some(LoadedUiFontManifest {
            source_path,
            asset: Some(manifest),
            family,
            render_mode,
            face_index,
        });
    }

    let source_path = resolve_font_asset_path(asset_ref)?;
    Some(LoadedUiFontManifest {
        source_path,
        asset: None,
        family: None,
        render_mode: None,
        face_index: 0,
    })
}

fn load_project_ui_font_manifest(
    asset_manager: &ProjectAssetManager,
    asset_ref: &str,
) -> Option<LoadedUiFontManifest> {
    if !asset_ref.starts_with("res://") {
        return None;
    }

    let uri = AssetUri::parse(asset_ref).ok()?;
    let project = asset_manager.current_project_manager()?;
    let manifest_id = asset_manager.resolve_asset_id(&uri)?;
    let manifest = asset_manager.load_font_asset(manifest_id).ok()?;
    let manifest_path = project.paths().assets_root().join(uri.path());
    let source_path = resolve_manifest_source_path_with_allowed_root(
        &manifest_path,
        manifest.source.as_str(),
        project.paths().assets_root(),
    )?;
    let render_mode = effective_ui_font_render_mode(&manifest);
    let family = manifest.family.clone();
    let face_index = manifest.face_index;
    Some(LoadedUiFontManifest {
        source_path,
        asset: Some(manifest),
        family,
        render_mode,
        face_index,
    })
}

fn effective_ui_font_render_mode(manifest: &FontAsset) -> Option<UiTextRenderMode> {
    manifest.effective_render_mode()
}

fn resolve_font_asset_path(asset_ref: &str) -> Option<PathBuf> {
    if let Some(relative) = asset_ref.strip_prefix("res://") {
        return Some(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("assets")
                .join(relative),
        );
    }
    let path = PathBuf::from(asset_ref);
    path.is_absolute()
        .then_some(path)
        .or(Some(Path::new(env!("CARGO_MANIFEST_DIR")).join(asset_ref)))
}

fn resolve_manifest_source_path(
    asset_ref: &str,
    manifest_path: &Path,
    source: &str,
) -> Option<PathBuf> {
    let allowed_root = if asset_ref.starts_with("res://") {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("assets")
    } else {
        manifest_path.parent()?.to_path_buf()
    };
    resolve_manifest_source_path_with_allowed_root(manifest_path, source, allowed_root)
}

fn resolve_manifest_source_path_with_allowed_root(
    manifest_path: &Path,
    source: &str,
    allowed_root: impl AsRef<Path>,
) -> Option<PathBuf> {
    let source = source.trim();
    if source.is_empty() {
        return None;
    }

    let source_path = PathBuf::from(source);
    if source_path.is_absolute() {
        return None;
    }

    let resolved = manifest_path.parent()?.join(&source_path);
    let canonical_allowed_root = std::fs::canonicalize(allowed_root).ok()?;
    let canonical_resolved = std::fs::canonicalize(&resolved).ok()?;

    canonical_resolved
        .starts_with(&canonical_allowed_root)
        .then_some(resolved)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::FontAssetRenderStrategy;

    #[test]
    fn render_strategy_default_mode_feeds_ui_font_default() {
        let manifest = font_manifest(
            None,
            FontAssetRenderStrategy {
                default_mode: Some(UiTextRenderMode::Sdf),
                allow_native: None,
                allow_sdf: None,
            },
        );

        assert_eq!(
            effective_ui_font_render_mode(&manifest),
            Some(UiTextRenderMode::Sdf)
        );
    }

    #[test]
    fn legacy_render_mode_takes_priority_over_strategy_default_mode() {
        let manifest = font_manifest(
            Some(UiTextRenderMode::Native),
            FontAssetRenderStrategy {
                default_mode: Some(UiTextRenderMode::Sdf),
                allow_native: None,
                allow_sdf: None,
            },
        );

        assert_eq!(
            effective_ui_font_render_mode(&manifest),
            Some(UiTextRenderMode::Native)
        );
    }

    #[test]
    fn render_strategy_constraints_clamp_disallowed_auto_default() {
        let manifest = font_manifest(
            None,
            FontAssetRenderStrategy {
                default_mode: Some(UiTextRenderMode::Auto),
                allow_native: Some(false),
                allow_sdf: Some(true),
            },
        );

        assert_eq!(
            effective_ui_font_render_mode(&manifest),
            Some(UiTextRenderMode::Sdf)
        );
    }

    fn font_manifest(
        render_mode: Option<UiTextRenderMode>,
        render_strategy: FontAssetRenderStrategy,
    ) -> FontAsset {
        FontAsset {
            source: "FiraMono-subset.ttf".to_string(),
            family: Some("Fira Mono".to_string()),
            render_mode,
            face_index: 0,
            family_members: Vec::new(),
            variable_instances: Vec::new(),
            fallback_families: Vec::new(),
            render_strategy,
            metadata: None,
        }
    }
}
