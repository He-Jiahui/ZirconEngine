use super::*;
use crate::host::slint_host::callback_dispatch::dispatch_builtin_floating_window_focus_for_source;
use crate::MainPageId;

pub(crate) fn asset_surface_visible(
    chrome: &crate::EditorChromeSnapshot,
    kind: ViewContentKind,
) -> bool {
    let Some(page) = chrome.workbench.main_pages.iter().find(|page| match page {
        crate::MainPageSnapshot::Workbench { id, .. }
        | crate::MainPageSnapshot::Exclusive { id, .. } => id == &chrome.workbench.active_main_page,
    }) else {
        return false;
    };

    match page {
        crate::MainPageSnapshot::Workbench { workspace, .. } => {
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
        crate::MainPageSnapshot::Exclusive { view, .. } => view.content_kind == kind,
    }
}

fn active_workspace_tab(
    workspace: &crate::DocumentWorkspaceSnapshot,
) -> Option<&crate::ViewTabSnapshot> {
    match workspace {
        crate::DocumentWorkspaceSnapshot::Split { first, second, .. } => {
            active_workspace_tab(first).or_else(|| active_workspace_tab(second))
        }
        crate::DocumentWorkspaceSnapshot::Tabs { tabs, active_tab } => active_tab
            .as_ref()
            .and_then(|active| tabs.iter().find(|tab| &tab.instance_id == active))
            .or_else(|| tabs.first()),
    }
}

pub(crate) fn viewport_size_from_frame(geometry: &WorkbenchShellGeometry) -> Option<UVec2> {
    let width = geometry.viewport_content_frame.width.max(0.0).round() as u32;
    let height = geometry.viewport_content_frame.height.max(0.0).round() as u32;
    if width == 0 || height == 0 {
        None
    } else {
        Some(UVec2::new(width, height))
    }
}

pub(crate) fn frame_rect_to_ui_frame(frame: FrameRect) -> UiFrame {
    UiFrame::new(frame.x, frame.y, frame.width, frame.height)
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

pub(crate) fn resolve_callback_source_window_id(ui: &WorkbenchShell) -> Option<MainPageId> {
    if !ui.get_native_floating_window_mode() {
        return None;
    }

    let window_id = ui.get_native_floating_window_id().to_string();
    if window_id.trim().is_empty() {
        None
    } else {
        Some(MainPageId::new(window_id))
    }
}

impl SlintEditorHost {
    pub(super) fn with_callback_source_window<T>(
        &mut self,
        source_window_id: Option<MainPageId>,
        callback: impl FnOnce(&mut Self) -> T,
    ) -> T {
        let previous = self.callback_source_window.clone();
        self.callback_source_window = source_window_id;
        let result = callback(self);
        self.callback_source_window = previous;
        result
    }

    pub(super) fn focus_callback_source_window(&mut self) {
        let source_window_id = self.callback_source_window.clone();
        let Some(source_window_id) = source_window_id else {
            self.last_focused_callback_window = None;
            return;
        };

        match dispatch_builtin_floating_window_focus_for_source(
            &self.runtime,
            Some(&source_window_id),
            self.last_focused_callback_window.as_ref(),
        ) {
            Some(Ok(effects)) => {
                self.apply_dispatch_effects(effects);
                self.last_focused_callback_window = Some(source_window_id);
            }
            Some(Err(error)) => self.set_status_line(error),
            None => {
                self.last_focused_callback_window = Some(source_window_id);
            }
        }
    }

    pub(super) fn note_focused_floating_window(&mut self, window_id: Option<MainPageId>) {
        self.last_focused_callback_window = window_id;
    }

    pub(super) fn note_focused_floating_window_surface(&mut self, surface_key: &str) {
        if surface_key == "main" {
            self.last_focused_callback_window = None;
            return;
        }

        self.last_focused_callback_window = self
            .runtime
            .chrome_snapshot()
            .workbench
            .floating_windows
            .iter()
            .find(|window| window.window_id.0 == surface_key)
            .map(|window| window.window_id.clone());
    }
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

#[cfg(test)]
mod tests {
    use super::resolve_callback_source_window_id;
    use crate::host::slint_host::WorkbenchShell;
    use crate::MainPageId;

    #[test]
    fn resolve_callback_source_window_id_returns_none_for_root_shell() {
        i_slint_backend_testing::init_no_event_loop();

        let ui = WorkbenchShell::new().expect("workbench shell should instantiate");
        assert_eq!(resolve_callback_source_window_id(&ui), None);
    }

    #[test]
    fn resolve_callback_source_window_id_reads_native_child_window_identity() {
        i_slint_backend_testing::init_no_event_loop();

        let ui = WorkbenchShell::new().expect("workbench shell should instantiate");
        ui.set_native_floating_window_mode(true);
        ui.set_native_floating_window_id("window:native-preview".into());

        assert_eq!(
            resolve_callback_source_window_id(&ui),
            Some(MainPageId::new("window:native-preview"))
        );
    }
}
