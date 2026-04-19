use super::*;
use crate::ui::slint_host::callback_dispatch::dispatch_builtin_floating_window_focus_for_source;
use crate::ui::slint_host::floating_window_projection::{
    resolve_floating_window_projection_content_frame,
    resolve_floating_window_projection_shared_source, resolve_native_floating_window_host_frame,
};
use crate::{ActivityDrawerSlot, MainPageId, ShellFrame};

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

pub(crate) fn resolve_callback_source_window_id(ui: &UiHostWindow) -> Option<MainPageId> {
    let host_shell = ui.get_host_shell();
    if !host_shell.native_floating_window_mode {
        return None;
    }

    let window_id = host_shell.native_floating_window_id.to_string();
    if window_id.trim().is_empty() {
        None
    } else {
        Some(MainPageId::new(window_id))
    }
}

fn is_valid_size(size: UiSize) -> bool {
    size.width > 0.0 && size.height > 0.0
}

fn frame_size(frame: ShellFrame) -> Option<UiSize> {
    let size = UiSize::new(frame.width.max(0.0), frame.height.max(0.0));
    is_valid_size(size).then_some(size)
}

fn ui_frame_size(frame: UiFrame) -> Option<UiSize> {
    let size = UiSize::new(frame.width.max(0.0), frame.height.max(0.0));
    is_valid_size(size).then_some(size)
}

fn drawer_slot_region(slot: ActivityDrawerSlot) -> ShellRegionId {
    match slot {
        ActivityDrawerSlot::LeftTop | ActivityDrawerSlot::LeftBottom => ShellRegionId::Left,
        ActivityDrawerSlot::RightTop | ActivityDrawerSlot::RightBottom => ShellRegionId::Right,
        ActivityDrawerSlot::BottomLeft | ActivityDrawerSlot::BottomRight => ShellRegionId::Bottom,
    }
}

fn active_drawer_region_for_kind(
    workbench: &crate::WorkbenchSnapshot,
    kind: ViewContentKind,
) -> Option<ShellRegionId> {
    workbench
        .drawers
        .values()
        .find(|drawer| {
            drawer.visible
                && drawer.mode != ActivityDrawerMode::Collapsed
                && drawer
                    .active_tab
                    .as_ref()
                    .and_then(|active| drawer.tabs.iter().find(|tab| &tab.instance_id == active))
                    .or_else(|| drawer.tabs.first())
                    .is_some_and(|tab| tab.content_kind == kind)
        })
        .map(|drawer| drawer_slot_region(drawer.slot))
}

fn active_main_page_matches_kind(
    workbench: &crate::WorkbenchSnapshot,
    kind: ViewContentKind,
) -> bool {
    let Some(page) = workbench.main_pages.iter().find(|page| match page {
        crate::MainPageSnapshot::Workbench { id, .. }
        | crate::MainPageSnapshot::Exclusive { id, .. } => id == &workbench.active_main_page,
    }) else {
        return false;
    };

    match page {
        crate::MainPageSnapshot::Workbench { workspace, .. } => {
            active_workspace_tab(workspace).is_some_and(|tab| tab.content_kind == kind)
        }
        crate::MainPageSnapshot::Exclusive { view, .. } => view.content_kind == kind,
    }
}

fn asset_surface_kind(surface_mode: &str) -> Option<ViewContentKind> {
    match surface_mode {
        "activity" => Some(ViewContentKind::Assets),
        "browser" => Some(ViewContentKind::AssetBrowser),
        _ => None,
    }
}

impl SlintEditorHost {
    pub(super) fn resolve_floating_window_content_frame_for_window(
        &self,
        window_id: &MainPageId,
    ) -> Option<ShellFrame> {
        self.floating_window_projection_bundle
            .content_frame(window_id)
            .or_else(|| {
                let chrome = self.runtime.chrome_snapshot();
                let model = WorkbenchViewModel::build(&chrome);
                let window_index = model
                    .floating_windows
                    .iter()
                    .position(|window| &window.window_id == window_id)?;
                let shared_source = resolve_floating_window_projection_shared_source(
                    &self.floating_window_source_bridge.source_frames(),
                );
                let native_window_hosts = self.editor_manager.native_window_hosts();
                let host_frame =
                    resolve_native_floating_window_host_frame(&native_window_hosts, window_id);
                Some(resolve_floating_window_projection_content_frame(
                    &model.floating_windows[window_index],
                    window_index,
                    shared_source,
                    &self.chrome_metrics,
                    host_frame,
                ))
            })
    }

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

    fn resolve_host_frame_backed_size_for_kind(&self, kind: ViewContentKind) -> Option<UiSize> {
        if let Some(window_id) = self.callback_source_window.as_ref() {
            return self
                .resolve_floating_window_content_frame_for_window(window_id)
                .and_then(frame_size);
        }

        let root_shell_frames = self.template_bridge.root_shell_frames();
        let workbench = &self.runtime.chrome_snapshot().workbench;
        if let Some(region) = active_drawer_region_for_kind(workbench, kind) {
            return root_shell_frames
                .drawer_content_frame(region)
                .and_then(ui_frame_size);
        }

        if active_main_page_matches_kind(workbench, kind) {
            return root_shell_frames
                .pane_surface_frame
                .and_then(ui_frame_size)
                .or_else(|| {
                    root_shell_frames
                        .document_host_frame
                        .and_then(ui_frame_size)
                });
        }

        None
    }

    pub(super) fn resolve_callback_surface_size_for_kind(
        &self,
        width: f32,
        height: f32,
        cached_size: UiSize,
        kind: ViewContentKind,
    ) -> UiSize {
        let callback_size = UiSize::new(width.max(0.0), height.max(0.0));
        if is_valid_size(callback_size) {
            return callback_size;
        }
        if is_valid_size(cached_size) {
            return cached_size;
        }

        self.resolve_host_frame_backed_size_for_kind(kind)
            .unwrap_or(UiSize::new(0.0, 0.0))
    }

    pub(super) fn resolve_callback_surface_size_for_asset_surface(
        &self,
        surface_mode: &str,
        width: f32,
        height: f32,
        cached_size: UiSize,
    ) -> Option<UiSize> {
        asset_surface_kind(surface_mode).map(|kind| {
            self.resolve_callback_surface_size_for_kind(width, height, cached_size, kind)
        })
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
    use crate::ui::slint_host::UiHostWindow;
    use crate::MainPageId;

    #[test]
    fn resolve_callback_source_window_id_returns_none_for_root_shell() {
        i_slint_backend_testing::init_no_event_loop();

        let ui = UiHostWindow::new().expect("workbench shell should instantiate");
        assert_eq!(resolve_callback_source_window_id(&ui), None);
    }

    #[test]
    fn resolve_callback_source_window_id_reads_native_child_window_identity() {
        i_slint_backend_testing::init_no_event_loop();

        let ui = UiHostWindow::new().expect("workbench shell should instantiate");
        let mut host_shell = ui.get_host_shell();
        host_shell.native_floating_window_mode = true;
        host_shell.native_floating_window_id = "window:native-preview".into();
        ui.set_host_shell(host_shell);

        assert_eq!(
            resolve_callback_source_window_id(&ui),
            Some(MainPageId::new("window:native-preview"))
        );
    }
}
