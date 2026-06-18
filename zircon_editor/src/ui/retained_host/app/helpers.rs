use super::*;
use std::path::Path;

mod animation_assets;
mod callback_surface;
pub(crate) use animation_assets::derive_animation_assets_from_model_source;
pub(crate) use callback_surface::resolve_callback_source_window_id;

use crate::ui::workbench::autolayout::ShellFrame;
use crate::ui::workbench::snapshot::{MainPageSnapshot, ViewContentKind};

pub(crate) fn asset_surface_visible(
    chrome: &crate::ui::workbench::snapshot::EditorChromeSnapshot,
    kind: ViewContentKind,
) -> bool {
    let Some(page) = chrome.workbench.main_pages.iter().find(|page| match page {
        MainPageSnapshot::Workbench { id, .. } | MainPageSnapshot::Exclusive { id, .. } => {
            id == &chrome.workbench.active_main_page
        }
    }) else {
        return false;
    };

    match page {
        MainPageSnapshot::Workbench { workspace, .. } => {
            let drawer_visible = chrome.workbench.drawers.values().any(|drawer| {
                drawer.visible
                    && drawer.mode != ActivityDrawerMode::Collapsed
                    && drawer
                        .active_tab
                        .as_ref()
                        .and_then(|active| {
                            drawer.tabs.iter().find(|tab| &tab.instance_id == active)
                        })
                        .or_else(|| drawer.tabs.first())
                        .is_some_and(|tab| tab.content_kind == kind)
            });
            drawer_visible
                || active_workspace_tab(workspace).is_some_and(|tab| tab.content_kind == kind)
        }
        MainPageSnapshot::Exclusive { view, .. } => view.content_kind == kind,
    }
}

fn active_workspace_tab(
    workspace: &crate::ui::workbench::snapshot::DocumentWorkspaceSnapshot,
) -> Option<&crate::ui::workbench::snapshot::ViewTabSnapshot> {
    match workspace {
        crate::ui::workbench::snapshot::DocumentWorkspaceSnapshot::Split {
            first, second, ..
        } => active_workspace_tab(first).or_else(|| active_workspace_tab(second)),
        crate::ui::workbench::snapshot::DocumentWorkspaceSnapshot::Tabs { tabs, active_tab } => {
            active_tab
                .as_ref()
                .and_then(|active| tabs.iter().find(|tab| &tab.instance_id == active))
                .or_else(|| tabs.first())
        }
    }
}

pub(crate) fn viewport_size_from_frame(frame: ShellFrame) -> Option<UVec2> {
    let width = frame.width.max(0.0).round() as u32;
    let height = frame.height.max(0.0).round() as u32;
    if width == 0 || height == 0 {
        None
    } else {
        Some(UVec2::new(width, height))
    }
}

pub(crate) fn compute_window_menu_popup_height(
    shell_height: f32,
    button_frame: UiFrame,
    preset_count: usize,
) -> f32 {
    let popup_y = button_frame.y + button_frame.height + 3.0;
    let content_height = 72.0 + preset_count as f32 * 30.0;
    let available_height = (shell_height - popup_y - 12.0).max(72.0);
    content_height.min(available_height)
}

pub(crate) fn shell_region_group_key(region: ShellRegionId) -> &'static str {
    match region {
        ShellRegionId::Left => "left",
        ShellRegionId::Right => "right",
        ShellRegionId::Bottom => "bottom",
        ShellRegionId::Document => "document",
    }
}

pub(crate) fn stage_model_source(
    paths: &ProjectPaths,
    source: &Path,
) -> Result<(ResourceLocator, String), String> {
    if let Ok(relative) = source.strip_prefix(paths.assets_root()) {
        let uri = asset_uri_from_relative_path(relative)?;
        return Ok((uri, source.to_string_lossy().into_owned()));
    }

    let extension = source
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if extension == "gltf" {
        return Err(
            "External .gltf import is not supported yet; copy the model folder into Project/assets or use .glb".to_string(),
        );
    }

    let destination = paths.assets_root().join("models").join(
        source
            .file_name()
            .ok_or_else(|| format!("model path has no file name: {}", source.display()))?,
    );
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    if source != destination {
        fs::copy(source, &destination).map_err(|error| {
            format!(
                "failed to copy model {} into project assets: {error}",
                source.display()
            )
        })?;
        if extension == "obj" {
            let sibling_mtl = source.with_extension("mtl");
            if sibling_mtl.exists() {
                let _ = fs::copy(sibling_mtl, destination.with_extension("mtl"));
            }
        }
    }

    Ok((
        asset_uri_from_relative_path(
            Path::new("models").join(destination.file_name().ok_or_else(|| {
                format!("model path has no file name: {}", destination.display())
            })?),
        )?,
        destination.to_string_lossy().into_owned(),
    ))
}

fn asset_uri_from_relative_path(relative: impl AsRef<Path>) -> Result<ResourceLocator, String> {
    let normalized = relative
        .as_ref()
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/");
    ResourceLocator::parse(&format!("res://{normalized}")).map_err(|error| error.to_string())
}
