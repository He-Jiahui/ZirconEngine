use super::*;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::ui::slint_host::callback_dispatch::dispatch_builtin_floating_window_focus_for_source;
use crate::ui::slint_host::floating_window_projection::{
    resolve_floating_window_projection_content_frame,
    resolve_floating_window_projection_shared_source, resolve_native_floating_window_host_frame,
};
use crate::ui::workbench::autolayout::ShellFrame;
use crate::ui::workbench::layout::{ActivityDrawerSlot, MainPageId};
use crate::ui::workbench::snapshot::{MainPageSnapshot, ViewContentKind};
use zircon_runtime::asset::{
    AnimationChannelAsset, AnimationChannelKeyAsset, AnimationChannelValueAsset,
    AnimationClipAsset, AnimationClipBoneTrackAsset, AnimationInterpolationAsset,
    AnimationSkeletonAsset, AnimationSkeletonBoneAsset, AssetReference,
};

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

pub(crate) fn resolve_callback_source_window_id(ui: &UiHostWindow) -> Option<MainPageId> {
    let host_shell = ui.get_host_presentation().host_shell;
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
    workbench: &crate::ui::workbench::snapshot::WorkbenchSnapshot,
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
    workbench: &crate::ui::workbench::snapshot::WorkbenchSnapshot,
    kind: ViewContentKind,
) -> bool {
    let Some(page) = workbench.main_pages.iter().find(|page| match page {
        MainPageSnapshot::Workbench { id, .. } | MainPageSnapshot::Exclusive { id, .. } => {
            id == &workbench.active_main_page
        }
    }) else {
        return false;
    };

    match page {
        MainPageSnapshot::Workbench { workspace, .. } => {
            active_workspace_tab(workspace).is_some_and(|tab| tab.content_kind == kind)
        }
        MainPageSnapshot::Exclusive { view, .. } => view.content_kind == kind,
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

pub(crate) fn derive_animation_assets_from_model_source(
    assets_root: &Path,
    model_source: &Path,
) -> Result<Vec<ResourceLocator>, String> {
    let extension = model_source
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if extension != "gltf" && extension != "glb" {
        return Ok(Vec::new());
    }

    let relative_model_path = model_source.strip_prefix(assets_root).map_err(|_| {
        format!(
            "model source {} is not inside project assets {}",
            model_source.display(),
            assets_root.display()
        )
    })?;
    let base_name = relative_model_path
        .file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("model source has no file stem: {}", model_source.display()))?;

    let (document, buffers, _) = gltf::import(model_source).map_err(|error| {
        format!(
            "parse gltf animation data from {}: {error}",
            model_source.display()
        )
    })?;
    let Some(skin) = document.skins().next() else {
        return Ok(Vec::new());
    };

    let parent_indices = node_parent_indices(&document);
    let skeleton_file_name = format!("{base_name}.skeleton.zranim");
    let skeleton_relative_path =
        sibling_relative_path(relative_model_path, Path::new(&skeleton_file_name));
    let skeleton_locator = asset_uri_from_relative_path(&skeleton_relative_path)?;
    let derived_skeleton =
        derive_skeleton_asset(&skin, &parent_indices, &skeleton_locator, base_name)?;
    write_animation_asset_bytes(
        &assets_root.join(&skeleton_relative_path),
        derived_skeleton.asset.to_bytes()?,
    )?;

    let mut generated = vec![skeleton_locator.clone()];
    for (animation_index, animation) in document.animations().enumerate() {
        let clip_segment = sanitize_animation_asset_segment(animation.name(), animation_index);
        let clip_file_name = format!("{base_name}.{clip_segment}.clip.zranim");
        let clip_relative_path =
            sibling_relative_path(relative_model_path, Path::new(&clip_file_name));
        let clip_locator = asset_uri_from_relative_path(&clip_relative_path)?;
        let clip_asset =
            derive_clip_asset(&animation, &buffers, &derived_skeleton, &skeleton_locator)?;
        write_animation_asset_bytes(
            &assets_root.join(&clip_relative_path),
            clip_asset.to_bytes()?,
        )?;
        generated.push(clip_locator);
    }

    generated.sort_by_key(|locator| locator.to_string());
    Ok(generated)
}

#[derive(Clone)]
struct DerivedSkeletonAsset {
    asset: AnimationSkeletonAsset,
    joints: BTreeMap<usize, DerivedJoint>,
}

#[derive(Clone)]
struct DerivedJoint {
    bone_name: String,
    local_translation: [f32; 3],
    local_rotation: [f32; 4],
    local_scale: [f32; 3],
}

#[derive(Clone)]
struct DerivedClipTrack {
    translation: AnimationChannelAsset,
    rotation: AnimationChannelAsset,
    scale: AnimationChannelAsset,
}

fn derive_skeleton_asset(
    skin: &gltf::Skin<'_>,
    parent_indices: &BTreeMap<usize, usize>,
    _skeleton_locator: &ResourceLocator,
    fallback_name: &str,
) -> Result<DerivedSkeletonAsset, String> {
    let joints = skin.joints().collect::<Vec<_>>();
    if joints.is_empty() {
        return Err("gltf skin does not define any joints".to_string());
    }

    let joint_lookup = joints
        .iter()
        .enumerate()
        .map(|(index, joint)| (joint.index(), index as u32))
        .collect::<BTreeMap<_, _>>();

    let mut derived_joints = BTreeMap::new();
    let bones = joints
        .into_iter()
        .map(|joint| {
            let (local_translation, local_rotation, local_scale) = joint.transform().decomposed();
            let bone_name = joint_display_name(&joint);
            derived_joints.insert(
                joint.index(),
                DerivedJoint {
                    bone_name: bone_name.clone(),
                    local_translation,
                    local_rotation,
                    local_scale,
                },
            );
            AnimationSkeletonBoneAsset {
                name: bone_name,
                parent_index: parent_indices
                    .get(&joint.index())
                    .and_then(|parent_index| joint_lookup.get(parent_index))
                    .copied(),
                local_translation,
                local_rotation,
                local_scale,
            }
        })
        .collect();

    Ok(DerivedSkeletonAsset {
        asset: AnimationSkeletonAsset {
            name: skin
                .name()
                .map(str::to_string)
                .or_else(|| Some(fallback_name.to_string())),
            bones,
        },
        joints: derived_joints,
    })
}

fn derive_clip_asset(
    animation: &gltf::Animation<'_>,
    buffers: &[gltf::buffer::Data],
    skeleton: &DerivedSkeletonAsset,
    skeleton_locator: &ResourceLocator,
) -> Result<AnimationClipAsset, String> {
    let mut tracks = skeleton
        .joints
        .values()
        .map(|joint| {
            (
                joint.bone_name.clone(),
                DerivedClipTrack {
                    translation: constant_vec3_channel(joint.local_translation),
                    rotation: constant_quaternion_channel(joint.local_rotation),
                    scale: constant_vec3_channel(joint.local_scale),
                },
            )
        })
        .collect::<BTreeMap<_, _>>();

    let mut duration_seconds = 0.0_f32;
    for channel in animation.channels() {
        let target_node = channel.target().node().index();
        let Some(joint) = skeleton.joints.get(&target_node) else {
            continue;
        };
        let reader = channel.reader(|buffer| Some(&buffers[buffer.index()].0));
        let times = reader
            .read_inputs()
            .ok_or_else(|| "gltf animation channel is missing keyframe times".to_string())?
            .collect::<Vec<_>>();
        if let Some(last_time) = times.last().copied() {
            duration_seconds = duration_seconds.max(last_time);
        }
        let interpolation = map_animation_interpolation(channel.sampler().interpolation());
        let track = tracks
            .get_mut(&joint.bone_name)
            .ok_or_else(|| format!("missing derived joint track for {}", joint.bone_name))?;

        match reader
            .read_outputs()
            .ok_or_else(|| "gltf animation channel is missing output values".to_string())?
        {
            gltf::animation::util::ReadOutputs::Translations(values) => {
                track.translation =
                    vec3_channel_from_samples(&times, &values.collect::<Vec<_>>(), interpolation)?;
            }
            gltf::animation::util::ReadOutputs::Rotations(values) => {
                track.rotation = quaternion_channel_from_samples(
                    &times,
                    &values.into_f32().collect::<Vec<_>>(),
                    interpolation,
                )?;
            }
            gltf::animation::util::ReadOutputs::Scales(values) => {
                track.scale =
                    vec3_channel_from_samples(&times, &values.collect::<Vec<_>>(), interpolation)?;
            }
            gltf::animation::util::ReadOutputs::MorphTargetWeights(_) => {}
        }
    }

    Ok(AnimationClipAsset {
        name: animation.name().map(str::to_string),
        skeleton: AssetReference::from_locator(skeleton_locator.clone()),
        duration_seconds,
        tracks: tracks
            .into_iter()
            .map(|(bone_name, track)| AnimationClipBoneTrackAsset {
                bone_name,
                translation: track.translation,
                rotation: track.rotation,
                scale: track.scale,
            })
            .collect(),
    })
}

fn node_parent_indices(document: &gltf::Document) -> BTreeMap<usize, usize> {
    let mut parents = BTreeMap::new();
    for node in document.nodes() {
        for child in node.children() {
            parents.insert(child.index(), node.index());
        }
    }
    parents
}

fn joint_display_name(joint: &gltf::Node<'_>) -> String {
    joint
        .name()
        .map(str::to_string)
        .unwrap_or_else(|| format!("joint_{}", joint.index()))
}

fn sibling_relative_path(model_relative_path: &Path, file_name: &Path) -> PathBuf {
    model_relative_path
        .parent()
        .map(|parent| parent.join(file_name))
        .unwrap_or_else(|| file_name.to_path_buf())
}

fn sanitize_animation_asset_segment(name: Option<&str>, animation_index: usize) -> String {
    let sanitized = name
        .unwrap_or_default()
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string();
    if sanitized.is_empty() {
        format!("clip_{animation_index:03}")
    } else {
        sanitized
    }
}

fn map_animation_interpolation(
    interpolation: gltf::animation::Interpolation,
) -> AnimationInterpolationAsset {
    match interpolation {
        gltf::animation::Interpolation::Step => AnimationInterpolationAsset::Step,
        gltf::animation::Interpolation::Linear | gltf::animation::Interpolation::CubicSpline => {
            AnimationInterpolationAsset::Hermite
        }
    }
}

fn constant_vec3_channel(value: [f32; 3]) -> AnimationChannelAsset {
    AnimationChannelAsset {
        interpolation: AnimationInterpolationAsset::Step,
        keys: vec![AnimationChannelKeyAsset {
            time_seconds: 0.0,
            value: AnimationChannelValueAsset::Vec3(value),
            in_tangent: None,
            out_tangent: None,
        }],
    }
}

fn constant_quaternion_channel(value: [f32; 4]) -> AnimationChannelAsset {
    AnimationChannelAsset {
        interpolation: AnimationInterpolationAsset::Step,
        keys: vec![AnimationChannelKeyAsset {
            time_seconds: 0.0,
            value: AnimationChannelValueAsset::Quaternion(value),
            in_tangent: None,
            out_tangent: None,
        }],
    }
}

fn vec3_channel_from_samples(
    times: &[f32],
    values: &[[f32; 3]],
    interpolation: AnimationInterpolationAsset,
) -> Result<AnimationChannelAsset, String> {
    if times.len() != values.len() {
        return Err("gltf animation translation/scaling key count mismatch".to_string());
    }
    Ok(AnimationChannelAsset {
        interpolation,
        keys: times
            .iter()
            .zip(values.iter())
            .map(|(time_seconds, value)| AnimationChannelKeyAsset {
                time_seconds: *time_seconds,
                value: AnimationChannelValueAsset::Vec3(*value),
                in_tangent: matches!(interpolation, AnimationInterpolationAsset::Hermite)
                    .then_some(AnimationChannelValueAsset::Vec3([0.0, 0.0, 0.0])),
                out_tangent: matches!(interpolation, AnimationInterpolationAsset::Hermite)
                    .then_some(AnimationChannelValueAsset::Vec3([0.0, 0.0, 0.0])),
            })
            .collect(),
    })
}

fn quaternion_channel_from_samples(
    times: &[f32],
    values: &[[f32; 4]],
    interpolation: AnimationInterpolationAsset,
) -> Result<AnimationChannelAsset, String> {
    if times.len() != values.len() {
        return Err("gltf animation rotation key count mismatch".to_string());
    }
    Ok(AnimationChannelAsset {
        interpolation,
        keys: times
            .iter()
            .zip(values.iter())
            .map(|(time_seconds, value)| AnimationChannelKeyAsset {
                time_seconds: *time_seconds,
                value: AnimationChannelValueAsset::Quaternion(*value),
                in_tangent: None,
                out_tangent: None,
            })
            .collect(),
    })
}

fn write_animation_asset_bytes(path: &Path, bytes: Vec<u8>) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(path, bytes).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::resolve_callback_source_window_id;
    use crate::ui::slint_host::UiHostWindow;
    use crate::ui::workbench::layout::MainPageId;
    use zircon_runtime::asset::project::{
        AssetMetaDocument, ProjectManager, ProjectManifest, ProjectPaths,
    };
    use zircon_runtime::asset::{AnimationClipAsset, AnimationSkeletonAsset, AssetUri};

    use super::derive_animation_assets_from_model_source;

    fn unique_temp_dir(prefix: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}_{unique}"))
    }

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
        let mut host_presentation = ui.get_host_presentation();
        host_presentation.host_shell.native_floating_window_mode = true;
        host_presentation.host_shell.native_floating_window_id = "window:native-preview".into();
        ui.set_host_presentation(host_presentation);

        assert_eq!(
            resolve_callback_source_window_id(&ui),
            Some(MainPageId::new("window:native-preview"))
        );
    }

    #[test]
    fn derive_animation_assets_from_model_source_writes_stable_sibling_skeleton_and_clip_files() {
        let root = unique_temp_dir("zircon_editor_derived_animation_assets");
        let assets_root = root.join("assets");
        let model_dir = assets_root.join("models");
        fs::create_dir_all(&model_dir).unwrap();
        let model_path = write_animated_gltf(&model_dir);

        let first = derive_animation_assets_from_model_source(&assets_root, &model_path).unwrap();
        let second = derive_animation_assets_from_model_source(&assets_root, &model_path).unwrap();

        assert_eq!(first, second);
        assert_eq!(
            first.iter().map(ToString::to_string).collect::<Vec<_>>(),
            vec![
                "res://models/hero.idle.clip.zranim".to_string(),
                "res://models/hero.skeleton.zranim".to_string(),
            ]
        );

        let skeleton = AnimationSkeletonAsset::from_bytes(
            &fs::read(model_dir.join("hero.skeleton.zranim")).unwrap(),
        )
        .unwrap();
        assert_eq!(skeleton.name.as_deref(), Some("HeroRig"));
        assert_eq!(skeleton.bones.len(), 2);
        assert_eq!(skeleton.bones[0].name, "Root");
        assert_eq!(skeleton.bones[1].name, "Hand");
        assert_eq!(skeleton.bones[1].parent_index, Some(0));

        let clip = AnimationClipAsset::from_bytes(
            &fs::read(model_dir.join("hero.idle.clip.zranim")).unwrap(),
        )
        .unwrap();
        assert_eq!(clip.name.as_deref(), Some("Idle"));
        assert_eq!(
            clip.skeleton.locator.to_string(),
            "res://models/hero.skeleton.zranim"
        );
        assert_eq!(clip.tracks.len(), 2);
        assert!(clip.tracks.iter().any(|track| track.bone_name == "Root"));
        assert!(clip.tracks.iter().any(|track| track.bone_name == "Hand"));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn derive_animation_assets_from_model_source_preserves_project_asset_ids_across_reimport_with_gltf_buffer_sidecars(
    ) {
        let root = unique_temp_dir("zircon_editor_derived_animation_reimport");
        let paths = ProjectPaths::from_root(&root).unwrap();
        paths.ensure_layout().unwrap();
        ProjectManifest::new(
            "Sandbox",
            AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
            1,
        )
        .save(paths.manifest_path())
        .unwrap();
        let model_dir = paths.assets_root().join("models");
        fs::create_dir_all(&model_dir).unwrap();
        let model_path = write_animated_gltf(&model_dir);

        let first_generated =
            derive_animation_assets_from_model_source(paths.assets_root(), &model_path).unwrap();

        let mut manager = ProjectManager::open(&root).unwrap();
        manager.scan_and_import().unwrap();

        let skeleton_uri = AssetUri::parse("res://models/hero.skeleton.zranim").unwrap();
        let clip_uri = AssetUri::parse("res://models/hero.idle.clip.zranim").unwrap();
        let first_skeleton_id = manager
            .registry()
            .get_by_locator(&skeleton_uri)
            .expect("derived skeleton should be imported")
            .id();
        let first_clip_id = manager
            .registry()
            .get_by_locator(&clip_uri)
            .expect("derived clip should be imported")
            .id();
        let first_skeleton_meta =
            AssetMetaDocument::load(model_dir.join("hero.skeleton.zranim.meta.toml")).unwrap();
        let first_clip_meta =
            AssetMetaDocument::load(model_dir.join("hero.idle.clip.zranim.meta.toml")).unwrap();

        let second_generated =
            derive_animation_assets_from_model_source(paths.assets_root(), &model_path).unwrap();
        manager.scan_and_import().unwrap();

        let second_skeleton_id = manager
            .registry()
            .get_by_locator(&skeleton_uri)
            .expect("reimported skeleton should stay registered")
            .id();
        let second_clip_id = manager
            .registry()
            .get_by_locator(&clip_uri)
            .expect("reimported clip should stay registered")
            .id();
        let second_skeleton_meta =
            AssetMetaDocument::load(model_dir.join("hero.skeleton.zranim.meta.toml")).unwrap();
        let second_clip_meta =
            AssetMetaDocument::load(model_dir.join("hero.idle.clip.zranim.meta.toml")).unwrap();

        assert_eq!(first_generated, second_generated);
        assert_eq!(first_skeleton_id, second_skeleton_id);
        assert_eq!(first_clip_id, second_clip_id);
        assert_eq!(
            first_skeleton_meta.asset_uuid,
            second_skeleton_meta.asset_uuid
        );
        assert_eq!(first_clip_meta.asset_uuid, second_clip_meta.asset_uuid);
        assert!(
            !model_dir.join("hero.bin.meta.toml").exists(),
            "gltf buffer sidecars should not get runtime asset metadata sidecars"
        );

        let _ = fs::remove_dir_all(root);
    }

    fn write_animated_gltf(model_dir: &Path) -> PathBuf {
        let model_path = model_dir.join("hero.gltf");
        let buffer_path = model_dir.join("hero.bin");

        let times = [0.0_f32, 1.0_f32];
        let root_translations = [[0.0_f32, 0.0, 0.0], [0.0_f32, 0.0, 0.0]];
        let hand_translations = [[0.2_f32, 0.8, 0.0], [0.4_f32, 1.1, 0.0]];

        let mut bytes = Vec::new();
        let times_offset = bytes.len();
        for value in times {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        let root_translation_offset = bytes.len();
        for sample in root_translations {
            for value in sample {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
        }
        let hand_translation_offset = bytes.len();
        for sample in hand_translations {
            for value in sample {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
        }
        fs::write(&buffer_path, bytes).unwrap();

        fs::write(
            &model_path,
            format!(
                r#"{{
  "asset": {{ "version": "2.0" }},
  "buffers": [
    {{ "uri": "hero.bin", "byteLength": {byte_length} }}
  ],
  "bufferViews": [
    {{ "buffer": 0, "byteOffset": {times_offset}, "byteLength": 8 }},
    {{ "buffer": 0, "byteOffset": {root_translation_offset}, "byteLength": 24 }},
    {{ "buffer": 0, "byteOffset": {hand_translation_offset}, "byteLength": 24 }}
  ],
  "accessors": [
    {{
      "bufferView": 0,
      "componentType": 5126,
      "count": 2,
      "type": "SCALAR",
      "min": [0.0],
      "max": [1.0]
    }},
    {{
      "bufferView": 1,
      "componentType": 5126,
      "count": 2,
      "type": "VEC3"
    }},
    {{
      "bufferView": 2,
      "componentType": 5126,
      "count": 2,
      "type": "VEC3"
    }}
  ],
  "nodes": [
    {{
      "name": "Root",
      "children": [1],
      "translation": [0.0, 0.0, 0.0]
    }},
    {{
      "name": "Hand",
      "translation": [0.2, 0.8, 0.0]
    }}
  ],
  "skins": [
    {{
      "name": "HeroRig",
      "joints": [0, 1],
      "skeleton": 0
    }}
  ],
  "animations": [
    {{
      "name": "Idle",
      "samplers": [
        {{ "input": 0, "output": 1, "interpolation": "LINEAR" }},
        {{ "input": 0, "output": 2, "interpolation": "LINEAR" }}
      ],
      "channels": [
        {{ "sampler": 0, "target": {{ "node": 0, "path": "translation" }} }},
        {{ "sampler": 1, "target": {{ "node": 1, "path": "translation" }} }}
      ]
    }}
  ],
  "scenes": [
    {{ "nodes": [0] }}
  ],
  "scene": 0
}}"#,
                byte_length = fs::metadata(&buffer_path).unwrap().len(),
                times_offset = times_offset,
                root_translation_offset = root_translation_offset,
                hand_translation_offset = hand_translation_offset,
            ),
        )
        .unwrap();

        model_path
    }
}
